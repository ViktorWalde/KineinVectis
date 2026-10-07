//! Despacho real: conexao ocupada precisa sair antes do evento de desconexao.
//! `SQLite` externo segura uma escrita; perfil, console e outro destino ficam.

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

fn rpc(core: &mut crate::Core, method: &str, params: Value) -> JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn scenario(label: &str) -> (crate::Core, PathBuf, Value, mpsc::Receiver<JsonRpcRequest>) {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-disconnect-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        root.join("bin"),
    ));
    core.enable_lsp(sender);
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":root}))
            .error
            .is_none()
    );
    for name in ["bank", "other"] {
        let path = root.join(format!("{name}.sqlite"));
        rusqlite::Connection::open(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE items(id INTEGER PRIMARY KEY); INSERT INTO items VALUES(1)",
            )
            .unwrap();
        assert!(
            rpc(
                &mut core,
                "datasource.save",
                json!({"profile":{
            "name":name,"engine":"sqlite","host":"","port":0,"database":path,"user":""}})
            )
            .error
            .is_none()
        );
    }
    let saved = rpc(&mut core, "datasource.list", json!({})).result.unwrap()["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "bank")
        .unwrap()
        .clone();
    let context = json!({"workspace":root,"profile":saved});
    (core, root, context, receiver)
}

fn event(receiver: &mpsc::Receiver<JsonRpcRequest>, method: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(50))
            && event.method == method
        {
            return event.params.unwrap();
        }
    }
    panic!("evento nao chegou: {method}");
}

#[test]
fn disconnect_checks_workspace_profile_and_token_without_requesting_secrets() {
    let mut core = crate::Core::default();
    assert_eq!(
        rpc(&mut core, "datasource.disconnect", json!({}))
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidRequest
    );
    let (mut core, root, mut context, receiver) = scenario("context");
    assert_eq!(
        rpc(&mut core, "datasource.disconnect", json!({"name":"bank"}))
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidParams
    );
    context["workspace"] = json!("/changed");
    assert_eq!(
        rpc(
            &mut core,
            "datasource.disconnect",
            json!({"name":"bank","expectedContext":context,"clientContext":"close.1"})
        )
        .error
        .unwrap()
        .code,
        JsonRpcErrorCode::DataSourceContextChanged
    );
    context["workspace"] = json!(root);
    assert_eq!(
        rpc(
            &mut core,
            "datasource.disconnect",
            json!({"name":"bank","expectedContext":context,"clientContext":""})
        )
        .error
        .unwrap()
        .code,
        JsonRpcErrorCode::InvalidParams
    );
    let profile = json!({"name":"prompt","host":"localhost","port":1,"database":"db","user":"user","secretSource":"prompt"});
    let saved = rpc(&mut core, "datasource.save", json!({"profile":profile}))
        .result
        .unwrap()["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "prompt")
        .unwrap()
        .clone();
    let response = rpc(
        &mut core,
        "datasource.disconnect",
        json!({"name":"prompt","expectedContext":{"workspace":root,"profile":saved},"clientContext":"close.2"}),
    );
    assert!(response.error.is_none(), "{:?}", response.error);
    let event = event(&receiver, "event.datasource.disconnected");
    assert_eq!(event["success"], true);
    assert_eq!(event["clientContext"], "close.2");
    assert_eq!(event["jobId"], response.result.unwrap()["jobId"]);
    assert_eq!(
        rpc(&mut core, "datasource.list", json!({})).result.unwrap()["profiles"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn disconnect_drains_an_accepted_write_blocks_new_work_and_preserves_files() {
    let (mut core, root, context, receiver) = scenario("drain");
    let console = rpc(&mut core, "datasource.console", json!({"name":"bank"}))
        .result
        .unwrap();
    let console_path = PathBuf::from(console["path"].as_str().unwrap());
    std::fs::write(&console_path, "SELECT 1;\n-- rascunho\n").unwrap();
    let profiles = std::fs::read(root.join(".kinein/datasources.json")).unwrap();
    let lock = rusqlite::Connection::open(root.join("bank.sqlite")).unwrap();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let writing = rpc(
        &mut core,
        "datasource.query",
        json!({"name":"bank","sql":"INSERT INTO items VALUES(2)","clientContext":"write.1","expectedContext":context}),
    );
    assert!(writing.error.is_none());
    let closing = rpc(
        &mut core,
        "datasource.disconnect",
        json!({"name":"bank","expectedContext":context,"clientContext":"close.1"}),
    );
    assert!(closing.error.is_none());
    for (method, sql) in [
        ("datasource.query", Some("SELECT 1")),
        ("datasource.test", None),
        ("datasource.introspect", None),
        ("datasource.impact", Some("DELETE FROM items")),
        ("datasource.destroy", None),
        ("datasource.disconnect", None),
    ] {
        let mut request =
            json!({"name":"bank","expectedContext":context,"clientContext":"blocked.1"});
        if let Some(sql) = sql {
            request["sql"] = json!(sql);
        }
        assert_eq!(
            rpc(&mut core, method, request).error.unwrap().code,
            JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }
    assert!(rpc(&mut core, "core.ping", json!({})).error.is_none());
    assert!(
        rpc(
            &mut core,
            "datasource.query",
            json!({"name":"other","sql":"SELECT count(*) FROM items","clientContext":"other.1"})
        )
        .error
        .is_none()
    );
    let deadline = Instant::now() + Duration::from_millis(100);
    while Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(10)) {
            assert_ne!(event.method, "event.datasource.disconnected");
        }
    }
    lock.execute_batch("COMMIT").unwrap();
    let completed = event(&receiver, "event.datasource.disconnected");
    assert_eq!(completed["success"], true);
    assert_eq!(completed["jobId"], closing.result.unwrap()["jobId"]);
    assert_eq!(
        lock.query_row("SELECT count(*) FROM items", [], |row| row.get::<_, u32>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        std::fs::read(root.join(".kinein/datasources.json")).unwrap(),
        profiles
    );
    assert_eq!(
        std::fs::read_to_string(console_path).unwrap(),
        "SELECT 1;\n-- rascunho\n"
    );
    assert!(
        rpc(
            &mut core,
            "datasource.query",
            json!({"name":"bank","sql":"SELECT 1"})
        )
        .error
        .is_none()
    );
    assert_eq!(
        event(&receiver, "event.datasource.queried")["success"],
        true
    );
}
