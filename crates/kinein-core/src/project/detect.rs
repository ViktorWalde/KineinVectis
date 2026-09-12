//! Reconhece o FRAMEWORK de um projeto embarcado pelo arquivo que o prova.
//!
//! Pilar 0 do `roadmaps/42` (2026-09-12). O `workspace/detect.rs` olha
//! marcadores de build system SO' NA RAIZ; aqui a busca desce ate'
//! [`PROFUNDIDADE`] niveis, ignora pastas de saida, e LE o conteudo dos
//! marcadores que so' se decidem por dentro — um `CMakeLists.txt` e' ESP-IDF
//! se inclui o `project.cmake`, e' Zephyr se faz `find_package(Zephyr)`, e'
//! pico-sdk se chama `pico_sdk_init()`; um `main.py` e' `MicroPython` se importa
//! `machine`. Cada achado devolve o caminho que o provou e o detalhe que o
//! arquivo disse (alvo do IDF, placa do Pico, dispositivo do Cube, triple do
//! cargo, MACHINE do Yocto) — nunca um palpite sem fonte.

use std::path::{Path, PathBuf};

use kinein_protocol::{Framework, FrameworkInfo};

/// Ate' onde a busca desce. Tres niveis cobrem `firmware/app/CMakeLists.txt`
/// e `projeto/build-dir/conf/local.conf`; mais que isso e' varrer o mundo.
pub const PROFUNDIDADE: usize = 3;

/// Pastas de SAIDA e de terceiros: o que ha' dentro nao e' o projeto.
const IGNORADAS: &[&str] = &[
    ".git",
    ".kinein",
    "target",
    "build",
    ".pio",
    "node_modules",
    "managed_components",
    ".venv",
    "venv",
    "__pycache__",
    "cmake-build-debug",
    "cmake-build-release",
];

/// Tamanho maximo de arquivo que vale a pena ler para decidir (256 KiB).
const LEITURA_MAXIMA: u64 = 256 * 1024;

/// Os frameworks reconhecidos sob `root`, o mais especifico primeiro, um por
/// framework (o mais raso vence).
#[must_use]
pub fn frameworks(root: &Path) -> Vec<FrameworkInfo> {
    let mut achados: Vec<((u8, usize), FrameworkInfo)> = Vec::new();
    let mut pilha: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];
    while let Some((dir, nivel)) = pilha.pop() {
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut arquivos: Vec<PathBuf> = Vec::new();
        for entrada in entradas.filter_map(Result::ok) {
            let caminho = entrada.path();
            let nome = entrada.file_name().to_string_lossy().into_owned();
            if caminho.is_dir() {
                if nivel < PROFUNDIDADE && !IGNORADAS.contains(&nome.as_str()) {
                    pilha.push((caminho, nivel + 1));
                }
            } else {
                arquivos.push(caminho);
            }
        }
        for caminho in &arquivos {
            for achado in reconhecer(root, caminho, &dir) {
                // Entre dois achados do MESMO framework vence o mais forte
                // (o `.cargo/config.toml` com o triple vale mais que um
                // `memory.x` solto) e, empatando, o mais raso.
                let peso = (forca(&achado), nivel);
                let ja = achados
                    .iter()
                    .position(|(_, a)| a.framework == achado.framework);
                match ja {
                    Some(i) if achados[i].0 <= peso => {}
                    Some(i) => achados[i] = (peso, achado),
                    None => achados.push((peso, achado)),
                }
            }
        }
    }
    achados.sort_by_key(|(peso, a)| (ordem(a.framework), *peso));
    achados.into_iter().map(|(_, a)| a).collect()
}

/// 0 = o marcador que DECIDE (tem o detalhe); 1 = um sinal auxiliar.
fn forca(achado: &FrameworkInfo) -> u8 {
    match achado.framework {
        Framework::CargoEmbedded if achado.evidence.ends_with("config.toml") => 0,
        Framework::CargoEmbedded => 1,
        Framework::Zephyr if achado.evidence.ends_with("west.yml") => 1,
        Framework::Yocto if achado.evidence.ends_with("layer.conf") => 1,
        _ => 0,
    }
}

/// O mais ESPECIFICO primeiro: um projeto ESP-IDF tambem e' `CMake`, e um
/// `PlatformIO` pode conter um `.ioc` — quem manda e' o framework de cima.
const fn ordem(framework: Framework) -> u8 {
    match framework {
        Framework::EspIdf => 0,
        Framework::Zephyr => 1,
        Framework::PicoSdk => 2,
        Framework::PlatformIo => 3,
        Framework::Stm32Cube => 4,
        Framework::CargoEmbedded => 5,
        Framework::MicroPython => 6,
        Framework::Yocto => 7,
        Framework::Buildroot => 8,
    }
}

fn relativo(root: &Path, caminho: &Path) -> String {
    caminho
        .strip_prefix(root)
        .unwrap_or(caminho)
        .to_string_lossy()
        .into_owned()
}

fn ler(caminho: &Path) -> Option<String> {
    let meta = std::fs::metadata(caminho).ok()?;
    (meta.len() <= LEITURA_MAXIMA)
        .then(|| std::fs::read_to_string(caminho).ok())
        .flatten()
}

/// O que UM arquivo prova. Pode provar mais de um framework (raro) — por isso
/// devolve lista.
fn reconhecer(root: &Path, caminho: &Path, dir: &Path) -> Vec<FrameworkInfo> {
    let nome = caminho
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let evidencia = relativo(root, caminho);
    let mut achados = Vec::new();
    let info = |framework, detail: Option<String>| FrameworkInfo {
        framework,
        evidence: evidencia.clone(),
        detail,
    };

    match nome.as_str() {
        "CMakeLists.txt" => {
            if let Some(texto) = ler(caminho) {
                for (framework, detalhe) in frameworks_do_cmake(&texto, dir) {
                    achados.push(info(framework, detalhe));
                }
            }
        }
        "west.yml" => achados.push(info(Framework::Zephyr, Some("manifesto west".to_owned()))),
        "platformio.ini" => {
            let detalhe = ler(caminho).map(|t| ambientes_do_platformio(&t));
            achados.push(info(
                Framework::PlatformIo,
                detalhe.filter(|d| !d.is_empty()),
            ));
        }
        "config.toml" if dir.file_name().is_some_and(|d| d == ".cargo") => {
            if let Some(texto) = ler(caminho) {
                if let Some(triple) = alvo_do_cargo(&texto) {
                    achados.push(info(Framework::CargoEmbedded, Some(triple)));
                }
            }
        }
        "memory.x" | "Embed.toml" => {
            if dir.join("Cargo.toml").is_file() {
                achados.push(info(Framework::CargoEmbedded, Some(nome.clone())));
            }
        }
        "boot.py" | "main.py" => {
            if let Some(texto) = ler(caminho) {
                if let Some(modulo) = importa_hardware(&texto) {
                    achados.push(info(
                        Framework::MicroPython,
                        Some(format!("importa {modulo}")),
                    ));
                }
            }
        }
        "local.conf" if dir.file_name().is_some_and(|d| d == "conf") => {
            let detalhe = ler(caminho)
                .and_then(|t| valor_de_atribuicao(&t, "MACHINE"))
                .map(|m| format!("MACHINE {m}"));
            achados.push(info(Framework::Yocto, detalhe));
        }
        "layer.conf" if dir.file_name().is_some_and(|d| d == "conf") => {
            achados.push(info(
                Framework::Yocto,
                Some("camada (layer.conf)".to_owned()),
            ));
        }
        ".config" => {
            if let Some(texto) = ler(caminho) {
                if texto.contains("BR2_") {
                    let detalhe = valor_de_atribuicao(&texto, "BR2_ARCH")
                        .map(|a| format!("BR2_ARCH {a}"))
                        .or_else(|| {
                            valor_de_atribuicao(&texto, "BR2_DEFCONFIG")
                                .map(|d| format!("BR2_DEFCONFIG {d}"))
                        });
                    achados.push(info(Framework::Buildroot, detalhe));
                }
            }
        }
        "external.desc" => {
            let detalhe = ler(caminho).and_then(|t| valor_de_atribuicao(&t, "name"));
            achados.push(info(
                Framework::Buildroot,
                detalhe.map(|n| format!("external {n}")),
            ));
        }
        _ if caminho
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("ioc")) =>
        {
            let detalhe = ler(caminho).and_then(|t| {
                valor_de_atribuicao(&t, "ProjectManager.DeviceId")
                    .or_else(|| valor_de_atribuicao(&t, "Mcu.Name"))
            });
            achados.push(info(Framework::Stm32Cube, detalhe));
        }
        _ => {}
    }
    achados
}

/// O que um `CMakeLists.txt` prova: ESP-IDF (inclui o `project.cmake` do
/// `IDF_PATH`), Zephyr (`find_package(Zephyr)`), pico-sdk (`pico_sdk_init`).
fn frameworks_do_cmake(texto: &str, dir: &Path) -> Vec<(Framework, Option<String>)> {
    let mut achados = Vec::new();
    if texto.contains("project.cmake") && texto.contains("IDF_PATH") {
        achados.push((Framework::EspIdf, alvo_do_idf(dir)));
    }
    if texto.contains("find_package(Zephyr") {
        achados.push((Framework::Zephyr, valor_de_set(texto, "BOARD")));
    }
    if texto.contains("pico_sdk_init") || texto.contains("pico_sdk_import.cmake") {
        let detalhe = valor_de_set(texto, "PICO_BOARD")
            .map(|b| format!("PICO_BOARD {b}"))
            .or_else(|| valor_de_set(texto, "PICO_PLATFORM").map(|p| format!("PICO_PLATFORM {p}")));
        achados.push((Framework::PicoSdk, detalhe));
    }
    achados
}

/// `CONFIG_IDF_TARGET="esp32"` do `sdkconfig` (ou do `sdkconfig.defaults`) ao
/// lado do `CMakeLists.txt` do projeto.
fn alvo_do_idf(dir: &Path) -> Option<String> {
    ["sdkconfig", "sdkconfig.defaults"]
        .iter()
        .filter_map(|nome| ler(&dir.join(nome)))
        .find_map(|texto| valor_de_atribuicao(&texto, "CONFIG_IDF_TARGET"))
        .map(|alvo| format!("IDF_TARGET {alvo}"))
}

/// `set(NOME valor)` num `CMakeLists`, sem parser de `CMake`: basta para `BOARD`,
/// `PICO_BOARD` e `PICO_PLATFORM`, que os projetos escrevem assim.
fn valor_de_set(texto: &str, nome: &str) -> Option<String> {
    texto.lines().find_map(|linha| {
        let linha = linha.trim();
        let resto = linha.strip_prefix("set(")?.trim_start();
        let resto = resto.strip_prefix(nome)?;
        let valor = resto.trim_start().trim_end_matches(')').trim();
        let valor = valor.split_whitespace().next()?.trim_matches('"');
        (!valor.is_empty()).then(|| valor.to_owned())
    })
}

/// `CHAVE = valor`, `CHAVE ?= "valor"`, `CHAVE="valor"` — a forma comum de
/// sdkconfig, local.conf, .config, .ioc e external.desc.
fn valor_de_atribuicao(texto: &str, chave: &str) -> Option<String> {
    texto.lines().find_map(|linha| {
        let linha = linha.trim();
        let resto = linha.strip_prefix(chave)?;
        let resto = resto.trim_start();
        let resto = resto
            .strip_prefix("?=")
            .or_else(|| resto.strip_prefix(":="))
            .or_else(|| resto.strip_prefix('='))
            .or_else(|| resto.strip_prefix(':'))?;
        let valor = resto.trim().trim_matches('"').trim();
        (!valor.is_empty()).then(|| valor.to_owned())
    })
}

/// `[env:nome]` com `platform`/`board`: "esp32dev (espressif32, esp32dev)".
fn ambientes_do_platformio(texto: &str) -> String {
    let mut ambientes: Vec<String> = Vec::new();
    let mut atual: Option<(String, Option<String>, Option<String>)> = None;
    let fecha = |atual: &mut Option<(String, Option<String>, Option<String>)>,
                 ambientes: &mut Vec<String>| {
        if let Some((nome, plataforma, placa)) = atual.take() {
            let detalhes: Vec<String> = [plataforma, placa].into_iter().flatten().collect();
            ambientes.push(if detalhes.is_empty() {
                nome
            } else {
                format!("{nome} ({})", detalhes.join(", "))
            });
        }
    };
    for linha in texto.lines() {
        let linha = linha.trim();
        if let Some(nome) = linha
            .strip_prefix("[env:")
            .and_then(|l| l.strip_suffix(']'))
        {
            fecha(&mut atual, &mut ambientes);
            atual = Some((nome.to_owned(), None, None));
        } else if linha.starts_with('[') {
            fecha(&mut atual, &mut ambientes);
        } else if let Some((_, plataforma, placa)) = atual.as_mut() {
            if let Some((chave, valor)) = linha.split_once('=') {
                match chave.trim() {
                    "platform" => *plataforma = Some(valor.trim().to_owned()),
                    "board" => *placa = Some(valor.trim().to_owned()),
                    _ => {}
                }
            }
        }
    }
    fecha(&mut atual, &mut ambientes);
    ambientes.join("; ")
}

/// `[build] target = "thumbv7em-none-eabihf"` do `.cargo/config.toml`, so' se
/// for alvo bare metal.
fn alvo_do_cargo(texto: &str) -> Option<String> {
    texto.lines().find_map(|linha| {
        let linha = linha.trim();
        let resto = linha.strip_prefix("target")?.trim_start();
        let resto = resto.strip_prefix('=')?.trim();
        let triple = resto.trim_matches('"');
        (triple.starts_with("thumbv")
            || triple.starts_with("riscv32")
            || triple.starts_with("xtensa"))
        .then(|| triple.to_owned())
    })
}

/// `import machine` / `from machine import` (`MicroPython`) ou `board`
/// (`CircuitPython`): o modulo que so' existe no microcontrolador.
fn importa_hardware(texto: &str) -> Option<&'static str> {
    texto.lines().find_map(|linha| {
        let linha = linha.trim();
        ["machine", "board", "microcontroller"]
            .into_iter()
            .find(|modulo| {
                linha == format!("import {modulo}")
                    || linha.starts_with(&format!("import {modulo},"))
                    || linha.starts_with(&format!("from {modulo} import"))
            })
    })
}
