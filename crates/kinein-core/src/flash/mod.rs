//! Gravar como CONFIGURACAO DE EXECUCAO (E4 do `integracoes/38` §6; decisao
//! do autor em 2026-09-11: "como o Upload do `PlatformIO` e o `OpenOCD` Download &
//! Run do `CLion`" — nao um dominio novo).
//!
//! Esta unidade e' PURA: planeja programa/argv do motor a partir do que o
//! `project.model` ja' LEU (a receita `flasher_args.json`, o ELF/UF2/BIN mais
//! novo, o alvo) e da porta escolhida. A previa deriva desse mesmo plano;
//! nenhuma linha salva e' analisada para reconstruir argumentos. Nada roda
//! aqui. A UI mostra a linha
//! como previa; salvar e' `runConfig.save`, rodar e' `run.start` — a saida
//! vai para a aba de execucao existente, e a linha salva e' editavel como
//! qualquer configuracao manual. A sintaxe da nova previa acompanha a versao
//! do binario consultado antes deste planejamento puro.
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

mod artifact;
mod command;
pub mod dfu;
mod elf;
pub mod esptool;
pub mod firmware;
pub mod frameworks;
pub mod idf_py;
pub mod native;
pub mod pico;
pub mod platformio;
pub mod probe_rs;
mod snapshot;

use std::path::PathBuf;

use kinein_protocol::{FlashProposalResult, ProjectModel};

pub use command::{FlashCommand, FlashPlan};
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
    /// Native selection fields of another destination kind (`INVALID_PARAMS`).
    InvalidSelection {
        /// Which field and why.
        message: String,
    },
    /// The selected binary could not establish a supported syntax.
    UnsupportedTool {
        /// Actionable version/probe failure.
        message: String,
    },
    /// Native provenance or snapshot could not be established.
    InvalidContext {
        /// Cause and next action, without treating missing proof as compatibility.
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
            | Self::NoDevice { message }
            | Self::UnsupportedTool { message }
            | Self::InvalidSelection { message }
            | Self::InvalidContext { message } => f.write_str(message),
            Self::UnknownEngine { engine } => write!(
                f,
                "motor de gravação `{engine}` desconhecido: use esptool, probe-rs, picotool ou dfu-util"
            ),
            Self::MissingTool { tool, hint } => {
                write!(f, "{tool} não está nesta máquina: {hint}")
            }
        }
    }
}

/// Quem acha o executavel de um motor (o `ToolDetector`, injetado para o
/// teste nao depender do PATH).
pub type FindTool<'a> = dyn Fn(&str) -> Option<PathBuf> + 'a;

/// Resolved paths and the esptool syntax confirmed before pure planning.
pub struct ProposalTools<'a> {
    /// Existing detector, without subprocess queries during planning.
    pub find: &'a FindTool<'a>,
    /// The selected esptool and its observed version; absent for other engines.
    pub esptool: Option<esptool::Selected>,
}

impl std::fmt::Debug for ProposalTools<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProposalTools")
            .field("esptool", &self.esptool)
            .finish_non_exhaustive()
    }
}

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
    framework: Option<&crate::build::Engine>,
    tools: &ProposalTools<'_>,
) -> Result<FlashProposalResult, FlashError> {
    plan(
        model,
        engine,
        device,
        flash_size_bytes,
        firmware,
        framework,
        tools,
    )
    .map(|plan| plan.proposal)
}

/// Prepare the existing proposal while retaining its original program and argv.
///
/// # Errors
/// The same engine/tool/artifact/device refusals as [`propose`].
pub fn plan(
    model: &ProjectModel,
    engine: Option<&str>,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    firmware: Option<&FirmwareImage>,
    framework: Option<&crate::build::Engine>,
    tools: &ProposalTools<'_>,
) -> Result<FlashPlan, FlashError> {
    let find_tool = tools.find;
    // Um firmware baixado (C5) substitui os artefatos do build: a linha e'
    // a da pagina da placa, com o motor que ela fixa.
    if let Some(imagem) = firmware {
        return firmware::propose(model, imagem, engine, device, flash_size_bytes, tools);
    }
    // O wrapper do framework (bloco E): pedido pelo nome, ou o que o modelo
    // sugere para Zephyr (`west`) e PlatformIO (`platformio`).
    let pedido = engine.map(str::trim).filter(|e| !e.is_empty());
    let sugerido = model.target.flash_engine.as_deref();
    if let Some(nome) = pedido
        .filter(|e| frameworks::is_framework_engine(e))
        .or_else(|| sugerido.filter(|e| pedido.is_none() && frameworks::is_framework_engine(e)))
    {
        return frameworks::propose(model, nome, device, framework);
    }
    let (engine, engine_source) = select_engine(model, engine, None)?;
    let mut source = vec![engine_source];
    let mut warnings = Vec::new();
    let command = match engine.as_str() {
        "esptool" => esptool::propose(
            model,
            device,
            flash_size_bytes,
            tools,
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
    Ok(FlashPlan::new(
        format!("Gravar ({engine})"),
        engine,
        source,
        warnings,
        command,
    ))
}

/// Resolve the engine using the same project/explicit precedence as planning.
///
/// # Errors
/// Refuses a project without a selected or evidenced engine.
pub fn select_engine(
    model: &ProjectModel,
    engine: Option<&str>,
    firmware: Option<&FirmwareImage>,
) -> Result<(String, String), FlashError> {
    if let Some(image) = firmware {
        return firmware::select_engine(image, engine);
    }
    let selected = if let Some(pedido) = engine.map(str::trim).filter(|e| !e.is_empty()) {
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
                    message: "o modelo do projeto não sugere motor de gravação (sem chip/familia \
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
    Ok(selected)
}

pub(super) fn tool(
    find_tool: &FindTool<'_>,
    binary: &str,
    hint: &str,
) -> Result<PathBuf, FlashError> {
    find_tool(binary).ok_or_else(|| FlashError::MissingTool {
        tool: binary.to_owned(),
        hint: hint.to_owned(),
    })
}

/// A porta escolhida pelo usuario. Sem porta nao ha' gravacao: adivinhar a
/// placa errada e' pior que um clique.
fn chosen_port(device: Option<&str>) -> Result<&str, FlashError> {
    device
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .ok_or_else(|| FlashError::NoDevice {
            message:
                "escolha a porta em Gravar e depurar › Placa usando Selecionar porta antes de \
                      gerar a prévia de gravação"
                    .to_owned(),
        })
}

fn probe_rs(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
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
    Ok(FlashCommand::new(
        probe_rs,
        vec![
            "download".to_owned(),
            "--chip".to_owned(),
            chip.to_owned(),
            elf.to_owned(),
        ],
    ))
}

fn picotool(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
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
    Ok(FlashCommand::new(
        picotool,
        vec![
            "load".to_owned(),
            "-f".to_owned(),
            "-x".to_owned(),
            uf2.to_owned(),
        ],
    ))
}

fn dfu_util(
    model: &ProjectModel,
    find_tool: &FindTool<'_>,
    source: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
    if model.target.family.as_deref() != Some("stm32") {
        return Err(FlashError::NoArtifact {
            message: format!(
                "dfu-util só tem tabela para STM32 (flash em {STM32_FLASH_BASE}); a familia \
                 deste projeto é {} — sem tabela, sem comando adivinhado",
                model.target.family.as_deref().unwrap_or("desconhecida")
            ),
        });
    }
    let dfu = tool(find_tool, "dfu-util", "pacote `dfu-util` da distro")?;
    let bin = mais_novo(&model.artifacts.bin).ok_or_else(|| FlashError::NoArtifact {
        message: "sem .bin nas pastas de build: gere o binário cru (objcopy -O binary) no build"
            .to_owned(),
    })?;
    source.push(format!("imagem: {bin} (o .bin mais novo do build)"));
    source.push(format!(
        "destino: alt 0, {STM32_FLASH_BASE} (início da flash interna STM32), :leave sai do DFU"
    ));
    Ok(FlashCommand::new(
        dfu,
        vec![
            "-a".to_owned(),
            "0".to_owned(),
            "-s".to_owned(),
            format!("{STM32_FLASH_BASE}:leave"),
            "-D".to_owned(),
            bin.to_owned(),
        ],
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

pub(super) fn readable_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 && bytes.is_multiple_of(1024 * 1024) {
        format!("{}MB", bytes / (1024 * 1024))
    } else if bytes >= 1024 && bytes.is_multiple_of(1024) {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
fn propose_v5(
    model: &ProjectModel,
    engine: Option<&str>,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    firmware: Option<&FirmwareImage>,
    framework: Option<&crate::build::Engine>,
    find: &FindTool<'_>,
) -> Result<FlashProposalResult, FlashError> {
    let selected = find("esptool")
        .or_else(|| find("esptool.py"))
        .map(|path| esptool::Selected::from_output(path, "5.4.0").unwrap());
    propose(
        model,
        engine,
        device,
        flash_size_bytes,
        firmware,
        framework,
        &ProposalTools {
            find,
            esptool: selected,
        },
    )
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{FlashFile, FlashRecipe, ProjectArtifacts, TargetModel};

    use super::propose_v5 as propose;
    use super::*;

    fn find_tools(binarios: &'static [&'static str]) -> impl Fn(&str) -> Option<PathBuf> {
        move |nome| {
            binarios
                .contains(&nome)
                .then(|| PathBuf::from("/x").join(nome))
        }
    }

    fn recipe() -> FlashRecipe {
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

    fn model(family: &str, engine: Option<&str>, chip: Option<&str>) -> ProjectModel {
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

    #[test]
    fn recipe_values_remain_literal_in_the_shell_preview() {
        let injected = "dio; printf 'UNEXPECTED-SHELL-CONTENT' #";
        let mut model = model("espressif", Some("esptool"), Some("esp32c3"));
        let mut recipe = recipe();
        recipe.flash_mode = Some(injected.to_owned());
        model.artifacts.flash_recipe = Some(recipe);
        let directory =
            std::env::temp_dir().join(format!("{}-flash-recipe-literal", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let program = crate::write_executable(
            directory.join("fake-esptool"),
            "#!/bin/sh\nprintf '<%s>' \"$@\"\n",
        );
        let proposal = propose(
            &model,
            None,
            Some("/dev/unused-fixture"),
            None,
            None,
            None,
            &|_| Some(program.clone()),
        )
        .unwrap();
        let output = crate::sh_command(&proposal.command)
            .env("MSYS2_ARG_CONV_EXCL", "*")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout).unwrap().contains(injected),
            "a recipe value became shell syntax instead of one literal argument"
        );
    }

    #[test]
    fn structured_plans_keep_engine_inputs_as_literal_arguments() {
        let chip = "chip; $(printf literal)";
        let path = "/build/a'b c-one_two.bin";
        let device = "/device/board' one_two";
        let mut model = model("stm32", None, Some(chip));
        let mut recipe = recipe();
        recipe.chip = Some(chip.to_owned());
        model.artifacts = ProjectArtifacts {
            elf: vec![path.to_owned()],
            uf2: vec![path.to_owned()],
            bin: vec![path.to_owned()],
            flash_recipe: Some(recipe),
            ..ProjectArtifacts::default()
        };
        let find = find_tools(&["esptool", "probe-rs", "picotool", "dfu-util"]);
        let tools = ProposalTools {
            find: &find,
            esptool: Some(esptool::Selected::from_output("/x/esptool".into(), "5.4.0").unwrap()),
        };
        for (engine, expected) in [
            ("probe-rs", vec!["download", "--chip", chip, path]),
            ("picotool", vec!["load", "-f", "-x", path]),
            (
                "dfu-util",
                vec!["-a", "0", "-s", "0x08000000:leave", "-D", path],
            ),
        ] {
            let prepared =
                plan(&model, Some(engine), Some(device), None, None, None, &tools).unwrap();
            assert_eq!(prepared.command.program, PathBuf::from("/x").join(engine));
            assert_eq!(prepared.command.args, expected);
        }
        let prepared = plan(
            &model,
            Some("esptool"),
            Some(device),
            None,
            None,
            None,
            &tools,
        )
        .unwrap();
        assert_eq!(
            &prepared.command.args[..6],
            ["--chip", chip, "--port", device, "--baud", "460800"]
        );
        assert_eq!(
            prepared.command.args.last().map(String::as_str),
            Some("/p/build/o meu app.bin")
        );

        let image = FirmwareImage {
            label: "Test firmware".to_owned(),
            engine: "esptool".to_owned(),
            offset: Some("0x0".to_owned()),
            chip: Some(chip.to_owned()),
            file: path.into(),
            size_bytes: 100,
            expected_sha256: None,
        };
        let prepared = plan(&model, None, Some(device), None, Some(&image), None, &tools).unwrap();
        assert_eq!(
            &prepared.command.args[..4],
            ["--chip", chip, "--port", device]
        );
        assert_eq!(
            &prepared.command.args[prepared.command.args.len() - 3..],
            ["write-flash", "0x0", path]
        );
        let image = FirmwareImage {
            engine: "picotool".to_owned(),
            ..image
        };
        let prepared = plan(&model, None, None, None, Some(&image), None, &tools).unwrap();
        assert_eq!(prepared.command.args, ["load", "-f", "-x", path]);
    }

    /// esptool: tudo da receita, porta entre aspas, arquivo com espaco entre
    /// aspas, cifrada avisada, e a linha e' a da v5.
    #[test]
    #[cfg_attr(
        windows,
        ignore = "W10: a linha de gravacao fala /dev/tty e caminho de Linux; a do Windows (COMx) e' da W10 (60 §3.3)"
    )]
    fn esptool_line_comes_from_the_recipe_and_the_chosen_port() {
        let mut m = model("espressif", Some("esptool"), Some("esp32c3"));
        m.artifacts = ProjectArtifacts {
            flash_recipe: Some(recipe()),
            flasher_args: Some("/p/build/flasher_args.json".to_owned()),
            ..ProjectArtifacts::default()
        };
        let p = propose(
            &m,
            None,
            Some("/dev/ttyUSB0"),
            Some(4 * 1024 * 1024),
            None,
            None,
            &find_tools(&["esptool"]),
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
                .any(|s| s == "porta: /dev/ttyUSB0 (escolhida em Gravar e depurar)")
        );
        assert_eq!(p.warnings.len(), 1);
        assert!(p.warnings[0].contains("app") && p.warnings[0].contains("--encrypt"));
    }

    /// Sem receita: "compile"; sem porta: "escolha"; sem esptool: o passo;
    /// `--no-stub` quando a receita diz; flash da placa menor que a receita
    /// avisa; o `esptool.py` serve.
    #[test]
    #[cfg_attr(
        windows,
        ignore = "W10: a linha de gravacao fala /dev/tty e caminho de Linux; a do Windows (COMx) e' da W10 (60 §3.3)"
    )]
    fn esptool_refusals_and_variants() {
        let mut m = model("espressif", Some("esptool"), None);
        let acha_tool = find_tools(&["esptool.py"]);
        let erro =
            propose(&m, None, Some("/dev/ttyUSB0"), None, None, None, &acha_tool).unwrap_err();
        assert!(matches!(erro, FlashError::NoArtifact { .. }), "{erro}");
        assert!(erro.to_string().contains("flasher_args.json"));

        let mut r = recipe();
        r.stub = false;
        r.chip = None;
        m.artifacts.flash_recipe = Some(r);
        let erro = propose(&m, None, None, None, None, None, &acha_tool).unwrap_err();
        assert!(matches!(erro, FlashError::NoDevice { .. }), "{erro}");

        let p = propose(
            &m,
            None,
            Some("/dev/ttyACM0"),
            Some(2 * 1024 * 1024),
            None,
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

        let erro = propose(
            &m,
            None,
            Some("/dev/ttyACM0"),
            None,
            None,
            None,
            &find_tools(&[]),
        )
        .unwrap_err();
        assert!(matches!(erro, FlashError::MissingTool { .. }));
        assert!(erro.to_string().contains("pipx install esptool"));
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "W10: a linha de gravacao fala /dev/tty e caminho de Linux; a do Windows (COMx) e' da W10 (60 §3.3)"
    )]
    fn probe_rs_picotool_and_dfu_util_lines() {
        let mut m = model("stm32", Some("probe-rs"), Some("STM32F401CC"));
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
            None,
            &find_tools(&["probe-rs", "dfu-util", "picotool"]),
        )
        .unwrap();
        assert_eq!(
            p.command,
            "'/x/probe-rs' download --chip STM32F401CC '/p/target/thumbv7em/debug/fw'"
        );
        assert_eq!(p.name, "Gravar (probe-rs)");

        // O motor escolhido na tela vence o sugerido.
        let p = propose(
            &m,
            Some("dfu-util"),
            None,
            None,
            None,
            None,
            &find_tools(&["dfu-util"]),
        )
        .unwrap();
        assert_eq!(
            p.command,
            "'/x/dfu-util' -a 0 -s 0x08000000:leave -D '/p/build/fw.bin'"
        );
        assert_eq!(p.source[0], "motor: escolhido na tela");

        // dfu-util fora de STM32: sem tabela, sem comando.
        let mut pico = model("rp2040", Some("picotool"), Some("rp2040"));
        pico.artifacts.bin = vec!["/p/x.bin".to_owned()];
        let erro = propose(
            &pico,
            Some("dfu-util"),
            None,
            None,
            None,
            None,
            &find_tools(&["dfu-util"]),
        )
        .unwrap_err();
        assert!(erro.to_string().contains("rp2040"), "{erro}");

        pico.artifacts.uf2 = vec!["/p/build/blink.uf2".to_owned()];
        let p = propose(
            &pico,
            None,
            None,
            None,
            None,
            None,
            &find_tools(&["picotool"]),
        )
        .unwrap();
        assert_eq!(p.command, "'/x/picotool' load -f -x '/p/build/blink.uf2'");

        // probe-rs sem chip: pede o kit; sem ELF: pede o build.
        let mut sem_chip = model("cortex-m", Some("probe-rs"), None);
        sem_chip.artifacts.elf = vec!["/p/fw".to_owned()];
        assert!(
            propose(
                &sem_chip,
                None,
                None,
                None,
                None,
                None,
                &find_tools(&["probe-rs"])
            )
            .unwrap_err()
            .to_string()
            .contains("--chip")
        );
        let sem_elf = model("cortex-m", Some("probe-rs"), Some("nRF52840_xxAA"));
        assert!(
            propose(
                &sem_elf,
                None,
                None,
                None,
                None,
                None,
                &find_tools(&["probe-rs"])
            )
            .unwrap_err()
            .to_string()
            .contains("sem ELF")
        );
    }

    #[test]
    fn no_engine_and_unknown_engine_are_named() {
        let m = model("linux", None, None);
        let erro = propose(&m, None, None, None, None, None, &find_tools(&[])).unwrap_err();
        assert!(matches!(erro, FlashError::NoEngine { .. }));
        let erro = propose(
            &m,
            Some("avrdude"),
            None,
            None,
            None,
            None,
            &find_tools(&[]),
        )
        .unwrap_err();
        assert_eq!(
            erro.to_string(),
            "motor de gravação `avrdude` desconhecido: use esptool, probe-rs, picotool ou dfu-util"
        );
        assert_eq!(quote("it's"), "'it'\\''s'");
        assert_eq!(readable_size(2 * 1024 * 1024), "2MB");
        assert_eq!(readable_size(512 * 1024), "512KB");
    }
}
