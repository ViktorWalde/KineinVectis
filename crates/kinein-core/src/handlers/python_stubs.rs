//! `python.stubs` (`impl Core`): os stubs do `MicroPython` da placa em
//! `<root>/typings`, como JOB (C4 do `roadmaps/41` bloco C, 2026-09-17).
//!
//! Fino: decide o pacote (pedido > sugestao do modelo), o instalador (`uv` >
//! `pip` do interpretador do projeto), sobe o job; o nome e a linha sao do
//! [`crate::python::stubs`]. Ao fim, `event.python.stubs` — e o
//! `observe_notification` reconfigura o basedpyright com o `stubPath`.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, PythonStubsEvent, PythonStubsParams,
    PythonStubsResult,
};
use serde_json::{Value, json};

use crate::jobs::JobOutcome;
use crate::python::stubs::{self, Installer};
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};
use crate::{Core, python};

impl Core {
    /// `python.stubs` — exige workspace (os stubs sao do projeto). Sem
    /// `port`, o modelo do projeto sugere; sem sugestao, recusa dizendo
    /// o que fixar (o chip no kit, ou o `port` no pedido).
    pub(crate) fn python_stubs_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<PythonStubsParams>(
            request_id.as_ref(),
            params,
            "python.stubs aceita os campos opcionais port e board",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "python.stubs");
        };
        let pedido_port = pedido
            .port
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| !p.is_empty());
        let (port, board) = if let Some(port) = pedido_port {
            (port, pedido.board)
        } else {
            let modelo = self.compute_project_model(&root);
            let Some((port, sugerida)) = stubs::suggest(
                modelo.target.chip.as_deref(),
                modelo.target.family.as_deref(),
            ) else {
                return JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(
                        JsonRpcErrorCode::InvalidRequest,
                        "o modelo do projeto nao diz a placa: identifique-a pelo canal ou fixe o \
                         chip no kit — ou peca `port` (esp32, rp2, stm32, nrf) e `board` no pedido",
                        Some(json!({ "method": "python.stubs" })),
                    ),
                );
            };
            (port, pedido.board.or(sugerida))
        };
        let package = stubs::package(&port, board.as_deref());
        let Some(instalador) = self
            .detector
            .find_in_path("uv")
            .map(Installer::Uv)
            .or_else(|| {
                python::env::python_env(&root, &self.python_tools())
                    .map(|e| Installer::Pip(PathBuf::from(e.interpreter)))
            })
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::ToolNotFound,
                    "nem `uv` nem um interpretador Python nesta maquina para instalar os stubs \
                     (o painel de instalacao mostra o passo do uv)",
                    Some(json!({ "tool": "uv" })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "python.stubs");
        };
        let target = root.join(stubs::PASTA);
        let (programa, args) = instalador.command_line(&package, &target);
        let command = format!("{} {}", programa.display(), args.join(" "));
        let titulo = format!("Instalar stubs {package}");
        let (pacote, linha, alvo) = (package.clone(), command.clone(), target.clone());
        let job_id = jobs.spawn("python", titulo, JobRisk::Medium, true, move |ctx| {
            instalar(ctx, &programa, &args, &pacote, &linha, &alvo)
        });
        JsonRpcResponse::success(
            request_id,
            json!(PythonStubsResult {
                job_id,
                package,
                command,
                target: target.display().to_string(),
            }),
        )
    }

    /// `event.python.stubs` com sucesso: o basedpyright recebe o `stubPath`
    /// e, se esta' vivo, reinicia.
    pub(crate) fn on_python_stubs_finished(&mut self, success: bool) {
        self.on_python_environment_finished(success);
    }
}

/// Roda o instalador com a saida na tela e fecha com o evento; sucesso e'
/// exit 0 E um `.pyi` na pasta.
fn instalar(
    ctx: &crate::jobs::JobContext,
    programa: &Path,
    args: &[String],
    package: &str,
    command: &str,
    target: &Path,
) -> JobOutcome {
    let cancel = ctx.cancellation();
    ctx.emit_output(&format!("$ {command}"));
    let mut comando = std::process::Command::new(programa);
    comando.args(args);
    let mut on_line = |_origem: &'static str, linha: String| ctx.emit_output(&linha);
    let ok = match crate::process::stream_command_lines_cancelable(comando, &cancel, &mut on_line) {
        Ok(status) => status.success(),
        Err(erro) => {
            ctx.emit_output(&match erro {
                crate::process::ProcessError::Spawn(e) => format!("nao foi possivel iniciar: {e}"),
                crate::process::ProcessError::Wait(e) => format!("falha aguardando: {e}"),
            });
            false
        }
    };
    let success = ok
        && target
            .parent()
            .is_some_and(|raiz| stubs::installed(raiz).is_some());
    if ok && !success {
        ctx.emit_output(&format!(
            "o instalador terminou sem erro, mas {} nao tem nenhum .pyi",
            target.display()
        ));
    }
    if success {
        ctx.emit_output(&format!(
            "stubs em {} — o basedpyright recebe a pasta como stubPath",
            target.display()
        ));
    }
    ctx.emit_event(
        "event.python.stubs",
        json!(PythonStubsEvent {
            job_id: ctx.id().to_owned(),
            success,
            package: package.to_owned(),
            command: command.to_owned(),
            target: target.display().to_string(),
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}
