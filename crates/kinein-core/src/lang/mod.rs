//! Incremental local syntax intelligence backed by Tree-sitter.
//!
//! This layer only owns structure-local features. Project semantics and safe
//! refactorings remain delegated to clangd and rust-analyzer through `lsp`.

mod folding;
mod outline;
mod positions;
pub(crate) mod registry;
pub(crate) mod service;

pub(crate) use service::{SyntaxTreeError, SyntaxTreeService};
