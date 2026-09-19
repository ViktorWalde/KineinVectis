//! Git dispatch: guardas, confinamento e fluxos reais sobre o binario Git.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

fn git_repo(test_name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-real-{test_name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["config", "user.name", "Kinein Test"]);
    run_git(
        &root,
        &["config", "user.email", "kinein-test@example.invalid"],
    );
    root.canonicalize().unwrap()
}

fn run_git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {} falhou: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn wait_for_remote_job(
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
    job_id: &str,
    operation: &str,
) {
    let mut saw_remote_finished = false;
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("evento do Git remoto dentro do timeout");
        if event.method == "event.git.remoteFinished" {
            let params = event.params.as_ref().unwrap();
            if params["jobId"] == job_id {
                assert_eq!(params["operation"], operation);
                assert_eq!(params["success"], true);
                saw_remote_finished = true;
            }
        }
        if event.method == "event.job.finished" {
            let params = event.params.as_ref().unwrap();
            if params["jobId"] == job_id {
                assert_eq!(params["status"], "success");
                break;
            }
        }
    }
    assert!(saw_remote_finished, "faltou event.git.remoteFinished");
}

#[test]
fn git_status_requires_workspace_and_reports_non_repo() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-status", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = core_with_empty_search_path("git-status");

    let no_workspace = core.handle_request(&JsonRpcRequest::new(1_i64, "git.status", None));
    assert_eq!(
        no_workspace.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    let opened = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Diretorio sem git: repo=false e nunca um erro.
    let status = core.handle_request(&JsonRpcRequest::new(3_i64, "git.status", None));
    let result = status.response().result.as_ref().unwrap().clone();
    assert_eq!(result["repo"], false);
    assert_eq!(result["entries"].as_array().unwrap().len(), 0);
}

#[test]
fn git_file_diff_confines_paths_to_the_workspace() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-diff", std::process::id()));
    let root = base.join("ws");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.txt"), "conteudo\n").unwrap();
    std::fs::write(base.join("fora.txt"), "fora\n").unwrap();
    let mut core = core_with_empty_search_path("git-diff");

    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outside = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "git.fileDiff",
        Some(json!({ "path": base.join("fora.txt").to_str().unwrap() })),
    ));
    assert_eq!(
        outside.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let missing = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "git.fileDiff",
        Some(json!({ "path": root.join("nao-existe").to_str().unwrap() })),
    ));
    assert_eq!(
        missing.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // Workspace sem git: repo=false, nunca erro.
    let diff = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "git.fileDiff",
        Some(json!({ "path": root.join("a.txt").to_str().unwrap() })),
    ));
    let result = diff.response().result.as_ref().unwrap().clone();
    assert_eq!(result["repo"], false);
}

#[test]
fn git_mutations_validate_params_and_repo_state() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-mut", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "conteudo\n").unwrap();
    let mut core = core_with_empty_search_path("git-mut");
    let opened = core.handle_request(&JsonRpcRequest::new(
        20_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // paths vazio e mensagem vazia sao INVALID_PARAMS antes de tocar o git.
    let empty_paths = core.handle_request(&JsonRpcRequest::new(
        21_i64,
        "git.stage",
        Some(json!({ "paths": [] })),
    ));
    assert_eq!(
        empty_paths.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    let empty_message = core.handle_request(&JsonRpcRequest::new(
        22_i64,
        "git.commit",
        Some(json!({ "message": "  " })),
    ));
    assert_eq!(
        empty_message.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // fora do workspace (mesmo caminho inexistente) e INVALID_PARAMS.
    let outside = core.handle_request(&JsonRpcRequest::new(
        23_i64,
        "git.stage",
        Some(json!({ "paths": ["/etc/hosts"] })),
    ));
    assert_eq!(
        outside.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // diretorio sem git: mutacao e INVALID_REQUEST (leitura seria repo:false).
    let not_repo = core.handle_request(&JsonRpcRequest::new(
        24_i64,
        "git.stage",
        Some(json!({ "paths": [dir.join("a.txt").to_str().unwrap()] })),
    ));
    let error = not_repo.response().error.as_ref().unwrap().clone();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert!(error.message.contains("nao e um repositorio"));
}

#[test]
fn git_branch_checkout_and_stash_flow_through_dispatch() {
    let repo = git_repo("branches-stash");
    std::fs::write(repo.join("tracked.txt"), "base\n").unwrap();
    run_git(&repo, &["add", "tracked.txt"]);
    run_git(&repo, &["commit", "-qm", "base"]);
    let initial_branch = run_git(&repo, &["branch", "--show-current"])
        .trim()
        .to_owned();

    let mut core = core_with_empty_search_path("git-branches-stash");
    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let branches = core.handle_request(&JsonRpcRequest::new(31_i64, "git.branches", None));
    let branch_entries = branches.response().result.as_ref().unwrap()["branches"]
        .as_array()
        .unwrap();
    assert_eq!(branch_entries.len(), 1);
    assert_eq!(branch_entries[0]["name"], initial_branch);
    assert_eq!(branch_entries[0]["current"], true);

    let invalid = core.handle_request(&JsonRpcRequest::new(
        32_i64,
        "git.branchCreate",
        Some(json!({ "name": "bad branch" })),
    ));
    assert_eq!(
        invalid.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let created = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "git.branchCreate",
        Some(json!({ "name": "feature/semantic" })),
    ));
    assert_eq!(
        created.response().result.as_ref().unwrap()["branch"],
        "feature/semantic"
    );

    let checked_out = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "git.checkout",
        Some(json!({ "branch": initial_branch })),
    ));
    assert_eq!(
        checked_out.response().result.as_ref().unwrap()["branch"],
        initial_branch
    );

    std::fs::write(repo.join("tracked.txt"), "local\n").unwrap();
    std::fs::write(repo.join("untracked.txt"), "new\n").unwrap();
    // workspace.open cria `.kinein`; ele nao pode desaparecer no stash.
    assert!(repo.join(".kinein").is_dir());
    let stashed = core.handle_request(&JsonRpcRequest::new(
        35_i64,
        "git.stash",
        Some(json!({ "action": "push", "message": "ide-checkpoint" })),
    ));
    assert!(stashed.response().error.is_none());
    assert_eq!(
        std::fs::read_to_string(repo.join("tracked.txt")).unwrap(),
        "base\n"
    );
    assert!(!repo.join("untracked.txt").exists());
    assert!(repo.join(".kinein").is_dir());

    let popped = core.handle_request(&JsonRpcRequest::new(
        36_i64,
        "git.stash",
        Some(json!({ "action": "pop" })),
    ));
    assert!(popped.response().error.is_none());
    assert_eq!(
        std::fs::read_to_string(repo.join("tracked.txt")).unwrap(),
        "local\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("untracked.txt")).unwrap(),
        "new\n"
    );
}

#[test]
fn git_blame_log_and_commit_diff_validate_params_and_non_repo() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-blame-guards", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "conteudo\n").unwrap();
    let mut core = core_with_empty_search_path("git-blame-guards");
    let opened = core.handle_request(&JsonRpcRequest::new(
        60_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // blame sem path e path fora do workspace sao INVALID_PARAMS.
    let no_path = core.handle_request(&JsonRpcRequest::new(61_i64, "git.blame", None));
    assert_eq!(
        no_path.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    let outside = core.handle_request(&JsonRpcRequest::new(
        62_i64,
        "git.blame",
        Some(json!({ "path": "/etc/hosts" })),
    ));
    assert_eq!(
        outside.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // maxCount fora de 1..=500 e INVALID_PARAMS antes de tocar o git.
    for max_count in [0, 501] {
        let bad_count = core.handle_request(&JsonRpcRequest::new(
            63_i64,
            "git.log",
            Some(json!({ "maxCount": max_count })),
        ));
        assert_eq!(
            bad_count.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams
        );
    }

    // sha precisa ser hex 4..=64: nunca virar argv arbitrario do git.
    for sha in ["--flag", "xyz", "ab", ""] {
        let bad_sha = core.handle_request(&JsonRpcRequest::new(
            64_i64,
            "git.commitDiff",
            Some(json!({ "sha": sha })),
        ));
        assert_eq!(
            bad_sha.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams
        );
    }

    // Leituras em nao-repo respondem repo:false; commitDiff (so nasce da
    // lista do historico) e INVALID_REQUEST.
    let blame = core.handle_request(&JsonRpcRequest::new(
        65_i64,
        "git.blame",
        Some(json!({ "path": dir.join("a.txt").to_str().unwrap() })),
    ));
    assert_eq!(blame.response().result.as_ref().unwrap()["repo"], false);
    let log = core.handle_request(&JsonRpcRequest::new(66_i64, "git.log", None));
    assert_eq!(log.response().result.as_ref().unwrap()["repo"], false);
    let show = core.handle_request(&JsonRpcRequest::new(
        67_i64,
        "git.commitDiff",
        Some(json!({ "sha": "abcd1234" })),
    ));
    assert_eq!(
        show.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn git_blame_log_and_commit_diff_follow_real_history() {
    let repo = git_repo("blame-log");
    std::fs::write(repo.join("a.txt"), "linha um\nlinha dois\n").unwrap();
    run_git(&repo, &["add", "a.txt"]);
    run_git(&repo, &["commit", "-qm", "primeiro"]);
    std::fs::write(repo.join("a.txt"), "linha um\nlinha dois v2\n").unwrap();
    run_git(&repo, &["add", "a.txt"]);
    run_git(
        &repo,
        &[
            "-c",
            "user.name=Outra Autora",
            "-c",
            "user.email=outra@example.invalid",
            "commit",
            "-qm",
            "segundo",
        ],
    );
    std::fs::write(repo.join("solto.txt"), "untracked\n").unwrap();

    let mut core = core_with_empty_search_path("git-blame-log");
    let opened = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Blame: linha 1 do primeiro autor, linha 2 do segundo, tudo committed.
    let blame = core.handle_request(&JsonRpcRequest::new(
        71_i64,
        "git.blame",
        Some(json!({ "path": repo.join("a.txt").to_str().unwrap() })),
    ));
    let result = blame.response().result.as_ref().unwrap().clone();
    assert_eq!(result["repo"], true);
    assert_eq!(result["tracked"], true);
    let groups = result["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0]["startLine"], 1);
    assert_eq!(groups[0]["author"], "Kinein Test");
    assert_eq!(groups[0]["committed"], true);
    assert_eq!(groups[1]["startLine"], 2);
    assert_eq!(groups[1]["author"], "Outra Autora");
    assert_eq!(groups[1]["summary"], "segundo");

    // Untracked: tracked:false com groups vazio, nunca erro.
    let untracked = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "git.blame",
        Some(json!({ "path": repo.join("solto.txt").to_str().unwrap() })),
    ));
    let untracked_result = untracked.response().result.as_ref().unwrap().clone();
    assert_eq!(untracked_result["tracked"], false);
    assert!(untracked_result["groups"].as_array().unwrap().is_empty());

    // Log: 2 commits, mais novo primeiro, campos estaveis.
    let log = core.handle_request(&JsonRpcRequest::new(73_i64, "git.log", None));
    let log_result = log.response().result.as_ref().unwrap().clone();
    let entries = log_result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["summary"], "segundo");
    assert_eq!(entries[0]["author"], "Outra Autora");
    assert_eq!(entries[1]["summary"], "primeiro");
    // 0.126.0 (HUD do Git): os pais desenham o grafo, os refs viram chips.
    assert_eq!(entries[0]["parents"][0], entries[1]["sha"]);
    assert!(
        entries[1].get("parents").is_none(),
        "o primeiro commit nao tem pai"
    );
    let refs = entries[0]["refs"].as_array().unwrap();
    assert!(
        refs.iter()
            .any(|r| r.as_str().unwrap().starts_with("HEAD -> ")),
        "{refs:?}"
    );

    // commitDiff do commit mais novo contem a mudanca da linha 2.
    let sha = entries[0]["sha"].as_str().unwrap().to_owned();
    let show = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "git.commitDiff",
        Some(json!({ "sha": sha })),
    ));
    let show_result = show.response().result.as_ref().unwrap().clone();
    assert_eq!(show_result["sha"], sha);
    let text = show_result["text"].as_str().unwrap();
    assert!(text.contains("+linha dois v2"));
    assert!(text.contains("-linha dois"));
}

/// Amend (0.126.0): reescreve o ultimo commit com a mensagem nova; sem nada
/// staged e' legitimo (so' a mensagem muda) e o log continua com o mesmo
/// numero de commits; um commit NOVO sem nada staged segue recusado.
#[test]
fn git_commit_amend_rewrites_the_last_commit() {
    let repo = git_repo("amend");
    std::fs::write(repo.join("a.txt"), "um\n").unwrap();
    run_git(&repo, &["add", "a.txt"]);
    run_git(&repo, &["commit", "-qm", "primeiro"]);
    std::fs::write(repo.join("a.txt"), "dois\n").unwrap();
    run_git(&repo, &["add", "a.txt"]);
    run_git(&repo, &["commit", "-qm", "segundo"]);
    let mut core = core_with_empty_search_path("git-amend");
    let opened = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    // Amend (0.126.0): reescreve o ultimo commit com a mensagem nova; sem
    // nada staged e' legitimo (so' a mensagem muda), e o log continua com 2.
    let amended = core.handle_request(&JsonRpcRequest::new(
        75_i64,
        "git.commit",
        Some(json!({ "message": "segundo, emendado", "amend": true })),
    ));
    assert!(
        amended.response().error.is_none(),
        "{:?}",
        amended.response().error
    );
    let log = core.handle_request(&JsonRpcRequest::new(76_i64, "git.log", None));
    let log_result = log.response().result.as_ref().unwrap().clone();
    let entries = log_result["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["summary"], "segundo, emendado");
    let sem_amend = core.handle_request(&JsonRpcRequest::new(
        77_i64,
        "git.commit",
        Some(json!({ "message": "nada staged" })),
    ));
    assert!(
        sem_amend.response().error.is_some(),
        "commit novo sem nada staged e' recusado"
    );
}

/// `git.log { ref }` (0.127.0): o historico de OUTRO branch, sem trocar de
/// branch; um ref que parece opcao ou intervalo e' recusado antes do git;
/// um ref inexistente e' erro do git (nao um log vazio).
#[test]
fn git_log_walks_from_a_named_ref_and_rejects_unsafe_refs() {
    let repo = git_repo("log-ref");
    std::fs::write(repo.join("a.txt"), "um\n").unwrap();
    run_git(&repo, &["add", "a.txt"]);
    run_git(&repo, &["commit", "-qm", "base"]);
    let principal = run_git(&repo, &["branch", "--show-current"])
        .trim()
        .to_owned();
    run_git(&repo, &["checkout", "-qb", "feature"]);
    std::fs::write(repo.join("b.txt"), "dois\n").unwrap();
    run_git(&repo, &["add", "b.txt"]);
    run_git(&repo, &["commit", "-qm", "so' na feature"]);
    run_git(&repo, &["checkout", "-q", &principal]);

    let mut core = core_with_empty_search_path("git-log-ref");
    let opened = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let atual = core.handle_request(&JsonRpcRequest::new(81_i64, "git.log", None));
    assert_eq!(
        atual.response().result.as_ref().unwrap()["entries"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let feature = core.handle_request(&JsonRpcRequest::new(
        82_i64,
        "git.log",
        Some(json!({ "ref": "feature" })),
    ));
    let entries = feature.response().result.as_ref().unwrap()["entries"].clone();
    assert_eq!(entries.as_array().unwrap().len(), 2, "{entries}");
    assert_eq!(entries[0]["summary"], "so' na feature");

    for ruim in ["--all", "main..feature", "a b"] {
        let r = core.handle_request(&JsonRpcRequest::new(
            83_i64,
            "git.log",
            Some(json!({ "ref": ruim })),
        ));
        assert_eq!(
            r.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams,
            "{ruim}"
        );
    }
    let inexistente = core.handle_request(&JsonRpcRequest::new(
        84_i64,
        "git.log",
        Some(json!({ "ref": "nao-existe" })),
    ));
    assert!(inexistente.response().error.is_some());
}

#[test]
fn git_log_answers_empty_on_repo_without_commits() {
    let repo = git_repo("log-empty");
    let mut core = core_with_empty_search_path("git-log-empty");
    let opened = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let log = core.handle_request(&JsonRpcRequest::new(81_i64, "git.log", None));
    let result = log.response().result.as_ref().unwrap().clone();
    assert_eq!(result["repo"], true);
    assert!(result["entries"].as_array().unwrap().is_empty());
}

#[test]
fn git_pull_and_push_run_as_jobs_against_a_local_remote() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-git-remote-jobs", std::process::id()));
    let remote = base.join("remote.git");
    let repo = base.join("workspace");
    let peer = base.join("peer");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    run_git(&base, &["init", "--bare", "-q", remote.to_str().unwrap()]);
    run_git(
        &base,
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            repo.to_str().unwrap(),
        ],
    );
    run_git(&repo, &["config", "user.name", "Kinein Test"]);
    run_git(
        &repo,
        &["config", "user.email", "kinein-test@example.invalid"],
    );
    std::fs::write(repo.join("shared.txt"), "base\n").unwrap();
    run_git(&repo, &["add", "shared.txt"]);
    run_git(&repo, &["commit", "-qm", "base"]);
    run_git(&repo, &["push", "-qu", "origin", "HEAD"]);

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("git-remote-jobs");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        90_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    std::fs::write(repo.join("pushed.txt"), "from ide\n").unwrap();
    run_git(&repo, &["add", "pushed.txt"]);
    run_git(&repo, &["commit", "-qm", "push from ide"]);
    let push = core.handle_request(&JsonRpcRequest::new(91_i64, "git.push", None));
    let push_id = push.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for_remote_job(&receiver, &push_id, "push");

    run_git(
        &base,
        &[
            "clone",
            "-q",
            remote.to_str().unwrap(),
            peer.to_str().unwrap(),
        ],
    );
    run_git(&peer, &["config", "user.name", "Peer Test"]);
    run_git(&peer, &["config", "user.email", "peer@example.invalid"]);
    std::fs::write(peer.join("pulled.txt"), "from peer\n").unwrap();
    run_git(&peer, &["add", "pulled.txt"]);
    run_git(&peer, &["commit", "-qm", "peer update"]);
    run_git(&peer, &["push", "-q"]);

    let pull = core.handle_request(&JsonRpcRequest::new(92_i64, "git.pull", None));
    let pull_id = pull.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_for_remote_job(&receiver, &pull_id, "pull");
    assert_eq!(
        std::fs::read_to_string(repo.join("pulled.txt")).unwrap(),
        "from peer\n"
    );
}

#[test]
fn command_list_includes_git_daily_driver_operations() {
    let mut core = core_with_empty_search_path("git-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(4_i64, "command.list", None));
    let result = outcome.response().result.as_ref().unwrap().clone();
    let ids = result["commands"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|command| command["id"].as_str())
        .collect::<Vec<_>>();
    for expected in [
        "git.status",
        "git.branches",
        "git.pull",
        "git.push",
        "git.stash",
    ] {
        assert!(ids.contains(&expected), "comando ausente: {expected}");
    }
}

#[test]
fn git_commit_rejects_staged_paths_outside_subworkspace_and_preserves_index() {
    let repo = git_repo("commit-scope");
    let workspace = repo.join("sub");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(workspace.join("inside.txt"), "inside base\n").unwrap();
    std::fs::write(repo.join("outside.txt"), "outside base\n").unwrap();
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-qm", "initial"]);

    std::fs::write(workspace.join("inside.txt"), "inside staged\n").unwrap();
    std::fs::write(repo.join("outside.txt"), "outside staged\n").unwrap();
    run_git(&repo, &["add", "outside.txt"]);

    let mut core = core_with_empty_search_path("git-commit-scope");
    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": workspace.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let inside = workspace.join("inside.txt").display().to_string();
    let staged = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "git.stage",
        Some(json!({ "paths": [inside] })),
    ));
    assert!(staged.response().error.is_none());

    // The worktree advances after staging; a staged-only commit must keep the
    // staged snapshot and leave this newer content unstaged.
    std::fs::write(workspace.join("inside.txt"), "inside worktree\n").unwrap();

    let visible = core.handle_request(&JsonRpcRequest::new(32_i64, "git.status", None));
    let entries = visible.response().result.as_ref().unwrap()["entries"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["path"], "inside.txt");
    assert_eq!(entries[0]["staged"], true);

    let head_before = run_git(&repo, &["rev-parse", "HEAD"]);
    let rejected = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "git.commit",
        Some(json!({ "message": "scoped commit" })),
    ));
    let error = rejected.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert_eq!(
        error.details.as_ref().unwrap()["paths"],
        json!(["outside.txt"])
    );
    assert_eq!(run_git(&repo, &["rev-parse", "HEAD"]), head_before);

    let staged_names = run_git(&repo, &["diff", "--cached", "--name-only", "--no-renames"]);
    assert!(staged_names.lines().any(|path| path == "outside.txt"));
    assert!(staged_names.lines().any(|path| path == "sub/inside.txt"));

    run_git(&repo, &["restore", "--staged", "--", "outside.txt"]);
    let committed = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "git.commit",
        Some(json!({ "message": "scoped commit" })),
    ));
    assert!(committed.response().error.is_none());

    assert_eq!(
        run_git(&repo, &["show", "HEAD:sub/inside.txt"]),
        "inside staged\n"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.join("inside.txt")).unwrap(),
        "inside worktree\n"
    );
    let committed_paths = run_git(&repo, &["show", "--pretty=format:", "--name-only", "HEAD"]);
    assert_eq!(committed_paths.trim(), "sub/inside.txt");
}

#[test]
fn git_commit_rejects_staged_kinein_metadata_hidden_from_status() {
    let repo = git_repo("commit-hidden-metadata");
    std::fs::write(repo.join("tracked.txt"), "base\n").unwrap();
    run_git(&repo, &["add", "tracked.txt"]);
    run_git(&repo, &["commit", "-qm", "initial"]);

    let mut core = core_with_empty_search_path("git-hidden-metadata");
    let opened = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    run_git(&repo, &["add", ".kinein/workspace.json"]);

    let visible = core.handle_request(&JsonRpcRequest::new(41_i64, "git.status", None));
    assert!(
        visible.response().result.as_ref().unwrap()["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let rejected = core.handle_request(&JsonRpcRequest::new(
        42_i64,
        "git.commit",
        Some(json!({ "message": "must stay visible" })),
    ));
    let error = rejected.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert_eq!(
        error.details.as_ref().unwrap()["paths"],
        json!([".kinein/workspace.json"])
    );
}

#[test]
fn git_index_failures_are_not_misclassified_as_normal_state() {
    let repo = git_repo("broken-index");
    std::fs::write(repo.join("tracked.txt"), "base\n").unwrap();
    run_git(&repo, &["add", "tracked.txt"]);
    run_git(&repo, &["commit", "-qm", "initial"]);

    let mut core = core_with_empty_search_path("git-broken-index");
    let opened = core.handle_request(&JsonRpcRequest::new(
        50_i64,
        "workspace.open",
        Some(json!({ "path": repo.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    std::fs::write(repo.join(".git/index"), "indice corrompido").unwrap();

    let diff = core.handle_request(&JsonRpcRequest::new(
        51_i64,
        "git.fileDiff",
        Some(json!({ "path": repo.join("tracked.txt").to_str().unwrap() })),
    ));
    assert_eq!(
        diff.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InternalError
    );

    let commit = core.handle_request(&JsonRpcRequest::new(
        52_i64,
        "git.commit",
        Some(json!({ "message": "must fail" })),
    ));
    assert_eq!(
        commit.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InternalError
    );
}
