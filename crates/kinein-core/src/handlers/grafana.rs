//! Handler dos pedidos `grafana.*` (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::grafana` e formata a
//! resposta. A regra — validacao, normalizacao da URL, cruzamento com o
//! catalogo de bancos — vive no dominio, nao aqui.

use kinein_protocol::{
    GrafanaForgetParams, GrafanaGetParams, GrafanaProbeAccepted, GrafanaProbeParams,
    GrafanaProfileResult, GrafanaSaveParams, JobRisk, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::datasource::secret::{Secret, SecretPlan};
use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, no_workspace_response, parse_params};

impl Core {
    /// Roteia os metodos `grafana.*`.
    pub(crate) fn grafana_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "grafana.get" => Some(self.grafana_get_response(request_id, params)),
            "grafana.save" => Some(self.grafana_save_response(request_id, params)),
            "grafana.forget" => Some(self.grafana_forget_response(request_id, params)),
            "grafana.probe" => Some(self.grafana_probe_response(request_id, params)),
            _ => None,
        }
    }

    /// `grafana.get` — a instancia salva neste workspace, se houver.
    fn grafana_get_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "grafana.get");
        };
        if let Err(response) = parse_params::<GrafanaGetParams>(
            request_id.as_ref(),
            params,
            "grafana.get nao aceita parametros",
        ) {
            return *response;
        }
        JsonRpcResponse::success(
            request_id,
            json!(GrafanaProfileResult {
                profile: crate::grafana::get(&root),
            }),
        )
    }

    /// `grafana.save` — grava a instancia, substituindo a anterior.
    fn grafana_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "grafana.save");
        };
        let pedido = match parse_params::<GrafanaSaveParams>(
            request_id.as_ref(),
            params,
            "grafana.save exige { profile }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        match crate::grafana::save(&root, &pedido.profile) {
            Ok(profile) => JsonRpcResponse::success(
                request_id,
                json!(GrafanaProfileResult {
                    profile: Some(profile)
                }),
            ),
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, None),
            ),
        }
    }

    /// `grafana.forget` — o workspace deixa de ter instancia.
    fn grafana_forget_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "grafana.forget");
        };
        if let Err(response) = parse_params::<GrafanaForgetParams>(
            request_id.as_ref(),
            params,
            "grafana.forget nao aceita parametros",
        ) {
            return *response;
        }
        match crate::grafana::forget(&root) {
            Ok(()) => JsonRpcResponse::success(request_id, json!(GrafanaProfileResult::default())),
            Err(mensagem) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, mensagem, None),
            ),
        }
    }

    /// `grafana.probe` — alcanca a instancia e conta o que ela tem.
    ///
    /// Roda como JOB pelo mesmo motivo do `datasource.test`: espera a REDE, e
    /// cinco segundos no laco de despacho travariam a IDE inteira.
    ///
    /// O cruzamento com o catalogo de bancos acontece AQUI, e nao no cliente
    /// HTTP, porque ele precisa dos dois lados: o que o Grafana respondeu e o
    /// que este workspace tem salvo.
    fn grafana_probe_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "grafana.probe");
        };
        let pedido = match parse_params::<GrafanaProbeParams>(
            request_id.as_ref(),
            params,
            "grafana.probe aceita { token }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let Some(profile) = crate::grafana::get(&root) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "nenhum Grafana configurado neste workspace".to_owned(),
                    None,
                ),
            );
        };
        let token = match Self::resolve_grafana_token(&profile, pedido.token) {
            Ok(token) => token,
            Err(response) => return com_id(*response, request_id),
        };

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "grafana.probe");
        };
        let perfis = crate::datasource::list(&root);
        let titulo = format!("Sondar {}", profile.url);
        let job_id = jobs.spawn("grafana", titulo, JobRisk::Low, false, move |ctx| {
            let mut resultado = crate::grafana::client::probe(&profile.url, token.as_ref());
            resultado.matches = crate::grafana::cross_reference(&perfis, &resultado.data_sources);
            let alcancou = resultado.reachable;
            let mut evento = json!(resultado);
            if let Some(objeto) = evento.as_object_mut() {
                objeto.insert("jobId".to_owned(), json!(ctx.id()));
                objeto.insert("url".to_owned(), json!(profile.url));
            }
            ctx.emit_event("event.grafana.probed", evento);
            if alcancou {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });

        JsonRpcResponse::success(request_id, json!(GrafanaProbeAccepted { job_id }))
    }

    /// Traduz a politica do perfil no token (ou na recusa) desta chamada.
    ///
    /// Mesmo desenho do `resolve_secret` do `datasource`, e com a mesma razao
    /// para existir uma vez so'. A diferenca esta' no `DelegateToDriver`: para
    /// o Grafana ele significa literalmente "nao mande token", e a sonda segue
    /// util assim — `/api/health` responde sem credencial.
    fn resolve_grafana_token(
        profile: &kinein_protocol::GrafanaProfile,
        token: Option<String>,
    ) -> Result<Option<Secret>, Box<JsonRpcResponse>> {
        let recusa = |mensagem: String| {
            Box::new(JsonRpcResponse::failure(
                None,
                JsonRpcError::new(JsonRpcErrorCode::SecretRequired, mensagem, None),
            ))
        };
        match crate::grafana::plan_for(profile) {
            SecretPlan::DelegateToDriver => Ok(token.map(Secret::new)),
            SecretPlan::ReadEnvironment(variavel) => {
                crate::datasource::secret::read_environment(&variavel)
                    .map(Some)
                    .ok_or_else(|| {
                        recusa(format!(
                            "a variavel `{variavel}` nao esta definida (ou esta vazia)"
                        ))
                    })
            }
            SecretPlan::AskUser => token
                .map(Secret::new)
                .map(Some)
                .ok_or_else(|| recusa("este Grafana pede o token a cada sessao".to_owned())),
        }
    }
}

/// Devolve a resposta com o `id` do pedido.
///
/// As recusas nascem sem `id` porque quem as monta nao o tem em maos; costurar
/// aqui evita passar o `id` por dentro de cada funcao auxiliar.
fn com_id(mut response: JsonRpcResponse, request_id: Option<Value>) -> JsonRpcResponse {
    response.id = request_id;
    response
}
