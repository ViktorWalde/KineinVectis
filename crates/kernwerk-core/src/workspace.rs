//! Workspace opening, project kind detection, and metadata persistence.
//!
//! Opening a workspace canonicalizes the root, identifies the project kind by
//! build system markers, and records the result in `.kernwerk/workspace.json`.
//! The persisted format is documented in `schemas/workspace.schema.json`.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use kernwerk_protocol::{
    ProjectKind, WorkspaceBrowseEntry, WorkspaceBrowseResult, WorkspaceInfo,
    WorkspaceProjectTemplate,
};
use serde::{Deserialize, Serialize};

/// Directory created inside the workspace root for Kernwerk metadata.
pub const WORKSPACE_DIR: &str = ".kernwerk";

/// File name of the persisted workspace metadata.
pub const WORKSPACE_FILE: &str = "workspace.json";

/// Schema version of the persisted workspace metadata.
pub const WORKSPACE_SCHEMA_VERSION: &str = "0.1.0";

/// Build system markers in precedence order. The first match decides the
/// primary project kind; every match is reported to the UI.
const MARKERS: &[(&str, ProjectKind)] = &[
    ("Cargo.toml", ProjectKind::RustCargo),
    ("CMakeLists.txt", ProjectKind::Cmake),
    ("pom.xml", ProjectKind::Maven),
    ("build.gradle", ProjectKind::Gradle),
    ("build.gradle.kts", ProjectKind::Gradle),
    ("settings.gradle", ProjectKind::Gradle),
    ("settings.gradle.kts", ProjectKind::Gradle),
    ("pyproject.toml", ProjectKind::Python),
    ("setup.py", ProjectKind::Python),
    ("requirements.txt", ProjectKind::Python),
];

/// On-disk representation of `.kernwerk/workspace.json`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedWorkspace {
    /// Schema version of this file.
    schema_version: String,
    /// Workspace metadata as exposed over IPC.
    #[serde(flatten)]
    workspace: WorkspaceInfo,
}

/// Error produced while opening or persisting a workspace.
#[derive(Debug)]
pub enum WorkspaceError {
    /// The requested root does not exist or cannot be resolved.
    InvalidRoot {
        /// Path as requested by the client.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// The requested root exists but is not a directory.
    NotADirectory {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Workspace metadata could not be written.
    Persist {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A directory could not be listed.
    ListDirectory {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A new directory/project name is not a safe single path segment.
    InvalidName {
        /// Name as requested by the client.
        name: String,
        /// Human-readable reason.
        reason: String,
    },
    /// The target path already exists.
    AlreadyExists {
        /// Existing path.
        path: String,
    },
    /// A directory could not be created.
    CreateDirectory {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A template file could not be written.
    WriteTemplate {
        /// Path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A required external tool was missing.
    MissingTool {
        /// Tool executable.
        tool: &'static str,
    },
    /// A required external tool returned a failure.
    ToolFailed {
        /// Human-readable command line.
        command: String,
        /// Process exit code.
        exit_code: Option<i32>,
        /// Captured stderr/stdout detail.
        message: String,
    },
    /// Workspace metadata could not be serialized.
    Serialize(serde_json::Error),
}

impl WorkspaceError {
    /// Returns `true` when the error was caused by an invalid client path.
    #[must_use]
    pub const fn is_invalid_path(&self) -> bool {
        matches!(
            self,
            Self::InvalidRoot { .. }
                | Self::NotADirectory { .. }
                | Self::InvalidName { .. }
                | Self::AlreadyExists { .. }
        )
    }

    /// Returns `true` when the operation failed because a tool is missing.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingTool { .. })
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot { path, source } => {
                write!(
                    formatter,
                    "nao foi possivel abrir o workspace em {path}: {source}"
                )
            }
            Self::NotADirectory { path } => {
                write!(formatter, "o caminho {path} nao e um diretorio")
            }
            Self::Persist { path, source } => {
                write!(
                    formatter,
                    "falha ao gravar metadados do workspace em {path}: {source}"
                )
            }
            Self::ListDirectory { path, source } => {
                write!(formatter, "falha ao listar diretorio {path}: {source}")
            }
            Self::InvalidName { name, reason } => {
                write!(formatter, "nome invalido {name}: {reason}")
            }
            Self::AlreadyExists { path } => {
                write!(formatter, "o caminho {path} ja existe")
            }
            Self::CreateDirectory { path, source } => {
                write!(formatter, "falha ao criar diretorio {path}: {source}")
            }
            Self::WriteTemplate { path, source } => {
                write!(formatter, "falha ao gravar template em {path}: {source}")
            }
            Self::MissingTool { tool } => {
                write!(formatter, "{tool} nao foi encontrado no PATH")
            }
            Self::ToolFailed {
                command,
                exit_code,
                message,
            } => {
                write!(
                    formatter,
                    "{command} falhou com codigo {exit_code:?}: {message}"
                )
            }
            Self::Serialize(error) => {
                write!(
                    formatter,
                    "falha ao serializar metadados do workspace: {error}"
                )
            }
        }
    }
}

impl Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRoot { source, .. }
            | Self::Persist { source, .. }
            | Self::ListDirectory { source, .. }
            | Self::CreateDirectory { source, .. }
            | Self::WriteTemplate { source, .. } => Some(source),
            Self::Serialize(error) => Some(error),
            Self::NotADirectory { .. }
            | Self::InvalidName { .. }
            | Self::AlreadyExists { .. }
            | Self::MissingTool { .. }
            | Self::ToolFailed { .. } => None,
        }
    }
}

/// Detects the project kind of a workspace root.
///
/// Returns the primary kind (first marker in precedence order) and every
/// recognized marker file present in the root.
#[must_use]
pub fn detect_project(root: &Path) -> (ProjectKind, Vec<String>) {
    let found = MARKERS
        .iter()
        .filter(|(marker, _)| root.join(marker).is_file())
        .collect::<Vec<_>>();

    let kind = found
        .first()
        .map_or(ProjectKind::Unknown, |(_, kind)| *kind);
    let markers = found
        .iter()
        .map(|(marker, _)| (*marker).to_owned())
        .collect();

    (kind, markers)
}

/// Opens a directory as workspace and persists `.kernwerk/workspace.json`.
pub fn open_workspace(path: &Path) -> Result<WorkspaceInfo, WorkspaceError> {
    let root = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !root.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: root.display().to_string(),
        });
    }

    let name = root.file_name().map_or_else(
        || root.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    );
    let (kind, markers) = detect_project(&root);
    let workspace = WorkspaceInfo {
        name,
        root: root.display().to_string(),
        kind,
        markers,
    };

    persist(&root, &workspace)?;

    Ok(workspace)
}

/// Lists child directories for the IDE-owned workspace picker.
pub fn browse_directories(path: &Path) -> Result<WorkspaceBrowseResult, WorkspaceError> {
    let directory = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !directory.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    let read_dir = fs::read_dir(&directory).map_err(|source| WorkspaceError::ListDirectory {
        path: directory.display().to_string(),
        source,
    })?;

    let mut entries = Vec::new();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.map_err(|source| WorkspaceError::ListDirectory {
            path: directory.display().to_string(),
            source,
        })?;
        let path = dir_entry.path();
        let metadata = dir_entry
            .metadata()
            .map_err(|source| WorkspaceError::ListDirectory {
                path: path.display().to_string(),
                source,
            })?;

        if metadata.is_dir() {
            let canonical_path =
                fs::canonicalize(&path).map_err(|source| WorkspaceError::ListDirectory {
                    path: path.display().to_string(),
                    source,
                })?;
            entries.push(WorkspaceBrowseEntry {
                name: dir_entry.file_name().to_string_lossy().into_owned(),
                path: canonical_path.display().to_string(),
            });
        }
    }

    entries.sort_by_key(|entry| entry.name.to_lowercase());

    let parent = directory
        .parent()
        .map(|parent| parent.display().to_string());

    Ok(WorkspaceBrowseResult {
        path: directory.display().to_string(),
        parent,
        entries,
    })
}

/// Creates a child directory under an existing parent for the workspace picker.
pub fn create_directory(parent: &Path, name: &str) -> Result<PathBuf, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Folder)?;
    let target = parent.join(name);
    if target.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: target.display().to_string(),
        });
    }
    fs::create_dir(&target).map_err(|source| WorkspaceError::CreateDirectory {
        path: target.display().to_string(),
        source,
    })?;
    fs::canonicalize(&target).map_err(|source| WorkspaceError::InvalidRoot {
        path: target.display().to_string(),
        source,
    })
}

/// Creates a new project directory and opens it as a workspace.
pub fn create_project(
    parent: &Path,
    name: &str,
    template: WorkspaceProjectTemplate,
) -> Result<WorkspaceInfo, WorkspaceError> {
    match template {
        WorkspaceProjectTemplate::Empty => {
            let root = create_directory(parent, name)?;
            open_workspace(&root)
        }
        WorkspaceProjectTemplate::CppCmake => create_cpp_cmake_project(parent, name),
        WorkspaceProjectTemplate::RustCargo => create_rust_cargo_project(parent, name),
    }
}

/// Returns the path of the persisted metadata file for a workspace root.
#[must_use]
pub fn metadata_path(root: &Path) -> PathBuf {
    root.join(WORKSPACE_DIR).join(WORKSPACE_FILE)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NameKind {
    Folder,
    Project,
}

fn canonical_directory(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let directory = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !directory.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    Ok(directory)
}

fn validate_child_name(name: &str, kind: NameKind) -> Result<&str, WorkspaceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceError::InvalidName {
            name: name.to_owned(),
            reason: "nome vazio".to_owned(),
        });
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        return Err(WorkspaceError::InvalidName {
            name: name.to_owned(),
            reason: "use apenas o nome, nao um caminho".to_owned(),
        });
    }
    if kind == NameKind::Project {
        let starts_ok = trimmed
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic);
        let chars_ok = trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if !starts_ok || !chars_ok {
            return Err(WorkspaceError::InvalidName {
                name: name.to_owned(),
                reason: "projetos usam letras, numeros, '-' ou '_' e comecam por letra".to_owned(),
            });
        }
    }
    Ok(trimmed)
}

fn create_cpp_cmake_project(parent: &Path, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Project)?;
    let root = parent.join(name);
    if root.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: root.display().to_string(),
        });
    }
    fs::create_dir(&root).map_err(|source| WorkspaceError::CreateDirectory {
        path: root.display().to_string(),
        source,
    })?;
    let src = root.join("src");
    fs::create_dir(&src).map_err(|source| WorkspaceError::CreateDirectory {
        path: src.display().to_string(),
        source,
    })?;

    write_template(
        &root.join("CMakeLists.txt"),
        &format!(
            "cmake_minimum_required(VERSION 3.25)\n\
             project({name} LANGUAGES CXX)\n\n\
             set(CMAKE_CXX_STANDARD 23)\n\
             set(CMAKE_CXX_STANDARD_REQUIRED ON)\n\
             set(CMAKE_CXX_EXTENSIONS OFF)\n\n\
             add_executable({name} src/main.cpp)\n\
             target_compile_options({name} PRIVATE\n\
                 -Wall -Wextra -Wpedantic -Werror\n\
                 -Wconversion -Wsign-conversion -Wshadow\n\
             )\n"
        ),
    )?;
    write_template(
        &root.join("CMakePresets.json"),
        "{\n  \"version\": 6,\n  \"configurePresets\": [\n    {\n      \"name\": \"debug\",\n      \"generator\": \"Ninja\",\n      \"binaryDir\": \"${sourceDir}/.kernwerk/build/debug\",\n      \"cacheVariables\": {\n        \"CMAKE_BUILD_TYPE\": \"Debug\",\n        \"CMAKE_EXPORT_COMPILE_COMMANDS\": \"ON\"\n      }\n    }\n  ],\n  \"buildPresets\": [\n    {\n      \"name\": \"debug\",\n      \"configurePreset\": \"debug\"\n    }\n  ]\n}\n",
    )?;
    write_template(
        &src.join("main.cpp"),
        "#include <iostream>\n\nint main()\n{\n    std::cout << \"Kernwerk Studio\" << '\\n';\n    return 0;\n}\n",
    )?;
    write_template(
        &root.join("README.md"),
        &format!("# {name}\n\nProjeto C++/CMake strict criado pelo Kernwerk Studio.\n"),
    )?;

    open_workspace(&root)
}

fn create_rust_cargo_project(parent: &Path, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Project)?;
    let root = parent.join(name);
    if root.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: root.display().to_string(),
        });
    }

    let output = Command::new("cargo")
        .args(["new", "--bin", "--vcs", "none", name])
        .current_dir(&parent)
        .output()
        .map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                WorkspaceError::MissingTool { tool: "cargo" }
            } else {
                WorkspaceError::CreateDirectory {
                    path: root.display().to_string(),
                    source,
                }
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(WorkspaceError::ToolFailed {
            command: "cargo new --bin --vcs none".to_owned(),
            exit_code: output.status.code(),
            message: format!("{stderr}{stdout}"),
        });
    }

    open_workspace(&root)
}

fn write_template(path: &Path, content: &str) -> Result<(), WorkspaceError> {
    fs::write(path, content).map_err(|source| WorkspaceError::WriteTemplate {
        path: path.display().to_string(),
        source,
    })
}

fn persist(root: &Path, workspace: &WorkspaceInfo) -> Result<(), WorkspaceError> {
    let directory = root.join(WORKSPACE_DIR);
    fs::create_dir_all(&directory).map_err(|source| WorkspaceError::Persist {
        path: directory.display().to_string(),
        source,
    })?;

    let persisted = PersistedWorkspace {
        schema_version: WORKSPACE_SCHEMA_VERSION.to_owned(),
        workspace: workspace.clone(),
    };
    let mut json = serde_json::to_string_pretty(&persisted).map_err(WorkspaceError::Serialize)?;
    json.push('\n');

    let file = metadata_path(root);
    fs::write(&file, json).map_err(|source| WorkspaceError::Persist {
        path: file.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kernwerk_protocol::{ProjectKind, WorkspaceProjectTemplate};
    use serde_json::Value;

    use super::{
        browse_directories, create_directory, create_project, detect_project, metadata_path,
        open_workspace,
    };

    fn temp_workspace(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-workspace-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn empty_directory_is_unknown() {
        let dir = temp_workspace("unknown");

        let (kind, markers) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::Unknown);
        assert!(markers.is_empty());
    }

    #[test]
    fn cargo_marker_wins_over_cmake_marker() {
        let dir = temp_workspace("precedence");
        fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(dir.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let (kind, markers) = detect_project(&dir);

        assert_eq!(kind, ProjectKind::RustCargo);
        assert_eq!(markers, ["Cargo.toml", "CMakeLists.txt"]);
    }

    #[test]
    fn each_build_system_is_detected() {
        let cases = [
            ("pom.xml", ProjectKind::Maven),
            ("build.gradle.kts", ProjectKind::Gradle),
            ("pyproject.toml", ProjectKind::Python),
        ];

        for (marker, expected) in cases {
            let dir = temp_workspace(marker);
            fs::write(dir.join(marker), "x\n").unwrap();

            let (kind, _) = detect_project(&dir);

            assert_eq!(kind, expected, "marker {marker}");
        }
    }

    #[test]
    fn open_workspace_persists_metadata_json() {
        let dir = temp_workspace("persist");
        fs::write(dir.join("pyproject.toml"), "[project]\n").unwrap();

        let workspace = open_workspace(&dir).unwrap();

        assert_eq!(workspace.kind, ProjectKind::Python);
        assert_eq!(
            workspace.root,
            dir.canonicalize().unwrap().display().to_string()
        );

        let raw = fs::read_to_string(metadata_path(&dir)).unwrap();
        let value = serde_json::from_str::<Value>(&raw).unwrap();
        assert_eq!(value["schemaVersion"], "0.1.0");
        assert_eq!(value["kind"], "python");
        assert_eq!(value["name"], dir.file_name().unwrap().to_str().unwrap());
    }

    #[test]
    fn open_workspace_rejects_missing_path() {
        let dir = temp_workspace("missing").join("does-not-exist");

        let error = open_workspace(&dir).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("does-not-exist"));
    }

    #[test]
    fn browse_directories_lists_only_directories() {
        let dir = temp_workspace("browse");
        fs::create_dir(dir.join("zeta")).unwrap();
        fs::create_dir(dir.join("Alpha")).unwrap();
        fs::write(dir.join("file.txt"), "x").unwrap();

        let result = browse_directories(&dir).unwrap();

        assert_eq!(
            result.path,
            dir.canonicalize().unwrap().display().to_string()
        );
        assert!(result.parent.is_some());
        let names = result
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["Alpha", "zeta"]);
    }

    #[test]
    fn browse_directories_rejects_files() {
        let dir = temp_workspace("browse-file");
        let file = dir.join("file.txt");
        fs::write(&file, "x").unwrap();

        let error = browse_directories(&file).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("nao e um diretorio"));
    }

    #[test]
    fn create_directory_creates_child_and_browse_can_see_it() {
        let dir = temp_workspace("create-dir");

        let created = create_directory(&dir, "novo modulo").unwrap();
        let result = browse_directories(&dir).unwrap();

        assert!(created.is_dir());
        assert_eq!(result.entries[0].name, "novo modulo");
        assert_eq!(result.entries[0].path, created.display().to_string());
    }

    #[test]
    fn create_directory_rejects_path_like_name() {
        let dir = temp_workspace("create-dir-invalid");

        let error = create_directory(&dir, "../escape").unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("nao um caminho"));
    }

    #[test]
    fn create_cpp_cmake_project_writes_strict_starter_files() {
        let dir = temp_workspace("create-cpp");

        let workspace =
            create_project(&dir, "demo_cpp", WorkspaceProjectTemplate::CppCmake).unwrap();

        assert_eq!(workspace.kind, ProjectKind::Cmake);
        assert_eq!(workspace.markers, ["CMakeLists.txt"]);
        let root = PathBuf::from(&workspace.root);
        assert!(root.join("CMakePresets.json").is_file());
        assert!(root.join("src/main.cpp").is_file());
        let cmake = fs::read_to_string(root.join("CMakeLists.txt")).unwrap();
        assert!(cmake.contains("-Werror"));
    }

    #[test]
    fn create_project_rejects_spaces_in_project_name() {
        let dir = temp_workspace("create-project-invalid");

        let error =
            create_project(&dir, "demo cpp", WorkspaceProjectTemplate::CppCmake).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("letras"));
    }
}
