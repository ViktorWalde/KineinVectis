//! Registry of existing adapters. Metadata only: no process, driver or secret.

use kinein_protocol::{
    DataSourceConnectionKind as ConnectionKind, DataSourceEngine as Engine,
    DataSourceProfileFeature as Feature, DataSourceProviderDescriptor,
};

/// Describes the implementation already used by the native execution paths.
#[must_use]
pub fn descriptor(engine: Engine) -> DataSourceProviderDescriptor {
    let (id, connection_kind, profile_features) = match engine {
        Engine::Postgres => (
            "builtin.postgres",
            ConnectionKind::Server,
            vec![Feature::Credentials, Feature::VerifiedTls],
        ),
        Engine::Sqlite => ("builtin.sqlite", ConnectionKind::File, vec![]),
        Engine::Mongo => (
            "builtin.mongo",
            ConnectionKind::Server,
            vec![Feature::Credentials, Feature::Sampling],
        ),
        Engine::Odbc => (
            "system.odbc",
            ConnectionKind::Dsn,
            vec![Feature::Credentials],
        ),
    };
    DataSourceProviderDescriptor {
        id: id.to_owned(),
        engine,
        connection_kind,
        profile_features,
    }
}

/// Presentation order of implemented engines. Research candidates are absent.
#[must_use]
pub fn list() -> Vec<DataSourceProviderDescriptor> {
    [
        Engine::Postgres,
        Engine::Sqlite,
        Engine::Mongo,
        Engine::Odbc,
    ]
    .into_iter()
    .map(descriptor)
    .collect()
}
