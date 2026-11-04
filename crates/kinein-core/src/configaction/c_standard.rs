//! Padrao C exigido por alvo, usando o plano/snapshot das acoes existentes.

use std::{collections::BTreeMap, path::Path};

use super::{
    c_standard_source::{commands, manual, validate},
    cmakelists::{CMAKELISTS, append_block, one_argument},
    error::ConfigActionError,
    optional_param,
    plan::{ActionPlan, PlannedFile, read_required},
    required_param,
};

pub(super) const SUGGESTED_STANDARDS: &[&str] = &["90", "99", "11", "17", "23"];

fn numeric_standard(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_digit() && c != '0')
        && value.bytes().all(|c| c.is_ascii_digit())
}

pub(super) fn plan(
    root: &Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let target = one_argument(required_param(params, "target")?, "target")?;
    if !target
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '+' | '-'))
    {
        return Err(manual("nome de alvo fora do recorte literal"));
    }
    let standard = optional_param(params, "standard").unwrap_or("23").trim();
    if !numeric_standard(standard) {
        return Err(ConfigActionError::InvalidParam {
            name: "standard",
            reason: "informe um identificador decimal positivo de C_STANDARD; o CMake valida o suporte (ex.: 90, 99, 11, 17, 23)".to_owned(),
        });
    }
    let extensions = optional_param(params, "extensions").unwrap_or("OFF").trim();
    if !matches!(extensions, "OFF" | "ON") {
        return Err(ConfigActionError::InvalidParam {
            name: "extensions",
            reason: "use OFF (ISO) ou ON (extensões do compilador)".to_owned(),
        });
    }
    let before = read_required(root, CMAKELISTS)?;
    let original_commands = commands(&before)?;
    let marker = format!("# Kinein: padrão C do alvo {target}\n");
    let mut existing = None;
    for command in &original_commands {
        if let [
            name,
            properties,
            standard_key,
            old_standard,
            required_key,
            required,
            extensions_key,
            old_extensions,
        ] = command.args.as_slice()
            && command.name == "set_target_properties"
            && name == &target
            && properties == "PROPERTIES"
            && standard_key == "C_STANDARD"
            && numeric_standard(old_standard)
            && required_key == "C_STANDARD_REQUIRED"
            && required == "ON"
            && extensions_key == "C_EXTENSIONS"
            && matches!(old_extensions.as_str(), "OFF" | "ON")
        {
            let block = block(&target, old_standard, old_extensions);
            if before.contains(&block) {
                if existing.is_some() || before.matches(&block).count() != 1 {
                    return Err(manual("bloco de padrão C duplicado"));
                }
                existing = Some(block);
            }
        }
    }
    let remaining = existing
        .as_ref()
        .map_or_else(|| before.clone(), |old| before.replacen(old, "", 1));
    if remaining.contains(&marker) {
        return Err(manual("bloco de padrão C alterado manualmente"));
    }
    validate(&commands(&remaining)?, &target)?;
    let block = block(&target, standard, extensions);
    let after = existing.as_ref().map_or_else(
        || append_block(&before, &block),
        |old| before.replacen(old, &block, 1),
    );
    Ok(ActionPlan::edit(
        format!("Exige C{standard} em {target}, extensões {extensions}"),
        PlannedFile { path: CMAKELISTS.to_owned(), before: Some(before), after },
    ).with_note(format!("Substitui os defaults C herdados somente em {target}; outros alvos e presets são preservados."))
     .with_note("A edição ainda precisa de Configurar/build. Confira o padrão efetivo no contexto; a toolchain pode recusar o padrão exigido."))
}

/// Bloco canonico compartilhado pela receita e pelo template de projeto C.
pub(super) fn block(target: &str, standard: &str, extensions: &str) -> String {
    format!(
        "# Kinein: padrão C do alvo {target}\nset_target_properties({target} PROPERTIES\n    C_STANDARD {standard}\n    C_STANDARD_REQUIRED ON\n    C_EXTENSIONS {extensions}\n)\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "cmake_minimum_required(VERSION 3.24)\nproject(sample C CXX)\nset(CMAKE_C_STANDARD 11)\nset(CMAKE_C_EXTENSIONS ON)\nadd_executable(app main.c)\nadd_library(legacy STATIC legacy.c)\nset_target_properties(legacy PROPERTIES C_STANDARD 99 C_EXTENSIONS ON)\nadd_executable(cpp main.cpp)\n";

    struct Project(std::path::PathBuf);
    impl Project {
        fn new(name: &str, source: &str) -> Self {
            let root = std::env::temp_dir()
                .join(format!("kinein-c-standard-{}-{name}", std::process::id()));
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join(CMAKELISTS), source).unwrap();
            Self(root)
        }
        fn plan(&self, target: &str, standard: &str) -> Result<ActionPlan, ConfigActionError> {
            plan(
                &self.0,
                &BTreeMap::from([
                    ("target".to_owned(), target.to_owned()),
                    ("standard".to_owned(), standard.to_owned()),
                ]),
            )
        }
    }
    impl Drop for Project {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn c_standard_preserves_defaults_manual_other_targets_and_cpp_parameters() {
        let p = Project::new("preserve", SOURCE);
        let plan = p.plan("app", "23").unwrap();
        assert!(plan.files[0].after.starts_with(SOURCE));
        assert_eq!(
            std::fs::read_to_string(p.0.join(CMAKELISTS)).unwrap(),
            SOURCE
        );
        let actions = super::super::list(
            &p.0,
            &[kinein_protocol::BuildSystem::Cmake],
            false,
            &super::super::Inputs::default(),
        )
        .actions;
        for (id, values) in [
            ("cmake.setCStandard", vec!["90", "99", "11", "17", "23"]),
            (
                "cmake.setCxxStandard",
                vec!["11", "14", "17", "20", "23", "26"],
            ),
        ] {
            let action = actions.iter().find(|a| a.id == id).unwrap();
            let param = action.params.iter().find(|p| p.name == "standard").unwrap();
            assert_eq!(param.suggestions, values);
            assert_eq!(param.description.contains("C++"), id.contains("Cxx"));
        }
        assert!(p.plan("legacy", "23").is_err());
        assert!(p.plan("cpp", "23").is_err());
    }

    #[test]
    fn c_standard_updates_only_its_exact_block() {
        let p = Project::new("update", SOURCE);
        let first = p.plan("app", "23").unwrap();
        std::fs::write(p.0.join(CMAKELISTS), &first.files[0].after).unwrap();
        let second = p.plan("app", "17").unwrap();
        assert_eq!(
            second.files[0].after.matches("# Kinein: padrão C").count(),
            1
        );
        assert!(second.files[0].after.contains("C_STANDARD 17"));
        let manual = first.files[0]
            .after
            .replace("C_STANDARD_REQUIRED ON", "C_STANDARD_REQUIRED OFF");
        std::fs::write(p.0.join(CMAKELISTS), &manual).unwrap();
        assert!(p.plan("app", "11").is_err());
        assert_eq!(
            std::fs::read_to_string(p.0.join(CMAKELISTS)).unwrap(),
            manual
        );
    }

    #[test]
    fn c_standard_future_values_can_be_previewed_and_replaced_without_a_new_enum() {
        let p = Project::new("future", SOURCE);
        let future = p.plan("app", "2099").unwrap();
        assert!(future.files[0].after.contains("C_STANDARD 2099"));
        std::fs::write(p.0.join(CMAKELISTS), &future.files[0].after).unwrap();
        let updated = p.plan("app", "26").unwrap();
        assert_eq!(
            updated.files[0].after.matches("# Kinein: padrão C").count(),
            1
        );
        assert!(!updated.files[0].after.contains("C_STANDARD 2099"));
        assert!(updated.files[0].after.contains("C_STANDARD 26"));
    }

    #[test]
    fn c_standard_refuses_manual_dialect_properties_and_dynamic_sources() {
        for (index, extra) in [
            "set_target_properties(app legacy PROPERTIES C_STANDARD 11)",
            "SET_TARGET_PROPERTIES (\"app\" PROPERTIES C_EXTENSIONS ON)",
            "set_property(TARGET app PROPERTY C_STANDARD_REQUIRED OFF)",
            "set_property(TARGET ${selected} PROPERTY C_STANDARD 11)",
            "target_compile_options(app PRIVATE -std=gnu11)",
            "target_compile_options(app PRIVATE -ansi)",
            "set_target_properties(app PROPERTIES COMPILE_FLAGS -std=c11)",
            "target_compile_options(app PRIVATE ${sdk_flags})",
            "set_source_files_properties(main.c PROPERTIES LANGUAGE CXX)",
            "target_compile_features(app PRIVATE c_std_23)",
            "set(CMAKE_C_FLAGS \"-Wall -std=c17\")",
            "add_compile_options(-std=c99)",
            "if(TRUE)\nset_property(TARGET app PROPERTY C_STANDARD 17)\nendif()",
            "include(sdk.cmake)",
            "add_subdirectory(dependency)",
            "custom_setup(app)",
            "#[=[ bracket comment ]=]",
        ]
        .iter()
        .enumerate()
        {
            let source = format!("{SOURCE}\n{extra}\n");
            let p = Project::new(&format!("manual-{index}"), &source);
            let error = p.plan("app", "23").unwrap_err();
            assert!(
                error.to_string().contains("manualmente"),
                "{extra}: {error}"
            );
            assert_eq!(
                std::fs::read_to_string(p.0.join(CMAKELISTS)).unwrap(),
                source
            );
        }
    }

    #[test]
    fn c_standard_comments_do_not_define_properties_and_text_is_not_a_managed_command() {
        let p = Project::new(
            "comments",
            &format!("{SOURCE}\n# set_target_properties(app PROPERTIES C_STANDARD 99)\n"),
        );
        assert!(p.plan("app", "23").is_ok());
        let fake = format!("{SOURCE}\nset(EXAMPLE \"{}\")\n", block("app", "23", "OFF"));
        std::fs::write(p.0.join(CMAKELISTS), fake).unwrap();
        assert!(p.plan("app", "17").is_err());
    }

    #[test]
    fn c_standard_validates_numeric_values_and_literal_target() {
        let p = Project::new("params", SOURCE);
        for standard in [
            "0",
            "-1",
            "02",
            "2y",
            "gnu23",
            "23)\nmessage(FATAL_ERROR bad)",
        ] {
            assert!(p.plan("app", standard).is_err());
        }
        for target in ["missing", "${app}", "app;legacy", "app/legacy"] {
            assert!(p.plan(target, "23").is_err());
        }
        for standard in SUGGESTED_STANDARDS.iter().copied().chain(["26", "2099"]) {
            assert!(p.plan("app", standard).is_ok());
        }
        let params = BTreeMap::from([
            ("target".to_owned(), "app".to_owned()),
            ("extensions".to_owned(), "maybe".to_owned()),
        ]);
        assert!(plan(&p.0, &params).is_err());
        let default = plan(
            &p.0,
            &BTreeMap::from([("target".to_owned(), "app".to_owned())]),
        )
        .unwrap();
        assert!(
            default.files[0]
                .after
                .contains("C_STANDARD 23\n    C_STANDARD_REQUIRED ON\n    C_EXTENSIONS OFF")
        );
    }
}
