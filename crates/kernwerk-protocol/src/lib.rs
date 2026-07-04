//! Shared JSON-RPC protocol types for Kernwerk Studio.
//!
//! This crate is intentionally UI-agnostic. Qt/QML talks to these structures
//! through serialized JSON, while the Rust core owns command execution.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC protocol version used by Kernwerk Studio.
pub const JSON_RPC_VERSION: &str = "2.0";

/// Kernwerk IPC protocol version implemented by this workspace.
pub const PROTOCOL_VERSION: &str = "0.19.0";

/// JSON-RPC request identifier.
///
/// JSON-RPC allows string, number, or null identifiers. The protocol keeps this
/// as a JSON value so the UI can preserve the exact identifier it sent.
pub type JsonRpcId = Value;

/// JSON-RPC request sent from a client to the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version. Kernwerk currently accepts only `2.0`.
    pub jsonrpc: String,
    /// Request identifier. Events and notifications omit this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    /// Method name, such as `core.ping`.
    pub method: String,
    /// Method parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Builds a JSON-RPC request with an identifier.
    #[must_use]
    pub fn new(id: impl Into<JsonRpcId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id: Some(id.into()),
            method: method.into(),
            params,
        }
    }

    /// Builds a JSON-RPC notification without an identifier.
    #[must_use]
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id: None,
            method: method.into(),
            params,
        }
    }

    /// Returns `true` when the request uses the supported JSON-RPC version.
    #[must_use]
    pub fn has_supported_version(&self) -> bool {
        self.jsonrpc == JSON_RPC_VERSION
    }
}

/// JSON-RPC response emitted by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version. Always `2.0`.
    pub jsonrpc: String,
    /// Identifier copied from the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    /// Successful response payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error response payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Builds a successful JSON-RPC response.
    #[must_use]
    pub fn success(id: Option<JsonRpcId>, result: Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Builds an error JSON-RPC response.
    #[must_use]
    pub fn failure(id: Option<JsonRpcId>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Error object used by JSON-RPC responses.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Stable machine-readable error code.
    pub code: JsonRpcErrorCode,
    /// Human-readable message suitable for logs or UI notifications.
    pub message: String,
    /// Structured technical details for diagnostics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl JsonRpcError {
    /// Builds a protocol error with optional structured details.
    #[must_use]
    pub fn new(code: JsonRpcErrorCode, message: impl Into<String>, details: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            details,
        }
    }
}

/// Stable error codes used by Kernwerk Studio.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JsonRpcErrorCode {
    /// The input was not valid JSON.
    ParseError,
    /// The JSON value was not a valid Kernwerk request.
    InvalidRequest,
    /// The requested method is not registered.
    MethodNotFound,
    /// Request parameters failed validation.
    InvalidParams,
    /// The core failed while processing the command.
    InternalError,
    /// A required external tool was not found.
    ToolNotFound,
}

/// Descriptor exposed to command palettes, menus, buttons, and shortcuts.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandDescriptor {
    /// Stable command identifier.
    pub id: String,
    /// Short title displayed in the UI.
    pub title: String,
    /// Group shown in command palettes and menus.
    pub category: String,
    /// Human-readable command description.
    pub description: String,
    /// Optional default shortcut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_shortcut: Option<String>,
    /// Whether the command requires an open workspace.
    pub requires_workspace: bool,
}

/// Lifecycle status of an external tool managed by the core.
///
/// The full lifecycle is documented in `docs/07-tooling-lifecycle.md`. Tool
/// detection uses `Missing`, `Detected`, and `Failed`; the remaining states are
/// reserved for process management.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolStatus {
    /// The tool requires manual configuration before it can be used.
    NotConfigured,
    /// The tool was not found on the search path.
    Missing,
    /// The tool was found and responded to a version probe.
    Detected,
    /// The tool is configured and ready to run.
    Ready,
    /// The tool is currently running as a managed process.
    Running,
    /// The tool was found but did not behave as expected.
    Failed,
    /// The tool was explicitly disabled by the user.
    Disabled,
}

/// Structured status of one external tool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    /// Stable tool identifier, such as `cargo` or `clangd`.
    pub id: String,
    /// Human-readable tool name.
    pub display_name: String,
    /// Current lifecycle status.
    pub status: ToolStatus,
    /// Absolute path of the detected binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Version string reported by the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Suggested installation command for CachyOS/Arch. The core never runs
    /// this command; the UI must show it and wait for explicit confirmation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_install: Option<String>,
    /// Human-readable explanation for `Missing` or `Failed` states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Result payload for `tools.detect` and `tools.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsDetectResult {
    /// Status of every tool known by the core.
    pub tools: Vec<ToolInfo>,
}

/// Project kind detected when a workspace is opened.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectKind {
    /// Rust project driven by Cargo (`Cargo.toml`).
    RustCargo,
    /// C/C++ project driven by `CMake` (`CMakeLists.txt`).
    Cmake,
    /// Java project driven by Maven (`pom.xml`).
    Maven,
    /// Java project driven by Gradle (`build.gradle`, `settings.gradle`).
    Gradle,
    /// Python project (`pyproject.toml`, `setup.py`, `requirements.txt`).
    Python,
    /// No known build system marker was found.
    Unknown,
}

/// Workspace opened by the core.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    /// Workspace display name, derived from the root directory name.
    pub name: String,
    /// Canonical absolute path of the workspace root.
    pub root: String,
    /// Primary project kind, chosen by marker precedence.
    pub kind: ProjectKind,
    /// Every recognized build system marker found in the root.
    pub markers: Vec<String>,
}

/// Parameters for `workspace.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceOpenParams {
    /// Directory to open as workspace root.
    pub path: String,
}

/// Result payload for `workspace.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusResult {
    /// Currently opened workspace, if any.
    #[serde(default)]
    pub workspace: Option<WorkspaceInfo>,
}

/// Parameters for `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceBrowseParams {
    /// Directory to list in the IDE-owned workspace picker.
    pub path: String,
}

/// One directory entry returned by `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseEntry {
    /// Directory name without the parent path.
    pub name: String,
    /// Canonical absolute path of the directory.
    pub path: String,
}

/// Result payload for `workspace.browse`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseResult {
    /// Canonical path that was listed.
    pub path: String,
    /// Canonical parent directory, absent for filesystem roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Child directories sorted case-insensitively.
    pub entries: Vec<WorkspaceBrowseEntry>,
}

/// Project template supported by `workspace.createProject`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceProjectTemplate {
    /// Empty directory opened as an unknown project.
    Empty,
    /// Strict C++ console project using `CMake`.
    CppCmake,
    /// Rust binary project created through `cargo new`.
    RustCargo,
}

/// Parameters for `workspace.createFolder`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceCreateFolderParams {
    /// Existing parent directory.
    pub parent: String,
    /// New child directory name, not a path.
    pub name: String,
}

/// Result payload for `workspace.createFolder`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCreateFolderResult {
    /// Canonical path of the created directory.
    pub path: String,
}

/// Parameters for `workspace.createProject`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceCreateProjectParams {
    /// Existing parent directory where the project directory will be created.
    pub parent: String,
    /// Project directory name, not a path.
    pub name: String,
    /// Template/scaffold to create.
    pub template: WorkspaceProjectTemplate,
}

/// Kind of a file system entry returned by `fs.list`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FsEntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symlink, socket, device, or anything else.
    Other,
}

/// One entry of a directory listing.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsEntry {
    /// Entry name without the parent path.
    pub name: String,
    /// Entry kind.
    pub kind: FsEntryKind,
    /// File size in bytes. Absent for directories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Parameters for `fs.list` and `fs.read`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsPathParams {
    /// Absolute path inside the open workspace root.
    pub path: String,
}

/// Parameters for `fs.createFile`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsCreateFileParams {
    /// Absolute path of the new file inside the workspace root.
    pub path: String,
    /// Initial UTF-8 content of the file.
    #[serde(default)]
    pub content: String,
}

/// Parameters for `fs.createDirectory`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsCreateDirectoryParams {
    /// Absolute path of the new directory inside the workspace root.
    pub path: String,
}

/// Parameters for `fs.write`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsWriteParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// New UTF-8 content of the file.
    pub content: String,
}

/// Parameters for `fs.rename` (also used to move within the workspace).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsRenameParams {
    /// Absolute path of the existing file or directory inside the workspace root.
    pub from: String,
    /// Absolute destination path inside the workspace root; must not exist yet.
    pub to: String,
}

/// Result payload for `fs.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsListResult {
    /// Canonical path of the listed directory.
    pub path: String,
    /// Entries sorted directories-first, then case-insensitive by name.
    pub entries: Vec<FsEntry>,
}

/// Result payload for `fs.read`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsReadResult {
    /// Canonical path of the file.
    pub path: String,
    /// UTF-8 file content.
    pub content: String,
}

/// Result payload for `fs.createFile`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsCreateFileResult {
    /// Canonical path of the created file.
    pub path: String,
    /// Number of bytes written as initial content.
    pub bytes_written: u64,
}

/// Result payload for `fs.createDirectory`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsCreateDirectoryResult {
    /// Canonical path of the created directory.
    pub path: String,
}

/// Result payload for `fs.write`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsWriteResult {
    /// Canonical path of the file.
    pub path: String,
    /// Number of bytes written.
    pub bytes_written: u64,
}

/// Result payload for `fs.rename`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsRenameResult {
    /// Canonical path of the source before the rename.
    pub from: String,
    /// Canonical path of the destination after the rename.
    pub to: String,
}

/// Result payload for `fs.delete`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsDeleteResult {
    /// Canonical path that was deleted.
    pub path: String,
}

/// Parameters for `fs.search`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsSearchParams {
    /// Literal text to look for. Not a regex.
    pub query: String,
    /// Match case exactly. Defaults to `false` (ASCII case-insensitive).
    #[serde(default)]
    pub case_sensitive: bool,
}

/// One match returned by `fs.search`. At most one match per line.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// One-based line number of the match.
    pub line: u64,
    /// One-based character column of the first occurrence in the line.
    pub column: u64,
    /// Trimmed line content for preview, capped by the core.
    pub preview: String,
}

/// Result payload for `fs.search`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsSearchResult {
    /// Matches in deterministic workspace order, capped by the core.
    pub matches: Vec<FsSearchMatch>,
    /// `true` when the match cap was reached and results were dropped.
    pub truncated: bool,
}

/// Parameters for `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FsFindFilesParams {
    /// Literal file-name query passed to `fd`.
    pub query: String,
}

/// One file result returned by `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsFileMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// File name without parent directories.
    pub name: String,
}

/// Result payload for `fs.findFiles`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FsFindFilesResult {
    /// File matches, capped by the core.
    pub matches: Vec<FsFileMatch>,
    /// `true` when the match cap was reached and results were dropped.
    pub truncated: bool,
}

/// Parameters for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStartParams {
    /// Shell command to execute in the workspace root. When absent, the core
    /// derives a default from the project kind (e.g. `cargo run`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Result payload for `run.start`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStartResult {
    /// Command that is now running.
    pub command: String,
}

/// Parameters for `run.stdin`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunStdinParams {
    /// Raw bytes forwarded to the child stdin. The UI appends the newline.
    pub data: String,
}

/// Result payload for `terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenResult {
    /// Shell the session is running (from `$SHELL`).
    pub shell: String,
}

/// Parameters for `terminal.input`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalInputParams {
    /// Raw bytes forwarded to the shell PTY. The UI appends the newline.
    pub data: String,
}

/// Parameters for LSP requests tied to the current editor cursor.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspTextDocumentPositionParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer. The core syncs it before sending the LSP request.
    pub content: String,
    /// One-based line number at the editor cursor.
    pub line: u64,
    /// One-based column number at the editor cursor.
    pub column: u64,
}

/// Result payload for `lsp.definition`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDefinitionResult {
    /// Target file, when the server found a definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// One-based target line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    /// One-based target column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

/// Result payload for `lsp.hover`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspHoverResult {
    /// Human-readable hover content, already flattened from LSP markup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// One completion entry returned by `lsp.completion`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionItem {
    /// Text shown in the completion popup.
    pub label: String,
    /// Text inserted when the item is accepted.
    pub insert_text: String,
    /// Extra detail (type signature, module), when the server provides it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Item kind, flattened from LSP numbers (`function`, `variable`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Result payload for `lsp.completion`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspCompletionResult {
    /// Completion entries, already sorted and capped by the core.
    pub items: Vec<LspCompletionItem>,
}

/// Parameters for `lsp.rename`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LspRenameParams {
    /// Absolute path of an existing file inside the workspace root.
    pub path: String,
    /// Current UTF-8 editor buffer. The core syncs it before sending the LSP request.
    pub content: String,
    /// One-based line number at the editor cursor.
    pub line: u64,
    /// One-based column number at the editor cursor.
    pub column: u64,
    /// New symbol name.
    pub new_name: String,
}

/// Result payload for `lsp.rename`, after the core applied every edit.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspRenameResult {
    /// Canonical absolute paths of the files rewritten by the rename.
    pub files: Vec<String>,
    /// Total number of text edits applied.
    pub edits: u64,
}

/// One code location returned by `lsp.references`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspReferenceItem {
    /// Canonical absolute path of the file.
    pub path: String,
    /// One-based line number.
    pub line: u64,
    /// One-based column number.
    pub column: u64,
}

/// Result payload for `lsp.references`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspReferencesResult {
    /// Every usage found by the server, capped by the core.
    pub references: Vec<LspReferenceItem>,
}

/// One semantic token resolved by `lsp.semanticTokens`.
///
/// Positions are tailored for the Qt text renderer: `line` is 1-based and
/// `start`/`length` are 0-based UTF-16 code units within the line, matching
/// `QString` indices, so the highlighter applies them without conversion.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSemanticToken {
    /// One-based line number.
    pub line: u64,
    /// Zero-based UTF-16 start column within the line.
    pub start: u64,
    /// Token length in UTF-16 code units.
    pub length: u64,
    /// Token kind name from the server legend (e.g. `variable`, `function`).
    pub kind: String,
}

/// Result payload for `lsp.semanticTokens`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSemanticTokensResult {
    /// Tokens in document order.
    pub tokens: Vec<LspSemanticToken>,
}

/// Severity of a structured build diagnostic.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildDiagnosticSeverity {
    /// Compilation error.
    Error,
    /// Compiler warning.
    Warning,
    /// Informational note attached to another diagnostic.
    Note,
}

/// Structured diagnostic extracted from build output.
///
/// Streamed as `event.build.diagnostic` notifications while a build runs.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDiagnostic {
    /// Diagnostic severity.
    pub severity: BuildDiagnosticSeverity,
    /// Human-readable message.
    pub message: String,
    /// Source file, relative to the workspace root when possible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// One-based line number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    /// One-based column number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

/// Result payload for `build.run`, sent after `event.build.finished`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRunResult {
    /// Whether the build finished successfully.
    pub success: bool,
    /// Exit code of the build tool, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of structured diagnostics emitted during the build.
    pub diagnostics: u64,
}

/// Result payload for `quality.run`, sent after `event.quality.finished`.
///
/// Mirrors `BuildRunResult`: quality analysis reuses the same structured
/// diagnostics pipeline as the build, only with a linter as the tool.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityRunResult {
    /// Whether the linter finished without erroring out.
    pub success: bool,
    /// Exit code of the linter, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of structured diagnostics emitted.
    pub diagnostics: u64,
}

/// Result payload for `test.run`, sent after `event.test.finished`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunResult {
    /// Whether the runner exited successfully (no failing test).
    pub success: bool,
    /// Exit code of the test runner, when it exited normally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Number of passed test cases.
    pub passed: u64,
    /// Number of failed test cases.
    pub failed: u64,
    /// Number of ignored test cases.
    pub ignored: u64,
}

/// Result payload for `core.ping`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorePingResult {
    /// Status marker used by smoke tests and UI health checks.
    pub status: String,
    /// Human-readable ping answer.
    pub message: String,
    /// Kernwerk IPC protocol version.
    pub protocol_version: String,
}

impl Default for CorePingResult {
    fn default() -> Self {
        Self {
            status: "ok".to_owned(),
            message: "pong".to_owned(),
            protocol_version: PROTOCOL_VERSION.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        CorePingResult, FsCreateDirectoryParams, FsCreateDirectoryResult, FsCreateFileParams,
        FsCreateFileResult, FsDeleteResult, FsFileMatch, FsFindFilesParams, FsFindFilesResult,
        FsRenameParams, FsRenameResult, FsSearchMatch, FsSearchParams, FsSearchResult,
        JSON_RPC_VERSION, JsonRpcError, JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse,
        LspCompletionItem, LspCompletionResult, LspDefinitionResult, LspRenameParams,
        LspRenameResult, LspTextDocumentPositionParams, ProjectKind, ToolInfo, ToolStatus,
        WorkspaceBrowseParams, WorkspaceCreateFolderParams, WorkspaceCreateProjectParams,
        WorkspaceInfo, WorkspaceOpenParams, WorkspaceProjectTemplate,
    };

    #[test]
    fn request_constructor_uses_json_rpc_version() {
        let request = JsonRpcRequest::new(1_i64, "core.ping", Some(json!({})));

        assert_eq!(request.jsonrpc, JSON_RPC_VERSION);
        assert_eq!(request.method, "core.ping");
        assert!(request.has_supported_version());
    }

    #[test]
    fn response_serializes_success_without_error_field() {
        let response = JsonRpcResponse::success(Some(json!(1)), json!(CorePingResult::default()));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["jsonrpc"], JSON_RPC_VERSION);
        assert_eq!(value["id"], 1);
        assert_eq!(value["result"]["message"], "pong");
        assert!(value.get("error").is_none());
    }

    #[test]
    fn tool_info_serializes_camel_case_and_omits_empty_fields() {
        let missing = ToolInfo {
            id: "clangd".to_owned(),
            display_name: "clangd".to_owned(),
            status: ToolStatus::Missing,
            path: None,
            version: None,
            suggested_install: Some("sudo pacman -S clang".to_owned()),
            message: Some("clangd nao foi encontrado no PATH".to_owned()),
        };
        let value = serde_json::to_value(missing).unwrap();

        assert_eq!(value["status"], "missing");
        assert_eq!(value["displayName"], "clangd");
        assert_eq!(value["suggestedInstall"], "sudo pacman -S clang");
        assert!(value.get("path").is_none());
        assert!(value.get("version").is_none());
    }

    #[test]
    fn workspace_info_serializes_camel_case_kind() {
        let workspace = WorkspaceInfo {
            name: "demo".to_owned(),
            root: "/home/user/demo".to_owned(),
            kind: ProjectKind::RustCargo,
            markers: vec!["Cargo.toml".to_owned()],
        };
        let value = serde_json::to_value(workspace).unwrap();

        assert_eq!(value["kind"], "rustCargo");
        assert_eq!(value["markers"][0], "Cargo.toml");
    }

    #[test]
    fn workspace_open_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceOpenParams>(json!({ "path": "/tmp" }));
        let invalid =
            serde_json::from_value::<WorkspaceOpenParams>(json!({ "path": "/tmp", "x": 1 }));

        assert_eq!(valid.unwrap().path, "/tmp");
        assert!(invalid.is_err());
    }

    #[test]
    fn workspace_browse_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceBrowseParams>(json!({ "path": "/tmp" }));
        let invalid =
            serde_json::from_value::<WorkspaceBrowseParams>(json!({ "path": "/tmp", "x": 1 }));

        assert_eq!(valid.unwrap().path, "/tmp");
        assert!(invalid.is_err());
    }

    #[test]
    fn fs_search_params_default_case_and_reject_unknown_fields() {
        let valid = serde_json::from_value::<FsSearchParams>(json!({ "query": "todo" })).unwrap();
        let explicit = serde_json::from_value::<FsSearchParams>(
            json!({ "query": "todo", "caseSensitive": true }),
        )
        .unwrap();
        let invalid = serde_json::from_value::<FsSearchParams>(json!({ "query": "x", "y": 1 }));

        assert_eq!(valid.query, "todo");
        assert!(!valid.case_sensitive);
        assert!(explicit.case_sensitive);
        assert!(invalid.is_err());
    }

    #[test]
    fn fs_create_file_params_default_content_and_reject_unknown_fields() {
        let valid =
            serde_json::from_value::<FsCreateFileParams>(json!({ "path": "/tmp/new.rs" })).unwrap();
        let invalid =
            serde_json::from_value::<FsCreateFileParams>(json!({ "path": "/tmp/new.rs", "x": 1 }));
        let result = FsCreateFileResult {
            path: "/tmp/new.rs".to_owned(),
            bytes_written: 3,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.path, "/tmp/new.rs");
        assert_eq!(valid.content, "");
        assert!(invalid.is_err());
        assert_eq!(value["bytesWritten"], 3);
    }

    #[test]
    fn fs_create_directory_params_and_result_use_camel_case() {
        let valid =
            serde_json::from_value::<FsCreateDirectoryParams>(json!({ "path": "/tmp/module" }))
                .unwrap();
        let invalid = serde_json::from_value::<FsCreateDirectoryParams>(
            json!({ "path": "/tmp/module", "recursive": true }),
        );
        let result = FsCreateDirectoryResult {
            path: "/tmp/module".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.path, "/tmp/module");
        assert!(invalid.is_err());
        assert_eq!(value["path"], "/tmp/module");
    }

    #[test]
    fn fs_rename_params_and_result_use_camel_case() {
        let valid = serde_json::from_value::<FsRenameParams>(
            json!({ "from": "/tmp/a.rs", "to": "/tmp/b.rs" }),
        )
        .unwrap();
        let invalid = serde_json::from_value::<FsRenameParams>(
            json!({ "from": "/tmp/a.rs", "to": "/tmp/b.rs", "x": 1 }),
        );
        let result = FsRenameResult {
            from: "/tmp/a.rs".to_owned(),
            to: "/tmp/b.rs".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.from, "/tmp/a.rs");
        assert_eq!(valid.to, "/tmp/b.rs");
        assert!(invalid.is_err());
        assert_eq!(value["from"], "/tmp/a.rs");
        assert_eq!(value["to"], "/tmp/b.rs");
    }

    #[test]
    fn fs_delete_result_serializes_camel_case() {
        let result = FsDeleteResult {
            path: "/tmp/gone.rs".to_owned(),
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["path"], "/tmp/gone.rs");
    }

    #[test]
    fn fs_search_result_serializes_camel_case() {
        let result = FsSearchResult {
            matches: vec![FsSearchMatch {
                path: "src/main.rs".to_owned(),
                line: 3,
                column: 5,
                preview: "let total = somar(2, 3);".to_owned(),
            }],
            truncated: false,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["matches"][0]["path"], "src/main.rs");
        assert_eq!(value["matches"][0]["line"], 3);
        assert_eq!(value["matches"][0]["column"], 5);
        assert_eq!(value["truncated"], false);
    }

    #[test]
    fn fs_find_files_params_and_result_use_camel_case() {
        let valid =
            serde_json::from_value::<FsFindFilesParams>(json!({ "query": "main" })).unwrap();
        let invalid = serde_json::from_value::<FsFindFilesParams>(
            json!({ "query": "main", "caseSensitive": true }),
        );
        let result = FsFindFilesResult {
            matches: vec![FsFileMatch {
                path: "src/main.rs".to_owned(),
                name: "main.rs".to_owned(),
            }],
            truncated: false,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(valid.query, "main");
        assert!(invalid.is_err());
        assert_eq!(value["matches"][0]["path"], "src/main.rs");
        assert_eq!(value["matches"][0]["name"], "main.rs");
        assert_eq!(value["truncated"], false);
    }

    #[test]
    fn lsp_position_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<LspTextDocumentPositionParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
        }));
        let invalid = serde_json::from_value::<LspTextDocumentPositionParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "offset": 3,
        }));

        assert_eq!(valid.unwrap().column, 4);
        assert!(invalid.is_err());
    }

    #[test]
    fn lsp_rename_params_reject_unknown_fields_and_use_camel_case() {
        let valid = serde_json::from_value::<LspRenameParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "start",
        }));
        let invalid = serde_json::from_value::<LspRenameParams>(json!({
            "path": "/tmp/main.rs",
            "content": "fn main() {}\n",
            "line": 1,
            "column": 4,
            "newName": "start",
            "force": true,
        }));

        assert_eq!(valid.unwrap().new_name, "start");
        assert!(invalid.is_err());
    }

    #[test]
    fn lsp_completion_item_omits_empty_optional_fields() {
        let result = LspCompletionResult {
            items: vec![LspCompletionItem {
                label: "main()".to_owned(),
                insert_text: "main".to_owned(),
                detail: None,
                kind: Some("function".to_owned()),
            }],
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["items"][0]["label"], "main()");
        assert_eq!(value["items"][0]["insertText"], "main");
        assert_eq!(value["items"][0]["kind"], "function");
        assert!(value["items"][0].get("detail").is_none());
    }

    #[test]
    fn lsp_rename_result_serializes_camel_case() {
        let result = LspRenameResult {
            files: vec!["/tmp/demo/src/main.rs".to_owned()],
            edits: 3,
        };
        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["files"][0], "/tmp/demo/src/main.rs");
        assert_eq!(value["edits"], 3);
    }

    #[test]
    fn lsp_definition_result_omits_missing_target() {
        let result = LspDefinitionResult {
            path: None,
            line: None,
            column: None,
        };
        let value = serde_json::to_value(result).unwrap();

        assert!(value.as_object().unwrap().is_empty());
    }

    #[test]
    fn workspace_create_folder_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<WorkspaceCreateFolderParams>(json!({
            "parent": "/tmp",
            "name": "demo",
        }));
        let invalid = serde_json::from_value::<WorkspaceCreateFolderParams>(json!({
            "parent": "/tmp",
            "name": "demo",
            "path": "/tmp/demo",
        }));

        assert_eq!(valid.unwrap().name, "demo");
        assert!(invalid.is_err());
    }

    #[test]
    fn workspace_project_template_serializes_camel_case() {
        let params = WorkspaceCreateProjectParams {
            parent: "/tmp".to_owned(),
            name: "demo".to_owned(),
            template: WorkspaceProjectTemplate::CppCmake,
        };
        let value = serde_json::to_value(params).unwrap();

        assert_eq!(value["template"], "cppCmake");
    }

    #[test]
    fn response_serializes_failure_without_result_field() {
        let error = JsonRpcError::new(JsonRpcErrorCode::MethodNotFound, "unknown method", None);
        let response = JsonRpcResponse::failure(Some(json!(7)), error);
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["id"], 7);
        assert_eq!(value["error"]["code"], "METHOD_NOT_FOUND");
        assert!(value.get("result").is_none());
    }
}
