//! File system operations confined to the open workspace root.
//!
//! Every path is canonicalized and rejected when it escapes the workspace.
//! The UI never touches the file system directly; it goes through `fs.list`,
//! `fs.read`, `fs.createFile`, `fs.createDirectory`, `fs.write`, `fs.rename`,
//! and `fs.delete`.

use std::{
    error::Error,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use kernwerk_protocol::{FsEntry, FsEntryKind, FsFileMatch, FsSearchMatch};

/// Maximum file size accepted by `fs.read`, in bytes.
pub const MAX_READ_BYTES: u64 = 1_048_576;

/// Maximum number of matches returned by `fs.search`.
pub const MAX_SEARCH_MATCHES: usize = 500;

/// Maximum number of file matches returned by `fs.findFiles`.
pub const MAX_FILE_MATCHES: usize = 100;

/// Maximum preview length of a search match, in characters.
const MAX_PREVIEW_CHARS: usize = 200;

/// Directory names skipped by `fs.search` (VCS, caches, build output).
const SEARCH_SKIP_DIRS: &[&str] = &[
    ".git",
    ".kernwerk",
    ".idea",
    ".cache",
    "target",
    "build",
    "node_modules",
];

/// Error produced by confined file system operations.
#[derive(Debug)]
pub enum FsError {
    /// The path does not exist or cannot be resolved.
    InvalidPath {
        /// Path as requested by the client.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// The path resolves outside the workspace root.
    OutsideRoot {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Expected a directory but found something else.
    NotADirectory {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The requested new file path has no valid file name.
    InvalidFileName {
        /// Path as requested by the client.
        path: String,
    },
    /// The operation would overwrite an existing file.
    AlreadyExists {
        /// Path that was rejected.
        path: String,
    },
    /// Expected a regular file but found something else.
    NotAFile {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The operation targets the workspace root itself, which is not allowed.
    WorkspaceRoot {
        /// Canonical path that was rejected.
        path: String,
    },
    /// The file exceeds [`MAX_READ_BYTES`].
    TooLarge {
        /// Canonical path that was rejected.
        path: String,
        /// Actual file size in bytes.
        size: u64,
    },
    /// The file is not valid UTF-8 text.
    NotText {
        /// Canonical path that was rejected.
        path: String,
    },
    /// Underlying IO failure while reading or writing.
    Io {
        /// Canonical path that failed.
        path: String,
        /// Underlying IO error.
        source: io::Error,
    },
    /// A required external search tool was missing.
    MissingTool {
        /// Tool executable.
        tool: &'static str,
    },
    /// A required external search tool returned a failure.
    ToolFailed {
        /// Tool executable.
        tool: &'static str,
        /// Captured stderr/stdout detail.
        message: String,
    },
}

impl FsError {
    /// Returns `true` when the error was caused by an invalid client path.
    #[must_use]
    pub const fn is_invalid_path(&self) -> bool {
        !matches!(
            self,
            Self::Io { .. } | Self::MissingTool { .. } | Self::ToolFailed { .. }
        )
    }

    /// Returns `true` when the operation failed because a tool is missing.
    #[must_use]
    pub const fn is_missing_tool(&self) -> bool {
        matches!(self, Self::MissingTool { .. })
    }
}

impl fmt::Display for FsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { path, source } => {
                write!(formatter, "caminho invalido {path}: {source}")
            }
            Self::OutsideRoot { path } => {
                write!(formatter, "o caminho {path} esta fora do workspace aberto")
            }
            Self::NotADirectory { path } => {
                write!(formatter, "o caminho {path} nao e um diretorio")
            }
            Self::InvalidFileName { path } => {
                write!(formatter, "o caminho {path} nao tem nome de arquivo valido")
            }
            Self::AlreadyExists { path } => {
                write!(formatter, "o arquivo {path} ja existe")
            }
            Self::NotAFile { path } => {
                write!(formatter, "o caminho {path} nao e um arquivo regular")
            }
            Self::WorkspaceRoot { path } => {
                write!(
                    formatter,
                    "o caminho {path} e a raiz do workspace e nao pode ser renomeado ou removido"
                )
            }
            Self::TooLarge { path, size } => {
                write!(
                    formatter,
                    "o arquivo {path} tem {size} bytes e excede o limite de {MAX_READ_BYTES}"
                )
            }
            Self::NotText { path } => {
                write!(formatter, "o arquivo {path} nao e texto UTF-8 valido")
            }
            Self::Io { path, source } => {
                write!(formatter, "falha de IO em {path}: {source}")
            }
            Self::MissingTool { tool } => {
                write!(formatter, "{tool} nao foi encontrado no PATH")
            }
            Self::ToolFailed { tool, message } => {
                write!(formatter, "{tool} falhou: {message}")
            }
        }
    }
}

impl Error for FsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPath { source, .. } | Self::Io { source, .. } => Some(source),
            Self::OutsideRoot { .. }
            | Self::NotADirectory { .. }
            | Self::InvalidFileName { .. }
            | Self::AlreadyExists { .. }
            | Self::NotAFile { .. }
            | Self::WorkspaceRoot { .. }
            | Self::TooLarge { .. }
            | Self::NotText { .. }
            | Self::MissingTool { .. }
            | Self::ToolFailed { .. } => None,
        }
    }
}

/// Canonicalizes `path` and ensures it stays inside `root`.
fn confine(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let canonical = fs::canonicalize(path).map_err(|source| FsError::InvalidPath {
        path: path.display().to_string(),
        source,
    })?;

    if !canonical.starts_with(root) {
        return Err(FsError::OutsideRoot {
            path: canonical.display().to_string(),
        });
    }

    Ok(canonical)
}

/// Canonicalizes a path and ensures it is an existing regular file inside the workspace root.
pub fn confine_file(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let file = confine(root, path)?;

    if !file.is_file() {
        return Err(FsError::NotAFile {
            path: file.display().to_string(),
        });
    }

    Ok(file)
}

/// Lists a directory inside the workspace root.
///
/// Entries are sorted directories-first, then case-insensitive by name.
pub fn list_dir(root: &Path, path: &Path) -> Result<(PathBuf, Vec<FsEntry>), FsError> {
    let directory = confine(root, path)?;

    if !directory.is_dir() {
        return Err(FsError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    let read_dir = fs::read_dir(&directory).map_err(|source| FsError::Io {
        path: directory.display().to_string(),
        source,
    })?;

    let mut entries = Vec::new();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.map_err(|source| FsError::Io {
            path: directory.display().to_string(),
            source,
        })?;
        let metadata = dir_entry.metadata().map_err(|source| FsError::Io {
            path: dir_entry.path().display().to_string(),
            source,
        })?;

        let kind = if metadata.is_dir() {
            FsEntryKind::Directory
        } else if metadata.is_file() {
            FsEntryKind::File
        } else {
            FsEntryKind::Other
        };
        entries.push(FsEntry {
            name: dir_entry.file_name().to_string_lossy().into_owned(),
            kind,
            size: (kind == FsEntryKind::File).then_some(metadata.len()),
        });
    }

    entries.sort_by(|left, right| {
        let left_is_dir = left.kind == FsEntryKind::Directory;
        let right_is_dir = right.kind == FsEntryKind::Directory;
        right_is_dir
            .cmp(&left_is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    Ok((directory, entries))
}

/// Reads a UTF-8 text file inside the workspace root.
pub fn read_file(root: &Path, path: &Path) -> Result<(PathBuf, String), FsError> {
    let file = confine_file(root, path)?;

    let metadata = fs::metadata(&file).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;
    if metadata.len() > MAX_READ_BYTES {
        return Err(FsError::TooLarge {
            path: file.display().to_string(),
            size: metadata.len(),
        });
    }

    let bytes = fs::read(&file).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;
    let content = String::from_utf8(bytes).map_err(|_utf8_error| FsError::NotText {
        path: file.display().to_string(),
    })?;

    Ok((file, content))
}

/// Creates a new UTF-8 text file inside the workspace root.
///
/// Parent directories must already exist and the operation never overwrites an
/// existing file.
pub fn create_file(root: &Path, path: &Path, content: &str) -> Result<(PathBuf, u64), FsError> {
    let file = new_child_path(root, path)?;
    if file.exists() {
        return Err(FsError::AlreadyExists {
            path: file.display().to_string(),
        });
    }

    let mut handle = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                FsError::AlreadyExists {
                    path: file.display().to_string(),
                }
            } else {
                FsError::Io {
                    path: file.display().to_string(),
                    source,
                }
            }
        })?;
    handle
        .write_all(content.as_bytes())
        .map_err(|source| FsError::Io {
            path: file.display().to_string(),
            source,
        })?;

    Ok((file, content.len() as u64))
}

/// Creates a new directory inside the workspace root.
///
/// The parent directory must already exist and the operation never overwrites
/// an existing path.
pub fn create_directory(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let directory = new_child_path(root, path)?;
    if directory.exists() {
        return Err(FsError::AlreadyExists {
            path: directory.display().to_string(),
        });
    }

    fs::create_dir(&directory).map_err(|source| {
        if source.kind() == io::ErrorKind::AlreadyExists {
            FsError::AlreadyExists {
                path: directory.display().to_string(),
            }
        } else {
            FsError::Io {
                path: directory.display().to_string(),
                source,
            }
        }
    })?;

    Ok(directory)
}

fn new_child_path(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let parent = path.parent().ok_or_else(|| FsError::InvalidFileName {
        path: path.display().to_string(),
    })?;
    let file_name = path.file_name().ok_or_else(|| FsError::InvalidFileName {
        path: path.display().to_string(),
    })?;
    if file_name.is_empty() {
        return Err(FsError::InvalidFileName {
            path: path.display().to_string(),
        });
    }

    let directory = confine(root, parent)?;
    if !directory.is_dir() {
        return Err(FsError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    Ok(directory.join(file_name))
}

/// Overwrites an existing UTF-8 text file inside the workspace root.
///
/// This function only saves files that already exist; use [`create_file`] for
/// explicit new-file creation.
pub fn write_file(root: &Path, path: &Path, content: &str) -> Result<(PathBuf, u64), FsError> {
    let file = confine_file(root, path)?;

    fs::write(&file, content).map_err(|source| FsError::Io {
        path: file.display().to_string(),
        source,
    })?;

    Ok((file, content.len() as u64))
}

/// Renames or moves a file or directory inside the workspace root.
///
/// `from` must exist and stay inside the workspace. `to` must resolve to a new
/// child path whose parent already exists inside the workspace and must not
/// already exist. The workspace root itself cannot be renamed. Returns the
/// canonical source and destination paths.
pub fn rename(root: &Path, from: &Path, to: &Path) -> Result<(PathBuf, PathBuf), FsError> {
    let source = confine(root, from)?;
    if source == root {
        return Err(FsError::WorkspaceRoot {
            path: source.display().to_string(),
        });
    }

    let target = new_child_path(root, to)?;
    if target.exists() {
        return Err(FsError::AlreadyExists {
            path: target.display().to_string(),
        });
    }

    fs::rename(&source, &target).map_err(|io_error| FsError::Io {
        path: source.display().to_string(),
        source: io_error,
    })?;

    Ok((source, target))
}

/// Deletes a file or directory inside the workspace root.
///
/// Directories are removed recursively. The path must stay inside the workspace
/// and cannot be the workspace root itself. Returns the canonical deleted path.
pub fn delete(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let target = confine(root, path)?;
    if target == root {
        return Err(FsError::WorkspaceRoot {
            path: target.display().to_string(),
        });
    }

    if target.is_dir() {
        fs::remove_dir_all(&target)
    } else {
        fs::remove_file(&target)
    }
    .map_err(|io_error| FsError::Io {
        path: target.display().to_string(),
        source: io_error,
    })?;

    Ok(target)
}

/// Searches every UTF-8 text file of the workspace for a literal query.
///
/// The walk is depth-first in case-insensitive name order, so results are
/// deterministic. Directories in [`SEARCH_SKIP_DIRS`], files larger than
/// [`MAX_READ_BYTES`], non-UTF-8 files, and symlinks are skipped silently;
/// individual IO failures skip the entry instead of aborting the search.
/// At most one match per line is reported, capped at [`MAX_SEARCH_MATCHES`].
pub fn search(
    root: &Path,
    query: &str,
    case_sensitive: bool,
) -> Result<(Vec<FsSearchMatch>, bool), FsError> {
    let mut matches = Vec::new();
    let mut pending_dirs = vec![root.to_path_buf()];

    while let Some(directory) = pending_dirs.pop() {
        let Ok(read_dir) = fs::read_dir(&directory) else {
            if directory == root {
                return Err(FsError::NotADirectory {
                    path: directory.display().to_string(),
                });
            }
            continue;
        };

        let mut files = Vec::new();
        let mut subdirs = Vec::new();
        for dir_entry in read_dir.flatten() {
            let Ok(file_type) = dir_entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let name = dir_entry.file_name().to_string_lossy().into_owned();
            if file_type.is_dir() {
                if !SEARCH_SKIP_DIRS.contains(&name.as_str()) {
                    subdirs.push((name.to_lowercase(), dir_entry.path()));
                }
            } else if file_type.is_file() {
                files.push((name.to_lowercase(), dir_entry.path()));
            }
        }

        files.sort_by(|left, right| left.0.cmp(&right.0));
        subdirs.sort_by(|left, right| right.0.cmp(&left.0));
        pending_dirs.extend(subdirs.into_iter().map(|(_name, path)| path));

        for (_name, file) in files {
            if search_file(root, &file, query, case_sensitive, &mut matches) {
                return Ok((matches, true));
            }
        }
    }

    Ok((matches, false))
}

/// Finds files by name using `fd`, confined to the workspace root.
///
/// `fd` is intentionally used instead of a custom indexer: it is mature,
/// fast, and respects ignore files. Results are relative to `root`.
pub fn find_files(root: &Path, query: &str) -> Result<(Vec<FsFileMatch>, bool), FsError> {
    match find_files_with_binary(root, query, Path::new("fd")) {
        Err(FsError::MissingTool { .. }) => {
            find_files_with_binary(root, query, Path::new("fdfind"))
        }
        result => result,
    }
}

fn find_files_with_binary(
    root: &Path,
    query: &str,
    binary: &Path,
) -> Result<(Vec<FsFileMatch>, bool), FsError> {
    let root = confine(root, root)?;
    if !root.is_dir() {
        return Err(FsError::NotADirectory {
            path: root.display().to_string(),
        });
    }

    let mut command = Command::new(binary);
    command
        .arg("--type")
        .arg("f")
        .arg("--fixed-strings")
        .arg("--hidden")
        .arg("--color")
        .arg("never")
        .arg("--strip-cwd-prefix");
    for skipped in SEARCH_SKIP_DIRS {
        command.arg("--exclude").arg(skipped);
    }
    command.arg(query).arg(".").current_dir(&root);

    let output = command.output().map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            FsError::MissingTool { tool: "fd" }
        } else {
            FsError::Io {
                path: root.display().to_string(),
                source,
            }
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = stderr
            .lines()
            .chain(stdout.lines())
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("sem detalhe")
            .to_owned();
        return Err(FsError::ToolFailed {
            tool: "fd",
            message: detail,
        });
    }

    let mut matches = Vec::new();
    let mut truncated = false;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let relative = line.trim();
        if relative.is_empty() {
            continue;
        }
        let candidate = root.join(relative);
        if confine_file(&root, &candidate).is_err() {
            continue;
        }
        matches.push(FsFileMatch {
            path: relative.to_owned(),
            name: Path::new(relative).file_name().map_or_else(
                || relative.to_owned(),
                |name| name.to_string_lossy().into_owned(),
            ),
        });
        if matches.len() == MAX_FILE_MATCHES {
            truncated = true;
            break;
        }
    }

    matches.sort_by_key(|entry| entry.path.to_lowercase());
    Ok((matches, truncated))
}

/// Searches one file, appending matches. Returns `true` when the cap is hit.
fn search_file(
    root: &Path,
    file: &Path,
    query: &str,
    case_sensitive: bool,
    matches: &mut Vec<FsSearchMatch>,
) -> bool {
    let Ok(metadata) = fs::metadata(file) else {
        return false;
    };
    if metadata.len() > MAX_READ_BYTES {
        return false;
    }
    let Ok(bytes) = fs::read(file) else {
        return false;
    };
    let Ok(content) = String::from_utf8(bytes) else {
        return false;
    };

    let relative = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string();
    for (index, line) in content.lines().enumerate() {
        let Some(offset) = find_literal(line, query, case_sensitive) else {
            continue;
        };
        let column = line[..offset].chars().count() as u64 + 1;
        matches.push(FsSearchMatch {
            path: relative.clone(),
            line: index as u64 + 1,
            column,
            preview: line.trim().chars().take(MAX_PREVIEW_CHARS).collect(),
        });
        if matches.len() >= MAX_SEARCH_MATCHES {
            return true;
        }
    }
    false
}

/// Finds the byte offset of the first literal occurrence of `query` in `line`.
///
/// Case-insensitive comparison is ASCII-only, which keeps byte offsets exact.
fn find_literal(line: &str, query: &str, case_sensitive: bool) -> Option<usize> {
    if case_sensitive {
        return line.find(query);
    }
    let line_bytes = line.as_bytes();
    let query_bytes = query.as_bytes();
    if query_bytes.is_empty() || query_bytes.len() > line_bytes.len() {
        return None;
    }
    line_bytes
        .windows(query_bytes.len())
        .position(|window| window.eq_ignore_ascii_case(query_bytes))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kernwerk_protocol::FsEntryKind;

    use super::{
        FsError, MAX_READ_BYTES, MAX_SEARCH_MATCHES, delete, find_files_with_binary, list_dir,
        read_file, rename, search, write_file,
    };

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-fsops-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn list_dir_sorts_directories_first() {
        let root = temp_root("list");
        fs::create_dir(root.join("zeta-dir")).unwrap();
        fs::write(root.join("alpha.txt"), "a").unwrap();
        fs::write(root.join("Beta.txt"), "b").unwrap();

        let (path, entries) = list_dir(&root, &root).unwrap();

        assert_eq!(path, root);
        let names = entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["zeta-dir", "alpha.txt", "Beta.txt"]);
        assert_eq!(entries[0].kind, FsEntryKind::Directory);
        assert_eq!(entries[1].size, Some(1));
    }

    #[test]
    fn paths_outside_root_are_rejected() {
        let root = temp_root("outside");

        let error = list_dir(&root, &std::env::temp_dir()).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
        assert!(error.is_invalid_path());
    }

    #[test]
    fn dot_dot_traversal_is_rejected() {
        let root = temp_root("traversal");
        let sneaky = root.join("..").join("..");

        let error = list_dir(&root, &sneaky).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn read_and_write_roundtrip() {
        let root = temp_root("roundtrip");
        let file = root.join("main.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let (path, content) = read_file(&root, &file).unwrap();
        assert_eq!(content, "fn main() {}\n");

        let (_, bytes) = write_file(&root, &path, "fn main() { println!(); }\n").unwrap();
        assert_eq!(bytes, 26);

        let (_, reread) = read_file(&root, &file).unwrap();
        assert_eq!(reread, "fn main() { println!(); }\n");
    }

    #[test]
    fn read_rejects_oversized_files() {
        let root = temp_root("oversized");
        let file = root.join("big.bin");
        fs::write(
            &file,
            vec![b'x'; usize::try_from(MAX_READ_BYTES).unwrap() + 1],
        )
        .unwrap();

        let error = read_file(&root, &file).unwrap_err();

        assert!(matches!(error, FsError::TooLarge { .. }));
    }

    #[test]
    fn read_rejects_binary_files() {
        let root = temp_root("binary");
        let file = root.join("blob.bin");
        fs::write(&file, [0xFF, 0xFE, 0x00, 0x81]).unwrap();

        let error = read_file(&root, &file).unwrap_err();

        assert!(matches!(error, FsError::NotText { .. }));
    }

    #[test]
    fn search_finds_matches_case_insensitive_by_default() {
        let root = temp_root("search-basic");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    Somar(2, 3);\n}\n",
        )
        .unwrap();
        fs::write(root.join("notas.txt"), "sem ocorrencia\n").unwrap();

        let (matches, truncated) = search(&root, "somar", false).unwrap();

        assert!(!truncated);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/main.rs");
        assert_eq!(matches[0].line, 2);
        assert_eq!(matches[0].column, 5);
        assert_eq!(matches[0].preview, "Somar(2, 3);");
    }

    #[test]
    fn search_respects_case_sensitive_flag() {
        let root = temp_root("search-case");
        fs::write(root.join("a.txt"), "Alpha\nalpha\n").unwrap();

        let (insensitive, _) = search(&root, "alpha", false).unwrap();
        let (sensitive, _) = search(&root, "alpha", true).unwrap();

        assert_eq!(insensitive.len(), 2);
        assert_eq!(sensitive.len(), 1);
        assert_eq!(sensitive[0].line, 2);
    }

    #[test]
    fn search_skips_build_dirs_binaries_and_symlinks() {
        let root = temp_root("search-skip");
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/gerado.rs"), "alvo aqui\n").unwrap();
        fs::write(root.join("blob.bin"), [0xFF, 0x00, b'a', b'l', b'v', b'o']).unwrap();
        fs::write(root.join("fonte.rs"), "alvo aqui\n").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("fonte.rs"), root.join("link.rs")).unwrap();

        let (matches, _) = search(&root, "alvo", false).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "fonte.rs");
    }

    #[test]
    fn search_truncates_at_the_match_cap() {
        let root = temp_root("search-cap");
        let line = "ocorrencia\n".repeat(MAX_SEARCH_MATCHES + 10);
        fs::write(root.join("muitos.txt"), line).unwrap();

        let (matches, truncated) = search(&root, "ocorrencia", false).unwrap();

        assert!(truncated);
        assert_eq!(matches.len(), MAX_SEARCH_MATCHES);
    }

    #[test]
    fn search_walks_in_deterministic_order() {
        let root = temp_root("search-order");
        fs::create_dir(root.join("zeta")).unwrap();
        fs::create_dir(root.join("alfa")).unwrap();
        fs::write(root.join("zeta/z.txt"), "x\n").unwrap();
        fs::write(root.join("alfa/a.txt"), "x\n").unwrap();
        fs::write(root.join("raiz.txt"), "x\n").unwrap();

        let (matches, _) = search(&root, "x", false).unwrap();

        let paths = matches
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, ["raiz.txt", "alfa/a.txt", "zeta/z.txt"]);
    }

    #[test]
    fn write_refuses_to_create_new_files() {
        let root = temp_root("no-create");

        let error = write_file(&root, &root.join("novo.txt"), "x").unwrap_err();

        assert!(matches!(error, FsError::InvalidPath { .. }));
    }

    #[test]
    fn rename_moves_file_within_workspace() {
        let root = temp_root("rename-file");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("old.rs"), "conteudo\n").unwrap();

        let (from, to) = rename(&root, &root.join("old.rs"), &root.join("src/new.rs")).unwrap();

        assert_eq!(from, root.join("old.rs"));
        assert!(to.starts_with(&root));
        assert!(to.ends_with("new.rs"));
        assert!(!root.join("old.rs").exists());
        assert_eq!(
            fs::read_to_string(root.join("src/new.rs")).unwrap(),
            "conteudo\n"
        );
    }

    #[test]
    fn rename_refuses_to_clobber_existing_target() {
        let root = temp_root("rename-clobber");
        fs::write(root.join("a.rs"), "a\n").unwrap();
        fs::write(root.join("b.rs"), "b\n").unwrap();

        let error = rename(&root, &root.join("a.rs"), &root.join("b.rs")).unwrap_err();

        assert!(matches!(error, FsError::AlreadyExists { .. }));
        assert_eq!(fs::read_to_string(root.join("a.rs")).unwrap(), "a\n");
    }

    #[test]
    fn rename_rejects_source_outside_root() {
        let root = temp_root("rename-outside");

        let error = rename(&root, &std::env::temp_dir(), &root.join("here")).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn rename_rejects_the_workspace_root() {
        let root = temp_root("rename-root");

        let error = rename(&root, &root, &root.join("renamed")).unwrap_err();

        assert!(matches!(error, FsError::WorkspaceRoot { .. }));
        assert!(error.is_invalid_path());
    }

    #[test]
    fn delete_removes_a_file() {
        let root = temp_root("delete-file");
        fs::write(root.join("gone.rs"), "x\n").unwrap();

        let path = delete(&root, &root.join("gone.rs")).unwrap();

        assert!(path.ends_with("gone.rs"));
        assert!(!root.join("gone.rs").exists());
    }

    #[test]
    fn delete_removes_a_directory_recursively() {
        let root = temp_root("delete-dir");
        fs::create_dir_all(root.join("mod/inner")).unwrap();
        fs::write(root.join("mod/inner/a.rs"), "a\n").unwrap();

        delete(&root, &root.join("mod")).unwrap();

        assert!(!root.join("mod").exists());
    }

    #[test]
    fn delete_rejects_paths_outside_root() {
        let root = temp_root("delete-outside");

        let error = delete(&root, &std::env::temp_dir()).unwrap_err();

        assert!(matches!(error, FsError::OutsideRoot { .. }));
    }

    #[test]
    fn delete_rejects_the_workspace_root() {
        let root = temp_root("delete-root");

        let error = delete(&root, &root).unwrap_err();

        assert!(matches!(error, FsError::WorkspaceRoot { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn find_files_uses_fd_output_and_confines_results() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("find-files");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("README.md"), "# demo\n").unwrap();

        let fd = root.join("fake-fd");
        fs::write(
            &fd,
            "#!/bin/sh\nprintf 'src/main.rs\\nREADME.md\\n../escape.rs\\n'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fd).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fd, permissions).unwrap();

        let (matches, truncated) = find_files_with_binary(&root, "main", &fd).unwrap();
        let paths = matches
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();

        assert!(!truncated);
        assert_eq!(paths, ["README.md", "src/main.rs"]);
        assert_eq!(matches[1].name, "main.rs");
    }
}
