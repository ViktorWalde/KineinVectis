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

/// Descricao de um language server suportado.
///
/// `language` e `language_id` sao vocabulario FECHADO do projeto (hoje `cpp` e
/// `rust`) e continuam `&'static`: sao chave de mapa e valor de protocolo. O
/// que muda em tempo de execucao e o EXECUTAVEL — por isso `command` e `args`
/// sao proprios. Ver [`ServerRegistry::set_command`].
#[derive(Debug, Clone)]
pub(super) struct ServerSpec {
    pub(super) language: &'static str,
    pub(super) command: String,
    pub(super) args: Vec<String>,
    pub(super) language_id: &'static str,
    /// Configuracao que o servidor le como `settings` (secoes por nome):
    /// vai num `workspace/didChangeConfiguration` logo apos o `initialized`
    /// e responde os `workspace/configuration` que ele pedir. `Null` = nada.
    /// E' assim que o basedpyright recebe `python.pythonPath` — o
    /// interpretador do projeto (2026-09-12).
    pub(super) settings: Value,
}

/// Tabela de servidores por linguagem.
///
/// Existe como ESTADO do manager, e nao como `const`, por um motivo medido: ate
/// 2026-09-02 nenhum teste deste repositorio conseguia observar o que o core
/// FALA com um language server, porque o executavel estava fixado no binario.
/// Com a tabela injetavel, um servidor FALSO entra no lugar e o `didOpen`/
/// `didChange`/`didClose` passam a ser verificaveis (roadmap 30, etapa 3).
///
/// A mesma costura serve ao produto na etapa 5 (toolchain como entidade): um
/// clangd fora do `PATH` e a mesma pergunta — "qual executavel roda esta
/// linguagem?".
#[derive(Debug, Clone)]
pub(super) struct ServerRegistry {
    specs: Vec<ServerSpec>,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self {
            specs: vec![
                ServerSpec {
                    language: "cpp",
                    command: "clangd".to_owned(),
                    args: vec!["--background-index".to_owned()],
                    language_id: "cpp",
                    settings: Value::Null,
                },
                ServerSpec {
                    language: "rust",
                    command: "rust-analyzer".to_owned(),
                    args: Vec::new(),
                    language_id: "rust",
                    settings: Value::Null,
                },
                // Python (cadeia do roadmaps/41 bloco B, fatia 2, 2026-09-12):
                // basedpyright pelo PyPI, `--stdio`; o interpretador do projeto
                // chega em `settings` pelo Core (`configure_python_lsp`).
                ServerSpec {
                    language: "python",
                    command: "basedpyright-langserver".to_owned(),
                    args: vec!["--stdio".to_owned()],
                    language_id: "python",
                    settings: Value::Null,
                },
            ],
        }
    }
}

impl ServerRegistry {
    /// Seleciona o servidor pela extensao do arquivo, quando houver.
    ///
    /// Devolve uma COPIA de proposito: o chamador precisa do spec e do `&mut
    /// self` ao mesmo tempo (subir o servidor), e o spec tem duas strings.
    pub(super) fn spec_for_path(&self, path: &Path) -> Option<ServerSpec> {
        let language = language_for_path(path)?;
        self.specs
            .iter()
            .find(|spec| spec.language == language)
            .cloned()
    }

    /// Troca o executavel de uma linguagem conhecida; `false` se ela nao existe.
    pub(super) fn set_command(&mut self, language: &str, command: &str, args: &[&str]) -> bool {
        let Some(spec) = self.specs.iter_mut().find(|spec| spec.language == language) else {
            return false;
        };
        command.clone_into(&mut spec.command);
        spec.args = args.iter().map(|argument| (*argument).to_owned()).collect();
        true
    }

    /// Troca a configuracao que uma linguagem recebe; vale na PROXIMA subida.
    pub(super) fn set_settings(&mut self, language: &str, settings: Value) -> bool {
        let Some(spec) = self.specs.iter_mut().find(|spec| spec.language == language) else {
            return false;
        };
        spec.settings = settings;
        true
    }
}

/// Linguagem do arquivo pela extensao, quando ha uma suportada.
pub(super) fn language_for_path(path: &Path) -> Option<&'static str> {
    let suffix = path.extension()?.to_str()?.to_lowercase();
    match suffix.as_str() {
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "ipp" => Some("cpp"),
        "rs" => Some("rust"),
        "py" | "pyi" | "pyw" => Some("python"),
        _ => None,
    }
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
    spec: &ServerSpec,
    root: &Path,
    events: super::EventSender,
    pending: PendingResponses,
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
        .stderr(Stdio::null())
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

    let stdin = Arc::new(Mutex::new(stdin));
    let mut reader = BufReader::new(stdout);

    let initialize = initialize_request(root);
    write_locked_message(&stdin, &initialize).map_err(|error| LspError::ServerFailed {
        command: spec.command.clone(),
        message: format!("falha no initialize: {error}"),
    })?;

    let semantic_token_types = wait_for_initialize(&mut reader, &stdin, &spec.command)?;

    let initialized = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
    write_locked_message(&stdin, &initialized).map_err(|error| LspError::ServerFailed {
        command: spec.command.clone(),
        message: format!("falha no initialized: {error}"),
    })?;
    // A configuracao vai EMPURRADA logo depois do initialized (e' como o
    // pyright a le: `workspace/didChangeConfiguration` com `settings`), e
    // fica com a thread leitora para responder os `workspace/configuration`.
    let settings = Arc::new(spec.settings.clone());
    if !settings.is_null() {
        let configuration = json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": { "settings": *settings },
        });
        write_locked_message(&stdin, &configuration).map_err(|error| LspError::ServerFailed {
            command: spec.command.clone(),
            message: format!("falha no didChangeConfiguration: {error}"),
        })?;
    }

    let diagnostics_by_uri = Arc::new(Mutex::new(HashMap::new()));
    spawn_reader_thread(
        spec.language,
        reader,
        Arc::clone(&stdin),
        events,
        pending,
        Arc::clone(&diagnostics_by_uri),
        settings,
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

fn spawn_reader_thread(
    language: &'static str,
    mut reader: BufReader<ChildStdout>,
    stdin: Arc<Mutex<ChildStdin>>,
    events: super::EventSender,
    pending: PendingResponses,
    diagnostics_by_uri: Arc<Mutex<HashMap<String, Value>>>,
    settings: Arc<Value>,
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
            Some(json!({ "language": language, "status": "exited" })),
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
fn answer_server_request(message: &Value, stdin: &Arc<Mutex<ChildStdin>>, settings: &Value) {
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::{ServerRegistry, semantic_token_legend};

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
        let registry = ServerRegistry::default();
        assert!(registry.spec_for_path(Path::new("/tmp/nota.txt")).is_none());
        assert!(registry.spec_for_path(Path::new("/tmp/main.rs")).is_some());
        assert!(registry.spec_for_path(Path::new("/tmp/app.cpp")).is_some());
    }

    /// A troca de executavel e o que torna o LSP testavel (roadmap 30 §3).
    #[test]
    fn the_command_of_a_known_language_can_be_replaced() {
        let mut registry = ServerRegistry::default();
        assert!(registry.set_command("rust", "/tmp/falso", &["--x"]));
        assert!(!registry.set_command("cobol", "/tmp/falso", &[]));

        let spec = registry.spec_for_path(Path::new("/tmp/main.rs")).unwrap();
        assert_eq!(spec.command, "/tmp/falso");
        assert_eq!(spec.args, ["--x"]);
        assert_eq!(spec.language_id, "rust", "o vocabulario nao muda");
    }
}
