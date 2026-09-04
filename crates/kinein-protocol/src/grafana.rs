//! Types for the `grafana.*` domain: the observability instance that watches
//! this project.
//!
//! # Why this is a domain and not another `DataSourceEngine`
//!
//! Grafana is not a database. It does not answer SQL, it has no schemas and no
//! tables, and reaching it means HTTP with a bearer token instead of a wire
//! protocol with a password. Putting it behind `DataSourceEngine` would give
//! the two a shared shape they do not share, and every `match` in the
//! datasource domain would gain an arm that cannot do the thing the arm is
//! for.
//!
//! # The licence decides the SHAPE of this integration
//!
//! Grafana is **AGPL-3.0** (`docs/integracoes/37-banco-e-observabilidade.md`
//! §2, author decision 2026-09-03). The IDE may *talk* to a Grafana the user
//! runs; it may never embed or redistribute one. Everything in this file is a
//! description of a conversation with a process that belongs to the user.
//!
//! # There is no token field on the profile, and that is the point
//!
//! Same rule as `datasource.rs`: the IDE owns the PROFILE, never the secret.
//! A Grafana service account token is a bearer credential — anyone holding it
//! is the service account — so it is exactly the thing that must not reach
//! `.kinein/grafana.json`.

use serde::{Deserialize, Serialize};

/// Where the Grafana service account token comes from, when a request is made.
///
/// # Why this is not [`crate::SecretSource`]
///
/// The two enums have the same three shapes and would tempt anyone into
/// reusing one. They mean different things, and the difference is exactly the
/// kind that makes a screen lie.
///
/// `SecretSource::Automatic` means *"send nothing, the driver will find it"* —
/// libpq really does read `~/.pgpass` on its own, so a connection with no
/// password can still be a fully authenticated one. There is no equivalent for
/// Grafana: HTTP has no credential helper to defer to. Here, sending nothing
/// means **there is no credential**, and only the unauthenticated part of the
/// API answers.
///
/// One name, two truths, would put "automatic" on a screen where the honest
/// word is "none".
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GrafanaTokenSource {
    /// No token at all.
    ///
    /// This is a useful state, not a broken one: `GET /api/health` needs no
    /// authentication, so the IDE can still answer *"there is a Grafana 13.0.2
    /// at this address and its database is healthy"* — it just cannot list
    /// what is inside.
    #[default]
    None,
    /// Read from an environment variable, named by the profile.
    Environment,
    /// Ask when the probe runs; keep it in memory for the session only.
    Prompt,
}

/// The Grafana instance that observes this workspace.
///
/// # Why ONE instance and not a catalogue
///
/// The datasource domain keeps a list, because a project genuinely talks to
/// several databases. The question this domain answers is different and
/// singular: *"which Grafana watches this project?"*. Copying the catalogue
/// machinery — save-by-name, ordering, remove-by-name — would be around two
/// hundred lines of near-identical code to support a case nobody asked for.
///
/// **The exit, if the case ever arrives:** this struct gains a `name`, the
/// store holds a `Vec`, and the handlers take a name like `datasource.*` do.
/// Nothing here forecloses that.
///
/// `deny_unknown_fields` is load-bearing for the same reason as the datasource
/// profile: a UI that sent `"token": "..."` must be REFUSED, not answered with
/// a success while the field is silently dropped.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrafanaProfile {
    /// Base URL of the instance, without a trailing slash — `http://localhost:3000`.
    pub url: String,
    /// Where the service account token comes from.
    #[serde(default)]
    pub token_source: GrafanaTokenSource,
    /// Environment variable holding the token, for [`GrafanaTokenSource::Environment`].
    ///
    /// Only the NAME lives here; the value is read at probe time and never
    /// stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_variable: Option<String>,
}

/// Parameters for `grafana.get`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrafanaGetParams {}

/// Result payload for `grafana.get`, `grafana.save` and `grafana.forget`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaProfileResult {
    /// The saved instance, or `None` when this workspace has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<GrafanaProfile>,
}

/// Parameters for `grafana.save` — replaces whatever was there.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrafanaSaveParams {
    /// The instance to store.
    pub profile: GrafanaProfile,
}

/// Parameters for `grafana.forget`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrafanaForgetParams {}

/// Parameters for `grafana.probe` — reach the instance and report what it has.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrafanaProbeParams {
    /// The session token, when the policy is [`GrafanaTokenSource::Prompt`].
    ///
    /// This is the ONE place a Grafana token crosses the wire, and it crosses
    /// a pipe between two processes of the same user — never the disk. The Qt
    /// client redacts fields named like this before writing its request log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Result payload for `grafana.probe` — the job that will report the answer.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaProbeAccepted {
    /// Job to follow; the answer arrives as `event.grafana.probed`.
    pub job_id: String,
}

/// One data source that Grafana knows about.
///
/// # What is deliberately absent
///
/// `GET /api/datasources` documents a `password` field, and older servers do
/// send one. Nothing here can hold it: the fields below are the whole struct,
/// and `serde` drops everything else on the way in. A test pins this, because
/// "we just don't read that field" is a habit and this needs to be a
/// guarantee.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaDataSource {
    /// Stable identifier inside the instance.
    pub uid: String,
    /// Display name, as the Grafana user named it.
    pub name: String,
    /// Plugin id — `grafana-postgresql-datasource`, `prometheus`, ...
    pub type_id: String,
    /// Human-readable plugin name (`PostgreSQL`), when the server sends one.
    #[serde(default)]
    pub type_name: String,
    /// Where the data source points — `localhost:5432` for a `PostgreSQL` one.
    #[serde(default)]
    pub url: String,
    /// Database name, for the engines that have one.
    #[serde(default)]
    pub database: String,
    /// Whether this is the instance's default data source.
    #[serde(default)]
    pub is_default: bool,
}

/// One dashboard, as `GET /api/search?type=dash-db` reports it.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaDashboard {
    /// Stable identifier inside the instance.
    pub uid: String,
    /// Dashboard title.
    pub title: String,
    /// Path within the instance — join it to the profile URL to open it.
    pub url: String,
    /// Folder it lives in; empty at the root.
    #[serde(default)]
    pub folder_title: String,
}

/// A data source of THIS project that Grafana is already watching.
///
/// This is the whole reason the domain exists. A link to Grafana is a
/// bookmark; knowing that the database you are developing against is already
/// on a dashboard is integration.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaMatch {
    /// Name of the profile in this workspace's `datasource` catalogue.
    pub profile_name: String,
    /// Name of the data source inside Grafana.
    pub data_source_name: String,
    /// Why the two were considered the same, in words the author can check.
    pub reason: String,
}

/// What `grafana.probe` found, delivered by `event.grafana.probed`.
///
/// # Why reachable and authenticated are separate booleans
///
/// `GET /api/health` needs no token and the rest of the API does. A single
/// "ok" would collapse two very different situations — *"there is no Grafana
/// at this address"* and *"Grafana is there and your token was refused"* —
/// into one message that helps with neither.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrafanaProbeResult {
    /// Whether `/api/health` answered.
    pub reachable: bool,
    /// Grafana version, from `/api/health`.
    #[serde(default)]
    pub version: String,
    /// Health of Grafana's own database — `ok` when it is well.
    #[serde(default)]
    pub database: String,
    /// Whether the token was accepted for the authenticated endpoints.
    pub authenticated: bool,
    /// What went wrong, in a sentence the author can act on.
    #[serde(default)]
    pub message: String,
    /// Data sources Grafana knows; empty when not authenticated.
    #[serde(default)]
    pub data_sources: Vec<GrafanaDataSource>,
    /// Dashboards; empty when not authenticated.
    #[serde(default)]
    pub dashboards: Vec<GrafanaDashboard>,
    /// Profiles of this workspace that Grafana is already watching.
    #[serde(default)]
    pub matches: Vec<GrafanaMatch>,
}
