//! One-level remote directory browsing for the existing SSH mirror flow.
//! The request only starts a Job: a slow SSH target never stalls IPC.

use std::{path::Path, process::Command};

use kinein_protocol::{
    JobRisk, JsonRpcResponse, RemoteDirectoriesEvent, RemoteDirectoriesParams,
    RemoteDirectoryEntry, RemoteJobResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::jobs::JobOutcome;
use crate::process::stream_command_lines_cancelable;
use crate::remote;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

use super::remote::{error_response, target_or_error};

// The remote target is Linux (the contract of this domain). `find -print0`
// preserves names containing newlines; `head` caps the data transferred to
// this process. Names with control characters are omitted, by the same rule
// remote.open applies (`remote::valid_remote_dir`); spaces are kept.
const LIST_SCRIPT: &str = r#"p=${1:-$HOME}; case "$p" in /*) ;; *) exit 3;; esac; [ -d "$p" ] && [ -r "$p" ] && [ -x "$p" ] || exit 4; printf '%s\0' "$p"; find "$p" -mindepth 1 -maxdepth 1 -type d -print0 | head -c 65536"#;

fn valid_path(path: &str) -> bool {
    remote::valid_remote_dir(path)
}

fn parse_listing(raw: &str) -> Result<(String, Option<String>, Vec<RemoteDirectoryEntry>), String> {
    let mut parts = raw.split('\0');
    let path = parts.next().unwrap_or_default();
    if !valid_path(path) {
        return Err("o alvo não devolveu uma pasta absoluta utilizável".to_owned());
    }
    let prefix = if path == "/" {
        "/".to_owned()
    } else {
        format!("{path}/")
    };
    let mut entries: Vec<_> = parts
        .filter(|entry| entry.starts_with(&prefix) && valid_path(entry))
        .filter_map(|entry| {
            let basename = entry.strip_prefix(&prefix)?;
            (!basename.is_empty() && !basename.contains('/')).then(|| RemoteDirectoryEntry {
                name: basename.to_owned(),
                path: entry.to_owned(),
            })
        })
        .collect();
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    entries.dedup_by(|a, b| a.path == b.path);
    let parent = Path::new(path)
        .parent()
        .filter(|parent| parent != &Path::new(path))
        .map(|parent| parent.display().to_string());
    Ok((path.to_owned(), parent, entries))
}

impl Core {
    /// Roteia o navegador da pasta remota, sem segundo cliente SSH.
    pub(crate) fn remote_directories_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "remote.directories" => Some(self.remote_directories_response(request_id, params)),
            _ => None,
        }
    }

    fn remote_directories_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteDirectoriesParams>(
            request_id.as_ref(),
            params,
            "remote.directories requer name e aceita path absoluto",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.directories");
        };
        let target = match target_or_error(&root, request_id.as_ref(), &parsed.name) {
            Ok(target) => target,
            Err(response) => return *response,
        };
        let path = parsed.path.unwrap_or_default();
        if !path.is_empty() && !valid_path(&path) {
            return error_response(
                request_id,
                "a pasta remota precisa ser absoluta e sem caracteres de controle",
            );
        }
        let Some(ssh) = self.detector.find_in_path("ssh") else {
            return error_response(
                request_id,
                "não achei `ssh` no PATH — instale o openssh-client",
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "remote.directories");
        };
        let mut args = remote::ssh_args(&target, true);
        let remote_line = format!(
            "sh -c {} sh {}",
            remote::quote(LIST_SCRIPT),
            remote::quote(&path)
        );
        args.push(remote_line);
        let command = format!("{} {}", ssh.display(), args.join(" "));
        let name = target.name.clone();
        let requested_path = path;
        let title = format!("Listar pastas de {name}");
        let job_id = jobs.spawn(
            "remote.directories",
            &title,
            JobRisk::Low,
            true,
            move |ctx| {
                let mut process = Command::new(&ssh);
                process.args(&args);
                let mut stdout = String::new();
                let mut stderr = String::new();
                let mut on_line = |stream: &'static str, line: String| {
                    let buffer = if stream == "stdout" {
                        &mut stdout
                    } else {
                        &mut stderr
                    };
                    buffer.push_str(&line);
                    buffer.push('\n');
                };
                let status =
                    stream_command_lines_cancelable(process, &ctx.cancellation(), &mut on_line);
                let success = matches!(&status, Ok(exit) if exit.success());
                let parsed = if success {
                    parse_listing(&stdout)
                } else {
                    Err(remote::describe_ssh_failure(&target, &stderr))
                };
                let (path, parent, entries, error) = match parsed {
                    Ok((path, parent, entries)) => (path, parent, entries, None),
                    Err(error) => (String::new(), None, Vec::new(), Some(error)),
                };
                let ok = error.is_none();
                ctx.emit_event(
                    "event.remote.directories",
                    json!(RemoteDirectoriesEvent {
                        job_id: ctx.id().to_owned(),
                        name,
                        requested_path,
                        success: ok,
                        path,
                        parent,
                        entries,
                        error,
                    }),
                );
                if ok {
                    JobOutcome::Success
                } else {
                    JobOutcome::Failed
                }
            },
        );
        JsonRpcResponse::success(request_id, json!(RemoteJobResult { job_id, command }))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_listing;

    #[test]
    fn listing_keeps_only_direct_usable_children() {
        let (path, parent, entries) = parse_listing(
            "/home/pi\0/home/pi/sensor\0/home/pi/with space\0/home/pi/nested/child\0/home/pi/b\nline\0",
        ).unwrap();
        assert_eq!(path, "/home/pi");
        assert_eq!(parent.as_deref(), Some("/home"));
        // Espaco fica (o espelho o aceita); quebra de linha e neto, nao.
        let paths: Vec<_> = entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["/home/pi/sensor", "/home/pi/with space"]);
    }
}
