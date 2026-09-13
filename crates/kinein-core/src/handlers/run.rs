//! Handlers for `run.*` requests (`impl Core`).
//!
//! The `run.*` router plus start/stdin/stop, driving the user process managed
//! by `crate::run`.

use std::{ffi::OsStr, path::Path};

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RunScriptParams, RunStartParams,
    RunStartResult, RunStdinParams,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, no_workspace_response, parse_params, run_error_response,
    run_unavailable_response,
};
use crate::{Core, run};

impl Core {
    /// Roteia os metodos `run.*`; `None` quando o metodo nao e de execucao.
    pub(crate) fn run_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "run.start" => Some(self.run_start_response(request_id, params)),
            "run.script" => Some(self.run_script_response(request_id, params)),
            "run.stdin" => Some(self.run_stdin_response(request_id, params)),
            "run.stop" => Some(self.run_stop_response(request_id)),
            _ => None,
        }
    }

    fn run_script_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "run.script");
        };
        let parsed = match parse_params::<RunScriptParams>(
            request_id.as_ref(),
            params,
            "run.script requer apenas o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = Path::new(&workspace.root).to_path_buf();
        let script = match crate::fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(script) => script,
            Err(error) => return fs_error_response(request_id, &error),
        };
        // Um .py roda com o Python DO PROJETO (ou `uv run`), nao com um
        // interpretador fixo: e' a fatia 3 da cadeia Python (41 bloco B).
        if script.extension().and_then(OsStr::to_str) == Some("py") {
            return self.run_python_script_response(request_id, &root, &script);
        }
        let Some(interpreter) = run::script_interpreter(&script) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "run.script aceita scripts .sh, .bash, .zsh ou .py",
                    Some(json!({ "path": parsed.path })),
                ),
            );
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.script");
        };
        let command = run::script_display_command(&root, interpreter, &script);
        let args = [OsStr::new("--"), script.as_os_str()];
        match runner.start_program(&root, interpreter, &args, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    /// O lancador Python deste workspace: `uv run` se o projeto e' do uv e o
    /// uv existe; senao o interpretador da precedencia do 29 §4.1. Sem medir
    /// a versao — executar nao precisa dela.
    pub(crate) fn python_launcher(
        &self,
        root: &Path,
    ) -> Option<crate::python::run::PythonLauncher> {
        let mut tools = self.python_tools();
        tools.medir_versao = false;
        crate::python::run::PythonLauncher::resolve(root, &tools, self.detector.find_in_path("uv"))
    }

    fn run_python_script_response(
        &mut self,
        request_id: Option<Value>,
        root: &Path,
        script: &Path,
    ) -> JsonRpcResponse {
        let Some(launcher) = self.python_launcher(root) else {
            return run_error_response(
                request_id,
                &run::RunError::NoDefaultCommand {
                    message: "sem interpretador Python para este workspace: crie o ambiente \
                              (.venv) pela faixa de saude ou instale o python3"
                        .to_owned(),
                },
            );
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.script");
        };
        let (program, prefix) = launcher.program();
        let mut args: Vec<&OsStr> = prefix.iter().map(std::ffi::OsString::as_os_str).collect();
        args.push(script.as_os_str());
        let command = launcher.script_display(root, script);
        match runner.start_program(root, &program.display().to_string(), &args, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_start_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "run.start");
        };
        let parsed = match parse_params::<RunStartParams>(
            request_id.as_ref(),
            params,
            "run.start aceita apenas o campo opcional command",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if self.run.is_none() {
            return run_unavailable_response(request_id, "run.start");
        }

        let root = Path::new(&workspace.root);
        let explicito = parsed.command.filter(|command| !command.trim().is_empty());
        let command =
            if let Some(command) = explicito.or_else(|| crate::runconfig::active_command(root)) {
                command
            } else {
                // Python nao passa por run::default_command: precisa do lancador do
                // projeto (interpretador ou `uv run`).
                let padrao = if workspace.kind == kinein_protocol::ProjectKind::Python {
                    let launcher = self.python_launcher(root);
                    crate::python::run::default_command(root, launcher.as_ref())
                } else {
                    run::default_command(workspace.kind, root)
                };
                match padrao {
                    Ok(command) => command,
                    Err(error) => return run_error_response(request_id, &error),
                }
            };

        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.start");
        };
        match runner.start(root, &command) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(RunStartResult { command })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stdin_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RunStdinParams>(
            request_id.as_ref(),
            params,
            "run.stdin requer o campo data",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stdin");
        };
        match runner.write_stdin(&parsed.data) {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }

    fn run_stop_response(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(runner) = self.run.as_mut() else {
            return run_unavailable_response(request_id, "run.stop");
        };
        match runner.stop() {
            Ok(()) => JsonRpcResponse::success(request_id, json!({ "status": "ok" })),
            Err(error) => run_error_response(request_id, &error),
        }
    }
}
