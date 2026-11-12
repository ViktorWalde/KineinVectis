//! The guard of the `.kinein` (INF-13): each component of the path, the
//! write that does not follow a link, and the summary for the health band.
//!
//! The links are of folders, or to what does not exist: `ln -s` on Unix and
//! the junction on Windows, which needs no privilege (60 §3.3).

use std::path::PathBuf;

use super::{file, link, problem, summary, write};

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-state-tests")
        .join(format!("{}-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

const FOLDER_LINK: &str = ".kinein é um link simbólico, que a IDE não segue no projeto";

/// Absent and regular components pass; the link, the folder in the middle
/// that is a file and the last one of the wrong kind say which and why.
#[test]
fn each_component_of_the_path_is_checked() {
    let root = base("componentes");
    assert_eq!(problem(&root, "", true), None);
    assert_eq!(problem(&root, "build/CMakeCache.txt", false), None);
    std::fs::create_dir_all(root.join(".kinein").join("build")).unwrap();
    std::fs::write(root.join(".kinein").join("session.json"), "{}").unwrap();
    assert_eq!(problem(&root, "session.json", false), None);
    assert_eq!(problem(&root, "build", true), None);
    assert_eq!(
        problem(&root, "build", false),
        Some(".kinein/build não é um arquivo regular".to_owned())
    );
    assert_eq!(
        problem(&root, "session.json/x", false),
        Some(".kinein/session.json não é uma pasta".to_owned())
    );
    link(
        &root.join("ausente"),
        &root.join(".kinein").join("build").join("app"),
    );
    assert_eq!(
        problem(&root, "build/app/CMakeCache.txt", false),
        Some(".kinein/build/app é um link simbólico, que a IDE não segue no projeto".to_owned())
    );

    let other = base("kinein-arquivo");
    std::fs::write(other.join(".kinein"), "x").unwrap();
    assert_eq!(
        problem(&other, "workspace.json", false),
        Some(".kinein não é uma pasta".to_owned())
    );
}

/// The write goes to the project, creates the folders, and refuses the
/// `.kinein` and the file that are links: nothing is created or changed
/// outside, and the link stays.
#[test]
fn the_write_refuses_a_link_and_writes_nothing_outside() {
    let root = base("escrita");
    write(&root, "workspace.json", b"{\"a\": 1}").unwrap();
    assert_eq!(
        std::fs::read(root.join(".kinein").join("workspace.json")).unwrap(),
        b"{\"a\": 1}"
    );
    assert_eq!(
        file(&root, "build/debug/CMakeCache.txt").unwrap(),
        root.join(".kinein")
            .join("build")
            .join("debug")
            .join("CMakeCache.txt")
    );
    assert!(root.join(".kinein").join("build").join("debug").is_dir());

    let outside = base("escrita-fora");
    let project = base("escrita-pasta-ligada");
    link(&outside, &project.join(".kinein"));
    assert_eq!(
        write(&project, "workspace.json", b"x"),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(
        file(&project, "build/debug/CMakeCache.txt"),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);

    let dangling = base("escrita-arquivo-ligado");
    std::fs::create_dir_all(dangling.join(".kinein")).unwrap();
    let missing = base("escrita-ausente").join("nao-existe");
    link(&missing, &dangling.join(".kinein").join("session.json"));
    assert_eq!(
        file(&dangling, "session.json"),
        Err(".kinein/session.json é um link simbólico, que a IDE não segue no projeto".to_owned())
    );
    assert!(write(&dangling, "session.json", b"x").is_err());
    assert!(!missing.exists());
    assert!(
        std::fs::symlink_metadata(dangling.join(".kinein").join("session.json"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

/// The band: nothing for a regular `.kinein`; the folder itself first; else
/// the first entry inside it, by name, that is a link.
#[test]
fn the_summary_names_the_folder_or_the_first_linked_entry() {
    let root = base("resumo");
    assert_eq!(summary(&root), None);
    std::fs::create_dir_all(root.join(".kinein").join("build")).unwrap();
    std::fs::write(root.join(".kinein").join("workspace.json"), "{}").unwrap();
    assert_eq!(summary(&root), None);
    let missing = base("resumo-ausente");
    link(
        &missing.join("b"),
        &root.join(".kinein").join("session.json"),
    );
    link(&missing.join("a"), &root.join(".kinein").join("kinein.db"));
    assert_eq!(
        summary(&root),
        Some(".kinein/kinein.db é um link simbólico, que a IDE não segue no projeto".to_owned())
    );

    let project = base("resumo-pasta");
    link(&missing, &project.join(".kinein"));
    assert_eq!(summary(&project), Some(FOLDER_LINK.to_owned()));
}
