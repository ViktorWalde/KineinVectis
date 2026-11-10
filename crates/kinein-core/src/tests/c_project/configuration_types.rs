//! C1-22: multi-config (consolidação C §20), de ponta a ponta pelo core, com
//! `CMake` 4.4.5, Ninja Multi-Config, GCC e o `ctest` reais.
//!
//! Até o 0.213.0 o build ia sem `--config` e compilava o
//! `CMAKE_DEFAULT_BUILD_TYPE`, o `ctest` ia sem `-C` e não rodava nenhum teste
//! ("Not Run"), e o run achava um executável por configuração compilada. A
//! ordem `Release;Debug`, com a padrão `Debug`, separa as três respostas: a
//! que o clangd lê é Release, e a do terminal é Debug. O executável imprime a
//! configuração com que foi gerado, e um teste só existe em Release.
//!
//! C1-30: as duas bibliotecas compartilhadas têm o bit de execução, e o run e
//! o debug as contavam no single-config; só o `multi` é `EXECUTABLE`.

use super::configurations::{configure, context_from, ctest, job, kind_define, run, status};
use super::{Project, environment::project};
use kinein_protocol::{JsonRpcRequest, ProjectKind};
use serde_json::json;
use std::{fs, path::Path, process::Command};

const MAIN: &str = "#include <stdio.h>\nint main(void) { puts(\"kv-run-\" KV_KIND); return 0; }\n";

const LISTS: &str = r#"cmake_minimum_required(VERSION 3.24)
project(multi LANGUAGES C)
add_executable(multi src/main.c)
target_compile_definitions(multi PRIVATE "KV_KIND=\"$<CONFIG>\"")
add_library(kv_shared SHARED src/shared.c)
add_library(kv_versioned SHARED src/shared.c)
set_target_properties(kv_versioned PROPERTIES VERSION 1.2.3 SOVERSION 1)
file(CONFIGURE OUTPUT kv-tool.sh CONTENT "exit 0\n")
file(CHMOD ${CMAKE_BINARY_DIR}/kv-tool.sh PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE)
enable_testing()
add_test(NAME kv_smoke COMMAND multi)
add_test(NAME kv_release_only COMMAND multi CONFIGURATIONS Release)
"#;

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"multi","generator":"Ninja Multi-Config","binaryDir":"${sourceDir}/build/multi",
   "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_EXPORT_COMPILE_COMMANDS":"ON",
                     "CMAKE_CONFIGURATION_TYPES":"Release;Debug","CMAKE_DEFAULT_BUILD_TYPE":"Debug"}},
  {"name":"single","generator":"Ninja","binaryDir":"${sourceDir}/build/single",
   "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_EXPORT_COMPILE_COMMANDS":"ON",
                     "CMAKE_CONFIGURATION_TYPES":"Release;Debug","CMAKE_BUILD_TYPE":"Debug"}}]}"#;

/// O `run.start` sem comando, recusado: o motivo.
fn run_refused(p: &mut Project) -> String {
    let response = p
        .core
        .handle_request(&JsonRpcRequest::new(9_i64, "run.start", Some(json!({}))))
        .response()
        .clone();
    response.error.expect("o run é recusado").message
}

/// `build.run` com a configuração já configurada: o comando do build.
fn build_command(p: &mut Project) -> String {
    let id = p.call("build.run", json!({}))["jobId"].clone();
    let (started, lines, success) = job(p, &id, "event.build.started", "event.build.finished");
    assert!(success, "{lines:?}");
    started["command"].as_str().unwrap().to_owned()
}

fn executable(directory: &Path) -> std::path::PathBuf {
    directory.join(if cfg!(windows) { "multi.exe" } else { "multi" })
}

#[test]
fn c1_22_the_ide_builds_tests_and_runs_the_configuration_clangd_reads() {
    let mut p = project("multi-config");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), MAIN).unwrap();
    fs::write(
        p.root.join("src/shared.c"),
        "int kv_shared(void) { return 1; }\n",
    )
    .unwrap();
    fs::write(p.root.join("CMakeLists.txt"), LISTS).unwrap();
    fs::write(p.root.join("CMakePresets.json"), PRESETS).unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    let folder = p.root.join("build/multi");

    let (_, lines, success) = configure(&mut p, "multi");
    assert!(success, "{lines:?}");
    let several = status(&mut p);
    assert_eq!(several["buildConfiguration"], "Release", "{several}");
    assert_eq!(several["defaultBuildConfiguration"], "Debug", "{several}");
    // O contexto da IDE é a primeira entrada da CDB, a que o clangd lê.
    assert_eq!(
        kind_define(&context_from(&mut p, &folder)),
        "KV_KIND=\"Release\""
    );

    // O build compila Release, não a padrão do terminal.
    assert_eq!(build_command(&mut p), "cmake --build --config Release");
    assert!(executable(&folder.join("Release")).is_file());
    assert!(!executable(&folder.join("Debug")).exists());

    // O ctest com `-C Release`: os dois testes rodam e passam, também o que
    // só existe em Release.
    let mut cases = ctest(&mut p);
    cases.sort();
    assert_eq!(
        cases,
        ["\"kv_release_only\" \"passed\"", "\"kv_smoke\" \"passed\""]
    );
    let mut listed = Vec::new();
    let mut names: Vec<_> = crate::test::discover_tests(
        &p.root,
        ProjectKind::Cmake,
        None,
        &std::sync::Arc::default(),
        &mut |event| {
            if let crate::test::TestEvent::Started { command } = event {
                listed.push(command);
            }
        },
    )
    .unwrap()
    .into_iter()
    .map(|case| case.name)
    .collect();
    names.sort();
    assert_eq!(names, ["kv_release_only", "kv_smoke"]);
    assert_eq!(listed, ["ctest -N -C Release"]);
    let entries: Vec<_> = crate::test::frameworks::ctest_entries(
        Path::new("ctest"),
        &folder,
        &std::sync::Arc::default(),
    )
    .into_iter()
    .map(|entry| (entry.name, entry.command))
    .collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert!(
        entries.iter().all(
            |(_, command)| command
                .first()
                .is_some_and(|program| Path::new(program) == executable(&folder.join("Release")))
        ),
        "{entries:?}"
    );

    // Com o Debug também compilado pelo terminal, o run e o debug ficam no
    // de Release.
    let terminal = Command::new("cmake")
        .args(["--build", "build/multi", "--config", "Debug"])
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(terminal.status.success(), "{terminal:?}");
    assert!(executable(&folder.join("Debug")).is_file());
    assert_eq!(
        run(&mut p, &executable(&folder.join("Release"))),
        "kv-run-Release"
    );
    assert_eq!(
        crate::dap::resolve_program(ProjectKind::Cmake, &p.root).unwrap(),
        crate::dap::DebugTarget::Program(executable(&folder.join("Release")))
    );

    // Sem o modelo do file-api, o run e o debug recusam com o motivo.
    fs::remove_dir_all(folder.join(".cmake/api/v1/reply")).unwrap();
    let reason = "o build multi-config em build/multi não tem o modelo do file-api, que diz \
                  quais executáveis são da configuração Release; configure pela IDE (Configurar)";
    assert_eq!(run_refused(&mut p), reason);
    assert_eq!(
        crate::dap::resolve_program(ProjectKind::Cmake, &p.root)
            .unwrap_err()
            .to_string(),
        reason
    );
    single_config(&mut p);
}

/// O preset `single`, depois do multi-config, no mesmo projeto.
fn single_config(p: &mut Project) {
    // O Ninja de configuração única ignora `CMAKE_CONFIGURATION_TYPES`: o
    // build vai sem `--config`, e o status não traz os campos.
    let (_, lines, success) = configure(p, "single");
    assert!(success, "{lines:?}");
    let single = status(p);
    assert!(single.get("buildConfiguration").is_none(), "{single}");
    assert!(
        single.get("defaultBuildConfiguration").is_none(),
        "{single}"
    );
    assert_eq!(build_command(p), "cmake --build");
    assert_eq!(ctest(p), ["\"kv_smoke\" \"passed\""]);

    // C1-30: pelo modelo, as bibliotecas e o script que o configure gerou com
    // o bit de execução ficam de fora. Sem o modelo, a varredura deixa de fora
    // as bibliotecas pelo nome, mas não sabe que o script não é um programa.
    let alone = p.root.join("build/single");
    for file in ["libkv_shared.so", "libkv_versioned.so.1.2.3", "kv-tool.sh"] {
        assert!(
            cfg!(windows) || crate::platform::is_executable(&alone.join(file)),
            "{file}"
        );
    }
    assert_eq!(run(p, &executable(&alone)), "kv-run-Debug");
    assert_eq!(
        crate::dap::resolve_program(ProjectKind::Cmake, &p.root).unwrap(),
        crate::dap::DebugTarget::Program(executable(&alone))
    );
    fs::remove_dir_all(alone.join(".cmake/api/v1/reply")).unwrap();
    if cfg!(windows) {
        assert_eq!(run(p, &executable(&alone)), "kv-run-Debug");
    } else {
        let reason = run_refused(p);
        assert!(
            reason.starts_with("mais de um executável em build/single (")
                && reason.contains(&alone.join("kv-tool.sh").display().to_string())
                && reason.contains(&executable(&alone).display().to_string())
                && !reason.contains("libkv"),
            "{reason}"
        );
    }
}
