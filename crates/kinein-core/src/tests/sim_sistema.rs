//! O integrador VETORIAL, verificado por ordem de convergencia — e a armadilha
//! que a medicao achou ANTES de este arquivo existir.
//!
//! ## A grandeza medida faz parte do teste
//!
//! Medido em 2026-09-06, na orbita circular de raio verdadeiro 1
//! (`roadmaps/31` §19.1.2), com a ordem calculada **no raio**:
//!
//! ```text
//! Euler explicito     0,98   0,98   0,99     bate com o teorico (1)
//! Euler simpletico    1,32   1,09   3,21     ruidosa
//! Runge-Kutta 4       5,00   4,99   5,30     ordem 5 num metodo de ordem 4
//! ```
//!
//! **O RK4 mede 5 porque o raio cancela erro:** a orbita circular e' solucao
//! especial e o raio nao ve' o erro de fase. Um teste escrito sobre o raio
//! aprovaria codigo errado e reprovaria codigo certo, mostrando um numero
//! bonito nos dois casos.
//!
//! Por isso **a ordem aqui e' medida na NORMA DO MAXIMO do vetor de estado**, e
//! esta frase esta' no arquivo em vez de escondida: e' a mesma familia da
//! armadilha da faixa de `dt` do `sim_integrador.rs`, num eixo diferente — la'
//! era ONDE medir, aqui e' O QUE medir.

use std::collections::BTreeMap;

use kinein_protocol::{SimBinding, SimComponentFormula, SimMethod, SimValue};

use crate::sim::corrida_sistema::{self, ErroDeSistemaExec};
use crate::sim::sistema::ErroDeSistema;
use crate::sim::{catalogo, exata, invariante};

/// Orbita circular de raio 1 com `mu = 1`: periodo `2*pi`, velocidade 1.
const INICIAL: [f64; 4] = [1.0, 0.0, 0.0, 1.0];
/// Uma volta inteira.
const PERIODO: f64 = std::f64::consts::TAU;

fn valores() -> Vec<SimValue> {
    vec![SimValue {
        quantity: "mu".to_string(),
        value: 1.0,
    }]
}

fn liga(variavel: &str, grandeza: &str) -> SimBinding {
    SimBinding {
        variable: variavel.to_string(),
        quantity: grandeza.to_string(),
    }
}

/// As quatro equacoes da orbita, como o usuario as escreveria e as ligaria.
///
/// Repare que `vx'` menciona `x` e `y`, e nao `vx`: **a formula de um componente
/// pode depender de qualquer outro**, e e' por isso que a ligacao aponta para
/// componentes por id.
fn equacoes_orbita() -> Vec<SimComponentFormula> {
    vec![
        SimComponentFormula {
            component: "x".to_string(),
            formula: "vx".to_string(),
            bindings: vec![liga("vx", "vx")],
        },
        SimComponentFormula {
            component: "y".to_string(),
            formula: "vy".to_string(),
            bindings: vec![liga("vy", "vy")],
        },
        SimComponentFormula {
            component: "vx".to_string(),
            formula: "-mu*x/(x^2+y^2)^1.5".to_string(),
            bindings: vec![liga("mu", "mu"), liga("x", "x"), liga("y", "y")],
        },
        SimComponentFormula {
            component: "vy".to_string(),
            formula: "-mu*y/(x^2+y^2)^1.5".to_string(),
            bindings: vec![liga("mu", "mu"), liga("x", "x"), liga("y", "y")],
        },
    ]
}

fn roda(metodo: SimMethod, passo: f64, duracao: f64) -> kinein_protocol::SimRunSystemResult {
    let conceito = catalogo::conceito("orbita-dois-corpos").expect("conceito no catalogo");
    corrida_sistema::executar(
        &conceito,
        &equacoes_orbita(),
        &valores(),
        &INICIAL,
        duracao,
        passo,
        metodo,
        100,
    )
    .expect("a corrida da orbita tem de sair")
}

/// A ordem observada entre dois passos, na grandeza que a assercao declara.
fn ordem(erro_grosso: f64, erro_fino: f64) -> f64 {
    (erro_grosso / erro_fino).log2()
}

#[test]
fn a_ordem_de_convergencia_e_medida_na_norma_do_maximo_do_estado() {
    // A GRANDEZA esta' declarada aqui, e nao e' detalhe: medir no raio da
    // orbita da' 5,00 para o RK4, que e' de ordem 4 (roadmaps/31 §19.1.2).
    for (metodo, teorica, minimo, maximo) in [
        (SimMethod::Euler, 1.0, 0.85, 1.20),
        (SimMethod::Rk4, 4.0, 3.70, 4.30),
    ] {
        let grosso = roda(metodo, 0.01, PERIODO).accuracy.expect("oraculo");
        let fino = roda(metodo, 0.005, PERIODO).accuracy.expect("oraculo");
        let medida = ordem(grosso.max_error, fino.max_error);
        assert!(
            (minimo..=maximo).contains(&medida),
            "{metodo:?} deveria medir ordem ~{teorica} na NORMA DO MAXIMO do vetor de \
             estado entre dt=0,01 e dt=0,005, e mediu {medida:.2} \
             (erros {:.3e} e {:.3e})",
            grosso.max_error,
            fino.max_error
        );
    }
}

#[test]
fn o_simpletico_conserva_energia_onde_o_explicito_espirala() {
    // O numero que paga a complicacao do pareamento declarado. Medido em
    // 2026-09-06: dt=0,01 por dez voltas leva o raio de 1 a 1,65 no explicito.
    let dez_voltas = 10.0 * PERIODO;
    let explicito = roda(SimMethod::Euler, 0.01, dez_voltas);
    let simpletico = roda(SimMethod::EulerSymplectic, 0.01, dez_voltas);

    let deriva = |r: &kinein_protocol::SimRunSystemResult| {
        r.invariants
            .iter()
            .find(|i| i.id == "energia")
            .expect("a orbita declara o invariante de energia")
            .drift
    };
    let (ruim, bom) = (deriva(&explicito), deriva(&simpletico));
    assert!(
        ruim > 1e-2,
        "o Euler explicito deveria derivar muito na energia da orbita, e derivou {ruim:.3e}"
    );
    assert!(
        bom < 1e-6,
        "o simpletico deveria conservar a energia da orbita, e derivou {bom:.3e}"
    );
    assert!(
        ruim / bom > 1e4,
        "a diferenca entre os dois metodos deveria ser de ordens de grandeza, e foi {:.1}x",
        ruim / bom
    );
}

#[test]
fn o_simpletico_e_recusado_quando_o_conceito_nao_declara_pareamento() {
    // Sem o par posicao/velocidade o metodo NAO ESTA DEFINIDO. Recusar com o
    // motivo e' melhor que aceitar e integrar outra coisa (arquitetura/34 §13.3).
    let mut conceito = catalogo::conceito("orbita-dois-corpos").expect("conceito");
    conceito.pairing.clear();
    let erro = corrida_sistema::executar(
        &conceito,
        &equacoes_orbita(),
        &valores(),
        &INICIAL,
        PERIODO,
        0.01,
        SimMethod::EulerSymplectic,
        10,
    )
    .expect_err("sem pareamento o simpletico tem de ser recusado");
    assert_eq!(
        erro,
        ErroDeSistemaExec::Integrador(ErroDeSistema::SemPareamento)
    );

    // E os outros dois metodos continuam valendo no mesmo conceito: a recusa e'
    // do METODO, nao do conceito.
    for metodo in [SimMethod::Euler, SimMethod::Rk4] {
        corrida_sistema::executar(
            &conceito,
            &equacoes_orbita(),
            &valores(),
            &INICIAL,
            PERIODO,
            0.01,
            metodo,
            10,
        )
        .unwrap_or_else(|erro| panic!("{metodo:?} nao deveria depender do pareamento: {erro:?}"));
    }
}

#[test]
fn trocar_duas_ligacoes_de_papel_muda_o_resultado() {
    // O GATE DA LIGACAO, na forma vetorial. Se trocar `x` por `y` na formula de
    // `vx` NAO mudar o resultado, e' porque o vetor de avaliacao voltou a ser
    // montado por posicao — a falha silenciosa da ADR-0006, armadilha 1.
    //
    // O estado inicial e' assimetrico de proposito: em (1,0) trocar x por y
    // muda a fisica. Com (1,1) a simetria esconderia a troca.
    let conceito = catalogo::conceito("orbita-dois-corpos").expect("conceito");
    let certo = roda(SimMethod::Rk4, 0.01, PERIODO / 4.0);

    let mut trocadas = equacoes_orbita();
    trocadas[2].bindings = vec![liga("mu", "mu"), liga("x", "y"), liga("y", "x")];
    let torto = corrida_sistema::executar(
        &conceito,
        &trocadas,
        &valores(),
        &INICIAL,
        PERIODO / 4.0,
        0.01,
        SimMethod::Rk4,
        100,
    )
    .expect("a corrida com a ligacao trocada ainda RODA — ela so' calcula outra coisa");

    let a = certo.trail.last().expect("trilha").values.clone();
    let b = torto.trail.last().expect("trilha").values.clone();
    assert!(
        a.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-6),
        "trocar duas ligacoes de papel TEM de mudar o resultado; \
         se nao muda, a ligacao nao esta sendo usada. certo={a:?} trocado={b:?}"
    );
}

#[test]
fn o_oraculo_recusa_a_orbita_que_nao_e_circular() {
    // Dizer "nao sei" e' obrigatorio: a orbita eliptica exige a equacao de
    // Kepler, e aplicar a formula circular daria um "valor exato" que nao e' o
    // valor.
    let mut parametros = BTreeMap::new();
    parametros.insert("mu".to_string(), 1.0);

    assert!(
        exata::orbita_circular(&parametros, &INICIAL, 1.0).is_some(),
        "a orbita circular tem solucao fechada e o oraculo tem de da-la"
    );
    // Mesmo |v|, mas radial: e' uma queda, nao uma orbita.
    assert!(
        exata::orbita_circular(&parametros, &[1.0, 0.0, 1.0, 0.0], 1.0).is_none(),
        "velocidade radial nao e orbita circular, e o oraculo tem de recusar"
    );
    // Perpendicular, mas rapida demais: elipse.
    assert!(
        exata::orbita_circular(&parametros, &[1.0, 0.0, 0.0, 1.3], 1.0).is_none(),
        "velocidade fora da circular faz uma elipse, e o oraculo tem de recusar"
    );
}

#[test]
fn o_oraculo_circular_bate_com_a_integracao_fina() {
    // Verificacao cruzada: a formula fechada e o RK4 com passo fino tem de
    // concordar. Se so' uma delas estivesse errada, os testes de ordem acima
    // ainda passariam — eles medem a TAXA, nao o valor.
    let resultado = roda(SimMethod::Rk4, 1e-4, PERIODO);
    let exatidao = resultado.accuracy.expect("a orbita circular tem oraculo");
    assert!(
        exatidao.max_error < 1e-9,
        "o RK4 com dt=1e-4 deveria bater com a solucao circular, e errou {:.3e}",
        exatidao.max_error
    );
    // E a periodicidade e' checada NO ORACULO, em `t` exatamente igual ao
    // periodo — nao no fim da corrida.
    //
    // A diferenca nao e' preciosismo: com `dt=1e-4` a corrida da' 62.832 passos
    // e para em 6,2832, que esta' a 1,47e-5 do `2*pi`. Nesse instante a orbita
    // ainda nao fechou, e o estado exato e' 0,99999999989 — que e' o valor
    // CERTO para aquele tempo. Cobrar 1,0 dele seria cobrar do oraculo um erro
    // que e' da discretizacao do tempo.
    let mut parametros = BTreeMap::new();
    parametros.insert("mu".to_string(), 1.0);
    let volta = exata::orbita_circular(&parametros, &INICIAL, PERIODO).expect("oraculo");
    for (indice, valor) in volta.iter().enumerate() {
        assert!(
            (valor - INICIAL[indice]).abs() < 1e-15,
            "uma volta inteira devolve o estado inicial; componente {indice} deu {valor}"
        );
    }
}

#[test]
fn o_estado_inicial_precisa_de_um_valor_por_componente() {
    let conceito = catalogo::conceito("orbita-dois-corpos").expect("conceito");
    let erro = corrida_sistema::executar(
        &conceito,
        &equacoes_orbita(),
        &valores(),
        &[1.0, 0.0],
        PERIODO,
        0.01,
        SimMethod::Rk4,
        10,
    )
    .expect_err("estado incompleto tem de ser recusado");
    assert_eq!(
        erro,
        ErroDeSistemaExec::Integrador(ErroDeSistema::EstadoIncompleto {
            esperado: 4,
            recebido: 2,
        })
    );
}

#[test]
fn a_checagem_acha_componente_sem_formula_e_formula_sem_componente() {
    let conceito = catalogo::conceito("orbita-dois-corpos").expect("conceito");

    let mut faltando = equacoes_orbita();
    faltando.pop();
    let resultado = corrida_sistema::checar(&conceito, &faltando);
    assert!(!resultado.ok);
    assert_eq!(resultado.missing, vec!["vy".to_string()]);

    let mut sobrando = equacoes_orbita();
    sobrando.push(SimComponentFormula {
        component: "vz".to_string(),
        formula: "0".to_string(),
        bindings: vec![],
    });
    let resultado = corrida_sistema::checar(&conceito, &sobrando);
    assert!(!resultado.ok);
    assert_eq!(resultado.unknown, vec!["vz".to_string()]);

    // E o caso completo passa, com o aviso da §5.1 junto.
    let resultado = corrida_sistema::checar(&conceito, &equacoes_orbita());
    assert!(resultado.ok, "as quatro equacoes fecham: {resultado:?}");
    assert!(
        resultado.caveat.is_some(),
        "o aviso vai a tela quando passa"
    );
}

#[test]
fn o_pendulo_duplo_nao_tem_oraculo_e_a_energia_e_o_que_sobra() {
    // O conceito que so' o invariante consegue medir. A energia e' conservada
    // pela fisica; sob RK4 com passo fino ela tem de ficar praticamente parada.
    // As equacoes de movimento foram DERIVADAS com o `dsolve`/Lagrange do SymPy
    // em 2026-09-06, e a forma compacta abaixo foi conferida contra a derivacao
    // em 2.000 pontos aleatorios (maior diferenca 7,1e-15). Escreve-las de
    // memoria foi a primeira tentativa, e ela reprovou aqui: a energia derivou
    // 2,18 onde deveria ficar parada. O invariante pegou.
    let conceito = catalogo::conceito("pendulo-duplo").expect("conceito");
    let equacoes = vec![
        SimComponentFormula {
            component: "t1".to_string(),
            formula: "w1".to_string(),
            bindings: vec![liga("w1", "w1")],
        },
        SimComponentFormula {
            component: "t2".to_string(),
            formula: "w2".to_string(),
            bindings: vec![liga("w2", "w2")],
        },
        SimComponentFormula {
            component: "w1".to_string(),
            formula: "(-3*g*sin(a) - g*sin(a-2*b) - 2*L*sin(a-b)*(d^2 + c^2*cos(a-b)))\
                      /(L*(3 - cos(2*a-2*b)))"
                .to_string(),
            bindings: vec![
                liga("a", "t1"),
                liga("b", "t2"),
                liga("c", "w1"),
                liga("d", "w2"),
                liga("g", "g"),
                liga("L", "l"),
            ],
        },
        SimComponentFormula {
            component: "w2".to_string(),
            formula: "(2*sin(a-b)*(2*L*c^2 + 2*g*cos(a) + L*d^2*cos(a-b)))\
                      /(L*(3 - cos(2*a-2*b)))"
                .to_string(),
            bindings: vec![
                liga("a", "t1"),
                liga("b", "t2"),
                liga("c", "w1"),
                liga("d", "w2"),
                liga("g", "g"),
                liga("L", "l"),
            ],
        },
    ];
    assert!(
        corrida_sistema::checar(&conceito, &equacoes).ok,
        "as quatro equacoes do pendulo duplo tem de fechar com o conceito"
    );
    let resultado = corrida_sistema::executar(
        &conceito,
        &equacoes,
        &[
            SimValue {
                quantity: "g".to_string(),
                value: 1.0,
            },
            SimValue {
                quantity: "l".to_string(),
                value: 1.0,
            },
        ],
        &[2.0, 2.0, 0.0, 0.0],
        10.0,
        1e-4,
        SimMethod::Rk4,
        50,
    )
    .expect("o pendulo duplo roda");

    assert!(
        resultado.accuracy.is_none(),
        "o pendulo duplo NAO tem solucao fechada, e a IDE tem de dizer que nao sabe"
    );
    let deriva = resultado
        .invariants
        .iter()
        .find(|i| i.id == "energia")
        .expect("ele declara a energia");
    assert!(
        deriva.drift < 1e-8,
        "sob RK4 com dt=1e-4 a energia do pendulo duplo deveria ficar parada, e derivou {:.3e}",
        deriva.drift
    );
}

#[test]
fn o_invariante_do_pendulo_bate_com_a_formula_lagrangiana() {
    // Verificacao independente do invariante: no repouso a 90 graus, a energia
    // e' so' a potencial, e ela vale -2*cos(pi/2) - cos(pi/2) = 0.
    let mut unitario = BTreeMap::new();
    unitario.insert("g".to_string(), 1.0);
    unitario.insert("l".to_string(), 1.0);
    let energia = invariante::energia_pendulo_duplo(
        &unitario,
        &[
            std::f64::consts::FRAC_PI_2,
            std::f64::consts::FRAC_PI_2,
            0.0,
            0.0,
        ],
    )
    .expect("a energia e calculavel com g e l");
    assert!(
        energia.abs() < 1e-12,
        "no repouso a 90 graus a energia do pendulo duplo e zero, e deu {energia}"
    );
    // E pendurado em repouso ela e' o minimo: -3.
    let minimo =
        invariante::energia_pendulo_duplo(&unitario, &[0.0, 0.0, 0.0, 0.0]).expect("calculavel");
    assert!(
        (minimo + 3.0).abs() < 1e-12,
        "pendurado em repouso a energia e -3, e deu {minimo}"
    );
}
