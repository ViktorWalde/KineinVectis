//! Handler dos pedidos `datasource.*` (`impl Core`).
//!
//! Fino: roteia, valida params, delega ao `crate::datasource` e formata a
//! resposta. A regra de negocio — validacao, ordenacao, identidade pelo nome —
//! vive no dominio, nao aqui.

use kinein_protocol::{
    DataSourceIntrospectParams, DataSourceListParams, DataSourceListResult, DataSourceProfile,
    DataSourceRemoveParams, DataSourceSaveParams, DataSourceTestAccepted, DataSourceTestParams,
    DataSourceWriteResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
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
            "datasource.introspect" => {
                Some(self.datasource_introspect_response(request_id, params))
            }
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
        let profile = match Self::find_profile(&root, &pedido.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };

        let secret = match Self::resolve_secret(&profile, pedido.password) {
            Ok(secret) => secret,
            Err(response) => return *response,
        };

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.test");
        };
        let titulo = format!("Testar {}", profile.name);
        let job_id = jobs.spawn("datasource", titulo, JobRisk::Low, false, move |ctx| {
            // O MOTOR decide quem responde. Um so' `datasource.test` para os
            // dois: a UI nao precisa saber com qual banco esta falando para
            // pedir um teste.
            // O MOTOR decide quem responde, e os tres devolvem a MESMA
            // forma: versao ou (mensagem, precisa de segredo). O `MongoDB` nao
            // tem SQLSTATE — o campo simplesmente nao vai no evento dele, e a
            // UI ja' decide pelo `secretRequired`, nunca pelo texto.
            let evento = match profile.engine {
                kinein_protocol::DataSourceEngine::Sqlite => resultado_do_teste(
                    ctx,
                    &profile,
                    crate::datasource::sqlite::probe_file(&profile),
                ),
                kinein_protocol::DataSourceEngine::Mongo => {
                    let bruto = crate::datasource::mongo::probe_server(&profile, secret.as_ref());
                    resultado_do_teste(
                        ctx,
                        &profile,
                        bruto.map_err(|falha| crate::datasource::connection::ConnectionFailure {
                            message: falha.message,
                            // O `MongoDB` nao tem SQLSTATE. `None` e' a
                            // verdade; uma string vazia seria um codigo que
                            // nao existe se passando por um que existe.
                            sql_state: None,
                            secret_required: falha.secret_required,
                        }),
                    )
                }
                kinein_protocol::DataSourceEngine::Postgres => resultado_do_teste(
                    ctx,
                    &profile,
                    crate::datasource::connection::probe_server(&profile, secret.as_ref()),
                ),
            };
            let ok = evento["ok"].as_bool().unwrap_or(false);
            ctx.emit_event("event.datasource.tested", evento);
            if ok {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });

        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }

    /// Traduz a politica do perfil na senha (ou na recusa) desta chamada.
    ///
    /// Um dono so' porque `datasource.test` e `datasource.introspect` fazem a
    /// MESMA pergunta: "tenho a senha para abrir esta conexao?". Duas copias
    /// divergiriam exatamente como as duas copias de `isWordChar` divergiram
    /// (`docs/roadmaps/39` §5).
    fn resolve_secret(
        profile: &DataSourceProfile,
        password: Option<String>,
    ) -> Result<Option<Secret>, Box<JsonRpcResponse>> {
        let recusa = |mensagem: String| {
            Box::new(JsonRpcResponse::failure(
                None,
                JsonRpcError::new(JsonRpcErrorCode::SecretRequired, mensagem, None),
            ))
        };
        match crate::datasource::secret::plan_for(profile) {
            SecretPlan::DelegateToDriver => Ok(password.map(Secret::new)),
            SecretPlan::ReadEnvironment(variavel) => {
                crate::datasource::secret::read_environment(&variavel)
                    .map(Some)
                    .ok_or_else(|| {
                        recusa(format!(
                            "a variavel `{variavel}` nao esta definida (ou esta vazia)"
                        ))
                    })
            }
            SecretPlan::AskUser => password.map(Secret::new).map(Some).ok_or_else(|| {
                recusa(format!(
                    "o perfil `{}` pede a senha a cada sessao",
                    profile.name
                ))
            }),
        }
    }

    /// Acha o perfil salvo, ou a resposta que explica que ele nao existe.
    fn find_profile(
        root: &std::path::Path,
        name: &str,
    ) -> Result<DataSourceProfile, Box<JsonRpcResponse>> {
        crate::datasource::list(root)
            .into_iter()
            .find(|candidato| candidato.name == name)
            .ok_or_else(|| {
                Box::new(JsonRpcResponse::failure(
                    None,
                    JsonRpcError::new(
                        JsonRpcErrorCode::InvalidParams,
                        format!("nao ha perfil chamado `{name}`"),
                        None,
                    ),
                ))
            })
    }

    /// `datasource.introspect` — esquemas, tabelas e colunas do banco.
    ///
    /// Job pelo mesmo motivo do `datasource.test`: sao tres consultas pela
    /// REDE, e o laco de despacho nao pode esperar por elas.
    fn datasource_introspect_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "datasource.introspect");
        };
        let pedido = match parse_params::<DataSourceIntrospectParams>(
            request_id.as_ref(),
            params,
            "datasource.introspect exige { name } e aceita { password }",
        ) {
            Ok(pedido) => pedido,
            Err(response) => return *response,
        };
        let profile = match Self::find_profile(&root, &pedido.name) {
            Ok(profile) => profile,
            Err(response) => return com_id(*response, request_id),
        };
        let secret = match Self::resolve_secret(&profile, pedido.password) {
            Ok(secret) => secret,
            Err(response) => return com_id(*response, request_id),
        };

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "datasource.introspect");
        };
        let titulo = format!("Ler {}", profile.name);
        let job_id = jobs.spawn("datasource", titulo, JobRisk::Low, false, move |ctx| {
            // DUAS FORMAS, NUNCA AS DUAS AO MESMO TEMPO. `schemas` e' a
            // arvore `esquema -> tabela -> coluna` dos motores relacionais;
            // `collections` e' a do `MongoDB`, onde campo nao e' coluna. A UI
            // escolhe a visao por QUAL das duas chegou — forcar o Mongo na
            // primeira faria a tela afirmar que todo documento tem o campo,
            // que ele tem um tipo so' e que nao ha' aninhamento.
            let evento = if profile.engine == kinein_protocol::DataSourceEngine::Mongo {
                match crate::datasource::mongo::read_structure(&profile, secret.as_ref()) {
                    Ok(collections) => json!({
                        "jobId": ctx.id(),
                        "name": profile.name,
                        "ok": true,
                        "collections": collections,
                    }),
                    Err(falha) => json!({
                        "jobId": ctx.id(),
                        "name": profile.name,
                        "ok": false,
                        "message": falha.message,
                        "secretRequired": falha.secret_required,
                    }),
                }
            } else {
                let resultado = if profile.engine == kinein_protocol::DataSourceEngine::Sqlite {
                    crate::datasource::sqlite::read_structure(&profile)
                } else {
                    crate::datasource::introspect::read_structure(&profile, secret.as_ref())
                };
                match &resultado {
                    Ok(schemas) => json!({
                        "jobId": ctx.id(),
                        "name": profile.name,
                        "ok": true,
                        "schemas": schemas,
                    }),
                    Err(falha) => json!({
                        "jobId": ctx.id(),
                        "name": profile.name,
                        "ok": false,
                        "message": falha.message,
                        "sqlState": falha.sql_state,
                        "secretRequired": falha.secret_required,
                    }),
                }
            };
            let ok = evento["ok"].as_bool().unwrap_or(false);
            ctx.emit_event("event.datasource.introspected", evento);
            if ok {
                JobOutcome::Success
            } else {
                JobOutcome::Failed
            }
        });

        JsonRpcResponse::success(request_id, json!(DataSourceTestAccepted { job_id }))
    }
}

/// Monta o evento de `datasource.test` a partir do resultado de um motor.
///
/// Existe uma vez so' porque os tres motores respondem a MESMA pergunta, e
/// tres copias do mesmo `json!` divergiriam no campo que menos se olha — que
/// aqui e' justamente o `secretRequired`, o que faz a UI pedir a senha.
fn resultado_do_teste(
    ctx: &crate::jobs::JobContext,
    profile: &DataSourceProfile,
    resultado: Result<String, crate::datasource::connection::ConnectionFailure>,
) -> Value {
    match resultado {
        Ok(versao) => json!({
            "jobId": ctx.id(),
            "name": profile.name,
            "ok": true,
            "serverVersion": versao,
        }),
        Err(falha) => json!({
            "jobId": ctx.id(),
            "name": profile.name,
            "ok": false,
            "message": falha.message,
            "sqlState": falha.sql_state,
            // A UI abre o dialogo de senha por ESTE campo, nunca lendo a
            // mensagem: o texto do servidor e' localizado.
            "secretRequired": falha.secret_required,
        }),
    }
}

/// Poe o `id` do pedido numa resposta de recusa construida sem ele.
///
/// As recusas nascem em funcoes que nao conhecem o `request_id` (elas servem
/// a dois metodos); o `id` e' colado aqui, no unico lugar que o tem.
fn com_id(mut resposta: JsonRpcResponse, request_id: Option<Value>) -> JsonRpcResponse {
    resposta.id = request_id;
    resposta
}
