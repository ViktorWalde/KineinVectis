//! Servico `CMake`: configure explicito, presets, targets via file-api e status.
//!
//! O diretorio de build e UNICO e imutavel (`<root>/.kinein/build`) — o mesmo
//! usado por `build.run` e `run.start`; presets nao mudam o diretorio (o `-B`
//! explicito tem precedencia). Targets vem da resposta `codemodel-v2` do
//! file-api oficial do `CMake`, nunca de parser proprio de `CMakeLists.txt`.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{CmakePresetInfo, CmakeTargetInfo, ToolchainRole};
use serde_json::Value;

/// Maximo de targets repassados a UI por request.
const MAX_TARGETS: usize = 200;

/// Diretorio de build canonico do workspace.
#[must_use]
pub fn build_dir(root: &Path) -> PathBuf {
    root.join(".kinein").join("build")
}

/// Estado do configure para `cmake.status`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CmakeStatus {
    /// `CMakeCache.txt` existe no build dir.
    pub configured: bool,
    /// `compile_commands.json` existe no build dir.
    pub has_compile_commands: bool,
    /// Build dir canonico.
    pub build_dir: PathBuf,
}

/// Le o estado atual por stat dos artefatos do configure.
#[must_use]
pub fn status(root: &Path) -> CmakeStatus {
    let build_dir = build_dir(root);
    CmakeStatus {
        configured: build_dir.join("CMakeCache.txt").is_file(),
        has_compile_commands: build_dir.join("compile_commands.json").is_file(),
        build_dir,
    }
}

/// Escreve a query `codemodel-v2` do file-api antes do configure, para o
/// `CMake` responder com os targets em `.cmake/api/v1/reply`.
pub fn write_file_api_query(root: &Path) -> io::Result<()> {
    let query = build_dir(root)
        .join(".cmake")
        .join("api")
        .join("v1")
        .join("query");
    fs::create_dir_all(&query)?;
    fs::write(query.join("codemodel-v2"), "")
}

/// Monta o comando de configure com a CDB exportada e o `-B` fixo.
///
/// `toolchain` e a escolha do usuario (roadmap 30, etapa 5). Ela entra por
/// dois caminhos: o EXECUTAVEL do `cmake` (quando fixado) e os argumentos
/// `-G`/`-DCMAKE_*_COMPILER`. Sem escolha nenhuma, o comando sai byte a byte
/// como saia antes — o padrao continua sendo o `PATH`.
#[must_use]
pub fn configure_command(
    root: &Path,
    preset: Option<&str>,
    toolchain: &crate::toolchain::Toolchain,
) -> Command {
    let programa = toolchain
        .program_for(ToolchainRole::Cmake)
        .unwrap_or_else(|| PathBuf::from("cmake"));
    let mut command = Command::new(programa);
    if let Some(preset) = preset {
        command.arg("--preset").arg(preset);
    }
    command.arg("-S").arg(root).arg("-B").arg(build_dir(root));
    command.arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON");
    // Depois do `-D` fixo e antes de nada: um preset que ja escolhe gerador
    // conflita com um `-G` explicito, e o CMake reclama em vez de adivinhar —
    // que e o comportamento certo, e a mensagem dele nomeia o conflito.
    command.args(toolchain.cmake_arguments());
    command
}

/// Lista os configure presets nao ocultos de `CMakePresets.json` e
/// `CMakeUserPresets.json`, na ordem dos arquivos.
pub fn list_presets(root: &Path) -> Result<Vec<CmakePresetInfo>, String> {
    let mut presets = Vec::new();
    for file in ["CMakePresets.json", "CMakeUserPresets.json"] {
        let path = root.join(file);
        if !path.is_file() {
            continue;
        }
        let body =
            fs::read_to_string(&path).map_err(|error| format!("falha lendo {file}: {error}"))?;
        let value: Value =
            serde_json::from_str(&body).map_err(|error| format!("{file} invalido: {error}"))?;
        let Some(items) = value.get("configurePresets").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            if item.get("hidden").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                continue;
            };
            presets.push(CmakePresetInfo {
                name: name.to_owned(),
                display_name: item
                    .get("displayName")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
    }
    Ok(presets)
}

/// Le os targets do reply `codemodel-v2` do ultimo configure.
///
/// Sem reply (nunca configurado com a query) retorna vazio; a UI explica.
#[must_use]
pub fn list_targets(root: &Path) -> Vec<CmakeTargetInfo> {
    let reply_dir = build_dir(root)
        .join(".cmake")
        .join("api")
        .join("v1")
        .join("reply");
    let Ok(entries) = fs::read_dir(&reply_dir) else {
        return Vec::new();
    };

    let mut codemodel: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("codemodel-v2")
            && std::path::Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            codemodel = Some(entry.path());
            break;
        }
    }
    let Some(codemodel) = codemodel else {
        return Vec::new();
    };
    let Some(model) = read_json(&codemodel) else {
        return Vec::new();
    };

    let mut targets = Vec::new();
    let configurations = model
        .pointer("/configurations/0/targets")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for target in configurations {
        if targets.len() >= MAX_TARGETS {
            break;
        }
        let Some(name) = target.get("name").and_then(Value::as_str) else {
            continue;
        };
        let kind = target
            .get("jsonFile")
            .and_then(Value::as_str)
            .and_then(|file| read_json(&reply_dir.join(file)))
            .and_then(|detail| {
                detail
                    .get("type")
                    .and_then(Value::as_str)
                    .map(target_kind_name)
            })
            .unwrap_or_else(|| "unknown".to_owned());
        targets.push(CmakeTargetInfo {
            name: name.to_owned(),
            kind,
        });
    }
    targets
}

fn read_json(path: &Path) -> Option<Value> {
    let body = fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

/// Converte o `type` do file-api (`EXECUTABLE`, `STATIC_LIBRARY`, ...) para
/// o kind camelCase do protocolo (`executable`, `staticLibrary`, ...).
fn target_kind_name(raw: &str) -> String {
    let mut kind = String::with_capacity(raw.len());
    let mut uppercase_next = false;
    for ch in raw.chars() {
        if ch == '_' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            kind.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            kind.extend(ch.to_lowercase());
        }
    }
    kind
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{configure_command, list_presets, list_targets, status, write_file_api_query};
    use crate::toolchain::Toolchain;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cmake-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn status_reflects_configure_artifacts() {
        let root = temp_root("status");
        let before = status(&root);
        assert!(!before.configured);
        assert!(!before.has_compile_commands);

        std::fs::create_dir_all(before.build_dir.clone()).unwrap();
        std::fs::write(before.build_dir.join("CMakeCache.txt"), "#\n").unwrap();
        std::fs::write(before.build_dir.join("compile_commands.json"), "[]\n").unwrap();
        let after = status(&root);
        assert!(after.configured);
        assert!(after.has_compile_commands);
    }

    #[test]
    fn configure_command_pins_build_dir_and_exports_cdb() {
        let root = temp_root("command");
        let command = configure_command(&root, Some("dev"), &Toolchain::resolve(&root, &[]));
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(arguments[0], "--preset");
        assert_eq!(arguments[1], "dev");
        assert!(arguments.contains(&"-S".to_owned()));
        assert!(arguments.contains(&"-DCMAKE_EXPORT_COMPILE_COMMANDS=ON".to_owned()));
        let build_index = arguments.iter().position(|arg| arg == "-B").unwrap();
        assert!(arguments[build_index + 1].ends_with(".kinein/build"));
    }

    #[test]
    fn file_api_query_is_written_inside_build_dir() {
        let root = temp_root("query");
        write_file_api_query(&root).unwrap();
        assert!(
            root.join(".kinein/build/.cmake/api/v1/query/codemodel-v2")
                .is_file()
        );
    }

    #[test]
    fn presets_merge_both_files_and_skip_hidden() {
        let root = temp_root("presets");
        std::fs::write(
            root.join("CMakePresets.json"),
            r#"{ "version": 6, "configurePresets": [
                { "name": "base", "hidden": true },
                { "name": "release", "displayName": "Release" }
            ] }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("CMakeUserPresets.json"),
            r#"{ "version": 6, "configurePresets": [ { "name": "dev-local" } ] }"#,
        )
        .unwrap();

        let presets = list_presets(&root).unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, "release");
        assert_eq!(presets[0].display_name.as_deref(), Some("Release"));
        assert_eq!(presets[1].name, "dev-local");

        std::fs::write(root.join("CMakeUserPresets.json"), "nao é json").unwrap();
        assert!(list_presets(&root).is_err());
    }

    #[test]
    fn targets_come_from_the_file_api_reply() {
        let root = temp_root("targets");
        let reply = root.join(".kinein/build/.cmake/api/v1/reply");
        std::fs::create_dir_all(&reply).unwrap();
        std::fs::write(
            reply.join("target-demo.json"),
            r#"{ "type": "EXECUTABLE" }"#,
        )
        .unwrap();
        std::fs::write(
            reply.join("target-util.json"),
            r#"{ "type": "STATIC_LIBRARY" }"#,
        )
        .unwrap();
        std::fs::write(
            reply.join("codemodel-v2-abc.json"),
            r#"{ "configurations": [ { "targets": [
                { "name": "demo", "jsonFile": "target-demo.json" },
                { "name": "util", "jsonFile": "target-util.json" }
            ] } ] }"#,
        )
        .unwrap();

        let targets = list_targets(&root);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].name, "demo");
        assert_eq!(targets[0].kind, "executable");
        assert_eq!(targets[1].kind, "staticLibrary");

        let empty = temp_root("targets-empty");
        assert!(list_targets(&empty).is_empty());
    }
}
