//! Handler de `project.fileContext` (`impl Core`).
//!
//! Fino: valida o caminho, confina a raiz do workspace e delega ao dominio
//! `crate::project_context`. Sincrono de proposito — ler o banco de comandos
//! e IO barato, e o painel que consome isto acompanha o cursor do usuario:
//! um job por troca de arquivo seria cerimonia com latencia.

use kinein_protocol::{FileContextParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse};
use serde_json::{Value, json};

use crate::rpc::parse_params;
use crate::{Core, project_context};

impl Core {
    pub(crate) fn project_file_context_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<FileContextParams>(
            request_id.as_ref(),
            params,
            "project.fileContext aceita apenas o campo `path`",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };

        let Some(root) = self.workspace_root() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    "project.fileContext exige workspace aberto",
                    None,
                ),
            );
        };

        // Confinamento a raiz, como todo caminho que entra pelo protocolo: o
        // contexto de compilacao nao e desculpa para ler fora do workspace.
        let bruto = std::path::Path::new(&parsed.path);
        let absoluto = if bruto.is_absolute() {
            bruto.to_path_buf()
        } else {
            root.join(bruto)
        };
        let Ok(canonico) = absoluto.canonicalize() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    "arquivo inexistente",
                    None,
                ),
            );
        };
        if !canonico.starts_with(&root) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    "o arquivo esta fora do workspace aberto",
                    None,
                ),
            );
        }

        let contexto = project_context::file_context(&root, &canonico);
        JsonRpcResponse::success(request_id, json!(contexto))
    }
}
