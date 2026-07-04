//! Rust core for Kernwerk Studio.
//!
//! The core owns command execution and external tool orchestration. The UI must
//! communicate through IPC and must not call build tools or language servers
//! directly.

#![forbid(unsafe_code)]

pub mod build;
pub mod fsops;
pub mod lsp;
pub mod process;
pub mod run;
pub mod terminal;
pub mod test;
pub mod tools;
pub mod workspace;

use std::{
    error::Error,
    fmt,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

use kernwerk_protocol::{
    BuildRunResult, CommandDescriptor, CorePingResult, FsCreateDirectoryParams,
    FsCreateDirectoryResult, FsCreateFileParams, FsCreateFileResult, FsDeleteResult,
    FsFindFilesParams, FsFindFilesResult, FsListResult, FsPathParams, FsReadResult, FsRenameParams,
    FsRenameResult, FsSearchParams, FsSearchResult, FsWriteParams, FsWriteResult, JsonRpcError,
    JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, LspCompletionResult, LspDefinitionResult,
    LspHoverResult, LspReferenceItem, LspReferencesResult, LspRenameParams, LspRenameResult,
    LspSemanticTokensResult, LspTextDocumentPositionParams, QualityRunResult, RunStartParams,
    RunStartResult, RunStdinParams, TerminalInputParams, TerminalOpenResult, TestRunResult,
    ToolInfo, ToolsDetectResult, WorkspaceBrowseParams, WorkspaceCreateFolderParams,
    WorkspaceCreateFolderResult, WorkspaceCreateProjectParams, WorkspaceInfo, WorkspaceOpenParams,
    WorkspaceStatusResult,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::tools::ToolDetector;

/// Stateful core runtime.
#[derive(Debug, Default)]
pub struct Core {
    detector: ToolDetector,
    tool_registry: Option<Vec<ToolInfo>>,
    workspace: Option<WorkspaceInfo>,
    lsp: Option<lsp::LspManager>,
    run: Option<run::RunManager>,
    terminal: Option<terminal::TerminalManager>,
}

impl Core {
    /// Creates a new core runtime that detects tools through `PATH`.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_detector(ToolDetector::from_environment())
    }

    /// Creates a core runtime with an injected tool detector.
    #[must_use]
    pub const fn with_detector(detector: ToolDetector) -> Self {
        Self {
            detector,
            tool_registry: None,
            workspace: None,
            lsp: None,
            run: None,
            terminal: None,
        }
    }

    /// Enables LSP, process execution and the terminal session, pushing
    /// async notifications (`event.lsp.*`, `event.run.*`,
    /// `event.terminal.*`) through `events`.
    ///
    /// Without this call (tests and `run_json_lines`), LSP operations are
    /// no-ops, no language server is spawned and `run.*`/`terminal.*` are
    /// unavailable.
    pub fn enable_lsp(&mut self, events: lsp::EventSender) {
        self.lsp = Some(lsp::LspManager::new(events.clone()));
        self.run = Some(run::RunManager::new(events.clone()));
        self.terminal = Some(terminal::TerminalManager::new(events));
    }

    /// Handles one already parsed JSON-RPC request.
    #[must_use]
    pub fn handle_request(&mut self, request: &JsonRpcRequest) -> RequestOutcome {
        let request_id = request.id.clone();

        if !request.has_supported_version() {
            let error = JsonRpcError::new(
                JsonRpcErrorCode::InvalidRequest,
                "Unsupported JSON-RPC version",
                Some(json!({
                    "expected": kernwerk_protocol::JSON_RPC_VERSION,
                    "received": request.jsonrpc.as_str(),
                })),
            );
            return RequestOutcome::Continue(JsonRpcResponse::failure(request_id, error));
        }

        if request.id.is_none() {
            let error = JsonRpcError::new(
                JsonRpcErrorCode::InvalidRequest,
                "Kernwerk commands must include a request id",
                Some(json!({ "method": request.method.as_str() })),
            );
            return RequestOutcome::Continue(JsonRpcResponse::failure(Some(Value::Null), error));
        }

        match request.method.as_str() {
            "core.ping" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!(CorePingResult::default()),
            )),
            "core.shutdown" => RequestOutcome::Shutdown(JsonRpcResponse::success(
                request_id,
                json!({
                    "status": "ok",
                    "message": "shutdown requested",
                }),
            )),
            "command.list" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!({ "commands": command_descriptors() }),
            )),
            "tools.detect" => {
                let tools = self.detector.detect_all();
                self.tool_registry = Some(tools.clone());
                RequestOutcome::Continue(JsonRpcResponse::success(
                    request_id,
                    json!(ToolsDetectResult { tools }),
                ))
            }
            "tools.status" => {
                let tools = self
                    .tool_registry
                    .get_or_insert_with(|| self.detector.detect_all())
                    .clone();
                RequestOutcome::Continue(JsonRpcResponse::success(
                    request_id,
                    json!(ToolsDetectResult { tools }),
                ))
            }
            "workspace.open" => RequestOutcome::Continue(
                self.open_workspace_response(request_id, request.params.as_ref()),
            ),
            "workspace.browse" => RequestOutcome::Continue(Self::browse_workspace_response(
                request_id,
                request.params.as_ref(),
            )),
            "workspace.createFolder" => RequestOutcome::Continue(
                Self::create_workspace_folder_response(request_id, request.params.as_ref()),
            ),
            "workspace.createProject" => RequestOutcome::Continue(
                self.create_workspace_project_response(request_id, request.params.as_ref()),
            ),
            "workspace.status" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!(WorkspaceStatusResult {
                    workspace: self.workspace.clone(),
                }),
            )),
            "workspace.close" => {
                RequestOutcome::Continue(self.close_workspace_response(request_id))
            }
            method => RequestOutcome::Continue(self.service_request_response(
                method,
                request_id,
                request.params.as_ref(),
            )),
        }
    }

    /// Encadeia os roteadores de servico (`fs.*`, `run.*`, `terminal.*`,
    /// `lsp.*`); metodo desconhecido vira `METHOD_NOT_FOUND`.
    fn service_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        self.fs_request_response(method, request_id.clone(), params)
            .or_else(|| self.run_request_response(method, request_id.clone(), params))
            .or_else(|| self.terminal_request_response(method, request_id.clone(), params))
            .or_else(|| self.lsp_request_response(method, request_id.clone(), params))
            .unwrap_or_else(|| {
                let error = JsonRpcError::new(
                    JsonRpcErrorCode::MethodNotFound,
                    "Command is not registered",
                    Some(json!({ "method": method })),
                );
                JsonRpcResponse::failure(request_id, error)
            })
    }

    /// Roteia os metodos `fs.*`; `None` quando o metodo nao e de arquivos.
    fn fs_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "fs.list" => Some(self.fs_list_response(request_id, params)),
            "fs.read" => Some(self.fs_read_response(request_id, params)),
            "fs.createFile" => Some(self.fs_create_file_response(request_id, params)),
            "fs.createDirectory" => Some(self.fs_create_directory_response(request_id, params)),
            "fs.write" => Some(self.fs_write_response(request_id, params)),
            "fs.rename" => Some(self.fs_rename_response(request_id, params)),
            "fs.delete" => Some(self.fs_delete_response(request_id, params)),
            "fs.findFiles" => Some(self.fs_find_files_response(request_id, params)),
            "fs.search" => Some(self.fs_search_response(request_id, params)),
            _ => None,
        }
    }

    /// Roteia os metodos `run.*`; `None` quando o metodo nao e de execucao.
    fn run_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "run.start" => Some(self.run_start_response(request_id, params)),
            "run.stdin" => Some(self.run_stdin_response(request_id, params)),
            "run.stop" => Some(self.run_stop_response(request_id)),
            _ => None,
        }
    }

    fn run_start_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "run.start");
        };
        let parsed = match parse_params::<RunStartParams>(
            request_id.as_ref(),
            params,
            "run.start aceita apenas o campo opcional command",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.start");
        };

        let root = Path::new(&workspace.root);
        let command = match parsed.command.filter(|command| !command.trim().is_empty()) {
            Some(command) => command,
            None => match run::default_command(workspace.kind, root) {
                Ok(command) => command,
                Err(error) => return run_error_response(request_id, &error),
            },
        };

        match runner.start(root, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stdin_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RunStdinParams>(
            request_id.as_ref(),
            params,
            "run.stdin requer o campo data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stdin");
        };
        match runner.write_stdin(&parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stop_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stop");
        };
        match runner.stop() {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    /// Roteia os metodos `terminal.*`; `None` quando o metodo nao e terminal.
    fn terminal_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "terminal.open" => Some(self.terminal_open_response(request_id)),
            "terminal.input" => Some(self.terminal_input_response(request_id, params)),
            "terminal.close" => Some(self.terminal_close_response(request_id)),
            _ => None,
        }
    }

    fn terminal_open_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "terminal.open");
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.open");
        };
        match session.open(&root) {
            Ok(shell) => JsonRpcResponse::success(request_id, json!(TerminalOpenResult { shell })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_input_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<TerminalInputParams>(
            request_id.as_ref(),
            params,
            "terminal.input requer o campo data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.input");
        };
        match session.write(&parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    fn terminal_close_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(session) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "terminal.close");
        };
        match session.close() {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }

    /// Roteia os metodos `lsp.*`; `None` quando o metodo nao e LSP.
    fn lsp_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "lsp.didChange" => Some(self.lsp_did_change_response(request_id, params)),
            "lsp.semanticTokens" => Some(self.lsp_semantic_tokens_response(request_id, params)),
            "lsp.definition" => Some(self.lsp_definition_response(request_id, params)),
            "lsp.hover" => Some(self.lsp_hover_response(request_id, params)),
            "lsp.completion" => Some(self.lsp_completion_response(request_id, params)),
            "lsp.references" => Some(self.lsp_references_response(request_id, params)),
            "lsp.rename" => Some(self.lsp_rename_response(request_id, params)),
            _ => None,
        }
    }

    fn close_workspace_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let closed = self.workspace.take();
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.set_root(None);
        }
        if let Some(runner) = self.run.as_mut() {
            drop(runner.stop());
        }
        if let Some(session) = self.terminal.as_mut() {
            drop(session.close());
        }
        JsonRpcResponse::success(
            request_id,
            json!({
                "status": "ok",
                "closed": closed.map(|workspace| workspace.root),
            }),
        )
    }

    fn browse_workspace_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceBrowseParams>(params) {
            Ok(params) => match workspace::browse_directories(Path::new(&params.path)) {
                Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.browse requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    fn create_workspace_folder_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateFolderParams>(params) {
            Ok(params) => {
                match workspace::create_directory(Path::new(&params.parent), &params.name) {
                    Ok(path) => JsonRpcResponse::success(
                        request_id,
                        json!(WorkspaceCreateFolderResult {
                            path: path.display().to_string(),
                        }),
                    ),
                    Err(error) => workspace_error_response(request_id, &error),
                }
            }
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createFolder requer parent e name",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    fn create_workspace_project_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateProjectParams>(params) {
            Ok(params) => match workspace::create_project(
                Path::new(&params.parent),
                &params.name,
                params.template,
            ) {
                Ok(opened) => {
                    self.workspace = Some(opened.clone());
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
                    }
                    JsonRpcResponse::success(request_id, json!(opened))
                }
                Err(error) => workspace_error_response(request_id, &error),
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createProject requer parent, name e template",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    fn open_workspace_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceOpenParams>(params) {
            Ok(params) => match workspace::open_workspace(Path::new(&params.path)) {
                Ok(opened) => {
                    self.workspace = Some(opened.clone());
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.set_root(Some(PathBuf::from(&opened.root)));
                    }
                    JsonRpcResponse::success(request_id, json!(opened))
                }
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.open requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace
            .as_ref()
            .map(|workspace| PathBuf::from(&workspace.root))
    }

    fn fs_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.list");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.list requer o campo path",
        ) {
            Ok(parsed) => match fsops::list_dir(&root, Path::new(&parsed.path)) {
                Ok((path, entries)) => JsonRpcResponse::success(
                    request_id,
                    json!(FsListResult {
                        path: path.display().to_string(),
                        entries,
                    }),
                ),
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_read_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.read");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.read requer o campo path",
        ) {
            Ok(parsed) => match fsops::read_file(&root, Path::new(&parsed.path)) {
                Ok((path, content)) => {
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.did_open(&path, &content);
                    }
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsReadResult {
                            path: path.display().to_string(),
                            content,
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_search_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.search");
        };
        match parse_params::<FsSearchParams>(
            request_id.as_ref(),
            params,
            "fs.search requer o campo query",
        ) {
            Ok(parsed) => {
                if parsed.query.is_empty() {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            "fs.search requer query nao vazia",
                            None,
                        ),
                    );
                }
                match fsops::search(&root, &parsed.query, parsed.case_sensitive) {
                    Ok((matches, truncated)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsSearchResult { matches, truncated }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_find_files_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.findFiles");
        };
        match parse_params::<FsFindFilesParams>(
            request_id.as_ref(),
            params,
            "fs.findFiles requer o campo query",
        ) {
            Ok(parsed) => {
                if parsed.query.is_empty() {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            "fs.findFiles requer query nao vazia",
                            None,
                        ),
                    );
                }
                match fsops::find_files(&root, &parsed.query) {
                    Ok((matches, truncated)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsFindFilesResult { matches, truncated }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_create_file_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.createFile");
        };
        match parse_params::<FsCreateFileParams>(
            request_id.as_ref(),
            params,
            "fs.createFile requer o campo path e aceita content opcional",
        ) {
            Ok(parsed) => {
                match fsops::create_file(&root, Path::new(&parsed.path), &parsed.content) {
                    Ok((path, bytes_written)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsCreateFileResult {
                            path: path.display().to_string(),
                            bytes_written,
                        }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_create_directory_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.createDirectory");
        };
        match parse_params::<FsCreateDirectoryParams>(
            request_id.as_ref(),
            params,
            "fs.createDirectory requer o campo path",
        ) {
            Ok(parsed) => match fsops::create_directory(&root, Path::new(&parsed.path)) {
                Ok(path) => JsonRpcResponse::success(
                    request_id,
                    json!(FsCreateDirectoryResult {
                        path: path.display().to_string(),
                    }),
                ),
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_write_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.write");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "fs.write requer os campos path e content",
        ) {
            Ok(parsed) => {
                match fsops::write_file(&root, Path::new(&parsed.path), &parsed.content) {
                    Ok((path, bytes_written)) => {
                        if let Some(lsp) = self.lsp.as_mut() {
                            lsp.did_save(&path, &parsed.content);
                        }
                        JsonRpcResponse::success(
                            request_id,
                            json!(FsWriteResult {
                                path: path.display().to_string(),
                                bytes_written,
                            }),
                        )
                    }
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_rename_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.rename");
        };
        match parse_params::<FsRenameParams>(
            request_id.as_ref(),
            params,
            "fs.rename requer os campos from e to",
        ) {
            Ok(parsed) => {
                match fsops::rename(&root, Path::new(&parsed.from), Path::new(&parsed.to)) {
                    Ok((from, to)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsRenameResult {
                            from: from.display().to_string(),
                            to: to.display().to_string(),
                        }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_delete_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.delete");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.delete requer o campo path",
        ) {
            Ok(parsed) => match fsops::delete(&root, Path::new(&parsed.path)) {
                Ok(path) => JsonRpcResponse::success(
                    request_id,
                    json!(FsDeleteResult {
                        path: path.display().to_string(),
                    }),
                ),
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn lsp_did_change_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.didChange");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.didChange requer os campos path e content",
        ) {
            Ok(parsed) => {
                let path = Path::new(&parsed.path);
                let canonical = match fsops::confine_file(&root, path) {
                    Ok(canonical) => canonical,
                    Err(error) => return fs_error_response(request_id, &error),
                };
                if let Some(lsp) = self.lsp.as_mut() {
                    lsp.did_change(&canonical, &parsed.content);
                }
                JsonRpcResponse::success(request_id, json!({ "status": "ok" }))
            }
            Err(response) => *response,
        }
    }

    fn lsp_semantic_tokens_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.semanticTokens");
        };
        match parse_params::<FsWriteParams>(
            request_id.as_ref(),
            params,
            "lsp.semanticTokens requer os campos path e content",
        ) {
            Ok(parsed) => {
                let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
                    Ok(canonical) => canonical,
                    Err(error) => return fs_error_response(request_id, &error),
                };
                let Some(lsp) = self.lsp.as_mut() else {
                    return lsp_unavailable_response(request_id, "lsp.semanticTokens");
                };
                match lsp.semantic_tokens(&path, &parsed.content) {
                    Ok(tokens) => JsonRpcResponse::success(
                        request_id,
                        json!(LspSemanticTokensResult { tokens }),
                    ),
                    Err(error) => lsp_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn lsp_definition_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.definition");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.definition requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.definition");
        };
        match lsp.definition(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(location) => JsonRpcResponse::success(
                request_id,
                json!(LspDefinitionResult {
                    path: location.as_ref().map(|target| target.path.clone()),
                    line: location.as_ref().map(|target| target.line),
                    column: location.as_ref().map(|target| target.column),
                }),
            ),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_hover_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.hover");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.hover requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.hover");
        };
        match lsp.hover(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(content) => JsonRpcResponse::success(request_id, json!(LspHoverResult { content })),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_completion_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.completion");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.completion requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.completion");
        };
        match lsp.completion(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(items) => JsonRpcResponse::success(request_id, json!(LspCompletionResult { items })),
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    fn lsp_references_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.references");
        };
        let (path, parsed) = match parse_lsp_position_params(
            &root,
            request_id.as_ref(),
            params,
            "lsp.references requer path, content, line e column",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.references");
        };
        match lsp.references(&path, &parsed.content, parsed.line, parsed.column) {
            Ok(locations) => {
                let references = locations
                    .into_iter()
                    .map(|location| LspReferenceItem {
                        path: location.path,
                        line: location.line,
                        column: location.column,
                    })
                    .collect();
                JsonRpcResponse::success(request_id, json!(LspReferencesResult { references }))
            }
            Err(error) => lsp_error_response(request_id, &error),
        }
    }

    /// Executa `lsp.rename` fim a fim: consulta o servidor, valida que todos
    /// os arquivos afetados estao dentro do workspace, aplica os edits em
    /// memoria e so entao reescreve os arquivos no disco.
    fn lsp_rename_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "lsp.rename");
        };
        let parsed = match parse_params::<LspRenameParams>(
            request_id.as_ref(),
            params,
            "lsp.rename requer path, content, line, column e newName",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.new_name.trim().is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "lsp.rename requer um novo nome nao vazio",
                    None,
                ),
            );
        }
        let active_path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(lsp) = self.lsp.as_mut() else {
            return lsp_unavailable_response(request_id, "lsp.rename");
        };
        let plan = match lsp.rename(
            &active_path,
            &parsed.content,
            parsed.line,
            parsed.column,
            parsed.new_name.trim(),
        ) {
            Ok(plan) => plan,
            Err(error) => return lsp_error_response(request_id, &error),
        };

        let edits = plan.edit_count();
        let mut updates: Vec<(PathBuf, String)> = Vec::with_capacity(plan.files.len());
        for file in &plan.files {
            let target = match fsops::confine_file(&root, Path::new(&file.path)) {
                Ok(target) => target,
                Err(error) => return fs_error_response(request_id, &error),
            };
            let base = if target == active_path {
                parsed.content.clone()
            } else {
                match fsops::read_file(&root, &target) {
                    Ok((_, content)) => content,
                    Err(error) => return fs_error_response(request_id, &error),
                }
            };
            let rewritten = match lsp::apply_text_edits(&base, &file.edits) {
                Ok(rewritten) => rewritten,
                Err(error) => return lsp_error_response(request_id, &error),
            };
            updates.push((target, rewritten));
        }

        let mut files = Vec::with_capacity(updates.len());
        for (target, content) in &updates {
            if let Err(error) = fsops::write_file(&root, target, content) {
                return fs_error_response(request_id, &error);
            }
            files.push(target.display().to_string());
        }
        for (target, content) in &updates {
            lsp.sync_if_open(target, content);
        }

        JsonRpcResponse::success(request_id, json!(LspRenameResult { files, edits }))
    }

    /// Handles a request, emitting streamed events for long-running methods.
    ///
    /// `build.run` streams `event.build.*` notifications through `emit`
    /// before returning its response. Every other method behaves exactly
    /// like [`Core::handle_request`].
    #[must_use]
    pub fn handle_request_streaming(
        &mut self,
        request: &JsonRpcRequest,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> RequestOutcome {
        if request.method == "build.run" && request.has_supported_version() && request.id.is_some()
        {
            return RequestOutcome::Continue(self.build_run_response(request.id.clone(), emit));
        }
        if request.method == "test.run" && request.has_supported_version() && request.id.is_some() {
            return RequestOutcome::Continue(self.test_run_response(
                request.id.clone(),
                request.params.as_ref(),
                emit,
            ));
        }
        if request.method == "quality.run"
            && request.has_supported_version()
            && request.id.is_some()
        {
            return RequestOutcome::Continue(self.quality_run_response(request.id.clone(), emit));
        }
        self.handle_request(request)
    }

    /// Parses and handles a single line-delimited JSON-RPC request.
    #[must_use]
    pub fn handle_json_line(&mut self, line: &str) -> RequestOutcome {
        self.handle_json_line_streaming(line, &mut |_notification| {})
    }

    /// Parses and handles one request line, streaming events through `emit`.
    #[must_use]
    pub fn handle_json_line_streaming(
        &mut self,
        line: &str,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> RequestOutcome {
        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => self.handle_request_streaming(&request, emit),
            Err(error) => {
                let protocol_error = JsonRpcError::new(
                    JsonRpcErrorCode::ParseError,
                    "Input line is not a valid Kernwerk JSON-RPC request",
                    Some(json!({ "error": error.to_string() })),
                );
                RequestOutcome::Continue(JsonRpcResponse::failure(
                    Some(Value::Null),
                    protocol_error,
                ))
            }
        }
    }

    fn build_run_response(
        &self,
        request_id: Option<Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "build.run");
        };
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: build::BuildEvent| {
            let (method, params) = match event {
                build::BuildEvent::Started { command } => {
                    ("event.build.started", json!({ "command": command }))
                }
                build::BuildEvent::Output { stream, line } => (
                    "event.build.output",
                    json!({ "stream": stream, "line": line }),
                ),
                build::BuildEvent::Diagnostic(diagnostic) => {
                    ("event.build.diagnostic", json!(diagnostic))
                }
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        match build::run_build(&root, kind, &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.build.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "diagnostics": outcome.diagnostics,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(BuildRunResult {
                        success: outcome.success,
                        exit_code: outcome.exit_code,
                        diagnostics: outcome.diagnostics,
                    }),
                )
            }
            Err(error) => {
                let code = if error.is_missing_tool() {
                    JsonRpcErrorCode::ToolNotFound
                } else if matches!(error, build::BuildError::Unsupported { .. }) {
                    JsonRpcErrorCode::InvalidRequest
                } else {
                    JsonRpcErrorCode::InternalError
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(code, error.to_string(), None),
                )
            }
        }
    }

    fn quality_run_response(
        &self,
        request_id: Option<Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "quality.run");
        };
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: build::BuildEvent| {
            let (method, params) = match event {
                build::BuildEvent::Started { command } => {
                    ("event.quality.started", json!({ "command": command }))
                }
                build::BuildEvent::Output { stream, line } => (
                    "event.quality.output",
                    json!({ "stream": stream, "line": line }),
                ),
                build::BuildEvent::Diagnostic(diagnostic) => {
                    ("event.quality.diagnostic", json!(diagnostic))
                }
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        match build::run_quality(&root, kind, &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.quality.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "diagnostics": outcome.diagnostics,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(QualityRunResult {
                        success: outcome.success,
                        exit_code: outcome.exit_code,
                        diagnostics: outcome.diagnostics,
                    }),
                )
            }
            Err(error) => {
                let code = if error.is_missing_tool() {
                    JsonRpcErrorCode::ToolNotFound
                } else if matches!(error, build::BuildError::Unsupported { .. }) {
                    JsonRpcErrorCode::InvalidRequest
                } else {
                    JsonRpcErrorCode::InternalError
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(code, error.to_string(), None),
                )
            }
        }
    }

    fn test_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
        emit: &mut dyn FnMut(&JsonRpcRequest),
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.as_ref() else {
            return no_workspace_response(request_id, "test.run");
        };
        let filter = params
            .and_then(|value| value.get("filter"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let root = PathBuf::from(&workspace.root);
        let kind = workspace.kind;

        let mut sink = |event: test::TestEvent| {
            let (method, params) = match event {
                test::TestEvent::Started { command } => {
                    ("event.test.started", json!({ "command": command }))
                }
                test::TestEvent::Output { stream, line } => (
                    "event.test.output",
                    json!({ "stream": stream, "line": line }),
                ),
                test::TestEvent::Case { name, status } => (
                    "event.test.case",
                    json!({ "name": name, "status": status.as_str() }),
                ),
            };
            emit(&JsonRpcRequest::notification(method, Some(params)));
        };

        match test::run_tests(&root, kind, filter.as_deref(), &mut sink) {
            Ok(outcome) => {
                emit(&JsonRpcRequest::notification(
                    "event.test.finished",
                    Some(json!({
                        "success": outcome.success,
                        "exitCode": outcome.exit_code,
                        "passed": outcome.passed,
                        "failed": outcome.failed,
                        "ignored": outcome.ignored,
                    })),
                ));
                JsonRpcResponse::success(
                    request_id,
                    json!(TestRunResult {
                        success: outcome.success,
                        exit_code: outcome.exit_code,
                        passed: outcome.passed,
                        failed: outcome.failed,
                        ignored: outcome.ignored,
                    }),
                )
            }
            Err(error) => {
                let code = if error.is_missing_tool() {
                    JsonRpcErrorCode::ToolNotFound
                } else if matches!(error, test::TestError::Unsupported { .. }) {
                    JsonRpcErrorCode::InvalidRequest
                } else {
                    JsonRpcErrorCode::InternalError
                };
                JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(code, error.to_string(), None),
                )
            }
        }
    }
}

fn no_workspace_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidRequest,
            "nenhum workspace aberto",
            Some(json!({ "method": method })),
        ),
    )
}

fn fs_error_response(request_id: Option<Value>, error: &fsops::FsError) -> JsonRpcResponse {
    let code = if error.is_invalid_path() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

fn workspace_error_response(
    request_id: Option<Value>,
    error: &workspace::WorkspaceError,
) -> JsonRpcResponse {
    let code = if error.is_invalid_path() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

fn terminal_unavailable_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "terminal nao esta habilitado neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

fn terminal_error_response(
    request_id: Option<Value>,
    error: &terminal::TerminalError,
) -> JsonRpcResponse {
    let code = match error {
        terminal::TerminalError::Process { .. } => JsonRpcErrorCode::InternalError,
        terminal::TerminalError::AlreadyOpen | terminal::TerminalError::NotOpen => {
            JsonRpcErrorCode::InvalidRequest
        }
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

fn run_unavailable_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "execucao de processos nao esta habilitada neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

fn run_error_response(request_id: Option<Value>, error: &run::RunError) -> JsonRpcResponse {
    let code = match error {
        run::RunError::Process { .. } => JsonRpcErrorCode::InternalError,
        run::RunError::AlreadyRunning
        | run::RunError::NotRunning
        | run::RunError::NoDefaultCommand { .. } => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

fn lsp_unavailable_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "LSP nao esta habilitado neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

fn lsp_error_response(request_id: Option<Value>, error: &lsp::LspError) -> JsonRpcResponse {
    let code = if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

fn parse_params<T>(
    request_id: Option<&Value>,
    params: Option<&Value>,
    hint: &str,
) -> Result<T, Box<JsonRpcResponse>>
where
    T: DeserializeOwned,
{
    let params = params.cloned().unwrap_or_else(|| json!({}));
    serde_json::from_value::<T>(params).map_err(|error| {
        Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                hint,
                Some(json!({ "error": error.to_string() })),
            ),
        ))
    })
}

fn parse_lsp_position_params(
    root: &Path,
    request_id: Option<&Value>,
    params: Option<&Value>,
    hint: &str,
) -> Result<(PathBuf, LspTextDocumentPositionParams), Box<JsonRpcResponse>> {
    let parsed = parse_params::<LspTextDocumentPositionParams>(request_id, params, hint)?;
    let path = fsops::confine_file(root, Path::new(&parsed.path))
        .map_err(|error| Box::new(fs_error_response(request_id.cloned(), &error)))?;
    Ok((path, parsed))
}

/// Result of handling a request.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestOutcome {
    /// The core should continue reading requests.
    Continue(JsonRpcResponse),
    /// The core should return this response and stop the loop.
    Shutdown(JsonRpcResponse),
}

impl RequestOutcome {
    /// Returns the JSON-RPC response.
    #[must_use]
    pub const fn response(&self) -> &JsonRpcResponse {
        match self {
            Self::Continue(response) | Self::Shutdown(response) => response,
        }
    }

    /// Returns `true` when the core should stop after writing the response.
    #[must_use]
    pub const fn should_shutdown(&self) -> bool {
        matches!(self, Self::Shutdown(_))
    }
}

/// Error returned by core IO loops.
#[derive(Debug)]
pub enum CoreError {
    /// Failed while reading a request.
    Read(io::Error),
    /// Failed while writing a response.
    Write(io::Error),
    /// Failed while serializing a response.
    Serialize(serde_json::Error),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "failed to read IPC request: {error}"),
            Self::Write(error) => write!(formatter, "failed to write IPC response: {error}"),
            Self::Serialize(error) => {
                write!(formatter, "failed to serialize IPC response: {error}")
            }
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read(error) | Self::Write(error) => Some(error),
            Self::Serialize(error) => Some(error),
        }
    }
}

/// Runs a line-delimited JSON-RPC loop over arbitrary IO streams.
///
/// This function is used by `kernwerk-core` over stdin/stdout and by tests over
/// in-memory buffers.
pub fn run_json_lines<R, W>(reader: R, mut writer: W) -> Result<(), CoreError>
where
    R: BufRead,
    W: Write,
{
    let mut core = Core::new();

    for line in reader.lines() {
        let line = line.map_err(CoreError::Read)?;

        if line.trim().is_empty() {
            continue;
        }

        let mut emit_error: Option<CoreError> = None;
        let outcome = {
            let mut emit = |notification: &JsonRpcRequest| {
                if emit_error.is_some() {
                    return;
                }
                emit_error = write_json_line(&mut writer, notification).err();
            };
            core.handle_json_line_streaming(&line, &mut emit)
        };
        if let Some(error) = emit_error {
            return Err(error);
        }

        write_json_line(&mut writer, outcome.response())?;

        if outcome.should_shutdown() {
            break;
        }
    }

    Ok(())
}

fn write_json_line<W, T>(writer: &mut W, payload: &T) -> Result<(), CoreError>
where
    W: Write,
    T: serde::Serialize,
{
    serde_json::to_writer(&mut *writer, payload).map_err(CoreError::Serialize)?;
    writer.write_all(b"\n").map_err(CoreError::Write)?;
    writer.flush().map_err(CoreError::Write)
}

/// Internal event of the stdio loop.
enum LoopEvent {
    /// One request line arrived on stdin.
    Line(String),
    /// An async notification (LSP diagnostics, server status) must be sent.
    Notification(Box<JsonRpcRequest>),
    /// Stdin closed; the core should stop.
    Eof,
}

/// Runs the core over stdin/stdout with support for async notifications.
///
/// A reader thread feeds stdin lines into a channel; LSP reader threads feed
/// notifications into the same channel. The main loop serializes everything
/// to stdout, so responses and events never interleave mid-line.
pub fn run_stdio() -> Result<(), CoreError> {
    let (events, inbox) = std::sync::mpsc::channel::<LoopEvent>();

    let line_events = events.clone();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if line_events.send(LoopEvent::Line(line)).is_err() {
                        return;
                    }
                }
                Err(_error) => break,
            }
        }
        drop(line_events.send(LoopEvent::Eof));
    });

    let (lsp_events, lsp_inbox) = std::sync::mpsc::channel::<JsonRpcRequest>();
    let notification_events = events;
    std::thread::spawn(move || {
        for notification in lsp_inbox {
            if notification_events
                .send(LoopEvent::Notification(Box::new(notification)))
                .is_err()
            {
                return;
            }
        }
    });

    let mut core = Core::new();
    core.enable_lsp(lsp_events);

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    for event in inbox {
        match event {
            LoopEvent::Line(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                let mut emit_error: Option<CoreError> = None;
                let outcome = {
                    let mut emit = |notification: &JsonRpcRequest| {
                        if emit_error.is_some() {
                            return;
                        }
                        emit_error = write_json_line(&mut writer, notification).err();
                    };
                    core.handle_json_line_streaming(&line, &mut emit)
                };
                if let Some(error) = emit_error {
                    return Err(error);
                }

                write_json_line(&mut writer, outcome.response())?;

                if outcome.should_shutdown() {
                    break;
                }
            }
            LoopEvent::Notification(notification) => {
                write_json_line(&mut writer, notification.as_ref())?;
            }
            LoopEvent::Eof => break,
        }
    }

    Ok(())
}

fn command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = core_command_descriptors();
    descriptors.extend(project_command_descriptors());
    descriptors.extend(run_command_descriptors());
    descriptors.extend(lsp_command_descriptors());
    descriptors
}

fn run_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "run.start".to_owned(),
            title: "Run".to_owned(),
            category: "Run".to_owned(),
            description: "Executa o projeto ou um comando no workspace, com saida ao vivo"
                .to_owned(),
            default_shortcut: Some("Shift+F10".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "run.stdin".to_owned(),
            title: "Send Input".to_owned(),
            category: "Run".to_owned(),
            description: "Envia texto para o stdin do processo em execucao".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "run.stop".to_owned(),
            title: "Stop".to_owned(),
            category: "Run".to_owned(),
            description: "Encerra o processo em execucao".to_owned(),
            default_shortcut: Some("Ctrl+F2".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.open".to_owned(),
            title: "Terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Abre o shell do usuario ($SHELL) num PTY na raiz do workspace".to_owned(),
            default_shortcut: Some("Alt+F12".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.input".to_owned(),
            title: "Terminal Input".to_owned(),
            category: "Terminal".to_owned(),
            description: "Envia texto para o shell do terminal aberto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.close".to_owned(),
            title: "Close Terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Fecha a sessao de terminal do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

fn core_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "core.ping".to_owned(),
            title: "Ping Core".to_owned(),
            category: "Core".to_owned(),
            description: "Verifica se o core esta respondendo".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "core.shutdown".to_owned(),
            title: "Shutdown Core".to_owned(),
            category: "Core".to_owned(),
            description: "Solicita encerramento limpo do core".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "command.list".to_owned(),
            title: "List Commands".to_owned(),
            category: "Core".to_owned(),
            description: "Lista comandos registrados no core".to_owned(),
            default_shortcut: Some("Ctrl+Shift+A".to_owned()),
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "tools.detect".to_owned(),
            title: "Detect Tools".to_owned(),
            category: "Tools".to_owned(),
            description: "Detecta ferramentas externas e sugere instalacao quando faltarem"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "tools.status".to_owned(),
            title: "Tools Status".to_owned(),
            category: "Tools".to_owned(),
            description: "Mostra o ultimo status conhecido das ferramentas externas".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
    ]
}

fn project_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = workspace_command_descriptors();
    descriptors.extend(file_command_descriptors());
    descriptors
}

fn workspace_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "workspace.open".to_owned(),
            title: "Open Workspace".to_owned(),
            category: "Workspace".to_owned(),
            description: "Abre uma pasta como workspace e identifica o tipo de projeto".to_owned(),
            default_shortcut: Some("Ctrl+O".to_owned()),
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.browse".to_owned(),
            title: "Browse Workspace Folders".to_owned(),
            category: "Workspace".to_owned(),
            description: "Lista diretorios para o seletor proprio de workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.createFolder".to_owned(),
            title: "New Folder".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria uma pasta pelo seletor proprio de workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.createProject".to_owned(),
            title: "New Project".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria um projeto inicial e abre como workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.status".to_owned(),
            title: "Workspace Status".to_owned(),
            category: "Workspace".to_owned(),
            description: "Mostra o workspace aberto no momento".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.close".to_owned(),
            title: "Close Workspace".to_owned(),
            category: "Workspace".to_owned(),
            description: "Fecha o workspace atual".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

fn file_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "fs.list".to_owned(),
            title: "List Directory".to_owned(),
            category: "Files".to_owned(),
            description: "Lista um diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.read".to_owned(),
            title: "Read File".to_owned(),
            category: "Files".to_owned(),
            description: "Le um arquivo de texto dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.createFile".to_owned(),
            title: "New File".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo arquivo de texto dentro do workspace sem sobrescrever"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.createDirectory".to_owned(),
            title: "New Directory".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo diretorio dentro do workspace sem sobrescrever".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.write".to_owned(),
            title: "Save File".to_owned(),
            category: "Files".to_owned(),
            description: "Salva um arquivo existente dentro do workspace".to_owned(),
            default_shortcut: Some("Ctrl+S".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.rename".to_owned(),
            title: "Rename".to_owned(),
            category: "Files".to_owned(),
            description: "Renomeia ou move um arquivo ou diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.delete".to_owned(),
            title: "Delete".to_owned(),
            category: "Files".to_owned(),
            description: "Remove um arquivo ou diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.search".to_owned(),
            title: "Find in Files".to_owned(),
            category: "Files".to_owned(),
            description: "Busca texto em todos os arquivos do workspace".to_owned(),
            default_shortcut: Some("Ctrl+Shift+F".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.findFiles".to_owned(),
            title: "Find File".to_owned(),
            category: "Files".to_owned(),
            description: "Busca arquivos por nome usando fd".to_owned(),
            default_shortcut: Some("Ctrl+Shift+N".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "build.run".to_owned(),
            title: "Build Project".to_owned(),
            category: "Build".to_owned(),
            description: "Compila o projeto do workspace e emite erros estruturados".to_owned(),
            default_shortcut: Some("Ctrl+F9".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "test.run".to_owned(),
            title: "Run Tests".to_owned(),
            category: "Build".to_owned(),
            description: "Roda os testes do projeto (cargo test / ctest) com resultado por caso"
                .to_owned(),
            default_shortcut: Some("Ctrl+Shift+F9".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "quality.run".to_owned(),
            title: "Analyze (Lint)".to_owned(),
            category: "Build".to_owned(),
            description: "Roda a analise de qualidade (cargo clippy) e lista os avisos".to_owned(),
            default_shortcut: Some("Ctrl+Shift+L".to_owned()),
            requires_workspace: true,
        },
    ]
}

fn lsp_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "lsp.semanticTokens".to_owned(),
            title: "Semantic Highlighting".to_owned(),
            category: "LSP".to_owned(),
            description: "Resolve cores semanticas do arquivo aberto via LSP".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.didChange".to_owned(),
            title: "Sync Editor Buffer".to_owned(),
            category: "LSP".to_owned(),
            description: "Sincroniza o buffer aberto com o servidor LSP gerenciado pelo core"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.definition".to_owned(),
            title: "Go to Definition".to_owned(),
            category: "LSP".to_owned(),
            description: "Resolve a definicao do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Ctrl+B".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.hover".to_owned(),
            title: "Quick Documentation".to_owned(),
            category: "LSP".to_owned(),
            description: "Mostra informacao rapida do simbolo na posicao atual do editor"
                .to_owned(),
            default_shortcut: Some("Ctrl+Q".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.completion".to_owned(),
            title: "Code Completion".to_owned(),
            category: "LSP".to_owned(),
            description: "Lista completions do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Ctrl+Space".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.references".to_owned(),
            title: "Find Usages".to_owned(),
            category: "LSP".to_owned(),
            description: "Lista os usos do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Alt+F7".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.rename".to_owned(),
            title: "Rename Symbol".to_owned(),
            category: "LSP".to_owned(),
            description: "Renomeia o simbolo em todos os arquivos do workspace via LSP".to_owned(),
            default_shortcut: Some("Shift+F6".to_owned()),
            requires_workspace: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use serde_json::{Value, json};

    use super::{Core, RequestOutcome, run_json_lines};
    use crate::tools::ToolDetector;
    use kernwerk_protocol::JsonRpcRequest;

    fn core_with_empty_search_path(test_name: &str) -> Core {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-dispatch-{test_name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Core::with_detector(ToolDetector::with_search_path(dir))
    }

    #[test]
    fn ping_returns_pong() {
        let mut core = Core::new();
        let request = JsonRpcRequest::new(1_i64, "core.ping", Some(json!({})));
        let outcome = core.handle_request(&request);
        let result = outcome.response().result.as_ref().unwrap();

        assert!(!outcome.should_shutdown());
        assert_eq!(result["status"], "ok");
        assert_eq!(result["message"], "pong");
    }

    #[test]
    fn unknown_command_returns_structured_error() {
        let mut core = Core::new();
        let request = JsonRpcRequest::new(9_i64, "missing.command", Some(json!({})));
        let outcome = core.handle_request(&request);
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::MethodNotFound
        );
        assert_eq!(error.details.as_ref().unwrap()["method"], "missing.command");
    }

    #[test]
    fn command_list_includes_lsp_navigation_commands() {
        let mut core = Core::new();
        let request = JsonRpcRequest::new(8_i64, "command.list", Some(json!({})));
        let outcome = core.handle_request(&request);
        let commands = outcome.response().result.as_ref().unwrap()["commands"]
            .as_array()
            .unwrap();
        let ids = commands
            .iter()
            .filter_map(|command| command["id"].as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"workspace.createFolder"));
        assert!(ids.contains(&"workspace.createProject"));
        assert!(ids.contains(&"fs.createFile"));
        assert!(ids.contains(&"fs.createDirectory"));
        assert!(ids.contains(&"fs.rename"));
        assert!(ids.contains(&"fs.delete"));
        assert!(ids.contains(&"fs.findFiles"));
        assert!(ids.contains(&"lsp.didChange"));
        assert!(ids.contains(&"lsp.definition"));
        assert!(ids.contains(&"lsp.hover"));
        assert!(ids.contains(&"lsp.completion"));
        assert!(ids.contains(&"lsp.references"));
        assert!(ids.contains(&"lsp.rename"));
    }

    #[test]
    fn invalid_json_returns_parse_error() {
        let mut core = Core::new();
        let outcome = core.handle_json_line("{not-json");
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(error.code, kernwerk_protocol::JsonRpcErrorCode::ParseError);
    }

    #[test]
    fn shutdown_outcome_is_explicit() {
        let mut core = Core::new();
        let request = JsonRpcRequest::new(2_i64, "core.shutdown", Some(json!({})));
        let outcome = core.handle_request(&request);

        assert!(matches!(outcome, RequestOutcome::Shutdown(_)));
        assert!(outcome.should_shutdown());
    }

    #[test]
    fn tools_detect_returns_structured_status_for_all_known_tools() {
        let mut core = core_with_empty_search_path("detect");
        let request = JsonRpcRequest::new(4_i64, "tools.detect", Some(json!({})));
        let outcome = core.handle_request(&request);
        let result = outcome.response().result.as_ref().unwrap();
        let tools = result["tools"].as_array().unwrap();

        assert_eq!(tools.len(), crate::tools::KNOWN_TOOLS.len());
        assert!(tools.iter().all(|tool| tool["status"] == "missing"));
        assert!(tools[0].get("suggestedInstall").is_none());
    }

    #[test]
    fn tools_status_runs_detection_once_and_reuses_registry() {
        let mut core = core_with_empty_search_path("status");
        let first =
            core.handle_request(&JsonRpcRequest::new(5_i64, "tools.status", Some(json!({}))));
        let second =
            core.handle_request(&JsonRpcRequest::new(6_i64, "tools.status", Some(json!({}))));

        assert_eq!(
            first.response().result.as_ref().unwrap()["tools"],
            second.response().result.as_ref().unwrap()["tools"]
        );
    }

    #[test]
    fn workspace_open_status_close_cycle_works() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-workspace-cycle", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let mut core = core_with_empty_search_path("workspace-cycle");

        let opened = core.handle_request(&JsonRpcRequest::new(
            10_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let opened_result = opened.response().result.as_ref().unwrap();
        assert_eq!(opened_result["kind"], "rustCargo");

        let status = core.handle_request(&JsonRpcRequest::new(
            11_i64,
            "workspace.status",
            Some(json!({})),
        ));
        let status_result = status.response().result.as_ref().unwrap();
        assert_eq!(status_result["workspace"]["kind"], "rustCargo");

        let closed = core.handle_request(&JsonRpcRequest::new(
            12_i64,
            "workspace.close",
            Some(json!({})),
        ));
        let closed_result = closed.response().result.as_ref().unwrap();
        assert_eq!(closed_result["status"], "ok");
        assert!(closed_result["closed"].is_string());

        let after = core.handle_request(&JsonRpcRequest::new(
            13_i64,
            "workspace.status",
            Some(json!({})),
        ));
        let after_result = after.response().result.as_ref().unwrap();
        assert!(after_result["workspace"].is_null());
    }

    #[test]
    fn workspace_open_without_path_returns_invalid_params() {
        let mut core = core_with_empty_search_path("workspace-bad-params");
        let outcome = core.handle_request(&JsonRpcRequest::new(
            14_i64,
            "workspace.open",
            Some(json!({})),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn workspace_browse_returns_directory_entries() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-workspace-browse", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let mut core = core_with_empty_search_path("workspace-browse");

        let outcome = core.handle_request(&JsonRpcRequest::new(
            16_i64,
            "workspace.browse",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let result = outcome.response().result.as_ref().unwrap();
        let entries = result["entries"].as_array().unwrap();

        assert_eq!(
            result["path"],
            dir.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["name"], "src");
    }

    #[test]
    fn workspace_browse_without_path_returns_invalid_params() {
        let mut core = core_with_empty_search_path("workspace-browse-bad-params");
        let outcome = core.handle_request(&JsonRpcRequest::new(
            17_i64,
            "workspace.browse",
            Some(json!({})),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn workspace_create_folder_creates_directory() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-workspace-create-folder", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut core = core_with_empty_search_path("workspace-create-folder");

        let outcome = core.handle_request(&JsonRpcRequest::new(
            18_i64,
            "workspace.createFolder",
            Some(json!({ "parent": dir.to_str().unwrap(), "name": "novo modulo" })),
        ));
        let result = outcome.response().result.as_ref().unwrap();
        let created = std::path::PathBuf::from(result["path"].as_str().unwrap());

        assert!(created.is_dir());
        assert_eq!(created.file_name().unwrap(), "novo modulo");
    }

    #[test]
    fn workspace_create_project_opens_created_cpp_project() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-workspace-create-project", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut core = core_with_empty_search_path("workspace-create-project");

        let outcome = core.handle_request(&JsonRpcRequest::new(
            19_i64,
            "workspace.createProject",
            Some(json!({
                "parent": dir.to_str().unwrap(),
                "name": "demo_cpp",
                "template": "cppCmake",
            })),
        ));
        let result = outcome.response().result.as_ref().unwrap();

        assert_eq!(result["kind"], "cmake");
        assert_eq!(result["name"], "demo_cpp");
        assert!(dir.join("demo_cpp/CMakeLists.txt").is_file());
        assert!(dir.join("demo_cpp/src/main.cpp").is_file());
        assert_eq!(
            core.handle_request(&JsonRpcRequest::new(
                20_i64,
                "workspace.status",
                Some(json!({}))
            ))
            .response()
            .result
            .as_ref()
            .unwrap()["workspace"]["name"],
            "demo_cpp"
        );
    }

    #[test]
    fn workspace_open_with_missing_directory_returns_invalid_params() {
        let mut core = core_with_empty_search_path("workspace-missing-dir");
        let outcome = core.handle_request(&JsonRpcRequest::new(
            15_i64,
            "workspace.open",
            Some(json!({ "path": "/definitely/not/a/real/path" })),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
        assert_eq!(
            error.details.as_ref().unwrap()["path"],
            "/definitely/not/a/real/path"
        );
    }

    #[test]
    fn fs_methods_require_open_workspace() {
        let mut core = core_with_empty_search_path("fs-no-workspace");
        let outcome = core.handle_request(&JsonRpcRequest::new(
            20_i64,
            "fs.list",
            Some(json!({ "path": "/tmp" })),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert_eq!(error.message, "nenhum workspace aberto");
    }

    #[test]
    fn fs_list_read_write_cycle_inside_workspace() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-cycle", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let mut core = core_with_empty_search_path("fs-cycle");

        let opened = core.handle_request(&JsonRpcRequest::new(
            21_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let root = opened.response().result.as_ref().unwrap()["root"]
            .as_str()
            .unwrap()
            .to_owned();

        let listed = core.handle_request(&JsonRpcRequest::new(
            22_i64,
            "fs.list",
            Some(json!({ "path": root })),
        ));
        let entries = listed.response().result.as_ref().unwrap()["entries"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(entries[0]["name"], ".kernwerk");
        assert_eq!(entries[1]["name"], "src");
        assert_eq!(entries[1]["kind"], "directory");

        let file_path = format!("{root}/src/main.rs");
        let read = core.handle_request(&JsonRpcRequest::new(
            23_i64,
            "fs.read",
            Some(json!({ "path": file_path })),
        ));
        assert_eq!(
            read.response().result.as_ref().unwrap()["content"],
            "fn main() {}\n"
        );

        let written = core.handle_request(&JsonRpcRequest::new(
            24_i64,
            "fs.write",
            Some(json!({ "path": file_path, "content": "// editado\n" })),
        ));
        assert_eq!(
            written.response().result.as_ref().unwrap()["bytesWritten"],
            11
        );

        let escape = core.handle_request(&JsonRpcRequest::new(
            25_i64,
            "fs.read",
            Some(json!({ "path": "/etc/hostname" })),
        ));
        let error = escape.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn fs_create_file_creates_once_and_stays_inside_workspace() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-create-file", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let mut core = core_with_empty_search_path("fs-create-file");

        let opened = core.handle_request(&JsonRpcRequest::new(
            30_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let root = opened.response().result.as_ref().unwrap()["root"]
            .as_str()
            .unwrap()
            .to_owned();
        let file_path = format!("{root}/src/lib.rs");

        let created = core.handle_request(&JsonRpcRequest::new(
            31_i64,
            "fs.createFile",
            Some(json!({ "path": file_path, "content": "pub fn answer() -> u8 { 42 }\n" })),
        ));
        let result = created.response().result.as_ref().unwrap();
        assert_eq!(result["path"], format!("{root}/src/lib.rs"));
        assert_eq!(result["bytesWritten"], 29);

        let duplicate = core.handle_request(&JsonRpcRequest::new(
            32_i64,
            "fs.createFile",
            Some(json!({ "path": format!("{root}/src/lib.rs") })),
        ));
        assert_eq!(
            duplicate.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );

        let escape = core.handle_request(&JsonRpcRequest::new(
            33_i64,
            "fs.createFile",
            Some(json!({ "path": format!("{root}/../escape.rs") })),
        ));
        assert_eq!(
            escape.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn fs_create_directory_creates_once_and_stays_inside_workspace() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-create-directory", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let mut core = core_with_empty_search_path("fs-create-directory");

        let opened = core.handle_request(&JsonRpcRequest::new(
            34_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let root = opened.response().result.as_ref().unwrap()["root"]
            .as_str()
            .unwrap()
            .to_owned();
        let directory_path = format!("{root}/src/features");

        let created = core.handle_request(&JsonRpcRequest::new(
            35_i64,
            "fs.createDirectory",
            Some(json!({ "path": directory_path })),
        ));
        assert_eq!(
            created.response().result.as_ref().unwrap()["path"],
            format!("{root}/src/features")
        );

        let duplicate = core.handle_request(&JsonRpcRequest::new(
            36_i64,
            "fs.createDirectory",
            Some(json!({ "path": format!("{root}/src/features") })),
        ));
        assert_eq!(
            duplicate.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );

        let escape = core.handle_request(&JsonRpcRequest::new(
            37_i64,
            "fs.createDirectory",
            Some(json!({ "path": format!("{root}/../outside") })),
        ));
        assert_eq!(
            escape.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn fs_rename_and_delete_stay_inside_workspace() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-rename-delete", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(dir.join("src/old.rs"), "fn old() {}\n").unwrap();
        let mut core = core_with_empty_search_path("fs-rename-delete");

        let opened = core.handle_request(&JsonRpcRequest::new(
            40_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        let root = opened.response().result.as_ref().unwrap()["root"]
            .as_str()
            .unwrap()
            .to_owned();

        let renamed = core.handle_request(&JsonRpcRequest::new(
            41_i64,
            "fs.rename",
            Some(json!({
                "from": format!("{root}/src/old.rs"),
                "to": format!("{root}/src/new.rs"),
            })),
        ));
        let result = renamed.response().result.as_ref().unwrap();
        assert_eq!(result["from"], format!("{root}/src/old.rs"));
        assert_eq!(result["to"], format!("{root}/src/new.rs"));
        assert!(std::path::Path::new(&format!("{root}/src/new.rs")).exists());

        let escape = core.handle_request(&JsonRpcRequest::new(
            42_i64,
            "fs.rename",
            Some(json!({
                "from": format!("{root}/src/new.rs"),
                "to": format!("{root}/../escape.rs"),
            })),
        ));
        assert_eq!(
            escape.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );

        let deleted = core.handle_request(&JsonRpcRequest::new(
            43_i64,
            "fs.delete",
            Some(json!({ "path": format!("{root}/src/new.rs") })),
        ));
        assert_eq!(
            deleted.response().result.as_ref().unwrap()["path"],
            format!("{root}/src/new.rs")
        );
        assert!(!std::path::Path::new(&format!("{root}/src/new.rs")).exists());

        let escape_delete = core.handle_request(&JsonRpcRequest::new(
            44_i64,
            "fs.delete",
            Some(json!({ "path": "/etc/hostname" })),
        ));
        assert_eq!(
            escape_delete.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn fs_search_finds_matches_and_requires_workspace() {
        let mut core = core_with_empty_search_path("fs-search-no-workspace");
        let denied = core.handle_request(&JsonRpcRequest::new(
            26_i64,
            "fs.search",
            Some(json!({ "query": "main" })),
        ));
        assert!(denied.response().error.is_some());

        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-search", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(
            dir.join("src/main.rs"),
            "fn main() {\n    Encontrar();\n}\n",
        )
        .unwrap();
        let opened = core.handle_request(&JsonRpcRequest::new(
            27_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let searched = core.handle_request(&JsonRpcRequest::new(
            28_i64,
            "fs.search",
            Some(json!({ "query": "encontrar" })),
        ));
        let result = searched.response().result.as_ref().unwrap();
        assert_eq!(result["truncated"], false);
        assert_eq!(result["matches"][0]["path"], "src/main.rs");
        assert_eq!(result["matches"][0]["line"], 2);
        assert_eq!(result["matches"][0]["column"], 5);

        let empty = core.handle_request(&JsonRpcRequest::new(
            29_i64,
            "fs.search",
            Some(json!({ "query": "" })),
        ));
        assert!(empty.response().error.is_some());
    }

    #[test]
    fn fs_find_files_requires_workspace_and_non_empty_query() {
        let mut core = core_with_empty_search_path("fs-find-files-no-workspace");
        let denied = core.handle_request(&JsonRpcRequest::new(
            38_i64,
            "fs.findFiles",
            Some(json!({ "query": "main" })),
        ));
        assert_eq!(
            denied.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );

        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-fs-find-files", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let opened = core.handle_request(&JsonRpcRequest::new(
            39_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let empty = core.handle_request(&JsonRpcRequest::new(
            40_i64,
            "fs.findFiles",
            Some(json!({ "query": "" })),
        ));
        assert_eq!(
            empty.response().error.as_ref().unwrap().code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn run_start_requires_workspace_and_enabled_manager() {
        let mut core = core_with_empty_search_path("run-no-workspace");
        let denied = core.handle_request(&JsonRpcRequest::new(40_i64, "run.start", None));
        assert!(denied.response().error.is_some());

        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-run-start", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let opened = core.handle_request(&JsonRpcRequest::new(
            41_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let unavailable = core.handle_request(&JsonRpcRequest::new(42_i64, "run.start", None));
        let error = unavailable.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InternalError
        );
    }

    #[test]
    fn run_start_executes_command_and_emits_events() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut core = core_with_empty_search_path("run-e2e");
        core.enable_lsp(sender);

        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-run-e2e", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let opened = core.handle_request(&JsonRpcRequest::new(
            43_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let no_default = core.handle_request(&JsonRpcRequest::new(44_i64, "run.start", None));
        assert!(no_default.response().error.is_some());

        let started = core.handle_request(&JsonRpcRequest::new(
            45_i64,
            "run.start",
            Some(json!({ "command": "printf 'executado\\n'" })),
        ));
        let result = started.response().result.as_ref().unwrap();
        assert_eq!(result["command"], "printf 'executado\\n'");

        let mut saw_output = false;
        loop {
            let event = receiver
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("evento run dentro do timeout");
            if event.method == "event.run.output" {
                saw_output = event.params.as_ref().unwrap()["line"] == "executado";
            }
            if event.method == "event.run.finished" {
                assert_eq!(event.params.as_ref().unwrap()["success"], true);
                break;
            }
        }
        assert!(saw_output);

        let stopped = core.handle_request(&JsonRpcRequest::new(46_i64, "run.stop", None));
        assert!(stopped.response().error.is_some());
    }

    #[test]
    fn build_run_requires_open_workspace() {
        let mut core = core_with_empty_search_path("build-no-workspace");
        let request = JsonRpcRequest::new(30_i64, "build.run", Some(json!({})));
        let outcome = core.handle_request_streaming(&request, &mut |_| {});
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert_eq!(error.message, "nenhum workspace aberto");
    }

    #[test]
    fn lsp_did_change_requires_open_workspace() {
        let mut core = core_with_empty_search_path("lsp-no-workspace");
        let request = JsonRpcRequest::new(
            33_i64,
            "lsp.didChange",
            Some(json!({ "path": "/tmp/main.rs", "content": "fn main() {}\n" })),
        );
        let outcome = core.handle_request(&request);
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert_eq!(error.message, "nenhum workspace aberto");
    }

    #[test]
    fn lsp_navigation_requires_open_workspace() {
        let mut core = core_with_empty_search_path("lsp-navigation-no-workspace");
        for method in [
            "lsp.definition",
            "lsp.hover",
            "lsp.completion",
            "lsp.references",
        ] {
            let request = JsonRpcRequest::new(
                36_i64,
                method,
                Some(json!({
                    "path": "/tmp/main.rs",
                    "content": "fn main() {}\n",
                    "line": 1,
                    "column": 1,
                })),
            );
            let outcome = core.handle_request(&request);
            let error = outcome.response().error.as_ref().unwrap();

            assert_eq!(
                error.code,
                kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
            );
            assert_eq!(error.message, "nenhum workspace aberto");
        }
    }

    #[test]
    fn lsp_rename_requires_open_workspace_and_non_empty_name() {
        let mut core = core_with_empty_search_path("lsp-rename-no-workspace");
        let request = JsonRpcRequest::new(
            40_i64,
            "lsp.rename",
            Some(json!({
                "path": "/tmp/main.rs",
                "content": "fn main() {}\n",
                "line": 1,
                "column": 4,
                "newName": "start",
            })),
        );
        let outcome = core.handle_request(&request);
        let error = outcome.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert_eq!(error.message, "nenhum workspace aberto");

        let workspace = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-lsp-rename-empty-name", std::process::id()));
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();
        let opened = core.handle_request(&JsonRpcRequest::new(
            41_i64,
            "workspace.open",
            Some(json!({ "path": workspace.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let empty_name = core.handle_request(&JsonRpcRequest::new(
            42_i64,
            "lsp.rename",
            Some(json!({
                "path": workspace.join("src/main.rs").to_str().unwrap(),
                "content": "fn main() {}\n",
                "line": 1,
                "column": 4,
                "newName": "   ",
            })),
        ));
        let empty_error = empty_name.response().error.as_ref().unwrap();
        assert_eq!(
            empty_error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn lsp_navigation_reports_unavailable_without_lsp_manager() {
        let workspace = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-lsp-unavailable", std::process::id()));
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(workspace.join("src/main.rs"), "fn main() {}\n").unwrap();
        let mut core = core_with_empty_search_path("lsp-unavailable");

        let opened = core.handle_request(&JsonRpcRequest::new(
            37_i64,
            "workspace.open",
            Some(json!({ "path": workspace.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let file_path = workspace.join("src/main.rs");
        let outcome = core.handle_request(&JsonRpcRequest::new(
            38_i64,
            "lsp.definition",
            Some(json!({
                "path": file_path.to_str().unwrap(),
                "content": "fn main() {}\n",
                "line": 1,
                "column": 1,
            })),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InternalError
        );
        assert_eq!(error.message, "LSP nao esta habilitado neste loop do core");
    }

    #[test]
    fn lsp_did_change_rejects_paths_outside_workspace() {
        let workspace = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-lsp-workspace", std::process::id()));
        let outside = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-lsp-outside", std::process::id()));
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(workspace.join("Cargo.toml"), "[package]\n").unwrap();
        std::fs::write(outside.join("main.rs"), "fn main() {}\n").unwrap();
        let mut core = core_with_empty_search_path("lsp-outside");

        let opened = core.handle_request(&JsonRpcRequest::new(
            34_i64,
            "workspace.open",
            Some(json!({ "path": workspace.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let outcome = core.handle_request(&JsonRpcRequest::new(
            35_i64,
            "lsp.didChange",
            Some(json!({
                "path": outside.join("main.rs").to_str().unwrap(),
                "content": "fn main() {}\n",
            })),
        ));
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidParams
        );
    }

    #[test]
    fn build_run_rejects_unsupported_project_kind() {
        let dir = std::env::temp_dir()
            .join("kernwerk-core-tests")
            .join(format!("{}-build-unsupported", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();
        let mut core = core_with_empty_search_path("build-unsupported");

        let opened = core.handle_request(&JsonRpcRequest::new(
            31_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());

        let mut events = Vec::new();
        let request = JsonRpcRequest::new(32_i64, "build.run", Some(json!({})));
        let outcome = core.handle_request_streaming(&request, &mut |notification| {
            events.push(notification.method.clone());
        });
        let error = outcome.response().error.as_ref().unwrap();

        assert_eq!(
            error.code,
            kernwerk_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert!(error.message.contains("python"));
        assert!(events.is_empty());
    }

    #[test]
    fn stdio_loop_writes_one_response_per_line_and_stops() {
        let input = concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"core.ping","params":{}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"core.shutdown","params":{}}"#,
            "\n",
            r#"{"jsonrpc":"2.0","id":3,"method":"core.ping","params":{}}"#,
            "\n",
        );
        let reader = BufReader::new(Cursor::new(input.as_bytes()));
        let mut output = Vec::new();

        run_json_lines(reader, &mut output).unwrap();

        let text = String::from_utf8(output).unwrap();
        let responses = text
            .lines()
            .map(serde_json::from_str::<Value>)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0]["result"]["message"], "pong");
        assert_eq!(responses[1]["result"]["message"], "shutdown requested");
    }
}
