//! Handlers dos metodos `sim.*` (`impl Core`): simulacao por conceito.
//!
//! Fino, como a §4 regra 2 manda: valida params, chama o dominio e formata a
//! resposta. Quem sabe o que e' um conceito e o [`crate::sim`].
//!
//! **Nenhum destes metodos exige workspace aberto.** Escolher um conceito e
//! conferir uma formula sao operacoes sobre o catalogo e sobre texto, nao sobre
//! disco. Guardar a simulacao em `.kinein/simulacoes/` e' fatia propria, e e' la
//! que o workspace passa a importar.

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, SimCatalogParams, SimCheckParams,
    SimCheckSystemParams, SimEstimateParams, SimEvaluateParams, SimForgetParams, SimInspectParams,
    SimInspectResult, SimRunParams, SimRunSystemParams, SimSaveParams,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::sim::corrida::ErroDeExecucao;
use crate::sim::corrida_sistema::ErroDeSistemaExec;
use crate::sim::integrador::{self, ErroDeCorrida};
use crate::sim::sistema::ErroDeSistema;
use crate::sim::{catalogo, corrida, corrida_sistema, formula, persistencia};
use crate::{Core, sim::formula::ErroAvaliacao};

impl Core {
    /// Roteia os metodos `sim.*`; `None` quando o metodo nao e deles.
    pub(crate) fn sim_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "sim.catalog" => Some(Self::sim_catalog_response(request_id, params)),
            "sim.inspectFormula" => Some(Self::sim_inspect_response(request_id, params)),
            "sim.checkFormula" => Some(Self::sim_check_response(request_id, params)),
            "sim.evaluate" => Some(Self::sim_evaluate_response(request_id, params)),
            "sim.run" => Some(self.sim_run_response(request_id, params)),
            "sim.checkSystem" => Some(Self::sim_check_system_response(request_id, params)),
            "sim.runSystem" => Some(self.sim_run_system_response(request_id, params)),
            "sim.estimate" => Some(Self::sim_estimate_response(request_id, params)),
            "sim.list" => Some(self.sim_list_response(request_id)),
            "sim.save" => Some(self.sim_save_response(request_id, params)),
            "sim.forget" => Some(self.sim_forget_response(request_id, params)),
            _ => None,
        }
    }

    fn sim_catalog_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<SimCatalogParams>(
            request_id.as_ref(),
            params,
            "sim.catalog aceita apenas course",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let conceitos = catalogo::conceitos();
        let filtrados: Vec<_> = match parsed.course {
            Some(curso) => conceitos
                .into_iter()
                .filter(|c| c.course == curso)
                .collect(),
            None => conceitos,
        };
        JsonRpcResponse::success(request_id, json!({ "concepts": filtrados }))
    }

    fn sim_inspect_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<SimInspectParams>(
            request_id.as_ref(),
            params,
            "sim.inspectFormula exige formula",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match formula::inspecionar(&parsed.formula) {
            Ok(variables) => {
                JsonRpcResponse::success(request_id, json!(SimInspectResult { variables }))
            }
            // A mensagem aqui ja e' a da IDE: o texto do crate tem endereco de
            // ponteiro dentro e nunca atravessa a fronteira (ADR-0006).
            Err(mensagem) => invalid(request_id, &mensagem),
        }
    }

    fn sim_check_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<SimCheckParams>(
            request_id.as_ref(),
            params,
            "sim.checkFormula exige concept, formula e bindings",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(conceito) = catalogo::conceito(&parsed.concept) else {
            return conceito_desconhecido(request_id, &parsed.concept);
        };
        let resultado = formula::checar(&conceito, &parsed.formula, &parsed.bindings);
        JsonRpcResponse::success(request_id, json!(resultado))
    }

    fn sim_evaluate_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<SimEvaluateParams>(
            request_id.as_ref(),
            params,
            "sim.evaluate exige concept, formula, bindings e values",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(conceito) = catalogo::conceito(&parsed.concept) else {
            return conceito_desconhecido(request_id, &parsed.concept);
        };
        match formula::avaliar(&conceito, &parsed.formula, &parsed.bindings, &parsed.values) {
            Ok(resultado) => JsonRpcResponse::success(request_id, json!(resultado)),
            // Cada erro vira um `code` proprio, e nao um texto para a UI casar.
            // Foi exatamente esse casamento por TEXTO que o `requestFailed` sem
            // `code` obrigou em 2026-09-04, e que o `SecretRequired` existe para
            // evitar.
            Err(erro) => erro_de_avaliacao(request_id, &erro),
        }
    }
}

impl Core {
    fn sim_list_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "sim.list");
        };
        JsonRpcResponse::success(
            request_id,
            json!({ "simulations": persistencia::listar(&root) }),
        )
    }

    fn sim_save_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "sim.save");
        };
        let parsed = match parse_params::<SimSaveParams>(
            request_id.as_ref(),
            params,
            "sim.save exige simulation",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match persistencia::salvar(&root, &parsed.simulation) {
            Ok(lista) => JsonRpcResponse::success(request_id, json!({ "simulations": lista })),
            Err(mensagem) => invalid(request_id, &mensagem),
        }
    }

    fn sim_forget_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "sim.forget");
        };
        let parsed = match parse_params::<SimForgetParams>(
            request_id.as_ref(),
            params,
            "sim.forget exige name",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match persistencia::esquecer(&root, &parsed.name) {
            Ok(lista) => JsonRpcResponse::success(request_id, json!({ "simulations": lista })),
            Err(mensagem) => invalid(request_id, &mensagem),
        }
    }

    fn sim_estimate_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<SimEstimateParams>(
            request_id.as_ref(),
            params,
            "sim.estimate exige duration, step e samples",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        JsonRpcResponse::success(
            request_id,
            json!(integrador::estimar(
                parsed.duration,
                parsed.step,
                parsed.samples
            )),
        )
    }

    fn sim_run_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<SimRunParams>(
            request_id.as_ref(),
            params,
            "sim.run exige concept, formula, bindings, values, initial, duration, step, \
             method e samples",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(conceito) = catalogo::conceito(&parsed.concept) else {
            return conceito_desconhecido(request_id, &parsed.concept);
        };
        match corrida::executar(
            &conceito,
            &parsed.formula,
            &parsed.bindings,
            &parsed.values,
            parsed.initial.y,
            parsed.initial.dy,
            parsed.duration,
            parsed.step,
            parsed.method,
            parsed.samples,
            // A configuracao do oraculo vem do `Core`, nao do ambiente. Quem
            // esta abaixo a recebe pronta e nao pergunta nada ao host — e' o
            // que deixa o teste apontar para um `Python` falso sem escrever
            // variavel de ambiente, que este projeto proibe.
            self.oraculo(),
        ) {
            Ok(resultado) => JsonRpcResponse::success(request_id, json!(resultado)),
            Err(erro) => erro_de_execucao(request_id, &erro),
        }
    }

    /// `sim.checkSystem` — as `n` formulas fecham com o conceito vetorial?
    fn sim_check_system_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<SimCheckSystemParams>(
            request_id.as_ref(),
            params,
            "sim.checkSystem exige concept e equations",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(conceito) = catalogo::conceito(&parsed.concept) else {
            return conceito_desconhecido(request_id, &parsed.concept);
        };
        JsonRpcResponse::success(
            request_id,
            json!(corrida_sistema::checar(&conceito, &parsed.equations)),
        )
    }

    /// `sim.runSystem` — integra `dY/dt = F(t, Y)`.
    fn sim_run_system_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<SimRunSystemParams>(
            request_id.as_ref(),
            params,
            "sim.runSystem exige concept, equations, values, initial, duration, step, \
             method e samples",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(conceito) = catalogo::conceito(&parsed.concept) else {
            return conceito_desconhecido(request_id, &parsed.concept);
        };
        match corrida_sistema::executar(
            &conceito,
            &parsed.equations,
            &parsed.values,
            &parsed.initial,
            parsed.duration,
            parsed.step,
            parsed.method,
            parsed.samples,
            self.oraculo(),
        ) {
            Ok(resultado) => JsonRpcResponse::success(request_id, json!(resultado)),
            Err(erro) => erro_de_sistema(request_id, &erro),
        }
    }
}

/// Cada recusa da forma vetorial vira um `reason` proprio.
///
/// A de PAREAMENTO e' a que mais importa: ela nao e' uma limitacao a contornar,
/// e' o metodo nao estar definido sem o par posicao/velocidade. Recusar dizendo
/// isso e' melhor que aceitar e integrar outra coisa (`arquitetura/34` §13.3).
fn erro_de_sistema(request_id: Option<Value>, erro: &ErroDeSistemaExec) -> JsonRpcResponse {
    let (mensagem, dados) = match erro {
        ErroDeSistemaExec::FormaNaoEhSistema => (
            "este conceito nao e de sistema: ele tem uma equacao, nao varias",
            json!({ "reason": "notASystem" }),
        ),
        ErroDeSistemaExec::NaoChecada => (
            "as equacoes ainda nao passaram na checagem contra o conceito",
            json!({ "reason": "notChecked" }),
        ),
        ErroDeSistemaExec::SemValor { grandeza } => (
            "falta o valor de uma grandeza ligada",
            json!({ "reason": "missingValue", "quantity": grandeza }),
        ),
        ErroDeSistemaExec::Integrador(ErroDeSistema::SemPareamento) => (
            "este conceito nao declara quais componentes sao posicao e velocidade, \
             entao o metodo simpletico nao se aplica a ele",
            json!({ "reason": "noPairing" }),
        ),
        ErroDeSistemaExec::Integrador(ErroDeSistema::EstadoIncompleto { esperado, recebido }) => (
            "o estado inicial precisa de um valor por componente",
            json!({ "reason": "incompleteState", "expected": esperado, "received": recebido }),
        ),
        ErroDeSistemaExec::Integrador(ErroDeSistema::Corrida(interno)) => {
            return erro_de_execucao(request_id, &ErroDeExecucao::Integrador(interno.clone()));
        }
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, Some(dados)),
    )
}

/// Cada recusa vira um `reason` proprio.
///
/// A UI casa por `reason`, nunca por texto — e as duas de ESCALA carregam os
/// numeros junto, porque recusar sem dizer quantos passos seriam e' uma parede.
/// Com o numero, o autor decide reduzir o passo ou a duracao.
fn erro_de_execucao(request_id: Option<Value>, erro: &ErroDeExecucao) -> JsonRpcResponse {
    let (mensagem, dados) = match erro {
        ErroDeExecucao::NaoChecada => (
            "a formula ainda nao passou na checagem contra o conceito",
            json!({ "reason": "notChecked" }),
        ),
        ErroDeExecucao::FormaNaoIntegra => (
            "este conceito nao tem trajetoria: ele e avaliado, nao integrado",
            json!({ "reason": "notIntegrable" }),
        ),
        ErroDeExecucao::SemValor { grandeza } => (
            "falta o valor de uma grandeza ligada",
            json!({ "reason": "missingValue", "quantity": grandeza }),
        ),
        ErroDeExecucao::Integrador(ErroDeCorrida::PassoInvalido) => (
            "o passo precisa ser um numero positivo",
            json!({ "reason": "invalidStep" }),
        ),
        ErroDeExecucao::Integrador(ErroDeCorrida::DuracaoInvalida) => (
            "a duracao precisa ser um numero positivo",
            json!({ "reason": "invalidDuration" }),
        ),
        ErroDeExecucao::Integrador(ErroDeCorrida::FaltaDerivadaInicial) => (
            "esta forma e de segunda ordem e precisa da velocidade inicial",
            json!({ "reason": "missingInitialDerivative" }),
        ),
        ErroDeExecucao::Integrador(ErroDeCorrida::CorridaLongaDemais { passos, teto }) => (
            "a corrida pedida e longa demais",
            json!({ "reason": "tooManySteps", "steps": passos, "limit": teto }),
        ),
        ErroDeExecucao::Integrador(ErroDeCorrida::Divergiu { passo, t }) => (
            "a integracao divergiu: o estado deixou de ser um numero",
            json!({ "reason": "diverged", "step": passo, "t": t }),
        ),
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, Some(dados)),
    )
}

fn invalid(request_id: Option<Value>, mensagem: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, None),
    )
}

fn conceito_desconhecido(request_id: Option<Value>, id: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::InvalidParams,
            "conceito nao existe no catalogo",
            Some(json!({ "concept": id })),
        ),
    )
}

fn erro_de_avaliacao(request_id: Option<Value>, erro: &ErroAvaliacao) -> JsonRpcResponse {
    let (mensagem, dados) = match erro {
        ErroAvaliacao::NaoChecada => (
            "a formula ainda nao passou na checagem contra o conceito",
            json!({ "reason": "notChecked" }),
        ),
        ErroAvaliacao::SemValor { grandeza } => (
            "falta o valor de uma grandeza ligada",
            json!({ "reason": "missingValue", "quantity": grandeza }),
        ),
        // Campo vazio e' recusa, nunca zero implicito: a IDE nao preenche por
        // voce (arquitetura/34 §2.1).
        ErroAvaliacao::NaoFinito { valor } => (
            "o resultado nao e um numero utilizavel",
            json!({ "reason": "notFinite", "value": valor }),
        ),
    };
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, mensagem, Some(dados)),
    )
}
