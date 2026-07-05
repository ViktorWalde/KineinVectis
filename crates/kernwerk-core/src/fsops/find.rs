//! File-name search delegated to `fd`, confined to the workspace root.

use std::{io, path::Path, process::Command};

use kernwerk_protocol::FsFileMatch;

use super::confine::{confine, confine_file};
use super::{FsError, MAX_FILE_MATCHES, SEARCH_SKIP_DIRS};

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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::find_files_with_binary;

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kernwerk-fsops-find-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
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
