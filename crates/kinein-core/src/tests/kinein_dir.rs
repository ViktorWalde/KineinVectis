//! INF-13 (`0.221.0`): o projeto com o `.kinein` ligado a uma pasta de fora
//! abre com o `stateProblem`, e nenhum dono que grava no `.kinein` cria ou
//! muda algo fora: o `workspace.json` e o banco ao abrir, a sessão, a
//! toolchain, os alvos remotos, o Grafana, a configuração ativa, a pasta de
//! build do configure e o LCOV da cobertura. Os links são de pasta ou para o
//! que não existe, com junção no Windows (60 §3.3).

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

use serde_json::json;

use super::core_with_empty_search_path;
use crate::kinein_dir::link;
use kinein_protocol::{JsonRpcRequest, ProjectKind};

const FOLDER_LINK: &str = ".kinein é um link simbólico, que a IDE não segue no projeto";

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-state-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

/// Um projeto C com o `.kinein` ligado a `outside`, vazia.
fn linked_project(name: &str) -> (PathBuf, PathBuf) {
    let dir = base(name);
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(
        root.join("src").join("main.c"),
        "int main(void) { return 0; }\n",
    )
    .unwrap();
    link(&outside, &root.join(".kinein"));
    (root, outside)
}

fn listing(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Pelo IPC: o `workspace.open` abre e diz por que, sem gravar o
/// `workspace.json` nem o banco fora; o `workspace.saveSession` falha com o
/// mesmo motivo. Trocado o link por uma pasta, o próximo abre sem o campo e
/// grava no projeto.
#[test]
fn the_project_with_a_linked_state_folder_opens_and_writes_nothing_outside() {
    let (root, outside) = linked_project("ipc");
    let mut core = core_with_empty_search_path("state-ipc");
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    let result = opened.response().result.as_ref().unwrap().clone();
    assert_eq!(result["stateProblem"], FOLDER_LINK);
    assert_eq!(result["root"], root.to_str().unwrap());

    let main = root.join("src").join("main.c");
    let saved = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.saveSession",
        Some(json!({ "openFiles": [main.to_str().unwrap()] })),
    ));
    let error = saved.response().error.as_ref().unwrap();
    assert!(error.message.contains(FOLDER_LINK), "{}", error.message);
    assert_eq!(listing(&outside), Vec::<String>::new());

    std::fs::remove_file(root.join(".kinein"))
        .or_else(|_| std::fs::remove_dir(root.join(".kinein")))
        .unwrap();
    let reopened = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    let result = reopened.response().result.as_ref().unwrap().clone();
    assert!(result.get("stateProblem").is_none(), "{result}");
    assert!(root.join(".kinein").join("workspace.json").is_file());
    assert_eq!(listing(&outside), Vec::<String>::new());
}

/// Cada dono, chamado direto, recusa com o motivo do guarda, e a pasta de
/// fora fica vazia.
#[test]
fn every_owner_refuses_the_linked_state_folder() {
    let (root, outside) = linked_project("donos");

    assert!(crate::db::DraftStore::open(&root).is_none());
    assert_eq!(
        crate::kits::store::save(&root, &BTreeMap::new()),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(
        crate::remote::remove(&root, "x"),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(crate::grafana::forget(&root), Err(FOLDER_LINK.to_owned()));
    assert_eq!(
        crate::cmake::select_configuration(&root, Some("debug"))
            .unwrap_err()
            .to_string(),
        FOLDER_LINK
    );
    let build = root.join(".kinein").join("build").join("debug");
    assert_eq!(
        crate::cmake::begin_configure(&root, &build, Some("debug"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        FOLDER_LINK
    );
    let coverage = crate::coverage::run(
        &root,
        ProjectKind::RustCargo,
        &crate::coverage::CoverageTools::default(),
        &Arc::new(AtomicBool::new(false)),
        &mut |_| {},
    );
    assert!(
        matches!(&coverage, Err(crate::coverage::CoverageError::Io(error)) if error.to_string() == FOLDER_LINK),
        "{coverage:?}"
    );
    assert_eq!(listing(&outside), Vec::<String>::new());
}

/// A pasta de build e os marcadores dela: com o `.kinein` de verdade, a
/// pasta de build ligada ou um marcador ligado para o que não existe param
/// o configure antes de criar algo, e o alvo segue sem existir.
#[test]
fn the_configure_refuses_a_linked_build_folder_or_marker() {
    let dir = base("build");
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(root.join(".kinein").join("build")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let build = root.join(".kinein").join("build").join("debug");
    link(&outside, &build);
    assert_eq!(
        crate::cmake::begin_configure(&root, &build, Some("debug"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        ".kinein/build/debug é um link simbólico, que a IDE não segue no projeto"
    );
    assert_eq!(listing(&outside), Vec::<String>::new());

    let release = root.join(".kinein").join("build").join("release");
    std::fs::create_dir_all(&release).unwrap();
    let missing = outside.join("trava");
    link(&missing, &release.join(".kinein-configure-lock"));
    assert_eq!(
        crate::cmake::begin_configure(&root, &release, Some("release"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        ".kinein/build/release/.kinein-configure-lock é um link simbólico, que a IDE não segue no projeto"
    );
    assert!(!missing.exists());
    assert!(!root.join(".kinein").join("configuration.json").exists());
}

/// O banco dos rascunhos e os arquivos do `SQLite` ao lado dele: com o
/// `.kinein` de verdade, o banco, o `-wal`, o `-shm` ou o `-journal` ligados
/// para o que não existe desligam a persistência, e o alvo segue sem existir.
/// Os três de lado o `SQLite` recusa sozinho (medido em 2026-11-12); o teste
/// fica como guarda se ele mudar.
#[test]
fn the_draft_store_refuses_each_linked_database_file() {
    for name in [
        "kinein.db",
        "kinein.db-wal",
        "kinein.db-shm",
        "kinein.db-journal",
    ] {
        let dir = base(&format!("banco-{name}"));
        let root = dir.join("projeto");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        let missing = dir.join("fora");
        link(&missing, &root.join(".kinein").join(name));
        assert!(crate::db::DraftStore::open(&root).is_none(), "{name}");
        assert!(!missing.exists(), "{name}");
    }
}
