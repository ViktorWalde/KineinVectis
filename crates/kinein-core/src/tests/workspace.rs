//! Workspace lifecycle dispatch (`workspace.*`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

#[test]
fn workspace_open_status_close_cycle_works() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-cycle", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-cycle");

    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let opened_result = opened.response().result.as_ref().unwrap();
    assert_eq!(opened_result["kind"], "rustCargo");

    let status = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let status_result = status.response().result.as_ref().unwrap();
    assert_eq!(status_result["workspace"]["kind"], "rustCargo");

    let closed = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "workspace.close",
        Some(json!({})),
    ));
    let closed_result = closed.response().result.as_ref().unwrap();
    assert_eq!(closed_result["status"], "ok");
    assert!(closed_result["closed"].is_string());

    let after = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "workspace.status",
        Some(json!({})),
    ));
    let after_result = after.response().result.as_ref().unwrap();
    assert!(after_result["workspace"].is_null());
}

#[test]
fn workspace_open_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "workspace.open",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

#[test]
fn recent_workspaces_list_is_empty_when_global_storage_is_disabled() {
    let mut core = core_with_empty_search_path("workspace-recent-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        140_i64,
        "workspace.recent.list",
        Some(json!({})),
    ));
    let result = outcome.response().result.as_ref().unwrap();

    assert_eq!(result["workspaces"].as_array().unwrap().len(), 0);
}

#[test]
fn recent_workspace_mutations_validate_typed_params_before_storage() {
    let mut core = core_with_empty_search_path("workspace-recent-params");
    let pin = core.handle_request(&JsonRpcRequest::new(
        141_i64,
        "workspace.recent.pin",
        Some(json!({ "root": "/tmp/demo" })),
    ));
    assert_eq!(
        pin.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let remove = core.handle_request(&JsonRpcRequest::new(
        142_i64,
        "workspace.recent.remove",
        Some(json!({ "root": "/tmp/demo", "extra": true })),
    ));
    assert_eq!(
        remove.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );

    let clear = core.handle_request(&JsonRpcRequest::new(
        143_i64,
        "workspace.recent.clear",
        Some(json!({ "extra": true })),
    ));
    assert_eq!(
        clear.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn workspace_browse_returns_directory_entries() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-browse", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let mut core = core_with_empty_search_path("workspace-browse");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        16_i64,
        "workspace.browse",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let entries = result["entries"].as_array().unwrap();

    assert_eq!(
        result["path"],
        dir.canonicalize().unwrap().display().to_string()
    );
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "src");
}

#[test]
fn workspace_browse_without_path_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-browse-bad-params");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        17_i64,
        "workspace.browse",
        Some(json!({})),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
}

#[test]
fn workspace_create_folder_creates_directory() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-create-folder", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-folder");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        18_i64,
        "workspace.createFolder",
        Some(json!({ "parent": dir.to_str().unwrap(), "name": "novo modulo" })),
    ));
    let result = outcome.response().result.as_ref().unwrap();
    let created = std::path::PathBuf::from(result["path"].as_str().unwrap());

    assert!(created.is_dir());
    assert_eq!(created.file_name().unwrap(), "novo modulo");
}

#[test]
fn workspace_create_project_opens_created_cpp_project() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-workspace-create-project", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("workspace-create-project");

    let outcome = core.handle_request(&JsonRpcRequest::new(
        19_i64,
        "workspace.createProject",
        Some(json!({
            "parent": dir.to_str().unwrap(),
            "name": "demo_cpp",
            "template": "cppCmake",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap();

    assert_eq!(result["kind"], "cmake");
    assert_eq!(result["name"], "demo_cpp");
    assert!(dir.join("demo_cpp/CMakeLists.txt").is_file());
    assert!(dir.join("demo_cpp/src/main.cpp").is_file());
    assert_eq!(
        core.handle_request(&JsonRpcRequest::new(
            20_i64,
            "workspace.status",
            Some(json!({}))
        ))
        .response()
        .result
        .as_ref()
        .unwrap()["workspace"]["name"],
        "demo_cpp"
    );
}

#[test]
fn workspace_open_with_missing_directory_returns_invalid_params() {
    let mut core = core_with_empty_search_path("workspace-missing-dir");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        15_i64,
        "workspace.open",
        Some(json!({ "path": "/definitely/not/a/real/path" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();

    assert_eq!(error.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert_eq!(
        error.details.as_ref().unwrap()["path"],
        "/definitely/not/a/real/path"
    );
}

#[test]
fn workspace_session_roundtrips_through_open() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-session-dispatch", std::process::id()));
    // O PID pode ser reutilizado entre execuções do gate (e é estável em
    // alguns sandboxes). Sem limpar, `.kinein/session.json` da rodada anterior
    // transforma esta "primeira abertura" em restore e torna o teste flaky.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn x() {}\n").unwrap();
    let mut core = core_with_empty_search_path("session-dispatch");

    let no_workspace = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.saveSession",
        Some(json!({ "openFiles": [] })),
    ));
    assert_eq!(
        no_workspace.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );

    let opened = core.handle_request(&JsonRpcRequest::new(
        71_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let first_open = opened.response().result.as_ref().unwrap().clone();
    assert!(
        first_open.get("session").is_none(),
        "primeira abertura nao deveria ter sessao"
    );
    let root = first_open["root"].as_str().unwrap().to_owned();

    let saved = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "workspace.saveSession",
        Some(json!({
            "openFiles": [format!("{root}/src/main.rs"), format!("{root}/src/lib.rs")],
            "activeFile": format!("{root}/src/lib.rs"),
        })),
    ));
    assert_eq!(saved.response().result.as_ref().unwrap()["files"], 2);

    let closed = core.handle_request(&JsonRpcRequest::new(73_i64, "workspace.close", None));
    assert!(closed.response().error.is_none());

    let reopened = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    let session = reopened.response().result.as_ref().unwrap()["session"].clone();
    let files = session["openFiles"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert!(files[0].as_str().unwrap().ends_with("src/main.rs"));
    assert!(
        session["activeFile"]
            .as_str()
            .unwrap()
            .ends_with("src/lib.rs")
    );
}

/// A suite NAO pode escrever no estado global real do usuario.
///
/// Achado em 2026-08-29: `enable_lsp` ligava a persistencia por tabela, entao
/// todo teste que abria um workspace gravava em
/// `~/.config/kinein-vectis/recent-workspaces.json`. Como o arquivo tem teto de
/// 12 entradas, uma unica execucao de `cargo test` DESPEJAVA a lista de
/// projetos recentes reais — 12 de 12 entradas eram `/tmp/kinein-core-tests`.
/// Falha silenciosa exemplar: nada reclamava, e o gate ficava verde.
#[test]
fn test_core_never_writes_to_the_real_global_storage() {
    let global = crate::settings::global_dir();
    let before = global_snapshot(&global);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-global-isolation", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("global-isolation");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        160_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    assert_eq!(
        global_snapshot(&global),
        before,
        "um core de teste alterou {}",
        global.display()
    );
}

/// Nome e hash do conteudo de cada arquivo do diretorio global.
///
/// Hash, e nao o conteudo: a mensagem de falha tem que caber na tela — despejar
/// o JSON inteiro em bytes esconde exatamente o que mudou.
fn global_snapshot(dir: &std::path::Path) -> std::collections::BTreeMap<String, u64> {
    use std::hash::{Hash as _, Hasher as _};

    let mut snapshot = std::collections::BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return snapshot;
    };
    for entry in entries.flatten() {
        if let Ok(bytes) = std::fs::read(entry.path()) {
            let mut hasher = std::hash::DefaultHasher::new();
            bytes.hash(&mut hasher);
            snapshot.insert(
                entry.file_name().to_string_lossy().into_owned(),
                hasher.finish(),
            );
        }
    }
    snapshot
}

/// A store de rascunhos tem de acompanhar a TROCA de workspace.
///
/// `workspace.createProject` mudava `self.workspace` sem trocar `self.drafts`:
/// o projeto recem-criado escrevia autosave no banco do projeto ANTERIOR (ou
/// respondia "persistencia local de rascunhos indisponivel", quando nao havia
/// anterior). A rede de seguranca de dados (docs/seguranca/23) desligava em
/// silencio. Sem `activate_workspace`, este teste reprova.
#[test]
fn creating_a_project_moves_the_draft_store_to_the_new_workspace() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-draft-store-move", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let anterior = base.join("anterior");
    std::fs::create_dir_all(&anterior).unwrap();
    std::fs::write(anterior.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(anterior.join("velho.txt"), "no disco\n").unwrap();
    let global = base.join("global");
    std::fs::create_dir_all(&global).unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("draft-store-move");
    core.enable_lsp(sender);
    core.enable_persistence(global);

    let opened = core.handle_request(&JsonRpcRequest::new(
        170_i64,
        "workspace.open",
        Some(json!({ "path": anterior.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let salvo = core.handle_request(&JsonRpcRequest::new(
        171_i64,
        "draft.save",
        Some(json!({
            "path": anterior.join("velho.txt").to_str().unwrap(),
            "content": "editado, nao salvo\n",
        })),
    ));
    assert!(salvo.response().error.is_none());

    // Cria um projeto novo: o workspace ativo passa a ser outro.
    let criado = core.handle_request(&JsonRpcRequest::new(
        172_i64,
        "workspace.createProject",
        Some(json!({
            "parent": base.to_str().unwrap(),
            "name": "novo",
            "template": "rustCargo",
        })),
    ));
    assert!(
        criado.response().error.is_none(),
        "createProject falhou: {:?}",
        criado.response().error
    );
    let novo = std::path::PathBuf::from(
        criado.response().result.as_ref().unwrap()["root"]
            .as_str()
            .unwrap(),
    );

    let novo_arquivo = novo.join("src").join("main.rs");
    let salvo_no_novo = core.handle_request(&JsonRpcRequest::new(
        173_i64,
        "draft.save",
        Some(json!({
            "path": novo_arquivo.to_str().unwrap(),
            "content": "fn main() { /* rascunho */ }\n",
        })),
    ));
    assert!(
        salvo_no_novo.response().error.is_none(),
        "autosave do projeto recem-criado indisponivel: {:?}",
        salvo_no_novo.response().error
    );

    // O rascunho foi para o banco do projeto NOVO...
    let no_novo = crate::db::DraftStore::open(&novo).expect("store do projeto novo");
    let paths_novo: Vec<String> = no_novo
        .list()
        .unwrap()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert_eq!(paths_novo.len(), 1);
    assert!(paths_novo[0].ends_with("main.rs"));

    // ...e o banco do projeto ANTERIOR nao foi contaminado.
    let no_anterior = crate::db::DraftStore::open(&anterior).expect("store do projeto anterior");
    let paths_anterior: Vec<String> = no_anterior
        .list()
        .unwrap()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert_eq!(paths_anterior.len(), 1);
    assert!(
        paths_anterior[0].ends_with("velho.txt"),
        "o banco do projeto anterior recebeu {paths_anterior:?}"
    );
}

/// Arquivo apagado nao volta como "rascunho recuperado".
///
/// Um rascunho so nasce contra arquivo EXISTENTE (`fsops::confine_file`), entao
/// um rascunho cujo arquivo sumiu significa que o usuario apagou (ou renomeou)
/// o arquivo depois do autosave. `recover_drafts` comparava o conteudo com o
/// disco e, como ler um arquivo ausente devolve `None` (que "difere" do
/// rascunho), oferecia de volta o buffer de um arquivo deliberadamente apagado.
#[test]
fn drafts_of_deleted_files_are_not_recovered() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-draft-deleted", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let projeto = base.join("projeto");
    std::fs::create_dir_all(&projeto).unwrap();
    std::fs::write(projeto.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(projeto.join("some.txt"), "no disco\n").unwrap();
    std::fs::write(projeto.join("fica.txt"), "no disco\n").unwrap();
    let global = base.join("global");
    std::fs::create_dir_all(&global).unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("draft-deleted");
    core.enable_lsp(sender);
    core.enable_persistence(global);
    let aberto = core.handle_request(&JsonRpcRequest::new(
        180_i64,
        "workspace.open",
        Some(json!({ "path": projeto.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    for arquivo in ["some.txt", "fica.txt"] {
        let salvo = core.handle_request(&JsonRpcRequest::new(
            181_i64,
            "draft.save",
            Some(json!({
                "path": projeto.join(arquivo).to_str().unwrap(),
                "content": "editado, nao salvo\n",
            })),
        ));
        assert!(salvo.response().error.is_none());
    }

    let apagado = core.handle_request(&JsonRpcRequest::new(
        182_i64,
        "fs.delete",
        Some(json!({ "path": projeto.join("some.txt").to_str().unwrap() })),
    ));
    assert!(apagado.response().error.is_none());

    // Reabrir: so o arquivo que continua no disco volta como rascunho.
    let fechado = core.handle_request(&JsonRpcRequest::new(183_i64, "workspace.close", None));
    assert!(fechado.response().error.is_none());
    let reaberto = core.handle_request(&JsonRpcRequest::new(
        184_i64,
        "workspace.open",
        Some(json!({ "path": projeto.to_str().unwrap() })),
    ));
    let drafts = reaberto.response().result.as_ref().unwrap()["drafts"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let paths: Vec<&str> = drafts
        .iter()
        .filter_map(|item| item["path"].as_str())
        .collect();
    assert_eq!(paths.len(), 1, "recuperou {paths:?}");
    assert!(paths[0].ends_with("fica.txt"));
}
