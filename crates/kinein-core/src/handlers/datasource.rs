//! Handler dos pedidos `datasource.*` (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::datasource` e formata a
//! resposta. A regra de negocio — validacao, ordenacao, identidade pelo nome —
//! vive no dominio, nao aqui.

use kinein_protocol::{
    DataSourceListParams, DataSourceListResult, DataSourceRemoveParams, DataSourceSaveParams,
    DataSourceWriteResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{no_workspace_response, parse_params};

impl Core {
    /// Roteia os metodos `datasource.*`.
    pub(crate) fn datasource_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "datasource.list" => Some(self.datasource_list_response(request_id, params)),
            "datasource.save" => Some(self.datasource_save_response(request_id, params)),
            "datasource.remove" => Some(self.datasource_remove_response(request_id, params)),
            _ => None,
        }
    }

    /// `datasource.list` — os perfis salvos neste workspace, ordenados.
    fn datasource_list_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.list");
        };
        if let Err(response) = parse_params::<DataSourceListParams>(
            request_id.as_ref(),
            params,
            "datasource.list nao aceita parametros",
        ) {
            return *response;
        }
        let resultado = DataSourceListResult {
            profiles: crate::datasource::list(&root),
        };
        JsonRpcResponse::success(request_id, json!(resultado))
    }

    /// `datasource.save` — cria ou substitui o perfil de mesmo nome.
    fn datasource_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.save");
        };
        let pedido = match parse_params::<DataSourceSaveParams>(
            request_id.as_ref(),
            params,
            "datasource.save exige { profile }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        match crate::datasource::save(&root, &pedido.profile) {
            Ok(profiles) => {
                JsonRpcResponse::success(request_id, json!(DataSourceWriteResult { profiles }))
            }
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, None),
            ),
        }
    }

    /// `datasource.remove` — tira o perfil do catalogo.
    ///
    /// Remover o que nao existe devolve sucesso com o catalogo atual: a UI nao
    /// precisa tratar "ja tinha sumido" como erro.
    fn datasource_remove_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.remove");
        };
        let pedido = match parse_params::<DataSourceRemoveParams>(
            request_id.as_ref(),
            params,
            "datasource.remove exige { name }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        match crate::datasource::remove(&root, &pedido.name) {
            Ok(profiles) => {
                JsonRpcResponse::success(request_id, json!(DataSourceWriteResult { profiles }))
            }
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
            ),
        }
    }
}
