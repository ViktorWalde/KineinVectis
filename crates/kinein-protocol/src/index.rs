//! Types for the `index.*` domain: the IDE's own index of the WHOLE project.
//!
//! Decision of the author on 2026-09-12 (`roadmaps/42` P0): the IDE reads the
//! entire project it opens — every folder, every file, every function and
//! type — for C, C++, Rust and Python, without waiting for a language server.
//! Language servers keep the deep semantics (types, references, rename); this
//! index is the structural map that exists from the first second, built with
//! the same Tree-sitter grammars the editor already uses.

use serde::{Deserialize, Serialize};

/// One declaration found by a grammar's tags query.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSymbol {
    /// Declared name.
    pub name: String,
    /// Kind from the tags query (`function`, `method`, `class`, `struct`,
    /// `enum`, `module`, `interface`, `macro`, `type`...).
    pub kind: String,
    /// Workspace-relative file path.
    pub path: String,
    /// Language id (`c`, `cpp`, `rust`, `python`).
    pub language: String,
    /// One-based line of the declaration.
    pub line: u64,
    /// One-based last line of the declaration.
    pub end_line: u64,
    /// Enclosing declaration name (`impl Foo` → `Foo`), when nested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
}

/// Per-language totals.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageStats {
    /// `c`, `cpp`, `rust`, `python`, `other`.
    pub language: String,
    /// Files of this language.
    pub files: u64,
    /// Lines across those files.
    pub lines: u64,
    /// Declarations extracted (0 when the language has no grammar yet).
    pub symbols: u64,
}

/// Where the index is in its life.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndexState {
    /// No workspace, or not started.
    Idle,
    /// A build is running; `files` grows as it goes.
    Building,
    /// Built; incremental updates keep it current.
    Ready,
    /// The build was cancelled or failed; `error` says why.
    Failed,
}

/// Result payload for `index.status`, and payload of `event.index.finished`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    /// Life-cycle state.
    pub state: IndexState,
    /// Folders walked (skip list excluded).
    pub folders: u64,
    /// Every file seen.
    pub files: u64,
    /// Files with a recognised language.
    pub source_files: u64,
    /// Lines across source files.
    pub lines: u64,
    /// Bytes across source files.
    pub bytes: u64,
    /// Declarations across the index.
    pub symbols: u64,
    /// Functions and methods among them.
    pub functions: u64,
    /// Types among them (struct, class, enum, union, interface, type, trait).
    pub types: u64,
    /// Totals per language, most files first.
    pub by_language: Vec<LanguageStats>,
    /// Files the indexer could not read or parse, workspace-relative.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<String>,
    /// Wall time of the last full build.
    pub elapsed_ms: u64,
    /// Why `Failed`, when it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parameters for `index.status`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexStatusParams {}

/// Parameters for `index.symbols`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IndexSymbolsParams {
    /// Case-insensitive needle; empty lists everything up to `limit`.
    pub query: String,
    /// Maximum results (default 200).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Restrict to a kind (`function`, `class`...), when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Result payload for `index.symbols`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSymbolsResult {
    /// Matches, best first: exact name, then prefix, then substring.
    pub symbols: Vec<IndexSymbol>,
    /// Matches before `limit` was applied.
    pub total: usize,
    /// The index state at query time — `building` means the answer is partial.
    pub state: IndexState,
}

/// Payload of `event.index.progress`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgressEvent {
    /// Files scanned so far.
    pub files: u64,
    /// Declarations extracted so far.
    pub symbols: u64,
}
