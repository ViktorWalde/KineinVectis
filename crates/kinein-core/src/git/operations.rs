//! Invocacao dos comandos Git e guardas das operacoes do dominio.

use std::{
    path::Path,
    process::{Command, Output},
};

use kinein_protocol::{
    GitBlameResult, GitBranchInfo, GitBranchesResult, GitCommitDiffResult, GitDiffHunkInfo,
    GitFileDiffResult, GitHunkKind, GitLogResult, GitStatusResult,
};

use super::GitError;
use super::parse::{
    non_repo_status, parse_blame, parse_hunks, parse_log, parse_status, workspace_relative,
};

/// Runs `git status` for the workspace and chews porcelain into protocol.
pub fn status(root: &Path) -> Result<GitStatusResult, GitError> {
    if !is_inside_work_tree(root)? {
        return Ok(non_repo_status());
    }
    let prefix = workspace_prefix(root)?;
    // --untracked-files=all: sem isso o git colapsa diretorio untracked
    // numa entry "dir/" — inutil para colorir arquivos e quebra a conversao
    // de prefixo quando o workspace e subdiretorio do repo.
    let body = run_git(
        root,
        &[
            "status",
            "--porcelain=v2",
            "--branch",
            "--untracked-files=all",
            "-z",
        ],
    )?;
    Ok(parse_status(&body, &prefix))
}

/// Lists local branches without parsing localized human output.
pub fn branches(root: &Path) -> Result<GitBranchesResult, GitError> {
    if !is_inside_work_tree(root)? {
        return Ok(GitBranchesResult {
            repo: false,
            branches: Vec::new(),
        });
    }
    let body = run_git(
        root,
        &[
            "for-each-ref",
            "--sort=refname",
            "--format=%(refname:short)%09%(HEAD)",
            "refs/heads/",
        ],
    )?;
    let branches = body
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(name, head)| GitBranchInfo {
            name: name.to_owned(),
            current: head.trim() == "*",
        })
        .collect();
    Ok(GitBranchesResult {
        repo: true,
        branches,
    })
}

/// Asks Git itself whether a proposed short branch name is valid.
pub fn branch_name_valid(root: &Path, name: &str) -> Result<bool, GitError> {
    let output = Command::new("git")
        .args(["check-ref-format", "--branch", name])
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    Ok(output.status.success())
}

/// Switches to an existing local branch and returns fresh status.
pub fn checkout(root: &Path, branch: &str) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    run_git(root, &["switch", "--", branch])?;
    status(root)
}

/// Creates a local branch, optionally switching to it.
pub fn create_branch(
    root: &Path,
    name: &str,
    checkout_branch: bool,
) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    if checkout_branch {
        // `-c` consome o nome como o argumento seguinte; `--` entre ambos
        // seria interpretado como o proprio nome. `branch_name_valid` e a
        // barreira obrigatoria antes desta chamada.
        run_git(root, &["switch", "-c", name])?;
    } else {
        run_git(root, &["branch", "--", name])?;
    }
    status(root)
}

/// Fast-forward-only pull. Called inside the job system by the handler.
pub fn pull(root: &Path) -> Result<String, GitError> {
    ensure_repo(root)?;
    run_git(root, &["pull", "--ff-only"])
}

/// Pushes the current branch to its configured upstream.
pub fn push(root: &Path) -> Result<String, GitError> {
    ensure_repo(root)?;
    run_git(root, &["push"])
}

/// Stores tracked and untracked changes in a new stash.
pub fn stash_push(root: &Path, message: Option<&str>) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    // `.` limita a operacao ao workspace quando ele e apenas um subdiretorio
    // do repositorio. O pathspec de exclusao preserva configuracao, drafts e
    // sessao da IDE, que deliberadamente nao aparecem no painel Git.
    let workspace_pathspecs = ["--", ".", ":(exclude).kinein"];
    if let Some(message) = message.filter(|message| !message.trim().is_empty()) {
        let mut args = vec!["stash", "push", "--include-untracked", "-m", message];
        args.extend(workspace_pathspecs);
        run_git(root, &args)?;
    } else {
        let mut args = vec!["stash", "push", "--include-untracked"];
        args.extend(workspace_pathspecs);
        run_git(root, &args)?;
    }
    status(root)
}

/// Applies and removes the latest stash; conflicts are reported by Git.
pub fn stash_pop(root: &Path) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    run_git(root, &["stash", "pop"])?;
    status(root)
}

/// Runs `git diff` twice for one file: `--unified=0` feeds the gutter
/// hunks, `--unified=3` feeds the diff view. `path` is workspace-absolute
/// and already confined by the handler.
pub fn file_diff(root: &Path, path: &Path) -> Result<GitFileDiffResult, GitError> {
    let echoed = path.display().to_string();
    if !is_inside_work_tree(root)? {
        return Ok(GitFileDiffResult {
            path: echoed,
            repo: false,
            tracked: false,
            hunks: Vec::new(),
            text: String::new(),
        });
    }
    if !is_tracked(root, path)? {
        // git diff nao cobre untracked: o arquivo inteiro e um hunk added.
        let line_count = std::fs::read_to_string(path).map_or(0, |content| {
            u32::try_from(content.lines().count()).unwrap_or(u32::MAX)
        });
        return Ok(GitFileDiffResult {
            path: echoed,
            repo: true,
            tracked: false,
            hunks: vec![GitDiffHunkInfo {
                kind: GitHunkKind::Added,
                start_line: 1,
                line_count: line_count.max(1),
            }],
            text: String::new(),
        });
    }
    let path_argument = echoed.clone();
    let gutter = run_git(
        root,
        &[
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--unified=0",
            "HEAD",
            "--",
            &path_argument,
        ],
    )?;
    let text = run_git(
        root,
        &[
            "diff",
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "HEAD",
            "--",
            &path_argument,
        ],
    )?;
    Ok(GitFileDiffResult {
        path: echoed,
        repo: true,
        tracked: true,
        hunks: parse_hunks(&gutter),
        text,
    })
}

/// Runs `git blame --porcelain` for one tracked file and chews the
/// stream into line groups. `path` is workspace-absolute and already
/// confined by the handler.
pub fn blame(root: &Path, path: &Path) -> Result<GitBlameResult, GitError> {
    let echoed = path.display().to_string();
    if !is_inside_work_tree(root)? {
        return Ok(GitBlameResult {
            path: echoed,
            repo: false,
            tracked: false,
            groups: Vec::new(),
        });
    }
    // Sem HEAD (repo recem-criado) o blame falha; arquivo so staged e
    // "tracked" mas 100% nao-commitado — responder vazio e honesto.
    if !is_tracked(root, path)? || !has_head(root)? {
        return Ok(GitBlameResult {
            path: echoed,
            repo: true,
            tracked: false,
            groups: Vec::new(),
        });
    }
    let body = run_git(root, &["blame", "--porcelain", "--", &echoed])?;
    Ok(GitBlameResult {
        path: echoed,
        repo: true,
        tracked: true,
        groups: parse_blame(&body),
    })
}

/// Lists up to `max_count` commits, newest first (stable `%x1f`/NUL
/// format). A repo without commits answers an empty list.
pub fn log(root: &Path, max_count: u32) -> Result<GitLogResult, GitError> {
    if !is_inside_work_tree(root)? {
        return Ok(GitLogResult {
            repo: false,
            entries: Vec::new(),
        });
    }
    if !has_head(root)? {
        return Ok(GitLogResult {
            repo: true,
            entries: Vec::new(),
        });
    }
    let count_argument = format!("--max-count={max_count}");
    let body = run_git(
        root,
        &[
            "log",
            &count_argument,
            "-z",
            "--pretty=format:%H\u{1f}%h\u{1f}%an\u{1f}%at\u{1f}%s\u{1f}%P\u{1f}%D",
        ],
    )?;
    Ok(GitLogResult {
        repo: true,
        entries: parse_log(&body),
    })
}

/// Answers the unified patch of one commit (`git show` with an empty
/// header; the UI already holds author/summary from `git.log`).
pub fn commit_diff(root: &Path, sha: &str) -> Result<GitCommitDiffResult, GitError> {
    ensure_repo(root)?;
    let text = run_git(
        root,
        &[
            "show",
            "--no-color",
            "--no-ext-diff",
            "--unified=3",
            "--pretty=format:",
            sha,
        ],
    )?;
    Ok(GitCommitDiffResult {
        sha: sha.to_owned(),
        text,
    })
}

/// Stages `paths` (`git add`) and answers the fresh status.
pub fn stage(root: &Path, paths: &[String]) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    run_git_with_paths(root, &["add", "--"], paths)?;
    status(root)
}

/// Unstages `paths` (`git restore --staged`) and answers the fresh status.
pub fn unstage(root: &Path, paths: &[String]) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    run_git_with_paths(root, &["restore", "--staged", "--"], paths)?;
    status(root)
}

/// Discards `paths` (DESTRUTIVO: restore p/ tracked, clean p/ untracked)
/// and answers the fresh status. A confirmacao e responsabilidade da UI.
pub fn discard(root: &Path, paths: &[String]) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    let mut tracked = Vec::new();
    let mut untracked = Vec::new();
    for path in paths {
        if is_tracked(root, Path::new(path))? {
            tracked.push(path.clone());
        } else {
            untracked.push(path.clone());
        }
    }
    if !tracked.is_empty() {
        run_git_with_paths(root, &["restore", "--staged", "--worktree", "--"], &tracked)?;
    }
    if !untracked.is_empty() {
        run_git_with_paths(root, &["clean", "-f", "--"], &untracked)?;
    }
    status(root)
}

/// Commits exactly the visible staged index and answers the fresh status.
///
/// A pathspec is intentionally not passed to `git commit`: that mode commits
/// worktree contents and would violate staged-only semantics. Instead, every
/// staged path must be visible in the opened workspace before the unchanged
/// index is committed.
pub fn commit(root: &Path, message: &str, amend: bool) -> Result<GitStatusResult, GitError> {
    ensure_repo(root)?;
    let staged = staged_paths(root)?;
    // Um amend so' de mensagem (nada staged) e' legitimo; um commit novo nao.
    if staged.is_empty() && !amend {
        return Err(GitError::NothingStaged);
    }

    let prefix = workspace_prefix(root)?;
    let invisible = staged
        .into_iter()
        .filter(|path| workspace_relative(path, &prefix).is_none())
        .collect::<Vec<_>>();
    if !invisible.is_empty() {
        return Err(GitError::InvisibleStagedPaths { paths: invisible });
    }

    if amend {
        run_git(root, &["commit", "--amend", "-m", message])?;
    } else {
        run_git(root, &["commit", "-m", message])?;
    }
    status(root)
}

/// Repository-relative staged paths, with rename detection disabled so both
/// sides of a cross-boundary rename are checked independently.
fn staged_paths(root: &Path) -> Result<Vec<String>, GitError> {
    let stdout = run_git_bytes(
        root,
        &[
            "diff",
            "--cached",
            "--name-only",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "-z",
        ],
    )?;
    let body = String::from_utf8(stdout).map_err(|error| GitError::Failed {
        message: format!("git diff retornou um path que nao e UTF-8 valido: {error}"),
    })?;
    Ok(body
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Workspace prefix relative to the repository root. Only Git's line ending
/// is removed; whitespace is a valid path character and must be preserved.
fn workspace_prefix(root: &Path) -> Result<String, GitError> {
    let prefix = run_git(root, &["rev-parse", "--show-prefix"])?;
    Ok(prefix.trim_end_matches(['\r', '\n']).to_owned())
}

/// Errors when the workspace is not a git repository (mutations only;
/// leituras respondem `repo: false`).
fn ensure_repo(root: &Path) -> Result<(), GitError> {
    if is_inside_work_tree(root)? {
        Ok(())
    } else {
        Err(GitError::NotARepo)
    }
}

/// Runs a git subcommand followed by `--`-separated paths.
fn run_git_with_paths(root: &Path, args: &[&str], paths: &[String]) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(args)
        .args(paths)
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    if !output.status.success() {
        return Err(command_failure(args, &output));
    }
    Ok(())
}

/// `true` when git tracks `path` (`ls-files --error-unmatch`, exit code).
fn is_tracked(root: &Path, path: &Path) -> Result<bool, GitError> {
    let args = ["ls-files", "--error-unmatch", "--"];
    let output = Command::new("git")
        .args(args)
        .arg(path)
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(command_failure(&args, &output)),
    }
}

/// `true` when the repository has any commit (`rev-parse --verify HEAD`,
/// exit code — nunca o stderr localizado).
fn has_head(root: &Path) -> Result<bool, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    Ok(output.status.success())
}

/// `true` when `root` lives inside a git work tree (exit-code based).
fn is_inside_work_tree(root: &Path) -> Result<bool, GitError> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    Ok(output.status.success())
}

/// Runs a git subcommand in `root`, capturing stdout (lossy UTF-8).
fn run_git(root: &Path, args: &[&str]) -> Result<String, GitError> {
    run_git_bytes(root, args).map(|stdout| String::from_utf8_lossy(&stdout).into_owned())
}

/// Runs a git subcommand in `root`, preserving stdout bytes for NUL parsing.
fn run_git_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| spawn_error(&error))?;
    if !output.status.success() {
        return Err(command_failure(args, &output));
    }
    Ok(output.stdout)
}

fn spawn_error(error: &std::io::Error) -> GitError {
    if error.kind() == std::io::ErrorKind::NotFound {
        GitError::MissingGit
    } else {
        GitError::Failed {
            message: format!("git nao pode ser executado: {error}"),
        }
    }
}

fn command_failure(args: &[&str], output: &Output) -> GitError {
    let stderr = String::from_utf8_lossy(&output.stderr);
    GitError::Failed {
        message: format!(
            "git {} falhou: {}",
            args.first().copied().unwrap_or("?"),
            stderr.trim().lines().last().unwrap_or("erro desconhecido")
        ),
    }
}
