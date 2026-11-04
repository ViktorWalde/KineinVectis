//! C1: criação, falhas e retomada usando produtores reais do projeto.

use super::Project;
use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{fs, path::Path, sync::mpsc, time::Duration};

fn project(name: &str) -> Project {
    let parent = std::env::temp_dir().join(format!("kinein C1 ação {} {name}", std::process::id()));
    fs::create_dir_all(&parent).unwrap();
    let (sender, events) = mpsc::channel();
    let mut core = super::super::core_with_empty_search_path(name);
    core.enable_lsp(sender);
    Project {
        root: parent.join("starter"),
        parent,
        core,
        events,
    }
}

fn create(
    p: &mut Project,
    executable: &Path,
    standard: Option<&str>,
) -> kinein_protocol::JsonRpcResponse {
    p.core
        .handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.createProject",
            Some(
                json!({"parent":p.parent,"name":"starter","template":"cCmake",
            "options":{"executable":executable,"standard":standard}}),
            ),
        ))
        .response()
        .clone()
}

fn configure(p: &mut Project, preset: &str) {
    let job = p.call("cmake.configure", json!({"preset":preset}))["jobId"].clone();
    let result = p.wait("event.cmake.finished", |e| e["jobId"] == job);
    assert_eq!(result["success"], true, "{result}");
    p.build(true);
}

fn context(p: &mut Project, executable: &Path, standard: Option<&str>) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(event) = p.events.try_recv() {
            p.core.observe_notification(&event);
        }
        let value = p.call("index.context", json!({"path":p.root.join("src/main.c")}));
        if value["unit"]["compiler"] == executable.to_str().unwrap() {
            assert_eq!(value["language"], "c", "{value}");
            assert_eq!(value["unit"]["standard"], json!(standard), "{value}");
            assert_eq!(value["effective"]["compiler"]["origin"], "cdb");
            assert_eq!(value["targets"], json!(["starter"]));
            return;
        }
        assert!(std::time::Instant::now() < deadline, "{value}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn c1_literal_dialects_build_and_survive_reopen_configuration_and_workspace_changes() {
    let detector = crate::tools::ToolDetector::from_environment();
    for tool in ["gcc", "clang"] {
        let executable = detector.find_in_path(tool).unwrap();
        for standard in [Some("gnu17"), Some("iso9899:1990"), Some("c2y"), None] {
            let mut p = project(&format!("{tool}-{}", standard.unwrap_or("auto")));
            let created = create(&mut p, &executable, standard);
            assert!(created.error.is_none(), "{created:?}");
            p.wait("event.index.finished", |_| true);
            let source = p.root.join("src/main.c");
            fs::write(&source, "#if defined(__cplusplus) || !defined(__STDC__)\n#error C only\n#endif\nint main(void) { return 0; }\n").unwrap();
            fs::write(
                p.root.join(".clangd"),
                "CompileFlags:\n  Add: [-DMANUAL_C1]\n",
            )
            .unwrap();
            let preserved: Vec<_> = [
                "src/main.c",
                "CMakeLists.txt",
                "CMakePresets.json",
                ".clangd",
            ]
            .map(|name| (name, fs::read(p.root.join(name)).unwrap()))
            .into();
            configure(&mut p, "debug");
            context(&mut p, &executable, standard);
            p.test();
            let cdb: Value = serde_json::from_slice(
                &fs::read(p.root.join(".kinein/build/compile_commands.json")).unwrap(),
            )
            .unwrap();
            let command = cdb[0]["command"].as_str().unwrap();
            if let Some(standard) = standard {
                assert!(command.contains(&format!("-std={standard}")), "{command}");
            } else {
                assert!(!command.contains("-std="), "{command}");
            }
            // Abrir outra raiz e reiniciar o core não depende de estado em memória.
            let other = p.parent.join("other");
            fs::create_dir(&other).unwrap();
            p.call("workspace.open", json!({"path":other}));
            p.call("workspace.close", json!({}));
            let (sender, events) = mpsc::channel();
            let mut core = super::super::core_with_empty_search_path("c1-reopen");
            core.enable_lsp(sender);
            p.core = core;
            p.events = events;
            p.call("workspace.open", json!({"path":p.root}));
            p.call("workspace.trust", json!({}));
            p.wait("event.index.finished", |_| true);
            assert_eq!(p.call("cmake.status", json!({}))["preset"], "debug");
            context(&mut p, &executable, standard);
            configure(&mut p, "release");
            context(&mut p, &executable, standard);
            assert_eq!(p.call("cmake.status", json!({}))["preset"], "release");
            assert!(
                fs::read_to_string(p.root.join(".kinein/build/CMakeCache.txt"))
                    .unwrap()
                    .contains("CMAKE_BUILD_TYPE:STRING=Release")
            );
            for (name, bytes) in preserved {
                assert_eq!(fs::read(p.root.join(name)).unwrap(), bytes, "{name}");
            }
        }
    }
}

#[test]
fn c1_invalid_driver_dialect_and_collision_preserve_workspace_and_allow_retry() {
    let detector = crate::tools::ToolDetector::from_environment();
    let gcc = detector.find_in_path("gcc").unwrap();
    let mut p = project("invalid");
    let existing = p.parent.join("existing");
    fs::create_dir(&existing).unwrap();
    let buffer = existing.join("buffer.c");
    fs::write(&buffer, "int saved;\n").unwrap();
    p.call("workspace.open", json!({"path":existing}));
    let bad_path = p.parent.join("removed-gcc");
    for (executable, standard) in [
        (detector.find_in_path("g++").unwrap(), None),
        (detector.find_in_path("clang++").unwrap(), Some("17")),
        (bad_path, Some("17")),
        (gcc.clone(), Some("vendor-C2099")),
        (gcc.clone(), Some("gnu17;message(FATAL_ERROR injected)")),
        (gcc.clone(), Some("gnu17 -DFORCED")),
    ] {
        let error = create(&mut p, &executable, standard);
        assert!(error.error.is_some(), "{error:?}");
        assert!(!p.root.exists(), "falha não cria destino");
        assert_eq!(
            p.call("workspace.status", json!({}))["workspace"]["root"],
            existing.to_str().unwrap()
        );
        assert_eq!(fs::read_to_string(&buffer).unwrap(), "int saved;\n");
    }
    assert!(create(&mut p, &gcc, Some("gnu17")).error.is_none());
    p.wait("event.index.finished", |_| true);
    configure(&mut p, "debug");
    context(&mut p, &gcc, Some("gnu17"));
    let original = fs::read(p.root.join("src/main.c")).unwrap();
    assert!(create(&mut p, &gcc, Some("gnu11")).error.is_some());
    assert_eq!(fs::read(p.root.join("src/main.c")).unwrap(), original);
    assert!(
        fs::read_to_string(p.root.join("CMakeLists.txt"))
            .unwrap()
            .contains("-std=gnu17")
    );
}

#[test]
fn c1_future_literal_without_driver_is_preserved_but_invalid_syntax_is_refused() {
    use kinein_protocol::{WorkspaceProjectOptions, WorkspaceProjectTemplate};
    let p = project("tokens");
    for standard in ["vendor-C2099", "iso9899:2099", "c2y"] {
        let options = WorkspaceProjectOptions {
            executable: None,
            standard: Some(standard.to_owned()),
        };
        let opened = crate::workspace::create_project_with_options(
            &p.parent,
            standard.replace(':', "_").as_str(),
            WorkspaceProjectTemplate::CCmake,
            &options,
        )
        .unwrap();
        assert!(
            fs::read_to_string(Path::new(&opened.root).join("CMakeLists.txt"))
                .unwrap()
                .contains(&format!("-std={standard})"))
        );
    }
    for standard in [
        "gnu17;injected",
        "c17)\nmessage(FATAL_ERROR injected)",
        "${bad}",
        "gnu17 -DBAD",
        "c17\"",
        "0",
        "017",
    ] {
        let options = WorkspaceProjectOptions {
            executable: None,
            standard: Some(standard.to_owned()),
        };
        assert!(
            crate::workspace::create_project_with_options(
                &p.parent,
                "bad",
                WorkspaceProjectTemplate::CCmake,
                &options,
            )
            .is_err(),
            "{standard}"
        );
        assert!(!p.parent.join("bad").exists());
    }
    // A propriedade futura não vira flag inventada pela sonda do driver.
    let executable = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let options = WorkspaceProjectOptions {
        executable: Some(executable.display().to_string()),
        standard: Some("2099".to_owned()),
    };
    let opened = crate::workspace::create_project_with_options(
        &p.parent,
        "future_numeric",
        WorkspaceProjectTemplate::CCmake,
        &options,
    )
    .unwrap();
    let result = std::process::Command::new("cmake")
        .args(["--preset", "debug"])
        .current_dir(&opened.root)
        .output()
        .unwrap();
    assert!(
        !result.status.success(),
        "CMake must refuse an unsupported required standard"
    );
    assert!(
        fs::read_to_string(Path::new(&opened.root).join("CMakeLists.txt"))
            .unwrap()
            .contains("C_STANDARD 2099")
    );
}

#[test]
fn c1_validation_uses_continuations_and_refuses_a_changed_workspace_before_creation() {
    let mut p = project("deferred");
    let existing = p.parent.join("existing");
    fs::create_dir(&existing).unwrap();
    p.call("workspace.open", json!({"path":existing}));
    let (sender, pending) = mpsc::channel::<crate::Continuation>();
    p.core.enable_continuations(sender);
    let (sender, responses) = mpsc::channel();
    p.core.enable_deferred_responses(sender);
    let listing = p.core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "workspace.projectEnvironments",
        Some(json!({})),
    ));
    assert!(listing.is_deferred(), "{listing:?}");
    assert_eq!(
        responses.recv_timeout(Duration::from_secs(8)).unwrap().id,
        Some(json!(4))
    );
    let executable = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let params = json!({"parent":p.parent,"name":"starter","template":"cCmake",
        "options":{"executable":executable,"standard":"gnu17"}});
    let result = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.createProject",
        Some(params.clone()),
    ));
    assert!(result.is_deferred(), "{result:?}");
    assert!(!p.root.exists());
    p.call("core.ping", json!({}));
    let finish = pending.recv_timeout(Duration::from_secs(8)).unwrap();
    assert!(!p.root.exists(), "validação não gera arquivos do projeto");
    let other = p.parent.join("other");
    fs::create_dir(&other).unwrap();
    p.call("workspace.open", json!({"path":other}));
    let refused = finish(&mut p.core);
    assert!(refused.error.unwrap().message.contains("workspace mudou"));
    assert!(!p.root.exists());
    assert_eq!(
        p.call("workspace.status", json!({}))["workspace"]["root"],
        other.to_str().unwrap()
    );
    let result = p.core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "workspace.createProject",
        Some(params),
    ));
    assert!(result.is_deferred());
    let finish = pending.recv_timeout(Duration::from_secs(8)).unwrap();
    assert!(finish(&mut p.core).error.is_none());
    assert!(p.root.join("src/main.c").is_file());
}

#[test]
fn c1_alternate_driver_path_removed_after_selection_and_probe_deadline_are_recoverable() {
    let mut p = project("alternate");
    let executable = crate::tools::ToolDetector::from_environment()
        .find_in_path("gcc")
        .unwrap();
    let bin = p.parent.join("instalação local GCC");
    fs::create_dir(&bin).unwrap();
    let log = bin.join("arguments.log");
    let forward = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec '{}' \"$@\"\n",
        log.display(),
        executable.display()
    );
    let alias = crate::write_executable(bin.join("gcc"), &forward);
    let entries =
        crate::tools::ToolDetector::with_search_path(bin.as_os_str()).project_environments();
    assert!(
        entries
            .environments
            .iter()
            .any(|e| e.executable == alias.to_str().unwrap())
    );
    fs::remove_file(&alias).unwrap();
    assert!(create(&mut p, &alias, None).error.is_some());
    assert!(!p.root.exists());
    let slow = crate::write_executable(
        bin.join("slow-gcc"),
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'gcc controlled'; else sleep 30; fi\n",
    );
    let start = std::time::Instant::now();
    let failed = create(&mut p, &slow, None);
    assert!(failed.error.unwrap().message.contains("prazo"));
    assert!(start.elapsed() < Duration::from_secs(8));
    assert!(!p.root.exists());
    crate::write_executable(bin.join("gcc"), &forward);
    assert!(create(&mut p, &alias, Some("17")).error.is_none());
    assert!(
        !fs::read_to_string(&log).unwrap().contains("-std="),
        "numeric standard spelling belongs to CMake, not the driver probe"
    );
    p.wait("event.index.finished", |_| true);
    configure(&mut p, "debug");
    context(&mut p, &alias, Some("c17"));
}
