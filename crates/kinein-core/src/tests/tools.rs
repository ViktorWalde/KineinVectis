//! Tool detection dispatch (`tools.detect`, `tools.status`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn tools_detect_returns_structured_status_for_all_known_tools() {
    let mut core = core_with_empty_search_path("detect");
    let request = JsonRpcRequest::new(4_i64, "tools.detect", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();
    let tools = result["tools"].as_array().unwrap();

    assert_eq!(tools.len(), crate::tools::KNOWN_TOOLS.len());
    assert!(tools.iter().all(|tool| tool["status"] == "missing"));
    assert!(tools[0].get("suggestedInstall").is_none());
}

#[test]
fn tools_status_runs_detection_once_and_reuses_registry() {
    let mut core = core_with_empty_search_path("status");
    let first = core.handle_request(&JsonRpcRequest::new(5_i64, "tools.status", Some(json!({}))));
    let second = core.handle_request(&JsonRpcRequest::new(6_i64, "tools.status", Some(json!({}))));

    assert_eq!(
        first.response().result.as_ref().unwrap()["tools"],
        second.response().result.as_ref().unwrap()["tools"]
    );
}

#[test]
fn integration_list_expoe_o_inventario_com_saude_via_dispatch() {
    // E2E: prova que `integration.list` esta LIGADO na cadeia de dispatch
    // (service_request_response) e devolve o inventario com saude. Com PATH
    // vazio, toda integracao aparece nao-instalada — a saude reusa tools.rs.
    let mut core = core_with_empty_search_path("integration-list");
    let request = JsonRpcRequest::new(7_i64, "integration.list", Some(json!({})));
    let outcome = core.handle_request(&request);
    let result = outcome.response().result.as_ref().unwrap();
    let integrations = result["integrations"].as_array().unwrap();

    assert_eq!(integrations.len(), crate::tools::KNOWN_TOOLS.len());
    for entry in integrations {
        assert_eq!(entry["health"]["installed"], false);
        assert!(
            !entry["descriptor"]["capabilities"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        // camelCase no fio, e descriptor.id casa com health.id.
        assert_eq!(entry["descriptor"]["id"], entry["health"]["id"]);
    }
}

#[test]
fn environment_scan_requires_jobs_enabled() {
    let mut core = core_with_empty_search_path("environment-no-jobs");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "environment.scan",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InternalError);
    assert_eq!(
        error.details.as_ref().unwrap()["method"],
        "environment.scan"
    );
}

#[test]
fn environment_scan_runs_as_job_and_updates_tools_status() {
    use std::time::Duration;

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("environment-job");
    core.enable_lsp(sender);

    let started = core.handle_request(&JsonRpcRequest::new(
        81_i64,
        "environment.scan",
        Some(json!({})),
    ));
    let job_id = started.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .expect("environment.scan deve retornar jobId")
        .to_owned();

    let mut saw_started = false;
    let mut saw_tool = false;
    let mut finished_tools = None;
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("eventos do environment.scan dentro do timeout");
        match event.method.as_str() {
            "event.environment.started" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["tools"], crate::tools::KNOWN_TOOLS.len());
                saw_started = true;
            }
            "event.environment.tool" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert!(params["tool"]["id"].is_string());
                saw_tool = true;
            }
            "event.environment.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["success"], true);
                assert_eq!(params["total"], crate::tools::KNOWN_TOOLS.len());
                assert_eq!(params["missing"], crate::tools::KNOWN_TOOLS.len());
                finished_tools = Some(params["tools"].clone());
            }
            "event.job.finished" => {
                assert_eq!(event.params.as_ref().unwrap()["status"], "success");
                break;
            }
            _ => {}
        }
    }

    assert!(saw_started, "faltou event.environment.started");
    assert!(saw_tool, "faltou event.environment.tool");
    let finished_tools = finished_tools.expect("faltou event.environment.finished");
    let status = core.handle_request(&JsonRpcRequest::new(
        82_i64,
        "tools.status",
        Some(json!({})),
    ));

    assert_eq!(
        status.response().result.as_ref().unwrap()["tools"],
        finished_tools
    );
}
