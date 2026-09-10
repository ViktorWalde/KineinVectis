//! A CORRIDA: liga a formula do usuario, o integrador e o oraculo de exatidao.
//!
//! Este arquivo e' onde as tres pecas se encontram, e ele nao faz mais nada:
//!
//! ```text
//! formula.rs      diz quais variaveis a formula usa e se ela bate com o
//!                 conceito. A ligacao e' do usuario, nunca casada por nome
//! integrador.rs   anda no tempo com o metodo e o passo que o usuario escolheu
//! catalogo.rs     entrega a SOLUCAO FECHADA, quando existe
//! ```
//!
//! ## Por que o oraculo importa mais que a curva
//!
//! Integracao numerica nao e' exata por construcao. Sem comparacao, uma curva
//! de Euler com passo grande e' indistinguivel de fisica — medido, ela erra por
//! 3,11 onde a resposta e' -0,276 (`roadmaps/31` §11.1). Com a solucao fechada
//! ao lado, o erro vira numero na tela, e "exato e preciso" deixa de ser
//! promessa.
//!
//! O `t` da formula e o estado tambem sao LIGADOS pelo usuario: se ele nao
//! disser que uma variavel e' o tempo, ela e' um parametro constante. A IDE nao
//! supoe que a variavel chamada `t` seja o tempo.

use std::collections::BTreeMap;

use exmex::prelude::*;
use kinein_protocol::{
    SimAccuracy, SimAccuracySource, SimBinding, SimConcept, SimForm, SimMethod, SimRunResult,
    SimValue,
};

use super::catalogo;
use super::formula;
use super::integrador::{Corrida, ErroDeCorrida};
use super::oraculo;

/// As grandezas que o dominio trata como ESTADO, e nao como parametro.
///
/// `y` e' a variavel que a EDO resolve, `dy` a derivada dela e `t` o tempo. Sao
/// nomes de GRANDEZA do catalogo, e nao nomes de variavel da formula: continua
/// valendo que o usuario liga o que quiser a eles.
pub const GRANDEZA_ESTADO: &str = "y";
/// A derivada primeira, como grandeza.
pub const GRANDEZA_DERIVADA: &str = "dy";
/// O tempo, como grandeza.
pub const GRANDEZA_TEMPO: &str = "t";

/// De onde sai o valor de cada posicao do vetor de avaliacao.
///
/// O mapeamento vem da LIGACAO que o usuario fez, nunca da posicao em que a
/// variavel aparece na formula (ADR-0006, armadilha 1).
enum Fonte {
    /// Um parametro: o mesmo numero em todo passo.
    Fixo(f64),
    /// A variavel de estado, que muda a cada passo.
    Estado,
    /// A derivada primeira.
    Derivada,
    /// O tempo.
    Tempo,
}

/// O que impede uma corrida de sair.
#[derive(Debug, Clone, PartialEq)]
pub enum ErroDeExecucao {
    /// A formula nao passou na checagem contra o conceito.
    NaoChecada,
    /// O conceito escolhido nao integra: ele e' da forma algebrica.
    ///
    /// Nao e' detalhe: pedir `sim.run` num conceito algebrico e' pedir uma
    /// trajetoria de algo que nao tem uma. A recusa diz isso em vez de devolver
    /// uma reta.
    FormaNaoIntegra,
    /// Uma grandeza ligada ficou sem valor.
    SemValor {
        /// A grandeza em branco.
        grandeza: String,
    },
    /// A integracao recusou ou parou.
    Integrador(ErroDeCorrida),
}

/// Roda a EDO que o usuario escreveu.
///
/// `duracao`, `passo`, `metodo` e `amostras` vem do usuario — nenhum deles tem
/// padrao neste dominio (`arquitetura/34` §2.1).
#[allow(clippy::too_many_arguments)]
pub fn executar(
    conceito: &SimConcept,
    texto: &str,
    ligacoes: &[SimBinding],
    valores: &[SimValue],
    y0: f64,
    dy0: Option<f64>,
    duracao: f64,
    passo: f64,
    metodo: SimMethod,
    amostras: usize,
    oraculo: &oraculo::Config,
) -> Result<SimRunResult, ErroDeExecucao> {
    let segunda_ordem = match conceito.form {
        SimForm::Ode2 => true,
        SimForm::Ode1 => false,
        _ => return Err(ErroDeExecucao::FormaNaoIntegra),
    };
    if !formula::checar(conceito, texto, ligacoes).ok {
        return Err(ErroDeExecucao::NaoChecada);
    }
    let expressao = exmex::parse::<f64>(texto).map_err(|_| ErroDeExecucao::NaoChecada)?;
    let nomes = expressao.var_names().to_vec();

    let grandeza_de: BTreeMap<&str, &str> = ligacoes
        .iter()
        .map(|l| (l.variable.as_str(), l.quantity.as_str()))
        .collect();
    let valor_de: BTreeMap<&str, f64> = valores
        .iter()
        .map(|v| (v.quantity.as_str(), v.value))
        .collect();

    // O vetor de avaliacao e' montado UMA VEZ e reaproveitado: alocar um `Vec`
    // por passo dominaria o custo numa corrida de milhoes de passos.
    //
    // Cada posicao guarda o que ela e': um valor fixo, ou o slot de estado que
    // muda a cada passo. E o mapeamento vem da LIGACAO, nunca da posicao em que
    // a variavel aparece na formula (ADR-0006, armadilha 1).
    let mut plano = Vec::with_capacity(nomes.len());
    for nome in &nomes {
        let grandeza = grandeza_de
            .get(nome.as_str())
            .copied()
            .ok_or(ErroDeExecucao::NaoChecada)?;
        plano.push(match grandeza {
            GRANDEZA_ESTADO => Fonte::Estado,
            GRANDEZA_DERIVADA => Fonte::Derivada,
            GRANDEZA_TEMPO => Fonte::Tempo,
            outra => Fonte::Fixo(valor_de.get(outra).copied().ok_or_else(|| {
                ErroDeExecucao::SemValor {
                    grandeza: outra.to_string(),
                }
            })?),
        });
    }

    let mut buffer = vec![0.0f64; nomes.len()];
    let derivada = |t: f64, y: f64, dy: f64| -> Option<f64> {
        for (slot, fonte) in buffer.iter_mut().zip(&plano) {
            *slot = match fonte {
                Fonte::Fixo(v) => *v,
                Fonte::Estado => y,
                Fonte::Derivada => dy,
                Fonte::Tempo => t,
            };
        }
        expressao.eval(&buffer).ok()
    };

    let corrida = Corrida {
        passo,
        duracao,
        metodo,
        amostras,
        inicial: kinein_protocol::SimInitial { y: y0, dy: dy0 },
        segunda_ordem,
    };
    let saida = corrida
        .integrar(derivada)
        .map_err(ErroDeExecucao::Integrador)?;

    // O instante ALCANCADO, nao o pedido: ver `Saida::t_final`.
    let (accuracy, oracle_note) = exatidao(
        oraculo,
        &Alvo {
            conceito,
            texto,
            grandeza_de: &grandeza_de,
            valores: &valor_de,
            y0,
            dy0: dy0.unwrap_or(0.0),
        },
        saida.t_final,
        saida.final_y,
    );

    Ok(SimRunResult {
        steps_taken: saida.passos,
        sample_every: saida.a_cada,
        trail: saida.trilha,
        accuracy,
        oracle_note,
        method_label: super::integrador::rotulo(metodo, passo),
    })
}

/// O que descreve a equacao a conferir: a formula, a ligacao e as condicoes.
///
/// Existe porque estes seis campos andam juntos e descrevem UMA coisa — a
/// pergunta que se faz sobre o resultado. Passa-los soltos fazia a assinatura
/// crescer a cada campo novo, e a catraca do clippy reclamou primeiro.
struct Alvo<'a> {
    /// O conceito escolhido.
    conceito: &'a SimConcept,
    /// A formula, como o usuario a digitou.
    texto: &'a str,
    /// O que cada variavel dela e', pela LIGACAO que ele fez.
    grandeza_de: &'a BTreeMap<&'a str, &'a str>,
    /// O valor de cada grandeza.
    valores: &'a BTreeMap<&'a str, f64>,
    /// `y(0)`.
    y0: f64,
    /// `y'(0)`; zero quando a forma nao a usa.
    dy0: f64,
}

/// A comparacao com a verdade — e DE ONDE a verdade veio.
///
/// **Duas procedencias, e a diferenca entre elas foi medida** (`roadmaps/31`
/// §19.0): a solucao do CONCEITO responde pela equacao canonica, e a do ORACULO
/// responde pela que o usuario digitou. Quando as duas divergem, o numero da
/// primeira e' a distancia ate' outra pergunta — foi assim que a IDE acusou
/// erro de 2,5e-2 numa integracao correta ate' 3,2e-7.
///
/// A ordem e' oraculo primeiro, conceito depois. Nao esconder o numero do
/// conceito e' deliberado: sem `SymPy` na maquina, ele continua sendo a melhor
/// resposta disponivel — o que faltava era **dizer que e' ele**.
///
/// Devolve a comparacao e a frase que explica a procedencia, quando ela nao e'
/// a do oraculo.
fn exatidao(
    config: &oraculo::Config,
    alvo: &Alvo<'_>,
    t: f64,
    numerico: f64,
) -> (Option<SimAccuracy>, Option<String>) {
    let (conceito, valores) = (alvo.conceito, alvo.valores);
    let (do_oraculo, nota) = match perguntar_ao_oraculo(config, alvo) {
        Ok(solucao) => match oraculo::avaliar(&solucao, t) {
            Ok(verdadeiro) => (Some((verdadeiro, solucao)), None),
            Err(motivo) => (None, Some(motivo.frase())),
        },
        Err(motivo) => (None, Some(motivo.frase())),
    };

    if let Some((verdadeiro, solucao)) = do_oraculo {
        return (
            Some(comparar(
                verdadeiro,
                numerico,
                SimAccuracySource::Oracle,
                Some(solucao),
            )),
            None,
        );
    }

    let Some(exata) = catalogo::exata(&conceito.id) else {
        return (None, nota);
    };
    let mapa: BTreeMap<String, f64> = valores
        .iter()
        .map(|(k, v)| ((*k).to_string(), *v))
        .collect();
    let Some(verdadeiro) = exata(&mapa, alvo.y0, alvo.dy0, t) else {
        return (None, nota);
    };
    (
        Some(comparar(
            verdadeiro,
            numerico,
            SimAccuracySource::Concept,
            None,
        )),
        nota,
    )
}

/// Monta a comparacao, com o erro RELATIVO junto.
///
/// O relativo existe porque so' o absoluto engana nos dois sentidos: `1,474`
/// sobre `83.178` e' uma integracao excelente, e o mesmo `1,474` ao lado de
/// `0,032` seria catastrofe (`roadmaps/31` §19.0).
fn comparar(
    verdadeiro: f64,
    numerico: f64,
    source: SimAccuracySource,
    solucao: Option<oraculo::Solucao>,
) -> SimAccuracy {
    let absoluto = (numerico - verdadeiro).abs();
    let (solved_by, closed_form) = match solucao {
        Some(s) => (Some(s.resolvedor), Some(s.expressao)),
        None => (None, None),
    };
    SimAccuracy {
        exact: verdadeiro,
        numeric: numerico,
        absolute_error: absoluto,
        // Zero nao tem relativo, e inventar um seria dividir por zero com cara
        // de numero.
        relative_error: (verdadeiro != 0.0).then(|| absoluto / verdadeiro.abs()),
        source,
        solved_by,
        closed_form,
    }
}

/// Monta a pergunta do oraculo a partir da LIGACAO que o usuario fez.
///
/// Os papeis viajam NOMEADOS: qual variavel e' a posicao, qual e' a velocidade,
/// qual e' o tempo. Nada e' deduzido do nome nem da posicao na formula — e' a
/// mesma regra que governa o vetor de avaliacao (ADR-0006, armadilha 1).
fn perguntar_ao_oraculo(
    config: &oraculo::Config,
    alvo: &Alvo<'_>,
) -> Result<oraculo::Solucao, oraculo::NaoSei> {
    let (conceito, texto) = (alvo.conceito, alvo.texto);
    let (grandeza_de, valores) = (alvo.grandeza_de, alvo.valores);
    let (y0, dy0) = (alvo.y0, alvo.dy0);
    let segunda_ordem = conceito.form == SimForm::Ode2;
    let mut estado = None;
    let mut derivada = None;
    let mut tempo = None;
    let mut parametros = Vec::new();
    for (variavel, grandeza) in grandeza_de {
        match *grandeza {
            GRANDEZA_ESTADO => estado = Some((*variavel).to_string()),
            GRANDEZA_DERIVADA => derivada = Some((*variavel).to_string()),
            GRANDEZA_TEMPO => tempo = Some((*variavel).to_string()),
            outra => {
                let valor = valores.get(outra).copied().unwrap_or(f64::NAN);
                if !valor.is_finite() {
                    return Err(oraculo::NaoSei::NaoResolve);
                }
                parametros.push(((*variavel).to_string(), decimal(valor)));
            }
        }
    }
    let Some(estado) = estado else {
        return Err(oraculo::NaoSei::NaoResolve);
    };
    if !y0.is_finite() || (segunda_ordem && !dy0.is_finite()) {
        return Err(oraculo::NaoSei::NaoResolve);
    }
    oraculo::resolver(
        config,
        &oraculo::Pergunta {
            formula: texto.to_owned(),
            estado,
            derivada,
            tempo,
            parametros,
            ordem: if segunda_ordem { 2 } else { 1 },
            y0: decimal(y0),
            dy0: segunda_ordem.then(|| decimal(dy0)),
        },
    )
}

/// Um `f64` como TEXTO decimal, para o outro lado racionalizar exato.
///
/// `Display` de `f64` em Rust nunca usa notacao exponencial, entao
/// `Rational("0.5")` do outro lado e' exato — e exato e' o que o `dsolve`
/// precisa: com `4.905` em float ele leva 4,2 s e devolve `RecursionError`.
fn decimal(valor: f64) -> String {
    format!("{valor}")
}
