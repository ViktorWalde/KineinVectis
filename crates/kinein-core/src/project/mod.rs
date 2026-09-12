//! O MODELO do projeto embarcado — pilar 0 do `roadmaps/42` (2026-09-12).
//!
//! O `workspace` diz que build system a raiz tem. Este dominio diz o que o
//! projeto E': o framework (com o arquivo que prova), os SDKs e toolchains que
//! ele exige (e se estao aqui), os artefatos do ultimo build, e o ALVO
//! deduzido de tudo isso — motor de gravacao, monitor, adaptador de debug.
//! Cada deducao carrega a evidencia; o que nao se decide vira `hint`, nunca
//! silencio (`roadmaps/35` §5.1 item 4).
//!
//! # O que este modulo NAO e'
//!
//! Nao e' indice semantico (isso continua no clangd/rust-analyzer) e nao roda
//! processo nenhum: e' leitura de arquivo e de ambiente. O que exige processo
//! (`rustup target list`, `idf.py`) e' do pilar 1 (`setup`) e do ciclo (P2).

pub mod artifacts;
pub mod detect;
pub mod sdk;

use std::path::{Path, PathBuf};

use kinein_protocol::{Framework, FrameworkInfo, ProjectModel, TargetModel};

use crate::tools::ToolDetector;

/// Computa o modelo lendo o ambiente REAL do processo.
#[must_use]
pub fn model(root: &Path, kit_chip: Option<&str>) -> ProjectModel {
    let detector = ToolDetector::from_environment();
    let var = |nome: &str| std::env::var(nome).ok();
    let binario = |nome: &str| detector.find_in_path(nome);
    let ambiente = sdk::Ambiente {
        var: &var,
        binario: &binario,
        home: std::env::var_os("HOME").map(PathBuf::from),
    };
    model_in(root, kit_chip, &ambiente)
}

/// Computa o modelo com o ambiente dado (o teste injeta o seu).
#[must_use]
pub fn model_in(root: &Path, kit_chip: Option<&str>, ambiente: &sdk::Ambiente<'_>) -> ProjectModel {
    let frameworks = detect::frameworks(root);
    let sdks = sdk::requisitos(&frameworks, ambiente);
    let artifacts = artifacts::artifacts(root);
    let target = deduzir_alvo(&frameworks, kit_chip);
    let mut hints = Vec::new();
    for req in sdks.iter().filter(|r| !r.found) {
        hints.push(format!(
            "falta {}: {}",
            req.label,
            req.hint.clone().unwrap_or_default()
        ));
    }
    if frameworks.len() > 1 {
        hints.push(format!(
            "mais de um framework reconhecido ({}); o primeiro manda no alvo",
            frameworks
                .iter()
                .map(|f| nome(f.framework))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !frameworks.is_empty() && artifacts.elf.is_empty() && artifacts.uf2.is_empty() {
        hints.push(
            "nenhum artefato de build encontrado: compile para o modelo ver ELF, imagens e mapa"
                .to_owned(),
        );
    }
    ProjectModel {
        root: root.display().to_string(),
        embedded: !frameworks.is_empty(),
        frameworks,
        sdks,
        artifacts,
        target,
        hints,
    }
}

/// O nome que aparece na tela e nos hints.
#[must_use]
pub const fn nome(framework: Framework) -> &'static str {
    match framework {
        Framework::EspIdf => "ESP-IDF",
        Framework::Zephyr => "Zephyr",
        Framework::PicoSdk => "pico-sdk",
        Framework::PlatformIo => "PlatformIO",
        Framework::Stm32Cube => "STM32Cube",
        Framework::CargoEmbedded => "Rust embarcado",
        Framework::MicroPython => "MicroPython",
        Framework::Yocto => "Yocto",
        Framework::Buildroot => "Buildroot",
    }
}

/// O alvo: chip (kit > framework), familia, triple, e os motores sugeridos —
/// cada um com a linha de evidencia.
fn deduzir_alvo(frameworks: &[FrameworkInfo], kit_chip: Option<&str>) -> TargetModel {
    let mut alvo = TargetModel::default();
    let principal = frameworks.first();

    // 1. O chip: o kit e' a palavra do usuario; senao o que o framework disse.
    if let Some(chip) = kit_chip.map(str::trim).filter(|c| !c.is_empty()) {
        alvo.chip = Some(chip.to_owned());
        alvo.evidence
            .push("chip: kit do projeto (toolchain.setKit)".to_owned());
    } else if let Some(info) = principal {
        if let Some((chip, de)) = chip_do_framework(info) {
            alvo.chip = Some(chip);
            alvo.evidence.push(format!("chip: {de}"));
        }
    }

    // 2. O triple, quando o build system declara um.
    if let Some(info) = principal {
        if info.framework == Framework::CargoEmbedded {
            if let Some(d) = info.detail.as_deref().filter(|d| d.contains("-none-")) {
                alvo.triple = Some(d.to_owned());
                alvo.evidence.push("triple: .cargo/config.toml".to_owned());
            }
        }
    }

    // 3. A familia, do chip ou do framework.
    alvo.family = familia(
        alvo.chip.as_deref(),
        alvo.triple.as_deref(),
        principal.map(|f| f.framework),
    );
    if let Some(f) = &alvo.family {
        alvo.evidence.push(format!("familia: {f}"));
    }

    // 4. Os motores, pela familia e pelo framework.
    let (gravar, monitor, debug, porque) = motores(
        alvo.family.as_deref(),
        alvo.chip.as_deref(),
        principal.map(|f| f.framework),
    );
    alvo.flash_engine = gravar.map(str::to_owned);
    alvo.monitor = monitor.map(str::to_owned);
    alvo.debug_adapter = debug.map(str::to_owned);
    if let Some(p) = porque {
        alvo.evidence.push(p);
    }
    alvo
}

fn chip_do_framework(info: &FrameworkInfo) -> Option<(String, String)> {
    let detalhe = info.detail.as_deref()?;
    match info.framework {
        Framework::EspIdf => detalhe.strip_prefix("IDF_TARGET ").map(|c| {
            (
                c.to_owned(),
                format!("{} (CONFIG_IDF_TARGET)", info.evidence),
            )
        }),
        Framework::Stm32Cube => Some((detalhe.to_owned(), format!("{} (CubeMX)", info.evidence))),
        Framework::PicoSdk => {
            let placa = detalhe
                .strip_prefix("PICO_BOARD ")
                .or_else(|| detalhe.strip_prefix("PICO_PLATFORM "))?;
            let chip = if placa.contains("2350") || placa.contains("pico2") {
                "rp2350"
            } else {
                "rp2040"
            };
            Some((chip.to_owned(), format!("{} ({detalhe})", info.evidence)))
        }
        Framework::PlatformIo => {
            // "esp32dev (espressif32, esp32dev)": a plataforma diz a familia,
            // nao o chip exato — e o modelo nao inventa o exato.
            let plataforma = detalhe.split('(').nth(1)?.split(',').next()?.trim();
            let chip = match plataforma {
                "espressif32" => "esp32",
                "ststm32" => "stm32",
                "raspberrypi" => "rp2040",
                "nordicnrf52" => "nrf52",
                _ => return None,
            };
            Some((
                chip.to_owned(),
                format!(
                    "{} (platform {plataforma}; familia, nao o chip exato)",
                    info.evidence
                ),
            ))
        }
        _ => None,
    }
}

fn familia(
    chip: Option<&str>,
    triple: Option<&str>,
    framework: Option<Framework>,
) -> Option<String> {
    let baixo = chip.map(str::to_ascii_lowercase).unwrap_or_default();
    let por_chip = if baixo.starts_with("esp32") || baixo.starts_with("esp8266") {
        Some("espressif")
    } else if baixo.starts_with("stm32") {
        Some("stm32")
    } else if baixo.starts_with("rp2") || baixo.starts_with("pico") {
        Some("rp2040")
    } else if baixo.starts_with("nrf") {
        Some("nrf")
    } else {
        None
    };
    if let Some(f) = por_chip {
        return Some(f.to_owned());
    }
    if let Some(t) = triple {
        if t.starts_with("thumbv") {
            return Some("cortex-m".to_owned());
        }
        if t.starts_with("riscv32") {
            return Some("riscv".to_owned());
        }
        if t.starts_with("xtensa") {
            return Some("espressif".to_owned());
        }
    }
    match framework {
        Some(Framework::Yocto | Framework::Buildroot) => Some("linux".to_owned()),
        Some(Framework::Zephyr) => Some("zephyr".to_owned()),
        _ => None,
    }
}

/// (gravar, monitor, debug, porque). Tudo SUGESTAO: quem grava confirma.
fn motores(
    familia: Option<&str>,
    chip: Option<&str>,
    framework: Option<Framework>,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<String>,
) {
    // Frameworks com motor proprio mandam antes da familia.
    match framework {
        Some(Framework::Zephyr) => {
            return (
                Some("west"),
                Some("serial"),
                Some("gdb"),
                Some("motores: west flash/debug (runner do board)".to_owned()),
            );
        }
        Some(Framework::PlatformIo) => {
            return (
                Some("platformio"),
                Some("serial"),
                Some("gdb"),
                Some("motores: pio run -t upload / pio device monitor / pio debug".to_owned()),
            );
        }
        Some(Framework::MicroPython) => {
            return (Some("esptool"), Some("serial"), None, Some("motores: firmware pelo esptool/UF2; REPL pelo mpremote; sem depurador (MicroPython nao tem DAP)".to_owned()));
        }
        _ => {}
    }
    let chip_baixo = chip.map(str::to_ascii_lowercase).unwrap_or_default();
    match familia {
        Some("espressif") => {
            let usb_jtag = [
                "esp32c3", "esp32c6", "esp32h2", "esp32s3", "esp32c5", "esp32p4", "esp32c61",
            ]
            .iter()
            .any(|c| chip_baixo.starts_with(c));
            if usb_jtag {
                (
                    Some("esptool"),
                    Some("espflash"),
                    Some("probe-rs"),
                    Some(format!(
                        "motores: esptool pela serial; probe-rs pelo USB-JTAG embutido do {chip_baixo}"
                    )),
                )
            } else {
                (Some("esptool"), Some("espflash"), None, Some("motores: esptool pela serial; sem JTAG embutido — depurar exige ESP-Prog + openocd-esp32 + esp-gdb".to_owned()))
            }
        }
        Some("stm32") => (
            Some("probe-rs"),
            Some("serial"),
            Some("probe-rs"),
            Some(
                "motores: probe-rs por SWD (ST-Link); dfu-util e' a alternativa sem sonda"
                    .to_owned(),
            ),
        ),
        Some("rp2040") => (
            Some("picotool"),
            Some("serial"),
            Some("probe-rs"),
            Some("motores: picotool/UF2 por BOOTSEL; probe-rs pela Debug Probe".to_owned()),
        ),
        Some("nrf" | "cortex-m" | "riscv") => (
            Some("probe-rs"),
            Some("rtt"),
            Some("probe-rs"),
            Some("motores: probe-rs (download/run com RTT)".to_owned()),
        ),
        Some("linux") => (
            None,
            Some("serial"),
            Some("gdb"),
            Some(
                "Linux embarcado: deploy e gdbserver por SSH (pilar 6); console pela UART"
                    .to_owned(),
            ),
        ),
        _ => (None, None, None, None),
    }
}
