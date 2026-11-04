//! Arquivos dos templates C/C++ no dono existente de criacao de projetos.
use super::{WorkspaceError, create::write_template};
use kinein_protocol::WorkspaceProjectOptions;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Language {
    C,
    Cpp,
}

pub(super) fn write(
    root: &Path,
    name: &str,
    language: Language,
    options: &WorkspaceProjectOptions,
) -> Result<(), WorkspaceError> {
    let src = root.join("src");
    write_template(
        &root.join("CMakeLists.txt"),
        &cmakelists(name, language, options),
    )?;
    write_template(
        &root.join("CMakePresets.json"),
        "{\n  \"version\": 6,\n  \"configurePresets\": [\n    {\n      \"name\": \"debug\",\n      \"displayName\": \"Debug\",\n      \"generator\": \"Ninja\",\n      \"binaryDir\": \"${sourceDir}/build/debug\",\n      \"cacheVariables\": {\n        \"CMAKE_BUILD_TYPE\": \"Debug\",\n        \"CMAKE_EXPORT_COMPILE_COMMANDS\": \"ON\"\n      }\n    },\n    {\n      \"name\": \"release\",\n      \"displayName\": \"Release\",\n      \"generator\": \"Ninja\",\n      \"binaryDir\": \"${sourceDir}/build/release\",\n      \"cacheVariables\": {\n        \"CMAKE_BUILD_TYPE\": \"Release\",\n        \"CMAKE_EXPORT_COMPILE_COMMANDS\": \"ON\"\n      }\n    }\n  ],\n  \"buildPresets\": [\n    {\n      \"name\": \"debug\",\n      \"configurePreset\": \"debug\"\n    },\n    {\n      \"name\": \"release\",\n      \"configurePreset\": \"release\"\n    }\n  ]\n}\n",
    )?;
    write_template(
        &src.join(if language == Language::C {
            "main.c"
        } else {
            "main.cpp"
        }),
        if language == Language::C {
            "#include <stdio.h>\n\nint main(void)\n{\n    puts(\"Kinein Vectis\");\n    return 0;\n}\n"
        } else {
            "#include <iostream>\n\nint main()\n{\n    std::cout << \"Kinein Vectis\" << '\\n';\n    return 0;\n}\n"
        },
    )?;
    write_template(&root.join("README.md"), &readme(name, language))?;
    write_template(
        &root.join(".gitignore"),
        if language == Language::C {
            "/build/\n/.cache/\n/.kinein/\n/compile_commands.json\n"
        } else {
            "/build/\n/.cache/\n/compile_commands.json\n"
        },
    )?;

    if let Some(executable) = options
        .executable
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        let path = root.join("CMakePresets.json");
        let mut presets: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).map_err(|source| {
                WorkspaceError::WriteTemplate {
                    path: path.display().to_string(),
                    source,
                }
            })?)
            .map_err(WorkspaceError::Serialize)?;
        if let Some(entries) = presets["configurePresets"].as_array_mut() {
            for entry in entries {
                entry["cacheVariables"][if language == Language::C {
                    "CMAKE_C_COMPILER"
                } else {
                    "CMAKE_CXX_COMPILER"
                }] = executable.into();
            }
        }
        write_template(
            &path,
            &serde_json::to_string_pretty(&presets).map_err(WorkspaceError::Serialize)?,
        )?;
    }
    Ok(())
}

fn cmakelists(name: &str, language: Language, options: &WorkspaceProjectOptions) -> String {
    let (family, extension) = if language == Language::C {
        ("C", "c")
    } else {
        ("CXX", "cpp")
    };
    let standard = options.standard.as_deref().filter(|value| !value.is_empty()).map_or_else(String::new, |standard| {
        if language == Language::C { return super::c_options::block(name, standard); }
        format!("set_target_properties({name} PROPERTIES {family}_STANDARD {standard} {family}_STANDARD_REQUIRED ON {family}_EXTENSIONS OFF)\n")
    });
    let minimum = if language == Language::C {
        "3.25"
    } else {
        "3.24"
    };
    let warnings = if language == Language::Cpp {
        format!(
            "target_compile_options({name} PRIVATE -Wall -Wextra -Wpedantic -Werror -Wconversion -Wsign-conversion -Wshadow)\n"
        )
    } else {
        String::new()
    };
    let tests = if language == Language::C {
        format!("enable_testing()\nadd_test(NAME {name}_smoke COMMAND {name})\n")
    } else {
        String::new()
    };
    format!(
        "cmake_minimum_required(VERSION {minimum})\n\nproject({name} VERSION 0.1.0 LANGUAGES {family})\n\nadd_executable({name} src/main.{extension})\ntarget_include_directories({name} PRIVATE include)\nset_property(TARGET {name} PROPERTY COMPILE_WARNING_AS_ERROR ON)\n\n{warnings}\n{standard}\n{tests}"
    )
}

fn readme(name: &str, language: Language) -> String {
    let language = if language == Language::C { "C" } else { "C++" };
    format!(
        "# {name}\n\nProjeto {language}/CMake criado pelo Kinein Vectis.\n\nCompilador e padrão seguem a escolha feita na criação; sem padrão explícito, vale o padrão da ferramenta.\nEscolha Debug ou Release, configure, compile e execute os testes CTest.\nA disponibilidade do padrão depende do CMake e da toolchain escolhidos.\n"
    )
}
