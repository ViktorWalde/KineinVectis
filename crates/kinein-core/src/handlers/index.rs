//! Handler for `index.*` requests (`impl Core`) — o indice do projeto inteiro
//! (pilar 0 do `roadmaps/42`, decisao do autor em 2026-09-12).
//!
//! O indice mora num `Arc<Mutex<ProjectIndex>>` porque quem o CONSTROI e' um
//! job em thread propria e quem o CONSULTA e' o loop principal. E' a mesma
//! forma do `tool_registry`; um `Arc` compartilhado e' aceitavel aqui porque a
//! carga (milhares de simbolos) nao cabe num evento, e o `observe_notification`
//! so' ve eventos.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use kinein_protocol::{
    FileContext, IndexContextParams, IndexProgressEvent, IndexState, IndexStatusParams,
    IndexSymbolsParams, IndexSymbolsResult, JobRisk, JsonRpcRequest, JsonRpcResponse,
    ToolchainRole,
};
use serde_json::{Value, json};

use crate::Core;
use crate::index::context::{CompileContext, Ferramentas};
use crate::index::{self, ProjectIndex};
use crate::jobs::JobOutcome;
use crate::rpc::{no_workspace_response, parse_params};

/// Teto padrao de resultados de `index.symbols`.
const LIMITE_PADRAO: usize = 200;

impl Core {
    /// Roteia os metodos `index.*`.
    pub(crate) fn index_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "index.status" => Some(self.index_status_response(request_id, params)),
            "index.symbols" => Some(self.index_symbols_response(request_id, params)),
            "index.context" => Some(self.index_context_response(request_id, params)),
            _ => None,
        }
    }

    /// `index.status` — os totais e o estado. Responde tambem sem workspace
    /// (estado `idle`), porque a UI pergunta ao subir.
    fn index_status_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<IndexStatusParams>(
            request_id.as_ref(),
            params,
            "index.status nao aceita parametros",
        ) {
            return *response;
        }
        let stats = self
            .index
            .lock()
            .map_or_else(|_| ProjectIndex::default().stats(), |indice| indice.stats());
        JsonRpcResponse::success(request_id, json!(stats))
    }

    /// `index.symbols` — busca por nome no indice.
    fn index_symbols_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<IndexSymbolsParams>(
            request_id.as_ref(),
            params,
            "index.symbols requer o campo query e aceita limit e kind",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        if self.workspace_root().is_none() {
            return no_workspace_response(request_id, "index.symbols");
        }
        let Ok(indice) = self.index.lock() else {
            return JsonRpcResponse::success(
                request_id,
                json!(IndexSymbolsResult {
                    symbols: Vec::new(),
                    total: 0,
                    state: IndexState::Failed,
                }),
            );
        };
        let (symbols, total) = indice.query(
            &pedido.query,
            pedido.kind.as_deref(),
            pedido.limit.unwrap_or(LIMITE_PADRAO),
        );
        JsonRpcResponse::success(
            request_id,
            json!(IndexSymbolsResult {
                symbols,
                total,
                state: indice.state,
            }),
        )
    }

    /// `index.context` — como UM arquivo e' compilado/executado: a unidade da
    /// CDB (C/C++), o alvo do cargo (Rust), o interpretador (Python).
    fn index_context_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<IndexContextParams>(
            request_id.as_ref(),
            params,
            "index.context requer o campo path",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "index.context");
        };
        let caminho = Path::new(&pedido.path);
        let language = index::linguagem_de(caminho);
        let contexto = self.index.lock().ok().and_then(|indice| {
            indice
                .context
                .as_ref()
                .map(|c| c.for_file(&root, caminho, language))
        });
        let resposta = contexto.unwrap_or_else(|| FileContext {
            path: root.join(caminho).display().to_string(),
            language: language.to_owned(),
            unit: None,
            cargo: None,
            python: None,
            targets: Vec::new(),
            source: None,
            hint: Some("o contexto do projeto ainda esta' sendo carregado".to_owned()),
        });
        JsonRpcResponse::success(request_id, json!(resposta))
    }

    /// O que o contexto pode EXECUTAR nesta maquina: o cargo do kit efetivo,
    /// o poetry e o python3 do PATH, o `VIRTUAL_ENV` da sessao. Nos testes o
    /// PATH e' vazio e nada disto existe — e nada roda.
    fn index_tools(&self, root: &Path) -> Ferramentas {
        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        Ferramentas {
            cargo: toolchain
                .effective_program(ToolchainRole::Cargo)
                .map(|(_, path)| PathBuf::from(path)),
            poetry: self.detector.find_in_path("poetry"),
            python_sistema: self.detector.find_in_path("python3"),
            virtual_env: std::env::var("VIRTUAL_ENV").ok(),
            medir_versao: true,
        }
    }

    /// (Re)constroi o indice para `root`. Com jobs, em thread propria com
    /// progresso e cancelamento; sem jobs (testes, `run_json_lines`), aqui
    /// mesmo — o resultado e' o mesmo, so' o tempo de resposta muda. O
    /// contexto de compilador e' carregado logo depois dos arquivos, na mesma
    /// thread, e entra no mesmo `event.index.finished`.
    pub(crate) fn start_index_build(&self, root: &Path) {
        if let Ok(mut indice) = self.index.lock() {
            *indice = ProjectIndex::empty(root);
            indice.state = IndexState::Building;
        }
        let compartilhado = std::sync::Arc::clone(&self.index);
        let raiz = root.to_path_buf();
        let ferramentas = self.index_tools(root);
        let Some(jobs) = self.jobs.as_ref() else {
            let parado = std::sync::atomic::AtomicBool::new(false);
            let mut construido = index::build(&raiz, &parado, &|_, _| {});
            construido.context = Some(CompileContext::load(&raiz, &ferramentas));
            if let Ok(mut indice) = compartilhado.lock() {
                *indice = construido;
            }
            // Sem jobs nao ha' eventos, e sem eventos nao ha' watcher (os dois
            // nascem no enable_lsp): nada a registrar aqui.
            return;
        };
        let titulo = format!("Indexar {}", raiz.display());
        jobs.spawn("index", titulo, JobRisk::Low, true, move |ctx| {
            let cancel = ctx.cancellation();
            let progresso = |files: u64, symbols: u64| {
                ctx.emit_event(
                    "event.index.progress",
                    json!(IndexProgressEvent { files, symbols }),
                );
            };
            let mut construido = index::build(&raiz, &cancel, &progresso);
            if !cancel.load(Ordering::SeqCst) {
                construido.context = Some(CompileContext::load(&raiz, &ferramentas));
            }
            let stats = construido.stats();
            let ok = matches!(construido.state, IndexState::Ready);
            if let Ok(mut indice) = compartilhado.lock() {
                *indice = construido;
            }
            ctx.emit_event("event.index.finished", json!(stats));
            if cancel.load(Ordering::SeqCst) {
                JobOutcome::Failed
            } else if ok {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });
    }

    /// Recarrega SO' o contexto de compilador (a CDB depois de um configure,
    /// os alvos do cargo depois de um `Cargo.toml` salvo). Os arquivos ficam;
    /// o `event.index.finished` sai de novo com o resumo novo.
    pub(crate) fn reload_index_context(&self) {
        let Some(root) = self.workspace_root() else {
            return;
        };
        let pronto = self
            .index
            .lock()
            .is_ok_and(|indice| matches!(indice.state, IndexState::Ready));
        if !pronto {
            return;
        }
        let compartilhado = std::sync::Arc::clone(&self.index);
        let ferramentas = self.index_tools(&root);
        let trocar = move |contexto: CompileContext| {
            let Ok(mut indice) = compartilhado.lock() else {
                return None;
            };
            indice.context = Some(contexto);
            Some(indice.stats())
        };
        let Some(jobs) = self.jobs.as_ref() else {
            let stats = trocar(CompileContext::load(&root, &ferramentas));
            if let (Some(events), Some(stats)) = (self.events.as_ref(), stats) {
                drop(events.send(JsonRpcRequest::notification(
                    "event.index.finished",
                    Some(json!(stats)),
                )));
            }
            return;
        };
        jobs.spawn(
            "index",
            "Reler o contexto de compilador".to_owned(),
            JobRisk::Low,
            false,
            move |ctx| {
                let Some(stats) = trocar(CompileContext::load(&root, &ferramentas)) else {
                    return JobOutcome::Failed;
                };
                ctx.emit_event("event.index.finished", json!(stats));
                JobOutcome::Success
            },
        );
    }

    /// O incremento: os caminhos de um `event.fs.changed` sao reindexados no
    /// loop principal (um arquivo e' milissegundos; uma pasta nova e'
    /// caminhada inteira) e os totais reemitidos — e o `event.index.finished`
    /// que sai daqui e' o que registra a pasta nova no watcher, pelo mesmo
    /// caminho do fim do build (`observe_notification`). Um `Cargo.toml`
    /// salvo tambem recarrega o contexto: um alvo novo muda a que pacote cada
    /// arquivo pertence.
    pub(crate) fn reindex_changed_paths(&mut self, paths: &[PathBuf]) {
        let mudou = match self.index.lock() {
            Ok(mut indice) if matches!(indice.state, IndexState::Ready) => {
                indice.reindex_paths(paths, &mut self.extractor)
            }
            _ => 0,
        };
        let manifesto_mudou = paths
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "Cargo.toml"));
        if manifesto_mudou {
            self.reload_index_context();
        }
        if mudou == 0 {
            return;
        }
        let (Some(events), Ok(indice)) = (self.events.as_ref(), self.index.lock()) else {
            return;
        };
        drop(events.send(JsonRpcRequest::notification(
            "event.index.finished",
            Some(json!(indice.stats())),
        )));
    }

    /// Registra no watcher TODAS as pastas que o indice caminhou — uma a uma,
    /// nao recursivo, com a mesma lista de pastas ignoradas (ADR-0001). Antes
    /// disto o watcher so' via as pastas que a UI abriu, e um arquivo criado
    /// pelo terminal numa pasta fechada ficava fora do indice ate' o proximo
    /// `workspace.open` — em silencio. Idempotente: o watcher ignora pasta ja'
    /// registrada. Para no PRIMEIRO erro (o limite de inotify e' o caso real),
    /// e o `watch_workspace_directory` ja' o relata uma vez.
    pub(crate) fn watch_index_folders(&mut self) {
        let pastas: Vec<PathBuf> = self
            .index
            .lock()
            .map(|indice| indice.folder_paths.clone())
            .unwrap_or_default();
        for pasta in pastas {
            if !self.watch_workspace_directory(&pasta) {
                break;
            }
        }
    }
}
