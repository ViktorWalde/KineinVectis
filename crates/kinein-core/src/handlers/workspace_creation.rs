//! Criação de projetos e consulta de instalações, sem exigir workspace aberto.
use crate::{
    Core,
    rpc::{parse_params, terminal_error_response, workspace_error_response},
    workspace,
};
use kinein_protocol::{JsonRpcResponse, ProjectEnvironmentsParams, WorkspaceCreateProjectParams};
use serde_json::{Value, json};
use std::path::Path;

impl Core {
    pub(crate) fn project_environments_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<ProjectEnvironmentsParams>(
            request_id.as_ref(),
            params,
            "workspace.projectEnvironments não aceita parâmetros",
        ) {
            return *response;
        }
        let detector = self.detector.clone();
        self.defer_work(request_id, move |request_id| {
            JsonRpcResponse::success(request_id, json!(detector.project_environments()))
        })
    }
    pub(crate) fn create_workspace_project_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = match parse_params::<WorkspaceCreateProjectParams>(
            request_id.as_ref(),
            params,
            "workspace.createProject requer parent, name e template",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match workspace::create_project_with_options(
            Path::new(&params.parent),
            &params.name,
            params.template,
            &params.options,
        ) {
            Ok(mut opened) => {
                // O projeto que a IDE criou nasce confiavel (16.4b).
                opened.trusted = true;
                self.remember_trust(&opened.root);
                if let Err(error) = self.activate_workspace(&opened) {
                    return terminal_error_response(
                        request_id,
                        &crate::terminal::TerminalError::Process {
                            message: format!(
                                "O projeto foi criado em {}, mas não foi possível trocar de projeto: {error}",
                                opened.root
                            ),
                        },
                    );
                }
                JsonRpcResponse::success(request_id, json!(opened))
            }
            Err(error) => workspace_error_response(request_id, &error),
        }
    }
}
