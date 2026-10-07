//! Versioned database adapter contract, separate from the UI/core IPC.
//!
//! These types do not launch tools, connect to databases or persist profiles.

mod api;
mod error;
mod object;
pub mod operation;
mod response;

pub use api::*;
pub use error::*;
pub use object::deserialize_object;
pub use response::*;

#[cfg(test)]
mod tests;
