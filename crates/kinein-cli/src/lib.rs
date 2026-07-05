//! CLI helper library for generating Kinein Vectis JSON-RPC requests.
//!
//! The binary (`main.rs`) is a thin shim over [`run`]; the dispatch and error
//! type live here so they can be unit-tested as a library. Each `run` call
//! parses argv and emits exactly one JSON-RPC request to stdout.

#![forbid(unsafe_code)]

mod commands;
mod error;

pub use commands::run;
pub use error::CliError;
