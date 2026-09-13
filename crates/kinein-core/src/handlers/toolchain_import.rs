//! Handlers do gerenciador de toolchain que LE o disco (`impl Core`).
//!
//! `toolchain.inspectSysroot` (o que uma pasta de sysroot contem) e
//! `toolchain.importKit` (a proposta de kit lida de um SDK Yocto, de uma
//! arvore Buildroot ou de uma pasta de toolchain). Nenhum dos dois grava:
//! aplicar e' o `toolchain.setKit`/`toolchain.set` de sempre.
//!
//! Fino: valida params, chama `toolchain::sysroot`/`toolchain::import`,
//! formata. Nao exigem workspace — leem caminhos desta maquina.

use std::path::Path;

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, KitImportParams, SysrootInspectParams,
};
use serde_json::{Value, json};

use crate::rpc::parse_params;
use crate::{Core, toolchain};

impl Core {
    /// Roteia `toolchain.inspectSysroot` e `toolchain.importKit`.
    pub(crate) fn toolchain_import_request_response(
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "toolchain.inspectSysroot" => Some(Self::inspect_sysroot_response(request_id, params)),
            "toolchain.importKit" => Some(Self::import_kit_response(request_id, params)),
            _ => None,
        }
    }

    fn inspect_sysroot_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<SysrootInspectParams>(
            request_id.as_ref(),
            params,
            "toolchain.inspectSysroot requer o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let caminho = Path::new(parsed.path.trim());
        if !caminho.is_absolute() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "toolchain.inspectSysroot requer um caminho absoluto",
                    Some(json!({ "path": parsed.path })),
                ),
            );
        }
        JsonRpcResponse::success(request_id, json!(toolchain::sysroot::inspect(caminho)))
    }

    fn import_kit_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<KitImportParams>(
            request_id.as_ref(),
            params,
            "toolchain.importKit requer o campo path",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let caminho = Path::new(parsed.path.trim());
        if !caminho.is_absolute() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "toolchain.importKit requer um caminho absoluto",
                    Some(json!({ "path": parsed.path })),
                ),
            );
        }
        match toolchain::import::import(caminho) {
            Ok(kit) => JsonRpcResponse::success(request_id, json!(kit)),
            // Nao reconhecer nao e' erro interno: e' a resposta a pergunta.
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    mensagem,
                    Some(json!({ "path": parsed.path })),
                ),
            ),
        }
    }
}
