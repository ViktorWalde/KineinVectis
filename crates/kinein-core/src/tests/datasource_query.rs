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

fn cenario(nome: &str) -> (Cenario, std::path::PathBuf) {
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

fn banco(dir: &std::path::Path) -> String {
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
    let (mut c, dir) = cenario("sqlite");
    let salvo = c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "arquivo", "engine": "sqlite", "host": "", "port": 0,
                             "database": banco(&dir), "user": "" } }),
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
    let (mut c, _dir) = cenario("tls");
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
