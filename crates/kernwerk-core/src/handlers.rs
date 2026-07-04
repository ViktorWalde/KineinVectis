//! Request handlers grouped by domain, each implemented as `impl Core` blocks.
//!
//! The top-level dispatch (`handle_request`, `service_request_response`) and the
//! shared `workspace_root` stay in `lib.rs`; these modules hold the per-domain
//! routers and leaf handlers.

pub mod build;
pub mod fs;
pub mod lsp;
pub mod run;
pub mod terminal;
pub mod workspace;
