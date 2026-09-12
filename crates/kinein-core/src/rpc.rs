//! JSON-RPC helpers shared by the request handlers: structured error responses
//! and parameter parsing. Split out of `lib.rs` to shrink the core surface.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, LspTextDocumentPositionParams,
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{fsops, lsp, run, terminal, workspace};

pub(crate) fn no_workspace_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidRequest,
            "nenhum workspace aberto",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn fs_error_response(
    request_id: Option<Value>,
    error: &fsops::FsError,
) -> JsonRpcResponse {
    let code = if error.changed_path().is_some() {
        JsonRpcErrorCode::FileChanged
    } else if error.is_invalid_path() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    let details = error.changed_path().map(|path| json!({ "path": path }));
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(code, error.to_string(), details),
    )
}

pub(crate) fn workspace_error_response(
    request_id: Option<Value>,
    error: &workspace::WorkspaceError,
) -> JsonRpcResponse {
    let code = if error.is_invalid_path() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

pub(crate) fn terminal_unavailable_response(
    request_id: Option<Value>,
    method: &str,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "terminal nao esta habilitado neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn terminal_error_response(
    request_id: Option<Value>,
    error: &terminal::TerminalError,
) -> JsonRpcResponse {
    let code = match error {
        terminal::TerminalError::Process { .. } => JsonRpcErrorCode::InternalError,
        terminal::TerminalError::TooMany
        | terminal::TerminalError::NotOpen
        // Gesto válido no contrato, sem comportamento ainda (R5): é um pedido
        // que o core recusa, não uma falha interna.
        | terminal::TerminalError::MouseUnimplemented => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

pub(crate) fn run_unavailable_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "execucao de processos nao esta habilitada neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn run_error_response(
    request_id: Option<Value>,
    error: &run::RunError,
) -> JsonRpcResponse {
    let code = match error {
        run::RunError::Process { .. } => JsonRpcErrorCode::InternalError,
        run::RunError::AlreadyRunning
        | run::RunError::NotRunning
        | run::RunError::NoDefaultCommand { .. } => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

pub(crate) fn debug_unavailable_response(
    request_id: Option<Value>,
    method: &str,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "debug nao esta habilitado neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn debug_error_response(
    request_id: Option<Value>,
    error: &crate::dap::DebugError,
) -> JsonRpcResponse {
    let code = match error {
        crate::dap::DebugError::MissingAdapter { .. } => JsonRpcErrorCode::ToolNotFound,
        crate::dap::DebugError::Adapter { .. } => JsonRpcErrorCode::InternalError,
        crate::dap::DebugError::AlreadyRunning
        | crate::dap::DebugError::NotRunning
        | crate::dap::DebugError::NotStopped
        | crate::dap::DebugError::NoTarget { .. } => JsonRpcErrorCode::InvalidRequest,
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

pub(crate) fn lsp_unavailable_response(request_id: Option<Value>, method: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "LSP nao esta habilitado neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn lsp_error_response(
    request_id: Option<Value>,
    error: &lsp::LspError,
) -> JsonRpcResponse {
    let code = if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else if error.is_missing_tool() {
        JsonRpcErrorCode::ToolNotFound
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

/// O loop deste core nao tem gerenciador de jobs.
///
/// Vive aqui, e nao no `handlers/build.rs` onde nasceu, porque desde
/// 2026-09-04 o `datasource.test` tambem precisa dela — e duas copias de uma
/// mensagem de erro divergem exatamente como as duas copias de `isWordChar`
/// divergiram (DocsPublic/roadmaps/39 §5).
pub(crate) fn jobs_unavailable_response(
    request_id: Option<Value>,
    method: &str,
) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InternalError,
            "jobs nao estao habilitados neste loop do core",
            Some(json!({ "method": method })),
        ),
    )
}

pub(crate) fn parse_params<T>(
    request_id: Option<&Value>,
    params: Option<&Value>,
    hint: &str,
) -> Result<T, Box<JsonRpcResponse>>
where
    T: DeserializeOwned,
{
    let params = params.cloned().unwrap_or_else(|| json!({}));
    serde_json::from_value::<T>(params).map_err(|error| {
        Box::new(JsonRpcResponse::failure(
            request_id.cloned(),
            JsonRpcError::new(
                JsonRpcErrorCode::InvalidParams,
                hint,
                Some(json!({ "error": error.to_string() })),
            ),
        ))
    })
}

pub(crate) fn parse_lsp_position_params(
    root: &Path,
    request_id: Option<&Value>,
    params: Option<&Value>,
    hint: &str,
) -> Result<(PathBuf, LspTextDocumentPositionParams), Box<JsonRpcResponse>> {
    let parsed = parse_params::<LspTextDocumentPositionParams>(request_id, params, hint)?;
    let path = fsops::confine_file(root, Path::new(&parsed.path))
        .map_err(|error| Box::new(fs_error_response(request_id.cloned(), &error)))?;
    Ok((path, parsed))
}
