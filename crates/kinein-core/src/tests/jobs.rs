//! Job system dispatch (`job.*`).
//!
//! The manager's lifecycle/cancel behaviour is unit-tested in `jobs::manager`;
//! here we cover the request routing through `Core`.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn job_list_starts_empty() {
    let mut core = core_with_empty_search_path("job-list-empty");
    let outcome = core.handle_request(&JsonRpcRequest::new(50_i64, "job.list", Some(json!({}))));
    let result = outcome.response().result.as_ref().unwrap();

    assert!(result["jobs"].as_array().unwrap().is_empty());
}

#[test]
fn job_cancel_unknown_id_reports_not_cancelled() {
    let mut core = core_with_empty_search_path("job-cancel-unknown");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        51_i64,
        "job.cancel",
        Some(json!({ "jobId": "job_999" })),
    ));
    let result = outcome.response().result.as_ref().unwrap();

    assert_eq!(result["jobId"], "job_999");
    assert_eq!(result["cancelled"], false);
}

#[test]
fn job_cancel_without_job_id_returns_invalid_params() {
    let mut core = core_with_empty_search_path("job-cancel-bad");
    let outcome = core.handle_request(&JsonRpcRequest::new(52_i64, "job.cancel", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}
