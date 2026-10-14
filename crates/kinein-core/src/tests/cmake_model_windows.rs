//! Os tres substitutos do modelo `CMake` no Windows (60 §3.3, D14, W6b3).
//! O reply e a CDB vem de projetos compilados pelo core com o MSVC real;
//! os testes sinteticos de GNU continuam em `cmake_model.rs`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::AtomicBool};

use kinein_protocol::{JsonRpcRequest, ProjectKind, RigorProfile};
use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::cmake::model::CmakeModel;

fn project(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-msvc-model-{name}", std::process::id()));
    std::fs::create_dir_all(root.join("include")).unwrap();
    std::fs::write(
        root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(app C CXX)\n\
         add_library(util STATIC util.c)\nadd_custom_target(all_gen)\n\
         file(MAKE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/gen)\n\
         add_custom_command(OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/gen/generated.cpp\n\
           COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_SOURCE_DIR}/generated.cpp\n\
             ${CMAKE_CURRENT_BINARY_DIR}/gen/generated.cpp DEPENDS generated.cpp)\n\
         add_executable(app main.cpp include/app.h ${CMAKE_CURRENT_BINARY_DIR}/gen/generated.cpp)\n\
         set_target_properties(app PROPERTIES CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON)\n\
         target_compile_definitions(app PRIVATE APP=1)\n\
         target_include_directories(app PRIVATE include)\n\
         target_link_libraries(app PRIVATE util)\nadd_dependencies(app all_gen)\n",
    )
    .unwrap();
    for (file, content) in [
        ("main.cpp", "#include <app.h>\nint main() { return 0; }\n"),
        ("util.c", "int util(void) { return 0; }\n"),
        ("generated.cpp", "int generated() { return 0; }\n"),
    ] {
        std::fs::write(root.join(file), content).unwrap();
    }
    std::fs::write(root.join("include").join("app.h"), "#pragma once\n").unwrap();
    let root = crate::platform::canonicalize(&root).unwrap();
    crate::cmake::write_file_api_query(&root).unwrap();
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
    .expect("o build roda; sem Visual Studio a prova reprova");
    assert!(built.success, "{lines:#?}");
    root
}

fn rpc(core: &mut crate::Core, method: &str, params: Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)));
    let response = outcome.response();
    assert!(response.error.is_none(), "{method}: {:?}", response.error);
    response.result.clone().unwrap()
}

fn open(root: &Path, name: &str) -> crate::Core {
    let mut core = core_with_empty_search_path(name);
    rpc(&mut core, "workspace.open", json!({ "path": root }));
    core
}

#[test]
fn the_real_windows_file_api_reply_becomes_the_model() {
    let root = project("read");
    let build = crate::cmake::build_dir(&root);
    let model = CmakeModel::load(&build).expect("o reply do CMake real");
    assert_eq!(model.source_dir, root);
    assert_eq!(model.build_dir, build);
    let app = model.targets.iter().find(|t| t.name == "app").unwrap();
    assert_eq!(app.kind, "EXECUTABLE");
    // O MSVC lista tambem o PDB, que o fixture do Linux nao mostrava.
    assert_eq!(
        app.artifacts,
        vec![build.join("app.exe"), build.join("app.pdb")]
    );
    assert!(app.artifacts.iter().all(|p| p.is_file()));
    assert!(app.dependencies.iter().any(|d| d == "util"));
    assert!(app.dependencies.iter().any(|d| d == "all_gen"));
    let generated = app.sources.iter().find(|s| s.generated).unwrap();
    assert_eq!(generated.path, build.join("gen").join("generated.cpp"));
    let header = app
        .sources
        .iter()
        .find(|s| s.path.ends_with("app.h"))
        .unwrap();
    assert_eq!(header.compile_group, None);
    assert_eq!(model.targets_for(&header.path)[0].name, "app");
    let (target, group) = model.compile_group_for(&root.join("main.cpp")).unwrap();
    assert_eq!(target.name, "app");
    assert_eq!(group.language, "CXX");
    assert_eq!(group.standard.as_deref(), Some("20"));
    assert!(group.defines.iter().any(|d| d == "APP=1"));
    assert!(
        group
            .includes
            .iter()
            .any(|p| Path::new(p) == root.join("include"))
    );
    assert!(group.fragments.iter().any(|f| f.contains("/EHsc")));
    assert!(group.fragments.iter().any(|f| f.contains("std:c++20")));
    let util = model.targets.iter().find(|t| t.name == "util").unwrap();
    assert_eq!(util.kind, "STATIC_LIBRARY");
    assert_eq!(util.artifacts, vec![build.join("util.lib")]);
    assert!(util.artifacts[0].is_file());
    assert_eq!(
        model
            .compile_group_for(&root.join("util.c"))
            .unwrap()
            .1
            .language,
        "C"
    );
    for language in ["C", "CXX"] {
        let compiler = model.compiler_for(language).unwrap();
        assert_eq!(compiler.id.as_deref(), Some("MSVC"));
        assert!(Path::new(&compiler.path).is_file());
        assert_eq!(Path::new(&compiler.path).file_name().unwrap(), "cl.exe");
        assert!(compiler.version.as_ref().is_some_and(|v| !v.is_empty()));
    }
}

#[test]
fn windows_targets_list_carries_the_real_model_and_hides_utilities() {
    let root = project("targets");
    let mut core = open(&root, "msvc-model-targets");
    let result = rpc(&mut core, "cmake.targets.list", json!({}));
    assert_eq!(result["origin"], "fileApi");
    let targets = result["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 2, "{result}");
    let app = targets.iter().find(|t| t["name"] == "app").unwrap();
    assert_eq!(app["kind"], "executable");
    assert_eq!(
        app["artifacts"][0],
        crate::cmake::build_dir(&root)
            .join("app.exe")
            .display()
            .to_string()
    );
    assert_eq!(app["sources"], 2);
    // O CMake lista a fonte gerada e tambem a regra que a produz.
    assert_eq!(app["generatedSources"], 2);
    assert_eq!(app["standard"], "20");
    assert_eq!(app["languages"], json!(["CXX"]));
    assert_eq!(app["includes"], 1);
    assert_eq!(app["defines"], 1);
    assert!(
        app["dependencies"]
            .as_array()
            .unwrap()
            .contains(&json!("util"))
    );
    assert!(app.get("sysroot").is_none());
    let util = targets.iter().find(|t| t["name"] == "util").unwrap();
    assert_eq!(util["kind"], "staticLibrary");
    assert_eq!(util["languages"], json!(["C"]));
    assert_eq!(
        util["artifacts"][0],
        crate::cmake::build_dir(&root)
            .join("util.lib")
            .display()
            .to_string()
    );
}

#[test]
fn windows_index_context_uses_the_real_file_api_then_prefers_the_msvc_cdb() {
    let root = project("context");
    let build = crate::cmake::build_dir(&root);
    let cdb = build.join("compile_commands.json");
    let saved = build.join("compile_commands.saved.json");
    std::fs::rename(&cdb, &saved).unwrap();
    let mut core = open(&root, "msvc-model-context");
    let main = rpc(&mut core, "index.context", json!({ "path": "main.cpp" }));
    assert_eq!(main["targets"], json!(["app"]));
    assert!(
        main["source"]
            .as_str()
            .unwrap()
            .starts_with("file-api codemodel-v2 (target app)")
    );
    let unit = &main["unit"];
    assert_eq!(
        Path::new(unit["compiler"].as_str().unwrap())
            .file_name()
            .unwrap(),
        "cl.exe"
    );
    assert_eq!(unit["directory"], build.display().to_string());
    assert_eq!(unit["standard"], "20");
    assert_eq!(
        unit["includes"],
        json!([root.join("include").display().to_string()])
    );
    assert_eq!(unit["defines"], json!(["APP=1"]));
    assert!(
        unit["arguments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("std:c++20"))
    );
    let header = rpc(
        &mut core,
        "index.context",
        json!({ "path": "include/app.h" }),
    );
    assert_eq!(header["targets"], json!(["app"]));
    assert!(header.get("unit").is_none());
    let util = rpc(&mut core, "index.context", json!({ "path": "util.c" }));
    assert_eq!(util["targets"], json!(["util"]));
    assert!(util["unit"].get("standard").is_none());
    std::fs::rename(&saved, &cdb).unwrap();
    core.observe_notification(&JsonRpcRequest::notification(
        "event.cmake.finished",
        Some(json!({ "success": true })),
    ));
    let main = rpc(&mut core, "index.context", json!({ "path": "main.cpp" }));
    assert_eq!(main["targets"], json!(["app"]));
    assert!(
        main["source"]
            .as_str()
            .unwrap()
            .starts_with("compile_commands.json")
    );
    assert_eq!(main["unit"]["standard"], "c++20");
    assert!(
        main["unit"]["defines"]
            .as_array()
            .unwrap()
            .contains(&json!("APP=1"))
    );
}
