//! Recovery boundaries, cancellation after the producer starts and manual CDB.

use super::existing::{configure, context, current, fixture};
use kinein_protocol::JsonRpcRequest;
use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};

#[test]
fn c1_existing_cancelled_configure_retains_obsolete_state_until_successful_retry() {
    let compiler = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let mut p = fixture("existing-cancel", compiler.to_str().unwrap());
    configure(&mut p, Some("debug local"), true);
    p.build(true);
    current(&mut p);
    let path = p.root.join("CMakeLists.txt");
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{original}\nmessage(STATUS \"C1 configure producer started\")\nexecute_process(COMMAND \"${{CMAKE_COMMAND}}\" -E sleep 30)\n")).unwrap();
    let job = p.call("cmake.configure", json!({"preset":"debug local"}))["jobId"].clone();
    p.wait("event.job.output", |value| {
        value["jobId"] == job
            && value["line"]
                .as_str()
                .is_some_and(|line| line.contains("C1 configure producer started"))
    });
    assert!(
        context(&mut p)["effective"].is_null(),
        "running configure is not current evidence"
    );
    let debug_local = p.root.join(".kinein/presets/debug local");
    let pending = ".kinein-configure-pending";
    let attempt = fs::read(debug_local.join(pending)).unwrap();
    // A mesma configuração não tem dois produtores: o segundo recusa antes
    // de tocar a tentativa em andamento e o registro.
    let same = p.call("cmake.configure", json!({"preset":"debug local"}))["jobId"].clone();
    p.wait("event.job.output", |value| {
        value["jobId"] == same
            && value["line"] == "não foi possível registrar o configure: outra instância está alterando as configurações; aguarde e tente novamente"
    });
    assert_eq!(
        p.wait("event.cmake.finished", |value| value["jobId"] == same)["success"],
        false
    );
    assert_eq!(fs::read(debug_local.join(pending)).unwrap(), attempt);
    assert_eq!(
        crate::cmake::selected_configuration(&p.root),
        Ok(Some("debug local".to_owned()))
    );
    // Outra configuração configura na pasta dela, em paralelo (C1-19), e é a
    // ativa desde que começa (C1-20); falha pelo C1_FAIL do preset.
    let parallel = p.call("cmake.configure", json!({"preset":"release local"}))["jobId"].clone();
    assert_eq!(
        p.wait("event.cmake.finished", |value| value["jobId"] == parallel)["success"],
        false
    );
    assert!(
        context(&mut p)["staleBecause"]
            .as_str()
            .unwrap()
            .contains("release local"),
        "the configuration begun last is the active one"
    );
    assert_eq!(
        fs::read(debug_local.join(pending)).unwrap(),
        attempt,
        "each folder keeps its own attempt"
    );
    assert!(
        p.root
            .join(".kinein/presets/release local")
            .join(pending)
            .is_file()
    );
    p.call("core.ping", json!({}));
    let start = Instant::now();
    p.call("job.cancel", json!({"jobId":job}));
    let finished = p.wait("event.cmake.finished", |value| value["jobId"] == job);
    assert_eq!(finished["success"], false);
    assert!(
        start.elapsed() < Duration::from_secs(8),
        "cancel must interrupt running producer"
    );
    assert!(
        context(&mut p)["staleBecause"]
            .as_str()
            .unwrap()
            .contains("configure")
    );
    assert_eq!(fs::read(debug_local.join(pending)).unwrap(), attempt);
    fs::write(path, original).unwrap();
    configure(&mut p, Some("debug local"), true);
    p.build(true);
    current(&mut p);
    assert!(!debug_local.join(pending).exists());
}

#[test]
fn c1_existing_removed_preset_and_corrupt_attempt_require_explicit_recovery() {
    let compiler = crate::tools::ToolDetector::from_environment()
        .find_in_path("clang")
        .unwrap();
    let mut p = fixture("existing-preset-removed", compiler.to_str().unwrap());
    configure(&mut p, Some("debug local"), true);
    configure(&mut p, Some("release local"), false);
    let path = p.root.join("CMakePresets.json");
    let mut presets: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    presets["configurePresets"].as_array_mut().unwrap().pop();
    fs::write(path, presets.to_string()).unwrap();
    let response = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    let removed = "configuração escolhida removida dos presets: release local; escolha outra";
    assert_eq!(response.response().error.as_ref().unwrap().message, removed);
    // O status diz por quê e oferece o que o automático escolheria.
    let status = p.call("cmake.status", json!({}));
    assert_eq!(status["configurationRefused"], removed, "{status}");
    assert_eq!(status["presetChoices"], json!(["debug local"]), "{status}");
    assert_eq!(status["presetChoicesFile"], "CMakePresets.json");
    assert!(status.get("buildDir").is_none(), "{status}");
    // Uma tentativa ilegível na pasta da configuração escolhida recusa o
    // configure implícito, sem virar o automático.
    p.call("cmake.selectConfiguration", json!({"preset":"debug local"}));
    let marker = p
        .root
        .join(".kinein/presets/debug local/.kinein-configure-pending");
    fs::write(marker, "invalid json").unwrap();
    let response = p.core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    assert!(
        response
            .response()
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("inválido")
    );
    configure(&mut p, Some("debug local"), true);
    p.build(true);
    current(&mut p);
}

#[test]
fn c1_existing_manual_database_remains_owned_by_its_external_tool() {
    use crate::index::context::{CompileContext, ContextTools};
    let compiler = crate::tools::ToolDetector::from_environment()
        .find_in_path("clang")
        .unwrap();
    let p = fixture("existing-external", compiler.to_str().unwrap());
    let result = std::process::Command::new("cmake")
        .args([
            "--preset",
            "debug local",
            "-B",
            "build",
            "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
        ])
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let result = std::process::Command::new("cmake")
        .args(["--build", "build", "--parallel", "2"])
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    // Uma tentativa da IDE de antes do C-04, pendente em .kinein/build, que
    // não é a configuração escolhida: a CDB do terminal continua sendo dela.
    let ide = crate::cmake::ide_build_dir(&p.root);
    drop(crate::cmake::begin_configure(&p.root, &ide, Some("release local"), "").unwrap());
    crate::cmake::select_configuration(&p.root, None).unwrap();
    let context = CompileContext::load(&p.root, &ContextTools::default()).for_file(
        &p.root,
        &p.root.join("main.c"),
        "c",
    );
    assert!(context.stale_because.is_none(), "{context:?}");
    assert_eq!(
        context.effective.unwrap().compiler.unwrap().value,
        compiler.to_str().unwrap()
    );
    assert_eq!(
        crate::cdb::status(&p.root).directory.as_deref(),
        Some("build")
    );
    assert_eq!(
        fs::read_to_string(p.root.join("compile_flags.txt")).unwrap(),
        "-DMANUAL_FLAGS\n"
    );
    assert!(
        fs::read_to_string(p.root.join(".clangd"))
            .unwrap()
            .contains("MANUAL_CLANGD")
    );
}

#[test]
fn c1_existing_late_context_reload_cannot_replace_the_new_workspaces_context() {
    let compiler = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    for initial in [false, true] {
        let mut p = fixture(
            &format!("existing-late-context-{initial}"),
            compiler.to_str().unwrap(),
        );
        configure(&mut p, Some("debug local"), true);
        current(&mut p);
        let bin = p.parent.join("cargo-bin");
        fs::create_dir(&bin).unwrap();
        let started = p.parent.join("metadata-started");
        let release = p.parent.join("metadata-release");
        crate::write_executable(
            bin.join("cargo"),
            format!(
                "#!/bin/sh\n: > '{}'\nwhile [ ! -e '{}' ]; do sleep 0.01; done\nprintf '%s' '{{\"packages\":[]}}'\n",
                started.display(),
                release.display()
            ),
        );
        fs::write(
            p.root.join("Cargo.toml"),
            "[package]\nname=\"fixture\"\nversion=\"0.1.0\"\n",
        )
        .unwrap();
        p.core.detector = crate::tools::ToolDetector::with_search_path(&bin);
        p.core.tool_registry.lock().unwrap().take();
        if initial {
            p.call("workspace.open", json!({"path":p.root}));
        } else {
            p.core.reload_index_context();
        }
        let job = p.wait("event.job.created", |value| {
            if initial {
                value["title"]
                    .as_str()
                    .is_some_and(|title| title.starts_with("Indexar "))
            } else {
                value["title"] == "Reler o contexto de compilador"
            }
        })["id"]
            .clone();
        let deadline = Instant::now() + Duration::from_secs(8);
        while !started.is_file() {
            assert!(Instant::now() < deadline, "controlled metadata must start");
            std::thread::sleep(Duration::from_millis(10));
        }
        let other = p.parent.join("other workspace");
        fs::create_dir(&other).unwrap();
        fs::write(other.join("other.c"), "int other;\n").unwrap();
        p.call("workspace.open", json!({"path":other}));
        p.wait("event.index.finished", |value| {
            value["context"]["cdbEntries"] == 0
        });
        fs::write(release, "finish").unwrap();
        p.wait("event.job.finished", |value| value["jobId"] == job);
        assert_eq!(
            p.call("index.status", json!({}))["context"]["cdbEntries"],
            0
        );
        assert_eq!(p.core.index.lock().unwrap().root, other);
    }
}

#[test]
fn c1_existing_automatic_retry_keeps_the_default_kit_and_the_absence_of_a_preset() {
    let compiler = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let mut p = fixture("existing-automatic-kit", compiler.to_str().unwrap());
    let path = p.root.join("CMakePresets.json");
    let mut presets: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    presets["configurePresets"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    presets["configurePresets"][0]
        .as_object_mut()
        .unwrap()
        .remove("inherits");
    presets["configurePresets"][0]["generator"] = json!("Ninja");
    fs::write(path, presets.to_string()).unwrap();
    fs::write(
        p.root.join(".kinein/toolchain.json"),
        json!({"schemaVersion":2,"kits":{"release local":{"selections":{"cCompiler":"clang"}}}})
            .to_string(),
    )
    .unwrap();
    configure(&mut p, None, false);
    assert!(
        context(&mut p)["effective"].is_null(),
        "partial first configure cache is not current context"
    );
    assert_eq!(
        crate::cmake::active_kit(&p.root),
        "",
        "project default must not become a named kit"
    );
    configure(&mut p, None, false);
    p.build(false);
    assert_eq!(crate::cmake::active_kit(&p.root), "");
    let ide = crate::cmake::ide_build_dir(&p.root);
    drop(crate::cmake::begin_configure(&p.root, &ide, None, "").unwrap());
    let (preset, source) = crate::cmake::choose_preset(&p.root, None).unwrap();
    assert!(
        preset.is_none(),
        "retry without preset must not pick a newly discovered preset"
    );
    assert_eq!(source, Some("incompleteConfigure"));
}
