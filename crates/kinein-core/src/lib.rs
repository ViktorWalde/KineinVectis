//! Rust core for Kinein Vectis.
//!
//! The core owns command execution and external tool orchestration. The UI must
//! communicate through IPC and must not call build tools or language servers
//! directly.

#![forbid(unsafe_code)]

pub mod build;
pub mod cargo;
pub mod cmake;
pub mod commands;
pub mod dap;
pub mod db;
pub mod format;
pub mod fsops;
pub mod fswatch;
pub mod git;
pub mod handlers;
pub mod jobs;
pub mod lang;
pub mod lsp;
pub mod process;
pub mod rpc;
pub mod run;
pub mod runconfig;
pub mod runtime;
pub mod settings;
pub mod terminal;
pub mod test;
pub mod tools;
pub mod workspace;
pub use runtime::{run_json_lines, run_stdio};

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use kinein_protocol::{
    CorePingResult, JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest,
    JsonRpcResponse, ToolInfo, ToolStatus, ToolsDetectResult, WorkspaceInfo, WorkspaceStatusResult,
};
use serde_json::{Value, json};

use crate::tools::{KNOWN_TOOLS, ToolDetector};

/// Stateful core runtime.
#[derive(Debug, Default)]
pub struct Core {
    detector: ToolDetector,
    tool_registry: Arc<Mutex<Option<Vec<ToolInfo>>>>,
    workspace: Option<WorkspaceInfo>,
    fswatch: Option<fswatch::WorkspaceWatcher>,
    syntax: lang::SyntaxTreeService,
    events: Option<lsp::EventSender>,
    lsp: Option<lsp::LspManager>,
    workspace_edits: lsp::WorkspaceEditTransactions,
    run: Option<run::RunManager>,
    debug: Option<dap::DebugManager>,
    terminal: Option<terminal::TerminalManager>,
    jobs: Option<jobs::JobManager>,
    /// Store local de rascunhos (autosave), aberta por-workspace (docs/23).
    drafts: Option<db::DraftStore>,
    /// Persistência local ligada (só no core completo, não em testes leves).
    persistence_enabled: bool,
}

impl Core {
    /// Creates a new core runtime that detects tools through `PATH`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_detector(ToolDetector::from_environment())
    }

    /// Creates a core runtime with an injected tool detector.
    #[must_use]
    pub fn with_detector(detector: ToolDetector) -> Self {
        Self {
            detector,
            tool_registry: Arc::new(Mutex::new(None)),
            workspace: None,
            fswatch: None,
            syntax: lang::SyntaxTreeService::default(),
            events: None,
            lsp: None,
            workspace_edits: lsp::WorkspaceEditTransactions::default(),
            run: None,
            debug: None,
            terminal: None,
            jobs: None,
            drafts: None,
            persistence_enabled: false,
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
        self.debug = Some(dap::DebugManager::new(events.clone()));
        self.jobs = Some(jobs::JobManager::new(events.clone()));
        self.terminal = Some(terminal::TerminalManager::new(events.clone()));
        self.events = Some(events);
        // M-S1: liga a persistência local (rascunhos em SQLite) — só no core
        // completo; a store real abre quando um workspace é aberto.
        self.persistence_enabled = true;
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
                    "expected": kinein_protocol::JSON_RPC_VERSION,
                    "received": request.jsonrpc.as_str(),
                })),
            );
            return RequestOutcome::Continue(JsonRpcResponse::failure(request_id, error));
        }

        if request.id.is_none() {
            let error = JsonRpcError::new(
                JsonRpcErrorCode::InvalidRequest,
                "Kinein Vectis commands must include a request id",
                Some(json!({ "method": request.method.as_str() })),
            );
            return RequestOutcome::Continue(JsonRpcResponse::failure(Some(Value::Null), error));
        }

        match request.method.as_str() {
            "core.ping" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!(CorePingResult::default()),
            )),
            "core.shutdown" => {
                // M4.3b: cancela jobs vivos já no shutdown (sinal pronto +
                // entrega os event.job.* enquanto o canal existe); o drain
                // final fica no Drop do JobManager, após a resposta.
                if let Some(jobs) = self.jobs.as_ref() {
                    jobs.cancel_all();
                }
                RequestOutcome::Shutdown(JsonRpcResponse::success(
                    request_id,
                    json!({
                        "status": "ok",
                        "message": "shutdown requested",
                    }),
                ))
            }
            "command.list" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!({ "commands": commands::command_descriptors() }),
            )),
            "tools.detect" => {
                let tools = self.detector.detect_all();
                set_tool_registry(&self.tool_registry, tools.clone());
                RequestOutcome::Continue(JsonRpcResponse::success(
                    request_id,
                    json!(ToolsDetectResult { tools }),
                ))
            }
            "tools.status" => {
                let tools = self.tool_registry_snapshot().unwrap_or_else(|| {
                    let tools = self.detector.detect_all();
                    set_tool_registry(&self.tool_registry, tools.clone());
                    tools
                });
                RequestOutcome::Continue(JsonRpcResponse::success(
                    request_id,
                    json!(ToolsDetectResult { tools }),
                ))
            }
            "environment.scan" => {
                RequestOutcome::Continue(self.environment_scan_response(request_id))
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
            "workspace.saveSession" => RequestOutcome::Continue(
                self.save_session_response(request_id, request.params.as_ref()),
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
            "build.run" => RequestOutcome::Continue(self.build_run_response(request_id)),
            "quality.run" => RequestOutcome::Continue(self.quality_run_response(request_id)),
            "test.run" => RequestOutcome::Continue(
                self.test_run_response(request_id, request.params.as_ref()),
            ),
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
            .or_else(|| self.recent_workspace_response(method, request_id.clone(), params))
            .or_else(|| self.ai_request_response(method, request_id.clone(), params))
            .or_else(|| self.cargo_request_response(method, request_id.clone(), params))
            .or_else(|| self.git_request_response(method, request_id.clone(), params))
            .or_else(|| self.runconfig_request_response(method, request_id.clone(), params))
            .or_else(|| self.settings_request_response(method, request_id.clone(), params))
            .or_else(|| self.cmake_request_response(method, request_id.clone(), params))
            .or_else(|| self.format_request_response(method, request_id.clone(), params))
            .or_else(|| self.run_request_response(method, request_id.clone(), params))
            .or_else(|| self.debug_request_response(method, request_id.clone(), params))
            .or_else(|| self.terminal_request_response(method, request_id.clone(), params))
            .or_else(|| self.syntax_request_response(method, request_id.clone(), params))
            .or_else(|| self.lsp_request_response(method, request_id.clone(), params))
            .or_else(|| self.jobs_request_response(method, request_id.clone(), params))
            .or_else(|| self.draft_request_response(method, request_id.clone(), params))
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

    /// Replaces the watcher when a workspace opens. Failure is reported as an
    /// async warning, while compare-before-save remains active as the final
    /// protection against silent overwrites.
    fn reset_workspace_watcher(&mut self, root: &Path) {
        self.fswatch = None;
        let Some(events) = self.events.as_ref() else {
            return;
        };
        match fswatch::WorkspaceWatcher::new(root, events.clone()) {
            Ok(watcher) => self.fswatch = Some(watcher),
            Err(error) => drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!({ "message": error.to_string() })),
            ))),
        }
    }

    /// Lazily observes a directory reached by the explorer or editor.
    fn watch_workspace_directory(&mut self, directory: &Path) {
        let error = self
            .fswatch
            .as_mut()
            .and_then(|watcher| watcher.watch_directory(directory).err());
        if let (Some(error), Some(events)) = (error, self.events.as_ref()) {
            drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!({ "message": error.to_string() })),
            )));
        }
    }

    fn tool_registry_snapshot(&self) -> Option<Vec<ToolInfo>> {
        self.tool_registry
            .lock()
            .ok()
            .and_then(|registry| registry.clone())
    }

    fn environment_scan_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": "environment.scan" })),
                ),
            );
        };

        let detector = self.detector.clone();
        let registry = Arc::clone(&self.tool_registry);
        let job_id = jobs.spawn(
            "environment.scan",
            "Environment Scan",
            JobRisk::Low,
            false,
            move |ctx| {
                let total = KNOWN_TOOLS.len();
                ctx.emit_event(
                    "event.environment.started",
                    json!({ "jobId": ctx.id(), "tools": total }),
                );
                ctx.report_progress(0.0, Some("iniciando scan de ambiente"));

                let mut tools = Vec::with_capacity(total);
                for (index, spec) in KNOWN_TOOLS.iter().enumerate() {
                    let info = detector.detect(spec);
                    ctx.emit_output(&format!(
                        "{}: {}",
                        info.display_name,
                        tool_status_label(info.status)
                    ));
                    ctx.emit_event(
                        "event.environment.tool",
                        json!({ "jobId": ctx.id(), "tool": info.clone() }),
                    );
                    tools.push(info);
                    ctx.report_progress(
                        progress_fraction(index + 1, total),
                        Some(spec.display_name),
                    );
                }

                set_tool_registry(&registry, tools.clone());
                let summary = ToolScanSummary::from_tools(&tools);
                ctx.emit_event(
                    "event.environment.finished",
                    json!({
                        "jobId": ctx.id(),
                        "success": true,
                        "total": summary.total,
                        "detected": summary.detected,
                        "missing": summary.missing,
                        "failed": summary.failed,
                        "tools": tools,
                    }),
                );

                jobs::JobOutcome::Success
            },
        );

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    /// Parses and handles a single line-delimited JSON-RPC request.
    ///
    /// Long-running methods (`build.run`, `quality.run`, `test.run`) return a
    /// `jobId` immediately and stream their progress as async `event.*`
    /// notifications through the core's event channel, so there is no separate
    /// synchronous-streaming entry point.
    #[must_use]
    pub fn handle_json_line(&mut self, line: &str) -> RequestOutcome {
        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => self.handle_request(&request),
            Err(error) => {
                let protocol_error = JsonRpcError::new(
                    JsonRpcErrorCode::ParseError,
                    "Input line is not a valid Kinein Vectis JSON-RPC request",
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

fn set_tool_registry(registry: &Arc<Mutex<Option<Vec<ToolInfo>>>>, tools: Vec<ToolInfo>) {
    if let Ok(mut slot) = registry.lock() {
        *slot = Some(tools);
    }
}

const fn tool_status_label(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::NotConfigured => "notConfigured",
        ToolStatus::Missing => "missing",
        ToolStatus::Detected => "detected",
        ToolStatus::Ready => "ready",
        ToolStatus::Running => "running",
        ToolStatus::Failed => "failed",
        ToolStatus::Disabled => "disabled",
    }
}

fn progress_fraction(done: usize, total: usize) -> f64 {
    let done = u32::try_from(done).unwrap_or(u32::MAX);
    let total = u32::try_from(total.max(1)).unwrap_or(u32::MAX);
    f64::from(done) / f64::from(total)
}

struct ToolScanSummary {
    total: u64,
    detected: u64,
    missing: u64,
    failed: u64,
}

impl ToolScanSummary {
    fn from_tools(tools: &[ToolInfo]) -> Self {
        let mut summary = Self {
            total: tools.len() as u64,
            detected: 0,
            missing: 0,
            failed: 0,
        };
        for tool in tools {
            match tool.status {
                ToolStatus::Detected | ToolStatus::Ready => summary.detected += 1,
                ToolStatus::Missing | ToolStatus::NotConfigured => summary.missing += 1,
                ToolStatus::Failed => summary.failed += 1,
                ToolStatus::Running | ToolStatus::Disabled => {}
            }
        }
        summary
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
