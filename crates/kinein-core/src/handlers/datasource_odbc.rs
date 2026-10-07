//! Descoberta e autorizacao ODBC, sem carregar driver implicitamente.

use super::datasource::com_id;
use crate::{
    Core,
    rpc::{no_workspace_response, parse_params},
};
use kinein_protocol::{
    DataSourceOdbcAuthorizeParams, DataSourceOdbcAuthorizeResult, DataSourceOdbcSourcesParams,
    DataSourceOdbcSourcesResult, DataSourceProfile, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse,
};
use serde_json::{Value, json};

impl Core {
    /// A lista nao precisa de perfil nem de uma conexao aberta.
    pub(super) fn datasource_odbc_sources_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<DataSourceOdbcSourcesParams>(
            request_id.as_ref(),
            params,
            "datasource.odbc.sources não aceita parâmetros",
        ) {
            return *response;
        }
        match self.odbc.sources() {
            Ok(sources) => {
                JsonRpcResponse::success(request_id, json!(DataSourceOdbcSourcesResult { sources }))
            }
            Err(message) => invalid(request_id, message),
        }
    }

    /// O gesto nao conecta: guarda apenas o consentimento na memoria do core.
    pub(super) fn datasource_odbc_authorize_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.odbc.authorize");
        };
        let request = match parse_params::<DataSourceOdbcAuthorizeParams>(
            request_id.as_ref(),
            params,
            "datasource.odbc.authorize exige { name, identity, workspace }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        if request.workspace != root.to_string_lossy() {
            return invalid(
                request_id,
                "o projeto mudou; abra o aviso novamente".to_owned(),
            );
        }
        let profile = match Self::find_profile(&root, &request.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };
        let _activity =
            match self.begin_datasource_operation(&root, &profile.name, None, request_id.clone()) {
                Ok(activity) => activity,
                Err(response) => return *response,
            };
        match self.odbc.authorize(&root, &profile, &request.identity) {
            Ok(()) => JsonRpcResponse::success(
                request_id,
                json!(DataSourceOdbcAuthorizeResult {
                    name: request.name,
                    identity: request.identity,
                    workspace: request.workspace
                }),
            ),
            Err(message) => invalid(request_id, message),
        }
    }

    /// Um dono da recusa para teste, catalogo e console; nenhum driver abre aqui.
    pub(super) fn require_odbc_driver(
        &self,
        root: &std::path::Path,
        profile: &DataSourceProfile,
        id: Option<Value>,
    ) -> Result<(), Box<JsonRpcResponse>> {
        match self.odbc.required(root, profile) {
            Ok(None) => Ok(()),
            Ok(Some(source)) => {
                let details = json!({ "name": profile.name, "dsn": source.dsn, "driver": source.driver, "identity": source.identity, "workspace": root });
                let error = JsonRpcError::new(
                    JsonRpcErrorCode::DriverApprovalRequired,
                    "carregar o driver ODBC exige autorização explícita nesta sessão",
                    Some(details),
                );
                Err(Box::new(JsonRpcResponse::failure(id, error)))
            }
            Err(message) => Err(Box::new(invalid(id, message))),
        }
    }
}

fn invalid(id: Option<Value>, message: String) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
    )
}
