//! Types for the `container.*` domain: Docker and Podman as a NATIVE domain.
//!
//! Decision of the author on 2026-07-17 (`roadmaps/28` §0): containers are a
//! first-class domain of the core, never a plugin. The invariants recorded
//! there shape these types: the UI never calls `docker` itself, every
//! lifecycle command is a cancellable job, and permissions (socket, group,
//! rootless) are visible — that is what `ContainerStatus` carries.
//!
//! One protocol, two engines: `docker` and `podman` speak the same CLI, and on
//! Fedora `docker` is often the `podman-docker` shim (measured 2026-09-12).
//! The core detects which one answers and says so (`emulated`).

use serde::{Deserialize, Serialize};

/// Which engine answered.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContainerEngine {
    /// Docker Engine (moby) — `docker` CLI talking to `dockerd`.
    Docker,
    /// Podman — daemonless; rootless by default on Fedora.
    Podman,
}

/// Result payload for `container.status`: is there an engine, and can this
/// user use it? This is the "activate the tool" screen.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerStatus {
    /// `None` when neither `docker` nor `podman` is on the `PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<ContainerEngine>,
    /// The executable the core will run (`podman`, `docker`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    /// Client version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `docker` on the `PATH` is the `podman-docker` shim: it prints
    /// *"Emulate Docker CLI using podman"* on stderr and answers as Podman.
    pub emulated: bool,
    /// Podman rootless mode, when the engine says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rootless: Option<bool>,
    /// The socket/daemon the engine reports (`/run/user/1000/podman/podman.sock`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
    /// The engine answered `ps`: daemon up and this user allowed.
    pub reachable: bool,
    /// Compose tool found: `docker compose`, `docker-compose` or `podman-compose`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose: Option<String>,
    /// The compose file the tool would pick up in the open workspace
    /// (`compose.yaml`, `docker-compose.yml`, ...), relative to the root.
    /// `None` without a workspace or when the project has none — `compose up`
    /// cannot work then, and the UI must not promise it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compose_file: Option<String>,
    /// What to do when something is missing — the official step, never `sudo`
    /// run by the IDE.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Raw stderr/stdout of the probe, for when the parser did not understand.
    pub raw_output: String,
}

/// One container, as `ps` reports it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInfo {
    /// Full or short id.
    pub id: String,
    /// Names (Podman lists several; Docker one, comma-separated).
    pub names: Vec<String>,
    /// Image reference the container runs.
    pub image: String,
    /// `running`, `exited`, `created`, `paused`...
    pub state: String,
    /// Human status (`Up 2 hours`, `Exited (0) 2 weeks ago`).
    pub status: String,
    /// `host:port->port/proto` lines.
    pub ports: Vec<String>,
    /// Creation timestamp as the engine printed it (RFC 3339 or a date).
    pub created: String,
}

/// One image, as `images` reports it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    /// Image id.
    pub id: String,
    /// Repository (`docker.io/library/postgres`).
    pub repository: String,
    /// Tag (`16-alpine`, `latest`, `<none>`).
    pub tag: String,
    /// Bytes when the engine gives a number; the engine's string otherwise.
    pub size: String,
    /// Creation time as the engine printed it.
    pub created: String,
}

/// Parameters for `container.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerListParams {
    /// Include stopped containers (`ps -a`). Default true: a stopped database
    /// is exactly what the user wants to start.
    #[serde(default = "default_true")]
    pub all: bool,
}

const fn default_true() -> bool {
    true
}

/// Result payload for `container.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerListResult {
    /// Containers the parser recognised.
    pub containers: Vec<ContainerInfo>,
    /// Engine that answered, so the UI can label it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<ContainerEngine>,
    /// Raw output — always sent, for the case the format changed.
    pub raw_output: String,
    /// What to do when the list is empty or unreadable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Parameters for `container.images`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerImagesParams {}

/// Result payload for `container.images`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerImagesResult {
    /// Images the parser recognised.
    pub images: Vec<ImageInfo>,
    /// Engine that answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<ContainerEngine>,
    /// Raw output — always sent.
    pub raw_output: String,
    /// What to do when the list is empty or unreadable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Lifecycle actions. Each one is a job (`roadmaps/28` §4: "todo comando de
/// container é Job cancelável").
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContainerAction {
    /// `start`.
    Start,
    /// `stop` (SIGTERM, then SIGKILL after the engine's grace period).
    Stop,
    /// `restart`.
    Restart,
    /// `rm` — refused while running; the user stops first, on purpose.
    Remove,
}

impl ContainerAction {
    /// The CLI verb.
    #[must_use]
    pub const fn verb(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Remove => "rm",
        }
    }
}

/// Parameters for `container.action`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerActionParams {
    /// Container id or name.
    pub id: String,
    /// What to do with it.
    pub action: ContainerAction,
}

/// Accepted: the job id to follow (`event.container.finished`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerActionAccepted {
    /// Job id; `event.container.finished` carries the same id.
    pub job_id: String,
}

/// What to open in a terminal tab for a container.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContainerOpenMode {
    /// `logs -f`: the stream, with Ctrl+C for free.
    Logs,
    /// `exec -it <id> sh`: a shell INSIDE the container — the seed of the
    /// remote context (`roadmaps/28` §4).
    Shell,
}

/// Parameters for `container.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerOpenParams {
    /// Container id or name.
    pub id: String,
    /// Logs or a shell.
    pub mode: ContainerOpenMode,
}

/// Result payload for `container.open`: the terminal session, like
/// `terminal.open`, plus the command line so the tab can be titled.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerOpenResult {
    /// Terminal session id (`terminal.input`/`resize`/`close` take it).
    pub id: String,
    /// The command line that runs in the tab.
    pub command: String,
}

/// Compose verbs. `Up` is detached: the tab of `logs` is where output lives.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComposeAction {
    /// `compose up -d`.
    Up,
    /// `compose down`.
    Down,
}

/// Parameters for `container.compose`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContainerComposeParams {
    /// Up or down.
    pub action: ComposeAction,
    /// Compose file relative to the workspace; absent, the tool's default
    /// (`compose.yaml`, `docker-compose.yml`) in the workspace root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// Payload of `event.container.finished`: a lifecycle or compose job ended.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerFinishedEvent {
    /// The job that ended.
    pub job_id: String,
    /// `start`, `stop`, `restart`, `rm`, `compose up`, `compose down`.
    pub action: String,
    /// Container id/name, or the compose file.
    pub target: String,
    /// Exit status zero.
    pub ok: bool,
    /// Last lines the engine printed — the reason when `ok` is false.
    pub message: String,
}
