//! `datasource.query` (0.121.0, `roadmaps/35` §7.4) pelo despacho, com o
//! UNICO motor que roda no gate sem servidor: um `SQLite` real em arquivo
//! temporario. O que se prova: leitura com teto e `truncated`, `NULL` como
//! `null`, a recusa `WRITE_CONFIRMATION_REQUIRED` antes do job, a escrita
//! confirmada com `affected`, o erro do motor em texto, e o perfil com TLS
//! que atravessa o `save` e volta (so' configuracao — nao ha' servidor
//! `PostgreSQL` com certificado nesta maquina).

use std::{sync::mpsc, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

struct Cenario {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn scenario(nome: &str) -> (Cenario, std::path::PathBuf) {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dsquery-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let mut c = Cenario {
        core,
        events: receiver,
    };
    let r = c.rpc("workspace.open", json!({ "path": dir.to_str().unwrap() }));
    assert!(r.error.is_none(), "{:?}", r.error);
    (c, dir)
}

impl Cenario {
    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn queried(&self) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < prazo {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == "event.datasource.queried"
            {
                return e.params.unwrap();
            }
        }
        panic!("event.datasource.queried nao chegou");
    }
}

fn database(dir: &std::path::Path) -> String {
    let caminho = dir.join("dados.db");
    let conexao = rusqlite::Connection::open(&caminho).unwrap();
    conexao
        .execute_batch(
            "CREATE TABLE leituras (id INTEGER PRIMARY KEY, placa TEXT, valor REAL); \
             INSERT INTO leituras (placa, valor) VALUES ('esp32', 21.5), ('pico', NULL), ('pi', 3);",
        )
        .unwrap();
    caminho.display().to_string()
}

#[test]
fn sqlite_query_reads_refuses_unconfirmed_writes_and_writes_when_confirmed() {
    let (mut c, dir) = scenario("sqlite");
    let salvo = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "arquivo", "engine": "sqlite", "host": "", "port": 0,
                             "database": database(&dir), "user": "" } }),
    );
    assert!(salvo.error.is_none(), "{:?}", salvo.error);

    // Vazio e' parametro invalido; perfil desconhecido idem.
    let vazio = c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "  " }),
    );
    assert_eq!(vazio.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
    let outro = c.rpc(
        "datasource.query",
        json!({ "name": "nao-existe", "sql": "SELECT 1" }),
    );
    assert!(outro.error.is_some());

    // Leitura com teto 2: duas linhas, truncated, NULL como null.
    let aceito = c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "SELECT id, placa, valor FROM leituras ORDER BY id", "maxRows": 2 }),
    );
    assert!(aceito.error.is_none(), "{:?}", aceito.error);
    assert!(aceito.result.unwrap()["jobId"].is_string());
    let ev = c.queried();
    assert_eq!(ev["success"], true, "{ev}");
    assert_eq!(ev["name"], "arquivo");
    assert_eq!(ev["columns"], json!(["id", "placa", "valor"]));
    assert_eq!(
        ev["rows"],
        json!([["1", "esp32", "21.5"], ["2", "pico", null]])
    );
    assert_eq!(ev["rowCount"], 2);
    assert_eq!(ev["truncated"], true);
    assert!(ev.get("affected").is_none());
    assert!(ev["elapsedMs"].is_u64());

    // Escrita sem confirmacao: recusa SINCRONA com codigo proprio; nada roda.
    let recusa = c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "DELETE FROM leituras WHERE placa = 'pi'" }),
    );
    let erro = recusa.error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::WriteConfirmationRequired);
    assert_eq!(erro.details.unwrap()["name"], "arquivo");

    // Confirmada: roda e conta.
    c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "DELETE FROM leituras WHERE placa = 'pi'", "confirmWrite": true }),
    );
    let ev = c.queried();
    assert_eq!(ev["success"], true, "{ev}");
    assert_eq!(ev["affected"], 1);
    assert_eq!(ev["columns"], json!([]));
    assert_eq!(ev["rowCount"], 0);
    c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "SELECT count(*) AS n FROM leituras" }),
    );
    let ev = c.queried();
    assert_eq!(ev["rows"], json!([["2"]]));
    assert_eq!(ev["truncated"], false);

    // O erro do motor vira texto; a leitura que escreve e' recusada pelo motor.
    c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "SELECT x FROM nada" }),
    );
    let ev = c.queried();
    assert_eq!(ev["success"], false);
    assert!(ev["message"].as_str().unwrap().contains("nada"), "{ev}");
    assert_eq!(ev["secretRequired"], false);
    c.rpc(
        "datasource.query",
        json!({ "name": "arquivo", "sql": "WITH x AS (SELECT 1) INSERT INTO leituras (placa) SELECT 'nunca' FROM x" }),
    );
    let ev = c.queried();
    assert_eq!(ev["success"], false);
    assert!(ev["message"].as_str().unwrap().contains("readonly"), "{ev}");
}

/// TLS e' campo do perfil: `require` + `caFile` sobrevivem ao save; `disable`
/// e' o mesmo que ausente e some; num motor que nao e' `PostgreSQL` some.
#[test]
fn tls_policy_is_saved_normalized_and_only_for_postgres() {
    let (mut c, _dir) = scenario("tls");
    let salvo = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "seguro", "host": "db.exemplo", "port": 5432, "database": "app",
                             "user": "app", "tls": "require", "caFile": " /etc/ssl/db.pem " } }),
    );
    let perfis = salvo.result.unwrap()["profiles"].clone();
    assert_eq!(perfis[0]["tls"], "require");
    assert_eq!(perfis[0]["caFile"], "/etc/ssl/db.pem");

    let plano = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "plano", "host": "localhost", "port": 5432, "database": "app",
                             "user": "app", "tls": "disable" } }),
    );
    let perfis = plano.result.unwrap()["profiles"].clone();
    assert!(perfis[0].get("tls").is_none(), "{perfis}");

    let sqlite = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "arq", "engine": "sqlite", "host": "", "port": 0,
                             "database": "/tmp/x.db", "user": "", "tls": "require" } }),
    );
    let perfis = sqlite.result.unwrap()["profiles"].clone();
    assert!(perfis[0].get("tls").is_none(), "{perfis}");

    let invalido = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "x", "host": "h", "port": 5432, "database": "d", "user": "u", "tls": "prefer" } }),
    );
    assert_eq!(
        invalido.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn ordinary_writes_run_directly_but_whole_table_updates_never_do() {
    let (mut c, dir) = scenario("selective");
    let path = database(&dir);
    assert!(
        c.rpc(
            "datasource.save",
            json!({"profile": {
                "name": "arquivo", "engine": "sqlite", "host": "", "port": 0,
                "database": path, "user": ""
            }})
        )
        .error
        .is_none()
    );
    for sql in [
        "INSERT INTO leituras (placa, valor) VALUES ('nova', 7)",
        "UPDATE leituras SET valor = 8 WHERE placa = 'nova'",
        "CREATE TABLE auxiliar (id int)",
        "ALTER TABLE auxiliar ADD COLUMN nome text",
    ] {
        let accepted = c.rpc("datasource.query", json!({"name": "arquivo", "sql": sql}));
        assert!(accepted.error.is_none(), "{sql}: {:?}", accepted.error);
        assert_eq!(c.queried()["success"], true, "{sql}");
    }
    for sql in [
        "DELETE FROM leituras WHERE placa = 'nova'",
        "UPDATE leituras SET valor = 0",
        "DROP TABLE auxiliar",
        "INSERT OR REPLACE INTO leituras (id, placa) VALUES (1, 'substituida')",
        "REPLACE INTO leituras (id, placa) VALUES (1, 'substituida')",
        "UPDATE OR REPLACE leituras SET id = 1 WHERE id = 2",
        "SELECT 1; COMMIT; DELETE FROM leituras",
        "SELECT ';' AS texto; DELETE FROM leituras",
        "INSERT INTO leituras (placa) VALUES ('nao'); DELETE FROM leituras",
    ] {
        let refused = c.rpc("datasource.query", json!({"name": "arquivo", "sql": sql}));
        assert_eq!(
            refused.error.unwrap().code,
            JsonRpcErrorCode::WriteConfirmationRequired,
            "{sql}"
        );
    }
    let whole = "UPDATE leituras SET valor = 999 WHERE id > 0";
    assert!(
        c.rpc("datasource.query", json!({"name": "arquivo", "sql": whole}))
            .error
            .is_none()
    );
    let measured = c.queried();
    assert_eq!(measured["confirmationSql"], whole, "{measured}");
    assert_eq!(measured["success"], false);
    let connection = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM leituras WHERE valor = 999", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        0
    );
    // Nao conseguir contar nao significa que pode escrever.
    assert!(
        c.rpc(
            "datasource.query",
            json!({"name": "arquivo",
        "sql": "UPDATE leituras SET valor = 5 WHERE unknown(id)"})
        )
        .error
        .is_none()
    );
    assert_eq!(
        c.queried()["confirmationSql"],
        "UPDATE leituras SET valor = 5 WHERE unknown(id)"
    );
    // Depois da confirmacao explicita, executa o MESMO comando.
    assert!(
        c.rpc(
            "datasource.query",
            json!({"name": "arquivo", "sql": whole,
        "confirmWrite": true})
        )
        .error
        .is_none()
    );
    assert_eq!(c.queried()["affected"], 4);
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM leituras WHERE valor = 999", [], |r| r
                .get::<_, u64>(0))
            .unwrap(),
        4
    );
    drop(connection);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn mongo_deletions_require_confirmation_before_any_connection() {
    let (mut c, dir) = scenario("mongo-refusal");
    assert!(
        c.rpc(
            "datasource.save",
            json!({"profile": {
                "name": "mongo", "engine": "mongo", "host": "127.0.0.1", "port": 1,
                "database": "teste", "user": ""
            }})
        )
        .error
        .is_none()
    );
    for sql in [
        r#"s.deleteOne({"id": 1})"#,
        r#"s.deleteMany({"id": 1})"#,
        "s.drop()",
        r#"s.updateMany({}, {"$set": {"x": 1}})"#,
    ] {
        let refused = c.rpc("datasource.query", json!({"name": "mongo", "sql": sql}));
        assert_eq!(
            refused.error.unwrap().code,
            JsonRpcErrorCode::WriteConfirmationRequired,
            "{sql}"
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}
