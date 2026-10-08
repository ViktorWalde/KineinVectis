//! Registry of existing adapters. Metadata only: no process, driver or secret.

use kinein_protocol::{
    DataSourceConnectionKind as ConnectionKind, DataSourceEngine as Engine, DataSourceInstallation,
    DataSourceInstallationKind as Kind, DataSourceProfileFeature as Feature,
    DataSourceProviderDescriptor,
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
        installations: if engine == Engine::Sqlite {
            vec![Kind::Builtin, Kind::Ide]
        } else {
            vec![Kind::Builtin]
        },
    }
}

/// O motor oferece esta instalacao? Hoje so' o `SQLite` tem o adaptador da
/// IDE (`roadmaps/40.7` §7.256); os outros motores entram com os seus.
#[must_use]
pub fn offers(engine: Engine, installation: DataSourceInstallation) -> bool {
    match installation {
        DataSourceInstallation::Ide {} => descriptor(engine).installations.contains(&Kind::Ide),
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
