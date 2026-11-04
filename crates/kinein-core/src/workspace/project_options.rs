//! Validação das escolhas antes de criar arquivos e persistência local de runtime.

use super::{WorkspaceError, create::write_template};
use kinein_protocol::{WorkspaceProjectOptions, WorkspaceProjectTemplate};
use std::path::Path;

pub(super) fn validate(
    template: WorkspaceProjectTemplate,
    options: &WorkspaceProjectOptions,
) -> Result<(), WorkspaceError> {
    let invalid = |reason: &str| WorkspaceError::InvalidName {
        name: "options".to_owned(),
        reason: reason.to_owned(),
    };
    let executable = options
        .executable
        .as_deref()
        .filter(|value| !value.is_empty());
    let standard = options
        .standard
        .as_deref()
        .filter(|value| !value.is_empty());
    if template == WorkspaceProjectTemplate::Empty && (executable.is_some() || standard.is_some()) {
        return Err(invalid("pasta vazia não tem ambiente ou padrão"));
    }
    if let Some(standard) = standard {
        if template == WorkspaceProjectTemplate::CCmake {
            if !super::c_options::valid_standard(standard) {
                return Err(invalid(
                    "informe um número CMake positivo ou um token literal de dialeto C (ex.: 17, gnu17, iso9899:1990), sem espaços ou metacaracteres",
                ));
            }
        } else if template == WorkspaceProjectTemplate::Python
            || standard.len() > 6
            || !standard.bytes().all(|byte| byte.is_ascii_digit())
            || !standard.bytes().any(|byte| byte != b'0')
        {
            return Err(invalid(
                "padrão/edição deve ser um identificador decimal positivo de C++ ou Rust",
            ));
        }
    }
    if let Some(executable) = executable {
        let path = Path::new(executable);
        if !path.is_absolute() || !path.is_file() {
            return Err(invalid(
                "selecione o caminho absoluto de uma ferramenta instalada",
            ));
        }
        let version =
            crate::tools::environment_version(path).map_err(|message| invalid(&message))?;
        let compatible = match template {
            WorkspaceProjectTemplate::Python => version.starts_with("Python "),
            WorkspaceProjectTemplate::RustCargo => {
                // The rustup proxy directory is not a toolchain root. Pinning
                // it as a local toolchain would make future commands recurse.
                let proxy = path.parent().is_some_and(|parent| {
                    crate::platform::executable_names("rustup")
                        .iter()
                        .any(|name| parent.join(name).is_file())
                });
                let rustc = path.parent().map(|parent| {
                    parent.join(
                        crate::platform::executable_names("rustc")
                            .into_iter()
                            .next()
                            .unwrap_or_default(),
                    )
                });
                !proxy
                    && version.starts_with("cargo ")
                    && rustc.as_deref().is_some_and(|path| {
                        crate::tools::environment_version(path)
                            .is_ok_and(|version| version.starts_with("rustc "))
                    })
            }
            WorkspaceProjectTemplate::CCmake | WorkspaceProjectTemplate::CppCmake => {
                let version = version.to_ascii_lowercase();
                version.contains("clang")
                    || version.contains("gcc")
                    || version.contains("g++")
                    || version.contains("free software foundation")
            }
            WorkspaceProjectTemplate::Empty => false,
        };
        if !compatible {
            return Err(invalid("ferramenta incompatível com a linguagem escolhida"));
        }
        if template == WorkspaceProjectTemplate::CCmake {
            super::c_options::probe(path, standard).map_err(|message| invalid(&message))?;
        }
    }
    Ok(())
}

pub(super) fn pin_python(root: &Path, executable: &str) -> Result<(), WorkspaceError> {
    let directory = root.join(".kinein");
    std::fs::create_dir_all(&directory).map_err(|source| WorkspaceError::CreateDirectory {
        path: directory.display().to_string(),
        source,
    })?;
    write_template(
        &directory.join("python.json"),
        &serde_json::json!({"schemaVersion":1,"interpreter":executable}).to_string(),
    )
}

pub(super) fn pin_rust(root: &Path, executable: &str) -> Result<(), WorkspaceError> {
    let toolchain = Path::new(executable)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| WorkspaceError::InvalidName {
            name: executable.to_owned(),
            reason: "cargo deve pertencer a uma toolchain local".to_owned(),
        })?;
    let path = serde_json::to_string(&toolchain.display().to_string())
        .map_err(WorkspaceError::Serialize)?;
    write_template(
        &root.join("rust-toolchain.toml"),
        &format!("[toolchain]\npath = {path}\n"),
    )
}
