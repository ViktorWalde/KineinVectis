//! `remote.open` / `remote.sync` / `remote.status` (`impl Core`) — o
//! workspace ESPELHADO (P6 fatia 2 do `roadmaps/42`, 2026-09-18) — e o
//! empurrao automatico depois de um `fs.write` num espelho.
//!
//! Tudo que move bytes e' JOB (`rsync` do sistema); `remote.status` e'
//! sincrono (le o marcador). A UI abre o espelho com o `workspace.open` de
//! sempre quando o `event.remote.synced { direction: pull }` chega.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{
    JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RemoteMirror, RemoteOpenParams,
    RemoteOpenResult, RemoteStatusParams, RemoteStatusResult, RemoteSyncDirection,
    RemoteSyncParams, RemoteSyncedEvent, RemoteTarget,
};
use serde_json::{Value, json};

use crate::Core;
use crate::jobs::{JobContext, JobOutcome};
use crate::process::stream_command_lines_cancelable;
use crate::remote::{self, mirror};
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

/// O que um job de sincronia precisa saber; nasce no handler, roda no job.
struct SyncPlan {
    rsync: PathBuf,
    target: RemoteTarget,
    remote_path: String,
    mirror: PathBuf,
    rels: Vec<String>,
    direction: RemoteSyncDirection,
    transport: String,
}

impl SyncPlan {
    fn commands(&self) -> Vec<Vec<String>> {
        self.rels
            .iter()
            .map(|rel| {
                mirror::rsync_args(
                    &self.target,
                    &self.remote_path,
                    &self.mirror,
                    rel,
                    self.direction,
                    &self.transport,
                )
            })
            .collect()
    }

    fn command_line(&self) -> String {
        self.commands()
            .iter()
            .map(|args| format!("{} {}", self.rsync.display(), args.join(" ")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Roda cada `rsync`; devolve os caminhos transferidos ou o erro.
    fn run(&self, ctx: &JobContext) -> Result<Vec<String>, String> {
        let mut changed = Vec::new();
        for args in self.commands() {
            ctx.emit_output(&format!("$ {} {}", self.rsync.display(), args.join(" ")));
            let mut command = Command::new(&self.rsync);
            command.args(&args);
            let mut saida = String::new();
            let mut on_line = |_: &'static str, line: String| {
                ctx.emit_output(&line);
                saida.push_str(&line);
                saida.push('\n');
            };
            let status =
                stream_command_lines_cancelable(command, &ctx.cancellation(), &mut on_line);
            match status {
                Ok(s) if s.success() => changed.extend(mirror::parse_itemized(&saida)),
                Ok(_) => return Err(remote::describe_ssh_failure(&self.target, &saida)),
                Err(crate::process::ProcessError::Spawn(e)) => {
                    return Err(format!("nao pude iniciar o rsync: {e}"));
                }
                Err(crate::process::ProcessError::Wait(e)) => {
                    return Err(format!("falha esperando o rsync: {e}"));
                }
            }
        }
        Ok(changed)
    }
}

fn falha(request_id: Option<Value>, mensagem: impl Into<String>) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, mensagem, None),
    )
}

impl Core {
    /// Roteia `remote.open`, `remote.sync` e `remote.status`.
    pub(crate) fn remote_mirror_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "remote.open" => Some(self.remote_open_response(request_id, params)),
            "remote.sync" => Some(self.remote_sync_response(request_id, params)),
            "remote.status" => Some(self.remote_status_response(request_id, params)),
            _ => None,
        }
    }

    /// O espelho que o workspace aberto e', se for um.
    pub(crate) fn current_mirror(&self) -> Option<RemoteMirror> {
        self.workspace_root()
            .and_then(|root| mirror::read_marker(&root))
    }

    fn rsync_and_transport(&self, target: &RemoteTarget) -> Result<(PathBuf, String), String> {
        let rsync = self
            .detector
            .find_in_path("rsync")
            .ok_or_else(|| "nao achei `rsync` no PATH — instale-o (e no alvo tambem)".to_owned())?;
        let home = self
            .sdk_home()
            .ok_or_else(|| "sem HOME: nao sei onde guardar o espelho".to_owned())?;
        let control = mirror::cache_root(&home);
        std::fs::create_dir_all(&control)
            .map_err(|e| format!("falha criando {}: {e}", control.display()))?;
        Ok((rsync, mirror::ssh_transport(target, &control)))
    }

    /// `remote.open { name, path }` -> `{ jobId, command, mirror }`; o job puxa
    /// a arvore, grava o marcador e emite `event.remote.synced { pull }`.
    fn remote_open_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteOpenParams>(
            request_id.as_ref(),
            params,
            "remote.open requer name e path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        // O catalogo e' do workspace ABERTO (e' de la' que o autor escolhe o
        // alvo); o espelho nasce fora dele, no cache.
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.open");
        };
        let Some(target) = remote::find(&root, &parsed.name) else {
            return falha(
                request_id,
                format!("nao ha' alvo remoto chamado `{}`", parsed.name),
            );
        };
        let remote_path = parsed.path.trim().trim_end_matches('/').to_owned();
        if remote_path.is_empty() || remote_path.contains(char::is_whitespace) {
            return falha(
                request_id,
                "informe a pasta no alvo (ex.: /home/pi/projeto), sem espacos",
            );
        }
        let (rsync, transport) = match self.rsync_and_transport(&target) {
            Ok(x) => x,
            Err(m) => return falha(request_id, m),
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "remote.open");
        };
        let home = self.sdk_home().unwrap_or_default();
        let espelho = mirror::mirror_root(&home, &target, &remote_path);
        let plan = SyncPlan {
            rsync,
            target,
            remote_path,
            mirror: espelho.clone(),
            rels: vec![String::new()],
            direction: RemoteSyncDirection::Pull,
            transport,
        };
        let command = plan.command_line();
        let titulo = format!("Espelhar {}:{}", plan.target.name, plan.remote_path);
        let job_id = jobs.spawn("remote.sync", &titulo, JobRisk::Low, true, move |ctx| {
            let resultado = std::fs::create_dir_all(&plan.mirror)
                .map_err(|e| format!("falha criando {}: {e}", plan.mirror.display()))
                .and_then(|()| plan.run(ctx))
                .and_then(|changed| {
                    mirror::write_marker(&plan.mirror, &plan.target, &plan.remote_path)?;
                    // O espelho e' autossuficiente: o alvo (usuario, porta,
                    // chave) vai para o catalogo DELE — o `.kinein` nao
                    // sincroniza, entao nada disto chega ao alvo.
                    remote::save(&plan.mirror, &plan.target)?;
                    Ok(changed)
                });
            emitir_synced(ctx, &plan, resultado)
        });
        JsonRpcResponse::success(
            request_id,
            json!(RemoteOpenResult {
                job_id,
                command,
                mirror: espelho.display().to_string()
            }),
        )
    }

    /// `remote.sync { direction, paths? }` no espelho aberto.
    fn remote_sync_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteSyncParams>(
            request_id.as_ref(),
            params,
            "remote.sync requer direction (pull | push) e aceita paths",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.sync");
        };
        let Some(espelho) = mirror::read_marker(&root) else {
            return falha(
                request_id,
                "este workspace nao e' um espelho remoto — abra uma pasta do alvo pelo painel Remoto",
            );
        };
        let rels: Vec<String> = match parsed.paths {
            None => vec![String::new()],
            Some(paths) => {
                let rels: Vec<String> = paths
                    .iter()
                    .map(|p| p.trim_matches('/').to_owned())
                    .collect();
                if rels.is_empty() || rels.iter().any(|r| !mirror::safe_relative(r)) {
                    return falha(
                        request_id,
                        "paths devem ser relativos ao espelho, sem `..`, e nunca `.kinein`",
                    );
                }
                rels
            }
        };
        match self.spawn_sync(&root, &espelho, rels, parsed.direction, "remote.sync") {
            Ok((job_id, command)) => {
                JsonRpcResponse::success(request_id, json!({ "jobId": job_id, "command": command }))
            }
            Err(mensagem) => falha(request_id, mensagem),
        }
    }

    fn remote_status_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RemoteStatusParams>(
            request_id.as_ref(),
            params,
            "remote.status nao aceita parametros",
        ) {
            return *response;
        }
        JsonRpcResponse::success(
            request_id,
            json!(RemoteStatusResult {
                mirror: self.current_mirror()
            }),
        )
    }

    /// Depois de um `fs.write`/`fs.createFile` num espelho: empurra so' esse
    /// arquivo, como job. Sem espelho, nada. Silencioso em falha de
    /// preparacao (o evento do job diz o resto).
    pub(crate) fn mirror_push_after_write(&self, path: &Path) {
        let Some(root) = self.workspace_root() else {
            return;
        };
        let Some(espelho) = mirror::read_marker(&root) else {
            return;
        };
        let Ok(rel) = path.strip_prefix(&root) else {
            return;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        if !mirror::safe_relative(&rel) {
            return;
        }
        let _ = self.spawn_sync(
            &root,
            &espelho,
            vec![rel],
            RemoteSyncDirection::Push,
            "remote.autopush",
        );
    }

    fn spawn_sync(
        &self,
        root: &Path,
        espelho: &RemoteMirror,
        rels: Vec<String>,
        direction: RemoteSyncDirection,
        kind: &str,
    ) -> Result<(String, String), String> {
        // O alvo vem do catalogo do espelho (o `.kinein/remotes.json` que
        // NAO sincroniza) ou, na falta, do proprio marcador — sem chave/porta.
        let target = remote::find(root, &espelho.name).unwrap_or_else(|| RemoteTarget {
            name: espelho.name.clone(),
            host: espelho.host.clone(),
            user: None,
            port: None,
            identity_file: None,
            deploy_dir: None,
        });
        let (rsync, transport) = self.rsync_and_transport(&target)?;
        let jobs = self
            .jobs
            .as_ref()
            .ok_or_else(|| "jobs indisponiveis".to_owned())?;
        let plan = SyncPlan {
            rsync,
            target,
            remote_path: espelho.path.clone(),
            mirror: root.to_path_buf(),
            rels,
            direction,
            transport,
        };
        let command = plan.command_line();
        let titulo = match (direction, plan.rels.first().map(String::as_str)) {
            (RemoteSyncDirection::Push, Some(rel)) if !rel.is_empty() => {
                format!("Empurrar {rel} para {}", plan.target.name)
            }
            (RemoteSyncDirection::Push, _) => format!("Empurrar tudo para {}", plan.target.name),
            (RemoteSyncDirection::Pull, _) => format!("Puxar de {}", plan.target.name),
        };
        let job_id = jobs.spawn(kind, &titulo, JobRisk::Low, true, move |ctx| {
            let resultado = plan.run(ctx);
            emitir_synced(ctx, &plan, resultado)
        });
        Ok((job_id, command))
    }
}

fn emitir_synced(
    ctx: &JobContext,
    plan: &SyncPlan,
    resultado: Result<Vec<String>, String>,
) -> JobOutcome {
    let (success, changed, error) = match resultado {
        Ok(changed) => {
            ctx.emit_output(&format!("{} caminho(s) transferido(s)", changed.len()));
            (true, changed, None)
        }
        Err(erro) => {
            ctx.emit_output(&erro);
            (false, Vec::new(), Some(erro))
        }
    };
    ctx.emit_event(
        "event.remote.synced",
        json!(RemoteSyncedEvent {
            job_id: ctx.id().to_owned(),
            name: plan.target.name.clone(),
            direction: plan.direction,
            success,
            command: plan.command_line(),
            changed,
            error,
            mirror: plan.mirror.display().to_string(),
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}
