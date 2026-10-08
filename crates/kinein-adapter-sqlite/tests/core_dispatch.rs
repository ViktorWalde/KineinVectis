//! O perfil com `installation: { kind: "ide" }` pelo DESPACHO real do `Core`
//! (passo 9a.2, `arquitetura/39` §6.2.1): teste, catalogo (com as instrucoes do
//! core), consulta, escrita, a mensagem do banco limitada, impacto, somente
//! leitura, salvar encerrando a instancia e o adaptador ausente. Mora no crate
//! do adaptador porque e' aqui que o binario existe.

#[cfg(test)]
mod dispatch {
    use std::{path::PathBuf, sync::mpsc, time::Duration};

    use kinein_core::datasource::external::Adapters;
    use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
    use serde_json::{Value, json};

    struct Scenario {
        core: kinein_core::Core,
        events: mpsc::Receiver<JsonRpcRequest>,
        dir: PathBuf,
    }

    fn bin_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_BIN_EXE_kinein-adapter-sqlite"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    impl Scenario {
        fn new(name: &str, adapters: Adapters) -> Self {
            let dir = std::env::temp_dir()
                .join("kinein-adapter-sqlite-tests")
                .join(format!("{}-dispatch-{name}", std::process::id()));
            drop(std::fs::remove_dir_all(&dir));
            std::fs::create_dir_all(dir.join("dados")).unwrap();
            std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
            rusqlite::Connection::open(dir.join("dados/estacao.db"))
                .unwrap()
                .execute_batch(
                    "CREATE TABLE leituras (id INTEGER PRIMARY KEY, sensor TEXT); \
                     INSERT INTO leituras (sensor) VALUES ('a'), ('b'), (NULL);",
                )
                .unwrap();
            let (sender, events) = mpsc::channel();
            let mut core = kinein_core::Core::with_detector(
                kinein_core::tools::ToolDetector::with_search_path(dir.join("bin")),
            );
            core.enable_lsp(sender);
            core.use_adapters(adapters);
            let mut scenario = Self { core, events, dir };
            let opened = scenario.rpc(
                "workspace.open",
                json!({ "path": scenario.dir.to_str().unwrap() }),
            );
            assert!(opened.error.is_none(), "{:?}", opened.error);
            scenario
        }

        fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
            self.core
                .handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
                .response()
                .clone()
        }

        fn save(&mut self, read_only: bool) {
            let saved = self.rpc(
                "datasource.save",
                json!({ "profile": { "name": "estacao", "engine": "sqlite", "host": "", "port": 0,
                                     "database": "dados/estacao.db", "user": "", "readOnly": read_only,
                                     "installation": { "kind": "ide" } } }),
            );
            assert!(saved.error.is_none(), "{:?}", saved.error);
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

        fn query(&mut self, sql: &str) -> Value {
            let accepted = self.rpc("datasource.query", json!({ "name": "estacao", "sql": sql }));
            assert!(accepted.error.is_none(), "{:?}", accepted.error);
            self.event("event.datasource.queried")
        }
    }

    #[test]
    fn the_ide_installation_runs_through_the_adapter() {
        let adapters = Adapters::in_dir(bin_dir());
        let mut c = Scenario::new("ligado", adapters.clone());
        c.save(false);
        let listed = c.rpc("datasource.list", json!({}));
        let result = listed.result.unwrap();
        assert_eq!(
            result["profiles"][0]["installation"],
            json!({ "kind": "ide" })
        );
        let sqlite = result["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["engine"] == "sqlite")
            .unwrap()
            .clone();
        assert_eq!(sqlite["installations"], json!(["builtin", "ide"]));

        assert!(
            c.rpc("datasource.test", json!({ "name": "estacao" }))
                .error
                .is_none()
        );
        let tested = c.event("event.datasource.tested");
        assert_eq!(tested["ok"], true, "{tested}");
        assert_eq!(adapters.live(), 1, "uma instancia aberta pelo teste");

        assert!(
            c.rpc("datasource.introspect", json!({ "name": "estacao" }))
                .error
                .is_none()
        );
        let read = c.event("event.datasource.introspected");
        let table = &read["schemas"][0]["tables"][0];
        assert_eq!(table["name"], "leituras", "{read}");
        // As instrucoes saem do CORE, nao do adaptador.
        assert_eq!(
            table["statements"]["select"], "SELECT * FROM \"main\".\"leituras\";",
            "{read}"
        );

        let rows = c.query("SELECT sensor FROM leituras ORDER BY id");
        assert_eq!(rows["success"], true, "{rows}");
        assert_eq!(rows["rows"], json!([["a"], ["b"], [null]]));
        let wrote = c.query("INSERT INTO leituras (sensor) VALUES ('c')");
        assert_eq!(wrote["affected"], 1, "{wrote}");
        let broken = c.query("SELECT * FROM nada");
        assert_eq!(broken["success"], false);
        assert!(
            broken["message"]
                .as_str()
                .unwrap()
                .contains("no such table"),
            "{broken}"
        );
        assert_eq!(adapters.live(), 1, "a mesma instancia serve as operacoes");

        let measured = c.rpc(
            "datasource.impact",
            json!({ "name": "estacao", "sql": "DELETE FROM leituras WHERE sensor = 'a'" }),
        );
        assert!(measured.error.is_none(), "{:?}", measured.error);
        let impact = c.event("event.datasource.impact");
        assert_eq!(impact["statements"][0]["rows"], 1, "{impact}");

        // Salvar (mudar o perfil) encerra a instancia; somente leitura recusa
        // a escrita no core, antes de qualquer processo.
        c.save(true);
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while adapters.live() != 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(adapters.live(), 0, "salvar nao encerrou a instancia");
        let refused = c.rpc(
            "datasource.query",
            json!({ "name": "estacao", "sql": "DELETE FROM leituras" }),
        );
        assert!(refused.error.is_some());
        let count = c.query("SELECT count(*) FROM leituras");
        assert_eq!(count["rows"], json!([["4"]]), "{count}");

        // Trocar de projeto encerra tudo.
        let closed = c.rpc("workspace.close", json!({}));
        assert!(closed.error.is_none(), "{:?}", closed.error);
        assert_eq!(adapters.live(), 0);
    }

    #[test]
    fn each_operation_opens_the_adapter_by_itself() {
        // O caminho interno daria o mesmo resultado; o que prova o roteamento
        // e' a instancia que CADA operacao, sendo a primeira, abre.
        for (name, method, params, event) in [
            (
                "teste",
                "datasource.test",
                json!({ "name": "estacao" }),
                "event.datasource.tested",
            ),
            (
                "catalogo",
                "datasource.introspect",
                json!({ "name": "estacao" }),
                "event.datasource.introspected",
            ),
            (
                "consulta",
                "datasource.query",
                json!({ "name": "estacao", "sql": "SELECT 1" }),
                "event.datasource.queried",
            ),
            (
                "impacto",
                "datasource.impact",
                json!({ "name": "estacao", "sql": "DELETE FROM leituras WHERE id = 1" }),
                "event.datasource.impact",
            ),
        ] {
            let adapters = Adapters::in_dir(bin_dir());
            let mut c = Scenario::new(name, adapters.clone());
            c.save(false);
            assert_eq!(adapters.live(), 0);
            assert!(c.rpc(method, params).error.is_none(), "{method}");
            c.event(event);
            assert_eq!(adapters.live(), 1, "{method} nao passou pelo adaptador");
        }
    }

    #[test]
    fn a_missing_adapter_says_where_it_looked() {
        let empty =
            std::env::temp_dir().join(format!("kinein-sem-adaptador-{}", std::process::id()));
        std::fs::create_dir_all(&empty).unwrap();
        let mut c = Scenario::new("ausente", Adapters::in_dir(empty.clone()));
        c.save(false);
        assert!(
            c.rpc("datasource.test", json!({ "name": "estacao" }))
                .error
                .is_none()
        );
        let tested = c.event("event.datasource.tested");
        assert_eq!(tested["ok"], false);
        let message = tested["message"].as_str().unwrap();
        assert!(
            message.contains("kinein-adapter-sqlite") && message.contains(empty.to_str().unwrap()),
            "{message}"
        );
    }

    #[test]
    fn an_engine_without_the_ide_adapter_refuses_the_choice() {
        let mut c = Scenario::new("recusa", Adapters::in_dir(bin_dir()));
        let saved = c.rpc(
            "datasource.save",
            json!({ "profile": { "name": "servidor", "engine": "postgres", "host": "db", "port": 5432,
                                 "database": "a", "user": "u", "installation": { "kind": "ide" } } }),
        );
        assert_eq!(saved.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
    }
}
