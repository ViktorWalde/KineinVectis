//! O catalogo: a TABELA das acoes de configuracao, e mais nada.
//!
//! DADO ESTATICO de proposito: nao ha registro dinamico nem contribuicao de
//! terceiro (`ARCHITECTURE.md` §8.1); acao nova e' uma entrada na tabela do
//! escopo dela (`cmake.rs`, `cargo.rs`, `platformio.rs`, separadas em
//! 2026-10-17) mais um planejador no modulo do arquivo que ela edita.
//! **Disponibilidade nao mora aqui**: quem mede o workspace e decide o estado
//! e' o [`super::availability`].

use kinein_protocol::{ConfigActionEffect, ConfigActionRisk, ConfigActionScope};

mod cargo;
mod cmake;
mod platformio;

/// Definicao estatica de uma acao, antes de olhar o workspace.
pub(super) struct ActionDefinition {
    /// Id estavel usado por `preview`/`apply`.
    pub(super) id: &'static str,
    /// Titulo curto.
    pub(super) title: &'static str,
    /// Descricao de uma linha.
    pub(super) description: &'static str,
    /// Build system dono da acao.
    pub(super) scope: ConfigActionScope,
    /// Categoria da lista.
    pub(super) category: &'static str,
    /// Risco declarado.
    pub(super) risk: ConfigActionRisk,
    /// Arquivos afetados, relativos a raiz.
    pub(super) affects: &'static [&'static str],
    /// O que `apply` faz.
    pub(super) effect: ConfigActionEffect,
    /// Parametros: `(nome, rotulo, obrigatorio, exemplo)`.
    pub(super) params: &'static [(&'static str, &'static str, bool, &'static str)],
    /// Documentacao associada: `(kind, titulo, url_key)`.
    pub(super) docs: &'static [(&'static str, &'static str, &'static str)],
}

/// As acoes, na ordem em que a UI as mostra: as do `CMake`, as do Cargo e as
/// do `PlatformIO`. O tamanho de cada tabela e' trava de compilacao sobre a
/// contagem: acrescentar sem atualizar reprova no build.
pub(super) fn definitions() -> impl Iterator<Item = &'static ActionDefinition> {
    cmake::ACTIONS
        .iter()
        .chain(cargo::ACTIONS.iter())
        .chain(platformio::ACTIONS.iter())
}

/// A definicao de um id, quando ele existe.
#[must_use]
pub(super) fn definition(id: &str) -> Option<&'static ActionDefinition> {
    definitions().find(|action| action.id == id)
}

const CMAKELISTS_ONLY: &[&str] = &["CMakeLists.txt"];
const PRESETS_ONLY: &[&str] = &["CMakePresets.json"];
const CARGO_ONLY: &[&str] = &["Cargo.toml"];
const CARGO_CONFIG_ONLY: &[&str] = &[".cargo/config.toml"];
const NO_FILE: &[&str] = &[];

const VISIBILITY_PARAM: (&str, &str, bool, &str) = ("visibility", "Visibilidade", false, "PRIVATE");
