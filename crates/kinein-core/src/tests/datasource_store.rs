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
