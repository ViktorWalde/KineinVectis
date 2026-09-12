//! O que cada framework EXIGE nesta maquina, e se esta' aqui.
//!
//! Pilar 0 do `roadmaps/42`: o modelo nao instala nada — diz o que falta e o
//! passo oficial, e o dominio `setup` (pilar 1) e' quem mostra o comando por
//! distro. Tudo aqui e' leitura: variavel de ambiente, pasta padrao, binario no
//! `PATH`. As variaveis e as pastas padrao vem da documentacao de cada projeto
//! (ESP-IDF "Get Started": `IDF_PATH`, `~/esp/esp-idf`, `~/.espressif`; Zephyr
//! "Getting Started": `ZEPHYR_BASE`, `~/zephyrproject`, `ZEPHYR_SDK_INSTALL_DIR`;
//! pico-sdk README: `PICO_SDK_PATH`), lidas em 2026-09-11/12.

use std::path::{Path, PathBuf};

use kinein_protocol::{Framework, FrameworkInfo, SdkRequirement};

/// De onde o modelo le o ambiente. Injetavel: o teste controla variaveis e
/// binarios sem tocar no processo (escrever env e' `unsafe`, e esta' proibido).
pub struct Ambiente<'a> {
    /// `std::env::var`, ou o falso do teste.
    pub var: &'a dyn Fn(&str) -> Option<String>,
    /// Onde um binario esta' no `PATH`, ou `None`.
    pub binario: &'a dyn Fn(&str) -> Option<PathBuf>,
    /// `$HOME`.
    pub home: Option<PathBuf>,
}

impl std::fmt::Debug for Ambiente<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ambiente")
            .field("home", &self.home)
            .finish_non_exhaustive()
    }
}

/// As exigencias de todos os frameworks achados, sem repetir.
#[must_use]
pub fn requisitos(frameworks: &[FrameworkInfo], ambiente: &Ambiente<'_>) -> Vec<SdkRequirement> {
    let mut lista: Vec<SdkRequirement> = Vec::new();
    for info in frameworks {
        for req in do_framework(info, ambiente) {
            if !lista.iter().any(|r| r.id == req.id) {
                lista.push(req);
            }
        }
    }
    lista
}

fn do_framework(info: &FrameworkInfo, amb: &Ambiente<'_>) -> Vec<SdkRequirement> {
    match info.framework {
        Framework::EspIdf => esp_idf(info, amb),
        Framework::Zephyr => zephyr(amb),
        Framework::PicoSdk => pico_sdk(amb),
        Framework::PlatformIo => vec![binario(
            amb,
            "pio",
            "PlatformIO Core",
            "pipx install platformio",
        )],
        Framework::Stm32Cube => vec![arm_gcc(amb), probe_rs(amb)],
        Framework::CargoEmbedded => cargo_embedded(info, amb),
        Framework::MicroPython => vec![
            binario(amb, "mpremote", "mpremote", "pipx install mpremote"),
            binario(
                amb,
                "esptool",
                "esptool (firmware em ESP32)",
                "pipx install esptool",
            ),
        ],
        Framework::Yocto => vec![binario(
            amb,
            "bitbake",
            "bitbake (entra no PATH com `source oe-init-build-env`)",
            "source poky/oe-init-build-env <build-dir>",
        )],
        Framework::Buildroot => vec![binario(amb, "make", "make", "dnf install make")],
    }
}

/// ESP-IDF: a arvore do IDF, a toolchain do ALVO (xtensa ou riscv32) e o esptool.
fn esp_idf(info: &FrameworkInfo, amb: &Ambiente<'_>) -> Vec<SdkRequirement> {
    let alvo = info
        .detail
        .as_deref()
        .and_then(|d| d.strip_prefix("IDF_TARGET "))
        .unwrap_or("esp32");
    let toolchain = if alvo.starts_with("esp32c") || alvo.starts_with("esp32h") || alvo == "esp32p4"
    {
        "riscv32-esp-elf-gcc"
    } else {
        "xtensa-esp-elf-gcc"
    };
    vec![
        pasta(
            amb,
            "esp-idf",
            "ESP-IDF",
            Some("IDF_PATH"),
            &["esp/esp-idf"],
            "git clone -b v6.1 --recursive https://github.com/espressif/esp-idf.git ~/esp/esp-idf && \
             ~/esp/esp-idf/install.sh; depois `. ~/esp/esp-idf/export.sh` no shell que compila",
        ),
        binario_ou_espressif(
            amb,
            toolchain,
            &format!("{toolchain} (toolchain do {alvo})"),
            "vem com o install.sh do ESP-IDF, em ~/.espressif/tools; o export.sh o poe no PATH",
        ),
        binario(
            amb,
            "esptool",
            "esptool",
            "pipx install esptool (o ESP-IDF traz o seu no ambiente do export.sh)",
        ),
    ]
}

fn zephyr(amb: &Ambiente<'_>) -> Vec<SdkRequirement> {
    vec![
        pasta(
            amb,
            "zephyr",
            "Zephyr (ZEPHYR_BASE)",
            Some("ZEPHYR_BASE"),
            &["zephyrproject/zephyr"],
            "pipx install west && west init ~/zephyrproject && cd ~/zephyrproject && west update",
        ),
        pasta(
            amb,
            "zephyr-sdk",
            "Zephyr SDK",
            Some("ZEPHYR_SDK_INSTALL_DIR"),
            &["zephyr-sdk-0.17.0", "zephyr-sdk"],
            "o bundle em github.com/zephyrproject-rtos/sdk-ng/releases, depois ./setup.sh",
        ),
        binario(amb, "west", "west", "pipx install west"),
    ]
}

fn pico_sdk(amb: &Ambiente<'_>) -> Vec<SdkRequirement> {
    vec![
        pasta(
            amb,
            "pico-sdk",
            "Pico SDK (PICO_SDK_PATH)",
            Some("PICO_SDK_PATH"),
            &["pico/pico-sdk", "pico-sdk"],
            "git clone https://github.com/raspberrypi/pico-sdk.git ~/pico/pico-sdk && \
             git -C ~/pico/pico-sdk submodule update --init",
        ),
        arm_gcc(amb),
        binario(
            amb,
            "picotool",
            "picotool",
            "git clone https://github.com/raspberrypi/picotool && cmake/ninja (BSD-3); o Fedora nao empacota",
        ),
    ]
}

/// Rust embarcado: probe-rs, cargo-embed e o alvo — espup para Xtensa, alvo
/// rustup para o resto (medido pelo setup, nao aqui: e' processo).
fn cargo_embedded(info: &FrameworkInfo, amb: &Ambiente<'_>) -> Vec<SdkRequirement> {
    let triple = info.detail.clone().unwrap_or_default();
    let mut lista = vec![
        probe_rs(amb),
        binario(
            amb,
            "cargo-embed",
            "cargo-embed",
            "vem com o probe-rs-tools",
        ),
    ];
    if triple.starts_with("xtensa") {
        lista.push(binario(
            amb,
            "espup",
            "espup (Rust para Xtensa)",
            "cargo install espup && espup install",
        ));
    } else if triple.contains("-none-") {
        lista.push(SdkRequirement {
            id: format!("rustup-target:{triple}"),
            label: format!("alvo rustup {triple}"),
            env: None,
            path: None,
            found: false,
            hint: Some(format!(
                "rustup target add {triple} (nao medido pelo modelo; o setup mede)"
            )),
        });
    }
    lista
}

fn arm_gcc(amb: &Ambiente<'_>) -> SdkRequirement {
    binario(
        amb,
        "arm-none-eabi-gcc",
        "arm-none-eabi-gcc",
        "dnf install arm-none-eabi-gcc-cs arm-none-eabi-newlib (Fedora)",
    )
}

fn probe_rs(amb: &Ambiente<'_>) -> SdkRequirement {
    binario(
        amb,
        "probe-rs",
        "probe-rs",
        "o instalador oficial em probe.rs/docs/getting-started/installation",
    )
}

/// Uma PASTA: a variavel, senao as pastas padrao sob `$HOME`.
fn pasta(
    amb: &Ambiente<'_>,
    id: &str,
    label: &str,
    env: Option<&str>,
    padroes: &[&str],
    hint: &str,
) -> SdkRequirement {
    let da_env = env
        .and_then(|e| (amb.var)(e))
        .map(PathBuf::from)
        .filter(|p| p.is_dir());
    let padrao = || {
        amb.home
            .as_ref()
            .and_then(|home| padroes.iter().map(|p| home.join(p)).find(|p| p.is_dir()))
    };
    let path = da_env.or_else(padrao);
    SdkRequirement {
        id: id.to_owned(),
        label: label.to_owned(),
        env: env.map(str::to_owned),
        found: path.is_some(),
        path: path.map(|p| p.display().to_string()),
        hint: Some(hint.to_owned()),
    }
}

/// Um BINARIO no `PATH`.
fn binario(amb: &Ambiente<'_>, id: &str, label: &str, hint: &str) -> SdkRequirement {
    let path = (amb.binario)(id);
    SdkRequirement {
        id: id.to_owned(),
        label: label.to_owned(),
        env: None,
        found: path.is_some(),
        path: path.map(|p| p.display().to_string()),
        hint: Some(hint.to_owned()),
    }
}

/// As toolchains da Espressif ficam em `~/.espressif/tools/<nome>/<versao>/
/// <nome>/bin` e so' entram no PATH pelo `export.sh`: se nao estao no PATH,
/// procurar la' e' o que evita dizer "falta" ao que esta' instalado.
fn binario_ou_espressif(amb: &Ambiente<'_>, id: &str, label: &str, hint: &str) -> SdkRequirement {
    let mut req = binario(amb, id, label, hint);
    if req.found {
        return req;
    }
    let raiz = (amb.var)("IDF_TOOLS_PATH")
        .map(PathBuf::from)
        .or_else(|| amb.home.as_ref().map(|h| h.join(".espressif")));
    if let Some(achado) = raiz.and_then(|r| procurar_em_espressif(&r.join("tools"), id)) {
        req.found = true;
        req.path = Some(achado.display().to_string());
    }
    req
}

fn procurar_em_espressif(tools: &Path, id: &str) -> Option<PathBuf> {
    let familia = id.strip_suffix("-gcc")?;
    let versoes = std::fs::read_dir(tools.join(familia)).ok()?;
    versoes
        .filter_map(Result::ok)
        .map(|v| v.path().join(familia).join("bin").join(id))
        .find(|p| p.is_file())
}
