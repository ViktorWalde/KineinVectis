//! W6b4: LLVM real sobre a compilation database produzida pelo MSVC.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::{Duration, Instant},
};

use kinein_protocol::{JsonRpcRequest, ProjectKind, RigorProfile};
use serde_json::{Value, json};

use crate::{Core, tools::ToolDetector};

const SOURCE: &str = "#include <vector>\n#include <windows.h>\n#include <app.h>\nint main() {\n    std::vector<int> values{answer()};\n    return values.front() == 7 ? 0 : 1;\n}\n";

fn rpc(core: &mut Core, method: &str, params: Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)));
    let response = outcome.response();
    assert!(response.error.is_none(), "{method}: {:?}", response.error);
    response.result.clone().unwrap()
}

fn project() -> PathBuf {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-clangd-windows", std::process::id()));
    std::fs::create_dir_all(root.join("include")).unwrap();
    std::fs::write(
        root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(clangd_proof CXX)\n\
         add_executable(app main.cpp)\n\
         set_target_properties(app PROPERTIES CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON)\n\
         target_compile_definitions(app PRIVATE APP=1)\n\
         target_include_directories(app PRIVATE include)\n",
    )
    .unwrap();
    std::fs::write(root.join("main.cpp"), SOURCE).unwrap();
    std::fs::write(
        root.join("include/app.h"),
        "#pragma once\ninline int answer() { return 7; }\n",
    )
    .unwrap();
    let root = crate::platform::canonicalize(&root).unwrap();
    crate::cmake::write_file_api_query(&root).unwrap();
    let mut lines = Vec::new();
    let built = crate::build::run_build(
        &root,
        ProjectKind::Cmake,
        RigorProfile::Balanced,
        &crate::toolchain::Toolchain::resolve(&root, &[]),
        &crate::build::BuildTools::default(),
        &Arc::new(AtomicBool::new(false)),
        &mut |event| {
            if let crate::build::BuildEvent::Output { line, .. } = event {
                lines.push(line);
            }
        },
    )
    .expect("build C++20 com MSVC");
    assert!(built.success, "{lines:#?}");
    let cdb = std::fs::read_to_string(crate::cmake::build_dir(&root).join("compile_commands.json"))
        .unwrap();
    assert!(cdb.contains("cl.exe") && cdb.contains("-DAPP=1"), "{cdb}");
    root
}

fn diagnostics_until(
    core: &mut Core,
    events: &mpsc::Receiver<JsonRpcRequest>,
    path: &Path,
    resync_from_disk: bool,
    expected: impl Fn(&[Value]) -> bool,
) -> Vec<Value> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut seen = Vec::new();
    let mut status = Vec::new();
    let mut next_resync = Instant::now();
    while Instant::now() < deadline {
        // O handshake pode passar da graca inicial: a UI tambem resincroniza.
        if resync_from_disk && Instant::now() >= next_resync {
            rpc(core, "fs.read", json!({ "path": path }));
            next_resync = Instant::now() + Duration::from_millis(500);
        }
        if let Ok(event) = events.recv_timeout(Duration::from_millis(300)) {
            if event.method == "event.lsp.status"
                && event
                    .params
                    .as_ref()
                    .is_some_and(|p| p["status"] == "failed")
            {
                panic!("clangd falhou: {:?}", event.params);
            }
            if status.len() < 12
                && matches!(event.method.as_str(), "event.lsp.status" | "event.lsp.log")
            {
                status.push((event.method.clone(), event.params.clone()));
            }
            if event.method == "event.lsp.diagnostics" {
                let params = event.params.unwrap();
                status.push(("diagnostics-path".to_owned(), Some(params["path"].clone())));
                if params["path"] == path.display().to_string() {
                    let diagnostics = params["diagnostics"].as_array().unwrap().clone();
                    seen.push(diagnostics.clone());
                    if expected(&diagnostics) {
                        return diagnostics;
                    }
                }
            }
        }
    }
    panic!("diagnostico do clangd nao chegou: {seen:#?}; status: {status:#?}");
}

#[test]
fn llvm_clangd_serves_a_real_msvc_project_and_live_edits() {
    let root = project();
    let source = root.join("main.cpp");
    let detector = ToolDetector::from_environment();
    let clangd = detector
        .find_in_path("clangd")
        .expect("LLVM clangd instalado");
    assert!(clangd.is_file());
    for binary in ["clang-format", "clang-tidy"] {
        assert!(
            detector.find_in_path(binary).is_some(),
            "{binary} do LLVM instalado"
        );
    }
    let (sender, events) = mpsc::channel();
    let mut core = Core::with_detector(detector);
    core.enable_lsp(sender);
    rpc(&mut core, "workspace.open", json!({ "path": root }));

    let clean = diagnostics_until(&mut core, &events, &source, true, <[Value]>::is_empty);
    assert!(clean.is_empty());

    let definition = rpc(
        &mut core,
        "lsp.definition",
        json!({ "path": source, "content": SOURCE, "line": 5, "column": 30 }),
    );
    assert!(
        definition["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("app.h")),
        "{definition}"
    );
    let hover = rpc(
        &mut core,
        "lsp.hover",
        json!({ "path": source, "content": SOURCE, "line": 5, "column": 30 }),
    );
    assert!(
        hover["content"]
            .as_str()
            .is_some_and(|s| s.contains("answer")),
        "{hover}"
    );

    let incomplete = SOURCE.replace("values.front()", "values.fr");
    let completion = rpc(
        &mut core,
        "lsp.completion",
        json!({ "path": source, "content": incomplete, "line": 6, "column": 21 }),
    );
    assert!(
        completion["items"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["insertText"] == "front")),
        "{completion}"
    );

    let broken = SOURCE.replace("answer()", "missing_name");
    rpc(
        &mut core,
        "lsp.didChange",
        json!({ "path": source, "content": broken }),
    );
    diagnostics_until(&mut core, &events, &source, false, |d| {
        d.iter().any(|item| {
            item["severity"] == "error"
                && item["message"]
                    .as_str()
                    .is_some_and(|s| s.contains("missing_name"))
        })
    });
    rpc(
        &mut core,
        "lsp.didChange",
        json!({ "path": source, "content": SOURCE }),
    );
    diagnostics_until(&mut core, &events, &source, false, <[Value]>::is_empty);
}
