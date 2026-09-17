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
    assert!(error.message.contains("workspace CMake"));
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
    assert!(
        result["buildDir"]
            .as_str()
            .unwrap()
            .ends_with(".kinein/build")
    );

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
fn espera_configure(
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
#[cfg(unix)]
fn configure_picks_the_project_default_preset_and_status_reports_it() {
    use std::os::unix::fs::PermissionsExt;
    let dir = cmake_workspace("preset-automatico");
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    // O cmake falso ecoa os argv e "configura" (cria o CMakeCache no -B).
    let cmake = dir.join("bin/cmake");
    std::fs::write(
        &cmake,
        "#!/bin/sh\necho \"cmake-falso $*\"\nwhile [ $# -gt 0 ]; do [ \"$1\" = -B ] && \
         { mkdir -p \"$2\"; : > \"$2/CMakeCache.txt\"; }; shift; done\nexit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&cmake, std::fs::Permissions::from_mode(0o755)).unwrap();
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
    let dir = dir.canonicalize().unwrap();

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
    let (inicio, saida) = espera_configure(&receiver);
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
    let (inicio, _) = espera_configure(&receiver);
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
    let (inicio, saida) = espera_configure(&receiver);
    assert_eq!(inicio["preset"], serde_json::Value::Null);
    assert_eq!(inicio["command"], "cmake configure");
    assert!(
        saida.iter().any(|l| l.starts_with("cmake-falso -S ")),
        "{saida:?}"
    );
    let status = status_de(&mut core, 97);
    assert!(status.get("preset").is_none(), "{status}");
}
