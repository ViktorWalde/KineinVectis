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
fn fake_server() -> PathBuf {
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
fn python3() -> &'static str {
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
    for language in ["rust", "cpp"] {
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
