//! Recusa ODBC no despacho: nenhum driver e' carregado pelo gerenciador falso.

use kinein_protocol::{DataSourceOdbcSource, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

fn scenario(label: &str) -> (crate::Core, std::path::PathBuf) {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-odbc-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        root.join("bin"),
    ));
    core.odbc = crate::datasource::odbc::Session::with_sources(vec![DataSourceOdbcSource {
        dsn: "prova".to_owned(),
        driver: "/nao-carregar.so".to_owned(),
        identity: "driver-atual".to_owned(),
    }]);
    assert!(
        rpc(&mut core, "workspace.open", json!({ "path": root }))
            .error
            .is_none()
    );
    assert!(
        rpc(&mut core, "datasource.save", profile("prova", ""))
            .error
            .is_none()
    );
    (core, root)
}

fn profile(dsn: &str, user: &str) -> Value {
    json!({ "profile": { "name": "outro", "engine": "odbc", "host": "", "port": 0, "database": dsn, "user": user } })
}

fn rpc(core: &mut crate::Core, method: &str, params: Value) -> JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn challenge(core: &mut crate::Core, method: &str) -> Value {
    let params = if method == "datasource.query" {
        json!({ "name": "outro", "sql": "SELECT 1" })
    } else {
        json!({ "name": "outro" })
    };
    let response = rpc(core, method, params);
    let error = response.error.unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::DriverApprovalRequired);
    error.details.unwrap()
}

#[test]
fn disconnect_revokes_consent_and_blocks_authorization_until_the_worker_exits() {
    let (mut core, root) = scenario("disconnect");
    let (sender, receiver) = std::sync::mpsc::channel();
    core.enable_lsp(sender);
    let details = challenge(&mut core, "datasource.query");
    let approval = json!({"name":"outro","identity":details["identity"],"workspace":root});
    assert!(
        rpc(&mut core, "datasource.odbc.authorize", approval.clone())
            .error
            .is_none()
    );
    let profile = crate::datasource::list(&root).pop().unwrap();
    assert!(core.odbc.required(&root, &profile).unwrap().is_none());
    let worker = core.datasource_activity.begin(&root, "outro").unwrap();
    let closing = rpc(
        &mut core,
        "datasource.disconnect",
        json!({"name":"outro",
        "clientContext":"disconnect.odbc", "expectedContext":{"workspace":root,"profile":profile}}),
    );
    assert!(closing.error.is_none());
    assert!(core.odbc.required(&root, &profile).unwrap().is_some());
    assert_eq!(
        rpc(&mut core, "datasource.odbc.authorize", approval.clone())
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidRequest
    );
    drop(worker);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        assert!(std::time::Instant::now() < deadline);
        let event = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        if event.method == "event.datasource.disconnected" {
            assert_eq!(event.params.unwrap()["success"], true);
            break;
        }
    }
    assert!(core.odbc.required(&root, &profile).unwrap().is_some());
    assert!(
        rpc(&mut core, "datasource.odbc.authorize", approval)
            .error
            .is_none()
    );
    assert!(core.odbc.required(&root, &profile).unwrap().is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn driver_consent_is_required_before_query_and_does_not_override_write_confirmation() {
    let (mut core, root) = scenario("recusa");
    for method in ["datasource.test", "datasource.introspect"] {
        assert_eq!(challenge(&mut core, method)["workspace"], json!(root));
    }
    let details = challenge(&mut core, "datasource.query");
    assert_eq!(details["dsn"], "prova");
    let approved = rpc(
        &mut core,
        "datasource.odbc.authorize",
        json!({ "name": "outro", "identity": details["identity"], "workspace": root }),
    );
    assert!(approved.error.is_none(), "{:?}", approved.error);
    for text in [
        "INSERT INTO t VALUES (1)",
        "DELETE FROM t",
        "WITH x AS (DELETE FROM t) SELECT * FROM x",
        "SELECT 1; DROP TABLE t",
    ] {
        let response = rpc(
            &mut core,
            "datasource.query",
            json!({ "name": "outro", "sql": text }),
        );
        assert_eq!(
            response.error.unwrap().code,
            JsonRpcErrorCode::WriteConfirmationRequired,
            "{text}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn approval_rejects_a_stale_profile_and_a_different_workspace_and_removal_revokes_it() {
    let (mut core, root) = scenario("identidade");
    let details = challenge(&mut core, "datasource.query");
    assert!(
        rpc(
            &mut core,
            "datasource.save",
            profile("prova", "outro-usuario")
        )
        .error
        .is_none()
    );
    let stale = json!({ "name": "outro", "identity": details["identity"], "workspace": root });
    assert!(
        rpc(&mut core, "datasource.odbc.authorize", stale)
            .error
            .is_some()
    );
    let details = challenge(&mut core, "datasource.query");
    let params = json!({ "name": "outro", "identity": details["identity"], "workspace": root });
    assert!(rpc(&mut core, "datasource.odbc.authorize", json!({ "name": "outro", "identity": details["identity"], "workspace": "/outro-projeto" })).error.is_some());
    assert!(
        rpc(&mut core, "datasource.odbc.authorize", params)
            .error
            .is_none()
    );
    assert!(
        rpc(&mut core, "datasource.remove", json!({ "name": "outro" }))
            .error
            .is_none()
    );
    assert!(
        rpc(
            &mut core,
            "datasource.save",
            profile("prova", "outro-usuario")
        )
        .error
        .is_none()
    );
    challenge(&mut core, "datasource.query");
    let details = challenge(&mut core, "datasource.test");
    assert!(
        rpc(
            &mut core,
            "datasource.odbc.authorize",
            json!({ "name": "outro", "identity": details["identity"], "workspace": root })
        )
        .error
        .is_none()
    );
    assert!(
        rpc(
            &mut core,
            "datasource.destroy",
            json!({ "name": "outro", "data": true })
        )
        .error
        .is_none()
    );
    assert!(
        rpc(
            &mut core,
            "datasource.save",
            profile("prova", "outro-usuario")
        )
        .error
        .is_none()
    );
    challenge(&mut core, "datasource.introspect");
    let details = challenge(&mut core, "datasource.query");
    assert!(
        rpc(
            &mut core,
            "datasource.odbc.authorize",
            json!({ "name": "outro", "identity": details["identity"], "workspace": root })
        )
        .error
        .is_none()
    );
    assert!(rpc(&mut core, "workspace.close", json!({})).error.is_none());
    assert!(
        rpc(&mut core, "workspace.open", json!({ "path": root }))
            .error
            .is_none()
    );
    challenge(&mut core, "datasource.query");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dsn_and_authorization_params_cannot_smuggle_connection_strings_or_password_fields() {
    let (mut core, root) = scenario("parametros");
    for dsn in [
        "x;UID=outro;PWD=segredo",
        "x\0",
        "x\n",
        "{driver}",
        "",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        assert!(
            rpc(&mut core, "datasource.save", profile(dsn, ""))
                .error
                .is_some(),
            "{dsn:?}"
        );
    }
    assert!(
        rpc(
            &mut core,
            "datasource.odbc.sources",
            json!({ "password": "nunca" })
        )
        .error
        .is_some()
    );
    assert!(
        rpc(
            &mut core,
            "datasource.odbc.authorize",
            json!({ "name": "outro", "identity": "qualquer", "workspace": root, "allow": true })
        )
        .error
        .is_some()
    );
    let saved = std::fs::read_to_string(root.join(".kinein/datasources.json")).unwrap();
    assert!(!saved.contains("password") && !saved.contains("segredo"));
    let sources = rpc(&mut core, "datasource.odbc.sources", json!({}))
        .result
        .unwrap();
    assert_eq!(sources["sources"][0]["dsn"], "prova");
    std::fs::remove_dir_all(root).unwrap();
}
