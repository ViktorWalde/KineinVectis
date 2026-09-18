//! `datasource.discover` e `datasource.create` (`0.124.0`, 2026-09-18):
//! o painel de banco deixa de ser so' "perfil + testar" e passa a fazer o que
//! o de containers ja' fazia — dizer o que responde NESTA maquina — e a
//! criar um banco quando nao ha' nenhum (pedido do autor no teste da
//! Etapa 2).

use serde::{Deserialize, Serialize};

use super::{DataSourceEngine, DataSourceProfile};
use crate::container::ContainerEngine;

/// Parameters for `datasource.discover` — none.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceDiscoverParams {}

/// Where a candidate was found.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceCandidateKind {
    /// A server answering on this machine (socket directory or loopback port).
    LocalServer,
    /// A container running a database image (published port on loopback).
    Container,
    /// A `SQLite` file inside the workspace.
    File,
}

/// One thing that answered, with the profile that reaches it, ready to save.
///
/// Nothing here is a guess: the socket exists, the port accepted a
/// connection, the container is listed, the file has the `SQLite` header.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceCandidate {
    /// Where it was found.
    pub kind: DataSourceCandidateKind,
    /// Short label (`PostgreSQL local`, `container kinein-pg`, `data/app.sqlite`).
    pub label: String,
    /// What was measured (`socket /var/run/postgresql`, `Up 2 hours · 127.0.0.1:5433`, `12 KB`).
    pub detail: String,
    /// Whether it answers NOW (a stopped container is listed, not running).
    pub running: bool,
    /// The profile that reaches it — no password, as always.
    pub profile: DataSourceProfile,
}

/// Result payload for `datasource.discover`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceDiscoverResult {
    /// What answered, servers first, then containers, then files.
    pub candidates: Vec<DataSourceCandidate>,
    /// The container engine consulted, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_engine: Option<ContainerEngine>,
    /// What to do when the list is empty (or short).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// What `datasource.create` makes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum DataSourceCreateKind {
    /// An empty `SQLite` file in the workspace (default `data/<name>.sqlite`).
    #[serde(rename_all = "camelCase")]
    SqliteFile {
        /// Profile name; also the file name when `path` is absent.
        name: String,
        /// Path relative to the workspace (or absolute); refused if it exists.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// A database SERVER in a container on loopback — for the machine that
    /// has Podman/Docker and no server. A job: it pulls an image, so it never
    /// runs without the click, and the command is returned for the screen to
    /// show before and after.
    #[serde(rename_all = "camelCase")]
    ContainerServer {
        /// `postgres` or `mongo` (`sqlite` is refused: it has no server).
        engine: DataSourceEngine,
        /// Profile and container name (`kinein-<name>`).
        name: String,
        /// Loopback port to publish (`127.0.0.1:<port>`).
        port: u16,
    },
}

/// Parameters for `datasource.create`.
///
/// No `deny_unknown_fields` here: serde does not support it together with
/// `flatten`; the tagged enum inside carries it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceCreateParams {
    /// What to make.
    #[serde(flatten)]
    pub what: DataSourceCreateKind,
}

/// Result payload for `datasource.create`: the profile at once (file), or the
/// job to follow (server) — the profile then arrives in `event.datasource.created`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceCreateResult {
    /// The saved profile, when the creation was immediate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<DataSourceProfile>,
    /// The job, when the creation runs a process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    /// The exact command the job runs, for the screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// `event.datasource.created` — the outcome of a `containerServer` job.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceCreatedEvent {
    /// The job that ran.
    pub job_id: String,
    /// Whether the container is up and the profile was saved.
    pub success: bool,
    /// The saved profile, on success.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<DataSourceProfile>,
    /// What happened, in one line (the last lines of the engine on failure).
    pub message: String,
}
