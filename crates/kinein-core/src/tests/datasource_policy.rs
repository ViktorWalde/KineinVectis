//! Tentativas de contornar a política pelo despacho real, antes de senha/job.

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

fn rpc(core: &mut crate::Core, method: &str, params: Value) -> JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn scenario(
    label: &str,
    engine: &str,
    production: bool,
    read_only: bool,
) -> (crate::Core, std::path::PathBuf, Value) {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-policy-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        root.join("bin"),
    ));
    assert!(
        rpc(&mut core, "workspace.open", json!({"path": root}))
            .error
            .is_none()
    );
    let response = rpc(
        &mut core,
        "datasource.save",
        json!({"profile": {
            "name": "destino", "engine": engine, "host": "localhost", "port": 1,
            "database": "dados", "user": "usuario", "secretSource": "prompt",
            "production": production, "readOnly": read_only
        }}),
    );
    assert!(response.error.is_none(), "{:?}", response.error);
    let profile = response.result.unwrap()["profiles"][0].clone();
    (core, root, profile)
}

#[test]
fn read_only_refuses_batches_ctes_unknown_operations_and_confirmation_before_secrets() {
    for engine in ["postgres", "sqlite", "mongo", "odbc"] {
        let (mut core, root, _) = scenario(engine, engine, false, true);
        let commands = if engine == "mongo" {
            vec![
                "itens.insertOne({\"id\":1})",
                "itens.updateMany({}, {\"$set\":{\"x\":1}})",
                "itens.drop()",
                "itens.naoExiste({})",
            ]
        } else {
            vec![
                "INSERT INTO t VALUES (1)",
                "SELECT 1; COMMIT; DELETE FROM t",
                "SELECT 1 -- comment\r; COMMIT; DELETE FROM t",
                "SELECT 1 AS é$$; COMMIT; DELETE FROM t; SELECT 1 AS fim$$",
                "WITH x AS (DELETE FROM t RETURNING *) SELECT * FROM x",
                "SELECT * INTO nova FROM t",
                "CALL limpa()",
                "SELECT 'sem fim",
            ]
        };
        for text in commands {
            for confirmed in [false, true] {
                let response = rpc(
                    &mut core,
                    "datasource.query",
                    json!({"name":"destino", "sql":text, "confirmWrite":confirmed,
                    "confirmation":{"connection":"destino","target":"t"}, "clientContext":"generation:1"}),
                );
                let error = response.error.unwrap();
                assert_eq!(
                    error.code,
                    JsonRpcErrorCode::ReadOnlyViolation,
                    "{engine}: {text}"
                );
                assert_eq!(error.details.unwrap()["clientContext"], "generation:1");
            }
        }
        if engine != "odbc" {
            let destroyed = rpc(
                &mut core,
                "datasource.destroy",
                json!({"name":"destino", "data":true}),
            );
            assert_eq!(
                destroyed.error.unwrap().code,
                JsonRpcErrorCode::ReadOnlyViolation
            );
        }
        assert!(
            rpc(
                &mut core,
                "datasource.destroy",
                json!({"name":"destino", "data":false})
            )
            .error
            .is_none()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn production_requires_a_warning_for_inserts_and_exact_names_for_removal() {
    let (mut core, root, _) = scenario("production", "postgres", true, false);
    let response = rpc(
        &mut core,
        "datasource.query",
        json!({"name":"destino", "sql":"INSERT INTO t VALUES (1)"}),
    );
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::WriteConfirmationRequired
    );
    for (connection, target) in [
        ("dest", "a.b"),
        ("destino", "b"),
        ("destino", "a"),
        ("outra", "a.b"),
    ] {
        let response = rpc(
            &mut core,
            "datasource.query",
            json!({"name":"destino", "sql":"DROP TABLE \"a.b\"", "confirmWrite":true,
            "confirmation":{"connection":connection,"target":target}}),
        );
        assert_eq!(
            response.error.unwrap().code,
            JsonRpcErrorCode::WriteConfirmationRequired
        );
    }
    let response = rpc(
        &mut core,
        "datasource.query",
        json!({"name":"destino", "sql":"DROP TABLE \"a.b\"", "confirmWrite":true,
        "confirmation":{"connection":"destino","target":"a.b"}}),
    );
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::SecretRequired,
        "nomes completos passam à resolução da senha, sem conectar"
    );
    let response = rpc(
        &mut core,
        "datasource.destroy",
        json!({"name":"destino", "data":true,
        "confirmation":{"connection":"destino","target":"errado"}}),
    );
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::WriteConfirmationRequired
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn destination_and_workspace_changes_refuse_operations_before_secrets() {
    let (mut core, root, profile) = scenario("context", "postgres", false, false);
    for method in [
        "datasource.query",
        "datasource.impact",
        "datasource.test",
        "datasource.introspect",
        "datasource.destroy",
    ] {
        let mut params = json!({"name":"destino", "clientContext":"generation:1"});
        if matches!(method, "datasource.query" | "datasource.impact") {
            params["sql"] = json!("SELECT 1");
        }
        let mut changed = profile.clone();
        changed["host"] = json!("outro-destino");
        for context in [
            json!({"workspace":"/outro-projeto","profile":profile}),
            json!({"workspace":root,"profile":changed}),
        ] {
            let mut wrong = params.clone();
            wrong["expectedContext"] = context;
            let response = rpc(&mut core, method, wrong);
            assert_eq!(
                response.error.unwrap().code,
                JsonRpcErrorCode::DataSourceContextChanged
            );
        }
        if method != "datasource.destroy" {
            let mut exact = params.clone();
            exact["expectedContext"] = json!({"workspace":root,"profile":profile});
            let response = rpc(&mut core, method, exact);
            assert_eq!(
                response.error.unwrap().code,
                JsonRpcErrorCode::SecretRequired
            );
        }
        for token in ["", "with space", "control\n", "a/b"] {
            let mut invalid = params.clone();
            invalid["clientContext"] = json!(token);
            let response = rpc(&mut core, method, invalid);
            assert_eq!(
                response.error.unwrap().code,
                JsonRpcErrorCode::InvalidParams
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
