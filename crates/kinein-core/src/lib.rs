//! Rust core for Kinein Vectis.
//!
//! The core owns command execution and external tool orchestration. The UI must
//! communicate through IPC and must not call build tools or language servers
//! directly.

#![forbid(unsafe_code)]

pub mod build;
pub mod cargo;
pub mod cdb;
pub mod cmake;
pub mod commands;
pub mod configaction;
pub mod container;
pub mod dap;
pub mod datasource;
pub mod db;
pub mod format;
pub mod fsops;
pub mod fswatch;
pub mod git;
pub mod grafana;
pub mod handlers;
pub mod index;
pub mod jobs;
pub mod lang;
pub mod library;
pub mod lsp;
pub mod probe;
pub mod process;
pub mod project;
pub mod rpc;
pub mod run;
pub mod runconfig;
pub mod runtime;
pub mod serial;
pub mod settings;
pub mod setup;
pub mod size;
pub mod terminal;
pub mod test;
pub mod toolchain;
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
    CorePingResult, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse, ToolInfo,
    WorkspaceInfo, WorkspaceStatusResult,
};
use serde_json::{Value, json};

use crate::tools::ToolDetector;

/// Stateful core runtime.
#[derive(Debug, Default)]
pub struct Core {
    detector: ToolDetector,
    tool_registry: Arc<Mutex<Option<Vec<ToolInfo>>>>,
    workspace: Option<WorkspaceInfo>,
    fswatch: Option<fswatch::WorkspaceWatcher>,
    syntax: lang::SyntaxTreeService,
    /// O indice do projeto inteiro (pilar 0 do roadmaps/42): construido por um
    /// job, consultado pelo loop; o extrator reindexa o que o watcher trouxe.
    index: Arc<Mutex<index::ProjectIndex>>,
    extractor: lang::extract::SymbolExtractor,
    events: Option<lsp::EventSender>,
    lsp: Option<lsp::LspManager>,
    workspace_edits: lsp::WorkspaceEditTransactions,
    run: Option<run::RunManager>,
    debug: Option<dap::DebugManager>,
    terminal: Option<terminal::TerminalManager>,
    jobs: Option<jobs::JobManager>,
    /// Store local de rascunhos (autosave), aberta por-workspace (docs/seguranca/23).
    drafts: Option<db::DraftStore>,
    /// Raiz do estado GLOBAL quando a persistência está ligada; `None` — o
    /// padrão — é persistência DESLIGADA (ver [`Core::enable_persistence`]).
    global_storage: Option<PathBuf>,
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
            index: Arc::new(Mutex::new(index::ProjectIndex::default())),
            extractor: lang::extract::SymbolExtractor::default(),
            events: None,
            lsp: None,
            workspace_edits: lsp::WorkspaceEditTransactions::default(),
            run: None,
            debug: None,
            terminal: None,
            jobs: None,
            drafts: None,
            global_storage: None,
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
    }

    /// Aponta uma linguagem para outro executavel de language server.
    ///
    /// Devolve `false` quando o LSP nao esta habilitado neste loop ou quando a
    /// linguagem nao existe na tabela. Ver
    /// [`lsp::LspManager::use_server_command`] para o porque: e a costura que
    /// permite ao gate observar o que o core FALA com um servidor.
    pub fn use_language_server_command(
        &mut self,
        language: &str,
        command: &str,
        args: &[&str],
    ) -> bool {
        self.lsp
            .as_mut()
            .is_some_and(|lsp| lsp.use_server_command(language, command, args))
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

        let params = request.params.as_ref();
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
            "tools.detect" => RequestOutcome::Continue(self.tools_detect_response(request_id)),
            "tools.status" => RequestOutcome::Continue(self.tools_status_response(request_id)),
            "environment.scan" => {
                RequestOutcome::Continue(self.environment_scan_response(request_id))
            }
            "workspace.open" => {
                RequestOutcome::Continue(self.open_workspace_response(request_id, params))
            }
            "workspace.browse" => {
                RequestOutcome::Continue(Self::browse_workspace_response(request_id, params))
            }
            "workspace.createFolder" => {
                RequestOutcome::Continue(Self::create_workspace_folder_response(request_id, params))
            }
            "workspace.createProject" => {
                RequestOutcome::Continue(self.create_workspace_project_response(request_id, params))
            }
            "workspace.saveSession" => {
                RequestOutcome::Continue(self.save_session_response(request_id, params))
            }
            "workspace.status" => RequestOutcome::Continue(JsonRpcResponse::success(
                request_id,
                json!(WorkspaceStatusResult {
                    workspace: self.workspace.clone(),
                }),
            )),
            "workspace.close" => {
                RequestOutcome::Continue(self.close_workspace_response(request_id))
            }
            "build.run" => RequestOutcome::Continue(self.build_run_response(request_id, params)),
            "build.size" => RequestOutcome::Continue(self.build_size_response(request_id, params)),
            "quality.run" => {
                RequestOutcome::Continue(self.quality_run_response(request_id, params))
            }
            "test.run" => RequestOutcome::Continue(self.test_run_response(request_id, params)),
            method => {
                RequestOutcome::Continue(self.service_request_response(method, request_id, params))
            }
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
            .or_else(|| self.cargo_request_response(method, request_id.clone(), params))
            .or_else(|| self.git_request_response(method, request_id.clone(), params))
            .or_else(|| self.runconfig_request_response(method, request_id.clone(), params))
            .or_else(|| self.settings_request_response(method, request_id.clone(), params))
            .or_else(|| self.cmake_request_response(method, request_id.clone(), params))
            .or_else(|| self.configaction_request_response(method, request_id.clone(), params))
            .or_else(|| self.toolchain_request_response(method, request_id.clone(), params))
            .or_else(|| self.format_request_response(method, request_id.clone(), params))
            .or_else(|| self.run_request_response(method, request_id.clone(), params))
            .or_else(|| self.debug_request_response(method, request_id.clone(), params))
            .or_else(|| self.terminal_request_response(method, request_id.clone(), params))
            .or_else(|| self.syntax_request_response(method, request_id.clone(), params))
            .or_else(|| self.lsp_request_response(method, request_id.clone(), params))
            .or_else(|| self.library_request_response(method, request_id.clone(), params))
            .or_else(|| self.setup_request_response(method, request_id.clone(), params))
            .or_else(|| self.datasource_request_response(method, request_id.clone(), params))
            .or_else(|| self.grafana_request_response(method, request_id.clone(), params))
            .or_else(|| self.probe_request_response(method, request_id.clone(), params))
            .or_else(|| self.container_request_response(method, request_id.clone(), params))
            .or_else(|| self.project_request_response(method, request_id.clone(), params))
            .or_else(|| self.index_request_response(method, request_id.clone(), params))
            .or_else(|| self.serial_request_response(method, request_id.clone(), params))
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

    /// Lazily observes a directory reached by the explorer, the editor — or
    /// the index, that registers every folder it walked. Returns `false` when
    /// the watcher refused (the error is reported once, here).
    fn watch_workspace_directory(&mut self, directory: &Path) -> bool {
        let error = self
            .fswatch
            .as_mut()
            .and_then(|watcher| watcher.watch_directory(directory).err());
        let Some(error) = error else {
            return true;
        };
        if let Some(events) = self.events.as_ref() {
            drop(events.send(JsonRpcRequest::notification(
                "event.fs.watchError",
                Some(json!({ "message": error.to_string() })),
            )));
        }
        false
    }

    /// Deixa o core reagir a um evento assincrono ANTES de ele ir para a UI.
    ///
    /// E o irmao do [`Self::handle_request`]: aquele roteia PEDIDO, este roteia
    /// FATO. Existe porque um job roda em thread propria e nao alcanca o
    /// `Core` (`arquitetura/04` §3) — mas o evento que ele emite volta ao loop
    /// principal, que e o dono do estado. Reagir aqui evita um segundo caminho
    /// (um `Arc<Atomic…>` compartilhado) para o mesmo fato.
    ///
    /// Silencioso de proposito: um evento sem reacao registrada nao e erro.
    pub fn observe_notification(&mut self, notification: &JsonRpcRequest) {
        // O modelo do projeto (pilar 0 do roadmaps/42) muda quando o build
        // muda: configure escreve a CDB e a file-api, build escreve os
        // artefatos. Recomputar aqui e' o que faz a tela SEGUIR o modelo.
        if matches!(
            notification.method.as_str(),
            "event.build.finished" | "event.cmake.finished"
        ) {
            self.emit_project_changed();
        }
        // O contexto de compilador segue a CDB: o configure a reescreve.
        if notification.method == "event.cmake.finished" {
            self.reload_index_context();
        }
        // O indice terminou (em job): as pastas que ele caminhou entram no
        // watcher, para o incremento alcancar o projeto INTEIRO.
        if notification.method == "event.index.finished" {
            self.watch_index_folders();
        }
        // O indice segue o disco: o que o watcher viu mudar e' reindexado
        // aqui, no loop principal, antes de o evento chegar a UI.
        if notification.method == "event.fs.changed" {
            let paths: Vec<PathBuf> = notification
                .params
                .as_ref()
                .and_then(|p| p.get("changes"))
                .and_then(Value::as_array)
                .map(|changes| {
                    changes
                        .iter()
                        .filter_map(|c| c.get("path").and_then(Value::as_str))
                        .map(PathBuf::from)
                        .collect()
                })
                .unwrap_or_default();
            self.reindex_changed_paths(&paths);
        }
        if notification.method != "event.cmake.finished" {
            return;
        }
        let success = notification
            .params
            .as_ref()
            .and_then(|params| params.get("success"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        self.on_cmake_configure_finished(success);
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
