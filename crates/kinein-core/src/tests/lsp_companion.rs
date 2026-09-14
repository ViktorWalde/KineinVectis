//! Dois language servers para UMA linguagem — o principal e o companheiro
//! (2026-09-13, o `ruff server` ao lado do basedpyright; `roadmaps/40` §4).
//!
//! O que se prova, com dois servidores FALSOS que publicam nomes diferentes
//! para o mesmo arquivo: o texto chega aos dois; os diagnosticos saem FUNDIDOS
//! num evento so' (a UI substitui por arquivo — dois eventos parciais se
//! apagariam); as code actions dos dois vem numa lista e aplicar a do
//! companheiro edita o arquivo; tirar o companheiro apaga so' a parte dele.

use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::lsp_server::{fake_server, python3};
use crate::lsp;

const DEADLINE: Duration = Duration::from_secs(10);

struct Dupla {
    core: crate::Core,
    root: PathBuf,
    log_principal: PathBuf,
    log_companheiro: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn dupla(name: &str) -> Dupla {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspdupla-{name}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(root.join("app.py"), "import os\n").unwrap();
    let root = root.canonicalize().unwrap();
    let log_principal = root.join("principal.jsonl");
    let log_companheiro = root.join("companheiro.jsonl");

    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = super::core_with_empty_search_path(&format!("lspdupla-{name}"));
    core.enable_lsp(sender as lsp::EventSender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");

    let script = fake_server();
    assert!(core.use_language_server_command(
        "python",
        python3(),
        &[
            script.to_str().unwrap(),
            log_principal.to_str().unwrap(),
            "--publica",
            "pyright",
        ],
    ));
    assert!(core.use_language_server_companion(
        "python",
        "python-ruff",
        python3(),
        &[
            script.to_str().unwrap(),
            log_companheiro.to_str().unwrap(),
            "--publica",
            "ruff",
        ],
    ));
    Dupla {
        core,
        root,
        log_principal,
        log_companheiro,
        events: receiver,
    }
}

impl Dupla {
    fn ok(&mut self, method: &str, params: Value) -> Value {
        let outcome = self
            .core
            .handle_request(&JsonRpcRequest::new(9_i64, method, Some(params)));
        let response = outcome.response();
        assert!(
            response.error.is_none(),
            "{method} falhou: {:?}",
            response.error
        );
        response.result.clone().unwrap_or(Value::Null)
    }

    fn app(&self) -> String {
        self.root.join("app.py").display().to_string()
    }

    fn mensagens(log: &PathBuf) -> Vec<Value> {
        std::fs::read_to_string(log)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .collect()
    }

    fn espera_no_wire(log: &PathBuf, method: &str) -> Value {
        let deadline = Instant::now() + DEADLINE;
        while Instant::now() < deadline {
            if let Some(m) = Self::mensagens(log)
                .into_iter()
                .find(|m| m.get("method").and_then(Value::as_str) == Some(method))
            {
                return m;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("{method} nao chegou a {}", log.display());
    }

    /// As mensagens dos `event.lsp.diagnostics` de app.py, na ordem, ate' um
    /// deles satisfazer `pronto` (ou o prazo acabar — falha).
    fn diagnosticos_ate(&self, pronto: impl Fn(&[String]) -> bool) -> Vec<Vec<String>> {
        let deadline = Instant::now() + DEADLINE;
        let mut vistos = Vec::new();
        while Instant::now() < deadline {
            match self.events.recv_timeout(Duration::from_millis(50)) {
                Ok(event) if event.method == "event.lsp.diagnostics" => {
                    let params = event.params.unwrap();
                    if params["path"] != self.app() {
                        continue;
                    }
                    let mensagens: Vec<String> = params["diagnostics"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|d| d["message"].as_str().unwrap().to_owned())
                        .collect();
                    let fim = pronto(&mensagens);
                    vistos.push(mensagens);
                    if fim {
                        return vistos;
                    }
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!("os diagnosticos esperados nao chegaram; vistos: {vistos:?}");
    }

    fn statuses(&self) -> Vec<(String, String)> {
        let mut saida = Vec::new();
        while let Ok(event) = self.events.try_recv() {
            if event.method == "event.lsp.status" {
                let p = event.params.unwrap();
                saida.push((
                    p["language"].as_str().unwrap().to_owned(),
                    p["status"].as_str().unwrap().to_owned(),
                ));
            }
        }
        saida
    }
}

const PYRIGHT: &str = "pyright: diagnostico falso";
const RUFF: &str = "ruff: diagnostico falso";

/// O texto vai aos DOIS servidores, e os diagnosticos dos dois saem FUNDIDOS
/// — principal primeiro — num evento so' por arquivo.
#[test]
fn both_servers_get_the_text_and_their_diagnostics_are_merged_into_one_event() {
    let mut d = dupla("fusao");
    d.ok("fs.read", json!({ "path": d.app() }));
    let aberto_principal = Dupla::espera_no_wire(&d.log_principal, "textDocument/didOpen");
    let aberto_companheiro = Dupla::espera_no_wire(&d.log_companheiro, "textDocument/didOpen");
    assert_eq!(
        aberto_companheiro["params"]["textDocument"]["text"],
        aberto_principal["params"]["textDocument"]["text"],
        "o mesmo texto"
    );
    assert_eq!(
        aberto_companheiro["params"]["textDocument"]["languageId"], "python",
        "o companheiro herda o languageId do principal"
    );
    let vistos = d.diagnosticos_ate(|m| m.len() == 2);
    assert_eq!(
        vistos.last().unwrap(),
        &[PYRIGHT.to_owned(), RUFF.to_owned()],
        "a uniao, principal primeiro: {vistos:?}"
    );
    assert!(
        vistos.iter().all(|m| !m.is_empty()),
        "nenhum evento intermediario APAGOU o outro servidor: {vistos:?}"
    );

    // Uma mudanca de texto: os dois recebem o didChange, cada um com a SUA
    // versao — e a uniao continua inteira.
    d.ok(
        "lsp.didChange",
        json!({ "path": d.app(), "content": "import os\nimport sys\n" }),
    );
    let mudou = Dupla::espera_no_wire(&d.log_companheiro, "textDocument/didChange");
    assert_eq!(mudou["params"]["textDocument"]["version"], 2);
    Dupla::espera_no_wire(&d.log_principal, "textDocument/didChange");
    let vistos = d.diagnosticos_ate(|m| m.len() == 2);
    assert_eq!(vistos.last().unwrap(), &[PYRIGHT.to_owned(), RUFF.to_owned()]);
}

/// As code actions dos dois servidores vem numa lista so', e aplicar a do
/// companheiro edita o arquivo (o indice aponta para o servidor certo).
#[test]
fn code_actions_of_both_servers_come_in_one_list_and_the_companion_one_applies() {
    let mut d = dupla("acoes");
    d.ok("fs.read", json!({ "path": d.app() }));
    d.diagnosticos_ate(|m| m.len() == 2);
    // O buffer mudou desde o didOpen: a consulta sincroniza o texto nos
    // DOIS antes de perguntar — no companheiro, um didChange (versao 2)
    // ANTES do codeAction, senao as posicoes das acoes dele seriam de outro
    // texto.
    let texto = "import os\nimport sys\n";
    let acoes = d.ok(
        "lsp.codeActions",
        json!({ "path": d.app(), "content": texto, "line": 1, "column": 1 }),
    );
    let wire = Dupla::mensagens(&d.log_companheiro);
    let pos = |m: &str| wire.iter().position(|x| x["method"] == m).unwrap_or(usize::MAX);
    assert!(
        pos("textDocument/didChange") < pos("textDocument/codeAction"),
        "o companheiro tem de ver o texto novo antes da pergunta: {wire:#?}"
    );
    assert_eq!(
        wire[pos("textDocument/didChange")]["params"]["textDocument"]["version"], 2
    );
    let titulos: Vec<&str> = acoes["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    assert_eq!(titulos, ["pyright: corrigir", "ruff: corrigir"], "{acoes}");
    // O contexto de cada consulta leva os diagnosticos DAQUELE servidor.
    let consulta_ruff = Dupla::espera_no_wire(&d.log_companheiro, "textDocument/codeAction");
    let contexto = consulta_ruff["params"]["context"]["diagnostics"]
        .as_array()
        .unwrap();
    assert_eq!(contexto.len(), 1, "{consulta_ruff}");
    assert_eq!(contexto[0]["source"], "ruff");

    let previa = d.ok(
        "lsp.applyCodeAction",
        json!({ "path": d.app(), "content": texto, "actionIndex": 1 }),
    );
    assert_eq!(previa["title"], "ruff: corrigir", "{previa}");
    assert_eq!(previa["edits"], 1);
    d.ok(
        "lsp.workspaceEdit.apply",
        json!({ "transactionId": previa["transactionId"] }),
    );
    assert_eq!(
        std::fs::read_to_string(d.root.join("app.py")).unwrap(),
        "# ruff\nimport os\nimport sys\n",
        "a acao do companheiro escreveu no arquivo"
    );
}

/// `lsp.restart` de uma linguagem reinicia o principal E o companheiro — e a
/// UI ve UM `event.lsp.restarted` por linguagem, nao um por processo.
#[test]
fn restarting_a_language_restarts_its_companion_too_with_one_restarted_event() {
    let mut d = dupla("restart");
    d.ok("fs.read", json!({ "path": d.app() }));
    d.diagnosticos_ate(|m| m.len() == 2);
    d.statuses();
    d.ok("lsp.restart", json!({ "language": "python" }));
    let mut reiniciados = 0;
    let mut statuses = Vec::new();
    let deadline = Instant::now() + DEADLINE;
    while Instant::now() < deadline {
        match d.events.recv_timeout(Duration::from_millis(50)) {
            Ok(e) if e.method == "event.lsp.restarted" => reiniciados += 1,
            Ok(e) if e.method == "event.lsp.status" => {
                let p = e.params.unwrap();
                statuses.push((
                    p["language"].as_str().unwrap().to_owned(),
                    p["status"].as_str().unwrap().to_owned(),
                ));
            }
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if statuses.iter().filter(|s| s.1 == "restarting").count() == 2 {
            break;
        }
    }
    assert!(statuses.contains(&("python".to_owned(), "restarting".to_owned())), "{statuses:?}");
    assert!(
        statuses.contains(&("python-ruff".to_owned(), "restarting".to_owned())),
        "{statuses:?}"
    );
    // Drena o que sobrou antes de contar os restarted.
    while let Ok(e) = d.events.recv_timeout(Duration::from_millis(200)) {
        if e.method == "event.lsp.restarted" {
            reiniciados += 1;
        }
    }
    assert_eq!(reiniciados, 1, "um restarted por LINGUAGEM");
}

/// Um companheiro cujo executavel nao sobe SAI da tabela: o motivo vai UMA
/// vez no status, e o principal nao paga por ele a cada sincronizacao.
#[test]
fn a_companion_that_fails_to_start_leaves_the_table_after_one_failed_status() {
    let mut d = dupla("falha");
    assert!(d.core.use_language_server_companion(
        "python",
        "python-ruff",
        "/nao/existe/ruff",
        &["server"],
    ));
    d.ok("fs.read", json!({ "path": d.app() }));
    d.diagnosticos_ate(|m| m == [PYRIGHT.to_owned()]);
    std::fs::write(d.root.join("outro.py"), "x = 1\n").unwrap();
    d.ok("fs.read", json!({ "path": d.root.join("outro.py").to_str().unwrap() }));
    d.diagnosticos_ate(|m| m == [PYRIGHT.to_owned()]);
    d.ok(
        "lsp.hover",
        json!({ "path": d.app(), "content": "import os\n", "line": 1, "column": 1 }),
    );
    let falhas = d
        .statuses()
        .into_iter()
        .filter(|s| s.0 == "python-ruff" && s.1 == "failed")
        .count();
    assert_eq!(falhas, 1, "uma falha, nao uma por sincronizacao");
}

/// O Core poe o `ruff server` como companheiro SOZINHO quando o binario
/// existe no PATH — sem a UI pedir e sem lista propria. (Aqui `ruff` e' um
/// wrapper do servidor falso que ignora o `server` e publica como "ruff".)
#[test]
fn the_core_registers_ruff_as_a_companion_when_the_binary_is_detected() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspdupla-ruff-detectado", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(dir.join("app.py"), "import os\n").unwrap();
    let log = dir.join("ruff.jsonl");
    let wrapper = dir.join("bin/ruff");
    std::fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\nexec {} {} {} --publica ruff\n",
            python3(),
            fake_server().display(),
            log.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let dir = dir.canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender as lsp::EventSender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    // O principal (basedpyright) nao existe nesta maquina de teste: o falso
    // entra no lugar, calado.
    assert!(core.use_language_server_command(
        "python",
        python3(),
        &[
            fake_server().to_str().unwrap(),
            dir.join("principal.jsonl").to_str().unwrap(),
            "--publica",
            "pyright",
        ],
    ));
    let mut d = Dupla {
        core,
        root: dir.clone(),
        log_principal: dir.join("principal.jsonl"),
        log_companheiro: log,
        events: receiver,
    };
    d.ok("fs.read", json!({ "path": d.app() }));
    let vistos = d.diagnosticos_ate(|m| m.len() == 2);
    assert_eq!(vistos.last().unwrap(), &[PYRIGHT.to_owned(), RUFF.to_owned()]);
    let comando = Dupla::mensagens(&d.log_companheiro);
    assert!(!comando.is_empty(), "o wrapper `ruff` foi o processo que subiu");
}

/// Sem o binario do companheiro (a maquina sem ruff), o Core o tira: os
/// diagnosticos dele somem na hora e o principal fica; fechar o documento
/// fecha nos dois.
#[test]
fn removing_the_companion_clears_only_its_diagnostics_and_close_reaches_both() {
    let mut d = dupla("remocao");
    d.ok("fs.read", json!({ "path": d.app() }));
    d.diagnosticos_ate(|m| m.len() == 2);
    d.statuses();
    // O ambiente Python "mudou": o Core reconfigura o LSP com o detector (sem
    // ruff no PATH falso) — o companheiro sai — e reinicia o principal.
    d.core.observe_notification(&JsonRpcRequest::notification(
        "event.python.finished",
        Some(json!({ "success": true })),
    ));
    let vistos = d.diagnosticos_ate(|m| m == [PYRIGHT.to_owned()]);
    assert_eq!(
        vistos.first().unwrap(),
        &[PYRIGHT.to_owned()],
        "o primeiro evento depois da saida ja' e' so' o principal: {vistos:?}"
    );
    let statuses = d.statuses();
    assert!(
        statuses.contains(&("python-ruff".to_owned(), "stopped".to_owned())),
        "{statuses:?}"
    );

    // Com os dois de volta, fechar chega aos dois.
    let mut d = dupla("fechar");
    d.ok("fs.read", json!({ "path": d.app() }));
    d.diagnosticos_ate(|m| m.len() == 2);
    d.ok("fs.delete", json!({ "path": d.app() }));
    Dupla::espera_no_wire(&d.log_principal, "textDocument/didClose");
    Dupla::espera_no_wire(&d.log_companheiro, "textDocument/didClose");
}
