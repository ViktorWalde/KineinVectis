//! Integration tests for `Core` request dispatch, grouped by domain.
//!
//! Declared as `#[cfg(test)] mod tests;` in `lib.rs`, so `crate::` reaches the
//! core surface and every submodule shares [`core_with_empty_search_path`].

mod build;
mod dispatch;
mod fs;
mod lsp;
mod run;
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
