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

#[test]
fn quality_run_aceita_workspace_cmake_e_o_funil_termina() {
    use std::time::Duration;

    // L2 fatia 1: quality.run deixou de ser so-Cargo — workspace CMake ganha
    // Cppcheck no MESMO funil. Este teste prova a FIACAO (gate sincrono +
    // spawn + evento terminal) sem depender do binario: com o Cppcheck
    // ausente o job nasce e termina com event.quality.finished de falha;
    // com ele presente, termina com a analise real. Os argumentos do
    // comando (perfil + extras da config 2.2) sao provados por
    // cppcheck_args_respeita_perfil_e_aplica_extras_da_config_por_ultimo.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("quality-cmake");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-quality-cmake", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo LANGUAGES CXX)\n",
    )
    .unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        60_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let accepted = core.handle_request(&JsonRpcRequest::new(61_i64, "quality.run", None));
    let result = accepted
        .response()
        .result
        .as_ref()
        .expect("quality.run em workspace CMake tem que ACEITAR o job (gate do L2)");
    assert!(result["jobId"].is_string());

    let mut terminou = false;
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(15)) {
        if event.method == "event.quality.finished" {
            terminou = true;
            break;
        }
    }
    assert!(
        terminou,
        "o funil do quality tem que emitir o evento terminal"
    );
}

#[test]
fn coverage_run_aceita_cmake_e_o_evento_terminal_chega() {
    use std::time::Duration;

    // L2 fatia 3: coverage.run e job com evento terminal proprio. Sem lcov
    // (ou sem build instrumentado) o evento chega com success=false e um
    // erro ACIONAVEL — a fiacao e o que se prova, nao a ferramenta.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("coverage-cmake");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-coverage-cmake", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo LANGUAGES CXX)\n",
    )
    .unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        65_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let accepted = core.handle_request(&JsonRpcRequest::new(66_i64, "coverage.run", None));
    let result = accepted
        .response()
        .result
        .as_ref()
        .expect("coverage.run em workspace CMake tem que aceitar o job");
    assert!(result["jobId"].is_string());

    let mut terminal = None;
    while let Ok(event) = receiver.recv_timeout(Duration::from_secs(15)) {
        if event.method == "event.coverage.finished" {
            terminal = event.params;
            break;
        }
    }
    let params = terminal.expect("event.coverage.finished tem que chegar");
    // Nesta maquina de teste nao ha dados instrumentados: falha EXPLICADA.
    if params["success"] == false {
        assert!(params["error"].as_str().unwrap_or("").len() > 3);
    } else {
        assert!(params["linesTotal"].is_u64());
    }
}

#[test]
fn audit_run_recusa_projeto_que_nao_e_cargo_e_aceita_o_que_e() {
    use std::time::Duration;

    // L2 fatia 5: auditoria le o Cargo.lock, entao CMake responde erro
    // SINCRONO — nao um job que nasce para falhar. Em Cargo, o job e aceito
    // e o evento terminal chega mesmo sem cargo-audit instalado: o que se
    // prova aqui e a FIACAO, nao a ferramenta.
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("audit");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-audit", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Primeiro como CMake: tem que recusar na hora.
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20)\nproject(demo LANGUAGES CXX)\n",
    )
    .unwrap();
    let aberto = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let recusado = core.handle_request(&JsonRpcRequest::new(71_i64, "audit.run", None));
    assert!(
        recusado.response().error.is_some(),
        "auditoria em projeto sem Cargo.lock tem que falhar SINCRONA"
    );

    // Agora como Cargo: aceita o job e o evento terminal chega.
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let _ = std::fs::remove_file(dir.join("CMakeLists.txt"));
    let aberto = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let aceito = core.handle_request(&JsonRpcRequest::new(73_i64, "audit.run", None));
    let resultado = aceito
        .response()
        .result
        .as_ref()
        .expect("audit.run em workspace Cargo tem que aceitar o job");
    assert!(resultado["jobId"].is_string());

    let mut terminal = None;
    let mut comando = None;
    while let Ok(evento) = receiver.recv_timeout(Duration::from_secs(15)) {
        if evento.method == "event.audit.started" {
            comando = evento.params;
        } else if evento.method == "event.audit.finished" {
            terminal = evento.params;
            break;
        }
    }
    // Sem opt-in de rede, o comando anunciado DIZ que nao vai buscar. Isso
    // aparece no log do job: o usuario ve a decisao, nao so a consequencia.
    let comando = comando.expect("event.audit.started tem que chegar");
    assert!(
        comando["command"]
            .as_str()
            .unwrap_or_default()
            .contains("--no-fetch"),
        "sem autorizacao, o comando anunciado tem que declarar --no-fetch"
    );

    let params = terminal.expect("event.audit.finished tem que chegar");
    if params["success"] == false {
        assert!(params["error"].as_str().unwrap_or("").len() > 3);
    } else {
        assert!(params["vulnerabilities"].is_u64());
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn audit_run_rejeita_parametro_inventado() {
    // deny_unknown_fields no contrato: parametro que a UI invente nao pode
    // ser aceito em silencio e nunca fazer nada (licao do
    // assistant_terminal_width).
    let mut core = core_with_empty_search_path("audit-params");
    let resposta = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "audit.run",
        Some(json!({ "buildSystem": "cargo" })),
    ));
    assert!(resposta.response().error.is_some());
}
