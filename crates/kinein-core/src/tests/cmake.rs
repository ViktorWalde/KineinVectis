//! `CMake` service dispatch (`cmake.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

fn cmake_workspace(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cmake-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo CXX)\n",
    )
    .unwrap();
    dir
}

#[test]
fn cmake_methods_require_workspace_and_cmake_kind() {
    let mut core = core_with_empty_search_path("cmake-guards");
    for method in [
        "cmake.configure",
        "cmake.presets.list",
        "cmake.targets.list",
        "cmake.status",
    ] {
        let outcome = core.handle_request(&JsonRpcRequest::new(80_i64, method, None));
        let error = outcome.response().error.as_ref().unwrap();
        assert_eq!(
            error.code,
            kinein_protocol::JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }

    let rust_dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-cmake-rust", std::process::id()));
    std::fs::create_dir_all(&rust_dir).unwrap();
    std::fs::write(rust_dir.join("Cargo.toml"), "[package]\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        81_i64,
        "workspace.open",
        Some(json!({ "path": rust_dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let wrong_kind = core.handle_request(&JsonRpcRequest::new(82_i64, "cmake.status", None));
    let error = wrong_kind.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("projeto CMake"));
}

#[test]
fn cmake_status_and_presets_work_without_jobs() {
    let dir = cmake_workspace("status");
    std::fs::write(
        dir.join("CMakePresets.json"),
        r#"{ "version": 6, "configurePresets": [ { "name": "release" } ] }"#,
    )
    .unwrap();
    let mut core = core_with_empty_search_path("cmake-status");
    let opened = core.handle_request(&JsonRpcRequest::new(
        83_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let status = core.handle_request(&JsonRpcRequest::new(84_i64, "cmake.status", None));
    let result = status.response().result.as_ref().unwrap().clone();
    assert_eq!(result["configured"], false);
    assert_eq!(result["hasCompileCommands"], false);
    assert!(std::path::Path::new(result["buildDir"].as_str().unwrap()).ends_with(".kinein/build"));

    let presets = core.handle_request(&JsonRpcRequest::new(85_i64, "cmake.presets.list", None));
    let listed = presets.response().result.as_ref().unwrap()["presets"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["name"], "release");

    let targets = core.handle_request(&JsonRpcRequest::new(86_i64, "cmake.targets.list", None));
    assert!(
        targets.response().result.as_ref().unwrap()["targets"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let configure = core.handle_request(&JsonRpcRequest::new(87_i64, "cmake.configure", None));
    let error = configure.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
    assert!(error.message.contains("jobs"));
}

/// `cmake.status` diagnostica a CDB que o clangd REALMENTE alcança.
///
/// `hasCompileCommands` olha só o build dir da IDE. O clangd, porém, acha uma
/// base em `build/` e nos diretórios pai por conta própria
/// (<https://clangd.llvm.org/installation>) — então um projeto Meson ou `bear`
/// funciona com `hasCompileCommands: false`. Sem o diagnóstico, a IDE não
/// consegue distinguir "sem CDB" de "CDB fora do meu build dir", e o usuário vê
/// erro de include sem causa.
#[test]
fn cmake_status_reports_a_database_outside_the_ide_build_dir() {
    let dir = cmake_workspace("cdb-fora-do-build-dir");
    let externo = dir.join("build");
    std::fs::create_dir_all(&externo).unwrap();
    std::fs::write(externo.join("compile_commands.json"), "[]\n").unwrap();

    let mut core = core_with_empty_search_path("cmake-cdb-externa");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        920_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let status = core.handle_request(&JsonRpcRequest::new(921_i64, "cmake.status", None));
    let resultado = status.response().result.as_ref().unwrap();

    // A IDE nao configurou nada: o build dir dela esta vazio...
    assert_eq!(resultado["hasCompileCommands"], false);
    // ...mas existe uma CDB alcancavel, e a resposta diz ONDE.
    assert_eq!(resultado["cdbDirectory"], "build");
}

/// CDB mais velha que o `CMakeLists.txt` é reportada com o culpado.
///
/// Sem isto, mudar o `CMakeLists.txt` sem reconfigurar deixa o clangd usando
/// flags de um projeto que não existe mais — e o erro que aparece na tela não
/// tem relação visível com a causa.
#[test]
fn cmake_status_reports_a_stale_database_and_names_the_culprit() {
    let dir = cmake_workspace("cdb-velha");
    let externo = dir.join("build");
    std::fs::create_dir_all(&externo).unwrap();
    std::fs::write(externo.join("compile_commands.json"), "[]\n").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    // Toca o CMakeLists DEPOIS da CDB.
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo CXX)\nadd_executable(a a.cpp)\n",
    )
    .unwrap();

    let mut core = core_with_empty_search_path("cmake-cdb-velha");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        922_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let status = core.handle_request(&JsonRpcRequest::new(923_i64, "cmake.status", None));
    let resultado = status.response().result.as_ref().unwrap();

    assert_eq!(resultado["cdbStale"], true);
    assert_eq!(resultado["cdbStaleBecause"], "CMakeLists.txt");
}

/// Projeto sadio não carrega campo de diagnóstico nenhum.
#[test]
fn cmake_status_stays_quiet_when_there_is_nothing_to_diagnose() {
    let dir = cmake_workspace("cdb-quieto");
    let mut core = core_with_empty_search_path("cmake-cdb-quieto");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        924_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let status = core.handle_request(&JsonRpcRequest::new(925_i64, "cmake.status", None));
    let resultado = status.response().result.as_ref().unwrap();

    // Sem CDB em lugar nenhum: nem `cdbDirectory`, nem ruido de `cdbStale`.
    assert!(resultado.get("cdbDirectory").is_none());
    assert!(resultado.get("cdbStale").is_none());
}

/// O preset no configure automatico (P0 do 40 §4.1, 2026-09-17), pelo
/// despacho real com um `cmake` FALSO fixado no kit: sem `preset` no pedido,
/// o core escolhe o padrao do projeto (`CMakeUserPresets` vence `CMakePresets`),
/// diz de onde veio em `event.cmake.started`, anota o preset no build dir e
/// o `cmake.status` o devolve; com `preset` no pedido (o kit), e' ele.
/// Espera o ciclo de um `cmake.configure`: devolve o `event.cmake.started`
/// e as linhas de saida do job ate' o `finished` com sucesso.
fn wait_configure(
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
) -> (serde_json::Value, Vec<String>) {
    let mut inicio = serde_json::Value::Null;
    let mut saida = Vec::new();
    loop {
        let event = receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("evento do configure");
        match event.method.as_str() {
            "event.cmake.started" => inicio = event.params.unwrap(),
            "event.job.output" => {
                saida.push(event.params.unwrap()["line"].as_str().unwrap().to_owned());
            }
            "event.cmake.finished" => {
                assert_eq!(event.params.as_ref().unwrap()["success"], true);
                return (inicio, saida);
            }
            _ => {}
        }
    }
}

fn status_de(core: &mut crate::Core, id: i64) -> serde_json::Value {
    core.handle_request(&JsonRpcRequest::new(id, "cmake.status", None))
        .response()
        .result
        .clone()
        .unwrap()
}

#[test]
fn configure_picks_the_project_default_preset_and_status_reports_it() {
    let dir = cmake_workspace("preset-automatico");
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    // O cmake falso ecoa os argv e "configura" (cria o CMakeCache no -B).
    let cmake = dir.join("bin/cmake");
    crate::write_executable(
        &cmake,
        "#!/bin/sh\necho \"cmake-falso $*\"\nwhile [ $# -gt 0 ]; do [ \"$1\" = -B ] && \
         { mkdir -p \"$2\"; : > \"$2/CMakeCache.txt\"; }; shift; done\nexit 0\n",
    );
    std::fs::write(
        dir.join("CMakePresets.json"),
        r#"{"version": 6, "configurePresets": [{"name": "base", "hidden": true}, {"name": "projeto"}]}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("CMakeUserPresets.json"),
        r#"{"version": 6, "configurePresets": [{"name": "meu-local"}]}"#,
    )
    .unwrap();
    let dir = crate::platform::canonicalize(&dir).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        90_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let fixado = core.handle_request(&JsonRpcRequest::new(
        91_i64,
        "toolchain.set",
        Some(json!({ "role": "cmake", "id": "cmake" })),
    ));
    assert!(
        fixado.response().error.is_none(),
        "{:?}",
        fixado.response().error
    );

    // Sem preset no pedido: o do usuario vence o do projeto.
    let aceito = core.handle_request(&JsonRpcRequest::new(
        92_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    assert!(aceito.response().error.is_none());
    let (inicio, saida) = wait_configure(&receiver);
    assert_eq!(inicio["preset"], "meu-local");
    assert_eq!(inicio["presetSource"], "CMakeUserPresets.json");
    assert_eq!(inicio["command"], "cmake configure --preset meu-local");
    assert!(
        saida
            .iter()
            .any(|l| l.starts_with("cmake-falso --preset meu-local -S ")),
        "{saida:?}"
    );
    let status = status_de(&mut core, 93);
    assert_eq!(status["configured"], true);
    assert_eq!(status["preset"], "meu-local");

    // Com preset no pedido (o kit ativo): e' ele, e a origem diz "kit".
    let aceito = core.handle_request(&JsonRpcRequest::new(
        94_i64,
        "cmake.configure",
        Some(json!({ "preset": "projeto" })),
    ));
    assert!(aceito.response().error.is_none());
    let (inicio, _) = wait_configure(&receiver);
    assert_eq!(inicio["preset"], "projeto");
    assert_eq!(inicio["presetSource"], "kit");
    let status = status_de(&mut core, 95);
    assert_eq!(status["preset"], "projeto");

    // Sem presets no projeto: configure como sempre, sem preset, e o status
    // nao afirma nenhum.
    std::fs::remove_file(dir.join("CMakePresets.json")).unwrap();
    std::fs::remove_file(dir.join("CMakeUserPresets.json")).unwrap();
    let aceito = core.handle_request(&JsonRpcRequest::new(
        96_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    assert!(aceito.response().error.is_none());
    let (inicio, saida) = wait_configure(&receiver);
    assert_eq!(inicio["preset"], serde_json::Value::Null);
    assert_eq!(inicio["command"], "cmake configure");
    assert!(
        saida.iter().any(|l| l.starts_with("cmake-falso -S ")),
        "{saida:?}"
    );
    let status = status_de(&mut core, 97);
    assert!(status.get("preset").is_none(), "{status}");
}

/// Um `CMakePresets.json` com um oculto e os nomes dados.
fn presets(names: &[&str]) -> String {
    let items: Vec<String> = names
        .iter()
        .map(|name| format!(r#"{{"name": "{name}"}}"#))
        .collect();
    format!(
        r#"{{"version": 6, "configurePresets": [{{"name": "base", "hidden": true}}, {}]}}"#,
        items.join(", ")
    )
}

/// O preset do configure automatico (0.4.0b, `roadmaps/52` §14.2): com
/// varios presets e nada anotado, o configure sem preset e' RECUSADO, sem job,
/// e o `cmake.status` diz entre quais escolher; escolhido um (o kit), ele fica
/// anotado e passa a valer no configure sem preset (`buildDir`); se some dos
/// presets, a pergunta volta.
#[test]
fn several_presets_ask_and_the_recorded_one_wins_until_it_vanishes() {
    let dir = cmake_workspace("preset-varios");
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    let cmake = dir.join("bin/cmake");
    crate::write_executable(
        &cmake,
        "#!/bin/sh\necho \"cmake-falso $*\"\nwhile [ $# -gt 0 ]; do [ \"$1\" = -B ] && \
         { mkdir -p \"$2\"; : > \"$2/CMakeCache.txt\"; }; shift; done\nexit 0\n",
    );
    std::fs::write(
        dir.join("CMakePresets.json"),
        presets(&["debug", "release"]),
    )
    .unwrap();
    let dir = crate::platform::canonicalize(&dir).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let fixed = core.handle_request(&JsonRpcRequest::new(
        81_i64,
        "toolchain.set",
        Some(json!({ "role": "cmake", "id": "cmake" })),
    ));
    assert!(fixed.response().error.is_none());

    let status = status_de(&mut core, 82);
    assert_eq!(
        status["presetChoices"],
        json!(["debug", "release"]),
        "{status}"
    );
    assert_eq!(status["presetChoicesFile"], "CMakePresets.json");
    let refused = core.handle_request(&JsonRpcRequest::new(
        83_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    let error = refused
        .response()
        .error
        .clone()
        .expect("vários presets: recusa");
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(
        error.message.contains("debug, release"),
        "{}",
        error.message
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(300);
    while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
        if let Ok(event) = receiver.recv_timeout(left) {
            assert_ne!(event.method, "event.cmake.started", "a recusa nao roda job");
        }
    }

    let accepted = core.handle_request(&JsonRpcRequest::new(
        84_i64,
        "cmake.configure",
        Some(json!({ "preset": "release" })),
    ));
    assert!(accepted.response().error.is_none());
    wait_configure(&receiver);
    let status = status_de(&mut core, 85);
    assert_eq!(status["preset"], "release");
    assert!(status.get("presetChoices").is_none(), "{status}");

    let again = core.handle_request(&JsonRpcRequest::new(
        86_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    assert!(again.response().error.is_none());
    let (start, _) = wait_configure(&receiver);
    assert_eq!(start["preset"], "release");
    assert_eq!(start["presetSource"], "buildDir");

    std::fs::write(dir.join("CMakePresets.json"), presets(&["debug", "rel2"])).unwrap();
    let status = status_de(&mut core, 87);
    assert_eq!(
        status["presetChoices"],
        json!(["debug", "rel2"]),
        "{status}"
    );
    let refused = core.handle_request(&JsonRpcRequest::new(
        88_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    assert!(
        refused.response().error.is_some(),
        "o anotado sumiu: a pergunta volta"
    );
}

/// O `clang-tidy` de verdade, o do Visual Studio, sobre a CDB de verdade do
/// MSVC (60 §3.3, W6b2): o `return x` de uma variavel sem valor vira o aviso
/// do analisador do clang, com arquivo e linha.
#[test]
#[cfg(windows)]
fn the_real_clang_tidy_checks_the_real_msvc_cdb() {
    use std::sync::{Arc, atomic::AtomicBool};

    use kinein_protocol::{ProjectKind, RigorProfile};

    let dir = cmake_workspace("msvc-clang-tidy");
    drop(std::fs::remove_dir_all(dir.join(".kinein")));
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(tidy CXX)\nadd_executable(tidy main.cpp)\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("main.cpp"),
        "int main() {\n  int x;\n  return x;\n}\n",
    )
    .unwrap();
    let root = crate::platform::canonicalize(&dir).unwrap();
    let toolchain = crate::toolchain::Toolchain::resolve(&root, &[]);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut lines = Vec::new();
    let built = crate::build::run_build(
        &root,
        ProjectKind::Cmake,
        RigorProfile::Balanced,
        &toolchain,
        &crate::build::BuildTools::default(),
        &cancel,
        &mut |event| {
            if let crate::build::BuildEvent::Output { line, .. } = event {
                lines.push(line);
            }
        },
    )
    .expect("o build roda");
    assert!(built.success, "{lines:#?}");

    let clang_tidy = crate::msvc::tool_dirs()
        .into_iter()
        .map(|dir| dir.join("clang-tidy.exe"))
        .find(|path| path.is_file())
        .expect("o Visual Studio traz o clang-tidy");
    let tools = crate::build::QualityTools {
        clang_tidy: Some(clang_tidy),
        ..crate::build::QualityTools::default()
    };
    let mut diagnostics = Vec::new();
    let mut output = Vec::new();
    crate::build::run_quality(
        &root,
        ProjectKind::Cmake,
        RigorProfile::Balanced,
        &toolchain,
        &tools,
        &cancel,
        &mut |event| match event {
            crate::build::BuildEvent::Diagnostic(diagnostic) => diagnostics.push(diagnostic),
            crate::build::BuildEvent::Output { line, .. } => output.push(line),
            crate::build::BuildEvent::Started { .. } => {}
        },
    )
    .expect("o clang-tidy roda");
    let undefined = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.message.contains("garbage") || diagnostic.message.contains("uninitialized")
        })
        .unwrap_or_else(|| panic!("{diagnostics:#?}\n{output:#?}"));
    assert!(
        undefined
            .file
            .as_deref()
            .map(std::path::Path::new)
            .is_some_and(|file| file.ends_with("main.cpp")),
        "{undefined:?}"
    );
    assert_eq!(undefined.line, Some(3), "{undefined:?}");
}

/// De ponta a ponta no Windows (60 §3.3, W6a): um projeto C++ de verdade,
/// configurado e compilado pelo core no ambiente do Visual Studio, com o
/// Ninja (que gera a CDB), e executado pelo comando padrao do Run, que roda
/// no PowerShell (D11). Nada de C/C++ esta' no `PATH` do teste.
#[test]
#[cfg(windows)]
fn a_real_cpp_project_builds_with_msvc_and_runs() {
    use std::sync::{Arc, atomic::AtomicBool};

    use kinein_protocol::{ProjectKind, RigorProfile};

    let dir = cmake_workspace("msvc-ponta-a-ponta");
    drop(std::fs::remove_dir_all(dir.join(".kinein")));
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(ola CXX)\nadd_executable(ola main.cpp)\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("main.cpp"),
        "#include <iostream>\nint main() { std::cout << \"ola do msvc\" << std::endl; }\n",
    )
    .unwrap();
    let root = crate::platform::canonicalize(&dir).unwrap();

    let toolchain = crate::toolchain::Toolchain::resolve(&root, &[]);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut lines = Vec::new();
    let outcome = crate::build::run_build(
        &root,
        ProjectKind::Cmake,
        RigorProfile::Balanced,
        &toolchain,
        &crate::build::BuildTools::default(),
        &cancel,
        &mut |event| {
            if let crate::build::BuildEvent::Output { line, .. } = event {
                lines.push(line);
            }
        },
    )
    .expect("o build roda");
    assert!(outcome.success, "{lines:#?}");
    assert!(
        crate::cmake::build_dir(&root)
            .join("compile_commands.json")
            .is_file(),
        "o Ninja gera a CDB; o gerador do Visual Studio nao: {lines:#?}"
    );

    let command = crate::run::default_command(ProjectKind::Cmake, &root).unwrap();
    assert!(command.starts_with("& '"), "{command}");
    let (shell, args) = crate::platform::shell_command(&command);
    let output = std::process::Command::new(shell)
        .args(args)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("ola do msvc"),
        "{command}: {output:?}"
    );
}
