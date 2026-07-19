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

#[test]
fn integration_config_cicla_set_get_reset_com_eventos_via_dispatch() {
    // E2E da fatia 2.2: os tres metodos LIGADOS na cadeia + o contrato do
    // evento (so mudanca REAL emite). Os eventos de config sao emitidos no
    // MESMO fio do handle_request, entao apos a resposta eles ja estao no
    // canal — a leitura por try_iter e deterministica.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("integration-config");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-integration-config-ws", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let integration_events = |receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>| {
        receiver
            .try_iter()
            .filter(|event| event.method == "event.integration.changed")
            .map(|event| event.params.unwrap())
            .collect::<Vec<_>>()
    };
    // Limpa eventos de outras origens (workspace.open dispara lsp/fs).
    let _ = integration_events(&receiver);

    // SET efetivo: grava e emite exatamente um evento kind=config.
    let set = core.handle_request(&JsonRpcRequest::new(
        71_i64,
        "integration.config.set",
        Some(json!({ "id": "clangd", "key": "args",
                     "value": "--log=error", "scope": "workspace" })),
    ));
    assert!(set.response().error.is_none());
    let eventos = integration_events(&receiver);
    assert_eq!(eventos.len(), 1);
    assert_eq!(eventos[0]["id"], "clangd");
    assert_eq!(eventos[0]["kind"], "config");

    // SET idempotente: mesma chave/valor NAO emite de novo.
    let repetido = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "integration.config.set",
        Some(json!({ "id": "clangd", "key": "args",
                     "value": "--log=error", "scope": "workspace" })),
    ));
    assert!(repetido.response().error.is_none());
    assert!(integration_events(&receiver).is_empty());

    // GET devolve a entrada no escopo workspace.
    let lido = core.handle_request(&JsonRpcRequest::new(
        73_i64,
        "integration.config.get",
        Some(json!({ "id": "clangd" })),
    ));
    let entries = lido.response().result.as_ref().unwrap()["entries"].clone();
    assert_eq!(entries[0]["key"], "args");
    assert_eq!(entries[0]["scope"], "workspace");

    // RESET remove (removed=true) e emite; GET volta VAZIO — reversibilidade.
    let reset = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "integration.config.reset",
        Some(json!({ "id": "clangd", "key": "args", "scope": "workspace" })),
    ));
    assert_eq!(reset.response().result.as_ref().unwrap()["removed"], true);
    assert_eq!(integration_events(&receiver).len(), 1);
    let vazio = core.handle_request(&JsonRpcRequest::new(
        75_i64,
        "integration.config.get",
        Some(json!({ "id": "clangd" })),
    ));
    assert!(
        vazio.response().result.as_ref().unwrap()["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // RESET de novo: no-op declarado (removed=false) e SEM evento.
    let noop = core.handle_request(&JsonRpcRequest::new(
        76_i64,
        "integration.config.reset",
        Some(json!({ "id": "clangd", "key": "args", "scope": "workspace" })),
    ));
    assert_eq!(noop.response().result.as_ref().unwrap()["removed"], false);
    assert!(integration_events(&receiver).is_empty());
}

#[test]
fn integration_config_recusa_id_desconhecido_e_escopo_sem_workspace() {
    let mut core = core_with_empty_search_path("integration-config-erros");

    // Id fora do inventario compilado: erro de parametro, nao sucesso vazio.
    let desconhecido = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "integration.config.set",
        Some(json!({ "id": "naoexiste", "key": "k", "value": "v", "scope": "global" })),
    ));
    let erro = desconhecido.response().error.as_ref().unwrap();
    assert_eq!(erro.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);

    // Escopo workspace sem workspace aberto: erro explicito.
    let sem_workspace = core.handle_request(&JsonRpcRequest::new(
        81_i64,
        "integration.config.set",
        Some(json!({ "id": "clangd", "key": "k", "value": "v", "scope": "workspace" })),
    ));
    let erro = sem_workspace.response().error.as_ref().unwrap();
    assert!(erro.message.contains("workspace"));
}

#[test]
fn tools_detect_emite_health_changed_quando_a_deteccao_muda() {
    // E2E do evento de SAUDE: 1a deteccao nao emite (old=None, senao todo
    // boot viraria uma rajada); a ferramenta APARECER entre deteccoes emite.
    use std::os::unix::fs::PermissionsExt;

    let (sender, receiver) = std::sync::mpsc::channel();
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-health-changed", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(&dir));
    core.enable_lsp(sender);

    // O rg JA existe na primeira deteccao: se a guarda old=None falhar, a
    // primeira deteccao emite (saude "instalado" difere do vazio) e o assert
    // abaixo pega — foi uma mutacao equivalente com PATH vazio que exigiu
    // esta ordem (2026-07-19).
    let rg = dir.join("rg");
    std::fs::write(&rg, "#!/bin/sh\necho 'ripgrep 14.0.0'\n").unwrap();
    std::fs::set_permissions(&rg, std::fs::Permissions::from_mode(0o755)).unwrap();

    let primeira = core.handle_request(&JsonRpcRequest::new(90_i64, "tools.detect", None));
    assert!(primeira.response().error.is_none());
    let so_integration = |receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>| {
        receiver
            .try_iter()
            .filter(|event| event.method == "event.integration.changed")
            .map(|event| event.params.unwrap())
            .collect::<Vec<_>>()
    };
    assert!(so_integration(&receiver).is_empty());

    // O rg SOME entre deteccoes: a saude derivada muda e o evento chega.
    std::fs::remove_file(&rg).unwrap();

    let segunda = core.handle_request(&JsonRpcRequest::new(91_i64, "tools.detect", None));
    assert!(segunda.response().error.is_none());
    let eventos = so_integration(&receiver);
    assert!(
        eventos
            .iter()
            .any(|params| params["id"] == "ripgrep" && params["kind"] == "health"),
        "esperava health-changed do ripgrep, veio: {eventos:?}"
    );
}
