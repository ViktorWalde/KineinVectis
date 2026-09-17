//! Build runner dispatch (`build.run`), including its streamed events.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn build_run_requires_open_workspace() {
    let mut core = core_with_empty_search_path("build-no-workspace");
    let request = JsonRpcRequest::new(30_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn build_run_rejects_unsupported_project_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-unsupported", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut core = core_with_empty_search_path("build-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let request = JsonRpcRequest::new(32_i64, "build.run", Some(json!({})));
    let outcome = core.handle_request(&request);
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("python"));
}

#[test]
fn build_run_requires_jobs_enabled() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-no-jobs", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("build-no-jobs");

    let opened = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Supported kind + open workspace, but the job manager is not enabled.
    let outcome = core.handle_request(&JsonRpcRequest::new(34_i64, "build.run", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
}

#[test]
fn hybrid_workspace_routes_cargo_and_cmake_without_changing_primary_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-hybrid", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(hybrid CXX)\n",
    )
    .unwrap();
    let mut core = core_with_empty_search_path("build-hybrid");

    let opened = core.handle_request(&JsonRpcRequest::new(
        341_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let workspace = opened.response().result.as_ref().unwrap();
    assert_eq!(workspace["kind"], "rustCargo");
    assert_eq!(
        workspace["capabilities"]["buildSystems"],
        json!(["cargo", "cmake"])
    );

    let cmake_status = core.handle_request(&JsonRpcRequest::new(
        342_i64,
        "cmake.status",
        Some(json!({})),
    ));
    assert!(cmake_status.response().error.is_none());

    for (request_id, build_system) in [(343_i64, "cargo"), (344_i64, "cmake")] {
        let outcome = core.handle_request(&JsonRpcRequest::new(
            request_id,
            "build.run",
            Some(json!({ "buildSystem": build_system })),
        ));
        let error = outcome.response().error.as_ref().unwrap();
        assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
        assert!(error.message.contains("jobs"));
    }

    let unavailable = core.handle_request(&JsonRpcRequest::new(
        345_i64,
        "build.run",
        Some(json!({ "buildSystem": "maven" })),
    ));
    let error = unavailable.response().error.as_ref().unwrap();
    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("Maven"));
}

#[test]
fn build_run_starts_a_job_and_finishes_successfully() {
    use std::time::Duration;

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("build-job");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-job", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();

    let opened = core.handle_request(&JsonRpcRequest::new(
        35_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let started = core.handle_request(&JsonRpcRequest::new(36_i64, "build.run", Some(json!({}))));
    let job_id = started.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .expect("build.run deve retornar jobId")
        .to_owned();
    assert!(!job_id.is_empty());

    let mut saw_created = false;
    let mut saw_build_finished = false;
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(60))
            .expect("eventos do build dentro do timeout");
        match event.method.as_str() {
            // O workspace.open sobe o job do INDICE junto: os eventos de job
            // sao filtrados pelo id, como no tests/runners.rs — sem isso o
            // job.finished do indice, que chega quando quer, e' tomado pelo
            // do build (falhou ao acaso em 2026-09-12).
            "event.job.created" => {
                saw_created |= event.params.as_ref().unwrap()["id"] == job_id.as_str();
            }
            "event.build.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["success"], true);
                saw_build_finished = true;
            }
            "event.job.finished" => {
                let params = event.params.as_ref().unwrap();
                if params["jobId"] != job_id.as_str() {
                    continue;
                }
                assert_eq!(params["status"], "success");
                break;
            }
            _ => {}
        }
    }
    assert!(saw_created, "faltou event.job.created");
    assert!(saw_build_finished, "faltou event.build.finished");
}

/// Roda um `build.run` e devolve as linhas do job (comando e saida) e o
/// sucesso — filtrando pelo jobId, porque o indice tambem emite jobs.
fn roda_build(
    core: &mut crate::Core,
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
) -> (Vec<String>, bool) {
    use std::time::Duration;
    let started = core.handle_request(&JsonRpcRequest::new(42_i64, "build.run", Some(json!({}))));
    assert!(
        started.response().error.is_none(),
        "{:?}",
        started.response().error
    );
    let job_id = started.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut linhas = Vec::new();
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("eventos do build");
        let params = event.params.clone().unwrap_or_default();
        if params["jobId"] != job_id.as_str() {
            continue;
        }
        match event.method.as_str() {
            "event.build.output" => linhas.push(params["line"].as_str().unwrap().to_owned()),
            "event.build.started" => {
                linhas.push(format!("$ {}", params["command"].as_str().unwrap()));
            }
            "event.build.finished" => return (linhas, params["success"] == true),
            _ => {}
        }
    }
}

/// Makefile puro (P0 do 40 §4.1, 2026-09-17): o workspace e' `Make`, o
/// `build.run` roda `bear -- make` quando o bear existe (o bear falso ecoa
/// os argv e grava a CDB, como o real) e `make` a seco sem ele — com a linha
/// que diz o passo. Um Makefile ao lado de um `CMakeLists` continua `CMake`.
#[test]
#[cfg(unix)]
fn a_plain_makefile_builds_with_bear_when_it_exists_and_says_so_when_not() {
    use std::os::unix::fs::PermissionsExt;

    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-build-make", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = base.join("projeto");
    let bin = base.join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(dir.join("Makefile"), "all:\n\t@echo compilando\n").unwrap();
    let executavel = |nome: &str, corpo: &str| {
        let p = bin.join(nome);
        std::fs::write(&p, corpo).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    executavel("make", "#!/bin/sh\necho \"make-falso $*\"\n");

    // Sem bear: make a seco e a linha que diz o passo.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(&bin));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let info = opened.response().result.clone().unwrap();
    assert_eq!(info["kind"], "make", "{info}");
    assert!(
        info["capabilities"]["buildSystems"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "make"),
        "{info}"
    );
    let (linhas, sucesso) = roda_build(&mut core, &receiver);
    assert!(sucesso, "{linhas:?}");
    assert!(linhas.iter().any(|l| l == "$ make"), "{linhas:?}");
    assert!(
        linhas.iter().any(|l| l.contains("sem `bear`")),
        "{linhas:?}"
    );
    assert!(linhas.iter().any(|l| l == "make-falso "), "{linhas:?}");

    // Com bear: `bear -- <make>`, e a CDB que o bear (falso, como o real)
    // grava na raiz e' vista pelo cmake.status/cdb como utilizavel.
    executavel(
        "bear",
        "#!/bin/sh\necho \"bear-falso $*\"\nprintf '[]' > compile_commands.json\nshift; exec \"$@\"\n",
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(&bin));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let (linhas, sucesso) = roda_build(&mut core, &receiver);
    assert!(sucesso, "{linhas:?}");
    assert!(linhas.iter().any(|l| l == "$ bear -- make"), "{linhas:?}");
    assert!(
        linhas
            .iter()
            .any(|l| l == &format!("bear-falso -- {}", bin.join("make").display())),
        "{linhas:?}"
    );
    assert!(!linhas.iter().any(|l| l.contains("sem `bear`")));
    assert!(dir.join("compile_commands.json").is_file());
    let cdb = crate::cdb::status(&dir);
    assert_eq!(cdb.directory.as_deref(), Some("."));

    // Um Makefile AO LADO de um CMakeLists: continua CMake (precedencia).
    std::fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        43_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let info = opened.response().result.clone().unwrap();
    assert_eq!(info["kind"], "cmake");
    let sistemas = info["capabilities"]["buildSystems"].as_array().unwrap();
    assert!(sistemas.iter().any(|b| b == "cmake") && sistemas.iter().any(|b| b == "make"));
}
