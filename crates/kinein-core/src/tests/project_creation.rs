//! Escolhas do assistente chegam às ferramentas, sem revisão fixa ou instalação.

use crate::python::EnvironmentCreators;
use crate::{tools::ToolDetector, workspace};
use kinein_protocol::{JsonRpcRequest, WorkspaceProjectOptions, WorkspaceProjectTemplate};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

struct Directory(PathBuf);
impl Directory {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "kinein creation ação {} {name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn python_versions(bin: &Path) {
    for (name, version) in [("python3.9", "3.9.99"), ("python3.14", "3.14.99")] {
        crate::write_executable(
            bin.join(name),
            format!(
                "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'Python {version}'; else echo 'selected {version}'; fi\n"
            ),
        );
    }
}

#[test]
fn project_creation_enumerates_parallel_versions_and_preserves_python_selection() {
    let directory = Directory::new("python");
    let bin = directory.0.join("versions with spaces");
    std::fs::create_dir_all(&bin).unwrap();
    python_versions(&bin);
    let detector = ToolDetector::with_search_path(bin.as_os_str());
    let mut core = crate::Core::with_detector(detector);
    let result = core
        .handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.projectEnvironments",
            Some(json!({})),
        ))
        .response()
        .result
        .clone()
        .unwrap();
    let entries = result["environments"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .any(|entry| entry["version"] == "Python 3.9.99")
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["version"] == "Python 3.14.99")
    );
    for (name, executable, expected) in [
        (
            "old",
            bin.join(if cfg!(windows) {
                "python3.9.exe"
            } else {
                "python3.9"
            }),
            "3.9.99",
        ),
        (
            "new",
            bin.join(if cfg!(windows) {
                "python3.14.exe"
            } else {
                "python3.14"
            }),
            "3.14.99",
        ),
    ] {
        let result = core.handle_request(&JsonRpcRequest::new(2_i64, "workspace.createProject", Some(json!({"parent":directory.0,"name":name,"template":"python","options":{"executable":executable}})))).response().clone();
        assert!(result.error.is_none(), "{result:?}");
        let root = directory.0.join(name);
        let context = crate::python::env::python_env(
            &root,
            &crate::python::env::PythonTools {
                virtual_env: Some(directory.0.display().to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(context.interpreter, executable.display().to_string());
        assert_eq!(context.origin, "projeto");
        let output = Command::new(&context.interpreter)
            .arg(root.join("main.py"))
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
        assert!(
            !std::fs::read_to_string(root.join("pyproject.toml"))
                .unwrap()
                .contains("requires-python")
        );
        let tools = crate::python::env::PythonTools::default();
        assert!(
            !crate::python::status(&root, &tools, &EnvironmentCreators::default()).has_environment
        );
        let (command, _) = crate::python::create_environment_command(
            &root,
            kinein_protocol::PythonEnvironmentTool::Venv,
            &EnvironmentCreators::default(),
            None,
        )
        .unwrap();
        assert_eq!(command.get_program(), executable.as_os_str());
        std::fs::write(root.join(".kinein/python.json"), "{}").unwrap();
        assert!(crate::python::env::python_env(&root, &tools).is_none());
        assert!(
            crate::python::create_environment_command(
                &root,
                kinein_protocol::PythonEnvironmentTool::Venv,
                &EnvironmentCreators::default(),
                Some(Path::new("python3"))
            )
            .is_none()
        );
    }
}

#[test]
fn project_creation_cmake_choices_drive_real_c_and_cpp_without_defaults() {
    let directory = Directory::new("cmake");
    let detector = ToolDetector::from_environment();
    for (name, tool, template, standard, language) in [
        (
            "legacy",
            "gcc",
            WorkspaceProjectTemplate::CCmake,
            Some("90"),
            "C",
        ),
        (
            "modern",
            "clang",
            WorkspaceProjectTemplate::CCmake,
            Some("23"),
            "C",
        ),
        (
            "cpp",
            "g++",
            WorkspaceProjectTemplate::CppCmake,
            Some("17"),
            "CXX",
        ),
        (
            "automatic",
            "clang",
            WorkspaceProjectTemplate::CCmake,
            None,
            "C",
        ),
    ] {
        let executable = detector.find_in_path(tool).unwrap();
        let options = WorkspaceProjectOptions {
            executable: Some(executable.display().to_string()),
            standard: standard.map(str::to_owned),
        };
        workspace::create_project_with_options(&directory.0, name, template, &options).unwrap();
        let root = directory.0.join(name);
        let configured = Command::new("cmake")
            .args(["--preset", "debug", "-B", ".kinein/build"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            configured.status.success(),
            "{}",
            String::from_utf8_lossy(&configured.stderr)
        );
        let built = Command::new("cmake")
            .args(["--build", ".kinein/build", "--parallel", "2"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stdout)
        );
        let cdb: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".kinein/build/compile_commands.json")).unwrap(),
        )
        .unwrap();
        let command = cdb[0]["command"].as_str().unwrap();
        assert!(command.contains(executable.to_str().unwrap()), "{command}");
        let cmake = std::fs::read_to_string(root.join("CMakeLists.txt")).unwrap();
        if let Some(standard) = standard {
            assert!(cmake.contains(&format!("{language}_STANDARD {standard}")));
        } else {
            assert!(!command.contains("-std="));
            assert!(!cmake.contains("C_STANDARD"));
        }
    }
    let invalid = WorkspaceProjectOptions {
        executable: None,
        standard: Some("23)\nmessage(FATAL_ERROR injected)".to_owned()),
    };
    assert!(
        workspace::create_project_with_options(
            &directory.0,
            "invalid",
            WorkspaceProjectTemplate::CCmake,
            &invalid
        )
        .is_err()
    );
    assert!(!directory.0.join("invalid").exists());
}

#[test]
fn project_creation_local_rust_toolchain_and_edition_are_used_without_downloads() {
    let directory = Directory::new("rust");
    assert_no_implicit_rustup_install(&directory);
    let detector = ToolDetector::from_environment();
    if let Some(proxy) = detector.find_in_path("cargo").filter(|path| {
        path.parent().is_some_and(|parent| {
            crate::platform::executable_names("rustup")
                .iter()
                .any(|name| parent.join(name).is_file())
        })
    }) {
        let options = WorkspaceProjectOptions {
            executable: Some(proxy.display().to_string()),
            standard: None,
        };
        assert!(
            workspace::create_project_with_options(
                &directory.0,
                "proxy",
                WorkspaceProjectTemplate::RustCargo,
                &options
            )
            .is_err()
        );
        assert!(!directory.0.join("proxy").exists());
    }
    let toolchains = ToolDetector::from_environment()
        .project_environments()
        .environments;
    let toolchain = toolchains
        .iter()
        .find(|entry| entry.language == "rust")
        .expect("installed local Rust toolchain");
    let options = WorkspaceProjectOptions {
        executable: Some(toolchain.executable.clone()),
        standard: Some("2021".to_owned()),
    };
    workspace::create_project_with_options(
        &directory.0,
        "local_rust",
        WorkspaceProjectTemplate::RustCargo,
        &options,
    )
    .unwrap();
    let root = directory.0.join("local_rust");
    assert!(
        std::fs::read_to_string(root.join("Cargo.toml"))
            .unwrap()
            .contains("edition = \"2021\"")
    );
    let pinned = Command::new("rustup")
        .args(["which", "rustc"])
        .current_dir(&root)
        .env_remove("RUSTUP_TOOLCHAIN")
        .output()
        .unwrap();
    assert!(
        pinned.status.success(),
        "{}",
        String::from_utf8_lossy(&pinned.stderr)
    );
    assert_eq!(
        crate::platform::canonicalize(Path::new(String::from_utf8_lossy(&pinned.stdout).trim()))
            .unwrap(),
        crate::platform::canonicalize(&Path::new(&toolchain.executable).with_file_name("rustc"))
            .unwrap()
    );
    let built = Command::new("cargo")
        .args(["check", "--offline"])
        .current_dir(root)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env("CARGO_BUILD_JOBS", "2")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
}

fn assert_no_implicit_rustup_install(directory: &Directory) {
    let bin = directory.0.join("controlled toolchain/bin");
    std::fs::create_dir_all(&bin).unwrap();
    let cargo = crate::write_executable(
        bin.join("cargo"),
        "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'cargo 2099.0'; exit 0; fi\n[ \"$RUSTUP_AUTO_INSTALL\" = 0 ] || exit 91\nmkdir -p \"$5/src\"\nprintf '[package]\\nname = \"controlled\"\\nversion = \"0.1.0\"\\n' > \"$5/Cargo.toml\"\n",
    );
    crate::write_executable(bin.join("rustc"), "#!/bin/sh\necho 'rustc 2099.0'\n");
    let options = WorkspaceProjectOptions {
        executable: Some(cargo.display().to_string()),
        standard: None,
    };
    workspace::create_project_with_options(
        &directory.0,
        "controlled",
        WorkspaceProjectTemplate::RustCargo,
        &options,
    )
    .unwrap();
    assert!(directory.0.join("controlled/Cargo.toml").is_file());
}
