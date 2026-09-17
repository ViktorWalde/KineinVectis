//! O MOTOR de build de um projeto embarcado (bloco E do `roadmaps/41`,
//! 2026-09-17): quem compila quando o framework tem um wrapper proprio.
//!
//! O `project.model` ja' reconhece o framework (pelos marcadores) e diz o que
//! ele exige nesta maquina (`sdks`). Esta unidade decide, a partir DISSO e
//! sem tocar o disco de novo, com que motor o `build.run` roda:
//!
//! ```text
//! PlatformIO   `pio run` na raiz (platformio.ini e' intencao explicita: vence
//!              ate' um CMakeLists de framework=espidf)
//! ESP-IDF      `idf.py build`, DENTRO do ambiente ativado: o "Get Started"
//!              (docs.espressif.com, lido em 2026-09-17) manda ativar antes de
//!              qualquer idf.py — v6 pelo EIM (`source ~/.espressif/tools/
//!              activate_idf_<versao>.sh`), legado pelo `. $IDF_PATH/export.sh`.
//!              Nao ha' idf.py sem isso: o script poe o Python do IDF e as
//!              toolchains no PATH. A IDE roda `bash -c '. <ativacao> && exec
//!              idf.py …'` — o mesmo que o usuario faria no terminal, sem
//!              exigir shell interativo
//! Zephyr       `west build -d build [-b <placa>]` na raiz; a placa vem do
//!              `build/CMakeCache.txt` (CACHED_BOARD) ou do `west config
//!              build.board` — a forma da doc do west, nunca um campo inventado
//! pico-sdk     o CMake de sempre com `-DPICO_SDK_PATH=<sdk>` (pico_sdk_import.cmake
//!              le a variavel; o README do pico-sdk a documenta)
//! ```
//!
//! O que falta e' dito antes de rodar: framework reconhecido e ferramenta
//! ausente e' [`EngineError::Missing`] com o passo do proprio modelo.

use std::path::{Path, PathBuf};

use kinein_protocol::{Framework, ProjectModel, SdkRequirement};

/// Com que o projeto compila, alem de cargo/cmake/make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Engine {
    /// `idf.py`, ativado por este script (`export.sh` ou `activate_idf_*.sh`).
    EspIdf {
        /// O script que se faz `source` antes de qualquer `idf.py`.
        activation: PathBuf,
    },
    /// `west`, o executavel detectado.
    Zephyr {
        /// O `west` desta maquina.
        west: PathBuf,
    },
    /// `pio`, o executavel detectado.
    PlatformIo {
        /// O `pio` desta maquina.
        pio: PathBuf,
    },
    /// O `CMake` nativo com o SDK do Pico.
    PicoSdk {
        /// `PICO_SDK_PATH`.
        sdk: PathBuf,
    },
}

impl Engine {
    /// O nome que a tela e o job mostram.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EspIdf { .. } => "idf.py",
            Self::Zephyr { .. } => "west",
            Self::PlatformIo { .. } => "pio",
            Self::PicoSdk { .. } => "cmake (pico-sdk)",
        }
    }

    /// Os `-D` que o `CMake` nativo recebe por causa do framework (so' o Pico).
    #[must_use]
    pub fn cmake_extra_args(&self) -> Vec<String> {
        match self {
            Self::PicoSdk { sdk } => vec![format!("-DPICO_SDK_PATH={}", sdk.display())],
            _ => Vec::new(),
        }
    }
}

/// Framework reconhecido, ferramenta ausente: o build nao roda, e diz o passo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    /// O framework (`ESP-IDF`).
    pub framework: String,
    /// O que falta e como obter, nas palavras do modelo.
    pub message: String,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.framework, self.message)
    }
}

/// O script que ativa o ESP-IDF nesta maquina.
///
/// Nas duas formas que a doc documenta: o `export.sh` da arvore do IDF
/// (`IDF_PATH`/`~/esp/esp-idf`, a exigencia `esp-idf` do modelo) ou, pelo
/// EIM (v6), o mais novo `~/.espressif/tools/activate_idf_*.sh`.
#[must_use]
pub fn esp_idf_activation(sdks: &[SdkRequirement], home: Option<&Path>) -> Option<PathBuf> {
    let export = sdks
        .iter()
        .find(|s| s.id == "esp-idf")
        .and_then(|s| s.path.as_deref())
        .map(|p| Path::new(p).join("export.sh"))
        .filter(|p| p.is_file());
    if export.is_some() {
        return export;
    }
    let tools = home?.join(".espressif").join("tools");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(tools)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                    n.starts_with("activate_idf_")
                        && std::path::Path::new(n)
                            .extension()
                            .is_some_and(|x| x == "sh")
                })
        })
        .collect();
    // O nome carrega a versao (`activate_idf_v5.4.2.sh`): o maior e' o mais novo.
    scripts.sort();
    scripts.pop()
}

/// O que `bash -c` executa para rodar um `idf.py` no ambiente ativado.
///
/// `$1` e' o script de ativacao, o resto sao os argumentos do `idf.py`. A
/// saida da ativacao (o banner do export.sh) vai para /dev/null; o erro
/// dela, nao.
pub const IDF_WRAPPER: &str = ". \"$1\" >/dev/null || { echo \"nao consegui ativar o ESP-IDF com $1\" >&2; exit 1; }; shift; exec idf.py \"$@\"";

/// Os argumentos de `bash` para `idf.py <args>` com `activation`.
#[must_use]
pub fn idf_command_args(activation: &Path, args: &[&str]) -> Vec<String> {
    let mut lista = vec![
        "-c".to_owned(),
        IDF_WRAPPER.to_owned(),
        "idf".to_owned(),
        activation.display().to_string(),
    ];
    lista.extend(args.iter().map(|a| (*a).to_owned()));
    lista
}

/// O motor deste projeto, pelo modelo. `Ok(None)` = cargo/cmake/make de
/// sempre; `Err` = o framework esta' la' e a ferramenta nao.
///
/// # Errors
///
/// [`EngineError`] com o passo do modelo quando o `pio`, o `west` ou a
/// ativacao do ESP-IDF nao estao nesta maquina.
pub fn resolve(model: &ProjectModel, home: Option<&Path>) -> Result<Option<Engine>, EngineError> {
    let tem = |f: Framework| model.frameworks.iter().any(|i| i.framework == f);
    let sdk = |id: &str| model.sdks.iter().find(|s| s.id == id);
    let binario = |id: &str, framework: &str| -> Result<PathBuf, EngineError> {
        sdk(id)
            .and_then(|s| s.path.as_deref().map(PathBuf::from))
            .ok_or_else(|| EngineError {
                framework: framework.to_owned(),
                message: sdk(id)
                    .and_then(|s| s.hint.clone())
                    .unwrap_or_else(|| format!("`{id}` nao esta' nesta maquina")),
            })
    };
    if tem(Framework::PlatformIo) {
        return Ok(Some(Engine::PlatformIo {
            pio: binario("pio", "PlatformIO")?,
        }));
    }
    if tem(Framework::EspIdf) {
        let activation = esp_idf_activation(&model.sdks, home).ok_or_else(|| EngineError {
            framework: "ESP-IDF".to_owned(),
            message: sdk("esp-idf")
                .and_then(|s| s.hint.clone())
                .unwrap_or_default()
                + " — ou instale pelo EIM (`eim install`), que deixa o activate_idf_<versao>.sh em ~/.espressif/tools",
        })?;
        return Ok(Some(Engine::EspIdf { activation }));
    }
    if tem(Framework::Zephyr) {
        return Ok(Some(Engine::Zephyr {
            west: binario("west", "Zephyr")?,
        }));
    }
    if tem(Framework::PicoSdk)
        && let Some(sdk) = sdk("pico-sdk").and_then(|s| s.path.as_deref())
    {
        return Ok(Some(Engine::PicoSdk {
            sdk: PathBuf::from(sdk),
        }));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::{Framework, FrameworkInfo, ProjectModel, SdkRequirement};

    use super::{Engine, esp_idf_activation, idf_command_args, resolve};

    fn req(id: &str, path: Option<&str>) -> SdkRequirement {
        SdkRequirement {
            id: id.to_owned(),
            label: id.to_owned(),
            env: None,
            path: path.map(str::to_owned),
            found: path.is_some(),
            hint: Some(format!("instale {id}")),
        }
    }

    fn modelo(frameworks: &[Framework], sdks: Vec<SdkRequirement>) -> ProjectModel {
        ProjectModel {
            root: "/p".to_owned(),
            embedded: true,
            frameworks: frameworks
                .iter()
                .map(|f| FrameworkInfo {
                    framework: *f,
                    evidence: "x".to_owned(),
                    detail: None,
                })
                .collect(),
            sdks,
            artifacts: kinein_protocol::ProjectArtifacts::default(),
            target: kinein_protocol::TargetModel::default(),
            hints: Vec::new(),
        }
    }

    #[test]
    fn the_engine_follows_the_framework_and_names_what_is_missing() {
        assert_eq!(resolve(&modelo(&[], vec![]), None), Ok(None));
        assert_eq!(
            resolve(
                &modelo(&[Framework::PlatformIo], vec![req("pio", Some("/x/pio"))]),
                None
            ),
            Ok(Some(Engine::PlatformIo {
                pio: PathBuf::from("/x/pio")
            }))
        );
        let erro = resolve(
            &modelo(&[Framework::PlatformIo], vec![req("pio", None)]),
            None,
        )
        .unwrap_err();
        assert_eq!(erro.framework, "PlatformIO");
        assert!(erro.message.contains("instale pio"));
        // O PlatformIO vence o ESP-IDF (framework=espidf).
        assert!(matches!(
            resolve(
                &modelo(
                    &[Framework::EspIdf, Framework::PlatformIo],
                    vec![req("pio", Some("/x/pio"))]
                ),
                None
            ),
            Ok(Some(Engine::PlatformIo { .. }))
        ));
        assert_eq!(
            resolve(
                &modelo(&[Framework::Zephyr], vec![req("west", Some("/x/west"))]),
                None
            ),
            Ok(Some(Engine::Zephyr {
                west: PathBuf::from("/x/west")
            }))
        );
        let pico = resolve(
            &modelo(&[Framework::PicoSdk], vec![req("pico-sdk", Some("/sdk"))]),
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(pico.cmake_extra_args(), ["-DPICO_SDK_PATH=/sdk"]);
        // Sem o SDK do Pico o CMake roda como sempre (pode buscar do git).
        assert_eq!(
            resolve(
                &modelo(&[Framework::PicoSdk], vec![req("pico-sdk", None)]),
                None
            ),
            Ok(None)
        );
        // ESP-IDF sem ativacao: erro com o passo E o EIM.
        let erro = resolve(
            &modelo(&[Framework::EspIdf], vec![req("esp-idf", None)]),
            None,
        )
        .unwrap_err();
        assert!(erro.message.contains("eim install"), "{erro}");
    }

    #[test]
    fn the_idf_activation_is_the_export_script_or_the_newest_eim_one() {
        let raiz = std::env::temp_dir().join(format!("kinein-engine-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        let idf = raiz.join("esp/esp-idf");
        std::fs::create_dir_all(&idf).unwrap();
        assert_eq!(
            esp_idf_activation(&[req("esp-idf", Some(idf.to_str().unwrap()))], None),
            None
        );
        std::fs::write(idf.join("export.sh"), "").unwrap();
        assert_eq!(
            esp_idf_activation(&[req("esp-idf", Some(idf.to_str().unwrap()))], None),
            Some(idf.join("export.sh"))
        );
        let tools = raiz.join("home/.espressif/tools");
        std::fs::create_dir_all(&tools).unwrap();
        std::fs::write(tools.join("activate_idf_v5.4.2.sh"), "").unwrap();
        std::fs::write(tools.join("activate_idf_v6.1.0.sh"), "").unwrap();
        std::fs::write(tools.join("outro.sh"), "").unwrap();
        assert_eq!(
            esp_idf_activation(&[req("esp-idf", None)], Some(&raiz.join("home"))),
            Some(tools.join("activate_idf_v6.1.0.sh"))
        );
        let args = idf_command_args(Path::new("/a/export.sh"), &["-p", "/dev/ttyUSB0", "flash"]);
        assert_eq!(args[0], "-c");
        assert!(args[1].contains("exec idf.py"));
        assert_eq!(
            &args[2..],
            ["idf", "/a/export.sh", "-p", "/dev/ttyUSB0", "flash"]
        );
        let _ = std::fs::remove_dir_all(&raiz);
    }
}
