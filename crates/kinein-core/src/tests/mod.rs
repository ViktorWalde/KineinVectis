//! Integration tests for `Core` request dispatch, grouped by domain.
//!
//! Declared as `#[cfg(test)] mod tests;` in `lib.rs`, so `crate::` reaches the
//! core surface and every submodule shares [`core_with_empty_search_path`].

use crate::EXECUTAVEIS;

mod build;
mod cargo;
mod cmake;
mod cmake_model;
mod configaction;
mod container;
mod coverage;
mod datasource;
mod datasource_query;
mod debug;
mod debug_attach;
mod debug_inspect;
mod dispatch;
mod flash_proposal;
mod format;
mod frameworks;
mod fs;
mod git;
mod grafana;
mod index;
mod index_context;
mod jobs;
mod lsp;
mod lsp_companion;
mod lsp_deferred;
mod lsp_server;
mod lsp_stderr;
mod project;
mod python;
mod remote;
mod remote_mirror;
mod run;
mod runconfig;
mod runners;
mod serial;
mod serial_files;
mod serial_identify;
mod settings;
mod syntax;
mod terminal;
mod toolchain;
mod tools;
mod workspace;

use crate::Core;
use crate::tools::ToolDetector;

/// Builds a `Core` whose tool detector searches an empty temp directory, so
/// detection is deterministic and never finds host tools.
fn core_with_empty_search_path(test_name: &str) -> Core {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dispatch-{test_name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    Core::with_detector(ToolDetector::with_search_path(dir))
}
