//! Types for the EXECUTION side of the `sim.*` domain: avaliar, estimar,
//! integrar e guardar.
//!
//! Separado do [`super::sim`] em 2026-09-06 por RESPONSABILIDADE, e o corte e' o
//! mesmo que o core ja' tem entre `formula.rs` e `corrida.rs`:
//!
//! ```text
//! sim.rs         AUTORIA    o catalogo, o conceito, a formula e a checagem —
//!                           o que a tela monta enquanto o autor digita
//! sim_corrida.rs EXECUCAO   avaliar, estimar, integrar e guardar — o que
//!                           acontece quando ele manda rodar
//! ```
//!
//! Sao dois momentos diferentes e dois motivos de mudanca diferentes: a checagem
//! muda quando o catalogo ganha uma regra, a corrida muda quando o motor ganha
//! um metodo.
//!
//! A decisao que governa os dois arquivos continua a mesma (`arquitetura/34`
//! §2.1): **nada e' adivinhado**. Todo campo que decide um resultado e'
//! obrigatorio — nao ha' metodo padrao, passo padrao nem amostragem padrao.

use serde::{Deserialize, Serialize};

use super::sim::{SimBinding, SimComponentFormula, SimInvariantDrift};

/// Params of `sim.evaluate`: run the ALGEBRAIC form once.
///
/// Every field that decides the result is required. There is no default method,
/// no default step and no default sampling anywhere in this domain (§2.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimEvaluateParams {
    /// The concept the user picked.
    pub concept: String,
    /// The formula as typed.
    pub formula: String,
    /// What each variable is.
    pub bindings: Vec<SimBinding>,
    /// The value the user gave each quantity, by `SimQuantity::id`.
    pub values: Vec<SimValue>,
}

/// One value the user filled in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimValue {
    /// `SimQuantity::id`.
    pub quantity: String,
    /// The number.
    pub value: f64,
}

/// One line of the substitution trail.
///
/// This is the honest form of "show the reasoning": it is not algebra, it is
/// the computation that ran, written out. No auditable tool narrates the
/// algebraic solution of an ODE (`roadmaps/31` §13), so the IDE shows what it
/// did rather than inventing a plausible derivation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimStep {
    /// What this line is: `formula`, `binding`, `substitution`, `result`.
    pub kind: String,
    /// The line as the panel shows it.
    pub text: String,
}

/// Result of `sim.evaluate`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimEvaluateResult {
    /// The number.
    pub value: f64,
    /// The trail that produced it, for the calculation panel.
    pub steps: Vec<SimStep>,
    /// The unit label the concept declares for its result, when it declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// O metodo de integracao, escolhido pelo USUARIO.
///
/// Nao ha' padrao e nao ha' sugestao: o campo comeca vazio e a corrida nao parte
/// sem ele (`arquitetura/34` §2.1). O motivo nao e' rigor — e' que o metodo
/// **muda o numero**. Medido em `roadmaps/31` §11.1, no oscilador amortecido
/// com `dt=0.1`: Euler explicito erra por 3,11 onde a resposta e' -0,276, e o
/// RK4 erra por 4,91e-5. Onze vezes a resposta contra cinco casas certas, com o
/// mesmo passo.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimMethod {
    /// Euler explicito. Ordem 1, medida. O mais simples de explicar e o pior de
    /// usar: ele injeta energia em sistema oscilante.
    Euler,
    /// Euler semi-implicito (simpletico). Ordem 1, e conserva melhor a energia
    /// de um oscilador que o explicito com o mesmo passo.
    EulerSymplectic,
    /// Runge-Kutta classico de quarta ordem. Ordem 4 confirmada por medicao na
    /// faixa onde ela e' mensuravel (`roadmaps/31` §11.3).
    Rk4,
}

/// Uma amostra da trilha: o estado num instante.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSample {
    /// Instante.
    pub t: f64,
    /// Posicao (ou a primeira variavel de estado).
    pub y: f64,
    /// Derivada primeira, quando a forma tem uma.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dy: Option<f64>,
}

/// Params de `sim.run`: integrar a EDO que o usuario escreveu.
///
/// **Todo campo que decide o resultado e' obrigatorio.** Nao ha' metodo padrao,
/// passo padrao nem amostragem padrao — a IDE calcula e mostra, o usuario
/// escolhe (`arquitetura/34` §2.1, revisado em 2026-09-05).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimRunParams {
    /// O conceito escolhido.
    pub concept: String,
    /// A formula da derivada, como o usuario escreveu.
    pub formula: String,
    /// O que cada variavel da formula e'.
    pub bindings: Vec<SimBinding>,
    /// Os valores dos parametros (massa, constante elastica, ...).
    pub values: Vec<SimValue>,
    /// Estado inicial: `y(0)` e, na segunda ordem, `y'(0)`.
    pub initial: SimInitial,
    /// Ate' quando integrar.
    pub duration: f64,
    /// O passo. Escolhido pelo usuario, sem padrao.
    pub step: f64,
    /// O metodo. Escolhido pelo usuario, sem padrao.
    pub method: SimMethod,
    /// Quantos pontos guardar na trilha. Escolhido pelo usuario.
    ///
    /// Guardar tudo e' inviavel por ordem de grandeza em corrida grande: 40
    /// milhoes de passos sao 1,28 GB de trilha (`roadmaps/31` §14). A IDE mostra
    /// o custo de cada escolha e o usuario decide.
    pub samples: usize,
}

/// O estado em t = 0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimInitial {
    /// `y(0)`.
    pub y: f64,
    /// `y'(0)`. Exigido pela forma de segunda ordem, ignorado pela primeira.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dy: Option<f64>,
}

/// O que a IDE sabe sobre a exatidao desta corrida.
///
/// So' existe quando o conceito tem solucao fechada — 88 dos 100 catalogados
/// (`arquitetura/34` apendice A). E' esta struct que transforma "exato e
/// preciso" de promessa em numero: sem ela a IDE mostraria uma curva bonita sem
/// dizer o quanto ela erra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimAccuracy {
    /// O valor verdadeiro no instante final.
    pub exact: f64,
    /// O que a integracao numerica produziu.
    pub numeric: f64,
    /// A diferenca absoluta. E' este o numero que separa resultado de animacao.
    pub absolute_error: f64,
    /// `|numerico - exato| / |exato|`, quando `exato` nao e' zero.
    ///
    /// Existe porque so' o absoluto engana nos dois sentidos, e isso foi medido
    /// (`roadmaps/31` §19.0): um erro absoluto de `1,474` sobre um valor de
    /// `83.178` e' uma integracao EXCELENTE (1,8e-5 relativo), e o mesmo `1,474`
    /// ao lado de um resultado de `0,032` seria catastrofe. A tela mostrava os
    /// dois numeros sem distinguir e deixava a conta para quem le.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_error: Option<f64>,
    /// DE ONDE veio o valor exato. O campo que faltava, e cuja ausencia fez
    /// esta coluna mentir — ver [`SimAccuracySource`].
    pub source: SimAccuracySource,
    /// Quem resolveu, quando foi o oraculo: `"SymPy 1.14.0"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solved_by: Option<String>,
    /// A solucao fechada que o oraculo devolveu, para a tela mostrar a resposta
    /// verdadeira ao lado do numero calculado (`arquitetura/34` §7.3, item 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_form: Option<String>,
}

/// De onde a IDE tirou o valor "exato" — e por que este campo existe.
///
/// **Medido em 2026-09-06** (`roadmaps/31` §19.0), contra o binario real: com
/// `-(k/m)*x - 2*(c/m)*v` no oscilador amortecido a IDE acusava erro de 2,5e-2
/// numa integracao correta ate' 3,2e-7 — **setenta e oito mil vezes** o que
/// reportava. A causa nao era o numero: era a AUSENCIA deste campo. A coluna
/// vinha de `catalogo::exata(conceito.id)` e nao olhava a formula digitada, e o
/// `sim.checkFormula` aprova a formula porque confere ligacao, nao fisica.
///
/// A regra da casa e' que **numero sem procedencia mente**, e quem a estava
/// quebrando era justamente a coluna que existe para dar procedencia. O
/// conserto nao e' esconder o numero: e' dizer de onde ele vem.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimAccuracySource {
    /// A solucao fechada do CONCEITO, escrita a mao no catalogo.
    ///
    /// Ela responde pela equacao canonica do conceito — **nao** pela que o
    /// usuario digitou. Se as duas coincidem, o erro e' o erro; se nao, o
    /// numero e' a distancia ate' outra pergunta. A tela diz isso.
    Concept,
    /// A solucao fechada da equacao que o usuario DIGITOU, resolvida pelo
    /// oraculo externo.
    ///
    /// E' a unica procedencia que responde a pergunta que o usuario fez.
    Oracle,
}

/// Resultado de `sim.run`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimRunResult {
    /// Quantos passos foram dados de fato.
    pub steps_taken: u64,
    /// Quantos passos ha' entre duas amostras guardadas.
    ///
    /// Vai a tela: uma tabela amostrada PARECE completa, e a IDE diz "mostrando
    /// 1 de cada N" em vez de entregar algo que parece a corrida inteira. Mesmo
    /// idioma do `$sample` da etapa 27.
    pub sample_every: u64,
    /// A trilha amostrada.
    pub trail: Vec<SimSample>,
    /// A comparacao com a verdade, quando ela existe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<SimAccuracy>,
    /// Por que o oraculo NAO respondeu, quando ele nao respondeu.
    ///
    /// Frase para a tela, nao codigo de erro: "o `SymPy` nao esta instalado",
    /// "ele nao soube resolver esta equacao", "ele passou do tempo". Existe
    /// porque a `arquitetura/34` §7 exige que a coluna do erro, ao sumir ou
    /// mudar de procedencia, **diga que mudou e por que** — em vez de degradar
    /// em silencio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle_note: Option<String>,
    /// O nome do metodo e o passo, para a tela nunca mostrar numero sem
    /// procedencia.
    pub method_label: String,
}

/// Params de `sim.estimate`: quanto esta corrida vai custar?
///
/// Existe porque a `arquitetura/34` §6 exige que a IDE **mostre** o custo antes
/// de rodar, e porque contar passos e' regra de negocio — a UI nao faz conta
/// (`ARCHITECTURE.md` §2). Ela pergunta e desenha a resposta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimEstimateParams {
    /// Ate' quando integrar.
    pub duration: f64,
    /// O passo.
    pub step: f64,
    /// Quantos pontos guardar.
    pub samples: usize,
}

/// Resultado de `sim.estimate`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimEstimateResult {
    /// Quantos passos a corrida daria.
    pub steps: u64,
    /// Se ela passa do teto e por isso seria recusada.
    pub too_many: bool,
    /// O teto, para a tela poder dizer de quanto ele e.
    pub limit: u64,
    /// De quantos em quantos passos a trilha guardaria um ponto.
    pub sample_every: u64,
    /// Quantos bytes a trilha ocuparia se guardasse TUDO.
    ///
    /// Vai a tela ao lado do que ela vai ocupar de fato. E' o numero que mostra
    /// por que a amostragem existe: 40 milhoes de passos sao 1,28 GB
    /// (`roadmaps/31` §14).
    pub full_trail_bytes: u64,
    /// Quantos bytes a trilha AMOSTRADA ocupa.
    pub sampled_trail_bytes: u64,
    /// Se compilar valeria a pena nesta escala.
    ///
    /// O ponto de virada e 21,5 milhoes de avaliacoes, medido (`roadmaps/31`
    /// §9.3). A IDE MOSTRA; quem escolhe o motor e o usuario (§2.1).
    pub compiling_would_pay: bool,
}

/// Uma simulacao salva: tudo que o autor montou, e nada do que a maquina
/// produziu.
///
/// **O resultado NAO entra aqui**, e essa e' a decisao mais importante do
/// formato (`arquitetura/34` §9). E' a licao precisa do `.ipynb`, que e' texto,
/// e' diffavel, e mesmo assim falha — porque mistura o que o usuario escreveu
/// com o que a maquina produziu: diff ilegivel, metadado que muda sozinho,
/// imagem em base64 dentro do diff, e conflito de merge que quebra o JSON a
/// ponto de o arquivo nao abrir mais. Nasceu o `nbdime` so' para remediar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSaved {
    /// Nome que o autor deu.
    pub name: String,
    /// O conceito escolhido.
    pub concept: String,
    /// A formula, como ele escreveu.
    pub formula: String,
    /// O que cada variavel e.
    pub bindings: Vec<SimBinding>,
    /// Os valores dos parametros.
    pub values: Vec<SimValue>,
    /// O estado inicial, nas formas que integram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<SimInitial>,
    /// O metodo escolhido, nas formas que integram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<SimMethod>,
    /// O passo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// A duracao.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Quantos pontos guardar na trilha.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub samples: Option<usize>,
}

/// Params de `sim.save`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSaveParams {
    /// A simulacao a gravar.
    pub simulation: SimSaved,
}

/// Params de `sim.forget`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimForgetParams {
    /// O nome da simulacao a remover.
    pub name: String,
}

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
