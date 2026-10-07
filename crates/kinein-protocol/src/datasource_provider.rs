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
}
