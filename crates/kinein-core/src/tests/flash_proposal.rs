//! E4 do `integracoes/38` §6 (2026-09-17): gravar como CONFIGURACAO DE
//! EXECUCAO, pelo despacho real. Um workspace ESP-IDF com o
//! `flasher_args.json` na forma exata do template do IDF, um esptool FALSO
//! que ecoa os argv, e o ciclo inteiro que a tela faz:
//! `runConfig.flashProposal` -> `runConfig.save` -> `runConfig.setActive` ->
//! `run.start {}` — e a saida do "motor" na sessao de execucao.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

/// A receita como o ESP-IDF a escreve (a mesma de `tests/project.rs`).
const FLASHER_ARGS: &str = r#"{
    "write_flash_args" : [ "--flash-mode", "dio", "--flash-size", "4MB", "--flash-freq", "40m" ],
    "flash_settings" : { "flash_mode": "dio", "flash_size": "4MB", "flash_freq": "40m" },
    "flash_files" : {
        "0x1000" : "bootloader/bootloader.bin",
        "0x8000" : "partition_table/partition-table.bin",
        "0x10000" : "hello world.bin"
    },
    "bootloader" : { "offset" : "0x1000", "file" : "bootloader/bootloader.bin", "encrypted" : "false" },
    "app" : { "offset" : "0x10000", "file" : "hello world.bin", "encrypted" : "false" },
    "partition-table" : { "offset" : "0x8000", "file" : "partition_table/partition-table.bin", "encrypted" : "false" },
    "extra_esptool_args" : { "after" : "hard-reset", "before" : "default-reset", "stub" : true, "chip" : "esp32c3" }
}"#;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/fixtures/projetos")
        .canonicalize()
        .unwrap()
}

struct Cenario {
    core: crate::Core,
    dir: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str, com_receita: bool) -> Cenario {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-flash-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    // CMakeLists + sdkconfig: e' o que faz o modelo dizer ESP-IDF, esp32c3,
    // familia espressif e motor esptool (o caso "aviso" prova o outro caminho:
    // sem sdkconfig, a receita sozinha sugere o esptool).
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        dir.join("CMakeLists.txt"),
    )
    .unwrap();
    if nome != "aviso" {
        std::fs::copy(fixtures().join("esp-idf/sdkconfig"), dir.join("sdkconfig")).unwrap();
    }
    if com_receita {
        std::fs::create_dir_all(dir.join("build/bootloader")).unwrap();
        std::fs::create_dir_all(dir.join("build/partition_table")).unwrap();
        std::fs::write(dir.join("build/flasher_args.json"), FLASHER_ARGS).unwrap();
    }
    let dir = dir.canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    Cenario {
        core,
        dir,
        events: receiver,
    }
}

impl Cenario {
    fn esptool_falso(&self) {
        let caminho = self.dir.join("bin/esptool");
        std::fs::write(&caminho, "#!/bin/sh\necho \"esptool-falso $*\"\n").unwrap();
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn rpc(&mut self, id: i64, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(id, method, Some(params)))
            .response()
            .clone()
    }

    fn proposta(&mut self, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.rpc(2, "runConfig.flashProposal", params)
    }

    /// As linhas de `event.run.output` ate' o `event.run.finished`.
    fn saida(&self) -> (Vec<String>, bool) {
        let mut linhas = Vec::new();
        let prazo = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < prazo {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(50)) {
                match event.method.as_str() {
                    "event.run.output" => {
                        linhas.push(event.params.unwrap()["line"].as_str().unwrap().to_owned());
                    }
                    "event.run.finished" => {
                        return (linhas, event.params.unwrap()["success"] == true);
                    }
                    _ => {}
                }
            }
        }
        panic!("o run.start nao terminou; visto: {linhas:?}");
    }
}

/// O ciclo inteiro: a proposta vem da receita e da porta; salva como
/// configuracao, ativa, e o botao Executar (`run.start {}`) roda a linha —
/// o esptool falso recebe EXATAMENTE os argv da receita, com o arquivo com
/// espaco inteiro.
#[test]
#[cfg(unix)]
fn the_proposal_becomes_a_run_configuration_that_run_start_executes() {
    let mut c = cenario("ciclo", true);
    c.esptool_falso();
    let resposta = c.proposta(json!({ "device": "/dev/ttyUSB0" }));
    let proposta = resposta
        .result
        .unwrap_or_else(|| panic!("{:?}", resposta.error));
    assert_eq!(proposta["engine"], "esptool");
    assert_eq!(proposta["name"], "Gravar (esptool)");
    let comando = proposta["command"].as_str().unwrap().to_owned();
    let build = c.dir.join("build");
    assert_eq!(
        comando,
        format!(
            "'{esptool}' --chip esp32c3 --port '/dev/ttyUSB0' --baud 460800 --before default-reset \
             --after hard-reset write-flash --flash-mode dio --flash-size 4MB --flash-freq 40m \
             0x1000 '{b}/bootloader/bootloader.bin' 0x8000 '{b}/partition_table/partition-table.bin' \
             0x10000 '{b}/hello world.bin'",
            esptool = c.dir.join("bin/esptool").display(),
            b = build.display()
        )
    );
    assert!(
        proposta["source"].as_array().unwrap().len() >= 4,
        "{proposta}"
    );
    assert!(
        proposta["warnings"].as_array().is_none_or(Vec::is_empty),
        "{proposta}"
    );

    let salvo = c
        .rpc(
            3,
            "runConfig.save",
            json!({ "name": proposta["name"], "command": comando }),
        )
        .result
        .unwrap();
    let id = salvo["activeId"].as_str().unwrap().to_owned();
    assert_eq!(salvo["configs"][0]["name"], "Gravar (esptool)");

    let iniciado = c.rpc(4, "run.start", json!({})).result.unwrap();
    assert_eq!(iniciado["command"], comando);
    let (linhas, sucesso) = c.saida();
    assert!(sucesso);
    assert_eq!(
        linhas,
        [format!(
            "esptool-falso --chip esp32c3 --port /dev/ttyUSB0 --baud 460800 --before default-reset \
             --after hard-reset write-flash --flash-mode dio --flash-size 4MB --flash-freq 40m \
             0x1000 {b}/bootloader/bootloader.bin 0x8000 {b}/partition_table/partition-table.bin \
             0x10000 {b}/hello world.bin",
            b = build.display()
        )]
    );
    // A configuracao ativa continua sendo a de gravar (o usuario a ve no menu).
    let lista = c.rpc(5, "runConfig.list", json!({})).result.unwrap();
    assert_eq!(lista["activeId"], id);
}

/// As recusas, cada uma com o codigo certo e sem nada salvo: sem porta
/// (`INVALID_REQUEST`), sem receita (`INVALID_REQUEST`), sem esptool
/// (`TOOL_NOT_FOUND`), motor desconhecido (`INVALID_PARAMS`), campo desconhecido.
#[test]
#[cfg(unix)]
fn refusals_name_the_missing_piece_and_save_nothing() {
    let mut c = cenario("recusas", true);
    let sem_esptool = c
        .proposta(json!({ "device": "/dev/ttyUSB0" }))
        .error
        .unwrap();
    assert_eq!(sem_esptool.code, JsonRpcErrorCode::ToolNotFound);
    assert!(sem_esptool.message.contains("pipx install esptool"));

    c.esptool_falso();
    let sem_porta = c.proposta(json!({})).error.unwrap();
    assert_eq!(sem_porta.code, JsonRpcErrorCode::InvalidRequest);
    assert!(
        sem_porta.message.contains("escolha a porta"),
        "{sem_porta:?}"
    );

    let desconhecido = c
        .proposta(json!({ "engine": "avrdude", "device": "/dev/ttyUSB0" }))
        .error
        .unwrap();
    assert_eq!(desconhecido.code, JsonRpcErrorCode::InvalidParams);

    let campo = c.proposta(json!({ "port": "/dev/ttyUSB0" })).error.unwrap();
    assert_eq!(campo.code, JsonRpcErrorCode::InvalidParams);

    let lista = c.rpc(6, "runConfig.list", json!({})).result.unwrap();
    assert!(lista["configs"].as_array().unwrap().is_empty());

    // Sem receita (nao compilou): a mensagem diz o que fazer.
    let mut sem_build = cenario("sem-build", false);
    sem_build.esptool_falso();
    let erro = sem_build
        .proposta(json!({ "device": "/dev/ttyUSB0" }))
        .error
        .unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains("compile o projeto"), "{erro:?}");
}

/// A flash que a placa relatou (E5) menor que a receita vira aviso, nao
/// recusa: quem grava e' o usuario, informado.
#[test]
#[cfg(unix)]
fn a_smaller_reported_flash_is_a_warning() {
    let mut c = cenario("aviso", true);
    c.esptool_falso();
    let proposta = c
        .proposta(json!({ "device": "/dev/ttyUSB0", "flashSizeBytes": 2_097_152 }))
        .result
        .unwrap();
    let avisos = proposta["warnings"].as_array().unwrap();
    assert_eq!(avisos.len(), 1, "{proposta}");
    assert!(avisos[0].as_str().unwrap().contains("2MB"));
    // Sem sdkconfig o modelo nao tem chip; a receita sozinha sugeriu o motor
    // e deu o chip.
    assert!(
        proposta["source"][0]
            .as_str()
            .unwrap()
            .contains("receita flasher_args.json"),
        "{proposta}"
    );
    assert!(
        proposta["command"]
            .as_str()
            .unwrap()
            .contains("--chip esp32c3")
    );
}

/// C5: um firmware do catalogo, BAIXADO na pasta da IDE, grava-se pela
/// proposta do E4 — a linha da pagina da placa (offset 0x1000 no ESP32
/// classico), com a porta escolhida e o aviso do erase-flash; e roda de
/// verdade pelo `run.start` com o esptool falso. O que nao foi baixado, ou
/// nao e' firmware, e' recusado antes de compor a linha.
#[test]
#[cfg(unix)]
fn a_downloaded_firmware_is_flashed_by_the_page_line_and_runs() {
    let mut c = cenario("firmware", false);
    c.esptool_falso();
    let raiz = c.dir.join("toolchains");
    c.core = {
        let (sender, receiver) = mpsc::channel();
        let mut core = crate::Core::with_detector(
            crate::tools::ToolDetector::with_search_path(c.dir.join("bin"))
                .with_install_root(&raiz),
        );
        core.enable_lsp(sender);
        c.events = receiver;
        core
    };
    assert!(
        c.rpc(
            1,
            "workspace.open",
            json!({ "path": c.dir.to_str().unwrap() })
        )
        .error
        .is_none()
    );

    // Ainda nao baixado: recusa que diz como baixar. Uma toolchain no lugar
    // de um firmware: recusa. Id desconhecido: recusa.
    for (id, trecho) in [
        ("micropython-esp32-generic", "ainda nao foi baixado"),
        ("arm-gnu-arm-none-eabi", "nao um firmware"),
        ("nao-existe", "nao esta' no catalogo"),
    ] {
        let erro = c
            .proposta(json!({ "device": "/dev/ttyUSB0", "firmware": id }))
            .error
            .unwrap();
        assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest, "{id}");
        assert!(erro.message.contains(trecho), "{id}: {}", erro.message);
    }

    // "Baixado": o arquivo onde o provedor o deixaria.
    let pasta = raiz.join("micropython-esp32-generic/v1.29.0");
    std::fs::create_dir_all(&pasta).unwrap();
    let bin = pasta.join("ESP32_GENERIC-20260824-v1.29.0.bin");
    std::fs::write(&bin, vec![0xE9; 4096]).unwrap();
    let r = c
        .proposta(json!({ "device": "/dev/ttyUSB0", "firmware": "micropython-esp32-generic" }))
        .result
        .expect("proposta");
    assert_eq!(
        r["command"],
        format!(
            "'{}' --chip esp32 --port '/dev/ttyUSB0' --baud 460800 --before default-reset \
             --after hard-reset write-flash 0x1000 '{}'",
            c.dir.join("bin/esptool").display(),
            bin.display()
        )
    );
    assert_eq!(r["engine"], "esptool");
    assert!(r["name"].as_str().unwrap().starts_with("Gravar firmware"));
    assert!(
        r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("erase-flash")),
        "{r}"
    );
    // O kit deste projeto diz esp32c3 (sdkconfig) e o firmware e' do esp32
    // classico: o aviso nomeia os dois.
    assert!(
        r["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("kit diz chip esp32c3")),
        "{r}"
    );
    // O catalogo agora o mostra instalado.
    let lista = c.rpc(3, "toolchain.installable", json!({})).result.unwrap();
    let fw = lista["toolchains"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "micropython-esp32-generic")
        .unwrap();
    assert_eq!(fw["installed"], true);

    // E roda de verdade: o esptool falso ecoa os argv.
    let started = c.rpc(4, "run.start", json!({ "command": r["command"] }));
    assert!(started.error.is_none(), "{:?}", started.error);
    let (linhas, sucesso) = c.saida();
    assert!(sucesso);
    assert!(
        linhas.iter().any(
            |l| l.contains("esptool-falso --chip esp32 --port /dev/ttyUSB0")
                && l.contains("write-flash 0x1000")
        ),
        "{linhas:?}"
    );
}
