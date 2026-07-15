//! Workspace opening, project kind detection, and metadata persistence.
//!
//! Opening a workspace canonicalizes the root, identifies the project kind by
//! build system markers, and records the result in `.kinein/workspace.json`.
//! The persisted format is documented in `schemas/workspace.schema.json`.
//!
//! Organizacao interna:
//! - [`error`]: o erro estruturado devolvido por toda operacao;
//! - [`detect`]: deteccao de project kind por marcadores de build;
//! - [`open`]: abertura, browse e persistencia de metadados;
//! - [`create`]: criacao de pastas e projetos a partir de templates;
//! - [`session`]: abas abertas/aba ativa por workspace (`session.json`).
//! - [`recent`]: histórico global versionado para Start Screen/menu Arquivo.

mod create;
mod detect;
mod error;
mod open;
mod recent;
mod session;

pub use create::{create_directory, create_project};
pub use detect::detect_project;
pub use error::WorkspaceError;
pub use open::{browse_directories, metadata_path, open_workspace};
pub use recent::{
    RecentWorkspaceError, clear_recent_workspaces, load_recent_workspaces, recent_workspaces_path,
    record_recent_workspace, remove_recent_workspace, set_recent_workspace_pinned,
};
pub use session::{load_session, save_session, session_path};

/// Directory created inside the workspace root for Kinein Vectis metadata.
pub const WORKSPACE_DIR: &str = ".kinein";

/// File name of the persisted workspace metadata.
pub const WORKSPACE_FILE: &str = "workspace.json";

/// Schema version of the persisted workspace metadata.
pub const WORKSPACE_SCHEMA_VERSION: &str = "0.2.0";
