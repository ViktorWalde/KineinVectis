//! Preview preparation and one-use decisions; execution stays in the job worker.

use kinein_protocol::{
    DataSourcePreviewDecideParams, DataSourcePreviewDecision, DataSourceProfile,
    DataSourceQueryParams, DataSourceTestAccepted, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::{policy, preview, preview_sql};
use crate::rpc::{no_workspace_response, parse_params};

pub(super) struct Prepared {
    pub(super) lease: preview::Lease,
    pub(super) sql: String,
}

pub(super) fn validate(
    profile: &DataSourceProfile,
    request: &DataSourceQueryParams,
) -> Result<Option<String>, policy::Rejection> {
    if !request.preview {
        return Ok(None);
    }
    if request.expected_context.is_none() || request.client_context.is_none() {
        return Err(policy::Rejection {
            code: JsonRpcErrorCode::InvalidParams,
            message: "A prévia exige o projeto, perfil e token da operação.",
        });
    }
    let sql = (profile.engine == kinein_protocol::DataSourceEngine::Postgres)
        .then(|| preview_sql::executed_sql(&request.sql)).flatten()
        .ok_or(policy::Rejection { code: JsonRpcErrorCode::DataSourcePreviewUnavailable,
            message: "A prévia exige uma única instrução INSERT, UPDATE ou DELETE elegível no PostgreSQL." })?;
    if !request.confirm_write {
        return Err(policy::Rejection {
            code: JsonRpcErrorCode::WriteConfirmationRequired,
            message: "A prévia executa uma escrita real; confira o impacto antes de continuar.",
        });
    }
    Ok(Some(sql))
}

impl Core {
    pub(super) fn reserve_preview(
        &self,
        request: &DataSourceQueryParams,
        sql: Option<String>,
    ) -> Result<Option<Prepared>, policy::Rejection> {
        let Some(sql) = sql else {
            return Ok(None);
        };
        let context = request.expected_context.clone().ok_or(policy::Rejection {
            code: JsonRpcErrorCode::InvalidParams,
            message: "A prévia exige o contexto da operação.",
        })?;
        self.previews
            .reserve(context, request.client_context.clone().unwrap_or_default())
            .map(|lease| Some(Prepared { lease, sql }))
            .map_err(|message| policy::Rejection {
                code: JsonRpcErrorCode::DataSourcePreviewUnavailable,
                message,
            })
    }

    pub(super) fn datasource_preview_decide_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.preview.decide");
        };
        let request = match parse_params::<DataSourcePreviewDecideParams>(
            request_id.as_ref(),
            params,
            "datasource.preview.decide exige previewId, decision, name e contexto",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        // Rollback may discard the original transaction even after a profile edit;
        // commit must still match the currently saved destination and policy.
        if request.decision == DataSourcePreviewDecision::Commit {
            let profile = match Self::find_profile(&root, &request.name) {
                Ok(profile) => profile,
                Err(response) => return com_id(*response, request_id),
            };
            if let Err(rejection) = policy::check_context(
                &root,
                &profile,
                Some(&request.expected_context),
                Some(&request.client_context),
            )
            .and_then(|()| policy::check_read_only(&profile, true))
            {
                self.previews.revoke(&root, &request.name);
                return rejection.response(
                    request_id,
                    &request.name,
                    Some(&request.client_context),
                );
            }
        }
        match self.previews.decide(&request) {
            Ok(job_id) => {
                JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
            }
            Err(message) => policy::Rejection {
                code: JsonRpcErrorCode::DataSourcePreviewUnavailable,
                message,
            }
            .response(request_id, &request.name, Some(&request.client_context)),
        }
    }
}
