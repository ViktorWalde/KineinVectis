//! A file of the project or of a tool, read as the build system reads it
//! (`0.223.0`, INF-18, R-DADOS-04 of the 63).
//!
//! The link is the person's and is followed, and there is no size limit of
//! the IDE's own: the `CMakeLists.txt`, the presets and the `CMakeCache.txt`
//! are the truth of the project (D-14). The open does not wait for a writer,
//! and the descriptor has to be a regular file before anything is read: a
//! FIFO in place of the file never holds the core, and a folder is refused
//! the same way. Whoever reads takes what does not pass as absent;
//! [`project_file_problem`] says why, with the path from the project, for the
//! `projectProblem` of the open and of the `CMake` status.

use std::{
    fmt, fs,
    io::{self, Read},
    path::Path,
};

/// The file exists and is not a regular file: a FIFO, a folder, a device.
#[derive(Debug)]
struct NotRegular;

impl fmt::Display for NotRegular {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("não é um arquivo regular")
    }
}

impl std::error::Error for NotRegular {}

/// Opens `path` following the link, without waiting, and only when the
/// descriptor is a regular file.
///
/// # Errors
/// [`io::ErrorKind::NotFound`] when it does not exist (a broken link
/// included); [`is_not_regular`] when it exists and is not a regular file;
/// the failure of the open otherwise.
pub fn open_project_file(path: &Path) -> io::Result<fs::File> {
    let file = crate::platform::open_without_waiting(path)?;
    if file.metadata()?.is_file() {
        Ok(file)
    } else {
        Err(io::Error::other(NotRegular))
    }
}

/// The bytes of `path`, whole, opened by [`open_project_file`].
///
/// # Errors
/// The ones of [`open_project_file`], or the failure of the read.
pub fn read_project_file(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    open_project_file(path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// The text of `path`, as [`read_project_file`] reads it.
///
/// # Errors
/// The ones of [`read_project_file`], and [`io::ErrorKind::InvalidData`]
/// when it is not UTF-8, as `fs::read_to_string`.
pub fn read_project_text(path: &Path) -> io::Result<String> {
    String::from_utf8(read_project_file(path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Whether `error` says that the file exists and is not a regular file.
#[must_use]
pub fn is_not_regular(error: &io::Error) -> bool {
    matches!(error.get_ref(), Some(inner) if inner.is::<NotRegular>())
}

/// Why `path` exists and cannot be read as a regular file, with the path from
/// `root` when it is inside it; `None` when it opens or does not exist.
#[must_use]
pub fn project_file_problem(root: &Path, path: &Path) -> Option<String> {
    let error = open_project_file(path).err()?;
    if error.kind() == io::ErrorKind::NotFound {
        return None;
    }
    let label = path.strip_prefix(root).map_or_else(
        |_| path.display().to_string(),
        crate::platform::portable_relative,
    );
    Some(if is_not_regular(&error) {
        format!("{label} {error}")
    } else {
        format!("falha abrindo {label}: {error}")
    })
}

#[cfg(test)]
mod tests;
