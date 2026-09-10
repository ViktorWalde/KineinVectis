//! A tabela dos conceitos de SISTEMA: `dY/dt = F(t, Y)`.
//!
//! Separada de [`super::entradas`] em 2026-09-06, e nao por tamanho. **Uma
//! entrada daqui declara cinco coisas que uma escalar nao tem**, e cada uma
//! delas e' uma afirmacao diferente que alguem tem de auditar:
//!
//! ```text
//! componentes     a ordem do estado, que as formulas e a trilha seguem
//! pares           quais componentes sao posicao e velocidade um do outro.
//!                 DECIDE se o metodo simpletico esta' definido: sem o par ele
//!                 nao esta', e o conceito nao o oferece (`arquitetura/34` §13.3)
//! plano           o par que a vista de trajetoria desenha
//! invariantes     as grandezas que a fisica conserva — o sinal de exatidao que
//!                 sobra quando nao ha' solucao fechada
//! exata_sistema   a solucao fechada vetorial, quando ela existe. Na orbita ela
//!                 vale SO' no caso circular; a eliptica exige Kepler
//! ```
//!
//! Auditar uma entrada escalar e' conferir uma formula fechada contra um livro.
//! Auditar uma daqui e' conferir tambem um pareamento e um invariante — e o
//! invariante do pendulo duplo foi DERIVADO com o `SymPy` em 2026-09-06 depois de
//! a versao escrita de memoria reprovar no teste de conservacao.

use kinein_protocol::{SimForm, SimView};

use super::catalogo::{C, Entrada, G};
use super::{exata, invariante};

use super::entradas::{GOLDSTEIN, HALLIDAY};

pub(super) const SISTEMAS: &[Entrada] = &[
    // --- a forma SISTEMA_EDO, acrescentada em 2026-09-06 -------------------
    //
    // Ela e' de PRIMEIRA ORDEM (`arquitetura/34` §13.1): a segunda ordem entra
    // escrevendo as velocidades como componentes, que e' a reducao padrao. Por
    // isso a orbita tem QUATRO componentes e nao dois.
    Entrada {
        id: "orbita-dois-corpos",
        nome: "Orbita de dois corpos",
        resumo: "Corpo em orbita sob gravitacao newtoniana, no plano.",
        curso: "Fisica I",
        forma: SimForm::OdeSystem,
        // A vista natural e' 3D, mas a trajetoria de uma orbita plana e' 2D de
        // verdade — nao e' projecao de conveniencia. A vista 3D depende do
        // `kinein-sim`, que e' fatia propria (`arquitetura/34` §13.6).
        vista: SimView::Plot2d,
        grandezas: &[G("mu", "parametro gravitacional (G*M)", "m^3/s^2", true)],
        // A solucao fechada existe SO' no caso circular; a eliptica exige a
        // equacao de Kepler, que e' transcendental. O `exata_sistema` recusa
        // fora do circular, entao a tela diz que nao sabe em vez de mentir.
        fechada: false,
        exata: None,
        componentes: &[
            C("x", "posicao x", "m"),
            C("y", "posicao y", "m"),
            C("vx", "velocidade x", "m/s"),
            C("vy", "velocidade y", "m/s"),
        ],
        // O pareamento DECLARADO: (x, vx) e (y, vy). Sem ele o simpletico nao
        // esta' definido — e ele vale oito ordens de grandeza aqui.
        pares: &[(0, 2), (1, 3)],
        plano: Some((0, 1)),
        invariantes: &[
            (
                "energia",
                "energia mecanica especifica",
                "J/kg",
                invariante::energia_orbita,
            ),
            (
                "momento-angular",
                "momento angular especifico",
                "m^2/s",
                invariante::momento_angular_orbita,
            ),
        ],
        exata_sistema: Some(exata::orbita_circular),
        fonte: HALLIDAY,
    },
    Entrada {
        id: "pendulo-duplo",
        nome: "Pendulo duplo",
        resumo: "Dois pendulos acoplados: o exemplo de caos determinista.",
        curso: "Fisica I",
        forma: SimForm::OdeSystem,
        vista: SimView::Plot2d,
        // `g` e `l` sao DECLARADOS em vez de normalizados a 1. Um catalogo com
        // a normalizacao embutida seria a IDE decidindo a fisica, que e' o que
        // a §2.1 proibe — e as duas aparecem nas equacoes que o usuario escreve.
        grandezas: &[
            G("g", "aceleracao da gravidade", "m/s^2", true),
            G("l", "comprimento de cada haste", "m", true),
        ],
        // NAO tem solucao fechada, e nao e' limitacao da IDE: ela nao existe.
        // Medido em 2026-09-06 (`roadmaps/31` §19.1.4), acima de uma energia
        // ele e' caotico — duas corridas que diferem por 1e-12 divergem a
        // 0,38/s. O invariante e' o unico sinal de exatidao que ele admite.
        fechada: false,
        exata: None,
        componentes: &[
            C("t1", "angulo do primeiro", "rad"),
            C("t2", "angulo do segundo", "rad"),
            C("w1", "velocidade angular do primeiro", "rad/s"),
            C("w2", "velocidade angular do segundo", "rad/s"),
        ],
        pares: &[(0, 2), (1, 3)],
        plano: Some((0, 1)),
        // Energia POR UNIDADE DE MASSA: ela precisa so' de `g` e `l`, e assim
        // toda grandeza declarada aparece nas equacoes. Exigir uma massa que a
        // equacao de movimento nao usa (ela cancela) daria uma grandeza
        // obrigatoria impossivel de ligar.
        invariantes: &[(
            "energia",
            "energia por unidade de massa",
            "J/kg",
            invariante::energia_pendulo_duplo,
        )],
        exata_sistema: None,
        fonte: GOLDSTEIN,
    },
    Entrada {
        id: "massa-mola-acoplada",
        nome: "Duas massas acopladas por molas",
        resumo: "Duas massas ligadas por tres molas: os modos normais.",
        curso: "Fisica II",
        forma: SimForm::OdeSystem,
        vista: SimView::Plot2d,
        grandezas: &[
            G("m", "massa de cada corpo", "kg", true),
            G("k", "constante das molas das pontas", "N/m", true),
            G("kc", "constante da mola de acoplamento", "N/m", true),
        ],
        fechada: false,
        exata: None,
        componentes: &[
            C("x1", "deslocamento do primeiro", "m"),
            C("x2", "deslocamento do segundo", "m"),
            C("v1", "velocidade do primeiro", "m/s"),
            C("v2", "velocidade do segundo", "m/s"),
        ],
        pares: &[(0, 2), (1, 3)],
        plano: Some((0, 1)),
        invariantes: &[(
            "energia",
            "energia mecanica",
            "J",
            invariante::energia_massa_mola,
        )],
        exata_sistema: None,
        fonte: HALLIDAY,
    },
];
