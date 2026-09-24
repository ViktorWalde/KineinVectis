//! Handlers de `remote.*` (`impl Core`) — P6 fatia 1 do `roadmaps/42`,
//! 2026-09-17.
//!
//! Catalogo sincrono (`remote.list/save/remove`, sem segredo em disco);
//! `remote.probe` e `remote.deploy` como JOBS (a rede espera; o laco de
//! despacho nao); `remote.command` PURO — compoe a linha `ssh …` que a UI
//! grava como configuracao de execucao ou no kit, nada roda aqui.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{
    JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RemoteDeployParams,
    RemoteDeployedEvent, RemoteJobResult, RemoteListParams, RemoteListResult, RemoteNameParams,
    RemoteProbedEvent, RemoteSaveParams, RemoteTarget,
};
use serde_json::{Value, json};

use crate::Core;
use crate::jobs::JobOutcome;
use crate::process::stream_command_lines_cancelable;
use crate::remote;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

fn descrever_processo(erro: &crate::process::ProcessError) -> String {
    match erro {
        crate::process::ProcessError::Spawn(e) => format!("nao pude iniciar o processo: {e}"),
        crate::process::ProcessError::Wait(e) => format!("falha esperando o processo: {e}"),
    }
}

pub(super) fn falha(request_id: Option<Value>, mensagem: impl Into<String>) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, mensagem, None),
    )
}

pub(super) fn alvo_ou_falha(
    root: &Path,
    request_id: Option<&Value>,
    name: &str,
) -> Result<RemoteTarget, Box<JsonRpcResponse>> {
    remote::find(root, name).ok_or_else(|| {
        Box::new(falha(
            request_id.cloned(),
            format!("nao ha' alvo remoto chamado `{name}` — salve-o no painel Remoto"),
        ))
    })
}

/// O que o deploy vai rodar: origem local, pasta no alvo, programa e se e'
/// `rsync` (senao `scp`).
struct DeployPlan {
    source: PathBuf,
    dest: String,
    program: PathBuf,
    use_rsync: bool,
}

fn nome_do_projeto(root: &Path) -> String {
    root.file_name().map_or_else(
        || "projeto".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    )
}

impl Core {
    /// Roteia `remote.*`.
    pub(crate) fn remote_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "remote.list" => Some(self.remote_list_response(request_id, params)),
            "remote.save" => Some(self.remote_save_response(request_id, params)),
            "remote.remove" => Some(self.remote_remove_response(request_id, params)),
            "remote.probe" => Some(self.remote_probe_response(request_id, params)),
            "remote.deploy" => Some(self.remote_deploy_response(request_id, params)),
            _ => None,
        }
    }

    fn remote_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RemoteListParams>(
            request_id.as_ref(),
            params,
            "remote.list nao aceita parametros",
        ) {
            return *response;
        }
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.list");
        };
        JsonRpcResponse::success(
            request_id,
            json!(RemoteListResult {
                targets: remote::list(&root)
            }),
        )
    }

    fn remote_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteSaveParams>(
            request_id.as_ref(),
            params,
            "remote.save requer target { name, host, user?, port?, identityFile?, deployDir? }",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.save");
        };
        match remote::save(&root, &parsed.target) {
            Ok(targets) => {
                JsonRpcResponse::success(request_id, json!(RemoteListResult { targets }))
            }
            Err(mensagem) => falha(request_id, mensagem),
        }
    }

    fn remote_remove_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteNameParams>(
            request_id.as_ref(),
            params,
            "remote.remove requer name",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.remove");
        };
        match remote::remove(&root, &parsed.name) {
            Ok(targets) => {
                JsonRpcResponse::success(request_id, json!(RemoteListResult { targets }))
            }
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
            ),
        }
    }

    /// `remote.probe { name }` -> `{ jobId, command }`; `event.remote.probed`
    /// no fim. `ssh` em `BatchMode`: sem chave falha em segundos e o evento
    /// diz `ssh-copy-id`.
    fn remote_probe_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteNameParams>(
            request_id.as_ref(),
            params,
            "remote.probe requer name",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.probe");
        };
        let target = match alvo_ou_falha(&root, request_id.as_ref(), &parsed.name) {
            Ok(target) => target,
            Err(response) => return *response,
        };
        let Some(ssh) = self.detector.find_in_path("ssh") else {
            return falha(
                request_id,
                "nao achei `ssh` no PATH — instale o openssh-client",
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "remote.probe");
        };
        let mut args = remote::ssh_args(&target, true);
        args.push(remote::probe_script());
        let command_line = format!("{} {}", ssh.display(), args.join(" "));
        let name = target.name.clone();
        let titulo = format!("Sondar {name}");
        let linha = command_line.clone();
        let job_id = jobs.spawn("remote.probe", &titulo, JobRisk::Low, true, move |ctx| {
            ctx.emit_output(&format!("$ {linha}"));
            let mut command = Command::new(&ssh);
            command.args(&args);
            let mut stdout = String::new();
            let mut stderr = String::new();
            let mut on_line = |stream: &'static str, line: String| {
                ctx.emit_output(&line);
                let alvo = if stream == "stdout" {
                    &mut stdout
                } else {
                    &mut stderr
                };
                alvo.push_str(&line);
                alvo.push('\n');
            };
            let status =
                stream_command_lines_cancelable(command, &ctx.cancellation(), &mut on_line);
            let sucesso = matches!(&status, Ok(s) if s.success());
            let (arch, kernel, tools) = remote::parse_probe(&stdout);
            let error = if sucesso {
                None
            } else {
                Some(match status {
                    Err(erro) => descrever_processo(&erro),
                    Ok(_) => remote::describe_ssh_failure(&target, &stderr),
                })
            };
            // Mesma causa que escolheu a frase, agora tipada: a UI oferece o
            // gesto certo sem ler a sentenca.
            let failure = (!sucesso).then(|| remote::classify_ssh_failure(&stderr));
            if let Some(erro) = &error {
                ctx.emit_output(erro);
            }
            ctx.emit_event(
                "event.remote.probed",
                json!(RemoteProbedEvent {
                    job_id: ctx.id().to_owned(),
                    name,
                    success: sucesso,
                    failure,
                    arch,
                    kernel,
                    tools,
                    error,
                    raw: format!("{stdout}{stderr}"),
                }),
            );
            if sucesso {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });
        JsonRpcResponse::success(
            request_id,
            json!(RemoteJobResult {
                job_id,
                command: command_line
            }),
        )
    }

    /// `remote.deploy { name, source?, dest? }` -> `{ jobId, command }`;
    /// `event.remote.deployed` no fim. `rsync` quando ha' na maquina, senao
    /// `scp -r`; a origem padrao e' o `build/` do projeto.
    fn remote_deploy_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteDeployParams>(
            request_id.as_ref(),
            params,
            "remote.deploy requer name",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.deploy");
        };
        let target = match alvo_ou_falha(&root, request_id.as_ref(), &parsed.name) {
            Ok(target) => target,
            Err(response) => return *response,
        };
        let DeployPlan {
            source,
            dest,
            program: programa,
            use_rsync,
        } = match self.deploy_plan(&root, &target, parsed, request_id.as_ref()) {
            Ok(plan) => plan,
            Err(response) => return *response,
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "remote.deploy");
        };
        let (_, args) = remote::deploy_command(&target, &source, &dest, use_rsync);
        let command_line = format!("{} {}", programa.display(), args.join(" "));
        let name = target.name.clone();
        let titulo = format!("Deploy em {name}");
        let linha = command_line.clone();
        let origem = source.display().to_string();
        let destino = dest;
        let job_id = jobs.spawn(
            "remote.deploy",
            &titulo,
            JobRisk::Medium,
            true,
            move |ctx| {
                ctx.emit_output(&format!("$ {linha}"));
                let mut command = Command::new(&programa);
                command.args(&args);
                let mut saida = String::new();
                let mut on_line = |_: &'static str, line: String| {
                    ctx.emit_output(&line);
                    saida.push_str(&line);
                    saida.push('\n');
                };
                let status =
                    stream_command_lines_cancelable(command, &ctx.cancellation(), &mut on_line);
                let sucesso = matches!(&status, Ok(s) if s.success());
                let error = (!sucesso).then(|| match status {
                    Err(erro) => descrever_processo(&erro),
                    Ok(_) => remote::describe_ssh_failure(&target, &saida),
                });
                ctx.emit_event(
                    "event.remote.deployed",
                    json!(RemoteDeployedEvent {
                        job_id: ctx.id().to_owned(),
                        name,
                        success: sucesso,
                        source: origem,
                        dest: destino,
                        command: linha,
                        error,
                    }),
                );
                if sucesso {
                    JobOutcome::Success
                } else {
                    JobOutcome::Failed
                }
            },
        );
        JsonRpcResponse::success(
            request_id,
            json!(RemoteJobResult {
                job_id,
                command: command_line
            }),
        )
    }

    fn deploy_plan(
        &self,
        root: &Path,
        target: &RemoteTarget,
        parsed: RemoteDeployParams,
        request_id: Option<&Value>,
    ) -> Result<DeployPlan, Box<JsonRpcResponse>> {
        let source = parsed
            .source
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map_or_else(|| root.join("build"), |s| root.join(s));
        if !source.exists() {
            return Err(Box::new(falha(
                request_id.cloned(),
                format!(
                    "a origem `{}` nao existe — compile antes, ou aponte outra pasta",
                    source.display()
                ),
            )));
        }
        let dest = parsed
            .dest
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| remote::deploy_dir(target, &nome_do_projeto(root)));
        let (program, use_rsync) = match self.detector.find_in_path("rsync") {
            Some(rsync) => (rsync, true),
            None => match self.detector.find_in_path("scp") {
                Some(scp) => (scp, false),
                None => {
                    return Err(Box::new(falha(
                        request_id.cloned(),
                        "nem `rsync` nem `scp` no PATH — instale o openssh-client",
                    )));
                }
            },
        };
        Ok(DeployPlan {
            source,
            dest,
            program,
            use_rsync,
        })
    }
}
