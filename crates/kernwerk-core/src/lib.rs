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
    path::PathBuf,
};

use kernwerk_protocol::{
    BuildRunResult, CorePingResult, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest,
    JsonRpcResponse, QualityRunResult, TestRunResult, ToolInfo, ToolsDetectResult, WorkspaceInfo,
    WorkspaceStatusResult,
};
use serde_json::{Value, json};

use crate::rpc::no_workspace_response;

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

    /// Canonical workspace root path, when a workspace is open. Shared by every
    /// domain handler module, so it stays on `Core` here in `lib.rs`.
    fn workspace_root(&self) -> Option<PathBuf> {
        self.workspace
            .as_ref()
            .map(|workspace| PathBuf::from(&workspace.root))
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
