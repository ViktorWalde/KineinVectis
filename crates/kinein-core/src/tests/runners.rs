//! `test.run` / `quality.run` dispatch — now async cancelable jobs.
//!
//! Both mirror `build.run` (see `tests/build.rs`); here we cover their
//! pre-flight validation and one real async test run.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn quality_run_rejects_non_rust_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-quality-unsupported", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();
    let mut core = core_with_empty_search_path("quality-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(71_i64, "quality.run", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("cmake"));
}

#[test]
fn test_run_rejects_unsupported_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-test-unsupported", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();
    let mut core = core_with_empty_search_path("test-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(73_i64, "test.run", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("python"));
}

#[test]
fn test_run_starts_a_job_and_reports_a_passing_case() {
    use std::time::Duration;

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("test-job");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-test-job", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "#[test]\nfn soma() {\n    assert_eq!(1 + 1, 2);\n}\n",
    )
    .unwrap();

    let opened = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let started = core.handle_request(&JsonRpcRequest::new(75_i64, "test.run", Some(json!({}))));
    let job_id = started.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .expect("test.run deve retornar jobId")
        .to_owned();

    let mut saw_created = false;
    let mut saw_job_output = false;
    let mut saw_passing_case = false;
    let mut saw_test_finished = false;
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(90))
            .expect("eventos de teste dentro do timeout");
        match event.method.as_str() {
            "event.job.created" => {
                saw_created = event.params.as_ref().unwrap()["id"] == job_id.as_str();
            }
            "event.job.output" => {
                let params = event.params.as_ref().unwrap();
                if params["jobId"] == job_id.as_str() && params["line"].is_string() {
                    saw_job_output = true;
                }
            }
            "event.test.case" => {
                let params = event.params.as_ref().unwrap();
                if params["name"] == "soma" && params["status"] == "passed" {
                    saw_passing_case = true;
                }
            }
            "event.test.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["success"], true);
                assert_eq!(params["passed"], 1);
                saw_test_finished = true;
            }
            "event.job.finished" => {
                assert_eq!(event.params.as_ref().unwrap()["status"], "success");
                break;
            }
            _ => {}
        }
    }
    assert!(saw_created, "faltou event.job.created");
    assert!(saw_job_output, "faltou event.job.output");
    assert!(saw_passing_case, "faltou event.test.case passed");
    assert!(saw_test_finished, "faltou event.test.finished");
}
