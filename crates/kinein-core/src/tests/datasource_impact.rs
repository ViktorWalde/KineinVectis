//! `datasource.impact` (0.150.0) pelo despacho, com um `SQLite` real. O que
//! se prova: a recusa do `datasource.query` ja' diz a gravidade; o impacto
//! conta as linhas que cada instrucao pegaria; o `WHERE` que pega todas vira
//! destrutivo; o `DROP TABLE` diz quantas linhas somem; e MEDIR NAO MUDA
//! NADA — as linhas continuam todas la' depois.

use std::{sync::mpsc, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

struct Scenario {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
}

impl Scenario {
    fn new(name: &str) -> (Self, std::path::PathBuf) {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-dsimpact-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        let (sender, receiver) = mpsc::channel();
        let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            dir.join("bin"),
        ));
        core.enable_lsp(sender);
        let mut scenario = Self {
            core,
            events: receiver,
        };
        let opened = scenario.rpc("workspace.open", json!({ "path": dir.to_str().unwrap() }));
        assert!(opened.error.is_none(), "{:?}", opened.error);
        let database = dir.join("loja.db");
        rusqlite::Connection::open(&database)
            .unwrap()
            .execute_batch(
                "CREATE TABLE clientes (id INTEGER PRIMARY KEY, nome TEXT, email TEXT); \
                 INSERT INTO clientes (nome, email) VALUES ('Ana', 'a@x'), ('Bruno', NULL), ('Caio', 'c@x');",
            )
            .unwrap();
        let saved = scenario.rpc(
            "datasource.save",
            json!({ "profile": { "name": "loja", "engine": "sqlite", "host": "", "port": 0,
                                 "database": database.display().to_string(), "user": "" } }),
        );
        assert!(saved.error.is_none(), "{:?}", saved.error);
        (scenario, dir)
    }

    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(9_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn event(&self, name: &str) -> Value {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < deadline {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == name
            {
                return e.params.unwrap();
            }
        }
        panic!("{name} nao chegou");
    }

    fn impact(&mut self, sql: &str) -> Value {
        let accepted = self.rpc("datasource.impact", json!({ "name": "loja", "sql": sql }));
        assert!(accepted.error.is_none(), "{:?}", accepted.error);
        self.event("event.datasource.impact")
    }
}

#[test]
fn the_refusal_tells_the_severity_and_the_impact_counts_without_changing_anything() {
    let (mut s, dir) = Scenario::new("sqlite");

    let refusal = s.rpc(
        "datasource.query",
        json!({ "name": "loja", "sql": "DELETE FROM clientes" }),
    );
    let error = refusal.error.unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::WriteConfirmationRequired);
    assert_eq!(error.details.unwrap()["severity"], "destructive");

    let all = s.impact("DELETE FROM clientes");
    assert_eq!(all["severity"], "destructive", "{all}");
    assert_eq!(all["sql"], "DELETE FROM clientes");
    assert_eq!(all["statements"][0]["rows"], 3);
    assert_eq!(all["statements"][0]["targets"], json!(["clientes"]));

    let some = s.impact("delete from clientes where email is null");
    assert_eq!(some["severity"], "write", "{some}");
    assert_eq!(some["statements"][0]["rows"], 1);
    assert_eq!(some["statements"][0]["totalRows"], 3);

    let forgotten = s.impact("UPDATE clientes SET nome = 'x' WHERE 1 = 1");
    assert_eq!(
        forgotten["severity"], "destructive",
        "WHERE que pega tudo: {forgotten}"
    );

    let both = s.impact("insert into clientes (nome) values ('D'); drop table clientes");
    assert_eq!(both["severity"], "destructive");
    assert_eq!(both["statements"][0]["kind"], "insert");
    assert_eq!(both["statements"][1]["rows"], 3);

    let column = s.impact("ALTER TABLE clientes DROP COLUMN email");
    assert_eq!(
        column["statements"][0]["rows"], 2,
        "valores nao nulos que somem"
    );

    let broken = s.impact("delete from sumiu");
    assert!(broken["statements"][0]["note"].is_string(), "{broken}");

    // Medir nao mudou nada.
    s.rpc(
        "datasource.query",
        json!({ "name": "loja", "sql": "SELECT count(*) FROM clientes" }),
    );
    let still = s.event("event.datasource.queried");
    assert_eq!(still["rows"], json!([["3"]]), "{still}");
    drop(std::fs::remove_dir_all(&dir));
}
