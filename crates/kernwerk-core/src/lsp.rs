//! LSP client manager: clangd, rust-analyzer e futuros servidores.
//!
//! O core sobe cada language server como processo filho, faz o handshake
//! `initialize`/`initialized`, sincroniza documentos (didOpen/didChange/
//! didSave com texto completo) e converte `textDocument/publishDiagnostics`
//! em notificacoes `event.lsp.diagnostics` do protocolo Kernwerk.
//!
//! A UI nunca fala LSP diretamente; ela envia `lsp.didChange` pelo IPC e
//! recebe diagnosticos ja traduzidos.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use kernwerk_protocol::{JsonRpcRequest, LspCompletionItem, LspSemanticToken};
use serde_json::{Value, json};

/// Sender usado para empurrar notificacoes assincronas ao loop principal.
pub type EventSender = Sender<JsonRpcRequest>;

type PendingResponses = Arc<Mutex<HashMap<i64, mpsc::Sender<Value>>>>;

/// Tempo maximo aguardando a resposta de `initialize` de um servidor.
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(15);

/// Tempo maximo aguardando respostas interativas do LSP.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(4);

/// Maximo de itens de completion repassados a UI por request.
const MAX_COMPLETION_ITEMS: usize = 50;

/// Maximo de referencias (find usages) repassadas a UI por request.
const MAX_REFERENCE_ITEMS: usize = 200;

/// Localizacao de codigo resolvida por um servidor LSP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LspLocation {
    /// Caminho absoluto do alvo.
    pub path: String,
    /// Linha 1-based.
    pub line: u64,
    /// Coluna 1-based.
    pub column: u64,
}

/// Um edit de texto LSP com posicoes 0-based em unidades UTF-16.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextSpanEdit {
    /// Linha inicial (0-based).
    pub start_line: u64,
    /// Coluna inicial (0-based, UTF-16).
    pub start_character: u64,
    /// Linha final (0-based).
    pub end_line: u64,
    /// Coluna final (0-based, UTF-16).
    pub end_character: u64,
    /// Texto que substitui o range.
    pub new_text: String,
}

/// Edits de rename agrupados por arquivo.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileEdits {
    /// Caminho absoluto do arquivo alvo.
    pub path: String,
    /// Edits a aplicar, na ordem enviada pelo servidor.
    pub edits: Vec<TextSpanEdit>,
}

/// Plano de aplicacao de um `WorkspaceEdit` de rename.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct WorkspaceEditPlan {
    /// Arquivos afetados; vazio quando o servidor nao encontrou o simbolo.
    pub files: Vec<FileEdits>,
}

impl WorkspaceEditPlan {
    /// Total de edits em todos os arquivos do plano.
    #[must_use]
    pub fn edit_count(&self) -> u64 {
        let total: usize = self.files.iter().map(|file| file.edits.len()).sum();
        u64::try_from(total).unwrap_or(u64::MAX)
    }
}

/// Erro produzido pelo cliente LSP gerenciado.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LspError {
    /// A extensao do arquivo ainda nao tem servidor configurado.
    UnsupportedFile {
        /// Caminho solicitado.
        path: String,
    },
    /// O executavel do language server nao foi encontrado.
    MissingServer {
        /// Comando esperado.
        command: &'static str,
    },
    /// Falha ao iniciar ou inicializar o servidor.
    ServerFailed {
        /// Comando do servidor.
        command: &'static str,
        /// Mensagem tecnica.
        message: String,
    },
    /// O servidor nao respondeu a tempo.
    Timeout {
        /// Metodo LSP solicitado.
        method: &'static str,
    },
    /// O servidor respondeu com erro LSP.
    RequestFailed {
        /// Metodo LSP solicitado.
        method: &'static str,
        /// Mensagem do servidor.
        message: String,
    },
    /// Resposta inesperada ou canal interno quebrado.
    Transport {
        /// Mensagem tecnica.
        message: String,
    },
}

impl LspError {
    /// Retorna `true` quando o erro e causado por arquivo sem servidor.
    #[must_use]
    pub const fn is_invalid_params(&self) -> bool {
        matches!(self, Self::UnsupportedFile { .. })
    }

    /// Retorna `true` quando o binario do language server esta ausente.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingServer { .. })
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFile { path } => {
                write!(formatter, "nenhum servidor LSP suporta o arquivo {path}")
            }
            Self::MissingServer { command } => {
                write!(formatter, "{command} nao foi encontrado no PATH")
            }
            Self::ServerFailed { command, message } => {
                write!(formatter, "falha ao iniciar {command}: {message}")
            }
            Self::Timeout { method } => {
                write!(formatter, "{method} nao respondeu a tempo")
            }
            Self::RequestFailed { method, message } => {
                write!(formatter, "{method} falhou: {message}")
            }
            Self::Transport { message } => formatter.write_str(message),
        }
    }
}

impl Error for LspError {}

/// Descricao estatica de um language server suportado.
#[derive(Debug, Clone, Copy)]
struct ServerSpec {
    language: &'static str,
    command: &'static str,
    args: &'static [&'static str],
    language_id: &'static str,
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

fn spec_for_path(path: &Path) -> Option<&'static ServerSpec> {
    let suffix = path.extension()?.to_str()?.to_lowercase();
    let language = match suffix.as_str() {
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "ipp" => "cpp",
        "rs" => "rust",
        _ => return None,
    };
    SERVERS.iter().find(|spec| spec.language == language)
}

/// Estado de um servidor em execucao.
struct ServerHandle {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    versions: HashMap<String, i64>,
    /// Legend de semantic tokens anunciada pelo servidor no `initialize`.
    semantic_token_types: Vec<String>,
}

impl std::fmt::Debug for ServerHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ServerHandle")
            .field("open_documents", &self.versions.len())
            .finish_non_exhaustive()
    }
}

/// Gerencia os language servers do workspace aberto.
#[derive(Debug)]
pub struct LspManager {
    events: EventSender,
    servers: HashMap<&'static str, ServerHandle>,
    pending: PendingResponses,
    root: Option<PathBuf>,
    next_request_id: i64,
}

impl LspManager {
    /// Cria um manager que envia notificacoes pelo canal do loop principal.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            servers: HashMap::new(),
            pending: Arc::new(Mutex::new(HashMap::new())),
            root: None,
            next_request_id: 2,
        }
    }

    /// Define a raiz do workspace, derrubando servidores da raiz anterior.
    pub fn set_root(&mut self, root: Option<PathBuf>) {
        if self.root == root {
            return;
        }
        self.shutdown_all();
        self.root = root;
    }

    /// Encerra todos os servidores gerenciados.
    pub fn shutdown_all(&mut self) {
        let drained: Vec<(&'static str, ServerHandle)> = self.servers.drain().collect();
        for (language, mut handle) in drained {
            // Melhor esforco: mata o processo; shutdown educado fica para
            // quando houver cancelamento generico no core.
            drop(handle.child.kill());
            drop(handle.child.wait());
            self.emit_status(language, "stopped");
        }
    }

    /// Abre (ou re-sincroniza) um documento no servidor da linguagem dele.
    pub fn did_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        if self.ensure_server(spec).is_err() {
            return;
        }
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };

        if let Some(version) = handle.versions.get_mut(&uri) {
            *version += 1;
            let params = full_change_params(&uri, *version, content);
            send_notification(&handle.stdin, "textDocument/didChange", &params);
            return;
        }

        handle.versions.insert(uri.clone(), 1);
        let params = json!({
            "textDocument": {
                "uri": uri,
                "languageId": spec.language_id,
                "version": 1,
                "text": content,
            }
        });
        send_notification(&handle.stdin, "textDocument/didOpen", &params);
    }

    /// Sincroniza o conteudo atual de um documento (texto completo).
    pub fn did_change(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let has_document = self
            .servers
            .get(spec.language)
            .is_some_and(|handle| handle.versions.contains_key(&uri));
        if !has_document {
            self.did_open(path, content);
            return;
        }
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        *version += 1;
        let params = full_change_params(&uri, *version, content);
        send_notification(&handle.stdin, "textDocument/didChange", &params);
    }

    /// Notifica salvamento de um documento.
    pub fn did_save(&mut self, path: &Path, content: &str) {
        self.did_change(path, content);
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let Some(handle) = self.servers.get(spec.language) else {
            return;
        };
        let params = json!({
            "textDocument": { "uri": uri_for_path(path) },
            "text": content,
        });
        send_notification(&handle.stdin, "textDocument/didSave", &params);
    }

    /// Resolve `textDocument/definition` para a posicao atual do editor.
    pub fn definition(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Option<LspLocation>, LspError> {
        let result = self.text_document_position_request(
            "textDocument/definition",
            path,
            content,
            line,
            column,
        )?;
        Ok(definition_location(&result))
    }

    /// Resolve `textDocument/hover` para a posicao atual do editor.
    pub fn hover(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Option<String>, LspError> {
        let result =
            self.text_document_position_request("textDocument/hover", path, content, line, column)?;
        Ok(hover_content(&result))
    }

    /// Resolve `textDocument/completion` para a posicao atual do editor.
    ///
    /// Os itens voltam ordenados por `sortText` e limitados a
    /// [`MAX_COMPLETION_ITEMS`].
    pub fn completion(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Vec<LspCompletionItem>, LspError> {
        let result = self.text_document_position_request(
            "textDocument/completion",
            path,
            content,
            line,
            column,
        )?;
        Ok(completion_items(&result))
    }

    /// Resolve `textDocument/references` (find usages) para a posicao atual.
    pub fn references(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Vec<LspLocation>, LspError> {
        let result = self.position_request_with(
            "textDocument/references",
            path,
            content,
            line,
            column,
            &json!({ "context": { "includeDeclaration": true } }),
        )?;
        Ok(reference_locations(&result))
    }

    /// Solicita `textDocument/rename` e devolve o plano de edits por arquivo.
    ///
    /// O manager NAO escreve arquivos; quem aplica o plano (confinado ao
    /// workspace) e o chamador.
    pub fn rename(
        &mut self,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
        new_name: &str,
    ) -> Result<WorkspaceEditPlan, LspError> {
        let result = self.position_request_with(
            "textDocument/rename",
            path,
            content,
            line,
            column,
            &json!({ "newName": new_name }),
        )?;
        workspace_edit_plan(&result)
    }

    /// Resolve `textDocument/semanticTokens/full` para o documento inteiro.
    ///
    /// Retorna tokens absolutos ja decodificados com a legend do servidor;
    /// vazio quando o servidor nao suporta semantic tokens.
    pub fn semantic_tokens(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<Vec<LspSemanticToken>, LspError> {
        let spec = self.sync_document(path, content)?;
        let legend = self
            .servers
            .get(spec.language)
            .map(|handle| handle.semantic_token_types.clone())
            .unwrap_or_default();
        if legend.is_empty() {
            return Ok(Vec::new());
        }
        let params = json!({ "textDocument": { "uri": uri_for_path(path) } });
        let result =
            self.send_request(spec.language, "textDocument/semanticTokens/full", &params)?;
        Ok(decode_semantic_tokens(&result, &legend))
    }

    /// Re-sincroniza um documento que ja esta aberto no servidor.
    ///
    /// Usado depois de um rename reescrever arquivos no disco; documentos que
    /// o servidor nao conhece ficam intactos (nenhum `didOpen` novo).
    pub fn sync_if_open(&mut self, path: &Path, content: &str) {
        let Some(spec) = spec_for_path(path) else {
            return;
        };
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return;
        };
        let Some(version) = handle.versions.get_mut(&uri) else {
            return;
        };
        *version += 1;
        let params = full_change_params(&uri, *version, content);
        send_notification(&handle.stdin, "textDocument/didChange", &params);
    }

    fn text_document_position_request(
        &mut self,
        method: &'static str,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
    ) -> Result<Value, LspError> {
        self.position_request_with(method, path, content, line, column, &Value::Null)
    }

    fn position_request_with(
        &mut self,
        method: &'static str,
        path: &Path,
        content: &str,
        line: u64,
        column: u64,
        extra: &Value,
    ) -> Result<Value, LspError> {
        let spec = self.sync_document(path, content)?;
        let mut params = json!({
            "textDocument": { "uri": uri_for_path(path) },
            "position": {
                "line": line.saturating_sub(1),
                "character": column.saturating_sub(1),
            },
        });
        if let (Some(target), Some(fields)) = (params.as_object_mut(), extra.as_object()) {
            for (key, value) in fields {
                target.insert(key.clone(), value.clone());
            }
        }
        self.send_request(spec.language, method, &params)
    }

    fn sync_document(
        &mut self,
        path: &Path,
        content: &str,
    ) -> Result<&'static ServerSpec, LspError> {
        let Some(spec) = spec_for_path(path) else {
            return Err(LspError::UnsupportedFile {
                path: path.display().to_string(),
            });
        };
        self.ensure_server(spec)?;
        let uri = uri_for_path(path);
        let Some(handle) = self.servers.get_mut(spec.language) else {
            return Err(LspError::Transport {
                message: format!("servidor {} nao esta registrado", spec.language),
            });
        };

        if let Some(version) = handle.versions.get_mut(&uri) {
            *version += 1;
            let params = full_change_params(&uri, *version, content);
            send_notification(&handle.stdin, "textDocument/didChange", &params);
        } else {
            handle.versions.insert(uri.clone(), 1);
            let params = json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": spec.language_id,
                    "version": 1,
                    "text": content,
                }
            });
            send_notification(&handle.stdin, "textDocument/didOpen", &params);
        }

        Ok(spec)
    }

    fn ensure_server(&mut self, spec: &'static ServerSpec) -> Result<(), LspError> {
        if self.servers.contains_key(spec.language) {
            return Ok(());
        }
        let Some(root) = self.root.clone() else {
            return Err(LspError::Transport {
                message: "nenhum workspace LSP configurado".to_owned(),
            });
        };

        match spawn_server(spec, &root, self.events.clone(), Arc::clone(&self.pending)) {
            Ok(handle) => {
                self.servers.insert(spec.language, handle);
                self.emit_status(spec.language, "running");
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                self.emit_status_message(spec.language, "failed", Some(&message));
                Err(error)
            }
        }
    }

    fn send_request(
        &mut self,
        language: &'static str,
        method: &'static str,
        params: &Value,
    ) -> Result<Value, LspError> {
        let id = self.next_request_id;
        self.next_request_id += 1;
        let Some(handle) = self.servers.get(language) else {
            return Err(LspError::Transport {
                message: format!("servidor {language} nao esta em execucao"),
            });
        };
        let (response_tx, response_rx) = mpsc::channel::<Value>();
        {
            let Ok(mut pending) = self.pending.lock() else {
                return Err(LspError::Transport {
                    message: "mapa de requests LSP envenenado".to_owned(),
                });
            };
            pending.insert(id, response_tx);
        }

        let message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if let Err(error) = write_locked_message(&handle.stdin, &message) {
            self.remove_pending(id);
            return Err(LspError::Transport {
                message: format!("falha ao enviar {method}: {error}"),
            });
        }

        match response_rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(response) => response_result(method, &response),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.remove_pending(id);
                Err(LspError::Timeout { method })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(LspError::Transport {
                message: format!("canal de resposta de {method} foi fechado"),
            }),
        }
    }

    fn remove_pending(&self, id: i64) {
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(&id);
        }
    }

    fn emit_status(&self, language: &str, status: &str) {
        self.emit_status_message(language, status, None);
    }

    fn emit_status_message(&self, language: &str, status: &str, message: Option<&str>) {
        let mut params = json!({ "language": language, "status": status });
        if let (Some(map), Some(text)) = (params.as_object_mut(), message) {
            map.insert("message".to_owned(), Value::String(text.to_owned()));
        }
        drop(self.events.send(JsonRpcRequest::notification(
            "event.lsp.status",
            Some(params),
        )));
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

fn full_change_params(uri: &str, version: i64, content: &str) -> Value {
    json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": content }],
    })
}

fn spawn_server(
    spec: &'static ServerSpec,
    root: &Path,
    events: EventSender,
    pending: PendingResponses,
) -> Result<ServerHandle, LspError> {
    let mut child = Command::new(spec.command)
        .args(spec.args)
        .current_dir(root)
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

    let initialize = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": Value::Null,
            "clientInfo": { "name": "kernwerk-studio", "version": "0.1.0" },
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
                "workspace": {},
            },
            "workspaceFolders": [{
                "uri": uri_for_path(root),
                "name": root.file_name().map_or_else(
                    || "workspace".to_owned(),
                    |name| name.to_string_lossy().into_owned(),
                ),
            }],
        }
    });
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

    spawn_reader_thread(spec, reader, Arc::clone(&stdin), events, pending);

    Ok(ServerHandle {
        child,
        stdin,
        versions: HashMap::new(),
        semantic_token_types,
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
    events: EventSender,
    pending: PendingResponses,
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

fn response_result(method: &'static str, response: &Value) -> Result<Value, LspError> {
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("erro LSP sem mensagem")
            .to_owned();
        return Err(LspError::RequestFailed { method, message });
    }
    Ok(response.get("result").cloned().unwrap_or(Value::Null))
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

fn send_notification(stdin: &Arc<Mutex<ChildStdin>>, method: &str, params: &Value) {
    let message = json!({ "jsonrpc": "2.0", "method": method, "params": params });
    drop(write_locked_message(stdin, &message));
}

fn write_locked_message(stdin: &Arc<Mutex<ChildStdin>>, message: &Value) -> io::Result<()> {
    let Ok(mut guard) = stdin.lock() else {
        return Err(io::Error::other("stdin do servidor envenenado"));
    };
    write_message(&mut *guard, message)
}

/// Escreve uma mensagem LSP com framing `Content-Length`.
fn write_message(writer: &mut impl Write, message: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(message).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

/// Le uma mensagem LSP com framing `Content-Length`. `None` significa EOF.
fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed
            .strip_prefix("Content-Length:")
            .map(str::trim)
            .and_then(|raw| raw.parse::<usize>().ok())
        {
            content_length = Some(value);
        }
    }

    let Some(length) = content_length else {
        return Err(io::Error::other("cabecalho Content-Length ausente"));
    };
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

/// Converte `textDocument/publishDiagnostics` no evento Kernwerk.
fn diagnostics_event(params: &Value) -> Option<JsonRpcRequest> {
    let uri = params.get("uri")?.as_str()?;
    let path = path_for_uri(uri)?;
    let empty = Vec::new();
    let raw_diagnostics = params
        .get("diagnostics")
        .and_then(Value::as_array)
        .unwrap_or(&empty);

    let diagnostics = raw_diagnostics
        .iter()
        .filter_map(|diagnostic| {
            let message = diagnostic.get("message")?.as_str()?.to_owned();
            let severity = match diagnostic.get("severity").and_then(Value::as_i64) {
                Some(1) | None => "error",
                Some(2) => "warning",
                _ => "note",
            };
            let start = diagnostic.get("range")?.get("start")?;
            let line = start.get("line").and_then(Value::as_u64).unwrap_or(0) + 1;
            let column = start.get("character").and_then(Value::as_u64).unwrap_or(0) + 1;
            Some(json!({
                "severity": severity,
                "message": message,
                "line": line,
                "column": column,
            }))
        })
        .collect::<Vec<_>>();

    Some(JsonRpcRequest::notification(
        "event.lsp.diagnostics",
        Some(json!({ "path": path, "diagnostics": diagnostics })),
    ))
}

fn definition_location(result: &Value) -> Option<LspLocation> {
    if result.is_null() {
        return None;
    }
    if let Some(locations) = result.as_array() {
        return locations.iter().find_map(location_from_value);
    }
    location_from_value(result)
}

fn location_from_value(value: &Value) -> Option<LspLocation> {
    let (uri, range) = if let Some(uri) = value.get("uri").and_then(Value::as_str) {
        (uri, value.get("range")?)
    } else {
        (
            value.get("targetUri").and_then(Value::as_str)?,
            value
                .get("targetSelectionRange")
                .or_else(|| value.get("targetRange"))?,
        )
    };
    let path = path_for_uri(uri)?;
    let start = range.get("start")?;
    let line = start.get("line").and_then(Value::as_u64).unwrap_or(0) + 1;
    let column = start.get("character").and_then(Value::as_u64).unwrap_or(0) + 1;
    Some(LspLocation { path, line, column })
}

fn hover_content(result: &Value) -> Option<String> {
    if result.is_null() {
        return None;
    }
    let contents = result.get("contents")?;
    flatten_markup(contents).filter(|text| !text.trim().is_empty())
}

/// Nome plano do `CompletionItemKind` numerico do LSP.
const fn completion_kind_name(kind: i64) -> Option<&'static str> {
    Some(match kind {
        1 => "text",
        2 => "method",
        3 => "function",
        4 => "constructor",
        5 => "field",
        6 => "variable",
        7 => "class",
        8 => "interface",
        9 => "module",
        10 => "property",
        11 => "unit",
        12 => "value",
        13 => "enum",
        14 => "keyword",
        15 => "snippet",
        16 => "color",
        17 => "file",
        18 => "reference",
        19 => "folder",
        20 => "enumMember",
        21 => "constant",
        22 => "struct",
        23 => "event",
        24 => "operator",
        25 => "typeParameter",
        _ => return None,
    })
}

/// Achata `CompletionItem[] | CompletionList | null` nos itens do protocolo.
fn completion_items(result: &Value) -> Vec<LspCompletionItem> {
    let empty = Vec::new();
    let raw = result
        .as_array()
        .or_else(|| result.get("items").and_then(Value::as_array))
        .unwrap_or(&empty);

    let mut entries = raw
        .iter()
        .filter_map(|item| {
            let label = item.get("label")?.as_str()?.trim().to_owned();
            if label.is_empty() {
                return None;
            }
            let insert_text = item
                .get("insertText")
                .and_then(Value::as_str)
                .or_else(|| {
                    item.get("textEdit")
                        .and_then(|edit| edit.get("newText"))
                        .and_then(Value::as_str)
                })
                .map_or_else(|| label.clone(), str::to_owned);
            let sort_key = item
                .get("sortText")
                .and_then(Value::as_str)
                .map_or_else(|| label.clone(), str::to_owned);
            let detail = item
                .get("detail")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_owned);
            let kind = item
                .get("kind")
                .and_then(Value::as_i64)
                .and_then(completion_kind_name)
                .map(str::to_owned);
            Some((
                sort_key,
                LspCompletionItem {
                    label,
                    insert_text,
                    detail,
                    kind,
                },
            ))
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.label.cmp(&right.1.label))
    });
    entries.truncate(MAX_COMPLETION_ITEMS);
    entries.into_iter().map(|(_sort, item)| item).collect()
}

/// Decodifica o array `data` de semantic tokens (grupos de 5 inteiros).
///
/// Cada grupo e `[deltaLine, deltaStart, length, tokenType, modifiers]`;
/// `deltaStart` e relativo ao token anterior apenas quando `deltaLine == 0`.
/// Posicoes ficam em UTF-16 (como o LSP entrega), que coincide com indices
/// de `QString` no highlighter da UI.
fn decode_semantic_tokens(result: &Value, legend: &[String]) -> Vec<LspSemanticToken> {
    let Some(data) = result.get("data").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    let mut line = 0_u64;
    let mut start = 0_u64;
    for group in data.chunks_exact(5) {
        let (Some(delta_line), Some(delta_start), Some(length), Some(kind_index)) = (
            group[0].as_u64(),
            group[1].as_u64(),
            group[2].as_u64(),
            group[3].as_u64(),
        ) else {
            continue;
        };
        line += delta_line;
        start = if delta_line == 0 {
            start + delta_start
        } else {
            delta_start
        };
        let Some(kind) = legend.get(usize::try_from(kind_index).unwrap_or(usize::MAX)) else {
            continue;
        };
        tokens.push(LspSemanticToken {
            line: line + 1,
            start,
            length,
            kind: kind.clone(),
        });
    }
    tokens
}

/// Converte `Location[]` de references nas localizacoes do Kernwerk.
fn reference_locations(result: &Value) -> Vec<LspLocation> {
    result.as_array().map_or_else(Vec::new, |locations| {
        locations
            .iter()
            .filter_map(location_from_value)
            .take(MAX_REFERENCE_ITEMS)
            .collect()
    })
}

/// Converte um `WorkspaceEdit` de rename no plano de edits por arquivo.
///
/// Operacoes de recurso (criar/renomear/apagar arquivo) ainda nao sao
/// suportadas e geram erro estruturado, sem tocar em nada.
fn workspace_edit_plan(result: &Value) -> Result<WorkspaceEditPlan, LspError> {
    let mut files = Vec::new();
    if result.is_null() {
        return Ok(WorkspaceEditPlan { files });
    }

    if let Some(changes) = result.get("changes").and_then(Value::as_object) {
        for (uri, edits) in changes {
            files.push(file_edits_from_value(uri, edits)?);
        }
    } else if let Some(changes) = result.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            if change.get("kind").and_then(Value::as_str).is_some() {
                return Err(LspError::RequestFailed {
                    method: "textDocument/rename",
                    message: "rename que cria/renomeia/apaga arquivos ainda nao e suportado"
                        .to_owned(),
                });
            }
            let uri = change
                .get("textDocument")
                .and_then(|document| document.get("uri"))
                .and_then(Value::as_str)
                .ok_or_else(|| LspError::Transport {
                    message: "documentChanges de rename sem textDocument.uri".to_owned(),
                })?;
            let edits = change.get("edits").unwrap_or(&Value::Null);
            files.push(file_edits_from_value(uri, edits)?);
        }
    }

    files.retain(|file| !file.edits.is_empty());
    Ok(WorkspaceEditPlan { files })
}

fn file_edits_from_value(uri: &str, edits: &Value) -> Result<FileEdits, LspError> {
    let path = path_for_uri(uri).ok_or_else(|| LspError::Transport {
        message: format!("uri de rename invalida: {uri}"),
    })?;
    let raw = edits.as_array().ok_or_else(|| LspError::Transport {
        message: format!("edits de rename invalidos para {path}"),
    })?;

    let mut spans = Vec::with_capacity(raw.len());
    for edit in raw {
        let new_text = edit
            .get("newText")
            .and_then(Value::as_str)
            .ok_or_else(|| LspError::Transport {
                message: format!("edit de rename sem newText em {path}"),
            })?
            .to_owned();
        let range = edit.get("range").ok_or_else(|| LspError::Transport {
            message: format!("edit de rename sem range em {path}"),
        })?;
        let start = range.get("start");
        let end = range.get("end");
        spans.push(TextSpanEdit {
            start_line: position_component(start, "line"),
            start_character: position_component(start, "character"),
            end_line: position_component(end, "line"),
            end_character: position_component(end, "character"),
            new_text,
        });
    }

    Ok(FileEdits { path, edits: spans })
}

fn position_component(position: Option<&Value>, key: &str) -> u64 {
    position
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

/// Aplica edits LSP (0-based, colunas UTF-16) a um conteudo UTF-8.
///
/// Os edits sao aplicados de tras para frente para preservar offsets; ranges
/// invertidos ou sobrepostos geram erro sem resultado parcial.
pub fn apply_text_edits(content: &str, edits: &[TextSpanEdit]) -> Result<String, LspError> {
    let starts = line_start_offsets(content);
    let mut spans = Vec::with_capacity(edits.len());
    for edit in edits {
        let start = position_to_offset(content, &starts, edit.start_line, edit.start_character);
        let end = position_to_offset(content, &starts, edit.end_line, edit.end_character);
        if end < start {
            return Err(LspError::Transport {
                message: "edit LSP com range invertido".to_owned(),
            });
        }
        spans.push((start, end, edit.new_text.as_str()));
    }
    spans.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));

    let mut previous_start = usize::MAX;
    let mut result = content.to_owned();
    for (start, end, new_text) in spans {
        if end > previous_start {
            return Err(LspError::Transport {
                message: "edits LSP sobrepostos".to_owned(),
            });
        }
        previous_start = start;
        result.replace_range(start..end, new_text);
    }
    Ok(result)
}

/// Offset em bytes do inicio de cada linha do conteudo.
fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(index + 1);
        }
    }
    starts
}

/// Converte posicao LSP (linha 0-based, coluna UTF-16) em offset de byte.
///
/// Posicoes fora do conteudo sao grampeadas ao fim do texto/linha.
fn position_to_offset(content: &str, starts: &[usize], line: u64, character: u64) -> usize {
    let Ok(line_index) = usize::try_from(line) else {
        return content.len();
    };
    let Some(&line_start) = starts.get(line_index) else {
        return content.len();
    };
    let line_end = starts
        .get(line_index + 1)
        .map_or(content.len(), |next| next.saturating_sub(1));
    line_start + utf16_col_to_byte(&content[line_start..line_end], character)
}

/// Converte uma coluna em unidades UTF-16 para offset de byte dentro da linha.
fn utf16_col_to_byte(line: &str, character: u64) -> usize {
    let target = usize::try_from(character).unwrap_or(usize::MAX);
    let mut units = 0_usize;
    for (byte_index, symbol) in line.char_indices() {
        if units >= target {
            return byte_index;
        }
        units += symbol.len_utf16();
    }
    line.len()
}

fn flatten_markup(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    if let Some(text) = value.get("value").and_then(Value::as_str) {
        return Some(text.to_owned());
    }
    if let Some(items) = value.as_array() {
        let parts = items.iter().filter_map(flatten_markup).collect::<Vec<_>>();
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("\n\n"));
    }
    None
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'A' + value - 10) as char,
    }
}

fn uri_for_path(path: &Path) -> String {
    let mut encoded = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        let is_plain = byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'-' | b'_');
        if is_plain {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(hex_digit(byte >> 4_u8));
            encoded.push(hex_digit(byte & 0x0F));
        }
    }
    encoded
}

fn path_for_uri(uri: &str) -> Option<String> {
    let raw = uri.strip_prefix("file://")?;
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let value = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::Path;

    use serde_json::json;

    use super::{
        TextSpanEdit, apply_text_edits, completion_items, decode_semantic_tokens,
        definition_location, diagnostics_event, hover_content, path_for_uri, read_message,
        reference_locations, semantic_token_legend, uri_for_path, workspace_edit_plan,
        write_message,
    };

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
    fn decode_semantic_tokens_applies_line_and_start_deltas() {
        let legend = vec!["variable".to_owned(), "function".to_owned()];
        // Linha 0: token em 4..7 (function); mesmo-linha em 10..13 (variable);
        // linha 2 (delta 2): token em 0..5 (variable).
        let result = json!({
            "data": [
                0, 4, 3, 1, 0,
                0, 6, 3, 0, 0,
                2, 0, 5, 0, 0,
            ]
        });

        let tokens = decode_semantic_tokens(&result, &legend);

        assert_eq!(tokens.len(), 3);
        assert_eq!(
            (tokens[0].line, tokens[0].start, tokens[0].length),
            (1, 4, 3)
        );
        assert_eq!(tokens[0].kind, "function");
        assert_eq!((tokens[1].line, tokens[1].start), (1, 10));
        assert_eq!(tokens[1].kind, "variable");
        assert_eq!((tokens[2].line, tokens[2].start), (3, 0));
    }

    #[test]
    fn decode_semantic_tokens_skips_kinds_outside_the_legend() {
        let legend = vec!["variable".to_owned()];
        let result = json!({ "data": [0, 0, 2, 9, 0, 0, 4, 2, 0, 0] });

        let tokens = decode_semantic_tokens(&result, &legend);

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].start, 4);
        assert_eq!(tokens[0].kind, "variable");
    }

    fn span(
        start_line: u64,
        start_character: u64,
        end_line: u64,
        end_character: u64,
        new_text: &str,
    ) -> TextSpanEdit {
        TextSpanEdit {
            start_line,
            start_character,
            end_line,
            end_character,
            new_text: new_text.to_owned(),
        }
    }

    #[test]
    fn framing_roundtrip_preserves_message() {
        let message = json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} });
        let mut buffer = Vec::new();
        write_message(&mut buffer, &message).unwrap();

        let text = String::from_utf8(buffer.clone()).unwrap();
        assert!(text.starts_with("Content-Length: "));

        let decoded = read_message(&mut Cursor::new(buffer)).unwrap().unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn read_message_reports_eof_as_none() {
        let mut empty = Cursor::new(Vec::new());
        assert!(read_message(&mut empty).unwrap().is_none());
    }

    #[test]
    fn uri_conversion_roundtrip_with_spaces() {
        let path = Path::new("/home/user/meu projeto/main.rs");
        let uri = uri_for_path(path);

        assert_eq!(uri, "file:///home/user/meu%20projeto/main.rs");
        assert_eq!(
            path_for_uri(&uri).unwrap(),
            "/home/user/meu projeto/main.rs"
        );
    }

    #[test]
    fn publish_diagnostics_becomes_kernwerk_event() {
        let params = json!({
            "uri": "file:///tmp/demo/src/main.rs",
            "diagnostics": [{
                "range": { "start": { "line": 4, "character": 8 }, "end": {} },
                "severity": 1,
                "message": "mismatched types",
            }],
        });

        let event = diagnostics_event(&params).unwrap();

        assert_eq!(event.method, "event.lsp.diagnostics");
        let event_params = event.params.unwrap();
        assert_eq!(event_params["path"], "/tmp/demo/src/main.rs");
        assert_eq!(event_params["diagnostics"][0]["line"], 5);
        assert_eq!(event_params["diagnostics"][0]["column"], 9);
        assert_eq!(event_params["diagnostics"][0]["severity"], "error");
    }

    #[test]
    fn definition_location_accepts_location_array() {
        let result = json!([{
            "uri": "file:///tmp/demo/src/lib.rs",
            "range": { "start": { "line": 9, "character": 4 }, "end": {} },
        }]);

        let location = definition_location(&result).unwrap();

        assert_eq!(location.path, "/tmp/demo/src/lib.rs");
        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
    }

    #[test]
    fn definition_location_accepts_location_link() {
        let result = json!([{
            "targetUri": "file:///tmp/demo/src/main.cpp",
            "targetSelectionRange": {
                "start": { "line": 2, "character": 11 },
                "end": {},
            },
        }]);

        let location = definition_location(&result).unwrap();

        assert_eq!(location.path, "/tmp/demo/src/main.cpp");
        assert_eq!(location.line, 3);
        assert_eq!(location.column, 12);
    }

    #[test]
    fn hover_content_flattens_markup_content() {
        let result = json!({
            "contents": {
                "kind": "markdown",
                "value": "```rust\nfn main()\n```",
            },
        });

        assert_eq!(hover_content(&result).unwrap(), "```rust\nfn main()\n```");
    }

    #[test]
    fn unsupported_extensions_have_no_server() {
        assert!(super::spec_for_path(Path::new("/tmp/nota.txt")).is_none());
        assert!(super::spec_for_path(Path::new("/tmp/main.rs")).is_some());
        assert!(super::spec_for_path(Path::new("/tmp/app.cpp")).is_some());
    }

    #[test]
    fn completion_items_accepts_list_and_array_and_sorts_by_sort_text() {
        let as_list = json!({
            "isIncomplete": false,
            "items": [
                {
                    "label": " zebra()",
                    "kind": 3,
                    "sortText": "b",
                    "detail": "fn zebra()",
                    "insertText": "zebra",
                },
                { "label": "alpha", "kind": 6, "sortText": "a" },
            ],
        });
        let items = completion_items(&as_list);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].label, "alpha");
        assert_eq!(items[0].insert_text, "alpha");
        assert_eq!(items[0].kind.as_deref(), Some("variable"));
        assert_eq!(items[1].label, "zebra()");
        assert_eq!(items[1].insert_text, "zebra");
        assert_eq!(items[1].detail.as_deref(), Some("fn zebra()"));
        assert_eq!(items[1].kind.as_deref(), Some("function"));

        let as_array = json!([{ "label": "solo" }]);
        assert_eq!(completion_items(&as_array).len(), 1);
        assert!(completion_items(&json!(null)).is_empty());
    }

    #[test]
    fn completion_items_uses_text_edit_and_caps_results() {
        let mut raw_items = Vec::new();
        for index in 0..80 {
            raw_items.push(json!({
                "label": format!("item{index:03}"),
                "textEdit": {
                    "range": { "start": { "line": 0, "character": 0 }, "end": {} },
                    "newText": format!("item{index:03}"),
                },
            }));
        }
        let items = completion_items(&json!(raw_items));

        assert_eq!(items.len(), super::MAX_COMPLETION_ITEMS);
        assert_eq!(items[0].insert_text, "item000");
    }

    #[test]
    fn reference_locations_parses_location_array() {
        let result = json!([
            {
                "uri": "file:///tmp/demo/src/main.rs",
                "range": { "start": { "line": 0, "character": 3 }, "end": {} },
            },
            {
                "uri": "file:///tmp/demo/src/lib.rs",
                "range": { "start": { "line": 7, "character": 0 }, "end": {} },
            },
        ]);
        let references = reference_locations(&result);

        assert_eq!(references.len(), 2);
        assert_eq!(references[0].path, "/tmp/demo/src/main.rs");
        assert_eq!(references[0].line, 1);
        assert_eq!(references[0].column, 4);
        assert!(reference_locations(&json!(null)).is_empty());
    }

    #[test]
    fn workspace_edit_plan_parses_changes_map() {
        let result = json!({
            "changes": {
                "file:///tmp/demo/src/main.rs": [
                    {
                        "range": {
                            "start": { "line": 0, "character": 3 },
                            "end": { "line": 0, "character": 7 },
                        },
                        "newText": "start",
                    },
                ],
            },
        });
        let plan = workspace_edit_plan(&result).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, "/tmp/demo/src/main.rs");
        assert_eq!(plan.files[0].edits[0].new_text, "start");
        assert_eq!(plan.files[0].edits[0].end_character, 7);
        assert_eq!(plan.edit_count(), 1);
    }

    #[test]
    fn workspace_edit_plan_parses_document_changes() {
        let result = json!({
            "documentChanges": [
                {
                    "textDocument": { "uri": "file:///tmp/demo/src/lib.rs", "version": 4 },
                    "edits": [
                        {
                            "range": {
                                "start": { "line": 2, "character": 0 },
                                "end": { "line": 2, "character": 3 },
                            },
                            "newText": "novo",
                        },
                    ],
                },
            ],
        });
        let plan = workspace_edit_plan(&result).unwrap();

        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.files[0].path, "/tmp/demo/src/lib.rs");
        assert_eq!(plan.edit_count(), 1);
    }

    #[test]
    fn workspace_edit_plan_rejects_resource_operations_and_accepts_null() {
        let with_resource_op = json!({
            "documentChanges": [
                { "kind": "rename", "oldUri": "file:///a.rs", "newUri": "file:///b.rs" },
            ],
        });
        assert!(workspace_edit_plan(&with_resource_op).is_err());

        let plan = workspace_edit_plan(&json!(null)).unwrap();
        assert!(plan.files.is_empty());
        assert_eq!(plan.edit_count(), 0);
    }

    #[test]
    fn apply_text_edits_applies_reverse_order_on_same_line() {
        let content = "let valor = valor + valor;\n";
        let edits = vec![
            span(0, 4, 0, 9, "total"),
            span(0, 12, 0, 17, "total"),
            span(0, 20, 0, 25, "total"),
        ];

        let result = apply_text_edits(content, &edits).unwrap();

        assert_eq!(result, "let total = total + total;\n");
    }

    #[test]
    fn apply_text_edits_handles_utf16_columns_and_multiline() {
        // "á" ocupa 1 unidade UTF-16 e 2 bytes UTF-8; o edit troca `nome`.
        let content = "// á comentário\nfn nome() {}\nnome();\n";
        let edits = vec![span(1, 3, 1, 7, "inicio"), span(2, 0, 2, 4, "inicio")];

        let result = apply_text_edits(content, &edits).unwrap();

        assert_eq!(result, "// á comentário\nfn inicio() {}\ninicio();\n");
    }

    #[test]
    fn apply_text_edits_rejects_overlapping_and_inverted_ranges() {
        let content = "abcdef\n";
        let overlapping = vec![span(0, 0, 0, 4, "x"), span(0, 2, 0, 6, "y")];
        assert!(apply_text_edits(content, &overlapping).is_err());

        let inverted = vec![span(0, 5, 0, 2, "x")];
        assert!(apply_text_edits(content, &inverted).is_err());
    }

    #[test]
    fn apply_text_edits_clamps_positions_past_the_end() {
        let content = "fim";
        let edits = vec![span(9, 9, 9, 9, "!")];

        assert_eq!(apply_text_edits(content, &edits).unwrap(), "fim!");
    }
}
