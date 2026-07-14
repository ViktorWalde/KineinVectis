//! Handlers for `git.*` requests (`impl Core`).

use std::path::Path;

use kinein_protocol::{
    GitBlameParams, GitBranchCreateParams, GitCheckoutParams, GitCommitDiffParams, GitCommitParams,
    GitFileDiffParams, GitLogParams, GitPathsParams, GitStashAction, GitStashParams,
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, git};

impl Core {
    /// Roteia os metodos `git.*`; `None` quando o metodo nao e de git.
    pub(crate) fn git_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "git.status" => Some(self.git_status_response(request_id)),
            "git.branches" => Some(self.git_branches_response(request_id)),
            "git.checkout" => Some(self.git_checkout_response(request_id, params)),
            "git.branchCreate" => Some(self.git_branch_create_response(request_id, params)),
            "git.pull" | "git.push" => Some(self.git_remote_response(request_id, method)),
            "git.stash" => Some(self.git_stash_response(request_id, params)),
            "git.fileDiff" => Some(self.git_file_diff_response(request_id, params)),
            "git.blame" => Some(self.git_blame_response(request_id, params)),
            "git.log" => Some(self.git_log_response(request_id, params)),
            "git.commitDiff" => Some(self.git_commit_diff_response(request_id, params)),
            "git.stage" | "git.unstage" | "git.discard" => {
                Some(self.git_paths_mutation_response(request_id, method, params))
            }
            "git.commit" => Some(self.git_commit_response(request_id, params)),
            _ => None,
        }
    }

    fn git_branches_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.branches");
        };
        match git::branches(&root) {
            Ok(branches) => JsonRpcResponse::success(request_id, json!(branches)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_checkout_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.checkout");
        };
        let parsed = match parse_params::<GitCheckoutParams>(
            request_id.as_ref(),
            params,
            "git.checkout requer branch",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Err(response) = validate_branch(&root, request_id.as_ref(), &parsed.branch) {
            return *response;
        }
        match git::checkout(&root, &parsed.branch) {
            Ok(status) => {
                self.syntax.clear();
                self.workspace_edits.clear();
                JsonRpcResponse::success(request_id, json!(status))
            }
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_branch_create_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.branchCreate");
        };
        let parsed = match parse_params::<GitBranchCreateParams>(
            request_id.as_ref(),
            params,
            "git.branchCreate requer name e checkout opcional",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Err(response) = validate_branch(&root, request_id.as_ref(), &parsed.name) {
            return *response;
        }
        match git::create_branch(&root, &parsed.name, parsed.checkout) {
            Ok(status) => {
                self.syntax.clear();
                self.workspace_edits.clear();
                JsonRpcResponse::success(request_id, json!(status))
            }
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_stash_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.stash");
        };
        let parsed = match parse_params::<GitStashParams>(
            request_id.as_ref(),
            params,
            "git.stash requer action (push ou pop) e message opcional",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let outcome = match parsed.action {
            GitStashAction::Push => git::stash_push(&root, parsed.message.as_deref()),
            GitStashAction::Pop => git::stash_pop(&root),
        };
        match outcome {
            Ok(status) => {
                self.syntax.clear();
                self.workspace_edits.clear();
                JsonRpcResponse::success(request_id, json!(status))
            }
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_remote_response(&mut self, request_id: Option<Value>, method: &str) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, method);
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": method })),
                ),
            );
        };
        self.syntax.clear();
        self.workspace_edits.clear();
        let operation = if method == "git.pull" { "pull" } else { "push" };
        let title = if operation == "pull" {
            "Git Pull"
        } else {
            "Git Push"
        };
        let job_id = jobs.spawn(method, title, JobRisk::High, false, move |ctx| {
            ctx.report_progress(0.05, Some(title));
            let outcome = if operation == "pull" {
                git::pull(&root)
            } else {
                git::push(&root)
            };
            match outcome {
                Ok(output) => {
                    for line in output.lines() {
                        ctx.emit_output(line);
                    }
                    ctx.report_progress(1.0, Some("concluido"));
                    ctx.emit_event(
                        "event.git.remoteFinished",
                        json!({
                            "jobId": ctx.id(),
                            "operation": operation,
                            "success": true,
                            "message": "concluido",
                        }),
                    );
                    crate::jobs::JobOutcome::Success
                }
                Err(error) => {
                    ctx.emit_output(&error.to_string());
                    ctx.emit_event(
                        "event.git.remoteFinished",
                        json!({
                            "jobId": ctx.id(),
                            "operation": operation,
                            "success": false,
                            "message": error.to_string(),
                        }),
                    );
                    crate::jobs::JobOutcome::Failed
                }
            }
        });
        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }

    fn git_blame_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.blame");
        };
        let parsed = match parse_params::<GitBlameParams>(
            request_id.as_ref(),
            params,
            "git.blame requer o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match confined_existing_file(&root, request_id.as_ref(), &parsed.path) {
            Ok(path) => path,
            Err(response) => return *response,
        };
        match git::blame(&root, &path) {
            Ok(blame) => JsonRpcResponse::success(request_id, json!(blame)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_log_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.log");
        };
        let parsed = match parse_params::<GitLogParams>(
            request_id.as_ref(),
            params,
            "git.log aceita apenas o campo opcional maxCount",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let max_count = parsed.max_count.unwrap_or(100);
        if !(1..=500).contains(&max_count) {
            return invalid_git_params(request_id, "maxCount deve estar entre 1 e 500");
        }
        match git::log(&root, max_count) {
            Ok(log) => JsonRpcResponse::success(request_id, json!(log)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_commit_diff_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.commitDiff");
        };
        let parsed = match parse_params::<GitCommitDiffParams>(
            request_id.as_ref(),
            params,
            "git.commitDiff requer o campo sha",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        // sha vira argv do git: validar o formato aqui e a unica barreira
        // contra strings arbitrarias (ex.: "--flag") chegarem ao binario.
        let valid_sha = (4..=64).contains(&parsed.sha.len())
            && parsed
                .sha
                .chars()
                .all(|character| character.is_ascii_hexdigit());
        if !valid_sha {
            return invalid_git_params(request_id, "sha deve ter 4..=64 caracteres hexadecimais");
        }
        match git::commit_diff(&root, &parsed.sha) {
            Ok(diff) => JsonRpcResponse::success(request_id, json!(diff)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_paths_mutation_response(
        &self,
        request_id: Option<Value>,
        method: &str,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, method);
        };
        let parsed = match parse_params::<GitPathsParams>(
            request_id.as_ref(),
            params,
            "requer o campo paths (lista de caminhos absolutos)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let paths = match confine_paths(&root, &parsed.paths) {
            Ok(paths) => paths,
            Err(message) => return invalid_git_params(request_id, &message),
        };
        let outcome = match method {
            "git.stage" => git::stage(&root, &paths),
            "git.unstage" => git::unstage(&root, &paths),
            _ => git::discard(&root, &paths),
        };
        match outcome {
            Ok(status) => JsonRpcResponse::success(request_id, json!(status)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_commit_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.commit");
        };
        let parsed = match parse_params::<GitCommitParams>(
            request_id.as_ref(),
            params,
            "git.commit requer o campo message",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.message.trim().is_empty() {
            return invalid_git_params(request_id, "a mensagem do commit nao pode ser vazia");
        }
        match git::commit(&root, &parsed.message) {
            Ok(status) => JsonRpcResponse::success(request_id, json!(status)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_file_diff_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.fileDiff");
        };
        let parsed = match parse_params::<GitFileDiffParams>(
            request_id.as_ref(),
            params,
            "git.fileDiff requer o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match confined_existing_file(&root, request_id.as_ref(), &parsed.path) {
            Ok(path) => path,
            Err(response) => return *response,
        };
        match git::file_diff(&root, &path) {
            Ok(diff) => JsonRpcResponse::success(request_id, json!(diff)),
            Err(error) => git_error_response(request_id, &error),
        }
    }

    fn git_status_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "git.status");
        };
        match git::status(&root) {
            Ok(status) => JsonRpcResponse::success(request_id, json!(status)),
            Err(error) => git_error_response(request_id, &error),
        }
    }
}

/// Mapeia um `GitError` para a resposta JSON-RPC padrao do dominio.
fn git_error_response(request_id: Option<Value>, error: &git::GitError) -> JsonRpcResponse {
    let code = match error {
        git::GitError::MissingGit => JsonRpcErrorCode::ToolNotFound,
        git::GitError::NotARepo
        | git::GitError::NothingStaged
        | git::GitError::InvisibleStagedPaths { .. } => JsonRpcErrorCode::InvalidRequest,
        git::GitError::Failed { .. } => JsonRpcErrorCode::InternalError,
    };
    let details = match error {
        git::GitError::InvisibleStagedPaths { paths } => Some(json!({ "paths": paths })),
        _ => None,
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(code, error.to_string(), details),
    )
}

/// Falha `INVALID_PARAMS` com a mensagem de dominio do git.
fn invalid_git_params(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}

fn validate_branch(
    root: &Path,
    request_id: Option<&Value>,
    branch: &str,
) -> Result<(), Box<JsonRpcResponse>> {
    if branch.trim().is_empty() {
        return Err(Box::new(invalid_git_params(
            request_id.cloned(),
            "o nome da branch nao pode ser vazio",
        )));
    }
    match git::branch_name_valid(root, branch) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Box::new(invalid_git_params(
            request_id.cloned(),
            "nome de branch invalido",
        ))),
        Err(error) => Err(Box::new(git_error_response(request_id.cloned(), &error))),
    }
}

/// Canonicaliza e confina ao workspace um arquivo que PRECISA existir em
/// disco (leituras `git.fileDiff`/`git.blame`; mutacoes com arquivo
/// deletado usam `confine_paths`).
fn confined_existing_file(
    root: &Path,
    request_id: Option<&Value>,
    raw: &str,
) -> Result<std::path::PathBuf, Box<JsonRpcResponse>> {
    let Ok(path) = Path::new(raw).canonicalize() else {
        return Err(Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                format!("arquivo nao encontrado: {raw}"),
                None,
            ),
        )));
    };
    if !path.starts_with(root) {
        return Err(Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                "arquivo fora do workspace aberto",
                None,
            ),
        )));
    }
    Ok(path)
}

/// Confina cada path ao workspace. Arquivo DELETADO nao canonicaliza:
/// nesse caso vale a checagem lexical (absoluto, dentro do root, sem "..").
fn confine_paths(root: &std::path::PathBuf, paths: &[String]) -> Result<Vec<String>, String> {
    if paths.is_empty() {
        return Err("paths nao pode ser vazio".to_owned());
    }
    let mut confined = Vec::with_capacity(paths.len());
    for raw in paths {
        let path = Path::new(raw);
        let inside = match path.canonicalize() {
            Ok(canonical) => canonical.starts_with(root),
            Err(_missing) => {
                path.is_absolute()
                    && path.starts_with(root)
                    && !raw.split('/').any(|segment| segment == "..")
            }
        };
        if !inside {
            return Err(format!("caminho fora do workspace aberto: {raw}"));
        }
        confined.push(raw.clone());
    }
    Ok(confined)
}
