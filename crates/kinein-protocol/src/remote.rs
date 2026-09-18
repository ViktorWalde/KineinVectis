//! Types for the `remote.*` domain (`0.120.0`, P6 of `roadmaps/42`): a Linux
//! target reached over SSH — the Raspberry Pi, a board with a custom image —
//! as a resource of the project: a profile WITHOUT secrets on disk, a probe
//! that MEASURES the target, deploy as a job, and run/debug as run
//! configurations plus the kit (`remoteTarget`/`debugServer`).

use serde::{Deserialize, Serialize};

/// One SSH target. No password field, structurally: SSH is by key here, and
/// whatever the `ssh` of the system needs to ask, it asks in the IDE's
/// terminal.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteTarget {
    /// Name the user gave (`pi`, `bancada`); the key of the catalogue.
    pub name: String,
    /// Host or IP.
    pub host: String,
    /// SSH user; absent = the `ssh` default (the local user / `~/.ssh/config`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// SSH port; absent = 22.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Private key file for `-i`; absent = the agent / `~/.ssh/config`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_file: Option<String>,
    /// Where deploys land on the target; absent = `~/kinein/<project>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy_dir: Option<String>,
}

/// Parameters for `remote.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteListParams {}

/// Result of `remote.list` / `remote.save` / `remote.remove`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteListResult {
    /// Targets sorted by name.
    pub targets: Vec<RemoteTarget>,
}

/// Parameters for `remote.save`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteSaveParams {
    /// The target to create or replace (by `name`).
    pub target: RemoteTarget,
}

/// Parameters for `remote.remove` / `remote.probe`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteNameParams {
    /// The target's name.
    pub name: String,
}

/// Result of `remote.probe` / `remote.deploy`: the job.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteJobResult {
    /// Job id.
    pub job_id: String,
    /// The command line the job runs.
    pub command: String,
}

/// One tool the probe looked for on the target (`command -v`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteTool {
    /// `gdbserver`, `python3`, `rsync`, `debugpy`.
    pub id: String,
    /// The target has it.
    pub found: bool,
    /// Where, when found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// `event.remote.probed`: what the target IS.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteProbedEvent {
    /// Job id.
    pub job_id: String,
    /// The target.
    pub name: String,
    /// The SSH session ran the probe to the end.
    pub success: bool,
    /// `uname -m` (`aarch64`, `armv7l`, `x86_64`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// `uname -sr` (`Linux 6.6.31+rpt-rpi-v8`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    /// The tools the probe asked for.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<RemoteTool>,
    /// Why not, with the next step (no key → `ssh-copy-id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Everything ssh printed.
    pub raw: String,
}

/// Parameters for `remote.deploy`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteDeployParams {
    /// The target.
    pub name: String,
    /// Local file or folder; absent = the newest ELF of the project model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Remote folder; absent = the target's `deployDir`, or `~/kinein/<project>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
}

/// `event.remote.deployed`: the deploy job ended.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDeployedEvent {
    /// Job id.
    pub job_id: String,
    /// The target.
    pub name: String,
    /// The copier exited 0.
    pub success: bool,
    /// What was sent.
    pub source: String,
    /// Where it landed (`<dest>/<basename>`).
    pub dest: String,
    /// The command line (`rsync` or `scp`).
    pub command: String,
    /// Why not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// What `remote.command` composes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RemoteCommandKind {
    /// Run the deployed binary (`ssh <target> '<dest>/<bin>'`) — a run configuration.
    Run,
    /// Start `gdbserver :<port> <dest>/<bin>` on the target — the kit's `debugServer`.
    DebugServer,
    /// Start `debugpy --listen 0.0.0.0:<port> --wait-for-client <script>` — the Python attach.
    Debugpy,
    /// An interactive shell on the target — a terminal tab.
    Shell,
}

/// Parameters for `remote.command`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteCommandParams {
    /// The target.
    pub name: String,
    /// Which line.
    pub kind: RemoteCommandKind,
    /// The remote path of the binary/script; absent = `<deployDir>/<basename of the
    /// newest ELF or Python entry point>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    /// Port for `gdbserver`/`debugpy`; absent = 2345 / 5678.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

/// Result of `remote.command`: pure — nothing runs.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCommandResult {
    /// The shell line (`ssh …`), single-quoted paths.
    pub command: String,
    /// `host:port` for the kit's `remoteTarget` (`debugServer`) or the attach
    /// `{ host, port }` (`debugpy`); absent for `run`/`shell`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_target: Option<String>,
    /// Suggested run-configuration name (`Rodar em pi`).
    pub name: String,
    /// Where each piece came from.
    pub source: Vec<String>,
}

/// A workspace that is a local MIRROR of a folder on a target (`0.122.0`,
/// P6 slice 2): the IDE works on the mirror; `rsync` moves the bytes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMirror {
    /// The target's name in the catalogue.
    pub name: String,
    /// The target's host, for the UI to show without a second lookup.
    pub host: String,
    /// The folder on the target.
    pub path: String,
    /// The local mirror root (the workspace root).
    pub mirror_root: String,
}

/// Parameters for `remote.open`: mirror `path` of target `name` locally.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteOpenParams {
    /// Target name.
    pub name: String,
    /// Folder on the target (absolute, or `~`-relative for the remote shell).
    pub path: String,
}

/// Result of `remote.open`: the job, and where the mirror will be.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteOpenResult {
    /// Job to follow; `event.remote.synced` with `direction: pull` ends it.
    pub job_id: String,
    /// The command line, for the job output.
    pub command: String,
    /// The local mirror root the UI opens once the pull succeeds.
    pub mirror: String,
}

/// Which way `remote.sync` moves the bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RemoteSyncDirection {
    /// Target -> mirror.
    Pull,
    /// Mirror -> target.
    Push,
}

/// Parameters for `remote.sync` (the open workspace must be a mirror).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteSyncParams {
    /// Pull or push.
    pub direction: RemoteSyncDirection,
    /// Paths relative to the mirror root; absent = the whole tree. Never
    /// deletes on the other side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
}

/// Payload of `event.remote.synced` (from `remote.open`, `remote.sync` and
/// the automatic push after a write in a mirror).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSyncedEvent {
    /// The job that ran it.
    pub job_id: String,
    /// Target name.
    pub name: String,
    /// Pull or push.
    pub direction: RemoteSyncDirection,
    /// Whether every `rsync` succeeded.
    pub success: bool,
    /// The command line(s) that ran.
    pub command: String,
    /// Paths `rsync` reported as transferred (its itemized output), relative.
    pub changed: Vec<String>,
    /// Why not, on failure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The mirror root.
    pub mirror: String,
}

/// Parameters for `remote.status`: none.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteStatusParams {}

/// Result of `remote.status`: the mirror the open workspace is, if any.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatusResult {
    /// Present when the open workspace is a mirror.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mirror: Option<RemoteMirror>,
}
