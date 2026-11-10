//! Os kits como o workspace os grava e quem pode cumprir cada papel.
//!
//! Abaixo do `toolchain` e da leitura dos presets, porque os dois os leem: o
//! `toolchain` explica e resolve cada kit lendo os presets, e os presets
//! perguntam ao `CMake` que o kit padrao fixa (C1-25, consolidacao C §18).
//! Sairam do `toolchain` em 2026-11-10 para essa leitura nao fechar um ciclo.

pub mod catalog;
pub mod store;

use std::path::{Path, PathBuf};

use kinein_protocol::ToolchainRole;

use crate::tools::{KNOWN_TOOLS, ToolDetector};

/// O valor gravado no papel e' uma instalacao por caminho absoluto?
///
/// So' o compilador C: e' o papel que tem receita para gravar caminho.
pub(crate) fn is_installation(role: ToolchainRole, value: &str) -> bool {
    role == ToolchainRole::CCompiler && Path::new(value).is_absolute()
}

/// `(resolvedPath, effectiveId)` de uma instalacao: ambos ausentes quando o
/// arquivo sumiu, para nenhum consumidor executar outro no lugar.
pub(crate) fn installed(path: &str) -> (Option<String>, Option<String>) {
    let file = Path::new(path);
    if !file.is_file() {
        return (None, None);
    }
    let id = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    (Some(path.to_owned()), id)
}

/// O executavel que o kit `kit` FIXA para `role`, achado como o core acha as
/// ferramentas. `None` sem fixacao, ou quando o fixado sumiu: quem executa cai
/// no `PATH`, como o configure (`program_for`). E' o `CMake` que responde
/// pelos presets do projeto, o do kit padrao (C1-25, autor em 2026-11-10).
#[must_use]
pub(crate) fn pinned_program(root: &Path, kit: &str, role: ToolchainRole) -> Option<PathBuf> {
    pinned_with(root, kit, role, &ToolDetector::from_environment())
}

/// Idem, com `detector` no lugar do que o core usa.
fn pinned_with(
    root: &Path,
    kit: &str,
    role: ToolchainRole,
    detector: &ToolDetector,
) -> Option<PathBuf> {
    let id = store::load(root)
        .remove(kit)?
        .selections
        .remove(role.as_str())?;
    if is_installation(role, &id) {
        return installed(&id).0.map(PathBuf::from);
    }
    let candidate = catalog::candidates_for(role)
        .iter()
        .find(|entry| entry.id == id)?;
    let spec = KNOWN_TOOLS
        .iter()
        .find(|spec| spec.id == candidate.tool_id)?;
    detector.find_in_path(spec.binary).or_else(|| {
        spec.alternative_binary
            .and_then(|binary| detector.find_in_path(binary))
    })
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };

    use kinein_protocol::ToolchainRole;

    use super::{
        pinned_with,
        store::{Kit, save},
    };
    use crate::tools::ToolDetector;

    fn kit(selections: &[(&str, &str)]) -> Kit {
        Kit {
            selections: selections
                .iter()
                .map(|(role, id)| ((*role).to_owned(), (*id).to_owned()))
                .collect(),
            ..Kit::default()
        }
    }

    fn pin(root: &Path, kits: &[(&str, Kit)]) {
        let kits: BTreeMap<String, Kit> = kits
            .iter()
            .map(|(name, kit)| ((*name).to_owned(), kit.clone()))
            .collect();
        save(root, &kits).unwrap();
    }

    /// Cada ramo da busca: o kit e o papel pela chave, a instalacao pelo
    /// caminho sem procurar, o binario antes do alternativo, e nada quando o
    /// fixado sumiu ou o id nao esta no catalogo.
    #[test]
    fn a_kit_pins_the_executable_of_its_role_as_the_core_finds_it() {
        let root = std::env::temp_dir()
            .join("kinein-kits")
            .join(format!("{}-pins ação", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(root.join("opt")).unwrap();
        let cmake = crate::write_executable(bin.join("cmake"), "#!/bin/sh\n");
        let alternative = crate::write_executable(bin.join("gmake"), "#!/bin/sh\n");
        let installed = crate::write_executable(root.join("opt/cc-12"), "#!/bin/sh\n");
        let detector = ToolDetector::with_search_path(bin.as_os_str());
        let found = |kit: &str, role| pinned_with(&root, kit, role, &detector);
        let installation = installed.to_str().unwrap();
        pin(
            &root,
            &[
                (
                    "",
                    kit(&[
                        ("cmake", "cmake"),
                        ("generator", "Unix Makefiles"),
                        ("cCompiler", installation),
                    ]),
                ),
                ("other", kit(&[("cxxCompiler", "g++")])),
            ],
        );
        assert_eq!(found("", ToolchainRole::Cmake), Some(cmake));
        assert_eq!(found("", ToolchainRole::Generator), Some(alternative));
        assert_eq!(found("", ToolchainRole::CCompiler), Some(installed.clone()));
        assert_eq!(found("", ToolchainRole::CxxCompiler), None);
        assert_eq!(found("other", ToolchainRole::Cmake), None);
        assert_eq!(found("nope", ToolchainRole::Cmake), None);

        let make = crate::write_executable(bin.join("make"), "#!/bin/sh\n");
        assert_eq!(found("", ToolchainRole::Generator), Some(make));
        std::fs::remove_file(&installed).unwrap();
        assert_eq!(found("", ToolchainRole::CCompiler), None::<PathBuf>);
        std::fs::remove_file(bin.join("cmake")).unwrap();
        assert_eq!(found("", ToolchainRole::Cmake), None);

        pin(&root, &[("", kit(&[("cmake", "cmake-99")]))]);
        crate::write_executable(bin.join("cmake"), "#!/bin/sh\n");
        assert_eq!(found("", ToolchainRole::Cmake), None);
    }
}
