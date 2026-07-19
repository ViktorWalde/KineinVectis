//! Shared JSON-RPC protocol types for Kinein Vectis.
//!
//! This crate is intentionally UI-agnostic. Qt/QML talks to these structures
//! through serialized JSON, while the Rust core owns command execution.
//!
//! Types are grouped by protocol domain in submodules and re-exported flat, so
//! consumers keep using `kinein_protocol::TypeName` regardless of the domain
//! a type lives in.

#![forbid(unsafe_code)]

mod build;
mod cargo;
mod cmake;
mod command;
mod core;
mod debug;
mod diagnostic;
mod draft;
mod format;
mod fs;
mod git;
mod integration;
mod job;
mod lsp;
mod rpc;
mod run;
mod runconfig;
mod settings;
mod syntax;
mod terminal;
mod tools;
mod workspace;

pub use build::*;
pub use cargo::*;
pub use cmake::*;
pub use command::*;
pub use core::*;
pub use debug::*;
pub use diagnostic::*;
pub use draft::*;
pub use format::*;
pub use fs::*;
pub use git::*;
pub use integration::*;
pub use job::*;
pub use lsp::*;
pub use rpc::*;
pub use run::*;
pub use runconfig::*;
pub use settings::*;
pub use syntax::*;
pub use terminal::*;
pub use tools::*;
pub use workspace::*;

/// JSON-RPC protocol version used by Kinein Vectis.
pub const JSON_RPC_VERSION: &str = "2.0";

/// Kinein Vectis IPC protocol version implemented by this workspace.
pub const PROTOCOL_VERSION: &str = "0.62.0";
