//! C1: projeto manual, tentativa incompleta e evidência anterior.

use super::{Project, environment::project};
use crate::index::context::{CompileContext, ContextTools};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};

pub(super) fn fixture(name: &str, compiler: &str) -> Project {
    let mut p = project(name);
    fs::create_dir(&p.root).unwrap();
    fs::write(
        p.root.join("main.c"),
        "#ifdef __cplusplus\n#error C only\n#endif\nint main(void) { return 0; }\n",
    )
    .unwrap();
    fs::write(p.root.join("CMakeLists.txt"), "cmake_minimum_required(VERSION 3.24)\nproject(existing LANGUAGES C)\nif(C1_FAIL)\n  message(FATAL_ERROR \"controlled configure failure\")\nendif()\nadd_executable(existing main.c)\ntarget_compile_options(existing PRIVATE -std=gnu17)\n").unwrap();
    fs::write(
        p.root.join("local toolchain.cmake"),
        format!("set(CMAKE_C_COMPILER [==[{compiler}]==])\n"),
    )
    .unwrap();
    let presets = json!({"version":6,"configurePresets":[
        {"name":"debug local","generator":"Ninja","toolchainFile":"${sourceDir}/local toolchain.cmake",
         "cacheVariables":{"CMAKE_BUILD_TYPE":"Debug","C1_FAIL":"OFF"}},
        {"name":"release local","inherits":"debug local","cacheVariables":{"CMAKE_BUILD_TYPE":"Release","C1_FAIL":"ON"}}
    ]});
    fs::write(p.root.join("CMakePresets.json"), presets.to_string()).unwrap();
    fs::write(
        p.root.join(".clangd"),
        "CompileFlags:\n  Add: [-DMANUAL_CLANGD]\n",
    )
    .unwrap();
    fs::write(p.root.join("compile_flags.txt"), "-DMANUAL_FLAGS\n").unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    p
}

pub(super) fn configure(p: &mut Project, preset: Option<&str>, success: bool) {
    let job = p.call("cmake.configure", json!({"preset":preset}))["jobId"].clone();
    let result = p.wait("event.cmake.finished", |e| e["jobId"] == job);
    assert_eq!(result["success"], success, "{result}");
}

pub(super) fn context(p: &mut Project) -> Value {
    p.call("index.context", json!({"path":p.root.join("main.c")}))
}

pub(super) fn current(p: &mut Project) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(event) = p.events.try_recv() {
            p.core.observe_notification(&event);
        }
        let value = context(p);
        if value["effective"]["standard"]["value"] == "gnu17" && value["staleBecause"].is_null() {
            return value;
        }
        assert!(std::time::Instant::now() < deadline, "{value}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn c1_existing_failed_configure_preserves_selection_and_marks_previous_commands() {
    for compiler in ["gcc", "clang"] {
        let executable = crate::tools::ToolDetector::from_environment()
            .find_in_path(compiler)
            .unwrap();
        let mut p = fixture(
            &format!("existing-failed-{compiler}"),
            executable.to_str().unwrap(),
        );
        let preserved: Vec<_> = [
            "main.c",
            "CMakeLists.txt",
            "local toolchain.cmake",
            ".clangd",
            "compile_flags.txt",
        ]
        .map(|name| (name, fs::read(p.root.join(name)).unwrap()))
        .into();
        configure(&mut p, Some("debug local"), true);
        p.build(true);
        let before = current(&mut p);
        assert_eq!(
            before["effective"]["compiler"]["value"],
            executable.to_str().unwrap()
        );
        assert_eq!(before["effective"]["compiler"]["origin"], "cdb");
        configure(&mut p, Some("release local"), false);
        let status = p.call("cmake.status", json!({}));
        assert_eq!(
            status["preset"], "debug local",
            "last success remains distinct"
        );
        assert_eq!(
            status["cdbStale"], true,
            "old CDB is not current evidence: {status}"
        );
        let stale = context(&mut p);
        assert!(
            stale["staleBecause"]
                .as_str()
                .unwrap()
                .contains("release local"),
            "{stale}"
        );
        assert!(stale["effective"].is_null(), "{stale}");
        assert_eq!(
            stale["unit"]["standard"], "gnu17",
            "history remains inspectable"
        );
        assert!(
            crate::index::context::verification_context(&p.root, &p.root.join("main.c")).is_err()
        );
        // Reabertura não transforma falha persistida em sucesso nem perde a escolha.
        p.call("workspace.close", json!({}));
        p.call("workspace.open", json!({"path":p.root}));
        p.wait("event.index.finished", |_| true);
        configure(&mut p, None, false);
        let mut presets: Value =
            serde_json::from_slice(&fs::read(p.root.join("CMakePresets.json")).unwrap()).unwrap();
        presets["configurePresets"][1]["cacheVariables"]["C1_FAIL"] = json!("OFF");
        fs::write(p.root.join("CMakePresets.json"), presets.to_string()).unwrap();
        configure(&mut p, None, true);
        p.build(true);
        current(&mut p);
        assert_eq!(p.call("cmake.status", json!({}))["preset"], "release local");
        let cdb = fs::read_to_string(p.root.join(".kinein/build/compile_commands.json")).unwrap();
        assert!(cdb.contains("-O3") && cdb.contains("-std=gnu17"), "{cdb}");
        for (name, bytes) in preserved {
            assert_eq!(fs::read(p.root.join(name)).unwrap(), bytes, "{name}");
        }
    }
}

#[test]
fn c1_existing_removed_compiler_and_replaced_database_never_claim_current_context() {
    let executable = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let mut p = project("existing-removed");
    let alias = crate::write_executable(
        p.parent.join("gcc local ação"),
        format!("#!/bin/sh\nexec '{}' \"$@\"\n", executable.display()),
    );
    // Reuse the manual fixture with an alternate path to the real driver.
    let mut opened = fixture("existing-removed-opened", alias.to_str().unwrap());
    configure(&mut opened, Some("debug local"), true);
    opened.build(true);
    current(&mut opened);
    fs::remove_file(&alias).unwrap();
    let stale = context(&mut opened);
    assert!(
        stale["staleBecause"]
            .as_str()
            .unwrap()
            .contains("compilador"),
        "{stale}"
    );
    assert!(stale["effective"].is_null(), "{stale}");
    fs::write(
        opened.root.join("main.c"),
        "int main(void) { return 0; } /* force compilation */\n",
    )
    .unwrap();
    opened.build(false);
    assert_eq!(
        context(&mut opened)["unit"]["compiler"],
        alias.to_str().unwrap()
    );
    crate::write_executable(
        &alias,
        format!("#!/bin/sh\nexec '{}' \"$@\"\n", executable.display()),
    );
    configure(&mut opened, Some("debug local"), true);
    opened.build(true);
    current(&mut opened);
    let loaded = CompileContext::load(&opened.root, &ContextTools::default());
    let cdb = opened.root.join(".kinein/build/compile_commands.json");
    let mut entries: Value = serde_json::from_slice(&fs::read(&cdb).unwrap()).unwrap();
    entries[0]["command"] = json!(format!(
        "'{}' -std=gnu11 -c '{}'",
        executable.display(),
        opened.root.join("main.c").display()
    ));
    fs::write(&cdb, entries.to_string()).unwrap();
    let stale = loaded.for_file(&opened.root, Path::new("main.c"), "c");
    assert!(
        stale.effective.is_none(),
        "old snapshot cannot describe replaced bytes"
    );
    // Keep the owner alive until the alternate installation is no longer used.
    p.call("core.ping", json!({}));
}
