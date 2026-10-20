//! Mixed workspaces must keep the primary project's Python transport.
#![cfg(unix)]

use std::path::PathBuf;
use std::time::Duration;

use kinein_protocol::JsonRpcRequest;
use serde_json::json;

fn workspace(name: &str) -> PathBuf {
    let path = std::env::temp_dir()
        .join("kinein-project-contexts")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("examples")).unwrap();
    std::fs::write(
        path.join("CMakeLists.txt"),
        "include($ENV{IDF_PATH}/tools/cmake/project.cmake)\nproject(demo)\n",
    )
    .unwrap();
    std::fs::write(path.join("examples/main.py"), "import machine\n").unwrap();
    std::fs::write(path.join("generate.py"), "print('host-tool')\n").unwrap();
    crate::platform::canonicalize(&path).unwrap()
}

#[test]
fn a_nested_micropython_example_does_not_redirect_host_scripts() {
    let _serial = crate::serialize_executables();
    let root = workspace("host-script");
    let python = root.join(".venv/bin/python");
    std::fs::create_dir_all(python.parent().unwrap()).unwrap();
    crate::write_executable(&python, "#!/bin/sh\necho \"host-interpreter $*\"\n");
    assert!(!crate::project::e_micropython(&root));

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = super::core_with_empty_search_path("mixed-project-context");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));
    assert!(opened.response().error.is_none());
    let started = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "run.script",
        Some(json!({ "path": root.join("generate.py") })),
    ));
    let result = started
        .response()
        .result
        .as_ref()
        .expect("host script runs");
    assert_eq!(result["command"], ".venv/bin/python 'generate.py'");
    let id = result["terminalId"].as_str().unwrap();
    let (lines, exit) = super::terminal_run_until_closed(&receiver, id, Duration::from_secs(10));
    assert_eq!(exit, Some(0), "{lines:?}");
    assert!(lines.iter().any(|line| line.contains("host-interpreter")));
}

#[test]
fn mixed_frameworks_keep_the_target_and_sdk_requirements_with_the_owner() {
    use kinein_protocol::Framework;

    let root = workspace("model");
    let environment = crate::project::sdk::Environment {
        var: &|_| None,
        binary: &|_| None,
        home: None,
    };
    let model = crate::project::model_in(&root, None, None, &environment);
    assert_eq!(model.frameworks[0].framework, Framework::EspIdf);
    assert!(!model.sdks.iter().any(|sdk| sdk.id == "mpremote"));

    std::fs::write(
        root.join("platformio.ini"),
        "[env:bench]\nplatform = espressif32\nboard = esp32dev\nframework = espidf\n",
    )
    .unwrap();
    let model = crate::project::model_in(&root, None, None, &environment);
    assert_eq!(model.frameworks[0].framework, Framework::PlatformIo);
    assert_eq!(model.target.flash_engine.as_deref(), Some("platformio"));
    assert_eq!(model.sdks.len(), 1);
    assert_eq!(model.sdks[0].id, "pio");

    std::fs::remove_file(root.join("platformio.ini")).unwrap();
    std::fs::remove_file(root.join("CMakeLists.txt")).unwrap();
    std::fs::create_dir_all(root.join("conf")).unwrap();
    std::fs::write(root.join("conf/local.conf"), "MACHINE = \"qemux86-64\"\n").unwrap();
    let model = crate::project::model_in(&root, None, None, &environment);
    assert_eq!(model.frameworks[0].framework, Framework::Yocto);
    assert_eq!(model.target.family.as_deref(), Some("linux"));
    assert!(!crate::project::e_micropython(&root));
    assert!(!model.sdks.iter().any(|sdk| sdk.id == "mpremote"));

    std::fs::remove_file(root.join("conf/local.conf")).unwrap();
    assert!(crate::project::e_micropython(&root));
    let model = crate::project::model_in(&root, None, None, &environment);
    assert!(model.sdks.iter().any(|sdk| sdk.id == "mpremote"));
}

#[test]
fn linux_application_model_follows_the_effective_kit_and_refreshes_after_save() {
    for preset in ["", "linux-sdk"] {
        let root = workspace(if preset.is_empty() {
            "linux-default"
        } else {
            "linux-named"
        });
        std::fs::remove_dir_all(root.join("examples")).unwrap();
        std::fs::write(root.join("CMakeLists.txt"), "project(linux_app C)\n").unwrap();
        let build = crate::cmake::build_dir(&root);
        std::fs::create_dir_all(&build).unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut core = super::core_with_empty_search_path(&format!("linux-context-{preset}"));
        core.enable_lsp(sender);
        let opened = core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": root })),
        ));
        assert!(opened.response().error.is_none());
        if !preset.is_empty() {
            // Model a completed configure; the test exercises context, not CMake.
            std::fs::write(build.join("CMakeCache.txt"), "").unwrap();
            crate::cmake::record_preset(&root, Some(preset));
        }
        let saved = core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "toolchain.setKit",
            Some(json!({
                "preset": preset, "targetTriple": "aarch64-linux-gnu"
            })),
        ));
        assert!(saved.response().error.is_none(), "{:?}", saved.response());
        let result = core.handle_request(&JsonRpcRequest::new(3_i64, "project.model", None));
        let model = result.response().result.as_ref().unwrap();
        assert_eq!(
            model["target"]["triple"], "aarch64-linux-gnu",
            "{preset}: {model}"
        );
        assert_eq!(model["target"]["family"], "linux");
        assert_eq!(model["embedded"], true);
        assert!(model["frameworks"].as_array().unwrap().is_empty());
        assert!(model["target"].get("flashEngine").is_none());
        assert!(
            model["target"]["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|line| {
                    line.as_str()
                        .is_some_and(|value| value.contains("triple: kit"))
                })
        );
        let event = receiver
            .try_iter()
            .filter(|event| event.method == "event.project.changed")
            .last()
            .unwrap();
        assert_eq!(event.params.as_ref().unwrap()["target"]["family"], "linux");
        let cleared = core.handle_request(&JsonRpcRequest::new(
            4_i64,
            "toolchain.setKit",
            Some(json!({
                "preset": preset, "targetTriple": ""
            })),
        ));
        assert!(cleared.response().error.is_none());
        let result = core.handle_request(&JsonRpcRequest::new(5_i64, "project.model", None));
        let model = result.response().result.as_ref().unwrap();
        assert_eq!(model["embedded"], false);
        assert!(model["target"].get("triple").is_none());
    }
}
