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

use kinein_protocol::{Framework, JsonRpcRequest};
use serde_json::json;

use super::core_with_empty_search_path;

use crate::project::sdk::Ambiente;
use crate::project::{artifacts, detect, esp, model_in, sdk};

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

/// Um `flasher_args.json` com a forma EXATA do template do ESP-IDF
/// (`components/esptool_py/flasher_args.json.in` + `project_include.cmake`).
const FLASHER_ARGS: &str = r#"{
    "write_flash_args" : [ "--flash-mode", "dio", "--flash-size", "4MB", "--flash-freq", "40m" ],
    "flash_settings" : { "flash_mode": "dio", "flash_size": "4MB", "flash_freq": "40m" },
    "flash_files" : {
        "0x1000" : "bootloader/bootloader.bin",
        "0x8000" : "partition_table/partition-table.bin",
        "0x10000" : "hello_world.bin"
    },
    "bootloader" : { "offset" : "0x1000", "file" : "bootloader/bootloader.bin", "encrypted" : "false" },
    "app" : { "offset" : "0x10000", "file" : "hello_world.bin", "encrypted" : "true" },
    "partition-table" : { "offset" : "0x8000", "file" : "partition_table/partition-table.bin", "encrypted" : "false" },
    "extra_esptool_args" : { "after" : "hard-reset", "before" : "default-reset", "stub" : true, "chip" : "esp32" }
}"#;

#[test]
fn the_flash_recipe_is_read_with_names_offsets_encryption_and_absolute_paths() {
    let receita = esp::flash_recipe(FLASHER_ARGS, Path::new("/proj/build")).unwrap();
    assert_eq!(receita.chip.as_deref(), Some("esp32"));
    assert_eq!(receita.flash_size.as_deref(), Some("4MB"));
    assert_eq!(receita.flash_size_bytes, Some(4 * 1024 * 1024));
    assert_eq!(
        (receita.flash_mode.as_deref(), receita.flash_freq.as_deref()),
        (Some("dio"), Some("40m"))
    );
    assert_eq!(
        (receita.before.as_deref(), receita.after.as_deref()),
        (Some("default-reset"), Some("hard-reset"))
    );
    assert!(receita.stub);
    let offsets: Vec<u32> = receita.files.iter().map(|f| f.offset).collect();
    assert_eq!(
        offsets,
        vec![0x1000, 0x8000, 0x10000],
        "por offset crescente"
    );
    let app = receita
        .files
        .iter()
        .find(|f| f.name.as_deref() == Some("app"))
        .unwrap();
    assert_eq!(app.file, "/proj/build/hello_world.bin");
    assert!(app.encrypted);
    assert!(
        !receita
            .files
            .iter()
            .find(|f| f.offset == 0x1000)
            .unwrap()
            .encrypted
    );
    // Um arquivo so' em flash_files (sem entrada nomeada) entra sem nome.
    let so_plano = r#"{"flash_files": {"0x20000": "extra.bin"}, "extra_esptool_args": {"stub": false, "chip": "esp32c3"}}"#;
    let r = esp::flash_recipe(so_plano, Path::new("/b")).unwrap();
    assert_eq!(r.files.len(), 1);
    assert_eq!(
        (r.files[0].name.as_deref(), r.files[0].file.as_str()),
        (None, "/b/extra.bin")
    );
    assert!(!r.stub);
    assert!(esp::flash_recipe("nao e json", Path::new("/b")).is_none());
}

#[test]
fn the_partition_table_resolves_blank_offsets_like_gen_esp32part() {
    // A tabela padrao do ESP-IDF (offsets explicitos) e uma com offsets em
    // branco: a primeira cai em tabela+0x1000, a `app` alinha a 0x10000.
    let explicita = std::fs::read_to_string(fixtures().join("esp-idf/partitions.csv")).unwrap();
    let t = esp::partition_table(&explicita, esp::TABLE_OFFSET_PADRAO);
    assert_eq!(t.entries.len(), 2);
    assert_eq!(
        (
            t.entries[0].name.as_str(),
            t.entries[0].offset,
            t.entries[0].size
        ),
        ("nvs", 0x9000, 0x6000)
    );
    assert_eq!(
        (
            t.entries[1].kind.as_str(),
            t.entries[1].offset,
            t.entries[1].size
        ),
        ("app", 0x10000, 1024 * 1024)
    );
    assert_eq!(t.end, 0x10000 + 1024 * 1024);
    assert!(t.unreadable.is_empty());

    let em_branco = "# Name, Type, SubType, Offset, Size, Flags\n\
                     nvs,      data, nvs,     ,        24K,\n\
                     factory,  app,  factory, ,        1M, encrypted:readonly\n\
                     phy_init, data, phy,     ,        4K,\n\
                     storage,  data, spiffs,  ,        0x100000,\n\
                     linha quebrada sem campos\n";
    let t = esp::partition_table(em_branco, esp::TABLE_OFFSET_PADRAO);
    let por_nome = |n: &str| t.entries.iter().find(|p| p.name == n).unwrap();
    assert_eq!(
        por_nome("nvs").offset,
        0x9000,
        "a primeira vem logo apos a tabela"
    );
    // nvs termina em 0xF000; um alinhamento de 4 KB deixaria a app ali. A app
    // alinha a 64 KB: 0x10000. E' a regra que o gen_esp32part impoe.
    assert_eq!(por_nome("factory").offset, 0x10000, "app alinha a 64 KB");
    assert_eq!(
        por_nome("factory").flags.as_deref(),
        Some("encrypted:readonly")
    );
    assert_eq!(por_nome("phy_init").offset, 0x10000 + 1024 * 1024);
    assert_eq!(por_nome("storage").offset, 0x10000 + 1024 * 1024 + 0x1000);
    assert_eq!(t.end, 0x10000 + 1024 * 1024 + 0x1000 + 1024 * 1024);
    assert_eq!(
        t.unreadable,
        vec!["linha quebrada sem campos".to_owned()],
        "o que nao se le e' dito, nao sumido"
    );

    // Tabela em 0x9000 (CONFIG_PARTITION_TABLE_OFFSET mudado): tudo se desloca.
    let t = esp::partition_table(em_branco, 0x9000);
    assert_eq!(t.entries[0].offset, 0xA000);
}

#[test]
fn the_model_reads_the_recipe_and_the_partitions_it_used_to_only_locate() {
    let raiz = temp_dir("esp-lido");
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        raiz.join("CMakeLists.txt"),
    )
    .unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/partitions.csv"),
        raiz.join("partitions.csv"),
    )
    .unwrap();
    std::fs::create_dir_all(raiz.join("build")).unwrap();
    std::fs::write(raiz.join("build/flasher_args.json"), FLASHER_ARGS).unwrap();
    let amb = ambiente_vazio(&nada_var, &nada_bin);
    let m = model_in(&raiz, None, &amb);
    let receita = m.artifacts.flash_recipe.as_ref().unwrap();
    assert_eq!(receita.chip.as_deref(), Some("esp32"));
    assert!(
        receita.files[2]
            .file
            .starts_with(raiz.join("build").to_str().unwrap())
    );
    let tabela = m.artifacts.partitions.as_ref().unwrap();
    assert_eq!(tabela.entries[1].name, "factory");
}

/// `build.size` num projeto ESP-IDF: a regiao de flash e' a particao `app`
/// que a receita aponta, e o usado e' a IMAGEM. Pelo despacho REAL, porque e'
/// no handler que o modelo e o `size` se encontram.
#[test]
fn build_size_reports_the_esp_idf_app_partition_as_the_flash_region() {
    let raiz = temp_dir("size-esp").canonicalize().unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/CMakeLists.txt"),
        raiz.join("CMakeLists.txt"),
    )
    .unwrap();
    std::fs::copy(
        fixtures().join("esp-idf/partitions.csv"),
        raiz.join("partitions.csv"),
    )
    .unwrap();
    std::fs::create_dir_all(raiz.join("build")).unwrap();
    std::fs::write(raiz.join("build/flasher_args.json"), FLASHER_ARGS).unwrap();
    // A imagem do app tem 123.456 bytes; o "ELF" e' so' um arquivo para o
    // `size` recusar — o que interessa aqui e' a regiao, nao as secoes.
    std::fs::write(raiz.join("build/hello_world.bin"), vec![0u8; 123_456]).unwrap();
    std::fs::write(raiz.join("build/hello_world.elf"), b"nao e um elf").unwrap();

    let mut core = core_with_empty_search_path("size-esp");
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let outcome = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "build.size",
        Some(json!({ "program": raiz.join("build/hello_world.elf").to_str().unwrap() })),
    ));
    let resposta = outcome.response();
    let resultado = resposta
        .result
        .clone()
        .unwrap_or_else(|| panic!("build.size falhou: {:?}", resposta.error));
    let regioes = resultado["regions"].as_array().unwrap();
    let app = regioes
        .iter()
        .find(|r| {
            r["name"]
                .as_str()
                .unwrap()
                .starts_with("factory (particao app")
        })
        .unwrap_or_else(|| panic!("sem regiao da particao: {regioes:?}"));
    assert_eq!(app["used"], 123_456);
    assert_eq!(app["size"], 1024 * 1024);
}
