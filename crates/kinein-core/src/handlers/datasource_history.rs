//! `datasource.history` e `datasource.history.clear` (`0.166.0`, passo 13b):
//! o historico de consultas por conexao, que o core anota e guarda fora do
//! projeto (`datasource::history`; `roadmaps/59` §5.2.1).

use kinein_protocol::{
    DataSourceHistoryCleared, DataSourceHistoryOutcome, DataSourceHistoryParams,
    DataSourceHistoryResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use super::datasource::com_id;
use crate::Core;
use crate::datasource::history::History;
use crate::rpc::{no_workspace_response, parse_params};

/// O que uma consulta que RODOU deixa para o historico.
pub(super) struct Ran {
    pub(super) outcome: DataSourceHistoryOutcome,
    pub(super) rows: Option<u64>,
}

/// Onde anotar: o historico ligado, o projeto e o texto pedido.
pub(super) struct HistoryTarget {
    history: History,
    root: std::path::PathBuf,
    name: String,
    sql: String,
}

impl HistoryTarget {
    /// Anota o que rodou; uma falha de disco vai a' saida do job, nunca
    /// muda o desfecho da consulta.
    pub(super) fn record(&self, ctx: &crate::jobs::JobContext, ran: &Ran) {
        if let Err(message) =
            self.history
                .record(&self.root, &self.name, &self.sql, ran.outcome, ran.rows)
        {
            ctx.emit_output(&format!("historico nao gravado: {message}"));
        }
    }
}

impl Core {
    /// Liga o historico de consultas. Gesto EXPLICITO do processo real, como
    /// [`Core::enable_persistence`]: sem ele o core nao escreve nada fora do
    /// projeto, e os testes nao tocam o estado da pessoa.
    pub fn enable_query_history(&mut self, history: History) {
        self.query_history = Some(history);
    }

    /// O alvo da anotacao de uma consulta, quando o historico esta' ligado.
    pub(super) fn history_target(
        &self,
        root: &std::path::Path,
        name: &str,
        sql: &str,
    ) -> Option<HistoryTarget> {
        self.query_history.clone().map(|history| HistoryTarget {
            history,
            root: root.to_path_buf(),
            name: name.to_owned(),
            sql: sql.to_owned(),
        })
    }

    /// `datasource.history { name }` e `datasource.history.clear { name }`.
    pub(super) fn datasource_history_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, method);
        };
        let request = match parse_params::<DataSourceHistoryParams>(
            request_id.as_ref(),
            params,
            "datasource.history exige { name }",
        ) {
            Ok(request) => request,
            Err(response) => return *response,
        };
        if let Err(response) = Self::find_profile(&root, &request.name) {
            return com_id(*response, request_id);
        }
        if method == "datasource.history.clear" {
            let cleared = self
                .query_history
                .as_ref()
                .map_or(Ok(()), |history| history.clear(&root, &request.name));
            return match cleared {
                Ok(()) => JsonRpcResponse::success(
                    request_id,
                    json!(DataSourceHistoryCleared { name: request.name }),
                ),
                Err(message) => JsonRpcResponse::failure(
                    request_id,
                    JsonRpcError::new(JsonRpcErrorCode::InternalError, message, None),
                ),
            };
        }
        let entries = self
            .query_history
            .as_ref()
            .map(|history| history.list(&root, &request.name))
            .unwrap_or_default();
        JsonRpcResponse::success(
            request_id,
            json!(DataSourceHistoryResult {
                name: request.name,
                entries
            }),
        )
    }
}
