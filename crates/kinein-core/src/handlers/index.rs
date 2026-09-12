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
    IndexProgressEvent, IndexState, IndexStatusParams, IndexSymbolsParams, IndexSymbolsResult,
    JobRisk, JsonRpcRequest, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
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

    /// (Re)constroi o indice para `root`. Com jobs, em thread propria com
    /// progresso e cancelamento; sem jobs (testes, `run_json_lines`), aqui
    /// mesmo — o resultado e' o mesmo, so' o tempo de resposta muda.
    pub(crate) fn start_index_build(&self, root: &Path) {
        if let Ok(mut indice) = self.index.lock() {
            *indice = ProjectIndex::empty(root);
            indice.state = IndexState::Building;
        }
        let compartilhado = std::sync::Arc::clone(&self.index);
        let raiz = root.to_path_buf();
        let Some(jobs) = self.jobs.as_ref() else {
            let parado = std::sync::atomic::AtomicBool::new(false);
            let construido = index::build(&raiz, &parado, &|_, _| {});
            if let Ok(mut indice) = compartilhado.lock() {
                *indice = construido;
            }
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
            let construido = index::build(&raiz, &cancel, &progresso);
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

    /// O incremento: os caminhos de um `event.fs.changed` sao reindexados no
    /// loop principal (um arquivo e' milissegundos) e os totais reemitidos.
    pub(crate) fn reindex_changed_paths(&mut self, paths: &[PathBuf]) {
        let mudou = match self.index.lock() {
            Ok(mut indice) if matches!(indice.state, IndexState::Ready) => {
                indice.reindex_paths(paths, &mut self.extractor)
            }
            _ => 0,
        };
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
}
