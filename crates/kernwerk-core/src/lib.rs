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
pub mod runtime;
pub mod terminal;
pub mod test;
pub mod tools;
pub mod workspace;
pub use runtime::{run_json_lines, run_stdio};

use std::{error::Error, fmt, io, path::PathBuf};

use kernwerk_protocol::{
    CorePingResult, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, ToolInfo,
    ToolsDetectResult, WorkspaceInfo, WorkspaceStatusResult,
};
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

#[cfg(test)]
mod tests;
