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
            "event.job.created" => {
                saw_created = event.params.as_ref().unwrap()["id"] == job_id.as_str();
            }
            "event.build.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["success"], true);
                saw_build_finished = true;
            }
            "event.job.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["status"], "success");
                break;
            }
            _ => {}
        }
    }
    assert!(saw_created, "faltou event.job.created");
    assert!(saw_build_finished, "faltou event.build.finished");
}
