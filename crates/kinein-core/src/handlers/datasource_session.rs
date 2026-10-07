//! Desconectar revoga permissoes e espera leases; nao remove perfil nem dados.
//! A espera pertence ao job, nunca ao laco de despacho.

use std::path::Path;

use kinein_protocol::{
    DataSourceDisconnectAccepted, DataSourceDisconnectParams, DataSourceDisconnectedEvent, JobRisk,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::datasource::{activity, policy};
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    pub(super) fn begin_datasource_operation(
        &self,
        root: &Path,
        name: &str,
        token: Option<&str>,
        request_id: Option<Value>,
    ) -> Result<activity::Operation, Box<JsonRpcResponse>> {
        self.datasource_activity
            .begin(root, name)
            .map_err(|message| {
                Box::new(
                    policy::Rejection {
                        code: JsonRpcErrorCode::InvalidRequest,
                        message,
                    }
                    .response(request_id, name, token),
                )
            })
    }

    pub(super) fn datasource_disconnect_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.disconnect");
        };
        let request = match parse_params::<DataSourceDisconnectParams>(
            request_id.as_ref(),
            params,
            "datasource.disconnect exige name, expectedContext e clientContext",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return super::datasource::com_id(*response, request_id),
        };
        if let Err(rejection) = policy::check_context(
            &root,
            &profile,
            Some(&request.expected_context),
            Some(&request.client_context),
        ) {
            return rejection.response(request_id, &request.name, Some(&request.client_context));
        }
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.disconnect");
        };
        let closing = match self.datasource_activity.disconnect(&root, &request.name) {
            Ok(closing) => closing,
            Err(message) => {
                return policy::Rejection {
                    code: JsonRpcErrorCode::InvalidRequest,
                    message,
                }
                .response(request_id, &request.name, Some(&request.client_context));
            }
        };
        self.odbc.revoke(&root, &request.name);
        self.previews.revoke(&root, &request.name);
        let name = request.name.clone();
        let token = request.client_context.clone();
        let job_id = jobs.spawn(
            "datasource",
            format!("Desconectar {name}"),
            JobRisk::Low,
            false,
            move |ctx| {
                let result = closing.wait();
                drop(closing);
                let success = result.is_ok();
                let message = result.map_or_else(str::to_owned, |()| {
                    "Conexão encerrada; perfil e dados preservados.".to_owned()
                });
                ctx.emit_output(&message);
                ctx.emit_event(
                    "event.datasource.disconnected",
                    json!(DataSourceDisconnectedEvent {
                        job_id: ctx.id().to_owned(),
                        name: request.name,
                        client_context: request.client_context,
                        success,
                        message,
                    }),
                );
                if success {
                    crate::jobs::JobOutcome::Success
                } else {
                    crate::jobs::JobOutcome::Failed
                }
            },
        );
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceDisconnectAccepted {
                job_id,
                name,
                client_context: token
            }),
        )
    }
}
