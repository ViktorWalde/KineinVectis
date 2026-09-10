//! A CORRIDA DE SISTEMA: liga as `n` formulas do usuario, o integrador vetorial,
//! o oraculo e os invariantes.
//!
//! ```text
//! formula.rs      checa cada formula contra o conceito, uma por componente
//! sistema.rs      anda no tempo com o metodo e o passo que o usuario escolheu
//! exata.rs        entrega a solucao fechada, quando ela existe
//! invariante.rs   entrega a grandeza conservada, quando o conceito declara
//! ```
//!
//! ## O que muda em relacao a' cola escalar
//!
//! Na forma escalar ha' UMA formula e UM vetor de avaliacao. Aqui ha' `n` de
//! cada, e a armadilha da ordem alfabetica do avaliador (ADR-0006, armadilha 1)
//! aparece `n` vezes. A defesa e' a mesma e vale por construcao: **cada vetor de
//! avaliacao e' montado pela LIGACAO daquela formula**, percorrendo o
//! `var_names()` dela e buscando o papel de cada nome. Nada e' posicional.
//!
//! E ha' uma armadilha nova, que a forma escalar nao tinha: **a formula do
//! componente `i` pode mencionar qualquer componente**, nao so' o proprio. Numa
//! orbita, `vx'` depende de `x` e de `y`. Por isso a ligacao aponta para
//! componentes por ID, e o vetor de avaliacao le' o estado inteiro a cada passo.

use std::collections::{BTreeMap, BTreeSet};

use exmex::prelude::*;
use kinein_protocol::{
    SimAccuracySource, SimCheckIssue, SimCheckResult, SimCheckSystemResult, SimComponentCheck,
    SimComponentFormula, SimConcept, SimForm, SimInvariantDrift, SimMethod, SimRunSystemResult,
    SimSystemAccuracy, SimValue,
};

use super::corrida::GRANDEZA_TEMPO;
use super::sistema::{CorridaSistema, ErroDeSistema};
use super::{catalogo, corrida, formula};

/// De onde sai o valor de cada posicao de um vetor de avaliacao.
enum Fonte {
    /// Um parametro: o mesmo numero em todo passo.
    Fixo(f64),
    /// Um COMPONENTE do estado, pelo indice dele na ordem do conceito.
    Componente(usize),
    /// O tempo.
    Tempo,
}

/// O que impede uma corrida de sistema de sair.
#[derive(Debug, Clone, PartialEq)]
pub enum ErroDeSistemaExec {
    /// O conceito escolhido nao e' da forma vetorial.
    FormaNaoEhSistema,
    /// Alguma formula nao passou na checagem contra o conceito.
    NaoChecada,
    /// Uma grandeza ligada ficou sem valor.
    SemValor {
        /// A grandeza em branco.
        grandeza: String,
    },
    /// A integracao recusou ou parou.
    Integrador(ErroDeSistema),
}

/// Confronta as `n` formulas com o conceito, e devolve TODOS os problemas.
///
/// **As regras nao sao as da forma escalar, e a diferenca e' de desenho.** Na
/// escalar ha' uma formula, e toda grandeza obrigatoria do conceito tem de estar
/// ligada NELA. Num sistema isso seria absurdo: a equacao `x' = vx` da orbita
/// nao menciona `mu`, e exigir que ela mencionasse tornaria a forma inutilizavel.
///
/// ```text
/// alvo valido de ligacao   grandeza do conceito | COMPONENTE de estado | o tempo
/// grandeza obrigatoria     tem de estar ligada em ALGUMA equacao, nao em todas
/// componente sem formula   e' problema do SISTEMA, e sai em campo proprio
/// ```
#[must_use]
pub fn checar(conceito: &SimConcept, equacoes: &[SimComponentFormula]) -> SimCheckSystemResult {
    let declarados: Vec<&str> = conceito.components.iter().map(|c| c.id.as_str()).collect();
    let mut alvos: BTreeSet<&str> = conceito.quantities.iter().map(|q| q.id.as_str()).collect();
    alvos.extend(declarados.iter().copied());
    alvos.insert(GRANDEZA_TEMPO);

    let mut componentes = Vec::with_capacity(equacoes.len());
    let mut desconhecidos = Vec::new();
    let mut vistos: Vec<&str> = Vec::new();
    let mut ligadas_no_sistema: BTreeSet<&str> = BTreeSet::new();

    for equacao in equacoes {
        let alvo = equacao.component.as_str();
        if !declarados.contains(&alvo) {
            desconhecidos.push(alvo.to_string());
            continue;
        }
        vistos.push(alvo);
        componentes.push(SimComponentCheck {
            component: alvo.to_string(),
            result: checar_uma(
                &equacao.formula,
                &equacao.bindings,
                &alvos,
                &mut ligadas_no_sistema,
            ),
        });
    }

    let faltando: Vec<String> = declarados
        .iter()
        .filter(|id| !vistos.contains(id))
        .map(|id| (*id).to_string())
        .collect();

    // A grandeza obrigatoria e' cobrada do SISTEMA inteiro, e o problema entra
    // na primeira equacao para a tela ter onde mostra-lo.
    let mut faltam_grandezas = Vec::new();
    for grandeza in &conceito.quantities {
        if grandeza.required && !ligadas_no_sistema.contains(grandeza.id.as_str()) {
            faltam_grandezas.push(SimCheckIssue::MissingQuantity {
                quantity: grandeza.id.clone(),
                label: grandeza.label.clone(),
            });
        }
    }
    if let Some(primeira) = componentes.first_mut() {
        for problema in faltam_grandezas {
            primeira.result.ok = false;
            primeira.result.caveat = None;
            primeira.result.issues.push(problema);
        }
    }

    let ok = desconhecidos.is_empty()
        && faltando.is_empty()
        && !componentes.is_empty()
        && componentes.iter().all(|c| c.result.ok);
    SimCheckSystemResult {
        ok,
        components: componentes,
        missing: faltando,
        unknown: desconhecidos,
        caveat: ok.then(|| formula::AVISO.to_string()),
    }
}

/// A checagem de UMA equacao do sistema.
///
/// Anota em `ligadas` toda grandeza que esta equacao ligou, para a cobranca das
/// obrigatorias ser feita no sistema inteiro.
fn checar_uma<'a>(
    texto: &str,
    ligacoes: &'a [kinein_protocol::SimBinding],
    alvos: &BTreeSet<&str>,
    ligadas: &mut BTreeSet<&'a str>,
) -> SimCheckResult {
    let variaveis = match formula::inspecionar(texto) {
        Ok(v) => v,
        Err(message) => {
            return SimCheckResult {
                ok: false,
                issues: vec![SimCheckIssue::ParseFailed { message }],
                variables: Vec::new(),
                caveat: None,
            };
        }
    };
    let na_formula: BTreeSet<&str> = variaveis.iter().map(String::as_str).collect();
    let mut problemas = Vec::new();
    let mut por_variavel: BTreeMap<&str, &str> = BTreeMap::new();
    let mut usos: BTreeMap<&str, usize> = BTreeMap::new();

    for ligacao in ligacoes {
        let (variavel, grandeza) = (ligacao.variable.as_str(), ligacao.quantity.as_str());
        if !alvos.contains(grandeza) {
            problemas.push(SimCheckIssue::UnknownQuantity {
                variable: variavel.to_string(),
                quantity: grandeza.to_string(),
            });
            continue;
        }
        if !na_formula.contains(variavel) {
            problemas.push(SimCheckIssue::VariableNotInFormula {
                variable: variavel.to_string(),
            });
            continue;
        }
        por_variavel.insert(variavel, grandeza);
        *usos.entry(grandeza).or_default() += 1;
        ligadas.insert(grandeza);
    }
    for (grandeza, quantas) in &usos {
        if *quantas > 1 {
            problemas.push(SimCheckIssue::DuplicateQuantity {
                quantity: (*grandeza).to_string(),
            });
        }
    }
    // Variavel sem papel — e e' tambem por aqui que o `pi` minusculo aparece
    // como pergunta em vez de incognita silenciosa (ADR-0006, armadilha 2).
    for variavel in &variaveis {
        if !por_variavel.contains_key(variavel.as_str()) {
            problemas.push(SimCheckIssue::UnboundVariable {
                variable: variavel.clone(),
            });
        }
    }

    let ok = problemas.is_empty();
    SimCheckResult {
        ok,
        issues: problemas,
        variables: variaveis,
        caveat: ok.then(|| formula::AVISO.to_string()),
    }
}

/// Monta os `n` planos de avaliacao e roda a integracao.
#[allow(clippy::too_many_arguments)]
pub fn executar(
    conceito: &SimConcept,
    equacoes: &[SimComponentFormula],
    valores: &[SimValue],
    inicial: &[f64],
    duracao: f64,
    passo: f64,
    metodo: SimMethod,
    amostras: usize,
    oraculo: &super::oraculo::Config,
) -> Result<SimRunSystemResult, ErroDeSistemaExec> {
    if conceito.form != SimForm::OdeSystem {
        return Err(ErroDeSistemaExec::FormaNaoEhSistema);
    }
    if !checar(conceito, equacoes).ok {
        return Err(ErroDeSistemaExec::NaoChecada);
    }

    let indice_de: BTreeMap<&str, usize> = conceito
        .components
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect();
    let valor_de: BTreeMap<&str, f64> = valores
        .iter()
        .map(|v| (v.quantity.as_str(), v.value))
        .collect();

    // Uma expressao e um plano por COMPONENTE, na ordem que o conceito declara
    // — nunca na ordem em que as equacoes chegaram.
    let mut expressoes = Vec::with_capacity(conceito.components.len());
    let mut planos = Vec::with_capacity(conceito.components.len());
    for componente in &conceito.components {
        let equacao = equacoes
            .iter()
            .find(|e| e.component == componente.id)
            .ok_or(ErroDeSistemaExec::NaoChecada)?;
        let expressao =
            exmex::parse::<f64>(&equacao.formula).map_err(|_| ErroDeSistemaExec::NaoChecada)?;
        let grandeza_de: BTreeMap<&str, &str> = equacao
            .bindings
            .iter()
            .map(|l| (l.variable.as_str(), l.quantity.as_str()))
            .collect();
        let mut plano = Vec::with_capacity(expressao.var_names().len());
        for nome in expressao.var_names() {
            let grandeza = grandeza_de
                .get(nome.as_str())
                .copied()
                .ok_or(ErroDeSistemaExec::NaoChecada)?;
            plano.push(if grandeza == GRANDEZA_TEMPO {
                Fonte::Tempo
            } else if let Some(indice) = indice_de.get(grandeza) {
                Fonte::Componente(*indice)
            } else {
                Fonte::Fixo(valor_de.get(grandeza).copied().ok_or_else(|| {
                    ErroDeSistemaExec::SemValor {
                        grandeza: grandeza.to_string(),
                    }
                })?)
            });
        }
        planos.push(plano);
        expressoes.push(expressao);
    }

    // Os buffers sao alocados UMA VEZ e reaproveitados: alocar por passo
    // dominaria o custo numa corrida de milhoes de passos.
    let mut buffers: Vec<Vec<f64>> = expressoes
        .iter()
        .map(|e| vec![0.0f64; e.var_names().len()])
        .collect();
    let campo = |t: f64, estado: &[f64], saida: &mut [f64]| -> bool {
        for (indice, (expressao, plano)) in expressoes.iter().zip(&planos).enumerate() {
            let buffer = &mut buffers[indice];
            for (slot, fonte) in buffer.iter_mut().zip(plano) {
                *slot = match fonte {
                    Fonte::Fixo(v) => *v,
                    Fonte::Componente(i) => estado[*i],
                    Fonte::Tempo => t,
                };
            }
            match expressao.eval(buffer) {
                Ok(v) => saida[indice] = v,
                Err(_) => return false,
            }
        }
        true
    };

    let corrida = CorridaSistema {
        passo,
        duracao,
        metodo,
        amostras,
        inicial: inicial.to_vec(),
        pareamento: conceito.pairing.clone(),
    };
    let saida = corrida
        .integrar(conceito.components.len(), campo)
        .map_err(ErroDeSistemaExec::Integrador)?;

    let mapa: BTreeMap<String, f64> = valor_de
        .iter()
        .map(|(k, v)| ((*k).to_string(), *v))
        .collect();
    let accuracy = exatidao(conceito, &mapa, inicial, saida.t_final, &saida.estado_final);
    let dimensions = conferir_unidades(oraculo, conceito, equacoes);
    Ok(SimRunSystemResult {
        steps_taken: saida.passos,
        sample_every: saida.a_cada,
        // O instante ALCANCADO, nao o pedido: ver `SaidaSistema::t_final`.
        // A nota acompanha o numero: sem ela, "exato" afirmaria o que a IDE
        // ainda nao sabe conferir num sistema.
        oracle_note: accuracy.as_ref().map(|_| RESSALVA_DO_CONCEITO.to_owned()),
        accuracy,
        dimensions,
        invariants: derivas(conceito, &mapa, inicial, &saida.estado_final),
        trail: saida.trilha,
        method_label: super::integrador::rotulo(metodo, passo),
    })
}

/// A ressalva que acompanha todo `exato` de sistema, hoje.
///
/// Ela existe pelo mesmo motivo que o `SimAccuracySource`: a coluna vem da
/// solucao fechada do CONCEITO, e o `sim.checkSystem` aprova as equacoes por
/// LIGACAO, nao por fisica. Se o autor escreveu outra equacao, este numero e' a
/// distancia ate' outra pergunta.
/// Confere a UNIDADE de cada uma das `n` equacoes, numa ida so'.
///
/// **Aqui a checagem vale mais que na forma escalar**, e a razao e' aritmetica:
/// sao `n` equacoes para escrever, e cada uma tem o proprio lado esquerdo. A
/// derivada de uma POSICAO e' uma velocidade e a de uma VELOCIDADE e' uma
/// aceleracao — trocar as duas de lugar passa no checador de ligacao, porque
/// ele confere ligacao e nao fisica, e nao passa aqui.
///
/// Vazio quando a ferramenta falta: a IDE diz que NAO checou, em vez de parar
/// de checar em silencio.
fn conferir_unidades(
    config: &super::oraculo::Config,
    conceito: &SimConcept,
    equacoes: &[SimComponentFormula],
) -> Vec<kinein_protocol::SimDimensionCheck> {
    let das_grandezas = corrida::unidades_do_conceito(conceito);
    let mut pedidos = Vec::with_capacity(equacoes.len());
    for componente in &conceito.components {
        let Some(equacao) = equacoes.iter().find(|e| e.component == componente.id) else {
            continue;
        };
        let unidades = equacao
            .bindings
            .iter()
            .map(|l| {
                let alvo = l.quantity.as_str();
                // O alvo pode ser um COMPONENTE do estado, uma grandeza ou o
                // tempo — e a unidade de cada um mora em lugar diferente.
                let unidade = conceito
                    .components
                    .iter()
                    .find(|c| c.id == alvo)
                    .map_or_else(
                        || corrida::unidade_da_grandeza(&das_grandezas, alvo),
                        |c| c.unit.clone(),
                    );
                (l.variable.clone(), unidade)
            })
            .collect();
        pedidos.push(super::oraculo::PedidoDimensao {
            label: componente.id.clone(),
            formula: equacao.formula.clone(),
            unidades,
            // A forma vetorial e' de PRIMEIRA ordem: cada equacao e' a derivada
            // do seu componente, e nunca a segunda (`arquitetura/34` §13.1).
            esquerda: corrida::lado_esquerdo(&componente.unit, 1),
        });
    }
    if pedidos.is_empty() {
        return Vec::new();
    }
    super::oraculo::perguntar(
        config,
        &super::oraculo::Consulta {
            edo: None,
            dimensoes: pedidos,
        },
    )
    .map(|r| {
        r.dimensoes
            .iter()
            .map(corrida::veredito_para_protocolo)
            .collect()
    })
    .unwrap_or_default()
}

const RESSALVA_DO_CONCEITO: &str = "O valor exato vem da solução do conceito, não das equações \
     que você escreveu: a IDE ainda não sabe resolver um sistema para conferir. As grandezas \
     conservadas abaixo, essas sim, são medidas na SUA trajetória.";

/// A comparacao com a verdade, quando o conceito tem solucao fechada.
///
/// O `max_error` e' a norma do MAXIMO, e ela existe porque e' a grandeza em que
/// a ordem de convergencia pode ser medida: grandeza DERIVADA cancela erro.
/// Medido em 2026-09-06, o raio da orbita circular da' ordem 5,00 para um metodo
/// de ordem 4 (`../roadmaps/31` §19.1.2).
fn exatidao(
    conceito: &SimConcept,
    valores: &BTreeMap<String, f64>,
    inicial: &[f64],
    t: f64,
    numerico: &[f64],
) -> Option<SimSystemAccuracy> {
    let exata = catalogo::exata_sistema(&conceito.id)?;
    let verdadeiro = exata(valores, inicial, t)?;
    if verdadeiro.len() != numerico.len() {
        return None;
    }
    let erros: Vec<f64> = verdadeiro
        .iter()
        .zip(numerico)
        .map(|(v, n)| (n - v).abs())
        .collect();
    Some(SimSystemAccuracy {
        // Sempre do CONCEITO: o oraculo resolve EDO escalar, e `dsolve` sobre
        // SISTEMA nao foi medido — entrar sem medir e' o oposto do que este
        // dominio faz. A ressalva vai a tela pelo `oracle_note`.
        source: SimAccuracySource::Concept,
        max_error: erros.iter().fold(0.0f64, |a, b| a.max(*b)),
        exact: verdadeiro,
        numeric: numerico.to_vec(),
        absolute_error: erros,
    })
}

/// O que aconteceu com cada invariante declarado.
///
/// Vazio quando o conceito nao declara nenhum, ou quando os parametros nao
/// permitem calcular — e nesse caso a tela nao mostra a linha, em vez de mostrar
/// um zero que pareceria conservacao perfeita.
fn derivas(
    conceito: &SimConcept,
    valores: &BTreeMap<String, f64>,
    inicial: &[f64],
    fim: &[f64],
) -> Vec<SimInvariantDrift> {
    conceito
        .invariants
        .iter()
        .filter_map(|info| {
            let calcula = catalogo::invariante(&conceito.id, &info.id)?;
            let antes = calcula(valores, inicial)?;
            let depois = calcula(valores, fim)?;
            Some(SimInvariantDrift {
                id: info.id.clone(),
                initial: antes,
                final_value: depois,
                drift: (depois - antes).abs(),
            })
        })
        .collect()
}
