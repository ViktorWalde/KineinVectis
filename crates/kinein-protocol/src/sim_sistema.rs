//! Os tipos da forma VETORIAL: `dY/dt = F(t, Y)`.
//!
//! Arquivo proprio desde 2026-09-10, e o corte e' o MESMO que o core ja' tem
//! entre `corrida.rs` e `corrida_sistema.rs`: **as regras sao outras**. Na
//! forma escalar ha' uma formula, um estado inicial e um valor exato; aqui ha'
//! `n` de cada, mais um pareamento que decide se o metodo simpletico existe e
//! um invariante que e' o unico sinal de exatidao que um sistema caotico
//! admite.
//!
//! Ele saiu do [`super::sim_corrida`] quando a catraca reprovou aquele arquivo
//! em 509/500 — e a pergunta que o gate manda fazer ("o que esta misturado
//! aqui?") tinha resposta pronta, escrita no core desde 2026-09-06.

use serde::{Deserialize, Serialize};

use super::sim::{SimComponentFormula, SimInvariantDrift};
use super::sim_corrida::{SimAccuracySource, SimDimensionCheck, SimMethod, SimValue};

/// Uma amostra da trilha de um SISTEMA: o estado inteiro num instante.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSystemSample {
    /// Instante.
    pub t: f64,
    /// Um valor por componente, na ORDEM que o conceito declara.
    pub values: Vec<f64>,
}

/// Params de `sim.runSystem`: integrar `dY/dt = F(t, Y)`.
///
/// **Todo campo que decide o resultado e' obrigatorio**, como em toda a familia
/// (`arquitetura/34` §2.1): nao ha' metodo padrao, passo padrao, amostragem
/// padrao nem estado inicial padrao.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimRunSystemParams {
    /// O conceito escolhido.
    pub concept: String,
    /// Uma formula por componente, cada uma com a sua ligacao.
    pub equations: Vec<SimComponentFormula>,
    /// Os valores dos parametros (massa, G, comprimento, ...).
    pub values: Vec<SimValue>,
    /// Estado inicial: um numero por componente, na ordem do conceito.
    pub initial: Vec<f64>,
    /// Ate' quando integrar.
    pub duration: f64,
    /// O passo.
    pub step: f64,
    /// O metodo.
    ///
    /// `eulerSymplectic` so' e' aceito quando o conceito DECLARA o pareamento
    /// posicao/velocidade; sem ele o metodo nao esta' definido e a corrida e'
    /// recusada com o motivo (`arquitetura/34` §13.3).
    pub method: SimMethod,
    /// Quantos pontos guardar na trilha.
    pub samples: usize,
}

/// Resultado de `sim.runSystem`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimRunSystemResult {
    /// Quantos passos foram dados.
    pub steps_taken: u64,
    /// Quantos passos ha' entre duas amostras guardadas.
    pub sample_every: u64,
    /// A trilha amostrada.
    pub trail: Vec<SimSystemSample>,
    /// A comparacao com a verdade, quando o conceito tem solucao fechada.
    ///
    /// Um erro por componente, mais a norma do maximo — que e' a grandeza em
    /// que a ordem de convergencia e' medida, porque grandeza DERIVADA cancela
    /// erro: medido em 2026-09-06, o raio da orbita circular da' ordem 5,00
    /// para um metodo de ordem 4 (`../roadmaps/31` §19.1.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<SimSystemAccuracy>,
    /// O que aconteceu com cada invariante declarado.
    ///
    /// Existe mesmo quando `accuracy` e' `None`, e e' o unico sinal de exatidao
    /// que um sistema caotico admite.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<SimInvariantDrift>,
    /// Por que a comparacao nao tem a procedencia da equacao DIGITADA.
    ///
    /// Mesma funcao do `oracle_note` da forma escalar: dizer que mudou, e por
    /// que, em vez de degradar calado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle_note: Option<String>,
    /// Um veredito de UNIDADE por componente. Vazio quando a IDE nao checou.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dimensions: Vec<SimDimensionCheck>,
    /// O nome do metodo e o passo.
    pub method_label: String,
}

/// A comparacao do estado final com a verdade, componente a componente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSystemAccuracy {
    /// DE ONDE veio o estado verdadeiro.
    ///
    /// Hoje sempre [`SimAccuracySource::Concept`]: o oraculo resolve EDO
    /// escalar, e resolver um SISTEMA com `dsolve` nao foi medido — entrar sem
    /// medir seria o oposto do que este dominio faz. A consequencia e' que a
    /// tela do sistema carrega a mesma ressalva da forma escalar sem oraculo:
    /// este valor responde pela equacao do CONCEITO, nao pela que voce digitou.
    pub source: SimAccuracySource,
    /// O estado verdadeiro no instante final.
    pub exact: Vec<f64>,
    /// O que a integracao produziu.
    pub numeric: Vec<f64>,
    /// `|numerico - exato|` por componente.
    pub absolute_error: Vec<f64>,
    /// A norma do MAXIMO do erro — a grandeza do estudo de convergencia.
    pub max_error: f64,
}
