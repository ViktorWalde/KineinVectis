//! Ciclo de vida de um language server: spawn, handshake e thread leitora.
//!
//! O core sobe cada servidor como processo filho, faz o handshake
//! `initialize`/`initialized` e deixa uma thread lendo o stdout: respostas de
//! request voltam pelo mapa `pending`, `publishDiagnostics` vira evento
//! Kinein Vectis e requests servidor->cliente recebem `null` por enquanto.

use std::{
    collections::HashMap,
    io::{self, BufReader},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use serde_json::{Value, json};

use super::PendingResponses;
use super::diagnostics_merge::{self, MergedDiagnostics};
use super::framing::{read_message, write_locked_message};
use super::parse::published_diagnostics;
use super::registry::ServerSpec;
use super::types::LspError;
use crate::stderr_tail::{CAPACIDADE_PADRAO, StderrTail};
use kinein_protocol::JsonRpcRequest;

/// Estado de um servidor em execucao.
pub(super) struct ServerHandle {
    /// O processo. Num `Mutex` porque a thread do handshake o mata quando o
    /// `initialize` falha, e o manager o mata no `kill_server`.
    pub(super) child: Arc<Mutex<Child>>,
    /// `true` depois do `initialized` (Etapa 2 F6, 2026-09-18): ate' la' o
    /// handshake corre NUMA THREAD e o laco nao espera os 15 s do
    /// rust-analyzer — pedidos e sincronizacao antes disso voltam
    /// `LspError::Starting`, e a UI re-sincroniza ao ver `status: running`.
    pub(super) ready: Arc<AtomicBool>,
    /// O motivo, quando o handshake falhou (o manager remove o servidor).
    pub(super) failed: Arc<Mutex<Option<String>>>,
    pub(super) stdin: Arc<Mutex<ChildStdin>>,
    pub(super) versions: HashMap<String, i64>,
    /// Hash do ultimo conteudo sincronizado por URI. Evita `didChange`
    /// redundante quando o buffer nao mudou (cada request posicional
    /// re-sincroniza) — sem isso o clangd invalida seus fix-its a cada
    /// consulta, e todo hover/completion reenviava o documento inteiro.
    pub(super) content_hashes: HashMap<String, u64>,
    /// Legend de semantic tokens anunciada pelo servidor no `initialize`
    /// (chega pela thread do handshake).
    pub(super) semantic_token_types: Arc<Mutex<Vec<String>>>,
    /// Ultimo array cru de diagnostics publicado por URI, para compor o
    /// `context` de `textDocument/codeAction` sem depender da UI.
    pub(super) diagnostics_by_uri: Arc<Mutex<HashMap<String, Value>>>,
}

impl std::fmt::Debug for ServerHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServerHandle")
            .field("open_documents", &self.versions.len())
            .finish_non_exhaustive()
    }
}

/// Sobe o servidor de `spec`, faz o handshake e inicia a thread leitora.
///
/// `merged` e' o cache que FUNDE os diagnosticos de todos os servidores de uma
/// linguagem por arquivo: a thread leitora deste servidor grava a parte dele e
/// emite a uniao (ver [`super::diagnostics_merge`]).
pub(super) fn spawn_server(
    spec: &ServerSpec,
    root: &Path,
    events: super::EventSender,
    pending: PendingResponses,
    merged: MergedDiagnostics,
) -> Result<ServerHandle, LspError> {
    let mut command = Command::new(&spec.command);
    command.args(&spec.args).current_dir(root);
    // clangd usa a compilation database gerada pelo cmake.configure quando ela
    // existe (fatia M2.2). O `--compile-commands-dir` e' necessario porque
    // `<root>/.kinein/build` NAO e' `$SRC/build/`: o clangd procura sozinho nos
    // diretorios pai e em subdiretorios `build/`, mas nao dentro de `.kinein`
    // (https://clangd.llvm.org/installation).
    //
    // CORRECAO de 2026-08-30: este comentario dizia "servidor ja em execucao nao
    // recarrega flags". Meio errado — o clangd TEM hot-reload da CDB desde a v12
    // (reconfere a cada ~5s, https://reviews.llvm.org/D92663). O que nao
    // atualiza e' o DOCUMENTO ja aberto, que fica com a compilacao em cache.
    // A acao certa apos um configure e' reabrir os documentos abertos; reiniciar
    // o servidor joga o indice fora.
    if spec.language == "cpp" {
        let compile_commands = crate::cmake::build_dir(root).join("compile_commands.json");
        if compile_commands.is_file() {
            command.arg(format!(
                "--compile-commands-dir={}",
                crate::cmake::build_dir(root).display()
            ));
        }
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                LspError::MissingServer {
                    command: spec.command.clone(),
                }
            } else {
                LspError::ServerFailed {
                    command: spec.command.clone(),
                    message: error.to_string(),
                }
            }
        })?;

    let Some(stdin) = child.stdin.take() else {
        drop(child.kill());
        return Err(LspError::ServerFailed {
            command: spec.command.clone(),
            message: "stdin indisponivel".to_owned(),
        });
    };
    let Some(stdout) = child.stdout.take() else {
        drop(child.kill());
        return Err(LspError::ServerFailed {
            command: spec.command.clone(),
            message: "stdout indisponivel".to_owned(),
        });
    };
    let Some(stderr) = child.stderr.take() else {
        drop(child.kill());
        return Err(LspError::ServerFailed {
            command: spec.command.clone(),
            message: "stderr indisponivel".to_owned(),
        });
    };
    // O stderr do servidor (2026-09-17): cada linha vira `event.lsp.log` e a
    // cauda entra no `initialize` que falha e no `status: exited` (a thread
    // leitora fica com um clone). E' onde o clangd diz "compile_commands.json
    // not found" e o rust-analyzer conta o indice — antes ia para /dev/null.
    let stderr = StderrTail::spawn(
        stderr,
        CAPACIDADE_PADRAO,
        Some(log_collector(spec.key, &events)),
    );

    // O handshake corre numa THREAD (Etapa 2 F6): o `initialize` do
    // rust-analyzer leva segundos num projeto grande, e ate' 2026-09-18 o
    // laco do core esperava por ele — a IDE inteira muda. O handle volta ja',
    // marcado `starting`; a thread marca `ready` e emite `running`, ou emite
    // `failed` com o stderr e mata o filho.
    let stdin = Arc::new(Mutex::new(stdin));
    let initialize = super::handshake::initialize_request(root);
    if let Err(error) = write_locked_message(&stdin, &initialize) {
        drop(child.kill());
        drop(child.wait());
        return Err(LspError::ServerFailed {
            command: spec.command.clone(),
            message: format!("falha no initialize: {error}"),
        });
    }
    let child = Arc::new(Mutex::new(child));
    let ready = Arc::new(AtomicBool::new(false));
    let failed = Arc::new(Mutex::new(None));
    let semantic_token_types = Arc::new(Mutex::new(Vec::new()));
    let diagnostics_by_uri = Arc::new(Mutex::new(HashMap::new()));
    super::handshake::spawn_handshake_thread(super::handshake::HandshakeParts {
        spec: spec.clone(),
        stdout,
        stdin: Arc::clone(&stdin),
        child: Arc::clone(&child),
        ready: Arc::clone(&ready),
        failed: Arc::clone(&failed),
        legend: Arc::clone(&semantic_token_types),
        diagnostics: Arc::clone(&diagnostics_by_uri),
        events,
        pending,
        merged,
        stderr,
    });
    Ok(ServerHandle {
        child,
        ready,
        failed,
        stdin,
        versions: HashMap::new(),
        content_hashes: HashMap::new(),
        semantic_token_types,
        diagnostics_by_uri,
    })
}

impl ServerHandle {
    /// O handshake terminou e o servidor aceita pedidos.
    pub(super) fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    /// A legend de semantic tokens, vazia ate' o handshake terminar.
    pub(super) fn legend(&self) -> Vec<String> {
        self.semantic_token_types
            .lock()
            .map(|l| l.clone())
            .unwrap_or_default()
    }
}

/// Cada linha do stderr do servidor sai como `event.lsp.log { language,
/// line }` (protocolo 0.111.0): a aba IDE mostra; nada e' interpretado.
fn log_collector(key: &'static str, events: &super::EventSender) -> crate::stderr_tail::Coletor {
    let events = events.clone();
    Box::new(move |linha: &str| {
        drop(events.send(JsonRpcRequest::notification(
            "event.lsp.log",
            Some(json!({ "language": key, "line": linha })),
        )));
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_reader_thread(
    key: &'static str,
    mut reader: BufReader<ChildStdout>,
    stdin: Arc<Mutex<ChildStdin>>,
    events: super::EventSender,
    pending: PendingResponses,
    diagnostics_by_uri: Arc<Mutex<HashMap<String, Value>>>,
    settings: Arc<Value>,
    merged: MergedDiagnostics,
    stderr: StderrTail,
) {
    thread::spawn(move || {
        while let Ok(Some(message)) = read_message(&mut reader) {
            if route_response(&message, &pending) {
                continue;
            }
            answer_server_request(&message, &stdin, &settings);

            if message.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
            {
                if let Some(params) = message.get("params") {
                    cache_diagnostics(&diagnostics_by_uri, params);
                    // O que ESTE servidor publicou entra no cache fundido, e o
                    // evento que sai leva a uniao com os outros servidores da
                    // linguagem — a UI substitui por arquivo, e dois eventos
                    // parciais se apagariam um ao outro.
                    if let Some((path, diagnostics)) = published_diagnostics(params) {
                        let event = diagnostics_merge::record(&merged, key, &path, diagnostics);
                        if events.send(event).is_err() {
                            break;
                        }
                    }
                }
            }
        }

        // O servidor saiu: a cauda do stderr e' o motivo que a faixa de saude
        // mostra (campo `message`, ja' opcional no contrato).
        thread::sleep(Duration::from_millis(50));
        let cauda = stderr.tail();
        let mut params = json!({ "language": key, "status": "exited" });
        if !cauda.is_empty() {
            params["message"] = Value::String(cauda);
        }
        drop(events.send(JsonRpcRequest::notification(
            "event.lsp.status",
            Some(params),
        )));
    });
}

fn route_response(message: &Value, pending: &PendingResponses) -> bool {
    if message.get("method").is_some() {
        return false;
    }
    let Some(id) = message.get("id").and_then(Value::as_i64) else {
        return false;
    };
    let sender = pending
        .lock()
        .ok()
        .and_then(|mut responses| responses.remove(&id));
    let Some(sender) = sender else {
        return false;
    };
    drop(sender.send(message.clone()));
    true
}

/// Guarda o array cru de diagnostics do `publishDiagnostics` por URI.
fn cache_diagnostics(cache: &Arc<Mutex<HashMap<String, Value>>>, params: &Value) {
    let Some(uri) = params.get("uri").and_then(Value::as_str) else {
        return;
    };
    let diagnostics = params
        .get("diagnostics")
        .cloned()
        .unwrap_or_else(|| json!([]));
    if let Ok(mut entries) = cache.lock() {
        entries.insert(uri.to_owned(), diagnostics);
    }
}

/// Responde os requests servidor->cliente: `workspace/configuration` com as
/// secoes pedidas de `settings` (e' como o pyright pergunta `python` e
/// `basedpyright`); os demais, `null` — ainda nao suportados.
pub(super) fn answer_server_request(
    message: &Value,
    stdin: &Arc<Mutex<ChildStdin>>,
    settings: &Value,
) {
    let Some(id) = message.get("id") else {
        return;
    };
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return;
    };
    let result = if method == "workspace/configuration" {
        configuration_answer(message.get("params"), settings)
    } else {
        Value::Null
    };
    let reply = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    drop(write_locked_message(stdin, &reply));
}

/// Um valor por item pedido: a secao (com pontos: `python.analysis`) dentro
/// de `settings`, ou `null` quando ela nao existe. Sem `section`, o objeto
/// inteiro — e' o que a especificacao LSP 3.6 descreve.
fn configuration_answer(params: Option<&Value>, settings: &Value) -> Value {
    let itens = params
        .and_then(|p| p.get("items"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Value::Array(
        itens
            .iter()
            .map(|item| match item.get("section").and_then(Value::as_str) {
                Some(secao) if !secao.is_empty() => secao
                    .split('.')
                    .try_fold(settings, |atual, chave| atual.get(chave))
                    .cloned()
                    .unwrap_or(Value::Null),
                _ => settings.clone(),
            })
            .collect(),
    )
}
