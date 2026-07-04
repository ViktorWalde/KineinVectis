//! Rust core for Kernwerk Studio.
//!
//! The core owns command execution and external tool orchestration. The UI must
//! communicate through IPC and must not call build tools or language servers
//! directly.

#![forbid(unsafe_code)]

pub mod build;
pub mod commands;
pub mod fsops;
pub mod handlers;
pub mod lsp;
pub mod process;
pub mod rpc;
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
    BuildRunResult, CorePingResult, FsWriteParams, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest,
    JsonRpcResponse, LspCompletionResult, LspDefinitionResult, LspHoverResult, LspReferenceItem,
    LspReferencesResult, LspRenameParams, LspRenameResult, LspSemanticTokensResult,
    QualityRunResult, RunStartParams, RunStartResult, RunStdinParams, TerminalInputParams,
    TerminalOpenResult, TestRunResult, ToolInfo, ToolsDetectResult, WorkspaceInfo,
    WorkspaceStatusResult,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, lsp_error_response, lsp_unavailable_response, no_workspace_response,
    parse_lsp_position_params, parse_params, run_error_response, run_unavailable_response,
    terminal_error_response, terminal_unavailable_response,
};

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
                json!({ "commands": commands::command_descriptors() }),
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

    fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace
            .as_ref()
            .map(|workspace| PathBuf::from(&workspace.root))
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

#[cfg(test)]
mod tests;
