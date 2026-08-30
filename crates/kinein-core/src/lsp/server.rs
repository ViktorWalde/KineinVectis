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
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::PendingResponses;
use super::framing::{read_message, write_locked_message};
use super::parse::diagnostics_event;
use super::types::LspError;
use super::uri::uri_for_path;
use kinein_protocol::JsonRpcRequest;

/// Tempo maximo aguardando a resposta de `initialize` de um servidor.
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(15);

/// Descricao estatica de um language server suportado.
#[derive(Debug, Clone, Copy)]
pub(super) struct ServerSpec {
    pub(super) language: &'static str,
    pub(super) command: &'static str,
    pub(super) args: &'static [&'static str],
    pub(super) language_id: &'static str,
}

const SERVERS: &[ServerSpec] = &[
    ServerSpec {
        language: "cpp",
        command: "clangd",
        args: &["--background-index"],
        language_id: "cpp",
    },
    ServerSpec {
        language: "rust",
        command: "rust-analyzer",
        args: &[],
        language_id: "rust",
    },
];

/// Seleciona o servidor pela extensao do arquivo, quando houver.
pub(super) fn spec_for_path(path: &Path) -> Option<&'static ServerSpec> {
    let suffix = path.extension()?.to_str()?.to_lowercase();
    let language = match suffix.as_str() {
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "ipp" => "cpp",
        "rs" => "rust",
        _ => return None,
    };
    SERVERS.iter().find(|spec| spec.language == language)
}

/// Estado de um servidor em execucao.
pub(super) struct ServerHandle {
    pub(super) child: Child,
    pub(super) stdin: Arc<Mutex<ChildStdin>>,
    pub(super) versions: HashMap<String, i64>,
    /// Hash do ultimo conteudo sincronizado por URI. Evita `didChange`
    /// redundante quando o buffer nao mudou (cada request posicional
    /// re-sincroniza) — sem isso o clangd invalida seus fix-its a cada
    /// consulta, e todo hover/completion reenviava o documento inteiro.
    pub(super) content_hashes: HashMap<String, u64>,
    /// Legend de semantic tokens anunciada pelo servidor no `initialize`.
    pub(super) semantic_token_types: Vec<String>,
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
pub(super) fn spawn_server(
    spec: &'static ServerSpec,
    root: &Path,
    events: super::EventSender,
    pending: PendingResponses,
) -> Result<ServerHandle, LspError> {
    let mut command = Command::new(spec.command);
    command.args(spec.args).current_dir(root);
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
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                LspError::MissingServer {
                    command: spec.command,
                }
            } else {
                LspError::ServerFailed {
                    command: spec.command,
                    message: error.to_string(),
                }
            }
        })?;

    let Some(stdin) = child.stdin.take() else {
        drop(child.kill());
        return Err(LspError::ServerFailed {
            command: spec.command,
            message: "stdin indisponivel".to_owned(),
        });
    };
    let Some(stdout) = child.stdout.take() else {
        drop(child.kill());
        return Err(LspError::ServerFailed {
            command: spec.command,
            message: "stdout indisponivel".to_owned(),
        });
    };

    let stdin = Arc::new(Mutex::new(stdin));
    let mut reader = BufReader::new(stdout);

    let initialize = initialize_request(root);
    write_locked_message(&stdin, &initialize).map_err(|error| LspError::ServerFailed {
        command: spec.command,
        message: format!("falha no initialize: {error}"),
    })?;

    let semantic_token_types = wait_for_initialize(&mut reader, &stdin, spec.command)?;

    let initialized = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
    write_locked_message(&stdin, &initialized).map_err(|error| LspError::ServerFailed {
        command: spec.command,
        message: format!("falha no initialized: {error}"),
    })?;

    let diagnostics_by_uri = Arc::new(Mutex::new(HashMap::new()));
    spawn_reader_thread(
        spec,
        reader,
        Arc::clone(&stdin),
        events,
        pending,
        Arc::clone(&diagnostics_by_uri),
    );

    Ok(ServerHandle {
        child,
        stdin,
        versions: HashMap::new(),
        content_hashes: HashMap::new(),
        semantic_token_types,
        diagnostics_by_uri,
    })
}

/// Monta o request `initialize` com as capabilities do cliente Kinein.
fn initialize_request(root: &Path) -> Value {
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
    command: &'static str,
) -> Result<Vec<String>, LspError> {
    let deadline = Instant::now() + INITIALIZE_TIMEOUT;
    while Instant::now() < deadline {
        let message = read_message(reader).map_err(|error| LspError::ServerFailed {
            command,
            message: format!("erro lendo resposta de {command}: {error}"),
        })?;
        let Some(message) = message else {
            return Err(LspError::ServerFailed {
                command,
                message: format!("{command} encerrou durante o initialize"),
            });
        };
        if message.get("id").and_then(Value::as_i64) == Some(1) {
            if let Some(result) = message.get("result") {
                return Ok(semantic_token_legend(result));
            }
        }
        answer_server_request(&message, stdin);
    }
    Err(LspError::ServerFailed {
        command,
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

fn spawn_reader_thread(
    spec: &'static ServerSpec,
    mut reader: BufReader<ChildStdout>,
    stdin: Arc<Mutex<ChildStdin>>,
    events: super::EventSender,
    pending: PendingResponses,
    diagnostics_by_uri: Arc<Mutex<HashMap<String, Value>>>,
) {
    thread::spawn(move || {
        while let Ok(Some(message)) = read_message(&mut reader) {
            if route_response(&message, &pending) {
                continue;
            }
            answer_server_request(&message, &stdin);

            if message.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
            {
                if let Some(params) = message.get("params") {
                    cache_diagnostics(&diagnostics_by_uri, params);
                    if let Some(event) = diagnostics_event(params) {
                        if events.send(event).is_err() {
                            break;
                        }
                    }
                }
            }
        }

        drop(events.send(JsonRpcRequest::notification(
            "event.lsp.status",
            Some(json!({ "language": spec.language, "status": "exited" })),
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

/// Responde `null` a requests servidor->cliente que nao suportamos ainda.
fn answer_server_request(message: &Value, stdin: &Arc<Mutex<ChildStdin>>) {
    let Some(id) = message.get("id") else {
        return;
    };
    if message.get("method").is_none() {
        return;
    }
    let reply = json!({ "jsonrpc": "2.0", "id": id, "result": Value::Null });
    drop(write_locked_message(stdin, &reply));
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::{semantic_token_legend, spec_for_path};

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

    #[test]
    fn unsupported_extensions_have_no_server() {
        assert!(spec_for_path(Path::new("/tmp/nota.txt")).is_none());
        assert!(spec_for_path(Path::new("/tmp/main.rs")).is_some());
        assert!(spec_for_path(Path::new("/tmp/app.cpp")).is_some());
    }
}
