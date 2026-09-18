//! O HANDSHAKE de um servidor de linguagem, fora do laco (Etapa 2 F6,
//! 2026-09-18): `initialize` -> `initialized` -> configuracao -> thread
//! leitora, numa thread propria. Saiu do `server.rs` quando ele bateu em
//! 500 ao ganhar isto. O `spawn_server` escreve o `initialize` e volta com o
//! handle marcado `starting`; aqui a thread espera a resposta (ate'
//! `INITIALIZE_TIMEOUT`), marca `ready` e emite `running` — ou `failed` com
//! o stderr, matando o filho.

use std::{
    collections::HashMap,
    io::BufReader,
    path::Path,
    process::{Child, ChildStdin, ChildStdout},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::PendingResponses;
use super::diagnostics_merge::MergedDiagnostics;
use super::framing::{read_message, write_locked_message};
use super::registry::ServerSpec;
use super::server::{answer_server_request, spawn_reader_thread};
use super::types::LspError;
use super::uri::uri_for_path;
use crate::stderr_tail::StderrTail;

/// Tempo maximo aguardando a resposta de `initialize` de um servidor.
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(15);

/// O que a thread do handshake leva.
pub(super) struct HandshakeParts {
    pub(super) spec: ServerSpec,
    pub(super) stdout: ChildStdout,
    pub(super) stdin: Arc<Mutex<ChildStdin>>,
    pub(super) child: Arc<Mutex<Child>>,
    pub(super) ready: Arc<AtomicBool>,
    pub(super) failed: Arc<Mutex<Option<String>>>,
    pub(super) legend: Arc<Mutex<Vec<String>>>,
    pub(super) diagnostics: Arc<Mutex<HashMap<String, Value>>>,
    pub(super) events: super::EventSender,
    pub(super) pending: PendingResponses,
    pub(super) merged: MergedDiagnostics,
    pub(super) stderr: StderrTail,
}

/// `initialize` -> `initialized` -> configuracao -> thread leitora, fora do
/// laco; no fim `ready` + `running`, ou `failed` com o stderr e o filho morto.
pub(super) fn spawn_handshake_thread(parts: HandshakeParts) {
    thread::spawn(move || {
        let HandshakeParts {
            spec,
            stdout,
            stdin,
            child,
            ready,
            failed,
            legend,
            diagnostics,
            events,
            pending,
            merged,
            stderr,
        } = parts;
        let mut reader = BufReader::new(stdout);
        match wait_for_initialize(&mut reader, &stdin, &spec.command)
            .and_then(|types| finish_handshake(&spec, &stdin).map(|()| types))
        {
            Ok(types) => {
                if let Ok(mut l) = legend.lock() {
                    *l = types;
                }
                spawn_reader_thread(
                    spec.key,
                    reader,
                    Arc::clone(&stdin),
                    events.clone(),
                    pending,
                    diagnostics,
                    Arc::new(spec.settings.clone()),
                    merged,
                    stderr,
                );
                ready.store(true, Ordering::SeqCst);
                emit_status(&events, spec.key, "running", None);
            }
            Err(error) => {
                let message = match error {
                    LspError::ServerFailed { message, .. } => {
                        stderr.anexar(&message, "stderr do servidor")
                    }
                    other => other.to_string(),
                };
                if let Ok(mut c) = child.lock() {
                    drop(c.kill());
                    drop(c.wait());
                }
                if let Ok(mut f) = failed.lock() {
                    *f = Some(message.clone());
                }
                emit_status(&events, spec.key, "failed", Some(&message));
            }
        }
    });
}

/// `event.lsp.status { language, status, message? }` a partir de uma thread.
fn emit_status(events: &super::EventSender, key: &str, status: &str, message: Option<&str>) {
    let mut params = json!({ "language": key, "status": status });
    if let (Some(map), Some(message)) = (params.as_object_mut(), message) {
        map.insert("message".to_owned(), Value::String(message.to_owned()));
    }
    drop(events.send(JsonRpcRequest::notification(
        "event.lsp.status",
        Some(params),
    )));
}

/// `initialized` + a configuracao, depois do `initialize` responder.
fn finish_handshake(spec: &ServerSpec, stdin: &Arc<Mutex<ChildStdin>>) -> Result<(), LspError> {
    let initialized = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
    write_locked_message(stdin, &initialized).map_err(|error| LspError::ServerFailed {
        command: spec.command.clone(),
        message: format!("falha no initialized: {error}"),
    })?;
    // A configuracao vai EMPURRADA logo depois do initialized (e' como o
    // pyright a le: `workspace/didChangeConfiguration` com `settings`), e
    // fica com a thread leitora para responder os `workspace/configuration`.
    if !spec.settings.is_null() {
        let configuration = json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": { "settings": spec.settings },
        });
        write_locked_message(stdin, &configuration).map_err(|error| LspError::ServerFailed {
            command: spec.command.clone(),
            message: format!("falha no didChangeConfiguration: {error}"),
        })?;
    }
    Ok(())
}

/// Monta o request `initialize` com as capabilities do cliente Kinein.
pub(super) fn initialize_request(root: &Path) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": Value::Null,
            "clientInfo": { "name": "kinein-vectis", "version": "0.1.0" },
            "rootUri": uri_for_path(root),
            "capabilities": {
                "textDocument": {
                    "publishDiagnostics": {},
                    "synchronization": { "didSave": true },
                    "completion": { "completionItem": { "snippetSupport": false } },
                    "definition": {},
                    "hover": {},
                    "references": {},
                    "rename": {},
                    "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    "codeAction": {
                        "codeActionLiteralSupport": {
                            "codeActionKind": {
                                "valueSet": [
                                    "", "quickfix", "refactor",
                                    "refactor.extract", "refactor.inline",
                                    "refactor.rewrite", "source",
                                    "source.organizeImports", "source.fixAll",
                                ],
                            },
                        },
                    },
                    "semanticTokens": {
                        "requests": { "full": true },
                        "tokenTypes": [
                            "namespace", "type", "class", "enum", "interface",
                            "struct", "typeParameter", "parameter", "variable",
                            "property", "enumMember", "event", "function",
                            "method", "macro", "keyword", "modifier", "comment",
                            "string", "number", "regexp", "operator", "decorator",
                        ],
                        "tokenModifiers": [],
                        "formats": ["relative"],
                    },
                },
                "workspace": { "symbol": {} },
            },
            "workspaceFolders": [{
                "uri": uri_for_path(root),
                "name": root.file_name().map_or_else(
                    || "workspace".to_owned(),
                    |name| name.to_string_lossy().into_owned(),
                ),
            }],
        }
    })
}

/// Aguarda a resposta do `initialize` e extrai a legend de semantic tokens.
fn wait_for_initialize(
    reader: &mut BufReader<ChildStdout>,
    stdin: &Arc<Mutex<ChildStdin>>,
    command: &str,
) -> Result<Vec<String>, LspError> {
    let deadline = Instant::now() + INITIALIZE_TIMEOUT;
    while Instant::now() < deadline {
        let message = read_message(reader).map_err(|error| LspError::ServerFailed {
            command: command.to_owned(),
            message: format!("erro lendo resposta de {command}: {error}"),
        })?;
        let Some(message) = message else {
            return Err(LspError::ServerFailed {
                command: command.to_owned(),
                message: format!("{command} encerrou durante o initialize"),
            });
        };
        if message.get("id").and_then(Value::as_i64) == Some(1) {
            if let Some(result) = message.get("result") {
                return Ok(semantic_token_legend(result));
            }
        }
        answer_server_request(&message, stdin, &Value::Null);
    }
    Err(LspError::ServerFailed {
        command: command.to_owned(),
        message: format!("{command} nao respondeu ao initialize a tempo"),
    })
}

/// Extrai `capabilities.semanticTokensProvider.legend.tokenTypes`.
fn semantic_token_legend(initialize_result: &Value) -> Vec<String> {
    initialize_result
        .pointer("/capabilities/semanticTokensProvider/legend/tokenTypes")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |types| {
            types
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::semantic_token_legend;

    #[test]
    fn semantic_token_legend_reads_token_types() {
        let result = json!({
            "capabilities": {
                "semanticTokensProvider": {
                    "legend": { "tokenTypes": ["variable", "function"] }
                }
            }
        });

        assert_eq!(semantic_token_legend(&result), ["variable", "function"]);
        assert!(semantic_token_legend(&json!({ "capabilities": {} })).is_empty());
    }
}
