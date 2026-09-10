//! O catalogo: a TABELA de conceitos, e mais nada.
//!
//! E a camada de CIMA do desenho de duas camadas
//! (`docs/arquitetura/34-simulacao-por-conceito.md` §4): o nome que o usuario
//! conhece, apontando para a FORMA matematica que o motor resolve. E' isso que
//! torna "todos os conceitos" sustentavel — conceito novo e' uma entrada aqui,
//! nao codigo novo.
//!
//! **Dado estatico de proposito**, como o `configaction::catalog`: nao ha
//! registro dinamico nem contribuicao de terceiro neste projeto
//! (`ARCHITECTURE.md` §8.1).
//!
//! **O que este arquivo NAO faz:** ele nao guarda a formula. A formula e' do
//! usuario — o conceito declara apenas QUAIS grandezas participam, e a
//! checagem em [`super::formula`] confronta uma coisa com a outra. Um catalogo
//! que trouxesse a formula pronta responderia a pergunta errada: o pedido do
//! autor em 2026-09-01 e' que a equacao seja informada por ele.
//!
//! **Escopo desta fatia.** O apendice A da `arquitetura/34` mapeia 100
//! conceitos; as entradas abaixo sao as da forma ALGEBRICA, que e' a mais
//! barata e carrega 66 dos 100. Crescer o catalogo e' acrescentar linhas aqui.

use kinein_protocol::{
    SimConcept, SimForm, SimInvariantInfo, SimPair, SimQuantity, SimSystemComponent, SimView,
};

use super::entradas::ESCALARES;
use super::entradas_sistema::SISTEMAS;
use super::exata::{Fechada, FechadaSistema};
use super::invariante::Invariante;

/// Uma grandeza declarada, na forma compacta da tabela.
pub(super) struct G(
    pub(super) &'static str,
    pub(super) &'static str,
    pub(super) &'static str,
    pub(super) bool,
);

/// Uma entrada do catalogo, antes de virar tipo de protocolo.
pub(super) struct Entrada {
    pub(super) id: &'static str,
    pub(super) nome: &'static str,
    pub(super) resumo: &'static str,
    pub(super) curso: &'static str,
    pub(super) forma: SimForm,
    pub(super) vista: SimView,
    pub(super) grandezas: &'static [G],
    pub(super) fechada: bool,
    /// A solucao exata ESCALAR, quando a IDE sabe escreve-la.
    pub(super) exata: Option<Fechada>,
    /// Os componentes de estado, na ordem — so' da forma `SISTEMA_EDO`.
    pub(super) componentes: &'static [C],
    /// Os pares posicao/velocidade DECLARADOS. Vazio = sem simpletico.
    pub(super) pares: &'static [(usize, usize)],
    /// O par de componentes que a vista de trajetoria desenha.
    pub(super) plano: Option<(usize, usize)>,
    /// As grandezas conservadas que este conceito declara.
    pub(super) invariantes: &'static [(&'static str, &'static str, &'static str, Invariante)],
    /// A solucao exata VETORIAL, quando ela existe.
    pub(super) exata_sistema: Option<FechadaSistema>,
    pub(super) fonte: &'static str,
}

/// Um componente de estado, na forma compacta da tabela.
pub(super) struct C(
    pub(super) &'static str,
    pub(super) &'static str,
    pub(super) &'static str,
);

/// As duas tabelas, lidas como uma so'.
///
/// A ordem e' a da UI: os conceitos de uma equacao primeiro, os de sistema
/// depois — e ela e' estavel porque as duas tabelas sao estaticas.
fn entradas() -> impl Iterator<Item = &'static Entrada> {
    ESCALARES.iter().chain(SISTEMAS.iter())
}

/// A solucao fechada de um conceito, quando ha' uma.
#[must_use]
pub fn exata(id: &str) -> Option<Fechada> {
    entradas().find(|e| e.id == id).and_then(|e| e.exata)
}

/// A solucao fechada VETORIAL de um conceito, quando ha' uma.
#[must_use]
pub fn exata_sistema(id: &str) -> Option<FechadaSistema> {
    entradas()
        .find(|e| e.id == id)
        .and_then(|e| e.exata_sistema)
}

/// A funcao que calcula um invariante declarado por um conceito.
#[must_use]
pub fn invariante(conceito: &str, invariante: &str) -> Option<Invariante> {
    entradas()
        .find(|e| e.id == conceito)?
        .invariantes
        .iter()
        .find(|(id, _, _, _)| *id == invariante)
        .map(|(_, _, _, f)| *f)
}

/// Os conceitos, na ordem em que a UI os mostra.
#[must_use]
pub fn conceitos() -> Vec<SimConcept> {
    entradas().map(monta).collect()
}

/// Um conceito pelo id, quando existe.
#[must_use]
pub fn conceito(id: &str) -> Option<SimConcept> {
    entradas().find(|e| e.id == id).map(monta)
}

fn monta(e: &Entrada) -> SimConcept {
    SimConcept {
        id: e.id.to_string(),
        name: e.nome.to_string(),
        summary: e.resumo.to_string(),
        course: e.curso.to_string(),
        form: e.forma,
        view: e.vista,
        quantities: e
            .grandezas
            .iter()
            .map(|G(id, rotulo, unidade, obrigatoria)| SimQuantity {
                id: (*id).to_string(),
                label: (*rotulo).to_string(),
                unit: (*unidade).to_string(),
                required: *obrigatoria,
            })
            .collect(),
        components: e
            .componentes
            .iter()
            .map(|C(id, rotulo, unidade)| SimSystemComponent {
                id: (*id).to_string(),
                label: (*rotulo).to_string(),
                unit: (*unidade).to_string(),
            })
            .collect(),
        pairing: e
            .pares
            .iter()
            .map(|(a, b)| SimPair {
                first: *a,
                second: *b,
            })
            .collect(),
        plane: e.plano.map(|(a, b)| SimPair {
            first: a,
            second: b,
        }),
        invariants: e
            .invariantes
            .iter()
            .map(|(id, rotulo, unidade, _)| SimInvariantInfo {
                id: (*id).to_string(),
                label: (*rotulo).to_string(),
                unit: (*unidade).to_string(),
            })
            .collect(),
        closed_form: e.fechada,
        source: e.fonte.to_string(),
    }
}
