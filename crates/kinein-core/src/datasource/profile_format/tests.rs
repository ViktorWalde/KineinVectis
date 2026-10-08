//! Formato 2, migracao e preservacao, sem disco: o store prova a escrita.

use super::*;

const LEGACY_FIXTURE: &str = include_str!("../fixtures/profiles/v1-legacy.json");
const MIXED_FIXTURE: &str = include_str!("../fixtures/profiles/v2-mixed.json");

fn by_name<'a>(catalogue: &'a Catalogue, name: &str) -> &'a DataSourceProfile {
    catalogue
        .profiles
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{name} ausente"))
}

#[test]
fn the_legacy_file_reads_exactly_as_the_closed_profile_did() {
    #[derive(serde::Deserialize)]
    struct Before {
        profiles: Vec<DataSourceProfile>,
    }
    let before: Before = serde_json::from_str(LEGACY_FIXTURE).unwrap();
    let catalogue = parse_legacy(LEGACY_FIXTURE).unwrap();
    assert_eq!(catalogue.profiles, before.profiles);
    assert!(catalogue.preserved.is_empty());
    // Posicional nao e' objeto, nem no legado.
    assert!(parse_legacy("[1, []]").is_none());
}

#[test]
fn migration_keeps_what_each_engine_uses_and_drops_what_it_never_read() {
    let legacy = parse_legacy(LEGACY_FIXTURE).unwrap();
    let text = serialize(&legacy).unwrap();
    assert!(text.contains("\"schemaVersion\": 2"));
    let migrated = parse_current(&text).unwrap();
    assert!(same_catalogue(&legacy, &migrated));

    let pg = by_name(&migrated, "telemetria");
    assert_eq!(
        (
            pg.host.as_str(),
            pg.port,
            pg.database.as_str(),
            pg.user.as_str()
        ),
        ("db.local", 5433, "tsdb", "kv")
    );
    assert_eq!(pg.tls, Some(DataSourceTls::Require));
    assert_eq!(pg.ca_file.as_deref(), Some("certs/ca.pem"));
    assert_eq!(pg.secret_source, SecretSource::Environment);
    assert_eq!(pg.secret_variable.as_deref(), Some("PGPASSWORD_TELEMETRIA"));
    assert!(pg.production);

    let sqlite = by_name(&migrated, "estacao-local");
    assert_eq!(sqlite.database, "dados/estacao.db");
    // O SQLite nunca leu host, porta nem amostra: nao migram.
    assert_eq!((sqlite.host.as_str(), sqlite.port), ("", 0));
    assert_eq!(sqlite.sample_size, None);

    let mongo = by_name(&migrated, "sensores");
    assert_eq!((mongo.port, mongo.sample_size), (27017, Some(200)));

    let odbc = by_name(&migrated, "dsn-fabrica");
    assert_eq!(
        (odbc.database.as_str(), odbc.user.as_str()),
        ("FabricaDSN", "leitor")
    );
    assert!(odbc.read_only);
    assert_eq!(odbc.secret_source, SecretSource::Prompt);

    // Gravar de novo o que foi lido no formato 2 nao muda nada.
    assert_eq!(serialize(&migrated).unwrap(), text);
}

#[test]
fn profiles_this_version_cannot_use_keep_their_bytes_and_a_public_reason() {
    let catalogue = parse_current(MIXED_FIXTURE).unwrap();
    assert_eq!(catalogue.profiles.len(), 1);
    assert_eq!(catalogue.profiles[0].name, "telemetria");
    let reasons: Vec<_> = catalogue
        .unavailable()
        .into_iter()
        .map(|p| (p.name, p.engine, p.adapter, p.reason))
        .collect();
    let row = |name: &str, engine: &str, adapter: &str, reason| {
        (
            name.to_owned(),
            engine.to_owned(),
            adapter.to_owned(),
            reason,
        )
    };
    assert_eq!(
        reasons,
        vec![
            row(
                "bordo",
                "influxdb3",
                "native.influxdb3",
                Reason::UnknownProvider
            ),
            row(
                "com-campo-futuro",
                "postgres",
                "builtin.postgres",
                Reason::UnsupportedOptions
            ),
            row(
                "externo",
                "postgres",
                "builtin.postgres",
                Reason::UnsupportedInstallation
            ),
            row(
                "opcoes-futuras",
                "sqlite",
                "builtin.sqlite",
                Reason::UnsupportedOptions
            ),
            row(
                "porta-errada",
                "postgres",
                "builtin.postgres",
                Reason::InvalidOptions
            ),
        ]
    );
    // Regravado, o texto de cada preservado volta igual, byte a byte.
    let reread = parse_current(&serialize(&catalogue).unwrap()).unwrap();
    assert!(same_catalogue(&catalogue, &reread));
    let edge_profile = reread
        .preserved
        .iter()
        .find(|p| p.public.name == "bordo")
        .unwrap();
    assert!(
        edge_profile
            .raw
            .get()
            .contains("\"url\": \"http://localhost:8181\"")
    );
    // A parte publica nao carrega opcao nenhuma.
    let public = serde_json::to_string(&catalogue.unavailable()).unwrap();
    assert!(!public.contains("8181") && !public.contains("INFLUX_TOKEN"));
}

#[test]
fn ambiguous_or_broken_structure_protects_the_whole_file() {
    let entry = |body: &str| format!(r#"{{"schemaVersion":2,"profiles":[{body}]}}"#);
    let pg = r#""engine":"postgres","adapter":"builtin.postgres","options":{"schemaVersion":1,"fields":{}}"#;
    for (case, text) in [
        (
            "nome repetido",
            format!(
                r#"{{"schemaVersion":2,"profiles":[{{"name":"a",{pg}}},{{"name":"a",{pg}}}]}}"#
            ),
        ),
        (
            "campo conhecido repetido",
            entry(&format!(r#"{{"name":"a","name":"b",{pg}}}"#)),
        ),
        (
            "campo futuro repetido",
            entry(&format!(r#"{{"name":"a","x":1,"x":2,{pg}}}"#)),
        ),
        (
            "tipo errado no core",
            entry(&format!(r#"{{"name":"a","production":"sim",{pg}}}"#)),
        ),
        (
            "sem adaptador",
            entry(r#"{"name":"a","engine":"postgres","options":{}}"#),
        ),
        ("nome vazio", entry(&format!(r#"{{"name":" ",{pg}}}"#))),
        (
            "motor fora do alfabeto",
            entry(r#"{"name":"a","engine":"Postgres SQL","adapter":"x","options":{}}"#),
        ),
        ("envelope posicional", "[2, []]".to_owned()),
        (
            "perfil posicional",
            entry(r#"["a","postgres","builtin.postgres"]"#),
        ),
        (
            "envelope com chave repetida",
            r#"{"schemaVersion":2,"profiles":[],"profiles":[]}"#.to_owned(),
        ),
    ] {
        assert!(parse_current(&text).is_none(), "{case}");
    }
}

#[test]
fn each_adapter_refuses_keys_and_values_it_does_not_know() {
    let catalogue = |engine: &str, adapter: &str, options: &str| {
        let text = format!(
            r#"{{"schemaVersion":2,"profiles":[{{"name":"p","engine":"{engine}","adapter":"{adapter}","options":{options}}}]}}"#
        );
        parse_current(&text).unwrap()
    };
    let reason = |engine: &str, adapter: &str, options: &str| {
        catalogue(engine, adapter, options).unavailable()[0].reason
    };
    let pg = |fields: &str| format!(r#"{{"schemaVersion":1,"fields":{fields}}}"#);
    assert_eq!(
        reason("postgres", "builtin.postgres", &pg(r#"{"sslmode":"x"}"#)),
        Reason::UnsupportedOptions
    );
    assert_eq!(
        reason("sqlite", "builtin.sqlite", &pg(r#"{"host":"x"}"#)),
        Reason::UnsupportedOptions
    );
    assert_eq!(
        reason("postgres", "builtin.postgres", &pg(r#"{"port":"5432"}"#)),
        Reason::InvalidOptions
    );
    assert_eq!(
        reason(
            "postgres",
            "builtin.postgres",
            &pg(r#"{"tls":"verify-ca"}"#)
        ),
        Reason::InvalidOptions
    );
    assert_eq!(
        reason("mongo", "builtin.mongo", &pg(r#"{"sampleSize":-1}"#)),
        Reason::InvalidOptions
    );
    assert_eq!(
        reason(
            "postgres",
            "builtin.postgres",
            &pg(r#"{"host":"a","host":"b"}"#)
        ),
        Reason::InvalidOptions
    );
    assert_eq!(
        reason(
            "postgres",
            "builtin.postgres",
            r#"{"schemaVersion":1,"fields":{},"extra":true}"#
        ),
        Reason::UnsupportedOptions
    );
    // Motor conhecido com adaptador de outro: nao escolhe substituto.
    assert_eq!(
        reason("postgres", "builtin.sqlite", &pg("{}")),
        Reason::UnknownProvider
    );
    // Um perfil valido do mesmo jeito que o core escreve.
    let usable = catalogue(
        "odbc",
        "system.odbc",
        &pg(r#"{"dsn":"Fabrica","user":"u"}"#),
    );
    assert_eq!(usable.profiles[0].database, "Fabrica");
}

#[test]
fn the_written_file_holds_where_the_secret_comes_from_never_a_secret() {
    fn keys(value: &serde_json::Value, out: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, inner) in map {
                    out.push(key.to_lowercase());
                    keys(inner, out);
                }
            }
            serde_json::Value::Array(items) => items.iter().for_each(|i| keys(i, out)),
            _ => {}
        }
    }
    let legacy = parse_legacy(LEGACY_FIXTURE).unwrap();
    let text = serialize(&legacy).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut found = Vec::new();
    keys(&value, &mut found);
    for key in found {
        let allowed = key == "secretsource" || key == "secretvariable";
        assert!(
            allowed
                || !["password", "secret", "token", "credential"]
                    .iter()
                    .any(|s| key.contains(s)),
            "{key}"
        );
    }
}
