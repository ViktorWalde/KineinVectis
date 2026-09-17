//! Request handlers grouped by domain, each implemented as `impl Core` blocks.
//!
//! The top-level dispatch (`handle_request`, `service_request_response`) and the
//! shared `workspace_root` stay in `lib.rs`; these modules hold the per-domain
//! routers and leaf handlers.

pub mod build;
pub mod cargo;
pub mod cmake;
pub mod configaction;
pub mod container;
pub mod datasource;
pub mod debug;
pub mod draft;
pub mod format;
pub mod fs;
pub mod git;
pub mod grafana;
pub mod index;
pub mod jobs;
pub mod library;
pub mod lsp;
pub mod probe;
pub mod project;
pub mod python;
pub mod python_stubs;
pub mod run;
pub mod runconfig;
pub mod serial;
pub mod serial_files;
pub mod settings;
pub mod setup;
pub mod syntax;
pub mod terminal;
pub mod test_discover;
pub mod toolchain;
pub mod toolchain_import;
pub mod toolchain_install;
pub mod tools;
pub mod workspace;
