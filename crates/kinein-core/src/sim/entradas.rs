//! A tabela dos conceitos de UMA EQUACAO: algebricos e EDO escalar.
//!
//! Saiu do `catalogo.rs` em 2026-09-06 pelo mesmo corte que separou o
//! `exata.rs`: **a API de leitura e a tabela mudam por motivos diferentes**. A
//! API muda quando o catalogo ganha uma regra nova; a tabela muda toda vez que
//! um conceito entra — que e' o gesto que a `arquitetura/34` §4.2 quer barato.
//!
//! ## O que uma entrada NAO traz
//!
//! **A formula.** Ela e' do usuario: o conceito declara QUAIS grandezas
//! participam, e a checagem confronta uma coisa com a outra. Um catalogo com a
//! formula pronta responderia a pergunta errada.
//!
//! ## Por que os conceitos de SISTEMA moram noutro arquivo
//!
//! Nao e' tamanho: **uma entrada de sistema declara cinco coisas que uma escalar
//! nao tem** — os componentes de estado, o pareamento posicao/velocidade, o
//! plano da vista de trajetoria, os invariantes e a solucao fechada vetorial. E
//! auditar uma e' um trabalho diferente de auditar a outra: aqui se confere uma
//! formula fechada contra um livro; la' se confere um pareamento que decide se
//! um METODO esta' definido. Ver `entradas_sistema.rs`.
//!
//! ## A regra de admissao
//!
//! **Entrada nao auditada nao entra**, e ela carrega fonte com data — mesma
//! regra do `setup.list`. Um conceito e' uma afirmacao de que a conta esta'
//! certa, e o catalogo e' divida por entrada.

use kinein_protocol::{SimForm, SimView};

use super::catalogo::{Entrada, G};
use super::exata;

/// Fonte comum das entradas de mecanica e eletromagnetismo desta fatia.
pub(super) const HALLIDAY: &str = "Halliday, Resnick & Walker, Fundamentos de Fisica \
                        (formulacao padrao de curso); revisado em 2026-09-05";
/// Fonte das entradas de sistema, acrescentada em 2026-09-06.
pub(super) const GOLDSTEIN: &str = "Goldstein, Classical Mechanics (formulacao \
                         lagrangiana padrao); revisado em 2026-09-06";
/// Fonte comum das entradas de calculo.
pub(super) const STEWART: &str = "Stewart, Calculo (formulacao padrao de curso); \
                       revisado em 2026-09-05";

pub(super) const ESCALARES: &[Entrada] = &[
    Entrada {
        id: "oscilador-amortecido",
        nome: "Oscilador harmonico amortecido",
        resumo: "Massa-mola com atrito: a amplitude decai enquanto oscila.",
        curso: "Fisica I",
        forma: SimForm::Ode2,
        vista: SimView::Plot2d,
        grandezas: &[
            G("m", "massa", "kg", true),
            G("k", "constante elastica", "N/m", true),
            G("c", "coeficiente de amortecimento", "N.s/m", false),
            G("y", "posicao", "m", true),
            G("dy", "velocidade", "m/s", false),
        ],
        fechada: true,
        exata: Some(exata::oscilador_amortecido),
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "queda-livre",
        nome: "Queda livre",
        resumo: "Corpo sob aceleracao constante da gravidade, sem arrasto.",
        curso: "Fisica basica",
        forma: SimForm::Ode2,
        vista: SimView::Plot2d,
        grandezas: &[G("g", "aceleracao da gravidade", "m/s^2", true)],
        fechada: true,
        exata: Some(exata::queda_livre),
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "decaimento-exponencial",
        nome: "Decaimento exponencial",
        resumo: "Grandeza que decai a taxa proporcional a ela mesma.",
        curso: "Calculo II",
        forma: SimForm::Ode1,
        vista: SimView::Plot2d,
        grandezas: &[
            G("k", "constante de decaimento", "1/s", true),
            G("y", "valor atual", "", true),
        ],
        fechada: true,
        exata: Some(exata::decaimento),
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: STEWART,
    },
    Entrada {
        id: "mru",
        nome: "Movimento retilineo uniforme",
        resumo: "Posicao em funcao do tempo com velocidade constante.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("x0", "posicao inicial", "m", true),
            G("v", "velocidade", "m/s", true),
            G("t", "tempo", "s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "mruv",
        nome: "Movimento retilineo uniformemente variado",
        resumo: "Posicao em funcao do tempo com aceleracao constante.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("x0", "posicao inicial", "m", true),
            G("v0", "velocidade inicial", "m/s", true),
            G("a", "aceleracao", "m/s^2", true),
            G("t", "tempo", "s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        // Entrou em 2026-09-05 porque um teste de MUTACAO mostrou que a
        // fronteira de identificador da substituicao estava SEM COBERTURA:
        // nenhum conceito do catalogo punha `v` e `v0` na MESMA formula, que e'
        // o caso em que a substituicao ingenua transforma `v0` em "50" ao
        // trocar o `v` por 5 e deixar o `0`. O impulso poe as duas.
        id: "impulso",
        nome: "Impulso e variacao da quantidade de movimento",
        resumo: "Impulso de uma forca igual a variacao da quantidade de movimento.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("m", "massa", "kg", true),
            G("v", "velocidade final", "m/s", true),
            G("v0", "velocidade inicial", "m/s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "torricelli",
        nome: "Equacao de Torricelli",
        resumo: "Velocidade em funcao do deslocamento, sem passar pelo tempo.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("v0", "velocidade inicial", "m/s", true),
            G("a", "aceleracao", "m/s^2", true),
            G("dx", "deslocamento", "m", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "energia-cinetica",
        nome: "Energia cinetica",
        resumo: "Energia associada ao movimento de um corpo.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("m", "massa", "kg", true),
            G("v", "velocidade", "m/s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "trabalho-forca-constante",
        nome: "Trabalho de forca constante",
        resumo: "Trabalho de uma forca constante ao longo de um deslocamento.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("f", "forca", "N", true),
            G("d", "deslocamento", "m", true),
            G("theta", "angulo entre forca e deslocamento", "rad", false),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "gases-ideais",
        nome: "Lei dos gases ideais",
        resumo: "Relacao entre pressao, volume, quantidade e temperatura.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("n", "quantidade de materia", "mol", true),
            G("r", "constante universal dos gases", "J/(mol.K)", true),
            G("t", "temperatura", "K", true),
            G("v", "volume", "m^3", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "lei-de-ohm",
        nome: "Lei de Ohm",
        resumo: "Tensao, corrente e resistencia num condutor ohmico.",
        curso: "Fisica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("i", "corrente", "A", true),
            G("r", "resistencia", "ohm", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "coulomb",
        nome: "Lei de Coulomb",
        resumo: "Forca entre duas cargas puntiformes.",
        curso: "Fisica III",
        forma: SimForm::Algebraic,
        vista: SimView::Space3d,
        grandezas: &[
            G("k", "constante eletrostatica", "N.m^2/C^2", true),
            G("q1", "primeira carga", "C", true),
            G("q2", "segunda carga", "C", true),
            G("d", "distancia entre as cargas", "m", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "lorentz",
        nome: "Fator de Lorentz",
        resumo: "Dilatacao temporal e contracao espacial na relatividade restrita.",
        curso: "Fisica IV",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("v", "velocidade do referencial", "m/s", true),
            G("c", "velocidade da luz", "m/s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "fotoeletrico",
        nome: "Efeito fotoeletrico",
        resumo: "Energia cinetica maxima do eletron arrancado por um foton.",
        curso: "Fisica IV",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("h", "constante de Planck", "J.s", true),
            G("f", "frequencia da luz", "Hz", true),
            G("phi", "funcao trabalho do material", "J", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
    Entrada {
        id: "funcao-quadratica",
        nome: "Funcao quadratica",
        resumo: "Parabola: a forma canonica de segundo grau.",
        curso: "Matematica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("a", "coeficiente de x^2", "", true),
            G("b", "coeficiente de x", "", true),
            G("c", "termo independente", "", true),
            G("x", "variavel independente", "", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: STEWART,
    },
    Entrada {
        id: "juros-compostos",
        nome: "Crescimento exponencial",
        resumo: "Grandeza que cresce ou decai a taxa proporcional a si mesma.",
        curso: "Matematica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Plot2d,
        grandezas: &[
            G("a0", "valor inicial", "", true),
            G("k", "taxa", "1/s", true),
            G("t", "tempo", "s", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: STEWART,
    },
    Entrada {
        id: "distancia-euclidiana",
        nome: "Distancia entre dois pontos no espaco",
        resumo: "Norma euclidiana da diferenca entre dois pontos.",
        curso: "Matematica basica",
        forma: SimForm::Algebraic,
        vista: SimView::Space3d,
        grandezas: &[
            G("dx", "diferenca em x", "m", true),
            G("dy", "diferenca em y", "m", true),
            G("dz", "diferenca em z", "m", true),
        ],
        fechada: true,
        exata: None,
        componentes: &[],
        pares: &[],
        plano: None,
        invariantes: &[],
        exata_sistema: None,
        fonte: STEWART,
    },
];
