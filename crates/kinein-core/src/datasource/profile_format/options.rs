//! As opcoes de cada adaptador atual (schema de opcoes 1): o que o perfil
//! do IPC vira no registro em disco e de volta. O formato do catalogo
//! (envelope, classificacao, migracao) e' do modulo pai; aqui so' as chaves e
//! os valores que cada motor aceita (`arquitetura/39` §5.1).

use std::collections::BTreeMap;

use kinein_protocol::driver::operation::{OptionValue, PublicOptions};
use kinein_protocol::{
    DataSourceEngine as Engine, DataSourceProfile, DataSourceTls,
    DataSourceUnavailableReason as Reason,
};
use serde::Deserialize;
use serde_json::value::RawValue;

use super::{OPTIONS_VERSION, object, unique_keys};

/// As opcoes v1 de um adaptador atual; `None` protege o arquivo (chave
/// repetida no envelope), `Err` deixa so' o perfil indisponivel.
pub(super) fn read_options(
    raw: &RawValue,
) -> Option<Result<BTreeMap<String, OptionValue>, Reason>> {
    #[derive(Deserialize)]
    struct Version {
        #[serde(rename = "schemaVersion")]
        version: u32,
    }
    let keys = unique_keys(raw.get())?;
    if keys.iter().any(|k| k != "schemaVersion" && k != "fields") {
        return Some(Err(Reason::UnsupportedOptions));
    }
    let Some(Version { version }) = object(raw.get()) else {
        return Some(Err(Reason::InvalidOptions));
    };
    if version != OPTIONS_VERSION {
        return Some(Err(Reason::UnsupportedOptions));
    }
    Some(
        object::<PublicOptions>(raw.get())
            .map(|options| options.fields)
            .ok_or(Reason::InvalidOptions),
    )
}

/// As chaves que cada adaptador atual aceita, schema de opcoes 1.
const fn allowed(engine: Engine) -> &'static [&'static str] {
    match engine {
        Engine::Postgres => &["host", "port", "database", "user", "tls", "caFile"],
        Engine::Sqlite => &["path"],
        Engine::Mongo => &["host", "port", "database", "user", "sampleSize"],
        Engine::Odbc => &["dsn", "user"],
    }
}

pub(in crate::datasource) fn options_for(
    profile: &DataSourceProfile,
) -> BTreeMap<String, OptionValue> {
    let mut fields = BTreeMap::new();
    let mut text = |key: &str, value: &str| {
        fields.insert(key.to_owned(), OptionValue::Text(value.to_owned()));
    };
    match profile.engine {
        Engine::Postgres | Engine::Mongo => {
            text("host", &profile.host);
            text("database", &profile.database);
            text("user", &profile.user);
        }
        Engine::Sqlite => text("path", &profile.database),
        Engine::Odbc => {
            text("dsn", &profile.database);
            text("user", &profile.user);
        }
    }
    match profile.engine {
        Engine::Postgres => {
            if profile.tls == Some(DataSourceTls::Require) {
                text("tls", "require");
            }
            if let Some(ca) = &profile.ca_file {
                text("caFile", ca);
            }
            fields.insert(
                "port".to_owned(),
                OptionValue::Integer(i64::from(profile.port)),
            );
        }
        Engine::Mongo => {
            fields.insert(
                "port".to_owned(),
                OptionValue::Integer(i64::from(profile.port)),
            );
            if let Some(sample) = profile.sample_size {
                fields.insert(
                    "sampleSize".to_owned(),
                    OptionValue::Integer(i64::from(sample)),
                );
            }
        }
        Engine::Sqlite | Engine::Odbc => {}
    }
    fields
}

pub(super) fn to_profile(
    base: &DataSourceProfile,
    engine: Engine,
    fields: &BTreeMap<String, OptionValue>,
) -> Result<DataSourceProfile, Reason> {
    if fields
        .keys()
        .any(|k| !allowed(engine).contains(&k.as_str()))
    {
        return Err(Reason::UnsupportedOptions);
    }
    let text = |key: &str| match fields.get(key) {
        None => Ok(None),
        Some(OptionValue::Text(value)) => Ok(Some(value.clone())),
        Some(_) => Err(Reason::InvalidOptions),
    };
    let integer = |key: &str| match fields.get(key) {
        None => Ok(None),
        Some(OptionValue::Integer(value)) => Ok(Some(*value)),
        Some(_) => Err(Reason::InvalidOptions),
    };
    let database = match engine {
        Engine::Sqlite => text("path")?,
        Engine::Odbc => text("dsn")?,
        Engine::Postgres | Engine::Mongo => text("database")?,
    };
    let tls = match text("tls")?.as_deref() {
        None | Some("disable") => None,
        Some("require") => Some(DataSourceTls::Require),
        Some(_) => return Err(Reason::InvalidOptions),
    };
    Ok(DataSourceProfile {
        name: base.name.clone(),
        engine,
        production: base.production,
        read_only: base.read_only,
        host: text("host")?.unwrap_or_default(),
        port: integer("port")?
            .map_or(Ok(0), u16::try_from)
            .map_err(|_| Reason::InvalidOptions)?,
        database: database.unwrap_or_default(),
        user: text("user")?.unwrap_or_default(),
        secret_source: base.secret_source,
        secret_variable: base.secret_variable.clone(),
        sample_size: integer("sampleSize")?
            .map(u32::try_from)
            .transpose()
            .map_err(|_| Reason::InvalidOptions)?,
        tls,
        ca_file: text("caFile")?,
        installation: base.installation,
    })
}
