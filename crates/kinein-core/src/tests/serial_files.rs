//! `serial.files` (C2 do `roadmaps/41` bloco C, 2026-09-17): os arquivos na
//! placa pelo `mpremote fs`, como JOB, com o mpremote FALSO ecoando os argv.
//! O que se prova: a linha de cada acao; o parser sobre a saida REAL do
//! ESP32 do autor; `local` relativo sob o workspace; a repeticao unica
//! quando a placa nao deixa entrar no raw REPL; o erro na linha `mpremote:`;
//! as recusas ANTES de tocar a porta (pedido invalido, porta, ferramenta);
//! o risco por acao; e o cancelamento matando o processo.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

/// A saida real de `mpremote connect /dev/ttyUSB0 fs ls` no ESP32 do autor
/// (2026-09-17, mpremote 1.29.0).
const LS_REAL: &str = "ls :\n         139 boot.py\n         785 config.py\n       13588 main.py\n        1137 wifi_lib.py\n";

struct Cenario {
    core: crate::Core,
    dir: PathBuf,
    porta: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str) -> Cenario {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-serial-files-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join("ws")).unwrap();
    let porta = dir.join("ttyUSB0");
    std::fs::write(&porta, "").unwrap();
    let dir = dir.canonicalize().unwrap();
    let porta = dir.join("ttyUSB0");
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    Cenario {
        core,
        dir,
        porta,
        events: receiver,
    }
}

fn executavel(caminho: &Path, corpo: &str) {
    std::fs::write(caminho, corpo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

impl Cenario {
    /// O mpremote falso: ecoa `argv:` e depois o corpo dado.
    fn mpremote_falso(&self, corpo: &str) {
        executavel(
            &self.dir.join("bin/mpremote"),
            &format!("#!/bin/sh\necho \"argv: $*\"\n{corpo}"),
        );
    }

    fn abrir_workspace(&mut self) {
        let r = self.core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": self.dir.join("ws").to_str().unwrap() })),
        ));
        assert!(r.response().error.is_none(), "{:?}", r.response().error);
        // O que o workspace.open emitiu nao interessa aqui.
        while self.events.try_recv().is_ok() {}
    }

    fn files(&mut self, mut params: Value) -> kinein_protocol::JsonRpcResponse {
        if params.get("device").is_none() {
            params["device"] = json!(self.porta.display().to_string());
        }
        self.core
            .handle_request(&JsonRpcRequest::new(7_i64, "serial.files", Some(params)))
            .response()
            .clone()
    }

    fn desfecho(&self) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(15);
        while std::time::Instant::now() < prazo {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50))
                && event.method == "event.serial.files"
            {
                return event.params.unwrap();
            }
        }
        panic!("event.serial.files nao chegou");
    }

    fn job(&mut self, id: &str) -> Value {
        let lista = self
            .core
            .handle_request(&JsonRpcRequest::new(8_i64, "job.list", Some(json!({}))))
            .response()
            .result
            .clone()
            .unwrap();
        lista["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|j| j["id"] == id)
            .cloned()
            .unwrap_or(Value::Null)
    }
}

/// `list`: a linha com `:` (a raiz), o parser sobre a saida real, o risco
/// medio (interrompe o programa da placa, nao escreve).
#[test]
#[cfg(unix)]
fn listing_reads_the_real_output_and_is_a_medium_risk_job() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("ls");
    c.mpremote_falso(&format!("printf '%s' '{LS_REAL}'\n"));
    let porta = c.porta.display().to_string();
    let resposta = c.files(json!({ "action": "list" }));
    let resultado = resposta.result.expect("aceita");
    let job_id = resultado["jobId"].as_str().unwrap().to_owned();
    assert_eq!(
        resultado["command"],
        format!(
            "{} connect {porta} fs ls :",
            c.dir.join("bin/mpremote").display()
        )
    );
    let evento = c.desfecho();
    assert_eq!(evento["success"], true, "{evento}");
    assert_eq!(evento["action"], "list");
    assert_eq!(evento["path"], "");
    assert_eq!(evento["device"], porta);
    let entradas = evento["entries"].as_array().unwrap();
    assert_eq!(entradas.len(), 4);
    assert_eq!(entradas[2]["name"], "main.py");
    assert_eq!(entradas[2]["size"], 13588);
    assert_eq!(entradas[2]["directory"], false);
    assert!(
        evento["raw"]
            .as_str()
            .unwrap()
            .contains(&format!("argv: connect {porta} fs ls :"))
    );
    let job = c.job(&job_id);
    assert_eq!(job["kind"], "serial.files");
    assert_eq!(job["risk"], "medium", "{job}");

    // Uma pasta: `ls :lib`.
    let r = c
        .files(json!({ "action": "list", "path": "lib" }))
        .result
        .unwrap();
    assert!(r["command"].as_str().unwrap().ends_with(" fs ls :lib"));
    c.desfecho();
}

/// `get` baixa para o workspace (`local` relativo), `put` envia o arquivo
/// que existe, `rm`/`mkdir` sao alto risco — e cada um leva o `:` do lado
/// certo.
#[test]
#[cfg(unix)]
fn get_put_rm_and_mkdir_build_their_lines_and_writes_are_high_risk() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("cp");
    c.abrir_workspace();
    c.mpremote_falso("exit 0\n");
    let porta = c.porta.display().to_string();
    let ws = c.dir.join("ws");

    let r = c
        .files(json!({ "action": "get", "path": "main.py", "local": "main.py" }))
        .result
        .unwrap();
    assert!(
        r["command"].as_str().unwrap().ends_with(&format!(
            " connect {porta} fs cp :main.py {}",
            ws.join("main.py").display()
        )),
        "{r}"
    );
    let evento = c.desfecho();
    assert_eq!(evento["success"], true);
    assert_eq!(evento["local"], ws.join("main.py").display().to_string());
    assert_eq!(evento["entries"], Value::Null);
    assert_eq!(c.job(r["jobId"].as_str().unwrap())["risk"], "medium");

    std::fs::write(ws.join("main.py"), "print(1)\n").unwrap();
    let r = c
        .files(json!({ "action": "put", "path": "main.py", "local": "main.py" }))
        .result
        .unwrap();
    assert!(
        r["command"]
            .as_str()
            .unwrap()
            .ends_with(&format!(" fs cp {} :main.py", ws.join("main.py").display())),
        "{r}"
    );
    c.desfecho();
    assert_eq!(c.job(r["jobId"].as_str().unwrap())["risk"], "high");

    let r = c
        .files(json!({ "action": "rm", "path": "old.py" }))
        .result
        .unwrap();
    assert!(r["command"].as_str().unwrap().ends_with(" fs rm :old.py"));
    c.desfecho();
    assert_eq!(c.job(r["jobId"].as_str().unwrap())["risk"], "high");

    let r = c
        .files(json!({ "action": "mkdir", "path": "lib" }))
        .result
        .unwrap();
    assert!(r["command"].as_str().unwrap().ends_with(" fs mkdir :lib"));
    assert_eq!(c.desfecho()["action"], "mkdir");
    assert_eq!(c.job(r["jobId"].as_str().unwrap())["risk"], "high");
}

/// A primeira conexao morre no raw REPL (medido no ESP32 real): o job
/// repete UMA vez, e a segunda vale. A terceira nunca acontece.
#[test]
#[cfg(unix)]
fn a_raw_repl_failure_is_retried_once() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("retry");
    let marca = c.dir.join("primeira");
    c.mpremote_falso(&format!(
        "if [ ! -f '{m}' ]; then touch '{m}'; echo 'mpremote.transport.TransportError: could not enter raw repl' >&2; exit 1; fi\nprintf '%s' '{LS_REAL}'\n",
        m = marca.display()
    ));
    assert!(c.files(json!({ "action": "list" })).error.is_none());
    let evento = c.desfecho();
    assert_eq!(evento["success"], true, "{evento}");
    assert_eq!(evento["entries"].as_array().unwrap().len(), 4);
    assert_eq!(
        evento["raw"].as_str().unwrap().matches("argv:").count(),
        2,
        "{}",
        evento["raw"]
    );

    // Sempre falhando: duas tentativas e o erro e' o do raw REPL.
    std::fs::remove_file(&marca).unwrap();
    c.mpremote_falso(
        "echo 'mpremote.transport.TransportError: could not enter raw repl' >&2\nexit 1\n",
    );
    assert!(c.files(json!({ "action": "list" })).error.is_none());
    let evento = c.desfecho();
    assert_eq!(evento["success"], false);
    assert_eq!(evento["raw"].as_str().unwrap().matches("argv:").count(), 2);
    assert!(
        evento["error"]
            .as_str()
            .unwrap()
            .contains("could not enter raw repl"),
        "{evento}"
    );
}

/// O erro da tela e' a linha `mpremote: …` (sem o prefixo), como o
/// mpremote a escreve para um arquivo que nao existe na placa.
#[test]
#[cfg(unix)]
fn the_error_is_the_mpremote_line() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("erro");
    c.mpremote_falso(
        "echo 'cp :nao.py ./nao.py'\necho 'mpremote: cp: nao.py: No such file or directory.' >&2\nexit 1\n",
    );
    let destino = c.dir.join("nao.py").display().to_string();
    assert!(
        c.files(json!({ "action": "get", "path": "nao.py", "local": destino }))
            .error
            .is_none()
    );
    let evento = c.desfecho();
    assert_eq!(evento["success"], false);
    assert_eq!(evento["error"], "cp: nao.py: No such file or directory.");
}

/// Tudo que e' recusado ANTES de tocar a porta — e nenhum job nasce.
#[test]
#[cfg(unix)]
fn refusals_happen_before_the_port_is_touched() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("recusas");
    c.mpremote_falso("exit 0\n");
    let porta = c.porta.display().to_string();

    // Pedido invalido: `:` na frente; rm sem path; get sem local; put de
    // arquivo que nao existe; local relativo sem workspace; acao desconhecida.
    for (params, trecho) in [
        (json!({ "action": "list", "path": ":lib" }), "sem `:`"),
        (json!({ "action": "rm" }), "exige o campo path"),
        (
            json!({ "action": "get", "path": "a.py" }),
            "exige o campo local",
        ),
        (
            json!({ "action": "put", "path": "a.py", "local": c.dir.join("nao.py").display().to_string() }),
            "nao ha' arquivo",
        ),
        (
            json!({ "action": "get", "path": "a.py", "local": "a.py" }),
            "exige um workspace aberto",
        ),
    ] {
        let erro = c
            .files(params.clone())
            .error
            .unwrap_or_else(|| panic!("{params}"));
        assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams, "{params}");
        assert!(erro.message.contains(trecho), "{params}: {}", erro.message);
    }
    assert_eq!(
        c.files(json!({ "action": "tree" })).error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // Porta inexistente; sem permissao.
    let erro = c
        .files(json!({ "device": c.dir.join("nao").display().to_string(), "action": "list" }))
        .error
        .unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    std::fs::set_permissions(&c.porta, std::fs::Permissions::from_mode(0o444)).unwrap();
    let erro = c.files(json!({ "action": "list" })).error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest, "{erro:?}");
    std::fs::set_permissions(&c.porta, std::fs::Permissions::from_mode(0o644)).unwrap();

    // Sem mpremote.
    std::fs::remove_file(c.dir.join("bin/mpremote")).unwrap();
    let erro = c.files(json!({ "action": "list" })).error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound);
    assert!(
        erro.message.contains("pipx install mpremote"),
        "{}",
        erro.message
    );
    let _ = porta;

    let jobs = c
        .core
        .handle_request(&JsonRpcRequest::new(8_i64, "job.list", Some(json!({}))))
        .response()
        .result
        .clone()
        .unwrap();
    assert!(jobs["jobs"].as_array().unwrap().is_empty(), "{jobs}");
}

/// Cancelar o job MATA o mpremote e o desfecho diz "cancelada".
#[test]
#[cfg(unix)]
fn cancelling_the_job_kills_mpremote() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("cancela");
    c.mpremote_falso("echo 'ls :'\nexec sleep 30\n");
    let job_id = c.files(json!({ "action": "list" })).result.unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let prazo = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(event) = c.events.recv_timeout(Duration::from_millis(50))
            && event.method == "event.job.output"
            && event.params.as_ref().unwrap()["line"] == "ls :"
        {
            break;
        }
        assert!(
            std::time::Instant::now() < prazo,
            "o mpremote falso nao comecou"
        );
    }
    let inicio = std::time::Instant::now();
    let cancelado = c
        .core
        .handle_request(&JsonRpcRequest::new(
            9_i64,
            "job.cancel",
            Some(json!({ "jobId": job_id })),
        ))
        .response()
        .clone();
    assert!(cancelado.error.is_none(), "{:?}", cancelado.error);
    let evento = c.desfecho();
    assert!(
        inicio.elapsed() < Duration::from_secs(10),
        "o sleep nao foi morto"
    );
    assert_eq!(evento["success"], false);
    assert_eq!(evento["error"], "operacao cancelada");
}
