//! Handler dos pedidos `datasource.*` (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::datasource` e formata a
//! resposta. A regra de negocio — validacao, ordenacao, identidade pelo nome —
//! vive no dominio, nao aqui.

use kinein_protocol::{
    DataSourceListParams, DataSourceListResult, DataSourceRemoveParams, DataSourceSaveParams,
    DataSourceTestAccepted, DataSourceTestParams, DataSourceWriteResult, JobRisk, JsonRpcError,
    JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::datasource::secret::{Secret, SecretPlan};
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

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
            "datasource.test" => Some(self.datasource_test_response(request_id, params)),
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

    /// `datasource.test` — conecta uma vez e conta o que aconteceu.
    ///
    /// E' a primeira acao util de um cliente de banco, e a unica que prova o
    /// caminho inteiro: perfil -> politica de segredo -> driver -> servidor.
    ///
    /// Roda como JOB porque conectar espera a REDE. Cinco segundos de timeout
    /// no laco de despacho travariam a IDE inteira — teclado incluso.
    ///
    /// Quando a politica do perfil e' "perguntar" e nenhuma senha veio, a
    /// resposta e' `SECRET_REQUIRED`, um codigo proprio: a UI sabe que deve
    /// abrir o dialogo e tentar de novo, sem ler texto de mensagem.
    fn datasource_test_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.test");
        };
        let pedido = match parse_params::<DataSourceTestParams>(
            request_id.as_ref(),
            params,
            "datasource.test exige { name } e aceita { password }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(profile) = crate::datasource::list(&root)
            .into_iter()
            .find(|candidato| candidato.name == pedido.name)
        else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!("nao ha perfil chamado `{}`", pedido.name),
                    None,
                ),
            );
        };

        let secret = match crate::datasource::secret::plan_for(&profile) {
            SecretPlan::DelegateToDriver => pedido.password.map(Secret::new),
            SecretPlan::ReadEnvironment(variavel) => {
                match crate::datasource::secret::read_environment(&variavel) {
                    Some(secret) => Some(secret),
                    None => {
                        return JsonRpcResponse::failure(
                            request_id,
                            JsonRpcError::new(
                                JsonRpcErrorCode::SecretRequired,
                                format!(
                                    "a variavel `{variavel}` nao esta definida (ou esta vazia)"
                                ),
                                None,
                            ),
                        );
                    }
                }
            }
            SecretPlan::AskUser => match pedido.password {
                Some(valor) => Some(Secret::new(valor)),
                None => {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::SecretRequired,
                            format!("o perfil `{}` pede a senha a cada sessao", profile.name),
                            None,
                        ),
                    );
                }
            },
        };

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.test");
        };
        let titulo = format!("Testar {}", profile.name);
        let job_id = jobs.spawn("datasource", titulo, JobRisk::Low, false, move |ctx| {
            let resultado = crate::datasource::connection::probe_server(&profile, secret.as_ref());
            let (ok, versao, mensagem) = match resultado {
                Ok(versao) => (true, Some(versao), None),
                Err(mensagem) => (false, None, Some(mensagem)),
            };
            ctx.emit_event(
                "event.datasource.tested",
                json!({
                    "jobId": ctx.id(),
                    "name": profile.name,
                    "ok": ok,
                    "serverVersion": versao,
                    "message": mensagem,
                }),
            );
            if ok {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });

        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }
}
