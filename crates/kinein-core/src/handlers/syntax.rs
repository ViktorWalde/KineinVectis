//! Handlers for local incremental syntax intelligence (`syntaxTree.*`).

use std::path::Path;

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SyntaxTreeIndentParams,
    SyntaxTreeIndentResult, SyntaxTreeUpdateParams,
};
use serde_json::{Value, json};

use crate::{
    Core, fsops,
    lang::{IndentTrigger, SyntaxTreeError},
    rpc::{fs_error_response, no_workspace_response, parse_params},
};

impl Core {
    /// Routes local syntax-tree requests.
    pub(crate) fn syntax_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "syntaxTree.update" => Some(self.syntax_update_response(request_id, params)),
            "syntaxTree.indent" => Some(self.syntax_indent_response(request_id, params)),
            _ => None,
        }
    }

    fn syntax_update_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "syntaxTree.update");
        };
        let parsed = match parse_params::<SyntaxTreeUpdateParams>(
            request_id.as_ref(),
            params,
            "syntaxTree.update requer path, content e version",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };

        match self.syntax.update(&path, &parsed.content, parsed.version) {
            Ok(snapshot) => JsonRpcResponse::success(request_id, json!(snapshot)),
            Err(error) => syntax_error_response(request_id, &path, &error),
        }
    }

    /// Answers what the grammar says about indentation at one position.
    ///
    /// Never an error when there is no tree: "no answer" is a legitimate state,
    /// and the editor already applied its local fallback before asking. Turning
    /// it into a failure would make the log noisy about something that is
    /// working as designed.
    fn syntax_indent_response(
        // `&self`, e nao `&mut self`: responder sobre indentacao NAO reparseia
        // nem toca na arvore — ela ja' esta' la'. O clippy pegou, e a assinatura
        // agora diz a verdade sobre o que esta consulta faz.
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "syntaxTree.indent");
        };
        let parsed = match parse_params::<SyntaxTreeIndentParams>(
            request_id.as_ref(),
            params,
            "syntaxTree.indent requer path, version, line, column e trigger",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let path = match fsops::confine_file(&root, Path::new(&parsed.path)) {
            Ok(path) => path,
            Err(error) => return fs_error_response(request_id, &error),
        };
        let Some(trigger) = parse_trigger(&parsed.trigger) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("trigger desconhecido: {}", parsed.trigger),
                    None,
                ),
            );
        };

        match self
            .syntax
            .indent(&path, parsed.line, parsed.column, trigger)
        {
            Some(answer) => JsonRpcResponse::success(
                request_id,
                json!(SyntaxTreeIndentResult {
                    path: path.display().to_string(),
                    version: answer.version,
                    language: answer.language,
                    level: answer.level,
                }),
            ),
            // Sem arvore para este arquivo: o editor fica com o fallback dele.
            None => JsonRpcResponse::success(request_id, Value::Null),
        }
    }
}

/// `None` para trigger desconhecido: lista de PERMITIDOS, e o nome do que veio
/// entra na mensagem.
fn parse_trigger(trigger: &str) -> Option<IndentTrigger> {
    match trigger {
        "newline" => Some(IndentTrigger::Newline),
        "closeDelimiter" => Some(IndentTrigger::CloseDelimiter),
        _ => None,
    }
}

fn syntax_error_response(
    request_id: Option<Value>,
    path: &Path,
    error: &SyntaxTreeError,
) -> JsonRpcResponse {
    let code = if matches!(error, SyntaxTreeError::TooLarge { .. }) {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            code,
            error.to_string(),
            Some(json!({ "path": path.display().to_string() })),
        ),
    )
}
