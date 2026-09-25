//! Bloco E do `roadmaps/41` (2026-09-17): os frameworks como MOTORES de
//! build/gravar/monitorar, por despacho, com as ferramentas FALSAS ecoando
//! os argv — nenhum SDK real nesta maquina. O que se prova: o motor que o
//! modelo escolhe por projeto; a linha que cada wrapper recebe (`idf.py
//! build` no ambiente ativado, `west build -d build -b <placa>`, `pio run`,
//! `cmake … -DPICO_SDK_PATH=…`); a recusa ANTES do job quando o framework
//! esta' la' e a ferramenta nao; o "Gravar" pelo wrapper; e o `platformio.ini`
//! como tipo de projeto.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/fixtures/projetos")
        .canonicalize()
        .unwrap()
}

struct Cenario {
    core: crate::Core,
    base: PathBuf,
    root: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

/// Copia a fixture do framework para uma raiz nova, com `bin/` (o PATH do
/// detector) e `home/` (o $HOME do modelo) ao lado.
fn cenario(nome: &str, fixture: &str) -> Cenario {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-frameworks-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = base.join("projeto");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("home")).unwrap();
    for entrada in std::fs::read_dir(fixtures().join(fixture))
        .unwrap()
        .flatten()
    {
        let destino = root.join(entrada.file_name());
        if entrada.path().is_dir() {
            copiar_dir(&entrada.path(), &destino);
        } else {
            std::fs::copy(entrada.path(), destino).unwrap();
        }
    }
    let base = base.canonicalize().unwrap();
    let root = base.join("projeto");
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ))
    .with_home(base.join("home"));
    core.enable_lsp(sender);
    Cenario {
        core,
        base,
        root,
        events: receiver,
    }
}

fn copiar_dir(de: &Path, para: &Path) {
    std::fs::create_dir_all(para).unwrap();
    for e in std::fs::read_dir(de).unwrap().flatten() {
        let destino = para.join(e.file_name());
        if e.path().is_dir() {
            copiar_dir(&e.path(), &destino);
        } else {
            std::fs::copy(e.path(), destino).unwrap();
        }
    }
}

fn executavel(caminho: &Path, corpo: &str) {
    if let Some(pasta) = caminho.parent() {
        std::fs::create_dir_all(pasta).unwrap();
    }
    std::fs::write(caminho, corpo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

impl Cenario {
    fn falso(&self, nome: &str, corpo: &str) {
        executavel(
            &self.base.join("bin").join(nome),
            &format!("#!/bin/sh\necho \"{nome}-falso $*\"\n{corpo}"),
        );
    }

    fn abrir(&mut self) -> Value {
        let r = self.core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": self.root.to_str().unwrap() })),
        ));
        assert!(r.response().error.is_none(), "{:?}", r.response().error);
        r.response().result.clone().unwrap()
    }

    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(2_i64, method, Some(params)))
            .response()
            .clone()
    }

    /// As linhas do job de build (comando e saida) e o sucesso.
    fn build(&mut self) -> (Vec<String>, bool) {
        let started = self.rpc("build.run", json!({}));
        assert!(started.error.is_none(), "{:?}", started.error);
        let job_id = started.result.unwrap()["jobId"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut linhas = Vec::new();
        loop {
            let event = self
                .events
                .recv_timeout(Duration::from_secs(30))
                .expect("eventos do build");
            let params = event.params.clone().unwrap_or_default();
            if params["jobId"] != job_id.as_str() {
                continue;
            }
            match event.method.as_str() {
                "event.build.output" => linhas.push(params["line"].as_str().unwrap().to_owned()),
                "event.build.started" => {
                    linhas.push(format!("$ {}", params["command"].as_str().unwrap()));
                }
                "event.build.finished" => return (linhas, params["success"] == true),
                _ => {}
            }
        }
    }
}

/// ESP-IDF: o `build.run` roda `idf.py build` DENTRO do ambiente ativado — o
/// `export.sh` falso poe o `idf.py` falso no PATH, como o real poe o do IDF.
/// Sem ativacao nenhuma, a recusa diz o passo (e o EIM). O "Gravar" com o
/// motor `idf.py` compoe a mesma ativacao com `-p <porta> flash`.
#[test]
#[cfg(unix)]
fn esp_idf_builds_through_the_activated_environment_or_says_how_to_get_it() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("idf", "esp-idf");
    c.abrir();
    // Sem export.sh nem EIM: framework reconhecido, ferramenta ausente.
    let erro = c.rpc("build.run", json!({})).error.expect("recusa");
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound);
    assert!(
        erro.message.contains("ESP-IDF") && erro.message.contains("eim install"),
        "{}",
        erro.message
    );

    // O IDF "instalado": ~/esp/esp-idf/export.sh que poe o idf.py falso no PATH.
    let idfbin = c.base.join("idfbin");
    executavel(
        &idfbin.join("idf.py"),
        "#!/bin/sh\necho \"idf.py-falso $*\"\nif [ \"$1\" = build ]; then mkdir -p build; echo '{}' > build/flasher_args.json; fi\n",
    );
    let export = c.base.join("home/esp/esp-idf/export.sh");
    executavel(
        &export,
        &format!(
            "echo 'banner do export.sh'\nexport PATH=\"{}:$PATH\"\n",
            idfbin.display()
        ),
    );
    let (linhas, sucesso) = c.build();
    assert!(sucesso, "{linhas:?}");
    assert!(linhas.iter().any(|l| l == "$ idf.py build"), "{linhas:?}");
    assert!(
        linhas.iter().any(|l| l == "idf.py-falso build"),
        "{linhas:?}"
    );
    assert!(
        linhas
            .iter()
            .any(|l| l.contains("ativando o ESP-IDF") && l.contains("export.sh")),
        "{linhas:?}"
    );
    assert!(
        !linhas.iter().any(|l| l.contains("banner")),
        "o banner da ativacao nao vaza: {linhas:?}"
    );
    assert!(c.root.join("build/flasher_args.json").is_file());

    // Gravar pelo wrapper: a linha ativa e chama `idf.py -p <porta> flash`.
    let p = c
        .rpc(
            "runConfig.flashProposal",
            json!({ "engine": "idf.py", "device": "/dev/ttyUSB0" }),
        )
        .result
        .expect("proposta");
    let comando = p["command"].as_str().unwrap();
    assert!(comando.starts_with("bash -c '"), "{comando}");
    assert!(
        comando.ends_with(&format!(
            "' idf '{}' -p '/dev/ttyUSB0' flash",
            export.display()
        )),
        "{comando}"
    );
    assert_eq!(p["engine"], "idf.py");
    // E roda de verdade: o idf.py falso recebe `-p /dev/ttyUSB0 flash`.
    let started = c.rpc("run.start", json!({ "command": comando }));
    assert!(started.error.is_none(), "{:?}", started.error);
    let terminal_id = started.result.as_ref().unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (linhas, _) =
        super::terminal_run_until_closed(&c.events, &terminal_id, Duration::from_secs(15));
    assert!(
        linhas
            .iter()
            .any(|l| l.contains("idf.py-falso -p /dev/ttyUSB0 flash")),
        "o idf.py falso nao recebeu o flash: {linhas:?}"
    );
    // Sem -p: o esptool do E4 continua sendo o padrao (a receita do build).
    let erro = c
        .rpc("runConfig.flashProposal", json!({ "engine": "idf.py" }))
        .error
        .unwrap();
    assert!(erro.message.contains("escolha a porta"), "{}", erro.message);
}

/// Zephyr: `west build -d build -b <placa>`, com a placa do `west config
/// build.board` (o west falso a responde) e, depois do primeiro build, do
/// `CMakeCache`. O "Gravar" padrao e' `west flash -d build` (o modelo sugere
/// `west`). Sem west, a recusa diz o passo.
#[test]
#[cfg(unix)]
fn zephyr_builds_with_west_and_the_board_from_west_config() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("zephyr", "zephyr");
    c.abrir();
    let erro = c.rpc("build.run", json!({})).error.expect("sem west");
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound);
    assert!(
        erro.message.contains("Zephyr") && erro.message.contains("west"),
        "{}",
        erro.message
    );

    // O west falso: `config build.board` responde SO' a placa (como o real);
    // `build` ecoa os argv e deixa o CMakeCache com a placa, como o real.
    executavel(
        &c.base.join("bin/west"),
        "#!/bin/sh\nif [ \"$1 $2\" = \"config build.board\" ]; then echo nrf52840dk/nrf52840; exit 0; fi\necho \"west-falso $*\"\nif [ \"$1\" = build ]; then mkdir -p build; echo 'CACHED_BOARD:STRING=nrf52840dk/nrf52840' > build/CMakeCache.txt; fi\n",
    );
    let (linhas, sucesso) = c.build();
    assert!(sucesso, "{linhas:?}");
    assert!(
        linhas
            .iter()
            .any(|l| l == "$ west build -d build -b nrf52840dk/nrf52840"),
        "{linhas:?}"
    );
    assert!(
        linhas
            .iter()
            .any(|l| l == "west-falso build -d build -b nrf52840dk/nrf52840"),
        "{linhas:?}"
    );

    let p = c
        .rpc("runConfig.flashProposal", json!({}))
        .result
        .expect("west e' o motor sugerido");
    assert_eq!(p["engine"], "west");
    assert_eq!(
        p["command"],
        format!("'{}' flash -d build", c.base.join("bin/west").display())
    );
    assert!(p["warnings"].is_null(), "sem porta nao ha aviso: {p}");
}

/// `PlatformIO`: o `platformio.ini` e' tipo de projeto (`platformIo`), o build
/// e' `pio run`, o "Gravar" e' `pio run -t upload --upload-port <porta>` —
/// e o pio vence ate' um `CMakeLists` do ESP-IDF ao lado.
#[test]
#[cfg(unix)]
fn platformio_is_a_project_kind_built_and_uploaded_by_pio() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("pio", "platformio");
    let info = c.abrir();
    assert_eq!(info["kind"], "platformIo", "{info}");
    assert!(
        info["capabilities"]["buildSystems"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "platformIo"),
        "{info}"
    );
    let erro = c.rpc("build.run", json!({})).error.expect("sem pio");
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound);
    assert!(erro.message.contains("PlatformIO"), "{}", erro.message);

    c.falso("pio", "");
    let (linhas, sucesso) = c.build();
    assert!(sucesso, "{linhas:?}");
    assert!(linhas.iter().any(|l| l == "$ pio run"), "{linhas:?}");
    assert!(linhas.iter().any(|l| l == "pio-falso run"), "{linhas:?}");

    let p = c
        .rpc(
            "runConfig.flashProposal",
            json!({ "device": "/dev/ttyUSB0" }),
        )
        .result
        .expect("platformio e' o motor sugerido");
    assert_eq!(p["engine"], "platformio");
    assert_eq!(
        p["command"],
        format!(
            "'{}' run -t upload --upload-port '/dev/ttyUSB0'",
            c.base.join("bin/pio").display()
        )
    );

    // Um CMakeLists do ESP-IDF ao lado: o tipo vira cmake, o MOTOR continua pio.
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        c.root.join("CMakeLists.txt"),
    )
    .unwrap();
    let info = c.abrir();
    assert_eq!(info["kind"], "cmake");
    let (linhas, _) = c.build();
    assert!(linhas.iter().any(|l| l == "$ pio run"), "{linhas:?}");
}

/// pico-sdk: o `CMake` de sempre, com `-DPICO_SDK_PATH=<sdk>` no configure —
/// tanto no `build.run` quanto no `cmake.configure` — quando o SDK esta' na
/// pasta padrao (`~/pico/pico-sdk`); sem SDK, o `CMake` roda como sempre.
#[test]
#[cfg(unix)]
fn pico_sdk_adds_the_sdk_path_to_the_cmake_configure() {
    let _serial = crate::serializar_executaveis();
    let mut c = cenario("pico", "pico-sdk");
    c.falso(
        "cmake",
        "if [ \"$1\" = -S ]; then mkdir -p \"$4\"; echo 'X' > \"$4/CMakeCache.txt\"; fi\n",
    );
    c.abrir();
    // O cmake falso FIXADO no kit: o build usa o do kit, nao o do PATH do processo.
    let fixado = c.rpc("toolchain.set", json!({ "role": "cmake", "id": "cmake" }));
    assert!(fixado.error.is_none(), "{:?}", fixado.error);
    let (linhas, sucesso) = c.build();
    assert!(sucesso, "{linhas:?}");
    let configure = linhas
        .iter()
        .find(|l| l.starts_with("cmake-falso -S"))
        .expect("configure");
    assert!(
        !configure.contains("PICO_SDK_PATH"),
        "sem SDK nao ha' -D: {configure}"
    );

    let sdk = c.base.join("home/pico/pico-sdk");
    std::fs::create_dir_all(&sdk).unwrap();
    let _ = std::fs::remove_dir_all(c.root.join(".kinein/build"));
    let (linhas, _) = c.build();
    let configure = linhas
        .iter()
        .find(|l| l.starts_with("cmake-falso -S"))
        .expect("configure");
    assert!(
        configure.ends_with(&format!("-DPICO_SDK_PATH={}", sdk.display())),
        "{configure}"
    );

    // O configure automatico (cmake.configure) leva o mesmo -D.
    let started = c.rpc("cmake.configure", json!({}));
    assert!(started.error.is_none(), "{:?}", started.error);
    let prazo = std::time::Instant::now() + Duration::from_secs(15);
    let mut viu = false;
    while std::time::Instant::now() < prazo {
        if let Ok(e) = c.events.recv_timeout(Duration::from_millis(50)) {
            let p = e.params.clone().unwrap_or_default();
            if e.method == "event.job.output"
                && p["line"]
                    .as_str()
                    .is_some_and(|l| l.contains(&format!("-DPICO_SDK_PATH={}", sdk.display())))
            {
                viu = true;
            }
            if e.method == "event.cmake.finished" {
                break;
            }
        }
    }
    assert!(viu, "o cmake.configure nao levou o PICO_SDK_PATH");
}
