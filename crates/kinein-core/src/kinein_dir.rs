//! The `.kinein` folder of a project (`0.222.0`, INF-12, INF-13, INF-15 and
//! INF-20).
//!
//! Every file and folder the core writes or removes in it passes here: the
//! `.kinein`, each folder under it and the file itself must be absent or
//! what they say they are, and none may be a symbolic link. A cloned
//! repository with `.kinein/workspace.json` linked to a file of the person
//! had it overwritten just by opening the project (measured on 2026-11-12);
//! with `.kinein` linked to another folder, the drafts of unsaved buffers
//! went there. Every file the core reads there passes here too, on the
//! descriptor: a FIFO in place of `.kinein/session.json` held the whole core
//! in the `open` when the project opened (measured on 2026-11-12). The
//! messages name the path from the project, like the settings ones.
//!
//! A leaf module, like `fsops`: every owner of a file in the `.kinein` uses
//! it, and it uses none of them (the module map, 01).

use std::{
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::platform::{self, NoFollow};

/// The folder of the IDE state inside the project root.
pub const WORKSPACE_DIR: &str = ".kinein";

/// The components of `relative`, which uses `/`.
fn parts(relative: &str) -> impl Iterator<Item = &str> {
    relative.split('/').filter(|part| !part.is_empty())
}

fn linked(shown: &str) -> String {
    format!("{shown} é um link simbólico, que a IDE não segue no projeto")
}

fn unreadable(shown: &str, error: &dyn std::fmt::Display) -> String {
    format!("{shown} não pôde ser lido ({error})")
}

/// Why the IDE does not use `.kinein/<relative>` of `root`: a component that
/// is a symbolic link, a folder in the middle that is not a folder, or the
/// last one that is not a regular file (`folder == false`) or a folder.
/// `None` when every component is absent or regular. `relative` uses `/`;
/// empty is the `.kinein` itself.
pub(crate) fn problem(root: &Path, relative: &str, folder: bool) -> Option<String> {
    let parts: Vec<&str> = parts(relative).collect();
    let mut path = root.join(WORKSPACE_DIR);
    let mut shown = WORKSPACE_DIR.to_owned();
    for index in 0..=parts.len() {
        if index > 0 {
            path.push(parts[index - 1]);
            shown = format!("{shown}/{}", parts[index - 1]);
        }
        let last = index == parts.len();
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
            Err(error) => return Some(unreadable(&shown, &error)),
            Ok(metadata) if metadata.file_type().is_symlink() => return Some(linked(&shown)),
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
    let path = parts(relative).fold(root.join(WORKSPACE_DIR), |path, part| path.join(part));
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

/// Removes `.kinein/<relative>` of `root`, a regular file, or with `folder`
/// a folder and everything in it, after [`problem`]; absent is done. With
/// `.kinein/presets` linked, the repair of the build folder removed the
/// folder outside the project (measured on 2026-11-13).
pub(crate) fn remove(root: &Path, relative: &str, folder: bool) -> Result<(), String> {
    if let Some(problem) = problem(root, relative, folder) {
        return Err(problem);
    }
    let path = parts(relative).fold(root.join(WORKSPACE_DIR), |path, part| path.join(part));
    let removed = if folder {
        fs::remove_dir_all(&path)
    } else {
        fs::remove_file(&path)
    };
    match removed {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(format!(
            "falha removendo {WORKSPACE_DIR}/{relative}: {error}"
        )),
        _ => Ok(()),
    }
}

/// Reads `.kinein/<relative>` of `root`, at most
/// [`crate::fsops::MAX_READ_BYTES`]; `None` when absent. Each component is
/// opened without following a link and without blocking, so a FIFO opens at
/// once, and its descriptor must be a folder in the middle and a regular
/// file at the end. The project root itself is the person's choice: it is
/// opened by its canonical path. The refusals are the ones of [`problem`].
pub(crate) fn read(root: &Path, relative: &str) -> Result<Option<Vec<u8>>, String> {
    let names: Vec<&str> = std::iter::once(WORKSPACE_DIR)
        .chain(parts(relative))
        .collect();
    let mut path = match platform::canonicalize(root) {
        Ok(path) => path,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(WORKSPACE_DIR, &error)),
    };
    let mut current = platform::open_nofollow(&path)
        .map_err(|error| unreadable(WORKSPACE_DIR, &refusal(error)))?;
    let mut shown = String::new();
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            shown.push('/');
        }
        shown.push_str(name);
        let child = match platform::open_child_nofollow(&current, &path, OsStr::new(name)) {
            Ok(child) => child,
            Err(NoFollow::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(NoFollow::Link) => return Err(linked(&shown)),
            Err(error) => return Err(unreadable(&shown, &refusal(error))),
        };
        let metadata = child
            .metadata()
            .map_err(|error| unreadable(&shown, &error))?;
        if index + 1 == names.len() {
            if !metadata.is_file() {
                return Err(format!("{shown} não é um arquivo regular"));
            }
        } else if !metadata.is_dir() {
            return Err(format!("{shown} não é uma pasta"));
        }
        path.push(name);
        current = child;
    }
    let mut bytes = Vec::new();
    current
        .take(crate::fsops::MAX_READ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| unreadable(&shown, &error))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > crate::fsops::MAX_READ_BYTES {
        return Err(format!(
            "{shown} passa de 1 MiB, o limite de leitura da IDE"
        ));
    }
    Ok(Some(bytes))
}

fn refusal(error: NoFollow) -> io::Error {
    match error {
        NoFollow::Io(error) => error,
        NoFollow::Link => io::Error::other("link simbólico no caminho"),
        NoFollow::InvalidPath => io::Error::other("caminho inválido"),
    }
}

/// The project root and the path under its `.kinein`, with `/`, of `path`
/// when it is inside one, like the configure folders of the IDE (INF-15).
/// The outermost `.kinein` counts: a preset named `.kinein` makes a folder of
/// that name under `.kinein/presets`.
pub(crate) fn inside(path: &Path) -> Option<(&Path, String)> {
    let folder = path
        .ancestors()
        .filter(|ancestor| ancestor.file_name() == Some(OsStr::new(WORKSPACE_DIR)))
        .last()?;
    let relative = path.strip_prefix(folder).ok()?;
    Some((folder.parent()?, platform::portable_relative(relative)))
}

/// What the health band says when the project opens: the problem of the
/// `.kinein` itself, else of the first entry right inside it, by name, that
/// is a symbolic link or neither a regular file nor a folder, like a FIFO
/// (`0.222.0`, INF-15). `None` when the core uses it as always.
pub(crate) fn summary(root: &Path) -> Option<String> {
    if let Some(problem) = problem(root, "", true) {
        return Some(problem);
    }
    let entries = fs::read_dir(root.join(WORKSPACE_DIR)).ok()?;
    let mut refused: Vec<(String, String)> = entries
        .flatten()
        .filter_map(|entry| {
            let kind = entry.file_type().ok()?;
            let shown = format!("{WORKSPACE_DIR}/{}", entry.file_name().to_string_lossy());
            if kind.is_symlink() {
                Some((shown.clone(), linked(&shown)))
            } else if kind.is_file() || kind.is_dir() {
                None
            } else {
                Some((
                    shown.clone(),
                    format!("{shown} não é um arquivo regular nem uma pasta"),
                ))
            }
        })
        .collect();
    refused.sort();
    refused.into_iter().next().map(|(_, problem)| problem)
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

/// A FIFO at `path`, for the tests: `mkfifo` on Unix. `false` on Windows,
/// whose file system has no FIFO: there the tests prove the folder and the
/// link (63 §E.2).
#[cfg(test)]
pub(crate) fn fifo(path: &Path) -> bool {
    if cfg!(windows) {
        return false;
    }
    let made = std::process::Command::new("mkfifo")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    true
}

/// What `work` returns, run in a thread. A read held in the `open` of the
/// FIFO `held` fails the test after 20 s, where the normal path takes
/// milliseconds: the other side is opened, so the thread ends and the run
/// never hangs (63 §E.2).
#[cfg(test)]
pub(crate) fn unblocked<T: Send + 'static>(
    held: &Path,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    use std::sync::mpsc::RecvTimeoutError;

    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || drop(sender.send(work())));
    match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(value) => value,
        Err(RecvTimeoutError::Disconnected) => match worker.join() {
            Err(panic) => std::panic::resume_unwind(panic),
            Ok(()) => unreachable!("o trabalho terminou sem resposta"),
        },
        Err(RecvTimeoutError::Timeout) => {
            let other = held.to_owned();
            // O `open` de escrita volta assim que a leitura segura o FIFO.
            std::thread::spawn(move || drop(fs::OpenOptions::new().write(true).open(other)));
            panic!("a leitura prendeu no FIFO {}", held.display());
        }
    }
}
