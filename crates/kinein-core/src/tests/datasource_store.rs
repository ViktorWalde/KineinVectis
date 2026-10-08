//! Protected catalogues cannot become writable empty lists through dispatch.

use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

fn request(core: &mut crate::Core, method: &str, params: Value) -> JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

#[test]
fn protected_catalogues_refuse_mutations_before_database_effects() {
    for (case, original) in [
        ("invalid", "PRIVATE_DATA_NOT_FOR_UI"),
        (
            "future",
            r#"{"schemaVersion":999,"profiles":[],"private":"PRIVATE_DATA_NOT_FOR_UI"}"#,
        ),
        (
            "extra",
            r#"{"schemaVersion":1,"profiles":[],"private":"PRIVATE_DATA_NOT_FOR_UI"}"#,
        ),
        (
            "engine",
            r#"{"schemaVersion":1,"profiles":[{"name":"missing","engine":"future"}]}"#,
        ),
    ] {
        let root =
            std::env::temp_dir().join(format!("{}-protected-catalogue-{case}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        let path = root.join(".kinein/datasources.json");
        std::fs::write(&path, original).unwrap();
        let mut core = super::core_with_empty_search_path(&format!("protected-catalogue-{case}"));
        assert!(
            request(&mut core, "workspace.open", json!({"path": root}))
                .error
                .is_none()
        );
        let profile = json!({"name":"new", "engine":"sqlite", "host":"", "port":0,
            "database":root.join("new.sqlite"), "user":""});
        for (method, params) in [
            ("datasource.list", json!({})),
            ("datasource.save", json!({"profile":profile.clone()})),
            ("datasource.remove", json!({"name":"missing"})),
            (
                "datasource.create",
                json!({"kind":"sqliteFile","name":"new","path":root.join("new.sqlite")}),
            ),
            (
                "datasource.create",
                json!({"kind":"containerServer","engine":"postgres","name":"new","port":5432}),
            ),
            ("datasource.destroy", json!({"name":"missing","data":true})),
            ("datasource.test", json!({"name":"missing"})),
        ] {
            let response = request(&mut core, method, params);
            let error = response.error.expect(method);
            assert!(!error.message.contains("PRIVATE_DATA"), "{case}: {method}");
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                original,
                "{case}: {method}"
            );
            assert!(!root.join("new.sqlite").exists(), "{case}: {method}");
        }
        let expected = serde_json::from_value(profile).unwrap();
        assert!(crate::datasource::remove_unchanged(&root, &expected).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn list_reports_preserved_profiles_without_their_options() {
    let root = std::env::temp_dir().join(format!("{}-preserved-list", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".kinein")).unwrap();
    let path = root.join(".kinein/datasources.json");
    let original = include_str!("../datasource/fixtures/profiles/v2-mixed.json");
    std::fs::write(&path, original).unwrap();
    let mut core = super::core_with_empty_search_path("preserved-list");
    assert!(
        request(&mut core, "workspace.open", json!({"path": root}))
            .error
            .is_none()
    );
    let listed = request(&mut core, "datasource.list", json!({}))
        .result
        .expect("list");
    assert_eq!(listed["profiles"].as_array().unwrap().len(), 1);
    let unavailable = listed["unavailable"].as_array().unwrap();
    assert_eq!(unavailable.len(), 5);
    assert_eq!(
        unavailable[0],
        json!({"name":"bordo","engine":"influxdb3","adapter":"native.influxdb3","reason":"unknownProvider"})
    );
    let wire = listed.to_string();
    assert!(!wire.contains("8181") && !wire.contains("INFLUX_TOKEN") && !wire.contains("70000"));

    // Salvar com o nome de um preservado e' recusado; o arquivo nao muda.
    let refused = request(
        &mut core,
        "datasource.save",
        json!({"profile":{"name":"bordo","engine":"sqlite","host":"","port":0,
            "database":root.join("b.sqlite"),"user":""}}),
    );
    assert_eq!(
        refused.error.expect("save").code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), original);

    // Remover e' o gesto explicito: o preservado sai, o resto fica.
    let removed = request(&mut core, "datasource.remove", json!({"name":"bordo"}))
        .result
        .expect("remove");
    assert_eq!(removed["unavailable"].as_array().unwrap().len(), 4);
    assert_eq!(removed["profiles"].as_array().unwrap().len(), 1);
}
