//! Metadata crosses the real dispatch without connecting or changing profiles.

use kinein_protocol::{
    DataSourceConnectionKind, DataSourceEngine, DataSourceListResult, DataSourceProfileFeature,
    JsonRpcErrorCode, JsonRpcRequest,
};
use serde_json::{Value, json};

fn request(
    core: &mut crate::Core,
    method: &str,
    params: Value,
) -> kinein_protocol::JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

#[test]
fn implemented_providers_are_metadata_without_credentials_or_database_access() {
    let root = std::env::temp_dir().join(format!("{}-providers-metadata", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".kinein")).unwrap();
    let path = root.join(".kinein/datasources.json");
    // Legacy engine omitted, unreachable destination and no available secret.
    let original = r#"{"schemaVersion":1,"profiles":[{"name":"legacy","host":"invalid.invalid","port":5432,"database":"app","user":"author","secretSource":"prompt"}]}"#;
    std::fs::write(&path, original).unwrap();
    let mut core = super::core_with_empty_search_path("providers-metadata");
    assert!(
        request(&mut core, "workspace.open", json!({"path": root}))
            .error
            .is_none()
    );
    let response = request(&mut core, "datasource.list", json!({}));
    assert!(response.error.is_none());
    let result: DataSourceListResult = serde_json::from_value(response.result.unwrap()).unwrap();
    assert_eq!(result.profiles[0].engine, DataSourceEngine::Postgres);
    assert_eq!(result.providers.len(), 4);
    let ids: Vec<_> = result
        .providers
        .iter()
        .map(|provider| provider.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "builtin.postgres",
            "builtin.sqlite",
            "builtin.mongo",
            "system.odbc"
        ]
    );
    let pg = &result.providers[0];
    assert_eq!(pg.connection_kind, DataSourceConnectionKind::Server);
    assert!(
        pg.profile_features
            .contains(&DataSourceProfileFeature::VerifiedTls)
    );
    let sqlite = &result.providers[1];
    assert_eq!(sqlite.connection_kind, DataSourceConnectionKind::File);
    assert!(sqlite.profile_features.is_empty());
    let mongo = &result.providers[2];
    assert!(
        mongo
            .profile_features
            .contains(&DataSourceProfileFeature::Sampling)
    );
    assert!(
        !mongo
            .profile_features
            .contains(&DataSourceProfileFeature::VerifiedTls)
    );
    assert_eq!(
        result.providers[3].connection_kind,
        DataSourceConnectionKind::Dsn
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    assert_eq!(
        request(
            &mut core,
            "datasource.list",
            json!({"password":"forbidden"})
        )
        .error
        .unwrap()
        .code,
        JsonRpcErrorCode::InvalidParams
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);

    let saved = request(
        &mut core,
        "datasource.save",
        json!({"profile":{
            "name":"file", "engine":"sqlite", "host":"", "port":0,
            "database":root.join("never-opened.sqlite"), "user":""
        }}),
    );
    let providers = serde_json::to_value(&result.providers).unwrap();
    assert_eq!(saved.result.unwrap()["providers"], providers);
    assert!(!root.join("never-opened.sqlite").exists());
    let removed = request(&mut core, "datasource.remove", json!({"name":"file"}));
    assert_eq!(removed.result.unwrap()["providers"], providers);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_engine_is_refused_instead_of_selecting_postgres() {
    let root = std::env::temp_dir().join(format!("{}-providers-unknown", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let mut core = super::core_with_empty_search_path("providers-unknown");
    assert!(
        request(&mut core, "workspace.open", json!({"path": root}))
            .error
            .is_none()
    );
    let response = request(
        &mut core,
        "datasource.save",
        json!({"profile":{
            "name":"unsupported", "engine":"influx3", "host":"db", "port":8086,
            "database":"app", "user":"author"
        }}),
    );
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    assert!(!root.join(".kinein/datasources.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}
