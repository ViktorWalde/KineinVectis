//! Command descriptors advertised to the UI (command palette, menus, shortcuts).
//!
//! Pure data: every function returns `CommandDescriptor` values and never
//! touches `Core` state. Split out of `lib.rs` to keep the core surface small.
//!
//! # Por que isto virou pasta em 2026-09-01
//!
//! Este arquivo cresce com o NUMERO DE DOMINIOS, nao com a complexidade de
//! nenhum deles — e por isso passou de 500 linhas fazendo uma coisa so. A
//! `ARCHITECTURE.md` §4 regra 8 diz o que fazer nesse caso: "o projeto cresce
//! somando unidades, nao engordando unidades". Quando as Configuration Actions
//! precisaram de uma entrada na paleta, a catraca cobrou o arquivo em debito;
//! a saida certa nao era cortar linha nem subir o baseline, e sim dividir o
//! catalogo por AREA, como o `core_client_dispatch.cpp` foi dividido em
//! 2026-08-30. O que cresce agora e a contagem de arquivos.
//!
//! - [`ide`]: nucleo, workspace, arquivos e configuracoes;
//! - [`editor`]: formatacao, busca no arquivo e `lsp.*`;
//! - [`build`]: build, `CMake`, Cargo, jobs e Configuration Actions;
//! - [`run`]: run, run configurations e debug;
//! - [`git`]: o dominio Git.

mod build;
mod editor;
mod git;
mod ide;
mod run;

use kinein_protocol::CommandDescriptor;

/// Returns every command descriptor advertised to the UI via `command.list`.
pub(crate) fn command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = ide::core_command_descriptors();
    descriptors.extend(ide::project_command_descriptors());
    descriptors.extend(build::build_command_descriptors());
    descriptors.extend(run::run_command_descriptors());
    descriptors.extend(editor::lsp_command_descriptors());
    descriptors.extend(build::jobs_command_descriptors());
    descriptors.extend(editor::format_command_descriptors());
    descriptors.extend(editor::editor_find_command_descriptors());
    descriptors.extend(build::cmake_command_descriptors());
    descriptors.extend(build::cargo_command_descriptors());
    descriptors.extend(build::configaction_command_descriptors());
    descriptors.extend(build::library_command_descriptors());
    descriptors.extend(build::datasource_command_descriptors());
    descriptors.extend(build::grafana_command_descriptors());
    descriptors.extend(build::probe_command_descriptors());
    descriptors.extend(build::setup_command_descriptors());
    descriptors.extend(run::runconfig_command_descriptors());
    descriptors.extend(run::debug_command_descriptors());
    descriptors.extend(git::git_command_descriptors());
    descriptors.extend(ide::settings_command_descriptors());
    descriptors
}
