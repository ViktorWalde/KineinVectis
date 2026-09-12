//! Integration tests for `Core` request dispatch, grouped by domain.
//!
//! Declared as `#[cfg(test)] mod tests;` in `lib.rs`, so `crate::` reaches the
//! core surface and every submodule shares [`core_with_empty_search_path`].

mod build;
mod cargo;
mod cmake;
mod configaction;
mod datasource;
mod debug;
mod dispatch;
mod format;
mod fs;
mod git;
mod grafana;
mod jobs;
mod lsp;
mod lsp_server;
mod run;
mod runconfig;
mod runners;
mod serial;
mod settings;
mod sim;
mod sim_integrador;
mod sim_oraculo;
mod sim_persistencia;
mod sim_sistema;
mod syntax;
mod terminal;
mod toolchain;
mod tools;
mod workspace;

use crate::Core;
use crate::tools::ToolDetector;

/// Builds a `Core` whose tool detector searches an empty temp directory, so
/// detection is deterministic and never finds host tools.
///
/// **E cujo oráculo de simulação também não alcança o host.** Sem isso, o
/// resultado de `sim.run` passaria a depender de a máquina ter `SymPy`
/// instalado: a coluna `exato` mudaria de procedência, e a suíte ficaria verde
/// aqui e vermelha ali. É a mesma razão do caminho de busca vazio, no domínio
/// vizinho — e a mesma classe de falha da qual o `verificar-shell.sh` nasceu.
fn core_with_empty_search_path(test_name: &str) -> Core {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dispatch-{test_name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = Core::with_detector(ToolDetector::with_search_path(dir));
    core.set_oraculo(crate::sim::oraculo::Config {
        // Um caminho que nao existe: o oraculo responde "sem Python" na hora, e
        // a corrida cai no caminho do CONCEITO, que e' deterministico.
        interpretador: "kinein-oraculo-ausente-de-proposito".to_owned(),
        teto: std::time::Duration::from_secs(1),
    });
    core
}
