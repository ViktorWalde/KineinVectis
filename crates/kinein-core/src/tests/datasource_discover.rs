//! `datasource.discover` e `datasource.create` (0.124.0) pelo despacho, com
//! um `podman` FALSO no PATH (ecoa um `ps` com um container de `PostgreSQL`
//! parado e registra o `run`) e um `SQLite` real no workspace. O que se
//! prova: o container vira candidato com a porta publicada; o `.sqlite` do
//! projeto vira candidato e o lixo em `target/` nao; `sqliteFile` cria o
//! arquivo, salva o perfil e recusa criar por cima; `containerServer` roda o
//! motor com o comando pinado no loopback, salva o perfil e emite
//! `event.datasource.created`; sem motor no PATH, `TOOL_NOT_FOUND`.

use std::{os::unix::fs::PermissionsExt, path::PathBuf, sync::mpsc, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

struct Cenario {
    core: crate::Core,
    events: mpsc::Receiver<JsonRpcRequest>,
    dir: PathBuf,
}

const PODMAN_FALSO: &str = r#"
case "$1" in
  --version|version) echo "podman version 5.7.0" ;;
  ps) cat <<'JSON'
[{"Id":"abc123","Names":["kinein-pg"],"Image":"docker.io/library/postgres:16","State":"exited","Status":"Exited (0) 2 hours ago","Ports":[{"host_ip":"127.0.0.1","container_port":5432,"host_port":5433,"protocol":"tcp"}],"Created":"2026-09-18T10:00:00Z"}]
JSON
  ;;
  run) echo "$@" > "$(dirname "$0")/run.argv"; echo "deadbeef" ;;
  rm) echo "$@" > "$(dirname "$0")/rm.argv"; echo "abc123" ;;
  *) echo "podman falso: $*" >&2; exit 1 ;;
esac
"#;

fn cenario(nome: &str, com_motor: bool) -> Cenario {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dsdiscover-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    if com_motor {
        let podman = dir.join("bin/podman");
        std::fs::write(&podman, format!("#!/bin/sh\n{PODMAN_FALSO}")).unwrap();
        std::fs::set_permissions(&podman, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let mut c = Cenario {
        core,
        events: receiver,
        dir,
    };
    let r = c.rpc("workspace.open", json!({ "path": c.dir.to_str().unwrap() }));
    assert!(r.error.is_none(), "{:?}", r.error);
    c
}

impl Cenario {
    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(9_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn evento(&self, nome: &str) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < prazo {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == nome
            {
                return e.params.unwrap();
            }
        }
        panic!("{nome} nao chegou");
    }
}

fn arquivo_sqlite(caminho: &std::path::Path) {
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    let mut bytes = b"SQLite format 3\0".to_vec();
    bytes.extend(std::iter::repeat_n(0_u8, 100));
    std::fs::write(caminho, bytes).unwrap();
}

#[test]
fn discover_lists_the_database_container_and_the_workspace_sqlite_files() {
    let mut c = cenario("lista", true);
    arquivo_sqlite(&c.dir.join("data/app.sqlite"));
    arquivo_sqlite(&c.dir.join("target/lixo.sqlite"));

    let r = c.rpc("datasource.discover", json!({}));
    assert!(r.error.is_none(), "{:?}", r.error);
    let resultado = r.result.unwrap();
    assert_eq!(resultado["containerEngine"], "podman");
    let candidatos = resultado["candidates"].as_array().unwrap();
    let container = candidatos
        .iter()
        .find(|c| c["kind"] == "container")
        .expect("o container de postgres");
    assert_eq!(container["label"], "container kinein-pg");
    assert_eq!(
        container["running"], false,
        "parado e' listado, nao rodando"
    );
    assert_eq!(container["profile"]["host"], "127.0.0.1");
    assert_eq!(container["profile"]["port"], 5433);
    assert_eq!(container["profile"]["engine"], "postgres");
    let arquivos: Vec<&Value> = candidatos.iter().filter(|c| c["kind"] == "file").collect();
    assert_eq!(arquivos.len(), 1, "{candidatos:?}");
    assert_eq!(arquivos[0]["label"], "data/app.sqlite");
    assert_eq!(arquivos[0]["profile"]["engine"], "sqlite");
    for c in candidatos {
        assert!(
            c["profile"].get("password").is_none(),
            "perfil nunca carrega senha"
        );
    }
}

#[test]
fn create_sqlite_file_saves_the_profile_and_refuses_to_overwrite() {
    let mut c = cenario("sqlite", false);
    let r = c.rpc(
        "datasource.create",
        json!({ "kind": "sqliteFile", "name": "notas" }),
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    let perfil = r.result.unwrap()["profile"].clone();
    assert_eq!(perfil["engine"], "sqlite");
    let caminho = perfil["database"].as_str().unwrap().to_owned();
    assert!(caminho.ends_with("data/notas.sqlite"), "{caminho}");
    assert!(
        std::fs::read(&caminho)
            .unwrap()
            .starts_with(b"SQLite format 3\0")
    );

    let lista = c.rpc("datasource.list", json!({}));
    assert_eq!(lista.result.unwrap()["profiles"][0]["name"], "notas");

    let de_novo = c.rpc(
        "datasource.create",
        json!({ "kind": "sqliteFile", "name": "notas" }),
    );
    assert_eq!(de_novo.error.unwrap().code, JsonRpcErrorCode::InvalidParams);

    // Agora o discover ve o arquivo que o create fez (o cabecalho e' real).
    let d = c.rpc("datasource.discover", json!({}));
    let candidatos = d.result.unwrap()["candidates"].clone();
    assert!(
        candidatos
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["label"] == "data/notas.sqlite"),
        "{candidatos}"
    );

    let invalido = c.rpc(
        "datasource.create",
        json!({ "kind": "sqliteFile", "name": "../fora" }),
    );
    assert_eq!(
        invalido.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn create_container_server_runs_the_pinned_command_and_saves_the_profile() {
    let mut c = cenario("servidor", true);
    let r = c.rpc(
        "datasource.create",
        json!({ "kind": "containerServer", "engine": "postgres", "name": "dev", "port": 5433 }),
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    let resultado = r.result.unwrap();
    assert!(resultado["jobId"].is_string());
    assert_eq!(
        resultado["command"],
        "podman run -d --name kinein-dev -p 127.0.0.1:5433:5432 -e POSTGRES_HOST_AUTH_METHOD=trust docker.io/library/postgres:16"
    );

    let criado = c.evento("event.datasource.created");
    assert_eq!(criado["success"], true, "{criado}");
    assert_eq!(criado["profile"]["name"], "dev");
    assert_eq!(criado["profile"]["port"], 5433);
    assert_eq!(criado["profile"]["user"], "postgres");

    let argv = std::fs::read_to_string(c.dir.join("bin/run.argv")).unwrap();
    assert!(argv.contains("-p 127.0.0.1:5433:5432"), "{argv}");
    assert!(argv.contains("postgres:16"), "{argv}");

    let lista = c.rpc("datasource.list", json!({}));
    assert_eq!(lista.result.unwrap()["profiles"][0]["name"], "dev");

    let sqlite = c.rpc(
        "datasource.create",
        json!({ "kind": "containerServer", "engine": "sqlite", "name": "x", "port": 1 }),
    );
    assert_eq!(sqlite.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
}

#[test]
fn create_container_server_without_an_engine_is_tool_not_found() {
    let mut c = cenario("sem-motor", false);
    let r = c.rpc(
        "datasource.create",
        json!({ "kind": "containerServer", "engine": "mongo", "name": "docs", "port": 27017 }),
    );
    assert_eq!(r.error.unwrap().code, JsonRpcErrorCode::ToolNotFound);
    let d = c.rpc("datasource.discover", json!({}));
    let resultado = d.result.unwrap();
    assert!(resultado.get("containerEngine").is_none());
    assert!(
        resultado["hint"]
            .as_str()
            .unwrap()
            .contains("sem Podman/Docker")
    );
}

/// `datasource.destroy` (0.129.0): sem `data`, so' o perfil; com `data`, o
/// arquivo `SQLite` do workspace some (fora dele fica, com a nota), o
/// container `kinein-<nome>` leva `rm -f` (job + evento) e o perfil sai; um
/// Mongo de servidor so' perde o perfil, com a nota.
#[test]
fn destroy_removes_the_profile_and_optionally_the_data() {
    let mut c = cenario("destroy", true);
    // SQLite criado pela IDE: com data, o arquivo some.
    let criado = c.rpc(
        "datasource.create",
        json!({ "kind": "sqliteFile", "name": "notas" }),
    );
    let caminho = criado.result.unwrap()["profile"]["database"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(std::path::Path::new(&caminho).exists());
    let r = c.rpc(
        "datasource.destroy",
        json!({ "name": "notas", "data": true }),
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    let resultado = r.result.unwrap();
    assert!(
        !std::path::Path::new(&caminho).exists(),
        "o arquivo devia ter sumido"
    );
    assert!(resultado["profiles"].as_array().unwrap().is_empty());
    assert!(resultado.get("note").is_none());

    // SQLite fora do workspace: fica, com a nota; o perfil sai.
    let fora = std::env::temp_dir().join(format!("kinein-fora-{}.sqlite", std::process::id()));
    std::fs::write(&fora, b"SQLite format 3\0").unwrap();
    c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "fora", "engine": "sqlite", "host": "", "port": 0,
        "database": fora.to_str().unwrap(), "user": "" } }),
    );
    let r = c.rpc(
        "datasource.destroy",
        json!({ "name": "fora", "data": true }),
    );
    let resultado = r.result.unwrap();
    assert!(fora.exists(), "fora do workspace a IDE nao apaga");
    assert!(
        resultado["note"]
            .as_str()
            .unwrap()
            .contains("fora do workspace")
    );
    let _ = std::fs::remove_file(&fora);

    // Servidor em container (o podman falso lista `kinein-pg`): rm -f + perfil fora.
    c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "pg", "engine": "postgres", "host": "127.0.0.1",
        "port": 5433, "database": "postgres", "user": "postgres" } }),
    );
    let r = c.rpc("datasource.destroy", json!({ "name": "pg", "data": true }));
    assert!(r.error.is_none(), "{:?}", r.error);
    let resultado = r.result.unwrap();
    assert_eq!(resultado["command"], "podman rm -f kinein-pg");
    let evento = c.evento("event.datasource.destroyed");
    assert_eq!(evento["success"], true, "{evento}");
    assert!(
        std::fs::read_to_string(c.dir.join("bin/rm.argv"))
            .unwrap()
            .contains("-f kinein-pg")
    );
    assert!(
        evento["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["name"] != "pg")
    );

    // Sem data: so' o perfil, seja o que for.
    c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "docs", "engine": "mongo", "host": "127.0.0.1",
        "port": 27017, "database": "test", "user": "" } }),
    );
    let r = c.rpc("datasource.destroy", json!({ "name": "docs" }));
    assert!(
        r.result.unwrap()["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["name"] != "docs")
    );
    // Mongo com data: o servidor fica, a nota diz.
    c.rpc(
        "datasource.save",
        json!({ "profile": { "name": "docs2", "engine": "mongo", "host": "127.0.0.1",
        "port": 27017, "database": "test", "user": "" } }),
    );
    let r = c.rpc(
        "datasource.destroy",
        json!({ "name": "docs2", "data": true }),
    );
    assert!(
        r.result.unwrap()["note"]
            .as_str()
            .unwrap()
            .contains("MongoDB")
    );

    let inexistente = c.rpc("datasource.destroy", json!({ "name": "nada" }));
    assert_eq!(
        inexistente.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
}
