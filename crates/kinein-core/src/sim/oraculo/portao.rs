//! O PORTAO: o que a IDE aceita de volta, e o que ela avalia.
//!
//! **Arquivo proprio desde 2026-09-10.** Ele responde uma pergunta so' — *"esta
//! resposta pode virar numero?"* —, e ela e' independente de como o processo
//! foi chamado.

use exmex::prelude::*;

use super::{NaoSei, Solucao, VARIAVEL_DO_TEMPO};

/// O PORTAO: so' passa expressao que o `exmex` le' e cuja unica variavel e' `t`.
///
/// A segunda metade e' a que defende contra a falha SILENCIOSA. Medido em
/// 2026-09-06: `0.1*cos(pi*t)` tem `var_names() == ["pi", "t"]`, e preencher o
/// vetor pelo tamanho — que e' o que um codigo descuidado faz — devolve
/// `0,086232` onde a resposta e' `0,100000`, **sem erro nenhum**.
///
/// # Errors
///
/// [`NaoSei::RespostaIlegivel`] quando o `exmex` recusa a expressao ou quando
/// ela tem variavel que nao seja `t`.
pub fn portao(expressao: &str) -> Result<String, NaoSei> {
    // `**` so' significa potencia; a troca e' segura por construcao.
    let convertida = expressao.replace("**", "^");
    let analisada = exmex::parse::<f64>(&convertida).map_err(|erro| NaoSei::RespostaIlegivel {
        motivo: format!("o avaliador recusou a expressão: {erro}"),
    })?;
    let variaveis = analisada.var_names();
    let so_o_tempo = variaveis.len() == 1 && variaveis[0] == VARIAVEL_DO_TEMPO;
    if !so_o_tempo {
        return Err(NaoSei::RespostaIlegivel {
            motivo: format!(
                "a solução depende de {} além do tempo",
                variaveis
                    .iter()
                    .filter(|nome| *nome != VARIAVEL_DO_TEMPO)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
    Ok(convertida)
}

/// Avalia a solucao do oraculo num instante.
///
/// # Errors
///
/// [`NaoSei::RespostaIlegivel`] quando a avaliacao falha — o portao ja' provou
/// que a expressao tem uma variavel so', entao aqui so' sobra erro numerico.
pub fn avaliar(solucao: &Solucao, t: f64) -> Result<f64, NaoSei> {
    let analisada =
        exmex::parse::<f64>(&solucao.expressao).map_err(|erro| NaoSei::RespostaIlegivel {
            motivo: format!("o avaliador recusou a expressão: {erro}"),
        })?;
    analisada
        .eval(&[t])
        .map_err(|erro| NaoSei::RespostaIlegivel {
            motivo: format!("a solução não pôde ser avaliada: {erro}"),
        })
}
