//! As SOLUCOES FECHADAS: o oraculo de exatidao do dominio.
//!
//! Saiu do `catalogo.rs` em 2026-09-06 por RESPONSABILIDADE, nao por tamanho:
//! o catalogo e' a TABELA de conceitos — dado declarado, que cresce por entrada
//! — e isto aqui e' MATEMATICA, que so' cresce quando alguem escreve e verifica
//! uma solucao nova. Sao dois motivos de mudanca diferentes no mesmo arquivo,
//! que e' o corte que a §4 do `ARCHITECTURE.md` pede.
//!
//! ## O que este arquivo promete, e o que ele recusa
//!
//! Cada funcao devolve o valor VERDADEIRO, e devolve `None` quando a formula
//! fechada **nao vale para os parametros dados**. Dizer "nao sei" e'
//! obrigatorio: usar a expressao errada daria um "valor exato" mentiroso, que e'
//! pior que nao ter oraculo nenhum (`arquitetura/34` §7).
//!
//! ## O `mul_add` aqui NAO e' estilo
//!
//! Ele e' fused multiply-add, que arredonda UMA vez em vez de duas. Este e' o
//! oraculo — o valor com que a IDE mede o erro de todo o resto —, entao cada
//! casa que ele ganha vale. No `integrador.rs` a decisao e' a oposta, e esta'
//! registrada la': o RK4 e' transcricao de metodo de livro e precisa continuar
//! legivel.

use std::collections::BTreeMap;

/// A solucao fechada de um conceito ESCALAR, quando ela existe.
///
/// Recebe os valores das grandezas por id, o estado inicial e o instante.
pub type Fechada = fn(&BTreeMap<String, f64>, f64, f64, f64) -> Option<f64>;

/// A solucao fechada de um conceito de SISTEMA, quando ela existe.
///
/// Recebe os valores dos parametros, o estado inicial inteiro e o instante, e
/// devolve o estado inteiro. `None` quando nao vale — e no sistema isso e' a
/// regra, nao a excecao: a orbita eliptica exige resolver a equacao de Kepler,
/// que e' transcendental, e o pendulo duplo nao tem solucao fechada nenhuma
/// (`arquitetura/34` §13.5).
pub type FechadaSistema = fn(&BTreeMap<String, f64>, &[f64], f64) -> Option<Vec<f64>>;

/// `y'' = -(k/m) y - (c/m) y'` — subamortecido, que e' o caso que a formula
/// abaixo cobre. Fora dele a funcao diz que nao sabe.
#[must_use]
pub fn oscilador_amortecido(
    valores: &BTreeMap<String, f64>,
    y0: f64,
    dy0: f64,
    tempo: f64,
) -> Option<f64> {
    let massa = *valores.get("m")?;
    let mola = *valores.get("k")?;
    let atrito = valores.get("c").copied().unwrap_or(0.0);
    if massa <= 0.0 || mola <= 0.0 {
        return None;
    }
    let w0 = (mola / massa).sqrt();
    let gama = atrito / (2.0 * massa);
    let dentro = w0.mul_add(w0, -(gama * gama));
    // Criticamente amortecido ou superamortecido tem OUTRA expressao. Aplicar
    // esta ali daria um "valor exato" que nao e' o valor.
    if dentro <= 0.0 {
        return None;
    }
    let wd = dentro.sqrt();
    let fase = wd * tempo;
    let combinacao = ((dy0 + gama * y0) / wd).mul_add(fase.sin(), y0 * fase.cos());
    Some((-gama * tempo).exp() * combinacao)
}

/// `y'' = -g` — queda com aceleracao constante.
#[must_use]
pub fn queda_livre(valores: &BTreeMap<String, f64>, y0: f64, dy0: f64, tempo: f64) -> Option<f64> {
    let gravidade = *valores.get("g")?;
    Some((0.5 * gravidade * tempo).mul_add(-tempo, dy0.mul_add(tempo, y0)))
}

/// `y' = -k y` — decaimento exponencial.
#[must_use]
pub fn decaimento(valores: &BTreeMap<String, f64>, y0: f64, _dy0: f64, tempo: f64) -> Option<f64> {
    let taxa = *valores.get("k")?;
    Some(y0 * (-taxa * tempo).exp())
}

/// Quao longe de circular a orbita pode estar e a formula abaixo ainda valer.
///
/// Nao e' tolerancia de gosto: acima disto a orbita e' eliptica e o angulo
/// deixa de crescer com velocidade constante, entao a expressao daria um
/// "valor exato" que nao e' o valor.
const TOLERANCIA_CIRCULAR: f64 = 1e-9;

/// Orbita de dois corpos, `Y = [x, y, vx, vy]`, no caso **circular**.
///
/// A condicao de circularidade e' `v^2 = mu/r` com `r` perpendicular a `v`.
/// Fora dela a orbita e' uma conica cuja posicao no tempo exige resolver a
/// equacao de Kepler (`M = E - e sin E`), que e' transcendental e nao tem
/// solucao fechada — e por isso esta funcao **recusa** em vez de aproximar.
///
/// No caso circular o movimento e' rotacao rigida com velocidade angular
/// `w = sqrt(mu/r^3)`, e o estado no instante `t` sai de uma rotacao do estado
/// inicial. Fonte: qualquer curso de mecanica celeste; verificado contra a
/// integracao em `tests/sim_sistema.rs`.
#[must_use]
pub fn orbita_circular(
    valores: &BTreeMap<String, f64>,
    inicial: &[f64],
    tempo: f64,
) -> Option<Vec<f64>> {
    let mu = *valores.get("mu")?;
    if mu <= 0.0 || inicial.len() != 4 {
        return None;
    }
    let (x0, y0, vx0, vy0) = (inicial[0], inicial[1], inicial[2], inicial[3]);
    let raio = x0.hypot(y0);
    if raio <= 0.0 {
        return None;
    }
    // Circular exige DUAS coisas: velocidade de modulo certo, e perpendicular
    // ao raio. Checar so' o modulo aceitaria uma elipse com o mesmo |v|.
    let velocidade2 = vx0.mul_add(vx0, vy0 * vy0);
    let radial = x0.mul_add(vx0, y0 * vy0) / raio;
    if (velocidade2 - mu / raio).abs() > TOLERANCIA_CIRCULAR * (mu / raio)
        || radial.abs() > TOLERANCIA_CIRCULAR * velocidade2.sqrt().max(1.0)
    {
        return None;
    }
    let omega = (mu / (raio * raio * raio)).sqrt();
    let angulo = omega * tempo;
    let (seno, cosseno) = angulo.sin_cos();
    Some(vec![
        x0.mul_add(cosseno, -(y0 * seno)),
        x0.mul_add(seno, y0 * cosseno),
        vx0.mul_add(cosseno, -(vy0 * seno)),
        vx0.mul_add(seno, vy0 * cosseno),
    ])
}
