//! O que o core FALA com um language server, e o que ele faz com a resposta.
//!
//! # Por que este arquivo existe, e por que ele nao esta no `tests/lsp.rs`
//!
//! O `tests/lsp.rs` cobre GUARDAS: sem workspace, sem manager, params
//! invalidos. Ele nunca sobe um servidor — e ate 2026-09-02 nenhum teste deste
//! repositorio subia. Os **15 metodos `lsp.*` nao tinham um unico teste de
//! comportamento**, e nada reprovava se o `didOpen` parasse de sair, se a
//! versao do documento parasse de subir ou se o curto-circuito por hash
//! deixasse de curto-circuitar. Era a maior lacuna de evidencia do projeto
//! (roadmap 30, etapa 3).
//!
//! Aqui o servidor e o `scripts/fake_lsp_server.py`: deterministico, instantaneo
//! e — o que importa — ele **grava tudo o que recebe**. Cada teste abaixo olha o
//! wire, nao a resposta do IPC; foi exatamente essa distincao que a etapa 1
//! ensinou em 2026-08-30, quando uma sonda ficou verde com o produto quebrado
//! por olhar a resposta em vez do efeito.

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use crate::{Core, RequestOutcome, lsp};
use kinein_protocol::JsonRpcRequest;

/// Prazo maximo esperando uma mensagem aparecer no log do servidor falso.
const DEADLINE: Duration = Duration::from_secs(10);

/// Um core com LSP ligado, workspace aberto e o servidor de Rust FALSIFICADO.
struct Harness {
    core: Core,
    root: PathBuf,
    log: PathBuf,
    /// Ponta receptora dos eventos que o CORE emite. Segurar e obrigatorio —
    /// sem ela, todo `send` falharia e a thread leitora do servidor morreria
    /// calada —, e [`Harness::wait_for_event`] a consome.
    events: mpsc::Receiver<JsonRpcRequest>,
}

/// Caminho absoluto do servidor falso, versionado junto com os testes.
pub(super) fn fake_server() -> PathBuf {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("scripts/fake_lsp_server.py");
    let script = script.canonicalize().unwrap_or(script);
    assert!(
        script.is_file(),
        "scripts/fake_lsp_server.py ausente em {}",
        script.display()
    );
    script
}

/// `python3` e requisito do gate deste repositorio (4 das 13 verificacoes o
/// usam). Faltar e' FALHA, nunca teste pulado: teste que pula nao prova nada.
pub(super) fn python3() -> &'static str {
    let ok = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(
        ok,
        "python3 nao encontrado; ele e requisito do gate (scripts/instalar-ambiente.sh)"
    );
    "python3"
}

fn harness(name: &str) -> Harness {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspwire-{name}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(
        root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(demo CXX)\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cpp"), "int main() { return 0; }\n").unwrap();
    std::fs::write(root.join("app.py"), "def main():\n    pass\n").unwrap();
    // Um .venv com interpretador: e' ele que o basedpyright tem de receber.
    std::fs::create_dir_all(root.join(".venv/bin")).unwrap();
    std::fs::write(
        root.join(".venv/bin/python"),
        "#!/bin/sh\necho Python 3.13.0\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            root.join(".venv/bin/python"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    let root = root.canonicalize().unwrap();
    let log = root.join("lsp-wire.jsonl");

    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = super::core_with_empty_search_path(&format!("lspwire-{name}"));
    core.enable_lsp(sender as lsp::EventSender);

    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");

    let script = fake_server();
    for language in ["rust", "cpp", "python"] {
        let replaced = core.use_language_server_command(
            language,
            python3(),
            &[script.to_str().unwrap(), log.to_str().unwrap()],
        );
        assert!(replaced, "a linguagem {language} precisa existir na tabela");
    }

    Harness {
        core,
        root,
        log,
        events: receiver,
    }
}

impl Harness {
    fn call(&mut self, method: &str, params: Value) -> RequestOutcome {
        self.core
            .handle_request(&JsonRpcRequest::new(9_i64, method, Some(params)))
    }

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let outcome = self.call(method, params);
        let response = outcome.response();
        assert!(response.error.is_none(), "{method}: {:?}", response.error);
        response.result.clone().unwrap()
    }

    fn main_rs(&self) -> String {
        self.root.join("src/main.rs").display().to_string()
    }

    fn main_cpp(&self) -> String {
        self.root.join("src/main.cpp").display().to_string()
    }

    /// Empurra pelo LOOP REAL o mesmo evento que o job do configure emite.
    ///
    /// Passa pelo [`crate::runtime::drain_loop_events`], e nao por uma chamada
    /// direta a `observe_notification`, de proposito: a primeira versao deste
    /// teste chamava o core direto e ficava VERDE com a fiacao do loop
    /// removida — a falha silenciosa que a §8 do `ARCHITECTURE.md` descreve.
    ///
    /// Rodar o cmake de verdade traria uma ferramenta externa e nenhuma prova a
    /// mais sobre ESTE comportamento: o contrato entre job e loop e o evento.
    fn configure_finished(&mut self, success: bool) {
        let (sender, inbox) = mpsc::channel::<crate::runtime::LoopEvent>();
        sender
            .send(crate::runtime::LoopEvent::Notification(Box::new(
                JsonRpcRequest::notification(
                    "event.cmake.finished",
                    Some(json!({ "jobId": "job-1", "success": success })),
                ),
            )))
            .unwrap();
        drop(sender);
        let mut saida = Vec::new();
        crate::runtime::drain_loop_events(&mut self.core, &mut saida, &inbox).unwrap();
        assert!(
            String::from_utf8_lossy(&saida).contains("event.cmake.finished"),
            "o loop tem que repassar o evento para a UI tambem"
        );
    }

    /// Mensagens que o servidor falso recebeu ate agora.
    fn messages(&self) -> Vec<Value> {
        let Ok(body) = std::fs::read_to_string(&self.log) else {
            return Vec::new();
        };
        body.lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect()
    }

    /// Espera a n-esima ocorrencia de um metodo no wire; falha por prazo.
    ///
    /// Esperar por PRAZO (e nao contar mensagens) e a mesma regra das sondas:
    /// resposta e evento nao tem ordem garantida entre si (arquitetura/04 §6).
    fn wait_for(&self, method: &str, occurrence: usize) -> Value {
        let deadline = Instant::now() + DEADLINE;
        while Instant::now() < deadline {
            let found: Vec<Value> = self
                .messages()
                .into_iter()
                .filter(|message| message.get("method").and_then(Value::as_str) == Some(method))
                .collect();
            if let Some(message) = found.get(occurrence) {
                return message.clone();
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "{method} #{occurrence} nao chegou ao servidor em {DEADLINE:?}; wire: {:#?}",
            self.messages()
        );
    }

    /// Espera um evento que o CORE emitiu (nao o que o servidor recebeu).
    fn wait_for_event(&self, method: &str) -> JsonRpcRequest {
        let deadline = Instant::now() + DEADLINE;
        while Instant::now() < deadline {
            match self.events.recv_timeout(Duration::from_millis(50)) {
                Ok(event) if event.method == method => return event,
                // Outro evento, ou nada ainda: seguir esperando ate o prazo.
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!("o core nao emitiu {method} em {DEADLINE:?}");
    }

    fn count_of(&self, method: &str) -> usize {
        self.messages()
            .iter()
            .filter(|message| message.get("method").and_then(Value::as_str) == Some(method))
            .count()
    }
}

/// O handshake acontece de verdade: sem ele, nada abaixo existiria.
#[test]
fn the_core_completes_the_initialize_handshake() {
    let mut harness = harness("handshake");
    harness.ok("fs.read", json!({ "path": harness.main_rs() }));

    let initialize = harness.wait_for("initialize", 0);
    assert_eq!(initialize["id"], 1);
    assert!(
        initialize["params"]["rootUri"]
            .as_str()
            .unwrap()
            .starts_with("file://"),
        "o servidor precisa saber a raiz"
    );
    harness.wait_for("initialized", 0);
}

/// `fs.read` abre o documento no servidor — o caminho real da IDE.
#[test]
fn reading_a_file_opens_the_document_in_the_server() {
    let mut harness = harness("didopen");
    harness.ok("fs.read", json!({ "path": harness.main_rs() }));

    let did_open = harness.wait_for("textDocument/didOpen", 0);
    let document = &did_open["params"]["textDocument"];

    assert_eq!(document["languageId"], "rust");
    assert_eq!(document["version"], 1);
    assert_eq!(document["text"], "fn main() {}\n");
    assert!(
        document["uri"].as_str().unwrap().ends_with("/src/main.rs"),
        "uri: {}",
        document["uri"]
    );
}

/// A versao sobe a cada mudanca REAL, e conteudo identico nao vira mensagem.
///
/// O curto-circuito por hash e' carga estrutural: sem ele o clangd invalida os
/// fix-its a cada request posicional (é o que o `sync.rs` documenta). Ate hoje
/// nada provava que ele funcionava.
#[test]
fn did_change_bumps_the_version_and_skips_identical_content() {
    let mut harness = harness("didchange");
    let path = harness.main_rs();
    harness.ok("fs.read", json!({ "path": &path }));
    harness.wait_for("textDocument/didOpen", 0);

    harness.ok(
        "lsp.didChange",
        json!({ "path": &path, "content": "fn main() { let a = 1; }\n" }),
    );
    let first = harness.wait_for("textDocument/didChange", 0);
    assert_eq!(first["params"]["textDocument"]["version"], 2);
    assert_eq!(
        first["params"]["contentChanges"][0]["text"],
        "fn main() { let a = 1; }\n"
    );

    harness.ok(
        "lsp.didChange",
        json!({ "path": &path, "content": "fn main() { let a = 2; }\n" }),
    );
    let second = harness.wait_for("textDocument/didChange", 1);
    assert_eq!(second["params"]["textDocument"]["version"], 3);

    // O MESMO conteudo de novo: nada pode ir para o wire.
    harness.ok(
        "lsp.didChange",
        json!({ "path": path, "content": "fn main() { let a = 2; }\n" }),
    );
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        harness.count_of("textDocument/didChange"),
        2,
        "conteudo identico nao pode virar didChange"
    );
}

/// Apagar o arquivo FECHA o documento: senao o servidor mantem diagnostico de
/// um arquivo que nao existe mais.
#[test]
fn deleting_a_file_closes_the_document_in_the_server() {
    let mut harness = harness("didclose");
    let path = harness.main_rs();
    harness.ok("fs.read", json!({ "path": &path }));
    let did_open = harness.wait_for("textDocument/didOpen", 0);
    let uri = did_open["params"]["textDocument"]["uri"].clone();

    harness.ok("fs.delete", json!({ "path": &path }));

    let did_close = harness.wait_for("textDocument/didClose", 0);
    assert_eq!(did_close["params"]["textDocument"]["uri"], uri);
    assert!(!Path::new(&path).exists(), "o arquivo tinha que sumir");
}

/// Reabrir depois de fechar recomeca na versao 1 — o que a etapa 4 vai usar
/// para o clangd recompilar com as flags novas do configure.
#[test]
fn reopening_a_closed_document_restarts_at_version_one() {
    let mut harness = harness("reopen");
    let path = harness.main_rs();
    harness.ok("fs.read", json!({ "path": &path }));
    harness.wait_for("textDocument/didOpen", 0);
    harness.ok(
        "lsp.didChange",
        json!({ "path": &path, "content": "fn main() { }\n" }),
    );
    harness.wait_for("textDocument/didChange", 0);

    // Fecha e reabre pelo mesmo caminho que a etapa 4 usara.
    std::fs::write(&path, "fn main() { }\n").unwrap();
    harness.ok("fs.delete", json!({ "path": &path }));
    harness.wait_for("textDocument/didClose", 0);
    std::fs::write(&path, "fn main() { }\n").unwrap();
    harness.ok("fs.read", json!({ "path": path }));

    let reopened = harness.wait_for("textDocument/didOpen", 1);
    assert_eq!(reopened["params"]["textDocument"]["version"], 1);
}

/// A resposta do servidor volta pelo wire e vira TIPO DO PROTOCOLO.
///
/// O servidor falso responde sempre `line: 2, character: 4` (0-based do LSP); a
/// resposta do core tem de sair 1-based. Esta conversao nunca tinha sido
/// exercida ponta a ponta.
#[test]
fn a_server_response_crosses_the_wire_and_becomes_a_protocol_type() {
    let mut harness = harness("definition");
    let path = harness.main_rs();

    let result = harness.ok(
        "lsp.definition",
        json!({
            "path": &path,
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
        }),
    );

    harness.wait_for("textDocument/definition", 0);
    assert_eq!(result["line"], 3, "0-based 2 vira 1-based 3");
    assert_eq!(result["column"], 5, "0-based 4 vira 1-based 5");
    assert_eq!(result["path"], path);
}

/// Trocar o executavel nao derruba servidor vivo: a troca vale na proxima vez.
#[test]
fn replacing_the_command_does_not_kill_a_running_server() {
    let mut harness = harness("troca");
    harness.ok("fs.read", json!({ "path": harness.main_rs() }));
    harness.wait_for("textDocument/didOpen", 0);

    assert!(
        harness
            .core
            .use_language_server_command("rust", "/nao/existe", &[])
    );

    // O servidor ja em execucao continua atendendo.
    harness.ok(
        "lsp.didChange",
        json!({ "path": harness.main_rs(), "content": "fn main() { }\n" }),
    );
    harness.wait_for("textDocument/didChange", 0);
}

/// Configure bem-sucedido FECHA os documentos C/C++ abertos (roadmap 30 §4).
///
/// O clangd recarrega a `compile_commands.json` sozinho, mas o documento ja
/// aberto fica com a compilacao em cache (`roadmaps/29` §5b). Fechar e' o unico
/// jeito de o proximo `didOpen` ser real — reenviar `didOpen` seria inerte por
/// causa do curto-circuito por hash.
#[test]
fn a_successful_configure_closes_the_open_cpp_documents() {
    let mut harness = harness("configure");
    let cpp = harness.main_cpp();
    harness.ok("fs.read", json!({ "path": &cpp }));
    let did_open = harness.wait_for("textDocument/didOpen", 0);
    let uri = did_open["params"]["textDocument"]["uri"].clone();

    harness.configure_finished(true);

    let did_close = harness.wait_for("textDocument/didClose", 0);
    assert_eq!(did_close["params"]["textDocument"]["uri"], uri);

    // E o proximo didOpen volta a ser REAL: versao 1, com o buffer do editor
    // (nao com o disco — o texto abaixo nunca foi gravado).
    harness.ok(
        "lsp.didChange",
        json!({ "path": &cpp, "content": "int main() { return 1; }\n" }),
    );
    let reopened = harness.wait_for("textDocument/didOpen", 1);
    assert_eq!(reopened["params"]["textDocument"]["version"], 1);
    assert_eq!(
        reopened["params"]["textDocument"]["text"], "int main() { return 1; }\n",
        "o reabrir carrega o BUFFER, nao o disco"
    );
}

/// Configure que FALHOU nao mexe em documento nenhum.
#[test]
fn a_failed_configure_closes_nothing() {
    let mut harness = harness("configure-falho");
    harness.ok("fs.read", json!({ "path": harness.main_cpp() }));
    harness.wait_for("textDocument/didOpen", 0);

    harness.configure_finished(false);

    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(harness.count_of("textDocument/didClose"), 0);
}

/// O configure e de C/C++: documento Rust aberto nao e' afetado.
#[test]
fn a_configure_does_not_touch_documents_of_other_languages() {
    let mut harness = harness("configure-rust");
    harness.ok("fs.read", json!({ "path": harness.main_rs() }));
    harness.wait_for("textDocument/didOpen", 0);

    harness.configure_finished(true);

    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(harness.count_of("textDocument/didClose"), 0);
}

/// A UI precisa saber que fechou, senao o arquivo ativo fica sem diagnostico
/// ate o usuario digitar. O evento e o gatilho da re-sincronizacao.
#[test]
fn closing_after_a_configure_tells_the_ui_to_resync() {
    let mut harness = harness("configure-evento");
    harness.ok("fs.read", json!({ "path": harness.main_cpp() }));
    harness.wait_for("textDocument/didOpen", 0);

    harness.configure_finished(true);

    let event = harness.wait_for_event("event.lsp.documentsClosed");
    assert_eq!(event.params.as_ref().unwrap()["language"], "cpp");
    assert_eq!(event.params.as_ref().unwrap()["count"], 1);
}

/// Fatia 2 da cadeia Python (2026-09-12): o servidor de Python sobe COM o
/// interpretador do projeto. No fio: o `workspace/didChangeConfiguration`
/// logo apos o `initialized`, com `python.pythonPath` = o `.venv/bin/python`
/// do workspace; e o `workspace/configuration` que o servidor pergunta (como
/// o pyright faz) respondido secao a secao — `python`, `python.analysis`, e
/// `null` para o que nao existe. Sem isto o basedpyright indexaria a stdlib
/// do Python do PATH e o completar mentiria.
#[test]
fn the_python_server_receives_the_project_interpreter() {
    let mut h = harness("python-interpretador");
    // O Core configurou o python ao abrir o workspace; o harness trocou o
    // COMANDO pelo falso, e o settings do Core tem de continuar valendo.
    let path = h.root.join("app.py");
    h.ok("fs.read", json!({ "path": path.to_str().unwrap() }));
    let config = h.wait_for("workspace/didChangeConfiguration", 0);
    let settings = &config["params"]["settings"];
    let esperado = h.root.join(".venv/bin/python").display().to_string();
    assert_eq!(settings["python"]["pythonPath"], esperado, "{settings}");
    assert_eq!(settings["python"]["analysis"]["autoSearchPaths"], true);
    // So' os arquivos abertos: `workspace` mandaria o basedpyright analisar o
    // projeto inteiro a cada mudanca — pesado e fora do que a tela mostra.
    for secao in ["python", "basedpyright"] {
        assert_eq!(
            settings[secao]["analysis"]["diagnosticMode"], "openFilesOnly",
            "{settings}"
        );
    }
    // O initialized veio ANTES da configuracao (ordem do protocolo).
    let mensagens = h.messages();
    let pos = |m: &str| mensagens.iter().position(|x| x["method"] == m).unwrap();
    assert!(pos("initialized") < pos("workspace/didChangeConfiguration"));
    // A resposta ao workspace/configuration do servidor: uma entrada por item.
    let deadline = Instant::now() + DEADLINE;
    let resposta = loop {
        if let Some(r) = h
            .messages()
            .into_iter()
            .find(|m| m["id"] == 9001 && m.get("result").is_some())
        {
            break r;
        }
        assert!(
            Instant::now() < deadline,
            "o core nao respondeu ao workspace/configuration"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    let itens = resposta["result"].as_array().unwrap();
    assert_eq!(itens.len(), 5, "{resposta}");
    assert_eq!(itens[0]["pythonPath"], esperado);
    assert_eq!(
        itens[1]["autoSearchPaths"], true,
        "secao com ponto: python.analysis"
    );
    assert!(itens[2].is_null(), "secao inexistente e' null, nao erro");
    // Item sem `section` (ou com secao vazia) = a configuracao INTEIRA (LSP
    // 3.17, ConfigurationItem.section opcional).
    for inteiro in [&itens[3], &itens[4]] {
        assert_eq!(inteiro["python"]["pythonPath"], esperado, "{inteiro}");
        assert!(inteiro["basedpyright"].is_object(), "{inteiro}");
    }
    // O didOpen do .py vai com languageId python.
    let aberto = h.wait_for("textDocument/didOpen", 0);
    assert_eq!(aberto["params"]["textDocument"]["languageId"], "python");

    // O ambiente mudou (event.python.finished com sucesso): o servidor de
    // Python e' REINICIADO para subir com o interpretador novo; um evento de
    // falha nao reinicia nada.
    h.core.observe_notification(&JsonRpcRequest::notification(
        "event.python.finished",
        Some(json!({ "jobId": "j", "success": false, "tool": "uv", "command": "uv venv .venv", "path": "x" })),
    ));
    // Emitido de forma sincrona pelo restart: se nada chegou em 300 ms, nao
    // houve restart.
    let prazo = Instant::now() + Duration::from_millis(300);
    while let Ok(evento) = h
        .events
        .recv_timeout(prazo.saturating_duration_since(Instant::now()))
    {
        assert_ne!(
            evento.method, "event.lsp.restarted",
            "falha do ambiente nao reinicia o servidor"
        );
    }
    h.core.observe_notification(&JsonRpcRequest::notification(
        "event.python.finished",
        Some(json!({ "jobId": "j", "success": true, "tool": "uv", "command": "uv venv .venv", "path": "x" })),
    ));
    let reiniciado = h.wait_for_event("event.lsp.restarted");
    assert_eq!(reiniciado.params.unwrap()["language"], "python");
}

/// C4: os stubs da placa chegam ao basedpyright. Sem `typings/` nada de
/// `stubPath`; quando `event.python.stubs` termina com sucesso e a pasta tem
/// um `.pyi`, o servidor reinicia e o `didChangeConfiguration` novo traz
/// `basedpyright.analysis.stubPath` = `<root>/typings` e
/// `reportMissingModuleSource: none` — as chaves do manual do basedpyright
/// e do `pyproject` de exemplo dos micropython-stubs (2026-09-17).
#[test]
fn the_python_server_receives_the_board_stubs_path_after_they_are_installed() {
    let mut h = harness("python-stubs");
    let path = h.root.join("app.py");
    h.ok("fs.read", json!({ "path": path.to_str().unwrap() }));
    let config = h.wait_for("workspace/didChangeConfiguration", 0);
    assert!(
        config["params"]["settings"]["basedpyright"]["analysis"]
            .get("stubPath")
            .is_none(),
        "sem typings/ nao ha' stubPath: {config}"
    );

    std::fs::create_dir_all(h.root.join("typings")).unwrap();
    std::fs::write(h.root.join("typings/machine.pyi"), "class Pin: ...\n").unwrap();
    h.core.observe_notification(&JsonRpcRequest::notification(
        "event.python.stubs",
        Some(json!({ "jobId": "j", "success": true, "package": "micropython-esp32-stubs",
                     "command": "uv pip install …", "target": h.root.join("typings").display().to_string() })),
    ));
    let reiniciado = h.wait_for_event("event.lsp.restarted");
    assert_eq!(reiniciado.params.unwrap()["language"], "python");
    // A reconfiguracao repos o comando DETECTADO do basedpyright (ausente
    // nesta maquina de teste): o harness aponta o falso de novo, como fez
    // ao nascer — o que se prova aqui sao as settings, nao o binario.
    let script = fake_server();
    let log = h.log.clone();
    assert!(h.core.use_language_server_command(
        "python",
        python3(),
        &[script.to_str().unwrap(), log.to_str().unwrap()],
    ));
    h.ok("fs.read", json!({ "path": path.to_str().unwrap() }));
    let esperado = h.root.join("typings").display().to_string();
    let deadline = Instant::now() + DEADLINE;
    let analysis = loop {
        let achado = h.messages().into_iter().find(|m| {
            m["method"] == "workspace/didChangeConfiguration"
                && m["params"]["settings"]["basedpyright"]["analysis"]["stubPath"] == esperado
        });
        if let Some(m) = achado {
            break m["params"]["settings"]["basedpyright"]["analysis"].clone();
        }
        assert!(
            Instant::now() < deadline,
            "o stubPath nao chegou ao basedpyright: {:?}",
            h.messages()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        analysis["diagnosticSeverityOverrides"]["reportMissingModuleSource"],
        "none"
    );
    assert_eq!(analysis["autoSearchPaths"], true);
}

/// O comando do basedpyright e' o DETECTADO (`~/.local/bin` do pipx/npm, que o
/// PATH do processo da IDE pode nao ter), com `--stdio` — a unica forma de
/// transporte que o core fala. Aqui ninguem troca o comando: um
/// `basedpyright-langserver` falso na pasta de busca do detector grava os
/// argumentos e encaminha para o servidor falso.
#[test]
#[cfg(unix)]
fn the_python_server_is_the_detected_binary_with_stdio() {
    use std::os::unix::fs::PermissionsExt;

    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-lspwire-python-detectado", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("ws")).unwrap();
    std::fs::write(
        base.join("ws/pyproject.toml"),
        "[project]\nname = \"demo\"\n",
    )
    .unwrap();
    std::fs::write(base.join("ws/app.py"), "x = 1\n").unwrap();
    let base = base.canonicalize().unwrap();
    let log = base.join("lsp-wire.jsonl");
    let args = base.join("args.txt");
    std::fs::write(
        base.join("bin/basedpyright-langserver"),
        format!(
            "#!/bin/sh\necho \"$@\" > {args}\nexec {py} {script} {log}\n",
            args = args.display(),
            py = python3(),
            script = fake_server().display(),
            log = log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(
        base.join("bin/basedpyright-langserver"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();

    let (sender, receiver) = mpsc::channel::<JsonRpcRequest>();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ));
    core.enable_lsp(sender as lsp::EventSender);
    let ws = base.join("ws");
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": ws.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let mut h = Harness {
        core,
        root: ws.clone(),
        log,
        events: receiver,
    };
    let app = ws.join("app.py");
    h.ok("fs.read", json!({ "path": app.to_str().unwrap() }));
    let aberto = h.wait_for("textDocument/didOpen", 0);
    assert_eq!(aberto["params"]["textDocument"]["languageId"], "python");
    assert_eq!(std::fs::read_to_string(&args).unwrap().trim(), "--stdio");
}

/// Sem interpretador nenhum (workspace sem .venv, PATH vazio), o servidor sobe
/// SEM configuracao — a IDE nao inventa um pythonPath — e o servidor de Rust
/// continua sem receber configuracao alguma.
#[test]
fn without_an_interpreter_no_configuration_is_pushed() {
    let mut h = harness("python-sem-interpretador");
    std::fs::remove_dir_all(h.root.join(".venv")).unwrap();
    // Reabrir o workspace refaz a configuracao do python sem o .venv.
    let root = h.root.clone();
    h.ok("workspace.open", json!({ "path": root.to_str().unwrap() }));
    let script = fake_server();
    let log = h.log.clone();
    for language in ["rust", "python"] {
        h.core.use_language_server_command(
            language,
            python3(),
            &[script.to_str().unwrap(), log.to_str().unwrap()],
        );
    }
    let py = h.root.join("app.py");
    h.ok("fs.read", json!({ "path": py.to_str().unwrap() }));
    h.wait_for("textDocument/didOpen", 0);
    let rs = h.main_rs();
    h.ok("fs.read", json!({ "path": rs }));
    h.wait_for("textDocument/didOpen", 1);
    assert_eq!(
        h.count_of("workspace/didChangeConfiguration"),
        0,
        "{:?}",
        h.messages()
    );
}

/// O alvo do kit chega ao rust-analyzer (P0 do 40 §4.1, 2026-09-17). Sem
/// `targetTriple` no kit, o servidor de Rust sobe SEM configuracao (o caso
/// acima). Com `toolchain.setKit { targetTriple }`, o servidor vivo e'
/// reiniciado e o novo sobe com `workspace/didChangeConfiguration` trazendo
/// `rust-analyzer.cargo.target` = o triple — a chave do manual — e o
/// `workspace/configuration` de secao `rust-analyzer` responde o mesmo.
#[test]
fn the_rust_server_receives_the_kit_target_and_restarts_when_it_changes() {
    let mut h = harness("rust-alvo-do-kit");
    let rs = h.main_rs();
    h.ok("fs.read", json!({ "path": rs }));
    h.wait_for("textDocument/didOpen", 0);
    // Sem alvo no kit: nenhuma configuracao foi ao servidor de Rust.
    assert_eq!(
        h.messages()
            .iter()
            .filter(|m| m["method"] == "workspace/didChangeConfiguration"
                && m["params"]["settings"].get("rust-analyzer").is_some())
            .count(),
        0
    );

    let antes = h.count_of("initialize");
    h.ok(
        "toolchain.setKit",
        json!({ "targetTriple": "thumbv7em-none-eabihf" }),
    );
    // O reinicio derruba o servidor e emite `event.lsp.restarted`; quem o
    // sobe de novo e' a proxima sincronizacao (a UI re-sincroniza o arquivo
    // ativo) — aqui, ler o arquivo de novo.
    h.ok("fs.read", json!({ "path": rs }));
    // O servidor de Rust reiniciou (um initialize a mais) e recebeu o alvo.
    let deadline = Instant::now() + DEADLINE;
    let config = loop {
        let achado = h.messages().into_iter().find(|m| {
            m["method"] == "workspace/didChangeConfiguration"
                && m["params"]["settings"]["rust-analyzer"]["cargo"]["target"]
                    == "thumbv7em-none-eabihf"
        });
        if let Some(m) = achado {
            break m;
        }
        assert!(
            Instant::now() < deadline,
            "o alvo nao chegou ao rust-analyzer: {:?}",
            h.messages()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        config["params"]["settings"]["rust-analyzer"]["cargo"]["target"],
        "thumbv7em-none-eabihf"
    );
    assert!(
        h.count_of("initialize") > antes,
        "o servidor de Rust nao reiniciou"
    );

    // Limpar o alvo ("" limpa, como nos outros campos) tira a configuracao
    // na proxima subida — o reinicio seguinte sobe sem `rust-analyzer`.
    let antes = h.count_of("initialize");
    h.ok("toolchain.setKit", json!({ "targetTriple": "" }));
    h.ok("fs.read", json!({ "path": rs }));
    let deadline = Instant::now() + DEADLINE;
    while h.count_of("initialize") <= antes {
        assert!(Instant::now() < deadline, "sem reinicio ao limpar o alvo");
        std::thread::sleep(Duration::from_millis(20));
    }
    h.wait_for("textDocument/didOpen", 2);
    let mensagens = h.messages();
    let ultimo_init = mensagens
        .iter()
        .rposition(|m| m["method"] == "initialize")
        .unwrap();
    assert!(
        !mensagens[ultimo_init..]
            .iter()
            .any(|m| m["method"] == "workspace/didChangeConfiguration"
                && m["params"]["settings"].get("rust-analyzer").is_some()),
        "a configuracao antiga sobreviveu ao alvo limpo"
    );
}
