//! Types for the `datasource.*` domain: connections to the user's databases.
//!
//! **There is no password field in this file, and that is the point.** The
//! decision is registered in `docs/seguranca/40-cofre-de-credencial.md`
//! (author, 2026-09-04): the IDE owns the PROFILE — which is not a secret —
//! and never writes a password to disk. The password reaches the driver from
//! the session prompt (memory only), from an environment variable, or from
//! `PostgreSQL`'s own `~/.pgpass`.
//!
//! Adding a `password` field here would silently undo that decision, since the
//! profile is what gets persisted.

use serde::{Deserialize, Serialize};

/// Where the password for a profile comes from, when a connection is opened.
///
/// This is a POLICY, not a secret: it is safe to persist because it says
/// *where to look*, never *what was found*.
///
/// # Why there is a "no password at all" default
///
/// The IDE is not the one asking for a password — the user's own server is. A
/// local development `PostgreSQL` reached over a Unix socket with `peer`
/// authentication has the OS user *as* the identity, and `trust` asks for
/// nothing either. Making the author type a password in that case would be an
/// obstacle the IDE invented on its own.
///
/// So the default is [`SecretSource::Automatic`]: send no password and let the
/// server decide. If it does demand one, the UI can still ask — falling back
/// costs one round trip and is invisible when it is not needed.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecretSource {
    /// Send no password: let the server and libpq settle it.
    ///
    /// This one option covers three real cases at once — Unix socket with
    /// `peer` auth, `trust` on a dev box, and `~/.pgpass` / `PGPASSFILE`
    /// (which libpq reads on its own, requiring mode 0600 and ignoring the
    /// file entirely when it is looser).
    #[default]
    Automatic,
    /// Read from an environment variable, named by the profile.
    Environment,
    /// Ask the author when the connection is opened; keep it in memory for the
    /// session only. Nothing is written to disk.
    Prompt,
}

/// A saved connection to a database. Never carries a password.
///
/// `deny_unknown_fields` is deliberate and load-bearing here, not tidiness.
/// Without it, a UI that sent `"password": "..."` would be answered with a
/// cheerful success while the field was silently dropped — the caller would
/// believe the password had been stored. Refusing is the honest answer, and a
/// test in `tests/datasource.rs` pins it.
///
/// The cost is accepted knowingly: a field removed from this struct later can
/// no longer be read back from an older `.kinein/datasources.json`. For a
/// boundary that guards a secret, strict beats forgiving.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceProfile {
    /// Stable identity, unique per workspace. Also what the UI shows.
    pub name: String,
    /// Host name, address, or — starting with `/` — a Unix socket DIRECTORY.
    ///
    /// libpq treats a host beginning with a slash as a socket directory (for
    /// example `/var/run/postgresql`), which is the usual way to reach a local
    /// server with `peer` authentication and no password at all.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Role used to connect.
    pub user: String,
    /// Where the password comes from.
    #[serde(default)]
    pub secret_source: SecretSource,
    /// Environment variable holding the password, for `Environment`.
    ///
    /// Only the NAME of the variable lives here; the value is read at connect
    /// time and never stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_variable: Option<String>,
}

/// Result payload for `datasource.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceListResult {
    /// Saved profiles, ordered by name.
    pub profiles: Vec<DataSourceProfile>,
}

/// Parameters for `datasource.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceListParams {}

/// Parameters for `datasource.save` — creates or replaces by `name`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceSaveParams {
    /// The profile to store.
    pub profile: DataSourceProfile,
}

/// Parameters for `datasource.remove`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceRemoveParams {
    /// Name of the profile to drop.
    pub name: String,
}

/// Result payload for `datasource.save` and `datasource.remove`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceWriteResult {
    /// The catalogue after the write, ordered by name.
    pub profiles: Vec<DataSourceProfile>,
}

/// Parameters for `datasource.test` — connect once and report back.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceTestParams {
    /// Which saved profile to try.
    pub name: String,
    /// The session password, when the profile's policy is `Prompt`.
    ///
    /// This is the ONE place a password crosses the wire, and it crosses a
    /// pipe between two processes of the same user — never the disk. The Qt
    /// client redacts fields named like this one before writing its request
    /// log, because that log is also written to a file
    /// (`ui/src/core_client_process.cpp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Result payload for `datasource.test` — the job that will report the answer.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceTestAccepted {
    /// Job to follow; the answer arrives as `event.datasource.tested`.
    pub job_id: String,
}
