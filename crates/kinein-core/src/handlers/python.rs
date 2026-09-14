//! Handlers for `python.*` requests (`impl Core`) — o AMBIENTE do projeto
//! Python (bloco B do `roadmaps/41`, cadeia iniciada em 2026-09-12).
//!
//! Fino: valida params, chama o dominio e formata a resposta. O interpretador
//! e' o mesmo que o `index.context` resolve (`python::env`), e as ferramentas
//! sao as DETECTADAS — nos testes o PATH e' vazio e nada da maquina entra.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    JobRisk, JsonRpcResponse, PythonCreateEnvironmentParams, PythonEnvironmentTool,
    PythonFinishedEvent, PythonStatusParams,
};
use serde_json::{Value, json};

use crate::Core;
use crate::jobs::JobOutcome;
use crate::python::{self, CriadoresDeAmbiente, PASTA_DO_AMBIENTE};
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

/// A chave do `ruff server` como companheiro do servidor `python`.
pub(crate) const RUFF_COMPANION: &str = "python-ruff";

impl Core {
    /// O basedpyright sobe COM o interpretador do projeto (fatia 2 da cadeia,
    /// 2026-09-12): `python.pythonPath` em `settings` — e' o `--query-driver`
    /// do Python. Sem isto ele indexa a stdlib do Python que achar no PATH e o
    /// completar mente. O executavel e' o DETECTADO (`basedpyright-langserver`
    /// do pipx/uv em ~/.local/bin); ausente, fica o nome nu e a subida falha
    /// dizendo qual. Vale na PROXIMA subida do servidor python.
    pub(crate) fn configure_python_lsp(&mut self, root: &Path) {
        let interpretador =
            crate::python::env::python_env(root, &self.python_tools()).map(|e| e.interpreter);
        let settings = interpretador.map_or(Value::Null, |caminho| {
            json!({
                "python": {
                    "pythonPath": caminho,
                    "analysis": { "autoSearchPaths": true, "diagnosticMode": "openFilesOnly" }
                },
                "basedpyright": {
                    "analysis": { "autoSearchPaths": true, "diagnosticMode": "openFilesOnly" }
                }
            })
        });
        let comando = self
            .detector
            .find_in_path("basedpyright-langserver")
            .map_or_else(
                || "basedpyright-langserver".to_owned(),
                |p| p.display().to_string(),
            );
        // O ruff como SERVIDOR ao lado do basedpyright (40 §4, 2026-09-13):
        // companheiro da linguagem — mesmo texto, diagnosticos fundidos, as
        // correcoes dele (F401, I001, `noqa`, fix all) no Alt+Enter. So' entra
        // quando o binario existe: sem ruff nao ha' companheiro, e nada falha.
        let ruff = self.detector.find_in_path("ruff");
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.use_server_command("python", &comando, &["--stdio"]);
            lsp.use_server_settings("python", settings);
            match ruff {
                Some(caminho) => {
                    lsp.use_companion(
                        "python",
                        RUFF_COMPANION,
                        &caminho.display().to_string(),
                        &["server"],
                    );
                }
                None => {
                    lsp.disable_companion(RUFF_COMPANION);
                }
            }
        }
    }

    /// O ambiente mudou (`event.python.finished` com sucesso): o basedpyright
    /// precisa subir de novo com o interpretador novo — reconfigura e, se ele
    /// esta' vivo, reinicia (a UI reabre os documentos no `event.lsp.restarted`).
    pub(crate) fn on_python_environment_finished(&mut self, success: bool) {
        if !success {
            return;
        }
        let Some(root) = self.workspace_root() else {
            return;
        };
        self.configure_python_lsp(&root);
        if let Some(lsp) = self.lsp.as_mut() {
            if lsp.is_running("python") {
                lsp.restart_language("python");
            }
        }
    }

    /// Roteia os metodos `python.*`.
    pub(crate) fn python_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "python.status" => Some(self.python_status_response(request_id, params)),
            "python.createEnvironment" => {
                Some(self.python_create_environment_response(request_id, params))
            }
            _ => None,
        }
    }

    fn criadores_de_ambiente(&self) -> CriadoresDeAmbiente {
        CriadoresDeAmbiente {
            uv: self.detector.find_in_path("uv"),
        }
    }

    /// `python.status` — o interpretador do projeto, se e' ambiente proprio,
    /// com que ferramenta a IDE criaria um, e o que falta.
    fn python_status_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<PythonStatusParams>(
            request_id.as_ref(),
            params,
            "python.status nao aceita parametros",
        ) {
            return *response;
        }
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "python.status");
        };
        let status = python::status(&root, &self.python_tools(), &self.criadores_de_ambiente());
        JsonRpcResponse::success(request_id, json!(status))
    }

    /// `python.createEnvironment` — `uv venv .venv` (ou `python3 -m venv
    /// .venv`) como JOB, com cada linha na tela. Ao fim, `event.python.finished`
    /// — e o `observe_notification` recarrega o contexto: o interpretador do
    /// projeto passa a ser o do ambiente novo.
    fn python_create_environment_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let pedido = match parse_params::<PythonCreateEnvironmentParams>(
            request_id.as_ref(),
            params,
            "python.createEnvironment aceita apenas o campo opcional tool (uv|venv)",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "python.createEnvironment");
        };
        let criadores = self.criadores_de_ambiente();
        let python_tools = self.python_tools();
        let padrao = if criadores.uv.is_some() {
            PythonEnvironmentTool::Uv
        } else {
            PythonEnvironmentTool::Venv
        };
        let tool = pedido.tool.unwrap_or(padrao);
        let Some((command, rotulo)) = python::create_environment_command(
            &root,
            tool,
            &criadores,
            python_tools.python_sistema.as_deref(),
        ) else {
            return JsonRpcResponse::failure(
                request_id,
                kinein_protocol::JsonRpcError::new(
                    kinein_protocol::JsonRpcErrorCode::InvalidParams,
                    match tool {
                        PythonEnvironmentTool::Uv => {
                            "o uv nao foi detectado nesta maquina (o painel de instalacao mostra o \
                             passo oficial); ou peca tool = venv"
                        }
                        PythonEnvironmentTool::Venv => {
                            "nenhum python3 detectado nesta maquina para criar o ambiente"
                        }
                    },
                    Some(json!({ "method": "python.createEnvironment", "tool": tool })),
                ),
            );
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "python.createEnvironment");
        };
        let destino: PathBuf = root.join(PASTA_DO_AMBIENTE);
        let job_id = jobs.spawn(
            "python",
            format!("Criar ambiente Python ({rotulo})"),
            JobRisk::Medium,
            true,
            move |ctx| criar_ambiente(ctx, command, tool, &rotulo, &destino),
        );
        JsonRpcResponse::success(request_id, json!({ "jobId": job_id }))
    }
}

/// Roda o criador com a saida na tela e fecha com o evento.
fn criar_ambiente(
    ctx: &crate::jobs::JobContext,
    command: std::process::Command,
    tool: PythonEnvironmentTool,
    rotulo: &str,
    destino: &Path,
) -> JobOutcome {
    let cancel = ctx.cancellation();
    ctx.emit_output(&format!("$ {rotulo}"));
    let mut on_line = |_origem: &'static str, linha: String| ctx.emit_output(&linha);
    let ok = match crate::process::stream_command_lines_cancelable(command, &cancel, &mut on_line) {
        Ok(status) => status.success(),
        Err(erro) => {
            ctx.emit_output(&match erro {
                crate::process::ProcessError::Spawn(e) => format!("nao foi possivel iniciar: {e}"),
                crate::process::ProcessError::Wait(e) => format!("falha aguardando: {e}"),
            });
            false
        }
    };
    let success = ok && destino.join("bin").join("python").is_file();
    if ok && !success {
        ctx.emit_output(&format!(
            "a ferramenta terminou sem erro, mas {} nao existe",
            destino.join("bin/python").display()
        ));
    }
    ctx.emit_event(
        "event.python.finished",
        json!(PythonFinishedEvent {
            job_id: ctx.id().to_owned(),
            success,
            tool,
            command: rotulo.to_owned(),
            path: destino.display().to_string(),
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}
