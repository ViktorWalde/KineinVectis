//! `datasource.history` (`0.166.0`, passo 13b) pelo despacho, com um `SQLite`
//! real. O que se prova: so' entra o que RODOU (a recusa antes do job nao);
//! a repeticao imediata atualiza o topo; a falha do motor entra como falha;
//! limpar e remover o perfil apagam; e o core sem o historico ligado nao grava
//! nada (o processo real e' quem liga, como a persistencia).

use std::{path::PathBuf, sync::mpsc, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

use crate::datasource::history::History;

struct Scenario {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
    dir: PathBuf,
}

impl Scenario {
    fn new(name: &str, history: Option<&PathBuf>) -> Self {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-dshistory-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        rusqlite::Connection::open(dir.join("dados.db"))
            .unwrap()
            .execute_batch("CREATE TABLE t (id INTEGER); INSERT INTO t VALUES (1), (2);")
            .unwrap();
        let (sender, events) = mpsc::channel();
        let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            dir.join("bin"),
        ));
        core.enable_lsp(sender);
        if let Some(state) = history {
            core.enable_query_history(History::new(state.clone()));
        }
        let mut scenario = Self { core, events, dir };
        let opened = scenario.rpc(
            "workspace.open",
            json!({ "path": scenario.dir.to_str().unwrap() }),
        );
        assert!(opened.error.is_none(), "{:?}", opened.error);
        let saved = scenario.save();
        assert!(saved.error.is_none(), "{:?}", saved.error);
        scenario
    }

    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn save(&mut self) -> kinein_protocol::JsonRpcResponse {
        let path = self.dir.join("dados.db").display().to_string();
        self.rpc(
            "datasource.save",
            json!({ "profile": { "name": "arquivo", "engine": "sqlite", "host": "", "port": 0,
                                 "database": path, "user": "" } }),
        )
    }

    fn run(&mut self, sql: &str) -> Value {
        let accepted = self.rpc("datasource.query", json!({ "name": "arquivo", "sql": sql }));
        assert!(accepted.error.is_none(), "{:?}", accepted.error);
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < deadline {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50))
                && event.method == "event.datasource.queried"
            {
                return event.params.unwrap();
            }
        }
        panic!("event.datasource.queried nao chegou");
    }

    fn history(&mut self) -> Vec<Value> {
        let listed = self.rpc("datasource.history", json!({ "name": "arquivo" }));
        assert!(listed.error.is_none(), "{:?}", listed.error);
        let result = listed.result.unwrap();
        assert_eq!(result["name"], "arquivo");
        result["entries"].as_array().unwrap().clone()
    }
}

#[test]
fn only_what_ran_is_recorded_and_clear_or_remove_erase_it() {
    let state = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dshistory-state", std::process::id()));
    let _ = std::fs::remove_dir_all(&state);
    let mut c = Scenario::new("ligado", Some(&state));
    assert!(c.history().is_empty());

    assert_eq!(c.run("SELECT count(*) FROM t")["success"], true);
    assert_eq!(c.run("SELECT count(*) FROM t")["success"], true);
    let entries = c.history();
    assert_eq!(
        entries.len(),
        1,
        "a repeticao imediata atualiza o topo: {entries:?}"
    );
    assert_eq!(entries[0]["sql"], "SELECT count(*) FROM t");
    assert_eq!(entries[0]["outcome"], "ok");
    assert_eq!(entries[0]["rows"], 1);
    assert!(entries[0]["at"].as_u64().unwrap() > 1_700_000_000);

    assert_eq!(c.run("SELECT * FROM nada")["success"], false);
    let entries = c.history();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["sql"], "SELECT * FROM nada");
    assert_eq!(entries[0]["outcome"], "failed");
    assert!(entries[0].get("rows").is_none());

    // A recusa ANTES do job (apagar sem confirmar) nao rodou: nao entra.
    let refused = c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "DELETE FROM t" }),
    );
    assert_eq!(
        refused.error.unwrap().code,
        JsonRpcErrorCode::WriteConfirmationRequired
    );
    assert_eq!(c.history().len(), 2);

    // Nome sem perfil salvo e' parametro invalido.
    let unknown = c.rpc("datasource.history", json!({ "name": "outro" }));
    assert_eq!(unknown.error.unwrap().code, JsonRpcErrorCode::InvalidParams);

    let cleared = c.rpc("datasource.history.clear", json!({ "name": "arquivo" }));
    assert!(cleared.error.is_none(), "{:?}", cleared.error);
    assert_eq!(cleared.result.unwrap()["name"], "arquivo");
    assert!(c.history().is_empty());

    // Remover o perfil apaga o historico dele: salvo de novo, comeca vazio.
    c.run("SELECT 1");
    assert_eq!(c.history().len(), 1);
    let removed = c.rpc("datasource.remove", json!({ "name": "arquivo" }));
    assert!(removed.error.is_none(), "{:?}", removed.error);
    let saved = c.save();
    assert!(saved.error.is_none(), "{:?}", saved.error);
    assert!(c.history().is_empty());
}

#[test]
fn a_core_without_history_records_nothing() {
    let mut c = Scenario::new("desligado", None);
    assert_eq!(c.run("SELECT count(*) FROM t")["success"], true);
    assert!(c.history().is_empty());
    let cleared = c.rpc("datasource.history.clear", json!({ "name": "arquivo" }));
    assert!(cleared.error.is_none(), "{:?}", cleared.error);
}
