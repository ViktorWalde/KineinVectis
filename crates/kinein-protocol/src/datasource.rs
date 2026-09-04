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
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecretSource {
    /// Ask the author when the connection is opened; keep it in memory for the
    /// session only. The default, because it requires no setup and leaves
    /// nothing behind.
    #[default]
    Prompt,
    /// Read from an environment variable, named by the profile.
    Environment,
    /// Let `PostgreSQL` resolve it from `~/.pgpass` / `PGPASSFILE`. libpq
    /// requires mode 0600 and ignores the file entirely when it is looser.
    PasswordFile,
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
    /// Host name or address as the user typed it.
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
