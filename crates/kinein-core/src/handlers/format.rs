//! Handlers for `format.*` requests (`impl Core`): the router plus the thin
//! `format.text` handler over `crate::format`.

use std::io;
use std::path::Path;

use kinein_protocol::{
    FormatCapabilitiesResult, FormatTextParams, FormatTextResult, FormatterCapability,
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::format::{self, FormatError};
use crate::rpc::{fs_error_response, no_workspace_response, parse_params};
use crate::{Core, fsops};

impl Core {
    /// Roteia os metodos `format.*`; `None` quando o metodo nao e de formatacao.
    pub(crate) fn format_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "format.text" => Some(self.format_text_response(request_id, params)),
            "format.capabilities" => Some(Self::format_capabilities_response(request_id)),
            _ => None,
        }
    }

    /// `format.capabilities` (0.61.0): publica o catálogo de formatters para
    /// a UI não manter uma segunda lista. Não exige workspace nem consulta o
    /// `PATH` — é o mapa estático de extensões, derivado da mesma constante
    /// que `formatter_for_path` usa para decidir.
    fn format_capabilities_response(request_id: Option<Value>) -> JsonRpcResponse {
        let formatters = format::capabilities()
            .into_iter()
            .map(|(kind, extensions)| FormatterCapability {
                id: kind.id().to_owned(),
                extensions: extensions.iter().map(|ext| (*ext).to_owned()).collect(),
            })
            .collect();
        JsonRpcResponse::success(request_id, json!(FormatCapabilitiesResult { formatters }))
    }

    fn format_text_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "format.text");
        };
        let parsed = match parse_params::<FormatTextParams>(
            request_id.as_ref(),
            params,
            "format.text requer os campos path e text",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let file = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(file) => file,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(kind) = format::formatter_for_path(&file) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "nenhum formatter registrado para esta extensao de arquivo",
                    Some(json!({ "path": parsed.path })),
                ),
            );
        };

        let command = format::formatter_command(kind, &root, &file);
        match format::run_formatter(command, kind.id(), &parsed.text) {
            Ok(text) => {
                let changed = text != parsed.text;
                JsonRpcResponse::success(
                    request_id,
                    json!(FormatTextResult {
                        path: file.display().to_string(),
                        text,
                        changed,
                        formatter: kind.id().to_owned(),
                    }),
                )
            }
            Err(error) => format_error_response(request_id, &error),
        }
    }
}

/// Maps a [`FormatError`] to the structured JSON-RPC failure for `format.text`.
fn format_error_response(request_id: Option<Value>, error: &FormatError) -> JsonRpcResponse {
    let rpc_error = match error {
        FormatError::Spawn { tool, source } if source.kind() == io::ErrorKind::NotFound => {
            JsonRpcError::new(
                JsonRpcErrorCode::ToolNotFound,
                format!("{tool} nao foi encontrado no PATH"),
                Some(json!({ "tool": tool })),
            )
        }
        FormatError::Spawn { tool, source } => JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            format!("falha ao iniciar {tool}: {source}"),
            Some(json!({ "tool": tool })),
        ),
        FormatError::Failed { tool, stderr } => JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            format!("{tool} falhou: {stderr}"),
            Some(json!({ "tool": tool })),
        ),
        FormatError::Io { tool, source } => JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            format!("erro de I/O com {tool}: {source}"),
            Some(json!({ "tool": tool })),
        ),
        FormatError::NotUtf8 { tool } => JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            format!("{tool} produziu saida que nao e UTF-8 valido"),
            Some(json!({ "tool": tool })),
        ),
    };
    JsonRpcResponse::failure(request_id, rpc_error)
}
