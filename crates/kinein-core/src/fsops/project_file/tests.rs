//! The reading of a file of the project or of a tool (INF-18): the link
//! followed, no limit of the IDE's own, and the FIFO and the folder refused
//! without waiting, with the path from the project.
//!
//! The FIFO exists only on Unix; on Windows those cases are skipped, and the
//! link of a file needs a privilege there: the folder is the proof (63 §E.2).

use std::{io, path::PathBuf};

use super::{is_not_regular, project_file_problem, read_project_file, read_project_text};
use crate::fsops::{MAX_READ_BYTES, fifo, link, unblocked};

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-project-file-tests")
        .join(format!("{}-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

/// A file above the 1 MiB of the `.kinein` is read whole, and so is the
/// file a link points to, outside the project: the build system reads both.
#[test]
fn the_link_is_followed_and_a_large_file_is_read_whole() {
    let root = base("link-grande");
    let large = "x".repeat(usize::try_from(2 * MAX_READ_BYTES).unwrap());
    std::fs::write(root.join("CMakeCache.txt"), &large).unwrap();
    assert_eq!(
        read_project_text(&root.join("CMakeCache.txt")).unwrap(),
        large
    );
    if cfg!(windows) {
        return;
    }
    let outside = base("link-grande-fora");
    std::fs::write(outside.join("CMakeLists.txt"), "project(fora)\n").unwrap();
    link(
        &outside.join("CMakeLists.txt"),
        &root.join("CMakeLists.txt"),
    );
    assert_eq!(
        read_project_file(&root.join("CMakeLists.txt")).unwrap(),
        b"project(fora)\n"
    );
    assert_eq!(
        project_file_problem(&root, &root.join("CMakeLists.txt")),
        None
    );
}

/// The FIFO and the folder exist and are not regular files: refused at once,
/// with the path from the project, and the read never waits for a writer.
#[test]
fn the_fifo_and_the_folder_are_refused_without_waiting() {
    let root = base("fifo-pasta");
    std::fs::create_dir(root.join("CMakePresets.json")).unwrap();
    let error = read_project_file(&root.join("CMakePresets.json")).unwrap_err();
    assert!(is_not_regular(&error), "{error}");
    assert_eq!(
        project_file_problem(&root, &root.join("CMakePresets.json")),
        Some("CMakePresets.json não é um arquivo regular".to_owned())
    );
    let held = root.join("CMakeLists.txt");
    if !fifo(&held) {
        return;
    }
    let path = held.clone();
    let error = unblocked(&held, move || read_project_text(&path)).unwrap_err();
    assert!(is_not_regular(&error), "{error}");
    let (problem_root, path) = (root, held.clone());
    assert_eq!(
        unblocked(&held, move || project_file_problem(&problem_root, &path)),
        Some("CMakeLists.txt não é um arquivo regular".to_owned())
    );
}

/// Absent, the broken link included, is `NotFound` and no problem; a text
/// that is not UTF-8 is `InvalidData`, as `fs::read_to_string`.
#[test]
fn absent_is_not_found_and_not_a_problem() {
    let root = base("ausente");
    let error = read_project_file(&root.join("Cargo.toml")).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(!is_not_regular(&error));
    assert_eq!(project_file_problem(&root, &root.join("Cargo.toml")), None);
    std::fs::write(root.join("main.c"), [0xff, 0xfe]).unwrap();
    assert_eq!(
        read_project_text(&root.join("main.c")).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    if cfg!(windows) {
        return;
    }
    link(&root.join("sumiu"), &root.join("CMakeLists.txt"));
    assert_eq!(
        read_project_file(&root.join("CMakeLists.txt"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(
        project_file_problem(&root, &root.join("CMakeLists.txt")),
        None
    );
}
