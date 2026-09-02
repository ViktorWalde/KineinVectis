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
    /// Segura a ponta receptora dos eventos: sem ela, todo `send` do core
    /// falharia e a thread leitora do servidor morreria calada.
    _events: mpsc::Receiver<JsonRpcRequest>,
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
    let replaced = core.use_language_server_command(
        "rust",
        python3(),
        &[script.to_str().unwrap(), log.to_str().unwrap()],
    );
    assert!(replaced, "a linguagem rust precisa existir na tabela");

    Harness {
        core,
        root,
        log,
        _events: receiver,
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
