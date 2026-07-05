//! Workspace opening, project kind detection, and metadata persistence.
//!
//! Opening a workspace canonicalizes the root, identifies the project kind by
//! build system markers, and records the result in `.kernwerk/workspace.json`.
//! The persisted format is documented in `schemas/workspace.schema.json`.
//!
//! Organizacao interna:
//! - [`error`]: o erro estruturado devolvido por toda operacao;
//! - [`detect`]: deteccao de project kind por marcadores de build;
//! - [`open`]: abertura, browse e persistencia de metadados;
//! - [`create`]: criacao de pastas e projetos a partir de templates.

mod create;
mod detect;
mod error;
mod open;

pub use create::{create_directory, create_project};
pub use detect::detect_project;
pub use error::WorkspaceError;
pub use open::{browse_directories, metadata_path, open_workspace};

/// Directory created inside the workspace root for Kernwerk metadata.
pub const WORKSPACE_DIR: &str = ".kernwerk";

/// File name of the persisted workspace metadata.
pub const WORKSPACE_FILE: &str = "workspace.json";

/// Schema version of the persisted workspace metadata.
pub const WORKSPACE_SCHEMA_VERSION: &str = "0.1.0";
