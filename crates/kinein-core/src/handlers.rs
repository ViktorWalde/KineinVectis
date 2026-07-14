//! Request handlers grouped by domain, each implemented as `impl Core` blocks.
//!
//! The top-level dispatch (`handle_request`, `service_request_response`) and the
//! shared `workspace_root` stay in `lib.rs`; these modules hold the per-domain
//! routers and leaf handlers.

pub mod ai;
pub mod build;
pub mod cargo;
pub mod cmake;
pub mod debug;
pub mod draft;
pub mod format;
pub mod fs;
pub mod git;
pub mod jobs;
pub mod lsp;
pub mod run;
pub mod runconfig;
pub mod settings;
pub mod syntax;
pub mod terminal;
pub mod workspace;
