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
//! - [`trust`]: a confiança no projeto, guardada fora dele (16.4b).

mod c_options;
mod cmake_template;
mod create;
mod detect;
mod error;
mod open;
mod places;
mod project_options;
mod recent;
mod session;
mod trust;

pub use create::{create_directory, create_project, create_project_with_options};
pub use detect::detect_project;
pub use error::WorkspaceError;
pub use open::{browse_directories, metadata_path, open_workspace};
pub use places::browse_places;
pub use recent::{
    RecentWorkspaceError, clear_recent_workspaces_in, load_recent_workspaces_in,
    recent_workspaces_path_in, record_recent_workspace_in, remove_recent_workspace_in,
    set_recent_workspace_pinned_in,
};
pub use session::{load_session, save_session, session_path};
pub use trust::{is_trusted_in, trust_in, trust_path_in};

/// Valida ferramentas fora do laço; a geração só começa na continuação.
pub(crate) fn prepare_project(
    parent: &std::path::Path,
    name: &str,
    template: kinein_protocol::WorkspaceProjectTemplate,
    options: kinein_protocol::WorkspaceProjectOptions,
) -> Result<ProjectCreation, WorkspaceError> {
    project_options::validate(template, &options)?;
    Ok(ProjectCreation {
        parent: parent.to_owned(),
        name: name.to_owned(),
        template,
        options,
    })
}

/// Confere o driver C de um projeto EXISTENTE com as regras da criacao,
/// devolvendo a versao medida (consolidacao C §15); roda fora do laco.
pub(crate) fn validate_c_driver(executable: &str) -> Result<String, String> {
    project_options::validate_c_driver(executable)
}

pub(crate) struct ProjectCreation {
    parent: std::path::PathBuf,
    name: String,
    template: kinein_protocol::WorkspaceProjectTemplate,
    options: kinein_protocol::WorkspaceProjectOptions,
}

impl ProjectCreation {
    pub(crate) fn create(self) -> Result<kinein_protocol::WorkspaceInfo, WorkspaceError> {
        create::create_prepared_project(&self.parent, &self.name, self.template, &self.options)
    }
}

/// Directory created inside the workspace root for Kinein Vectis metadata.
pub const WORKSPACE_DIR: &str = ".kinein";

/// File name of the persisted workspace metadata.
pub const WORKSPACE_FILE: &str = "workspace.json";

/// Schema version of the persisted workspace metadata.
pub const WORKSPACE_SCHEMA_VERSION: &str = "0.2.0";
