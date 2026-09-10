//! Os INVARIANTES: as grandezas que a fisica conserva, e a deriva delas.
//!
//! ## Por que este arquivo existe, e por que ele nao existia antes
//!
//! Na forma escalar o oraculo e' a solucao fechada: a IDE compara o numero
//! calculado com o verdadeiro e mostra o erro. **No sistema isso cobre pouco.**
//! A orbita circular tem forma fechada; a eliptica exige a equacao de Kepler,
//! que e' transcendental; e o pendulo duplo **nao tem** — medido em 2026-09-06,
//! ele e' caotico acima de uma energia (`../roadmaps/31` §19.1.4).
//!
//! O invariante e' o sinal que sobra, e ele nao precisa de oraculo nenhum:
//! energia e momento angular sao conservados pela fisica, entao a DERIVA deles
//! mede exatamente o modo de falha que a integracao de sistema tem. Medido na
//! orbita circular (`../roadmaps/31` §19.1.1):
//!
//! ```text
//! Euler explicito    dt=0,01, 10 voltas   deriva de energia   2,032e-01
//! Euler simpletico   dt=0,01, 10 voltas   deriva de energia   2,800e-10
//! ```
//!
//! Oito ordens de grandeza — a diferenca entre uma orbita que espirala para
//! fora e uma que fecha.
//!
//! ## O limite, que vai junto na tela
//!
//! **Invariante conservado NAO significa resultado certo.** Um erro que
//! respeita a simetria conservada passa por ele intacto. Por isso a tela chama
//! isto de DERIVA e nunca de erro: sao coisas diferentes, e confundi-las seria
//! prometer exatidao de novo (`arquitetura/34` §13.5).

use std::collections::BTreeMap;

/// Uma grandeza conservada, calculada a partir do estado.
///
/// Recebe os parametros e o vetor de estado, e devolve o valor. `None` quando
/// os parametros nao permitem calcular — a mesma regra do oraculo: dizer "nao
/// sei" em vez de devolver um numero que nao e' o numero.
pub type Invariante = fn(&BTreeMap<String, f64>, &[f64]) -> Option<f64>;

/// Energia mecanica especifica da orbita de dois corpos: `v^2/2 - mu/r`.
///
/// "Especifica" e' por unidade de massa, que e' a forma em que a orbita e'
/// escrita quando `mu = G*M` absorve a massa central.
#[must_use]
pub fn energia_orbita(valores: &BTreeMap<String, f64>, estado: &[f64]) -> Option<f64> {
    let mu = *valores.get("mu")?;
    if estado.len() != 4 {
        return None;
    }
    let raio = estado[0].hypot(estado[1]);
    if raio <= 0.0 {
        return None;
    }
    let velocidade2 = estado[2].mul_add(estado[2], estado[3] * estado[3]);
    Some(0.5f64.mul_add(velocidade2, -(mu / raio)))
}

/// Momento angular especifico da orbita: `x*vy - y*vx`.
///
/// Nao depende de parametro nenhum, e e' o invariante que denuncia um
/// integrador que gira errado sem mudar o raio.
#[must_use]
pub fn momento_angular_orbita(_valores: &BTreeMap<String, f64>, estado: &[f64]) -> Option<f64> {
    if estado.len() != 4 {
        return None;
    }
    Some(estado[0].mul_add(estado[3], -(estado[1] * estado[2])))
}

/// Energia POR UNIDADE DE MASSA do pendulo duplo, `Y = [t1, t2, w1, w2]`.
///
/// Duas hastes de comprimento `l` e massas iguais na ponta de cada uma:
///
/// ```text
/// T/m = l^2 * (w1^2 + w2^2/2 + w1*w2*cos(t1 - t2))
/// V/m = -g*l * (2*cos(t1) + cos(t2))
/// ```
///
/// **Derivada com o `SymPy` em 2026-09-06** a partir da lagrangiana, e nao
/// copiada de memoria — a primeira versao escrita de cabeca passou nesta funcao
/// e reprovou no teste de conservacao, com deriva de 2,18 onde deveria ser
/// zero. Com `g` e `l` iguais a 1 ela se reduz a
/// `w1^2 + w1*w2*cos(t1-t2) + w2^2/2 - 2*cos(t1) - cos(t2)`, que e' exatamente
/// o que o `SymPy` imprimiu.
#[must_use]
pub fn energia_pendulo_duplo(valores: &BTreeMap<String, f64>, estado: &[f64]) -> Option<f64> {
    let gravidade = *valores.get("g")?;
    let comprimento = *valores.get("l")?;
    if estado.len() != 4 {
        return None;
    }
    let (t1, t2, w1, w2) = (estado[0], estado[1], estado[2], estado[3]);
    let cinetica = comprimento
        * comprimento
        * w2.mul_add(w2 / 2.0, w1 * w1)
            .mul_add(1.0, w1 * w2 * (t1 - t2).cos());
    let potencial = -gravidade * comprimento * 2.0f64.mul_add(t1.cos(), t2.cos());
    Some(cinetica + potencial)
}

/// Energia de duas massas acopladas por tres molas, `Y = [x1, x2, v1, v2]`.
///
/// Molas de constante `k` nas pontas e `kc` no meio, massas `m`:
///
/// ```text
/// T = m*(v1^2 + v2^2)/2
/// V = k*(x1^2 + x2^2)/2 + kc*(x2 - x1)^2/2
/// ```
#[must_use]
pub fn energia_massa_mola(valores: &BTreeMap<String, f64>, estado: &[f64]) -> Option<f64> {
    let massa = *valores.get("m")?;
    let mola = *valores.get("k")?;
    let acoplamento = *valores.get("kc")?;
    if estado.len() != 4 || massa <= 0.0 {
        return None;
    }
    let (x1, x2, v1, v2) = (estado[0], estado[1], estado[2], estado[3]);
    let cinetica = massa * v1.mul_add(v1, v2 * v2) / 2.0;
    let diferenca = x2 - x1;
    let potencial = mola.mul_add(
        x1.mul_add(x1, x2 * x2) / 2.0,
        acoplamento * diferenca * diferenca / 2.0,
    );
    Some(cinetica + potencial)
}
