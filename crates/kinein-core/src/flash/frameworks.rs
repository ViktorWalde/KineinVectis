//! Gravar pelo WRAPPER do framework (bloco E do `roadmaps/41`, 2026-09-17):
//! `idf.py flash`, `west flash`, `pio run -t upload`. Linhas lidas na fonte:
//!
//! ```text
//! idf.py      `idf.py -p PORT flash` (ESP-IDF "Start a Project": "Replace PORT
//!             with your ESP32 board's USB port name"; sem -p ele autodetecta —
//!             aqui a porta e' exigida, como no esptool do E4). Roda no ambiente
//!             ativado: `bash -c '<IDF_WRAPPER>' bash <ativacao> -p PORT flash`
//! west        `west flash -d build` (Zephyr "west flash": grava o build da
//!             pasta com o runner do board; `--runner` fica para o usuario editar)
//! pio         `pio run -t upload [--upload-port PORT]` (PlatformIO "pio run":
//!             `-t upload`; `--upload-port` quando se quer fixar a porta)
//! ```
//!
//! O esptool do E4 continua sendo o padrao num projeto ESP-IDF (a receita
//! `flasher_args.json` e' mais transparente); o `idf.py` e' escolha explicita.

use kinein_protocol::ProjectModel;

use super::{FlashCommand, FlashError, FlashPlan};
use crate::build::engine::{Engine, idf_command_args};

/// Os nomes de motor que esta unidade atende (a tela os lista ao lado dos
/// do E4); `platformio` e' a palavra do `project.model`, `pio` a do binario.
pub const ENGINES: [&str; 3] = ["idf.py", "west", "platformio"];

/// `true` quando o nome pedido e' de um wrapper de framework.
#[must_use]
pub fn is_framework_engine(engine: &str) -> bool {
    matches!(engine, "idf.py" | "idf" | "west" | "platformio" | "pio")
}

/// A proposta pelo wrapper. `engine` ja' e' um dos nomes acima; `framework`
/// e' o motor que o modelo resolveu (a ativacao do IDF, o west, o pio) —
/// sem ele, o framework nao esta' nesta maquina e a mensagem diz o passo.
pub(super) fn propose(
    model: &ProjectModel,
    engine: &str,
    device: Option<&str>,
    framework: Option<&Engine>,
) -> Result<FlashPlan, FlashError> {
    let device = device.map(str::trim).filter(|d| !d.is_empty());
    let mut source = Vec::new();
    let mut warnings = Vec::new();
    let (nome, command) = match (engine, framework) {
        ("idf.py" | "idf", Some(Engine::EspIdf { activation })) => {
            let porta = device.ok_or_else(|| FlashError::NoDevice {
                message: "escolha a porta no painel de Embarcados: o idf.py autodetecta, mas \
                          gravar na placa errada é pior que um clique"
                    .to_owned(),
            })?;
            source.push(format!(
                "ambiente: . {} (a ativação do ESP-IDF, como o Get Started manda)",
                activation.display()
            ));
            source.push(format!(
                "porta: {porta} (escolhida no painel de Embarcados)"
            ));
            source.push(
                "o que grava: o build/ do idf.py (bootloader, tabela de partições, app)".to_owned(),
            );
            let args = idf_command_args(activation, &["-p", porta, "flash"]);
            ("idf.py", FlashCommand::new("bash", args))
        }
        ("idf.py" | "idf", _) => return Err(sem_framework("ESP-IDF", model, "esp-idf")),
        ("west", Some(Engine::Zephyr { west })) => {
            source
                .push("o que grava: build/ pelo runner do board (west flash -d build)".to_owned());
            if device.is_some() {
                warnings.push(
                    "o west grava pelo runner do board (sonda/USB), não pela porta escolhida; \
                     para um runner serial acrescente `--dev-id`/`--runner` a linha"
                        .to_owned(),
                );
            }
            (
                "west",
                FlashCommand::new(
                    west.clone(),
                    vec!["flash".to_owned(), "-d".to_owned(), "build".to_owned()],
                ),
            )
        }
        ("west", _) => return Err(sem_framework("Zephyr", model, "west")),
        ("platformio" | "pio", Some(Engine::PlatformIo { pio })) => {
            let mut args = vec!["run".to_owned(), "-t".to_owned(), "upload".to_owned()];
            if let Some(porta) = device {
                args.extend(["--upload-port".to_owned(), porta.to_owned()]);
                source.push(format!("porta: {porta} (--upload-port)"));
            } else {
                source.push("porta: a que o pio detectar (sem --upload-port)".to_owned());
            }
            source
                .push("o que grava: o env padrão do platformio.ini (pio run -t upload)".to_owned());
            ("platformio", FlashCommand::new(pio.clone(), args))
        }
        ("platformio" | "pio", _) => return Err(sem_framework("PlatformIO", model, "pio")),
        (outro, _) => {
            return Err(FlashError::UnknownEngine {
                engine: outro.to_owned(),
            });
        }
    };
    source.insert(0, format!("motor: {nome} (o wrapper do framework)"));
    Ok(FlashPlan::new(
        format!("Gravar ({nome})"),
        nome.to_owned(),
        source,
        warnings,
        command,
    ))
}

fn sem_framework(framework: &str, model: &ProjectModel, sdk: &str) -> FlashError {
    let hint = model
        .sdks
        .iter()
        .find(|s| s.id == sdk)
        .and_then(|s| s.hint.clone())
        .unwrap_or_default();
    FlashError::MissingTool {
        tool: framework.to_owned(),
        hint: if model.frameworks.is_empty() {
            format!("este projeto não é {framework} (o modelo não achou o marcador)")
        } else {
            hint
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::{ProjectArtifacts, ProjectModel, SdkRequirement, TargetModel};

    use super::super::FlashError;
    use super::propose;
    use crate::build::engine::Engine;

    fn modelo() -> ProjectModel {
        ProjectModel {
            root: "/p".to_owned(),
            embedded: true,
            frameworks: Vec::new(),
            sdks: vec![SdkRequirement {
                id: "esp-idf".to_owned(),
                label: "ESP-IDF".to_owned(),
                env: Some("IDF_PATH".to_owned()),
                path: None,
                found: false,
                hint: Some("git clone … esp-idf".to_owned()),
            }],
            artifacts: ProjectArtifacts::default(),
            target: TargetModel::default(),
            hints: Vec::new(),
        }
    }

    #[test]
    fn each_wrapper_has_its_documented_line() {
        let idf = Engine::EspIdf {
            activation: PathBuf::from("/e/export.sh"),
        };
        let plan = propose(&modelo(), "idf.py", Some("/dev/ttyUSB0"), Some(&idf)).unwrap();
        assert_eq!(plan.command.program, PathBuf::from("bash"));
        assert_eq!(
            plan.command.args,
            crate::build::engine::idf_command_args(
                Path::new("/e/export.sh"),
                &["-p", "/dev/ttyUSB0", "flash"]
            )
        );
        let p = plan.proposal;
        assert!(p.command.starts_with("'bash' -c '"), "{}", p.command);
        assert!(
            p.command
                .ends_with("' bash '/e/export.sh' -p '/dev/ttyUSB0' flash"),
            "{}",
            p.command
        );
        assert_eq!(p.engine, "idf.py");
        assert!(matches!(
            propose(&modelo(), "idf.py", None, Some(&idf)),
            Err(FlashError::NoDevice { .. })
        ));
        // Sem o IDF nesta maquina: o passo do modelo.
        match propose(&modelo(), "idf.py", Some("/dev/ttyUSB0"), None) {
            Err(FlashError::MissingTool { tool, hint }) => {
                assert_eq!(tool, "ESP-IDF");
                assert!(hint.contains("não é ESP-IDF"), "{hint}");
            }
            outro => panic!("{outro:?}"),
        }

        let west = Engine::Zephyr {
            west: PathBuf::from("/x/west"),
        };
        let plan = propose(&modelo(), "west", Some("/dev/ttyACM0"), Some(&west)).unwrap();
        assert_eq!(plan.command.args, ["flash", "-d", "build"]);
        let p = plan.proposal;
        assert_eq!(p.command, "'/x/west' flash -d build");
        assert_eq!(p.warnings.len(), 1);
        assert!(
            propose(&modelo(), "west", None, Some(&west))
                .unwrap()
                .proposal
                .warnings
                .is_empty()
        );

        let pio = Engine::PlatformIo {
            pio: PathBuf::from("/x/pio"),
        };
        let plan = propose(&modelo(), "platformio", Some("/dev/ttyUSB0"), Some(&pio)).unwrap();
        assert_eq!(
            plan.command.args,
            ["run", "-t", "upload", "--upload-port", "/dev/ttyUSB0"]
        );
        let p = plan.proposal;
        assert_eq!(
            p.command,
            "'/x/pio' run -t upload --upload-port '/dev/ttyUSB0'"
        );
        assert_eq!(p.engine, "platformio");
        let p = propose(&modelo(), "pio", None, Some(&pio))
            .unwrap()
            .proposal;
        assert_eq!(p.command, "'/x/pio' run -t upload");
        assert!(matches!(
            propose(&modelo(), "avrdude", None, None),
            Err(FlashError::UnknownEngine { .. })
        ));
    }
}
