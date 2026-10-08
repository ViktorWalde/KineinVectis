//! Public metadata of implemented database adapters; never a live connection.

use serde::{Deserialize, Serialize};

/// How the current adapter addresses a database.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceConnectionKind {
    /// Host, port and database name.
    Server,
    /// Local database file.
    File,
    /// DSN registered in the system ODBC manager.
    Dsn,
}

/// Profile fields accepted by an adapter, independent of live permissions.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceProfileFeature {
    /// User and secret-source policy; never a saved password.
    Credentials,
    /// TLS with both certificate chain and host name verification.
    VerifiedTls,
    /// Sample size for inferred document structure.
    Sampling,
}

/// Metadata consumed by the connection form. Listing it executes no tool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceProviderDescriptor {
    /// Implementation identity, distinct from engine and future instance IDs.
    pub id: String,
    /// Existing profile value; extensible profile persistence is a later slice.
    pub engine: crate::DataSourceEngine,
    /// Addressing model used by the existing implementation.
    pub connection_kind: DataSourceConnectionKind,
    /// Supported profile options, not negotiated database or LSP capabilities.
    pub profile_features: Vec<DataSourceProfileFeature>,
    /// Adapters the person can choose for this engine (`0.167.0`).
    pub installations: Vec<DataSourceInstallationKind>,
}

/// One adapter choice offered by a provider descriptor.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceInstallationKind {
    /// The adapter compiled into the core (absent `installation`).
    Builtin,
    /// The external adapter installed with the IDE.
    Ide,
}

/// Why a saved profile is preserved but cannot be used by this version.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceUnavailableReason {
    /// Engine or adapter identity this core does not implement.
    UnknownProvider,
    /// The profile names an external installation, which needs the runtime.
    UnsupportedInstallation,
    /// A field or option schema version this core does not know.
    UnsupportedOptions,
    /// A known option holds a value its adapter refuses.
    InvalidOptions,
}

/// Public identity of a preserved profile; its options never cross the wire.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceUnavailableProfile {
    /// Workspace-unique profile name.
    pub name: String,
    /// Engine identity as written in the file (validated characters only).
    pub engine: String,
    /// Adapter identity as written in the file (validated characters only).
    pub adapter: String,
    /// Fixed public reason, never file content.
    pub reason: DataSourceUnavailableReason,
}
