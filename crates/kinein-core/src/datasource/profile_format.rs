//! Formato do catalogo `.kinein/datasources.json`: schema 1 (legado) e 2.
//!
//! Dono unico da conversao entre o registro em disco e o `DataSourceProfile`
//! do IPC (D1a.4, `DocsPublic/arquitetura/39` §5.1). O schema 2 separa o que e'
//! do core (nome, motor, adaptador, politica, origem do segredo) das opcoes do
//! adaptador (`PublicOptions`, as mesmas do contrato do driver). Um perfil que
//! esta versao nao sabe usar fica com o texto original, byte a byte; dele so'
//! saem nome, motor, adaptador e motivo.

use std::collections::{BTreeMap, BTreeSet};

use kinein_protocol::driver::deserialize_object;
use kinein_protocol::driver::operation::PublicOptions;
use kinein_protocol::{
    DataSourceEngine as Engine, DataSourceInstallation, DataSourceProfile,
    DataSourceUnavailableProfile, DataSourceUnavailableReason as Reason, SecretSource,
};
use serde::de::{IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::RawValue;

use super::providers;
pub(super) use options::options_for;
use options::{read_options, to_profile};

/// Schema que esta versao escreve.
pub(super) const CURRENT: u32 = 2;
/// Schema legado: lido como sempre e migrado na primeira gravacao.
pub(super) const LEGACY: u32 = 1;
/// Versao das opcoes dos adaptadores atuais.
const OPTIONS_VERSION: u32 = 1;
/// Campos de um perfil schema 2 que esta versao conhece.
const RECORD_FIELDS: [&str; 9] = [
    "name",
    "engine",
    "adapter",
    "installation",
    "production",
    "readOnly",
    "secretSource",
    "secretVariable",
    "options",
];

/// Perfil que esta versao nao usa, guardado com o texto original.
#[derive(Debug, Clone)]
pub(super) struct Preserved {
    /// O que a UI pode saber dele.
    pub(super) public: DataSourceUnavailableProfile,
    /// O texto JSON lido, regravado sem reinterpretar.
    pub(super) raw: Box<RawValue>,
}

/// O catalogo de um arquivo reconhecido.
#[derive(Debug, Clone, Default)]
pub(super) struct Catalogue {
    /// Perfis que esta versao usa.
    pub(super) profiles: Vec<DataSourceProfile>,
    /// Perfis preservados sem uso.
    pub(super) preserved: Vec<Preserved>,
}

impl Catalogue {
    /// A parte publica dos preservados, por nome.
    pub(super) fn unavailable(&self) -> Vec<DataSourceUnavailableProfile> {
        let mut list: Vec<_> = self.preserved.iter().map(|p| p.public.clone()).collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Se `name` pertence a um perfil preservado.
    pub(super) fn is_preserved(&self, name: &str) -> bool {
        self.preserved.iter().any(|p| p.public.name == name)
    }
}

/// Le um arquivo schema 1: o perfil inteiro, fechado, como ate' a `0.164.0`.
pub(super) fn parse_legacy(text: &str) -> Option<Catalogue> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Legacy {
        #[serde(rename = "schemaVersion")]
        _schema: u32,
        #[serde(default)]
        profiles: Vec<DataSourceProfile>,
    }
    let file: Legacy = object(text)?;
    Some(Catalogue {
        profiles: file.profiles,
        preserved: Vec::new(),
    })
}

/// Le um arquivo schema 2. `None` torna o arquivo inteiro protegido.
pub(super) fn parse_current(text: &str) -> Option<Catalogue> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct File {
        #[serde(rename = "schemaVersion")]
        _schema: u32,
        #[serde(default)]
        profiles: Vec<Box<RawValue>>,
    }
    let file: File = object(text)?;
    let mut catalogue = Catalogue::default();
    let mut names = BTreeSet::new();
    for raw in file.profiles {
        match classify(raw)? {
            Entry::Usable(profile) => {
                if !names.insert(profile.name.clone()) {
                    return None;
                }
                catalogue.profiles.push(profile);
            }
            Entry::Preserved(preserved) => {
                if !names.insert(preserved.public.name.clone()) {
                    return None;
                }
                catalogue.preserved.push(preserved);
            }
        }
    }
    Some(catalogue)
}

/// O texto schema 2 do catalogo, com os preservados intactos, por nome.
///
/// # Errors
/// Falha de serializacao, que nao deveria ocorrer com estes tipos.
pub(super) fn serialize(catalogue: &Catalogue) -> Result<String, String> {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Out<'a> {
        Typed(WriteRecord<'a>),
        Raw(&'a RawValue),
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct File<'a> {
        schema_version: u32,
        profiles: Vec<Out<'a>>,
    }
    let mut entries: Vec<(&str, Out<'_>)> = catalogue
        .profiles
        .iter()
        .map(|p| (p.name.as_str(), Out::Typed(write_record(p))))
        .chain(
            catalogue
                .preserved
                .iter()
                .map(|p| (p.public.name.as_str(), Out::Raw(&p.raw))),
        )
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    serde_json::to_string_pretty(&File {
        schema_version: CURRENT,
        profiles: entries.into_iter().map(|(_, out)| out).collect(),
    })
    .map_err(|error| format!("falha serializando perfis: {error}"))
}

/// O perfil como o schema 2 o guarda e devolve: os campos que o motor nao usa
/// (o `host` de um `SQLite`, a amostra de um `PostgreSQL`) nao sobrevivem.
#[must_use]
pub(super) fn effective(profile: &DataSourceProfile) -> DataSourceProfile {
    let options = PublicOptions {
        schema_version: OPTIONS_VERSION,
        fields: options_for(profile),
    };
    // A conversao de volta e' a mesma da leitura; um perfil que ela recusasse
    // nao chega aqui, porque so' perfis validos sao gravados.
    to_profile(profile, profile.engine, &options.fields).unwrap_or_else(|_| profile.clone())
}

/// A releitura reproduz o catalogo gravado?
#[must_use]
pub(super) fn same_catalogue(written: &Catalogue, reread: &Catalogue) -> bool {
    let mut expected: Vec<_> = written.profiles.iter().map(effective).collect();
    let mut found = reread.profiles.clone();
    expected.sort_by(|a, b| a.name.cmp(&b.name));
    found.sort_by(|a, b| a.name.cmp(&b.name));
    let raw = |c: &Catalogue| -> BTreeMap<String, String> {
        c.preserved
            .iter()
            .map(|p| (p.public.name.clone(), p.raw.get().to_owned()))
            .collect()
    };
    expected == found && raw(written) == raw(reread)
}

enum Entry {
    Usable(DataSourceProfile),
    Preserved(Preserved),
}

/// Campos lidos de um perfil schema 2, sem recusar campo desconhecido: ele e'
/// de uma versao futura e torna so' este perfil indisponivel. Duplicata ou
/// tipo errado num campo conhecido tornam o ARQUIVO protegido.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    name: String,
    engine: String,
    adapter: String,
    #[serde(default)]
    installation: Option<Box<RawValue>>,
    #[serde(default)]
    production: bool,
    #[serde(default)]
    read_only: bool,
    #[serde(default)]
    secret_source: SecretSource,
    #[serde(default)]
    secret_variable: Option<String>,
    options: Box<RawValue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteRecord<'a> {
    name: &'a str,
    engine: String,
    adapter: String,
    production: bool,
    read_only: bool,
    secret_source: SecretSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_variable: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    installation: Option<DataSourceInstallation>,
    options: PublicOptions,
}

fn write_record(profile: &DataSourceProfile) -> WriteRecord<'_> {
    WriteRecord {
        name: &profile.name,
        engine: engine_id(profile.engine),
        adapter: providers::descriptor(profile.engine).id,
        production: profile.production,
        read_only: profile.read_only,
        secret_source: profile.secret_source,
        secret_variable: profile.secret_variable.as_deref(),
        installation: profile.installation,
        options: PublicOptions {
            schema_version: OPTIONS_VERSION,
            fields: options_for(profile),
        },
    }
}

fn classify(raw: Box<RawValue>) -> Option<Entry> {
    let keys = unique_keys(raw.get())?;
    let record: Record = object(raw.get())?;
    if record.name.trim().is_empty() || !identity(&record.engine) || !identity(&record.adapter) {
        return None;
    }
    let reason = match known_engine(&record.engine)
        .filter(|engine| providers::descriptor(*engine).id == record.adapter)
    {
        None => Some(Reason::UnknownProvider),
        Some(_) if keys.iter().any(|k| !RECORD_FIELDS.contains(&k.as_str())) => {
            Some(Reason::UnsupportedOptions)
        }
        Some(engine) => match (
            installation(&record, engine),
            read_options(&record.options)?,
        ) {
            (Ok(chosen), Ok(fields)) => {
                match to_profile(&base(&record, engine, chosen), engine, &fields) {
                    Ok(profile) => return Some(Entry::Usable(profile)),
                    Err(reason) => Some(reason),
                }
            }
            (Err(reason), _) | (Ok(_), Err(reason)) => Some(reason),
        },
    };
    reason.map(|reason| {
        Entry::Preserved(Preserved {
            public: DataSourceUnavailableProfile {
                name: record.name,
                engine: record.engine,
                adapter: record.adapter,
                reason,
            },
            raw,
        })
    })
}

/// O perfil so' com o que e' do core; o endereco vem das opcoes.
fn base(
    record: &Record,
    engine: Engine,
    installation: Option<DataSourceInstallation>,
) -> DataSourceProfile {
    DataSourceProfile {
        name: record.name.clone(),
        engine,
        production: record.production,
        read_only: record.read_only,
        host: String::new(),
        port: 0,
        database: String::new(),
        user: String::new(),
        secret_source: record.secret_source,
        secret_variable: record.secret_variable.clone(),
        sample_size: None,
        tls: None,
        ca_file: None,
        installation,
    }
}

/// A instalacao do registro: ausente e' o interno; `{ kind: "ide" }` so' nos
/// motores que oferecem o adaptador da IDE. Outra forma torna o perfil
/// indisponivel (preservado, como antes do `0.167.0`).
fn installation(record: &Record, engine: Engine) -> Result<Option<DataSourceInstallation>, Reason> {
    let Some(raw) = record.installation.as_deref() else {
        return Ok(None);
    };
    object::<DataSourceInstallation>(raw.get())
        .filter(|chosen| providers::offers(engine, *chosen))
        .map(Some)
        .ok_or(Reason::UnsupportedInstallation)
}

/// Decodifica um objeto JSON, recusando a forma posicional (array).
fn object<'de, T: Deserialize<'de>>(text: &'de str) -> Option<T> {
    let mut decoder = serde_json::Deserializer::from_str(text);
    let value = deserialize_object(&mut decoder).ok()?;
    decoder.end().ok()?;
    Some(value)
}

/// As chaves de um objeto, cada uma uma vez so'.
fn unique_keys(text: &str) -> Option<BTreeSet<String>> {
    struct Keys;
    impl<'de> Visitor<'de> for Keys {
        type Value = BTreeSet<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object with unique keys")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut keys = BTreeSet::new();
            while let Some(key) = map.next_key::<String>()? {
                if !keys.insert(key) {
                    return Err(serde::de::Error::custom("duplicate key"));
                }
                map.next_value::<IgnoredAny>()?;
            }
            Ok(keys)
        }
    }
    let mut decoder = serde_json::Deserializer::from_str(text);
    let keys = decoder.deserialize_map(Keys).ok()?;
    decoder.end().ok()?;
    Some(keys)
}

/// Identidade de motor ou adaptador: `[a-z0-9._-]`, ate' 64 bytes.
fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
}

/// O nome do motor como o `DataSourceEngine` o serializa — um dono so'.
pub(super) fn engine_id(engine: Engine) -> String {
    serde_json::to_value(engine)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn known_engine(id: &str) -> Option<Engine> {
    serde_json::from_value(serde_json::Value::String(id.to_owned())).ok()
}

mod options;

#[cfg(test)]
mod tests;
