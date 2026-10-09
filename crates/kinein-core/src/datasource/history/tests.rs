//! O historico prova o que promete: ordem, repeticao, tetos, separacao por
//! projeto e por conexao, arquivo estranho intacto e permissoes privadas.

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::*;

fn dirs(name: &str) -> (PathBuf, PathBuf, History) {
    let base = std::env::temp_dir()
        .join("kinein-history-unit")
        .join(format!("{}-{name}", std::process::id()));
    drop(fs::remove_dir_all(&base));
    let project = base.join("projeto");
    fs::create_dir_all(&project).unwrap();
    let state = base.join("estado").join("datasource-history");
    (project, state.clone(), History::new(state))
}

fn texts(entries: &[DataSourceHistoryEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.sql.as_str()).collect()
}

#[test]
fn newest_first_and_an_immediate_repeat_updates_the_top() {
    let (project, state, history) = dirs("ordem");
    history
        .record(
            &project,
            "a",
            "select 1",
            DataSourceHistoryOutcome::Ok,
            Some(1),
        )
        .unwrap();
    // As pastas que o core cria ja' nascem so' do dono, a mae inclusive. No
    // Windows quem garante isso e' a ACL da pasta do usuario (60 §3.1, D3).
    #[cfg(unix)]
    {
        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(state.parent().unwrap()), 0o700);
    }
    #[cfg(not(unix))]
    let _ = &state;
    history
        .record(
            &project,
            "a",
            "select 2",
            DataSourceHistoryOutcome::Failed,
            None,
        )
        .unwrap();
    history
        .record(
            &project,
            "a",
            "select 2",
            DataSourceHistoryOutcome::Ok,
            Some(400),
        )
        .unwrap();
    let entries = history.list(&project, "a");
    assert_eq!(texts(&entries), ["select 2", "select 1"]);
    assert_eq!(entries[0].outcome, DataSourceHistoryOutcome::Ok);
    assert_eq!(entries[0].rows, Some(400));
    // Repetir um texto MAIS ANTIGO e' uma entrada nova no topo.
    history
        .record(
            &project,
            "a",
            "select 1",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    assert_eq!(
        texts(&history.list(&project, "a")),
        ["select 1", "select 2", "select 1"]
    );
}

#[test]
fn the_ceilings_keep_the_newest_and_skip_a_huge_text() {
    let (project, _, history) = dirs("tetos");
    for index in 0..MAX_ENTRIES + 5 {
        history
            .record(
                &project,
                "a",
                &format!("select {index}"),
                DataSourceHistoryOutcome::Ok,
                None,
            )
            .unwrap();
    }
    let entries = history.list(&project, "a");
    assert_eq!(entries.len(), MAX_ENTRIES);
    assert_eq!(entries[0].sql, format!("select {}", MAX_ENTRIES + 4));
    let huge = "x".repeat(MAX_SQL_BYTES + 1);
    history
        .record(&project, "a", &huge, DataSourceHistoryOutcome::Ok, None)
        .unwrap();
    history
        .record(&project, "a", "   ", DataSourceHistoryOutcome::Ok, None)
        .unwrap();
    assert_eq!(
        history.list(&project, "a")[0].sql,
        format!("select {}", MAX_ENTRIES + 4)
    );
}

#[test]
fn connections_and_projects_are_separate_and_clear_is_per_connection() {
    let (project, state, history) = dirs("separado");
    let other = project.with_file_name("outro");
    fs::create_dir_all(&other).unwrap();
    history
        .record(
            &project,
            "a",
            "select a",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    history
        .record(
            &project,
            "b",
            "select b",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    history
        .record(
            &other,
            "a",
            "select outro",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    assert_eq!(texts(&history.list(&project, "a")), ["select a"]);
    assert_eq!(texts(&history.list(&other, "a")), ["select outro"]);
    assert_eq!(
        fs::read_dir(&state).unwrap().count(),
        2,
        "um arquivo por projeto"
    );
    history.clear(&project, "a").unwrap();
    assert!(history.list(&project, "a").is_empty());
    assert_eq!(texts(&history.list(&project, "b")), ["select b"]);
    assert_eq!(texts(&history.list(&other, "a")), ["select outro"]);
    // Limpar o que nao existe nao cria arquivo nem falha.
    let (empty, empty_state, fresh) = dirs("limpar-vazio");
    fresh.clear(&empty, "a").unwrap();
    assert!(!empty_state.exists());
}

#[test]
#[cfg(unix)]
fn the_same_project_by_a_symlink_shares_the_history() {
    let (project, _, history) = dirs("link");
    let link = project.with_file_name("atalho");
    std::os::unix::fs::symlink(&project, &link).unwrap();
    history
        .record(&link, "a", "select 1", DataSourceHistoryOutcome::Ok, None)
        .unwrap();
    assert_eq!(texts(&history.list(&project, "a")), ["select 1"]);
}

#[test]
fn an_unreadable_or_foreign_file_stays_intact() {
    let (project, state, history) = dirs("intacto");
    history
        .record(
            &project,
            "a",
            "select 1",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    let path = fs::read_dir(&state)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    // Ilegivel, de outra versao e (versao certa) de OUTRO projeto.
    let foreign = r#"{"schemaVersion":1,"workspace":"/outro/projeto","connections":{"a":[{"sql":"select alheio","at":1,"outcome":"ok"}]}}"#;
    for body in [
        "isto nao e json",
        r#"{"schemaVersion":9,"workspace":"/x","connections":{}}"#,
        foreign,
    ] {
        fs::write(&path, body).unwrap();
        assert!(history.list(&project, "a").is_empty());
        assert!(
            history
                .record(
                    &project,
                    "a",
                    "select 2",
                    DataSourceHistoryOutcome::Ok,
                    None
                )
                .is_err()
        );
        assert!(history.clear(&project, "a").is_err());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            body,
            "o arquivo foi reescrito"
        );
    }
}

#[test]
#[cfg(unix)]
fn the_folder_and_the_file_are_private() {
    let (project, state, history) = dirs("privado");
    // A pasta ja' existia mais aberta: passa a ser so' do dono.
    fs::create_dir_all(&state).unwrap();
    fs::set_permissions(&state, fs::Permissions::from_mode(0o755)).unwrap();
    history
        .record(
            &project,
            "a",
            "select 1",
            DataSourceHistoryOutcome::Ok,
            None,
        )
        .unwrap();
    let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&state), 0o700);
    let path = fs::read_dir(&state)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(mode(&path), 0o600);
}
