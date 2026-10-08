//! O caminho RELATIVO de um perfil `SQLite` e' a partir do projeto (decisao do
//! autor, 2026-10-08; `arquitetura/37`). Antes, o motor abria o relativo
//! contra o diretorio corrente do core, e um perfil versionado no repositorio
//! do projeto embarcado so' funcionava por acaso. O que se prova, pelo
//! despacho: testar, ler a estrutura, consultar e medir o impacto abrem o
//! arquivo do projeto; o perfil guardado (e o contexto que a UI devolve)
//! continua com o caminho que a pessoa escreveu.

use std::{path::Path, sync::mpsc, time::Duration};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

struct Scenario {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
}

impl Scenario {
    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn event(&self, method: &str) -> Value {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < deadline {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50))
                && event.method == method
            {
                return event.params.unwrap();
            }
        }
        panic!("{method} nao chegou");
    }
}

const RELATIVE: &str = "dados/kinein-relativo-estacao.db";

#[test]
fn a_relative_sqlite_path_opens_the_project_file_and_stays_as_written() {
    // O teste roda com o diretorio do crate como corrente: ali nao ha' o
    // arquivo, entao abrir pelo diretorio do core falharia.
    assert!(!Path::new(RELATIVE).exists());
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dssqlite-relative", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("dados")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    rusqlite::Connection::open(dir.join(RELATIVE))
        .unwrap()
        .execute_batch(
            "CREATE TABLE leituras (id INTEGER PRIMARY KEY, valor REAL); \
             INSERT INTO leituras (valor) VALUES (1.5), (2.5), (3.5);",
        )
        .unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let mut c = Scenario {
        core,
        events: receiver,
    };
    let opened = c.rpc("workspace.open", json!({ "path": dir.to_str().unwrap() }));
    assert!(opened.error.is_none(), "{:?}", opened.error);
    let saved = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "estacao", "engine": "sqlite", "host": "", "port": 0,
                             "database": RELATIVE, "user": "" } }),
    );
    assert!(saved.error.is_none(), "{:?}", saved.error);

    let accepted = c.rpc("datasource.test", json!({ "name": "estacao" }));
    assert!(accepted.error.is_none(), "{:?}", accepted.error);
    let tested = c.event("event.datasource.tested");
    assert_eq!(tested["ok"], true, "{tested}");

    let accepted = c.rpc("datasource.introspect", json!({ "name": "estacao" }));
    assert!(accepted.error.is_none(), "{:?}", accepted.error);
    let read = c.event("event.datasource.introspected");
    assert_eq!(read["ok"], true, "{read}");
    assert_eq!(
        read["schemas"][0]["tables"][0]["name"], "leituras",
        "{read}"
    );

    // O perfil listado e' o que a pessoa escreveu; com ele no contexto, a
    // consulta passa (o core nao compara o caminho resolvido com o da UI).
    let listed = c.rpc("datasource.list", json!({}));
    let profile = listed.result.unwrap()["profiles"][0].clone();
    assert_eq!(profile["database"], RELATIVE, "{profile}");
    let accepted = c.rpc(
        "datasource.query",
        json!({ "name": "estacao", "sql": "SELECT count(*) FROM leituras",
                "clientContext": "t:1",
                "expectedContext": { "workspace": dir.to_str().unwrap(), "profile": profile } }),
    );
    assert!(accepted.error.is_none(), "{:?}", accepted.error);
    let queried = c.event("event.datasource.queried");
    assert_eq!(queried["success"], true, "{queried}");
    assert_eq!(queried["rows"], json!([["3"]]), "{queried}");

    let accepted = c.rpc(
        "datasource.impact",
        json!({ "name": "estacao", "sql": "DELETE FROM leituras WHERE id = 1" }),
    );
    assert!(accepted.error.is_none(), "{:?}", accepted.error);
    let impact = c.event("event.datasource.impact");
    assert_eq!(impact["statements"][0]["rows"], 1, "{impact}");
}
