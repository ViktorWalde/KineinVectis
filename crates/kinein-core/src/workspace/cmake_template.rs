//! Arquivos dos templates C/C++ no dono existente de criacao de projetos.
use super::{WorkspaceError, create::write_template};
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Language {
    C,
    Cpp,
}

pub(super) fn write(root: &Path, name: &str, language: Language) -> Result<(), WorkspaceError> {
    let src = root.join("src");
    write_template(&root.join("CMakeLists.txt"), &cmakelists(name, language))?;
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

    Ok(())
}

fn cmakelists(name: &str, language: Language) -> String {
    if language == Language::C {
        return format!(
            "cmake_minimum_required(VERSION 3.25)\n\nproject({name} VERSION 0.1.0 LANGUAGES C)\n\nadd_executable({name} src/main.c)\ntarget_include_directories({name} PRIVATE include)\nset_property(TARGET {name} PROPERTY COMPILE_WARNING_AS_ERROR ON)\n\n{}\nenable_testing()\nadd_test(NAME {name}_smoke COMMAND {name})\n",
            crate::configaction::c_standard_block(name, "23", "OFF")
        );
    }
    format!(
        "cmake_minimum_required(VERSION 3.24)\n\n\
             project({name}\n\
                 VERSION 0.1.0\n\
                 LANGUAGES CXX\n\
             )\n\n\
             add_executable({name}\n\
                 src/main.cpp\n\
             )\n\n\
             target_compile_features({name}\n\
                 PRIVATE\n\
                     cxx_std_23\n\
             )\n\n\
             target_include_directories({name}\n\
                 PRIVATE\n\
                     ${{CMAKE_CURRENT_SOURCE_DIR}}/include\n\
             )\n\n\
             target_compile_options({name} PRIVATE\n\
                 -Wall -Wextra -Wpedantic -Werror\n\
                 -Wconversion -Wsign-conversion -Wshadow\n\
             )\n"
    )
}

fn readme(name: &str, language: Language) -> String {
    if language == Language::C {
        format!(
            "# {name}\n\nProjeto C/CMake criado pelo Kinein Vectis.\n\nPadrão inicial: C23 exigido no alvo, sem extensões. A fonte inicial é compatível com C90.\nEscolha Debug ou Release, configure, compile e execute os testes CTest.\nPara mudar o padrão, use Ações de configuração → Fixar o padrão C do alvo; revise a prévia antes de aplicar.\nA disponibilidade do padrão depende do CMake e da toolchain escolhidos.\n"
        )
    } else {
        format!("# {name}\n\nProjeto C++/CMake strict criado pelo Kinein Vectis.\n")
    }
}
