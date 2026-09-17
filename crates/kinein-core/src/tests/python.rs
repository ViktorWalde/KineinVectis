//! O dominio `python` (bloco B do `roadmaps/41`, fatia 1 em 2026-09-12): o
//! AMBIENTE do projeto — o interpretador que vale, se e' ambiente proprio, e
//! criar um `.venv` num clique.
//!
//! O que se prova: que `python.status` diz a verdade nos tres estados (sem
//! Python nenhum; Python do sistema com a dica e a ferramenta que criaria o
//! ambiente; ambiente proprio sem dica); que a ferramenta e' uv quando ha' uv
//! e venv quando so' ha' python3; que `python.createEnvironment` recusa com
//! motivo o que a maquina nao tem; e, com jobs, que o job roda a ferramenta
//! DETECTADA (um `uv` falso que grava o pedido), o evento sai com sucesso so'
//! quando o `.venv/bin/python` existe, e o `index.context` passa a apontar o
//! ambiente novo — a razao de o dominio existir.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

use crate::Core;
use crate::tools::ToolDetector;

/// Os testes que escrevem scripts e os executam ficam serializados (ETXTBSY
/// entre threads — a mesma razao do `tests/index_context.rs`).
static EXECUTAVEIS: Mutex<()> = Mutex::new(());

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-python-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn escrever(caminho: &Path, conteudo: &str) {
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, conteudo).unwrap();
}

fn script(caminho: &Path, corpo: &str) {
    escrever(caminho, &format!("#!/bin/sh\n{corpo}\n"));
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn abrir(core: &mut Core, raiz: &Path) {
    let r = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(r.response().error.is_none(), "{:?}", r.response().error);
}

fn status(core: &mut Core) -> Value {
    core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "python.status",
        Some(json!({})),
    ))
    .response()
    .result
    .clone()
    .unwrap()
}

#[test]
fn status_tells_the_truth_in_the_three_states() {
    let _serial = EXECUTAVEIS.lock().unwrap();
    let raiz = temp_dir("status");
    escrever(&raiz.join("pyproject.toml"), "[project]\nname = \"demo\"\n");
    escrever(&raiz.join("requirements.txt"), "");
    let bin = temp_dir("status-bin");

    // 1. Sem Python nenhum: nada de interpretador, nada de ferramenta, a dica.
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    let s = status(&mut core);
    assert!(s.get("interpreter").is_none(), "{s}");
    assert_eq!(s["hasEnvironment"], false);
    assert!(s.get("environmentTool").is_none());
    assert_eq!(
        s["projectFiles"],
        json!(["pyproject.toml", "requirements.txt"])
    );
    assert!(s["hint"].as_str().unwrap().contains("nenhum Python"), "{s}");

    // 1b. So' o uv, sem python3 no PATH: o uv basta (ele baixa um Python).
    script(&bin.join("uv"), "echo uv 0.9.0");
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    let s = status(&mut core);
    assert!(s.get("interpreter").is_none(), "{s}");
    assert_eq!(s["environmentTool"], "uv");
    assert!(
        s["hint"].as_str().unwrap().contains("uv baixa um Python"),
        "{s}"
    );
    std::fs::remove_file(bin.join("uv")).unwrap();

    // 2. So' o python3 do sistema: interpretador com aviso, ferramenta = venv,
    //    a dica pede o ambiente.
    script(&bin.join("python3"), "echo Python 3.14.7");
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    let s = status(&mut core);
    assert_eq!(s["interpreter"]["origin"], "sistema", "{s}");
    assert_eq!(s["interpreter"]["version"], "Python 3.14.7");
    assert_eq!(s["hasEnvironment"], false);
    assert_eq!(s["environmentTool"], "venv");
    assert!(s.get("uv").is_none());
    assert!(s["hint"].as_str().unwrap().contains("python3 -m"), "{s}");

    // 3. Com uv: a ferramenta e' uv, e a dica fala dele.
    script(&bin.join("uv"), "echo uv 0.9.0");
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    let s = status(&mut core);
    assert_eq!(s["environmentTool"], "uv");
    assert_eq!(s["uv"], bin.join("uv").display().to_string());
    assert!(s["hint"].as_str().unwrap().contains("com o uv"), "{s}");

    // 4. Ambiente proprio: sem dica, hasEnvironment.
    script(&raiz.join(".venv/bin/python"), "echo Python 3.14.7");
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    let s = status(&mut core);
    assert_eq!(s["interpreter"]["origin"], ".venv", "{s}");
    assert_eq!(s["hasEnvironment"], true);
    assert!(s.get("hint").is_none(), "{s}");
    // Sem workspace: erro.
    let mut fechado = Core::with_detector(ToolDetector::with_search_path(&bin));
    let r = fechado.handle_request(&JsonRpcRequest::new(
        3_i64,
        "python.status",
        Some(json!({})),
    ));
    assert!(r.response().error.is_some());
}

#[test]
fn create_environment_refuses_with_a_reason_what_the_machine_lacks() {
    let raiz = temp_dir("recusa");
    let bin = temp_dir("recusa-bin");
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    abrir(&mut core, &raiz);
    // Sem uv nem python3: os dois pedidos sao recusados, cada um com o motivo.
    let sem_uv = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "python.createEnvironment",
        Some(json!({ "tool": "uv" })),
    ));
    let erro = sem_uv.response().error.clone().unwrap();
    assert!(erro.message.contains("uv nao foi detectado"), "{erro:?}");
    let sem_python = core.handle_request(&JsonRpcRequest::new(
        5_i64,
        "python.createEnvironment",
        Some(json!({})),
    ));
    assert!(
        sem_python
            .response()
            .error
            .clone()
            .unwrap()
            .message
            .contains("nenhum python3"),
        "{:?}",
        sem_python.response().error
    );
    let ruim = core.handle_request(&JsonRpcRequest::new(
        6_i64,
        "python.createEnvironment",
        Some(json!({ "tool": "conda" })),
    ));
    assert!(
        ruim.response().error.is_some(),
        "tool desconhecida e' erro de params"
    );
}

/// Com jobs: o `uv` DETECTADO e' o que roda (um falso que grava o pedido e
/// cria o interpretador), o evento sai com sucesso, e o contexto do projeto
/// passa a apontar o ambiente novo. O mesmo `uv` falso, mandado nao criar o
/// interpretador, produz `success: false` mesmo saindo com 0.
#[test]
fn the_job_runs_the_detected_uv_and_the_context_follows_the_new_environment() {
    let _serial = EXECUTAVEIS.lock().unwrap();
    let raiz = temp_dir("job");
    escrever(&raiz.join("pyproject.toml"), "[project]\nname = \"demo\"\n");
    escrever(&raiz.join("app.py"), "def main():\n    pass\n");
    let bin = temp_dir("job-bin");
    let registro = raiz.join("pedido.txt");
    script(&bin.join("python3"), "echo Python 3.14.7");
    script(
        &bin.join("uv"),
        &format!(
            "echo \"$@\" > {reg}\nif [ -z \"$KINEIN_UV_FALHA\" ] && [ \"$1\" = venv ]; then mkdir -p \"$2/bin\"; printf '#!/bin/sh\\necho Python 3.13.0\\n' > \"$2/bin/python\"; chmod +x \"$2/bin/python\"; fi\necho criado",
            reg = registro.display()
        ),
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    core.enable_lsp(sender);
    abrir(&mut core, &raiz);
    let espera = |core: &mut Core, ate: &dyn Fn(&JsonRpcRequest) -> bool| {
        let limite = Instant::now() + Duration::from_secs(20);
        loop {
            let evento = receiver
                .recv_timeout(limite.saturating_duration_since(Instant::now()))
                .expect("evento dentro do prazo");
            core.observe_notification(&evento);
            if ate(&evento) {
                return evento;
            }
        }
    };
    espera(&mut core, &|e| {
        e.method == "event.index.finished"
            && e.params.as_ref().is_some_and(|p| p["state"] == "ready")
    });
    let antes = status(&mut core);
    assert_eq!(antes["interpreter"]["origin"], "sistema");

    let aceito = core.handle_request(&JsonRpcRequest::new(
        7_i64,
        "python.createEnvironment",
        Some(json!({})),
    ));
    let job_id = aceito.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let fim = espera(&mut core, &|e| e.method == "event.python.finished");
    let p = fim.params.unwrap();
    assert_eq!(p["jobId"], job_id);
    assert_eq!(p["success"], true, "{p}");
    assert_eq!(p["tool"], "uv");
    assert_eq!(p["command"], "uv venv .venv");
    assert_eq!(p["path"], raiz.join(".venv").display().to_string());
    assert_eq!(
        std::fs::read_to_string(&registro).unwrap().trim(),
        "venv .venv",
        "o uv recebeu exatamente o comando da fonte"
    );
    // O contexto seguiu o ambiente: o job.finished do recarregamento chega e
    // o interpretador do projeto passa a ser o do .venv.
    espera(&mut core, &|e| {
        e.method == "event.index.finished"
            && e.params
                .as_ref()
                .is_some_and(|p| p["context"]["pythonOrigin"] == ".venv")
    });
    let depois = status(&mut core);
    assert_eq!(depois["interpreter"]["origin"], ".venv", "{depois}");
    assert_eq!(depois["hasEnvironment"], true);
    let ctx = core
        .handle_request(&JsonRpcRequest::new(
            8_i64,
            "index.context",
            Some(json!({ "path": "app.py" })),
        ))
        .response()
        .result
        .clone()
        .unwrap();
    assert_eq!(ctx["python"]["origin"], ".venv", "{ctx}");
    assert_eq!(ctx["python"]["version"], "Python 3.13.0");

    // A ferramenta que sai com 0 sem criar o interpretador NAO e' sucesso: o
    // evento diz false, e o job falha — em vez de a IDE anunciar um ambiente
    // que nao existe.
    std::fs::remove_dir_all(raiz.join(".venv")).unwrap();
    script(&bin.join("uv"), "echo fingindo");
    let aceito = core.handle_request(&JsonRpcRequest::new(
        9_i64,
        "python.createEnvironment",
        Some(json!({ "tool": "uv" })),
    ));
    assert!(aceito.response().error.is_none());
    let segundo = aceito.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let fim = espera(&mut core, &|e| e.method == "event.python.finished");
    assert_eq!(fim.params.clone().unwrap()["success"], false, "{fim:?}");
    let job = espera(&mut core, &|e| {
        e.method == "event.job.finished"
            && e.params
                .as_ref()
                .is_some_and(|p| p["jobId"] == segundo.as_str())
    });
    assert_eq!(job.params.clone().unwrap()["status"], "failed", "{job:?}");
}

/// C4: os stubs da placa. Num projeto `MicroPython` (o `main.py` importa
/// `machine`) o `python.status` sugere o pacote pelo chip do kit; o
/// `python.stubs` roda o `uv pip install --target typings` DETECTADO com o
/// pacote sugerido (ou o pedido), o desfecho e' `event.python.stubs`, e o
/// status passa a dizer onde os stubs estao. Sem sugestao nem pedido, recusa
/// dizendo o que fixar.
#[test]
fn the_board_stubs_are_suggested_installed_into_typings_and_reported() {
    let _serial = EXECUTAVEIS.lock().unwrap();
    let raiz = temp_dir("stubs");
    escrever(
        &raiz.join("main.py"),
        "import machine\nprint(machine.Pin(2))\n",
    );
    let bin = temp_dir("stubs-bin");
    let registro = raiz.join("pedido.txt");
    script(&bin.join("python3"), "echo Python 3.14.7");
    // O uv falso: grava os argv e "instala" um .pyi na pasta --target.
    script(
        &bin.join("uv"),
        &format!(
            "echo \"$@\" > {reg}\nalvo=\"\"\nwhile [ $# -gt 0 ]; do if [ \"$1\" = --target ]; then alvo=\"$2\"; fi; shift; done\nmkdir -p \"$alvo\" && touch \"$alvo/machine.pyi\"\necho instalado",
            reg = registro.display()
        ),
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = Core::with_detector(ToolDetector::with_search_path(&bin));
    core.enable_lsp(sender);
    abrir(&mut core, &raiz);
    let espera = |core: &mut Core, ate: &dyn Fn(&JsonRpcRequest) -> bool| {
        let limite = Instant::now() + Duration::from_secs(20);
        loop {
            let evento = receiver
                .recv_timeout(limite.saturating_duration_since(Instant::now()))
                .expect("evento dentro do prazo");
            core.observe_notification(&evento);
            if ate(&evento) {
                return evento;
            }
        }
    };
    espera(&mut core, &|e| e.method == "event.index.finished");

    // Sem chip no kit: MicroPython sem placa nao tem sugestao, e o pedido
    // sem `port` e' recusado dizendo o que fixar.
    let antes = status(&mut core);
    assert_eq!(antes["stubsPath"], Value::Null);
    assert_eq!(antes["stubsSuggested"], Value::Null, "{antes}");
    let recusa = core.handle_request(&JsonRpcRequest::new(8_i64, "python.stubs", Some(json!({}))));
    let erro = recusa.response().error.clone().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(
        erro.message.contains("fixe o chip no kit"),
        "{}",
        erro.message
    );

    // O chip no kit: a sugestao segue a lista publicada (placa C3).
    let kit = core.handle_request(&JsonRpcRequest::new(
        9_i64,
        "toolchain.setKit",
        Some(json!({ "chip": "esp32c3" })),
    ));
    assert!(kit.response().error.is_none(), "{:?}", kit.response().error);
    let com_chip = status(&mut core);
    assert_eq!(
        com_chip["stubsSuggested"], "micropython-esp32-esp32_generic_c3-stubs",
        "{com_chip}"
    );

    // Instalar: a linha do uv com --target <root>/typings e o pacote sugerido.
    let aceito = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "python.stubs",
        Some(json!({})),
    ));
    let r = aceito.response().result.clone().expect("aceito");
    assert_eq!(r["package"], "micropython-esp32-esp32_generic_c3-stubs");
    assert_eq!(r["target"], raiz.join("typings").display().to_string());
    let fim = espera(&mut core, &|e| e.method == "event.python.stubs");
    let p = fim.params.unwrap();
    assert_eq!(p["jobId"], r["jobId"]);
    assert_eq!(p["success"], true, "{p}");
    assert_eq!(
        std::fs::read_to_string(&registro).unwrap().trim(),
        format!(
            "pip install -U --link-mode=copy --target {} micropython-esp32-esp32_generic_c3-stubs",
            raiz.join("typings").display()
        )
    );
    assert!(raiz.join("typings/machine.pyi").is_file());
    let depois = status(&mut core);
    assert_eq!(
        depois["stubsPath"],
        raiz.join("typings").display().to_string()
    );

    // O pedido explicito vence a sugestao.
    let aceito = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "python.stubs",
        Some(json!({ "port": "rp2", "board": "rpi_pico_w" })),
    ));
    assert_eq!(
        aceito.response().result.clone().unwrap()["package"],
        "micropython-rp2-rpi_pico_w-stubs"
    );
    espera(&mut core, &|e| e.method == "event.python.stubs");
}
