//! The `.kinein` folder of a project (`0.221.0`, INF-12 and INF-13).
//!
//! Every file and folder the core writes in it passes here: the `.kinein`,
//! each folder under it and the file itself must be absent or what they say
//! they are, and none may be a symbolic link. A cloned repository with
//! `.kinein/workspace.json` linked to a file of the person had it overwritten
//! just by opening the project (measured on 2026-11-12); with `.kinein`
//! linked to another folder, the drafts of unsaved buffers went there. The
//! messages name the path from the project, like the settings ones.
//!
//! A leaf module, like `fsops`: every owner of a file in the `.kinein` uses
//! it, and it uses none of them (the module map, 01).

use std::{
    fs,
    path::{Path, PathBuf},
};

/// The folder of the IDE state inside the project root.
pub const WORKSPACE_DIR: &str = ".kinein";

/// Why the IDE does not use `.kinein/<relative>` of `root`: a component that
/// is a symbolic link, a folder in the middle that is not a folder, or the
/// last one that is not a regular file (`folder == false`) or a folder.
/// `None` when every component is absent or regular. `relative` uses `/`;
/// empty is the `.kinein` itself.
pub(crate) fn problem(root: &Path, relative: &str, folder: bool) -> Option<String> {
    let parts: Vec<&str> = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let mut path = root.join(WORKSPACE_DIR);
    let mut shown = WORKSPACE_DIR.to_owned();
    for index in 0..=parts.len() {
        if index > 0 {
            path.push(parts[index - 1]);
            shown = format!("{shown}/{}", parts[index - 1]);
        }
        let last = index == parts.len();
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => return Some(format!("{shown} não pôde ser lido ({error})")),
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Some(format!(
                    "{shown} é um link simbólico, que a IDE não segue no projeto"
                ));
            }
            Ok(metadata) if last && !folder => {
                if !metadata.is_file() {
                    return Some(format!("{shown} não é um arquivo regular"));
                }
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Some(format!("{shown} não é uma pasta"));
            }
            Ok(_) => {}
        }
    }
    None
}

/// `.kinein/<relative>` of `root`, checked, with the folders above it
/// created, for a file the core (or a tool it runs) writes.
pub(crate) fn file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if let Some(problem) = problem(root, relative, false) {
        return Err(problem);
    }
    let path = relative
        .split('/')
        .filter(|part| !part.is_empty())
        .fold(root.join(WORKSPACE_DIR), |path, part| path.join(part));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("falha criando {}: {error}", parent.display()))?;
    }
    Ok(path)
}

/// Writes `bytes` to `.kinein/<relative>` of `root`, after [`file`], with
/// the atomic swap that replaces the path without following a link.
pub(crate) fn write(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let path = file(root, relative)?;
    crate::fsops::atomic_write(&path, bytes)
        .map_err(|error| format!("falha escrevendo {WORKSPACE_DIR}/{relative}: {error}"))
}

/// What the health band says when the project opens: the problem of the
/// `.kinein` itself, else of the first entry right inside it, by name, that
/// is a symbolic link. `None` when the core writes there as always.
pub(crate) fn summary(root: &Path) -> Option<String> {
    if let Some(problem) = problem(root, "", true) {
        return Some(problem);
    }
    let entries = fs::read_dir(root.join(WORKSPACE_DIR)).ok()?;
    let mut linked: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_symlink()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    linked.sort();
    linked.first().map(|name| {
        format!("{WORKSPACE_DIR}/{name} é um link simbólico, que a IDE não segue no projeto")
    })
}

#[cfg(test)]
mod tests;

/// A link at `link` to `target`, a folder or what does not exist yet, for the
/// tests: `ln -s` on Unix and the junction on Windows, which needs no
/// privilege (60 §3.3), so they run on both.
#[cfg(test)]
pub(crate) fn link(target: &Path, link: &Path) {
    let made = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
    } else {
        std::process::Command::new("ln")
            .arg("-s")
            .arg(target)
            .arg(link)
            .output()
    }
    .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
}
