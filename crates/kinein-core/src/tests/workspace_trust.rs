//! 16.4b (`roadmaps/59` §7.1.4): a confianca no projeto mora FORA dele. Um
//! `trusted-projects.json` que o repositorio traga nao conta; o
//! `workspace.trust` grava no estado global e o reabrir lembra; outro
//! projeto continua sem confianca; o projeto que a IDE cria nasce confiavel.

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::core_with_empty_search_path;

fn call(core: &mut crate::Core, method: &str, params: Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)));
    let response = outcome.response();
    assert!(response.error.is_none(), "{method}: {:?}", response.error);
    response.result.clone().unwrap()
}

#[test]
fn trust_is_kept_outside_the_project_and_remembered() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-trust", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let project = base.join("clonado");
    let other = base.join("outro");
    let global = base.join("global");
    for dir in [&project, &other, &global] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(project.join("CMakeLists.txt"), "project(x C)\n").unwrap();
    let project = crate::platform::canonicalize(&project).unwrap();
    // O repositorio tenta se declarar confiavel: nao conta.
    std::fs::create_dir_all(project.join(".kinein")).unwrap();
    std::fs::write(
        project.join(".kinein/trusted-projects.json"),
        json!({ "schemaVersion": 1, "roots": [project.display().to_string()] }).to_string(),
    )
    .unwrap();

    let mut core = core_with_empty_search_path("workspace-trust");
    core.enable_persistence(global.clone());
    let path = json!({ "path": project.to_str().unwrap() });
    assert_eq!(
        call(&mut core, "workspace.open", path.clone())["trusted"],
        false
    );
    assert_eq!(
        call(&mut core, "workspace.trust", json!({}))["trusted"],
        true
    );
    assert_eq!(
        call(&mut core, "workspace.status", json!({}))["workspace"]["trusted"],
        true
    );
    assert!(crate::workspace::trust_path_in(&global).is_file());
    // O arquivo do projeto nunca guarda a confianca.
    let metadata = std::fs::read_to_string(project.join(".kinein/workspace.json")).unwrap();
    assert!(!metadata.contains("\"trusted\": true"), "{metadata}");

    // Outro core (a IDE reaberta) lembra; outro projeto, nao.
    let mut reopened = core_with_empty_search_path("workspace-trust-2");
    reopened.enable_persistence(global);
    assert_eq!(call(&mut reopened, "workspace.open", path)["trusted"], true);
    let other_path = json!({ "path": other.to_str().unwrap() });
    assert_eq!(
        call(&mut reopened, "workspace.open", other_path)["trusted"],
        false
    );

    // O projeto que a IDE cria nasce confiavel.
    let created = call(
        &mut reopened,
        "workspace.createProject",
        json!({ "parent": base.to_str().unwrap(), "name": "novo", "template": "rustCargo" }),
    );
    assert_eq!(created["trusted"], true);
    let _ = std::fs::remove_dir_all(&base);
}

/// Sem workspace, `workspace.trust` e' recusado.
#[test]
fn trust_needs_an_open_workspace() {
    let mut core = core_with_empty_search_path("workspace-trust-none");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.trust",
        Some(json!({})),
    ));
    assert!(outcome.response().error.is_some());
}
