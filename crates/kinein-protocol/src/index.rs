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
    /// The compiler context the index loaded alongside the files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextSummary>,
}

/// Where the per-file compiler context comes from, in numbers.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSummary {
    /// `compile_commands.json` found (workspace-relative directory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_directory: Option<String>,
    /// Translation units in it.
    pub cdb_entries: u64,
    /// The CDB is older than a build file that defines it.
    pub cdb_stale: bool,
    /// The build file that is newer, workspace-relative, when `cdb_stale`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cdb_stale_because: Option<String>,
    /// Cargo packages seen by `cargo metadata --no-deps`.
    pub cargo_packages: u64,
    /// Cargo targets across them.
    pub cargo_targets: u64,
    /// Python interpreter resolved for the project, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_interpreter: Option<String>,
    /// How it was chosen (`VIRTUAL_ENV`, `.venv`, `poetry`, `sistema`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python_origin: Option<String>,
}

/// One C/C++ translation unit, as `compile_commands.json` describes it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileUnit {
    /// `argv[0]`: the compiler the build system invokes.
    pub compiler: String,
    /// Working directory of the command.
    pub directory: String,
    /// `-std=` value, when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    /// `-I`, `-isystem`, `-iquote` directories, absolute.
    pub includes: Vec<String>,
    /// `-D` definitions as written (`QT_CORE_LIB`, `FOO=1`).
    pub defines: Vec<String>,
    /// Object file, when the entry names it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// The full argument vector, for the panel that wants everything.
    pub arguments: Vec<String>,
}

/// The Cargo package and target that own a Rust file.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoUnit {
    /// Package name.
    pub package: String,
    /// Target name.
    pub target: String,
    /// `lib`, `bin`, `test`, `bench`, `example`, `proc-macro`.
    pub kind: String,
    /// Rust edition of the package.
    pub edition: String,
    /// `Cargo.toml` of the package.
    pub manifest: String,
    /// Target source root (`src/lib.rs`), absolute.
    pub src_path: String,
    /// Feature names the package declares.
    pub features: Vec<String>,
}

/// The Python interpreter a project resolves to (`roadmaps/29` §4.1 order).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnv {
    /// Interpreter path.
    pub interpreter: String,
    /// `python --version` output, when it ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `VIRTUAL_ENV`, `.venv`, `venv`, `poetry`, `sistema`.
    pub origin: String,
    /// Warning for the system interpreter: installing there breaks the distro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Result payload for `index.context`: how ONE file is compiled/run.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContext {
    /// The file asked about, absolute.
    pub path: String,
    /// `c`, `cpp`, `rust`, `python`, `other`.
    pub language: String,
    /// C/C++: the translation unit from the CDB, when the file is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<CompileUnit>,
    /// Rust: the package/target that owns the file.
    #[serde(default, rename = "crate", skip_serializing_if = "Option::is_none")]
    pub cargo: Option<CargoUnit>,
    /// Python: the interpreter the project resolves to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<PythonEnv>,
    /// Where the answer came from (`compile_commands.json em build/dev-local`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// What could not be decided, in words (header without a unit; file
    /// outside every cargo target; no interpreter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Parameters for `index.context`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IndexContextParams {
    /// Absolute or workspace-relative path.
    pub path: String,
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
