//! Gravar como CONFIGURACAO DE EXECUCAO (E4 do `integracoes/38` §6; decisao
//! do autor em 2026-09-11: "como o Upload do `PlatformIO` e o `OpenOCD` Download &
//! Run do `CLion`" — nao um dominio novo).
//!
//! Esta unidade e' PURA: compoe a linha de shell do motor a partir do que o
//! `project.model` ja' LEU (a receita `flasher_args.json`, o ELF/UF2/BIN mais
//! novo, o alvo) e da porta escolhida. Nada roda aqui. A UI mostra a linha
//! como previa; salvar e' `runConfig.save`, rodar e' `run.start` — a saida
//! vai para a aba de execucao existente, e a linha salva e' editavel como
//! qualquer configuracao (e' assim que um `esptool` v4 troca `write-flash`
//! por `write_flash`).
//!
//! # Os motores, e de onde cada linha veio (2026-09-17)
//!
//! ```text
//! esptool    `esptool --chip <chip> --port <porta> --baud 460800 --before <b>
//!            --after <a> [--no-stub] write-flash --flash-mode <m> --flash-size <s>
//!            --flash-freq <f> <offset> <arquivo> ...` — tudo da receita que o
//!            ESP-IDF escreve em build/flasher_args.json (integracoes/38 §5:
//!            "e' isto que uma integracao nativa le"); `write-flash` e' o nome
//!            da v5 (`esptool write-flash --help`, 5.4.0 desta maquina; a v5
//!            ainda aceita `write_flash` com aviso de deprecacao)
//! probe-rs   `probe-rs download --chip <chip> <elf>` (probe-rs CLI; o chip e'
//!            o do kit/modelo, como no launch do DAP)
//! picotool   `picotool load -f -x <uf2>` (`-f` forca BOOTSEL pelo USB, `-x`
//!            executa depois)
//! dfu-util   `dfu-util -a 0 -s 0x08000000:leave -D <bin>` — SO' STM32: o
//!            offset e' o inicio da flash interna dos STM32; para outra familia
//!            nao ha' tabela, e sem tabela nao ha' comando (nada adivinhado)
//! ```
//!
//! Caminhos vao entre aspas simples POSIX (a linha roda por `sh -c` no
//! `run.start`). Toda peca tem uma linha de EVIDENCIA (`source`) e o que o
//! usuario deve saber antes de apertar vai em `warnings`.

pub mod firmware;

use std::path::PathBuf;

use kinein_protocol::{FlashProposalResult, FlashRecipe, ProjectModel};

pub use firmware::FirmwareImage;

/// Baud de gravacao: o que o `idf.py flash` usa por padrao (`ESPBAUD`).
pub(super) const ESPTOOL_BAUD: &str = "460800";
/// Inicio da flash interna dos STM32 (mapa de memoria de toda a familia).
const STM32_FLASH_BASE: &str = "0x08000000";

/// Por que nao ha' linha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlashError {
    /// Nem o pedido nem o modelo dizem o motor.
    NoEngine {
        /// O que fazer.
        message: String,
    },
    /// Motor pedido que esta unidade nao conhece.
    UnknownEngine {
        /// O nome pedido.
        engine: String,
    },
    /// O motor existe mas o executavel nao esta' nesta maquina.
    MissingTool {
        /// O binario procurado.
        tool: String,
        /// O passo de instalacao.
        hint: String,
    },
    /// Falta o artefato/receita que o motor grava.
    NoArtifact {
        /// O que faltou e como obter.
        message: String,
    },
    /// O motor exige a porta e ela nao veio.
    NoDevice {
        /// O que fazer.
        message: String,
    },
}

impl std::fmt::Display for FlashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEngine { message }
            | Self::NoArtifact { message }
            | Self::NoDevice { message } => f.write_str(message),
            Self::UnknownEngine { engine } => write!(
                f,
                "motor de gravacao `{engine}` desconhecido: use esptool, probe-rs, picotool ou dfu-util"
            ),
            Self::MissingTool { tool, hint } => {
                write!(f, "{tool} nao esta' nesta maquina: {hint}")
            }
        }
    }
}

/// Quem acha o executavel de um motor (o `ToolDetector`, injetado para o
/// teste nao depender do PATH).
pub type FindTool<'a> = dyn Fn(&str) -> Option<PathBuf> + 'a;

/// A proposta: a linha do motor para este modelo, esta porta e esta flash.
///
/// # Errors
///
/// Sem motor, motor desconhecido, ferramenta ausente, artefato/receita
/// ausente ou porta ausente onde o motor a exige.
pub fn propose(
    model: &ProjectModel,
    engine: Option<&str>,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    firmware: Option<&FirmwareImage>,
    find_tool: &FindTool<'_>,
) -> Result<FlashProposalResult, FlashError> {
    // Um firmware baixado (C5) substitui os artefatos do build: a linha e'
    // a da pagina da placa, com o motor que ela fixa.
    if let Some(imagem) = firmware {
        return firmware::propose(model, imagem, engine, device, flash_size_bytes, find_tool);
    }
    let (engine, engine_source) = if let Some(pedido) =
        engine.map(str::trim).filter(|e| !e.is_empty())
    {
        (pedido.to_owned(), "motor: escolhido na tela".to_owned())
    } else if model.artifacts.flash_recipe.is_some() && model.target.flash_engine.is_none() {
        // Uma receita do ESP-IDF no build e' evidencia de sobra: so' o
        // esptool a le. Vale mesmo sem chip no kit/sdkconfig.
        (
            "esptool".to_owned(),
            "motor: esptool (o build tem a receita flasher_args.json do ESP-IDF)".to_owned(),
        )
    } else {
        let sugerido =
            model
                .target
                .flash_engine
                .as_deref()
                .ok_or_else(|| FlashError::NoEngine {
                    message: "o modelo do projeto nao sugere motor de gravacao (sem chip/familia \
                          reconhecidos): fixe o chip no kit ou escolha o motor na tela"
                        .to_owned(),
                })?;
        let familia = model
            .target
            .family
            .as_deref()
            .map(|f| format!(", familia {f}"))
            .unwrap_or_default();
        (
            sugerido.to_owned(),
            format!("motor: {sugerido} (sugerido pelo modelo do projeto{familia})"),
        )
    };
    let mut source = vec![engine_source];
    let mut warnings = Vec::new();
    let command = match engine.as_str() {
        "esptool" => esptool(
            model,
            device,
            flash_size_bytes,
            find_tool,
            &mut source,
            &mut warnings,
        )?,
        "probe-rs" => probe_rs(model, find_tool, &mut source)?,
        "picotool" => picotool(model, find_tool, &mut source)?,
        "dfu-util" => dfu_util(model, find_tool, &mut source)?,
        outro => {
            return Err(FlashError::UnknownEngine {
                engine: outro.to_owned(),
            });
        }
    };
    Ok(FlashProposalResult {
        name: format!("Gravar ({engine})"),
        command,
        engine,
        source,
        warnings,
    })
}

pub(super) fn tool(
    find_tool: &FindTool<'_>,
    binary: &str,
    hint: &str,
) -> Result<String, FlashError> {
    find_tool(binary)
        .map(|p| quote(&p.display().to_string()))
        .ok_or_else(|| FlashError::MissingTool {
            tool: binary.to_owned(),
            hint: hint.to_owned(),
        })
}

fn esptool(
    model: &ProjectModel,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<String, FlashError> {
    let esptool = find_tool("esptool")
        .or_else(|| find_tool("esptool.py"))
        .map(|p| quote(&p.display().to_string()))
        .ok_or_else(|| FlashError::MissingTool {
            tool: "esptool".to_owned(),
            hint: "pipx install esptool (o painel de instalacao mostra o passo)".to_owned(),
        })?;
    let receita: &FlashRecipe = model
        .artifacts
        .flash_recipe
        .as_ref()
        .filter(|r| !r.files.is_empty())
        .ok_or_else(|| FlashError::NoArtifact {
            message: "sem receita de gravacao: o ESP-IDF escreve build/flasher_args.json no \
                      build — compile o projeto primeiro"
                .to_owned(),
        })?;
    let device = device
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or_else(|| FlashError::NoDevice {
            message: "escolha a porta no painel de Embarcados (chip \"Executar\" na porta): \
                      gravar na placa errada e' pior que um clique"
                .to_owned(),
        })?;
    let chip = receita
        .chip
        .as_deref()
        .or(model.target.chip.as_deref())
        .unwrap_or("auto");
    let mut partes = vec![
        esptool,
        "--chip".to_owned(),
        chip.to_owned(),
        "--port".to_owned(),
        quote(device),
        "--baud".to_owned(),
        ESPTOOL_BAUD.to_owned(),
        "--before".to_owned(),
        receita
            .before
            .clone()
            .unwrap_or_else(|| "default-reset".to_owned()),
        "--after".to_owned(),
        receita
            .after
            .clone()
            .unwrap_or_else(|| "hard-reset".to_owned()),
    ];
    if !receita.stub {
        partes.push("--no-stub".to_owned());
    }
    partes.push("write-flash".to_owned());
    for (flag, valor) in [
        ("--flash-mode", &receita.flash_mode),
        ("--flash-size", &receita.flash_size),
        ("--flash-freq", &receita.flash_freq),
    ] {
        if let Some(v) = valor {
            partes.push(flag.to_owned());
            partes.push(v.clone());
        }
    }
    let mut cifradas = Vec::new();
    for imagem in &receita.files {
        partes.push(format!("0x{:x}", imagem.offset));
        partes.push(quote(&imagem.file));
        if imagem.encrypted {
            cifradas.push(imagem.name.clone().unwrap_or_else(|| imagem.file.clone()));
        }
    }
    esptool_evidencia(model, receita, chip, device, source);
    esptool_avisos(receita, &cifradas, flash_size_bytes, warnings);
    Ok(partes.join(" "))
}

/// As linhas de evidencia do esptool: receita, chip e porta.
fn esptool_evidencia(
    model: &ProjectModel,
    receita: &FlashRecipe,
    chip: &str,
    device: &str,
    source: &mut Vec<String>,
) {
    let tamanho = receita
        .flash_size
        .as_deref()
        .map(|s| format!(", flash {s}"))
        .unwrap_or_default();
    let modo = receita
        .flash_mode
        .as_deref()
        .map(|m| format!(" {m}"))
        .unwrap_or_default();
    source.push(format!(
        "receita: {} ({} imagens{tamanho}{modo})",
        model
            .artifacts
            .flasher_args
            .as_deref()
            .unwrap_or("flasher_args.json"),
        receita.files.len(),
    ));
    let origem = if receita.chip.is_some() {
        "receita"
    } else {
        "kit/modelo"
    };
    source.push(format!("chip: {chip} ({origem})"));
    source.push(format!(
        "porta: {device} (escolhida no painel de Embarcados)"
    ));
}

/// O que o usuario deve saber antes de gravar: imagens cifradas e flash
/// menor que a receita.
fn esptool_avisos(
    receita: &FlashRecipe,
    cifradas: &[String],
    flash_size_bytes: Option<u64>,
    warnings: &mut Vec<String>,
) {
    if !cifradas.is_empty() {
        warnings.push(format!(
            "imagens marcadas como cifradas ({}): write-flash grava em claro; com flash \
             encryption ativa na placa, acrescente `--encrypt` a linha antes de rodar",
            cifradas.join(", ")
        ));
    }
    if let (Some(placa), Some(receita_bytes)) = (flash_size_bytes, receita.flash_size_bytes) {
        if u64::from(receita_bytes) > placa {
            warnings.push(format!(
                "a receita foi gerada para {} de flash e a placa identificada tem {}: ajuste \
                 CONFIG_ESPTOOLPY_FLASHSIZE e recompile",
                receita.flash_size.as_deref().unwrap_or("?"),
                tamanho_legivel(placa)
            ));
        }
    }
}

fn probe_rs(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<String, FlashError> {
    let probe_rs = tool(
        find_tool,
        "probe-rs",
        "instale pelo script oficial (probe.rs/docs/getting-started/installation)",
    )?;
    let chip = model
        .target
        .chip
        .as_deref()
        .ok_or_else(|| FlashError::NoArtifact {
            message: "probe-rs download precisa do chip (`--chip`, como no `probe-rs chip list`): \
                  fixe-o no kit ou identifique a placa"
                .to_owned(),
        })?;
    let elf = mais_novo(&model.artifacts.elf).ok_or_else(|| FlashError::NoArtifact {
        message: "sem ELF nas pastas de build: compile o projeto primeiro".to_owned(),
    })?;
    source.push(format!("chip: {chip} (kit/modelo)"));
    source.push(format!("imagem: {elf} (o ELF mais novo do build)"));
    Ok(format!("{probe_rs} download --chip {chip} {}", quote(elf)))
}

fn picotool(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<String, FlashError> {
    let picotool = tool(
        find_tool,
        "picotool",
        "pacote `picotool` da distro ou o build oficial (github.com/raspberrypi/picotool)",
    )?;
    let uf2 = mais_novo(&model.artifacts.uf2).ok_or_else(|| FlashError::NoArtifact {
        message:
            "sem .uf2 nas pastas de build: o pico-sdk o gera no build (pico_add_extra_outputs)"
                .to_owned(),
    })?;
    source.push(format!("imagem: {uf2} (o UF2 mais novo do build)"));
    source.push("modo: -f forca o BOOTSEL pelo USB; -x executa depois de gravar".to_owned());
    Ok(format!("{picotool} load -f -x {}", quote(uf2)))
}

fn dfu_util(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<String, FlashError> {
    if model.target.family.as_deref() != Some("stm32") {
        return Err(FlashError::NoArtifact {
            message: format!(
                "dfu-util so' tem tabela para STM32 (flash em {STM32_FLASH_BASE}); a familia \
                 deste projeto e' {} — sem tabela, sem comando adivinhado",
                model.target.family.as_deref().unwrap_or("desconhecida")
            ),
        });
    }
    let dfu = tool(find_tool, "dfu-util", "pacote `dfu-util` da distro")?;
    let bin = mais_novo(&model.artifacts.bin).ok_or_else(|| FlashError::NoArtifact {
        message: "sem .bin nas pastas de build: gere o binario cru (objcopy -O binary) no build"
            .to_owned(),
    })?;
    source.push(format!("imagem: {bin} (o .bin mais novo do build)"));
    source.push(format!(
        "destino: alt 0, {STM32_FLASH_BASE} (inicio da flash interna STM32), :leave sai do DFU"
    ));
    Ok(format!(
        "{dfu} -a 0 -s {STM32_FLASH_BASE}:leave -D {}",
        quote(bin)
    ))
}

/// O `project.model` lista do mais novo ao mais velho.
fn mais_novo(lista: &[String]) -> Option<&str> {
    lista.first().map(String::as_str)
}

/// Aspas simples POSIX: dentro delas so' a propria aspa precisa de escape.
pub(super) fn quote(texto: &str) -> String {
    format!("'{}'", texto.replace('\'', "'\\''"))
}

pub(super) fn tamanho_legivel(bytes: u64) -> String {
    if bytes >= 1024 * 1024 && bytes % (1024 * 1024) == 0 {
        format!("{}MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 && bytes % 1024 == 0 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{FlashFile, ProjectArtifacts, TargetModel};

    use super::*;

    fn acha(binarios: &'static [&'static str]) -> impl Fn(&str) -> Option<PathBuf> {
        move |nome| {
            binarios
                .contains(&nome)
                .then(|| PathBuf::from("/x").join(nome))
        }
    }

    fn receita() -> FlashRecipe {
        FlashRecipe {
            chip: Some("esp32c3".to_owned()),
            flash_mode: Some("dio".to_owned()),
            flash_size: Some("4MB".to_owned()),
            flash_size_bytes: Some(4 * 1024 * 1024),
            flash_freq: Some("80m".to_owned()),
            before: Some("default-reset".to_owned()),
            after: Some("hard-reset".to_owned()),
            stub: true,
            files: vec![
                FlashFile {
                    offset: 0x0,
                    file: "/p/build/bootloader/bootloader.bin".to_owned(),
                    name: Some("bootloader".to_owned()),
                    encrypted: false,
                },
                FlashFile {
                    offset: 0x8000,
                    file: "/p/build/partition_table/partition-table.bin".to_owned(),
                    name: Some("partition-table".to_owned()),
                    encrypted: false,
                },
                FlashFile {
                    offset: 0x10000,
                    file: "/p/build/o meu app.bin".to_owned(),
                    name: Some("app".to_owned()),
                    encrypted: true,
                },
            ],
        }
    }

    fn modelo(family: &str, engine: Option<&str>, chip: Option<&str>) -> ProjectModel {
        ProjectModel {
            target: TargetModel {
                chip: chip.map(str::to_owned),
                family: Some(family.to_owned()),
                flash_engine: engine.map(str::to_owned),
                ..TargetModel::default()
            },
            root: "/p".to_owned(),
            embedded: true,
            frameworks: Vec::new(),
            sdks: Vec::new(),
            artifacts: ProjectArtifacts::default(),
            hints: Vec::new(),
        }
    }

    /// esptool: tudo da receita, porta entre aspas, arquivo com espaco entre
    /// aspas, cifrada avisada, e a linha e' a da v5.
    #[test]
    fn esptool_line_comes_from_the_recipe_and_the_chosen_port() {
        let mut m = modelo("espressif", Some("esptool"), Some("esp32c3"));
        m.artifacts = ProjectArtifacts {
            flash_recipe: Some(receita()),
            flasher_args: Some("/p/build/flasher_args.json".to_owned()),
            ..ProjectArtifacts::default()
        };
        let p = propose(
            &m,
            None,
            Some("/dev/ttyUSB0"),
            Some(4 * 1024 * 1024),
            None,
            &acha(&["esptool"]),
        )
        .unwrap();
        assert_eq!(p.name, "Gravar (esptool)");
        assert_eq!(p.engine, "esptool");
        assert_eq!(
            p.command,
            "'/x/esptool' --chip esp32c3 --port '/dev/ttyUSB0' --baud 460800 --before default-reset \
             --after hard-reset write-flash --flash-mode dio --flash-size 4MB --flash-freq 80m \
             0x0 '/p/build/bootloader/bootloader.bin' 0x8000 '/p/build/partition_table/partition-table.bin' \
             0x10000 '/p/build/o meu app.bin'"
        );
        assert_eq!(
            p.source[0],
            "motor: esptool (sugerido pelo modelo do projeto, familia espressif)"
        );
        assert!(
            p.source[1]
                .starts_with("receita: /p/build/flasher_args.json (3 imagens, flash 4MB dio)"),
            "{:?}",
            p.source
        );
        assert!(
            p.source
                .iter()
                .any(|s| s == "porta: /dev/ttyUSB0 (escolhida no painel de Embarcados)")
        );
        assert_eq!(p.warnings.len(), 1);
        assert!(p.warnings[0].contains("app") && p.warnings[0].contains("--encrypt"));
    }

    /// Sem receita: "compile"; sem porta: "escolha"; sem esptool: o passo;
    /// `--no-stub` quando a receita diz; flash da placa menor que a receita
    /// avisa; o `esptool.py` serve.
    #[test]
    fn esptool_refusals_and_variants() {
        let mut m = modelo("espressif", Some("esptool"), None);
        let acha_tool = acha(&["esptool.py"]);
        let erro = propose(&m, None, Some("/dev/ttyUSB0"), None, None, &acha_tool).unwrap_err();
        assert!(matches!(erro, FlashError::NoArtifact { .. }), "{erro}");
        assert!(erro.to_string().contains("flasher_args.json"));

        let mut r = receita();
        r.stub = false;
        r.chip = None;
        m.artifacts.flash_recipe = Some(r);
        let erro = propose(&m, None, None, None, None, &acha_tool).unwrap_err();
        assert!(matches!(erro, FlashError::NoDevice { .. }), "{erro}");

        let p = propose(
            &m,
            None,
            Some("/dev/ttyACM0"),
            Some(2 * 1024 * 1024),
            None,
            &acha_tool,
        )
        .unwrap();
        assert!(
            p.command
                .starts_with("'/x/esptool.py' --chip auto --port '/dev/ttyACM0'"),
            "{}",
            p.command
        );
        assert!(p.command.contains(" --no-stub write-flash "));
        assert!(
            p.warnings
                .iter()
                .any(|w| w.contains("4MB") && w.contains("2MB")),
            "{:?}",
            p.warnings
        );

        let erro = propose(&m, None, Some("/dev/ttyACM0"), None, None, &acha(&[])).unwrap_err();
        assert!(matches!(erro, FlashError::MissingTool { .. }));
        assert!(erro.to_string().contains("pipx install esptool"));
    }

    #[test]
    fn probe_rs_picotool_and_dfu_util_lines() {
        let mut m = modelo("stm32", Some("probe-rs"), Some("STM32F401CC"));
        m.artifacts.elf = vec![
            "/p/target/thumbv7em/debug/fw".to_owned(),
            "/p/old.elf".to_owned(),
        ];
        m.artifacts.bin = vec!["/p/build/fw.bin".to_owned()];
        let p = propose(
            &m,
            None,
            None,
            None,
            None,
            &acha(&["probe-rs", "dfu-util", "picotool"]),
        )
        .unwrap();
        assert_eq!(
            p.command,
            "'/x/probe-rs' download --chip STM32F401CC '/p/target/thumbv7em/debug/fw'"
        );
        assert_eq!(p.name, "Gravar (probe-rs)");

        // O motor escolhido na tela vence o sugerido.
        let p = propose(&m, Some("dfu-util"), None, None, None, &acha(&["dfu-util"])).unwrap();
        assert_eq!(
            p.command,
            "'/x/dfu-util' -a 0 -s 0x08000000:leave -D '/p/build/fw.bin'"
        );
        assert_eq!(p.source[0], "motor: escolhido na tela");

        // dfu-util fora de STM32: sem tabela, sem comando.
        let mut pico = modelo("rp2040", Some("picotool"), Some("rp2040"));
        pico.artifacts.bin = vec!["/p/x.bin".to_owned()];
        let erro = propose(
            &pico,
            Some("dfu-util"),
            None,
            None,
            None,
            &acha(&["dfu-util"]),
        )
        .unwrap_err();
        assert!(erro.to_string().contains("rp2040"), "{erro}");

        pico.artifacts.uf2 = vec!["/p/build/blink.uf2".to_owned()];
        let p = propose(&pico, None, None, None, None, &acha(&["picotool"])).unwrap();
        assert_eq!(p.command, "'/x/picotool' load -f -x '/p/build/blink.uf2'");

        // probe-rs sem chip: pede o kit; sem ELF: pede o build.
        let mut sem_chip = modelo("cortex-m", Some("probe-rs"), None);
        sem_chip.artifacts.elf = vec!["/p/fw".to_owned()];
        assert!(
            propose(&sem_chip, None, None, None, None, &acha(&["probe-rs"]))
                .unwrap_err()
                .to_string()
                .contains("--chip")
        );
        let sem_elf = modelo("cortex-m", Some("probe-rs"), Some("nRF52840_xxAA"));
        assert!(
            propose(&sem_elf, None, None, None, None, &acha(&["probe-rs"]))
                .unwrap_err()
                .to_string()
                .contains("sem ELF")
        );
    }

    #[test]
    fn no_engine_and_unknown_engine_are_named() {
        let m = modelo("linux", None, None);
        let erro = propose(&m, None, None, None, None, &acha(&[])).unwrap_err();
        assert!(matches!(erro, FlashError::NoEngine { .. }));
        let erro = propose(&m, Some("avrdude"), None, None, None, &acha(&[])).unwrap_err();
        assert_eq!(
            erro.to_string(),
            "motor de gravacao `avrdude` desconhecido: use esptool, probe-rs, picotool ou dfu-util"
        );
        assert_eq!(quote("it's"), "'it'\\''s'");
        assert_eq!(tamanho_legivel(2 * 1024 * 1024), "2MB");
        assert_eq!(tamanho_legivel(512 * 1024), "512KB");
    }
}
