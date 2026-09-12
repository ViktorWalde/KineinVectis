//! O MODELO do projeto embarcado (pilar 0 do `roadmaps/42`), contra as
//! fixtures REAIS e minimas de cada framework em `scripts/fixtures/projetos/`.
//!
//! O que se prova: que cada framework e' reconhecido pelo arquivo certo e com
//! o detalhe que o arquivo diz; que a deteccao desce em subpasta e ignora
//! pastas de saida; que o alvo deduzido carrega a evidencia; que o kit VENCE o
//! framework; que os SDKs exigidos saem certos e que "achado" so' vem de
//! variavel, pasta padrao ou binario — nunca de suposicao; e que os artefatos
//! do build sao listados do mais novo ao mais velho, com o `flasher_args.json`
//! e o ELF sem extensao do cargo reconhecidos.

use std::path::{Path, PathBuf};

use kinein_protocol::Framework;

use crate::project::sdk::Ambiente;
use crate::project::{artifacts, detect, model_in, sdk};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/fixtures/projetos")
        .canonicalize()
        .unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-project-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Um ambiente onde NADA esta' instalado e nenhuma variavel existe.
fn ambiente_vazio<'a>(
    var: &'a dyn Fn(&str) -> Option<String>,
    bin: &'a dyn Fn(&str) -> Option<PathBuf>,
) -> Ambiente<'a> {
    Ambiente {
        var,
        binario: bin,
        home: None,
    }
}

fn nada_var(_: &str) -> Option<String> {
    None
}

fn nada_bin(_: &str) -> Option<PathBuf> {
    None
}

#[test]
fn each_fixture_is_recognised_by_its_marker_with_the_detail_the_file_says() {
    let casos: &[(&str, Framework, &str, Option<&str>)] = &[
        (
            "esp-idf",
            Framework::EspIdf,
            "CMakeLists.txt",
            Some("IDF_TARGET esp32c3"),
        ),
        (
            "zephyr",
            Framework::Zephyr,
            "CMakeLists.txt",
            Some("nrf52840dk/nrf52840"),
        ),
        (
            "pico-sdk",
            Framework::PicoSdk,
            "CMakeLists.txt",
            Some("PICO_BOARD pico_w"),
        ),
        (
            "platformio",
            Framework::PlatformIo,
            "platformio.ini",
            Some("esp32dev (espressif32, esp32dev); bluepill (ststm32, bluepill_f103c8)"),
        ),
        (
            "stm32cube",
            Framework::Stm32Cube,
            "fixture.ioc",
            Some("STM32F401CCUx"),
        ),
        (
            "cargo-embarcado",
            Framework::CargoEmbedded,
            ".cargo/config.toml",
            Some("thumbv7em-none-eabihf"),
        ),
        (
            "micropython",
            Framework::MicroPython,
            "main.py",
            Some("importa machine"),
        ),
        (
            "yocto",
            Framework::Yocto,
            "conf/local.conf",
            Some("MACHINE raspberrypi4-64"),
        ),
        (
            "buildroot",
            Framework::Buildroot,
            ".config",
            Some("BR2_ARCH aarch64"),
        ),
    ];
    for (pasta, framework, evidencia, detalhe) in casos {
        let achados = detect::frameworks(&fixtures().join(pasta));
        let achado = achados
            .iter()
            .find(|a| a.framework == *framework)
            .unwrap_or_else(|| panic!("{pasta}: nao reconheceu {framework:?}: {achados:?}"));
        assert_eq!(achado.evidence, *evidencia, "{pasta}");
        assert_eq!(achado.detail.as_deref(), *detalhe, "{pasta}");
    }
}

#[test]
fn detection_descends_into_subfolders_but_not_into_build_output_and_the_shallowest_wins() {
    let raiz = temp_dir("subpasta");
    // O projeto ESP-IDF mora em firmware/; o build/ tem um CMakeLists gerado
    // que TAMBEM cita project.cmake e nao pode contar.
    // O projeto de verdade esta' DOIS niveis abaixo; a copia em build/ esta'
    // a UM. Se a busca entrasse em build/, a copia venceria por ser mais rasa.
    let firmware = raiz.join("firmware").join("app");
    std::fs::create_dir_all(firmware.join("main")).unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        firmware.join("CMakeLists.txt"),
    )
    .unwrap();
    std::fs::create_dir_all(raiz.join("build")).unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        raiz.join("build/CMakeLists.txt"),
    )
    .unwrap();
    // Um segundo CMakeLists do IDF mais fundo: o mais raso e' a evidencia.
    std::fs::create_dir_all(raiz.join("firmware/app/extra")).unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        raiz.join("firmware/app/extra/CMakeLists.txt"),
    )
    .unwrap();

    let achados = detect::frameworks(&raiz);
    assert_eq!(achados.len(), 1, "{achados:?}");
    assert_eq!(achados[0].framework, Framework::EspIdf);
    assert_eq!(achados[0].evidence, "firmware/app/CMakeLists.txt");
    // Sem sdkconfig ao lado, nao ha' detalhe — e nao se inventa "esp32".
    assert_eq!(achados[0].detail, None);

    // Um CMakeLists comum NAO e' embarcado.
    let comum = temp_dir("comum");
    std::fs::write(
        comum.join("CMakeLists.txt"),
        "project(x)\nadd_executable(x x.c)\n",
    )
    .unwrap();
    assert!(detect::frameworks(&comum).is_empty());
    // Um main.py que nao importa hardware NAO e' MicroPython.
    std::fs::write(comum.join("main.py"), "import os\nprint(os.getcwd())\n").unwrap();
    assert!(detect::frameworks(&comum).is_empty());
}

#[test]
fn the_target_carries_its_evidence_and_the_kit_chip_wins_over_the_framework() {
    let amb = ambiente_vazio(&nada_var, &nada_bin);

    // ESP-IDF com IDF_TARGET esp32c3: familia espressif, USB-JTAG -> probe-rs.
    let m = model_in(&fixtures().join("esp-idf"), None, &amb);
    assert!(m.embedded);
    assert_eq!(m.target.chip.as_deref(), Some("esp32c3"));
    assert_eq!(m.target.family.as_deref(), Some("espressif"));
    assert_eq!(m.target.flash_engine.as_deref(), Some("esptool"));
    assert_eq!(m.target.monitor.as_deref(), Some("espflash"));
    assert_eq!(m.target.debug_adapter.as_deref(), Some("probe-rs"));
    assert!(
        m.target
            .evidence
            .iter()
            .any(|e| e.contains("CONFIG_IDF_TARGET")),
        "{:?}",
        m.target.evidence
    );
    assert_eq!(
        m.artifacts
            .partition_table
            .as_deref()
            .map(|p| p.ends_with("partitions.csv")),
        Some(true)
    );

    // O kit diz esp32 (o classico): o chip do kit vence, e o classico NAO tem
    // USB-JTAG — o modelo diz que depurar exige ESP-Prog em vez de sugerir probe-rs.
    let m = model_in(&fixtures().join("esp-idf"), Some("esp32"), &amb);
    assert_eq!(m.target.chip.as_deref(), Some("esp32"));
    assert!(m.target.evidence.iter().any(|e| e.contains("kit")));
    assert_eq!(m.target.debug_adapter, None);
    assert!(m.target.evidence.iter().any(|e| e.contains("ESP-Prog")));

    // STM32 pelo CubeMX: probe-rs; Pico W: rp2040 + picotool; Zephyr: west.
    let m = model_in(&fixtures().join("stm32cube"), None, &amb);
    assert_eq!(
        (
            m.target.chip.as_deref(),
            m.target.family.as_deref(),
            m.target.flash_engine.as_deref()
        ),
        (Some("STM32F401CCUx"), Some("stm32"), Some("probe-rs"))
    );
    assert_eq!(m.artifacts.linker_scripts.len(), 1);
    let m = model_in(&fixtures().join("pico-sdk"), None, &amb);
    assert_eq!(
        (m.target.chip.as_deref(), m.target.flash_engine.as_deref()),
        (Some("rp2040"), Some("picotool"))
    );
    let m = model_in(&fixtures().join("zephyr"), None, &amb);
    assert_eq!(
        (m.target.family.as_deref(), m.target.flash_engine.as_deref()),
        (Some("zephyr"), Some("west"))
    );
    // Rust: o triple vira familia cortex-m e o memory.x entra nos artefatos.
    let m = model_in(&fixtures().join("cargo-embarcado"), None, &amb);
    assert_eq!(m.target.triple.as_deref(), Some("thumbv7em-none-eabihf"));
    assert_eq!(m.target.family.as_deref(), Some("cortex-m"));
    assert!(m.artifacts.memory_x.is_some());
    // Yocto: familia linux, sem motor de gravar (e' o pilar 6).
    let m = model_in(&fixtures().join("yocto"), None, &amb);
    assert_eq!(m.target.family.as_deref(), Some("linux"));
    assert_eq!(m.target.flash_engine, None);
    // Sem framework: nao e' embarcado e nao ha' alvo inventado.
    let m = model_in(&temp_dir("nao-embarcado"), None, &amb);
    assert!(!m.embedded && m.target.family.is_none() && m.hints.is_empty());
}

#[test]
fn sdk_requirements_are_found_only_by_env_default_folder_or_binary() {
    let raiz = temp_dir("sdks");
    let idf = raiz.join("esp-idf-aqui");
    std::fs::create_dir_all(&idf).unwrap();
    let home = raiz.join("home");
    std::fs::create_dir_all(home.join("pico/pico-sdk")).unwrap();
    // A toolchain xtensa "instalada pelo install.sh", fora do PATH.
    let xtensa = home.join(".espressif/tools/xtensa-esp-elf/esp-14.2.0/xtensa-esp-elf/bin");
    std::fs::create_dir_all(&xtensa).unwrap();
    std::fs::write(xtensa.join("xtensa-esp-elf-gcc"), "").unwrap();

    let idf_str = idf.display().to_string();
    let var = move |nome: &str| (nome == "IDF_PATH").then(|| idf_str.clone());
    let bin = |nome: &str| (nome == "esptool").then(|| PathBuf::from("/x/esptool"));
    let amb = Ambiente {
        var: &var,
        binario: &bin,
        home: Some(home),
    };

    // ESP-IDF classico (sem sdkconfig -> esp32 -> xtensa).
    let frameworks = detect::frameworks(&fixtures().join("esp-idf"));
    let mut so_idf = frameworks.clone();
    so_idf[0].detail = None;
    let reqs = sdk::requisitos(&so_idf, &amb);
    let por_id = |id: &str| {
        reqs.iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("{id}: {reqs:?}"))
    };
    assert!(
        por_id("esp-idf").found,
        "IDF_PATH aponta para pasta existente"
    );
    assert_eq!(por_id("esp-idf").env.as_deref(), Some("IDF_PATH"));

    // IDF_PATH apontando para pasta que NAO existe: nao e' "achado". A
    // variavel definida e' promessa; a pasta e' o fato.
    let sumida = raiz.join("nao-existe").display().to_string();
    let var_sumida = move |nome: &str| (nome == "IDF_PATH").then(|| sumida.clone());
    let amb_sumida = Ambiente {
        var: &var_sumida,
        binario: &nada_bin,
        home: None,
    };
    let reqs_sumida = sdk::requisitos(&so_idf, &amb_sumida);
    assert!(
        !reqs_sumida
            .iter()
            .find(|r| r.id == "esp-idf")
            .unwrap()
            .found
    );
    assert!(
        por_id("xtensa-esp-elf-gcc").found,
        "achado em ~/.espressif/tools mesmo fora do PATH"
    );
    assert!(por_id("esptool").found);

    // Com sdkconfig esp32c3 a toolchain exigida e' a riscv — e ela NAO esta'.
    let reqs = sdk::requisitos(&frameworks, &amb);
    assert!(
        reqs.iter()
            .any(|r| r.id == "riscv32-esp-elf-gcc" && !r.found)
    );
    assert!(!reqs.iter().any(|r| r.id == "xtensa-esp-elf-gcc"));

    // pico-sdk pela pasta padrao sob $HOME; arm gcc e picotool faltam.
    let reqs = sdk::requisitos(&detect::frameworks(&fixtures().join("pico-sdk")), &amb);
    assert!(reqs.iter().find(|r| r.id == "pico-sdk").unwrap().found);
    assert!(
        !reqs
            .iter()
            .find(|r| r.id == "arm-none-eabi-gcc")
            .unwrap()
            .found
    );
    assert!(
        reqs.iter().all(|r| r.hint.is_some()),
        "toda exigencia diz como se instala"
    );

    // Sem nada: tudo falta, e os hints viram a lista do modelo.
    let vazio = ambiente_vazio(&nada_var, &nada_bin);
    let m = model_in(&fixtures().join("zephyr"), None, &vazio);
    assert!(m.sdks.iter().all(|r| !r.found));
    assert!(m.hints.iter().filter(|h| h.starts_with("falta ")).count() >= 3);
}

#[test]
fn artifacts_come_from_the_build_dirs_newest_first_and_the_cargo_elf_has_no_extension() {
    let raiz = temp_dir("artefatos");
    let build = raiz.join("build");
    std::fs::create_dir_all(build.join("partition_table")).unwrap();
    std::fs::write(build.join("velho.elf"), "x").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(build.join("novo.elf"), "x").unwrap();
    std::fs::write(build.join("app.bin"), "x").unwrap();
    std::fs::write(build.join("app.map"), "x").unwrap();
    std::fs::write(build.join("flasher_args.json"), "{}").unwrap();
    std::fs::write(build.join("partition_table/partition-table.bin"), "x").unwrap();
    // cargo: target/<triple>/debug/<bin> sem extensao, com magico ELF; e um
    // arquivo sem extensao que NAO e' ELF.
    let cargo = raiz.join("target/thumbv7em-none-eabihf/debug");
    std::fs::create_dir_all(cargo.join("deps")).unwrap();
    std::fs::write(cargo.join("firmware"), [0x7f, b'E', b'L', b'F', 1, 1, 1, 0]).unwrap();
    std::fs::write(cargo.join("firmware.d"), "texto").unwrap();
    std::fs::write(cargo.join("notas"), "isto nao e um elf").unwrap();
    std::fs::write(cargo.join("deps/lixo.elf"), "x").unwrap();
    // target/debug e' o host: nao entra.
    std::fs::create_dir_all(raiz.join("target/debug")).unwrap();
    std::fs::write(raiz.join("target/debug/host.elf"), "x").unwrap();

    let art = artifacts::artifacts(&raiz);
    assert!(
        art.elf[0].ends_with("firmware") || art.elf[0].ends_with("novo.elf"),
        "{:?}",
        art.elf
    );
    assert!(art.elf.iter().any(|e| e.ends_with("novo.elf")));
    assert!(art.elf.iter().any(|e| e.ends_with("/firmware")));
    assert!(
        !art.elf
            .iter()
            .any(|e| e.ends_with("notas") || e.contains("/deps/") || e.contains("target/debug"))
    );
    let i_novo = art
        .elf
        .iter()
        .position(|e| e.ends_with("novo.elf"))
        .unwrap();
    let i_velho = art
        .elf
        .iter()
        .position(|e| e.ends_with("velho.elf"))
        .unwrap();
    assert!(i_novo < i_velho, "o mais novo primeiro: {:?}", art.elf);
    assert_eq!(art.bin.len(), 1);
    assert_eq!(art.map.len(), 1);
    assert!(
        art.flasher_args
            .as_deref()
            .unwrap()
            .ends_with("flasher_args.json")
    );
    assert!(
        art.partition_table
            .as_deref()
            .unwrap()
            .ends_with("partition-table.bin")
    );
}
