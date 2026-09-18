//! `serial.identify` (E5 do `integracoes/38` §6, 2026-09-17): a identidade
//! Espressif pelo canal, como JOB, com o esptool FALSO — sem placa nesta
//! maquina. O que se prova: a linha de comando que o esptool recebe; o parser
//! sobre a saida da v5 e a queda para a v4; a recusa ANTES de abrir a porta
//! (no' inexistente, sem permissao, sem ferramenta); o desfecho em
//! `event.serial.identified` com a sugestao de kit; e o cancelamento matando
//! o processo.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

/// A saida da v5 (a mesma fixture de `serial/identify.rs`), ecoada pelo
/// esptool falso junto com os argv — para provar o comando E o parser.
const SAIDA_V5: &str = "esptool v5.4.0\nConnected to ESP32-C3 on $PORTA:\nChip type:          ESP32-C3 (QFN32) (revision v0.4)\nFeatures:           Wi-Fi, BLE\nCrystal frequency:  40MHz\nMAC:                34:b4:72:0a:1b:2c\n\nFlash Memory Information:\n=========================\nManufacturer: c8\nDevice: 4016\nDetected flash size: 4MB\n";

struct Cenario {
    core: crate::Core,
    dir: PathBuf,
    porta: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str) -> Cenario {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-serial-identify-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    // A "porta" e' um arquivo comum: o que o handler mede e' existencia e
    // permissao, e o esptool falso nao a abre.
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
    fn esptool_falso(&self, corpo: &str) {
        executavel(&self.dir.join("bin/esptool"), corpo);
    }

    fn identify(&mut self, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(7_i64, "serial.identify", Some(params)))
            .response()
            .clone()
    }

    fn identified(&self) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(15);
        while std::time::Instant::now() < prazo {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50)) {
                if event.method == "event.serial.identified" {
                    return event.params.unwrap();
                }
            }
        }
        panic!("event.serial.identified nao chegou");
    }
}

/// O caminho feliz: comando com a porta e o `--after hard-reset`, parser
/// sobre a v5, sugestao de kit pela tabela do project.model, job com sucesso.
#[test]
#[cfg(unix)]
fn a_v5_esptool_yields_the_identity_and_a_kit_suggestion() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("v5");
    c.esptool_falso(&format!(
        "#!/bin/sh\necho \"argv: $*\"\nprintf '%s' '{}'\n",
        SAIDA_V5.replace("$PORTA", "/dev/x")
    ));
    let porta = c.porta.display().to_string();
    let resposta = c.identify(json!({ "device": porta }));
    let resultado = resposta.result.expect("aceita");
    assert!(resultado["jobId"].as_str().unwrap().starts_with("job_"));
    assert_eq!(
        resultado["command"],
        format!(
            "{} --port {porta} --chip auto --before default-reset --after hard-reset flash-id",
            c.dir.join("bin/esptool").display()
        )
    );

    let evento = c.identified();
    assert_eq!(evento["success"], true, "{evento}");
    assert_eq!(evento["device"], porta);
    assert!(
        evento["raw"].as_str().unwrap().contains(&format!(
            "argv: --port {porta} --chip auto --before default-reset --after hard-reset flash-id"
        )),
        "{}",
        evento["raw"]
    );
    let id = &evento["identity"];
    assert_eq!(id["chip"], "esp32c3");
    assert_eq!(id["mac"], "34:b4:72:0a:1b:2c");
    assert_eq!(id["flashSize"], "4MB");
    assert_eq!(id["flashSizeBytes"], 4_194_304);
    assert_eq!(id["features"], json!(["Wi-Fi", "BLE"]));
    let alvo = &evento["target"];
    assert_eq!(alvo["chip"], "esp32c3");
    assert_eq!(alvo["family"], "espressif");
    assert_eq!(alvo["flashEngine"], "esptool");
    assert_eq!(alvo["monitor"], "espflash");
    assert_eq!(alvo["debugAdapter"], "probe-rs");
    assert!(
        alvo["evidence"][0]
            .as_str()
            .unwrap()
            .contains("esptool flash-id"),
        "{alvo}"
    );
}

/// Uma v4 nao conhece `flash-id`: o job cai para `flash_id` e o comando
/// ecoado no evento e' o segundo.
#[test]
#[cfg(unix)]
fn a_v4_esptool_gets_the_underscore_command_on_retry() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("v4");
    c.esptool_falso(
        "#!/bin/sh\ncase \"$*\" in\n  *flash-id) echo \"esptool.py: error: argument operation: \
         invalid choice: 'flash-id'\" >&2; exit 2 ;;\n  *flash_id) echo 'Chip type:          \
         ESP32-D0WD-V3 (revision v3.1)'; echo 'Detected flash size: Unknown' ;;\nesac\n",
    );
    let porta = c.porta.display().to_string();
    assert!(c.identify(json!({ "device": porta })).error.is_none());
    let evento = c.identified();
    assert_eq!(evento["success"], true, "{evento}");
    assert!(
        evento["command"].as_str().unwrap().ends_with(" flash_id"),
        "{}",
        evento["command"]
    );
    assert_eq!(
        evento["identity"]["chip"], "esp32",
        "encapsulamento D0WD-V3 -> esp32"
    );
    assert_eq!(evento["identity"]["flashSize"], Value::Null);
    // ESP32 classico: sem JTAG embutido, sem depurador sugerido.
    assert_eq!(evento["target"]["debugAdapter"], Value::Null);
}

/// A recusa vem ANTES de abrir a porta: no' inexistente, sem permissao,
/// sem esptool, campo desconhecido — e nenhum job nasce.
#[test]
#[cfg(unix)]
fn refusals_happen_before_the_port_is_opened() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("recusas");
    let porta = c.porta.display().to_string();

    let sem_ferramenta = c.identify(json!({ "device": porta })).error.unwrap();
    assert_eq!(sem_ferramenta.code, JsonRpcErrorCode::ToolNotFound);
    assert!(sem_ferramenta.message.contains("pipx install esptool"));

    c.esptool_falso("#!/bin/sh\necho nunca\n");
    let inexistente = c
        .identify(json!({ "device": c.dir.join("ttyNAO").display().to_string() }))
        .error
        .unwrap();
    assert_eq!(inexistente.code, JsonRpcErrorCode::InvalidParams);

    let campo = c
        .identify(json!({ "device": porta, "baud": 9600 }))
        .error
        .unwrap();
    assert_eq!(campo.code, JsonRpcErrorCode::InvalidParams);

    // Sem permissao: um no' so'-leitura do proprio usuario (root ignoraria o
    // modo, e nesse caso o teste nao prova nada — pula com aviso).
    std::fs::set_permissions(&c.porta, std::fs::Permissions::from_mode(0o444)).unwrap();
    let resposta = c.identify(json!({ "device": porta }));
    if rustix::process::geteuid().is_root() {
        eprintln!("rodando como root: a prova de permissao nao vale");
    } else {
        let erro = resposta.error.expect("sem permissao recusa");
        assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest, "{erro:?}");
        assert!(erro.message.contains("r--r--r--"), "{erro:?}");
    }
    std::fs::set_permissions(&c.porta, std::fs::Permissions::from_mode(0o644)).unwrap();

    // Nenhum job foi criado por nenhuma recusa.
    let jobs = c
        .core
        .handle_request(&JsonRpcRequest::new(8_i64, "job.list", Some(json!({}))))
        .response()
        .result
        .clone()
        .unwrap();
    assert!(jobs["jobs"].as_array().unwrap().is_empty(), "{jobs}");
}

/// O esptool que nao acha placa fala e sai com erro: `success: false`, o
/// erro e' o que ele disse, e nao ha' identidade nem sugestao.
#[test]
#[cfg(unix)]
fn a_failing_esptool_reports_its_last_lines() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("falha");
    c.esptool_falso(
        "#!/bin/sh\necho 'Connecting......'\necho 'A fatal error occurred: Failed to connect \
         to Espressif device: No serial data received.' >&2\nexit 2\n",
    );
    let porta = c.porta.display().to_string();
    assert!(c.identify(json!({ "device": porta })).error.is_none());
    let evento = c.identified();
    assert_eq!(evento["success"], false);
    assert!(
        evento["error"]
            .as_str()
            .unwrap()
            .contains("Failed to connect"),
        "{evento}"
    );
    assert_eq!(evento["identity"], Value::Null);
    assert_eq!(evento["target"], Value::Null);
}

/// Cancelar o job MATA o esptool (que aqui dormiria 30 s) e o desfecho diz
/// "cancelada" — nao "expirou".
#[test]
#[cfg(unix)]
fn cancelling_the_job_kills_esptool() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("cancela");
    c.esptool_falso("#!/bin/sh\necho 'Connecting......'\nexec sleep 30\n");
    let porta = c.porta.display().to_string();
    let job_id = c.identify(json!({ "device": porta })).result.unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    // Espera o processo estar rodando (a primeira linha saiu como output).
    let prazo = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(event) = c.events.recv_timeout(Duration::from_millis(50)) {
            if event.method == "event.job.output"
                && event.params.as_ref().unwrap()["line"] == "Connecting......"
            {
                break;
            }
        }
        assert!(
            std::time::Instant::now() < prazo,
            "o esptool falso nao comecou"
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
        .result
        .clone()
        .unwrap();
    assert_eq!(cancelado["cancelled"], true);
    let evento = c.identified();
    assert!(
        inicio.elapsed() < Duration::from_secs(10),
        "cancelar levou {:?}: o sleep de 30 s nao foi morto",
        inicio.elapsed()
    );
    assert_eq!(evento["success"], false);
    assert!(
        evento["error"].as_str().unwrap().contains("cancelada"),
        "{evento}"
    );
}
