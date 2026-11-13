//! The guard of the `.kinein` (INF-13, INF-15 and INF-20): each component of
//! the path, the write and the removal that do not follow a link, the read
//! that does not follow one nor wait on a FIFO, and the summary for the
//! health band.
//!
//! The links are of folders, or to what does not exist: `ln -s` on Unix and
//! the junction on Windows, which needs no privilege (60 §3.3). The FIFO
//! exists only on Unix; on Windows those cases are skipped (63 §E.2).

use std::path::PathBuf;

use super::{fifo, file, inside, link, problem, read, remove, summary, unblocked, write};
use crate::fsops::MAX_READ_BYTES;

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

/// The removal (INF-20): a file, and a folder with everything in it; absent
/// is done. The `.kinein`, a folder in the middle and the file that are links
/// are refused, and what is outside stays; so are the entry of the wrong kind
/// and the FIFO, which stay too.
#[test]
fn the_removal_refuses_a_link_and_removes_nothing_outside() {
    let root = base("remocao");
    write(&root, "configuration.json", b"{}").unwrap();
    write(&root, "presets/dev/.cmake/api/v1/query/codemodel-v2", b"").unwrap();
    assert_eq!(remove(&root, "configuration.json", false), Ok(()));
    assert!(!root.join(".kinein").join("configuration.json").exists());
    assert_eq!(remove(&root, "presets/dev", true), Ok(()));
    assert!(!root.join(".kinein").join("presets").join("dev").exists());
    assert!(root.join(".kinein").join("presets").is_dir());
    assert_eq!(remove(&root, "presets/dev", true), Ok(()));
    assert_eq!(remove(&root, "configuration.json", false), Ok(()));

    let outside = base("remocao-fora");
    std::fs::create_dir_all(outside.join("dev")).unwrap();
    std::fs::write(outside.join("dev").join("da-pessoa.txt"), "fica").unwrap();
    std::fs::write(outside.join("configuration.json"), "fica").unwrap();
    let preserved = || {
        assert_eq!(
            std::fs::read_to_string(outside.join("dev").join("da-pessoa.txt")).unwrap(),
            "fica"
        );
        assert_eq!(
            std::fs::read_to_string(outside.join("configuration.json")).unwrap(),
            "fica"
        );
    };
    let folder = base("remocao-pasta-ligada");
    link(&outside, &folder.join(".kinein"));
    assert_eq!(
        remove(&folder, "configuration.json", false),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(remove(&folder, "dev", true), Err(FOLDER_LINK.to_owned()));
    preserved();

    let middle = base("remocao-meio-ligado");
    std::fs::create_dir_all(middle.join(".kinein")).unwrap();
    link(&outside, &middle.join(".kinein").join("presets"));
    assert_eq!(
        remove(&middle, "presets/dev", true),
        Err(".kinein/presets é um link simbólico, que a IDE não segue no projeto".to_owned())
    );
    preserved();

    let leaf = base("remocao-arquivo-ligado");
    std::fs::create_dir_all(leaf.join(".kinein")).unwrap();
    let placed = leaf.join(".kinein").join("configuration.json");
    let missing = outside.join("nao-existe");
    link(&missing, &placed);
    assert!(remove(&leaf, "configuration.json", false).is_err());
    assert!(
        std::fs::symlink_metadata(&placed)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!missing.exists());
    preserved();

    let kinds = base("remocao-tipos");
    write(&kinds, "session.json", b"{}").unwrap();
    std::fs::create_dir_all(kinds.join(".kinein").join("build")).unwrap();
    assert_eq!(
        remove(&kinds, "build", false),
        Err(".kinein/build não é um arquivo regular".to_owned())
    );
    assert_eq!(
        remove(&kinds, "session.json", true),
        Err(".kinein/session.json não é uma pasta".to_owned())
    );
    assert!(kinds.join(".kinein").join("build").is_dir());
    assert!(kinds.join(".kinein").join("session.json").is_file());
    let held = kinds.join(".kinein").join("rigor");
    if fifo(&held) {
        assert_eq!(
            remove(&kinds, "rigor", false),
            Err(".kinein/rigor não é um arquivo regular".to_owned())
        );
        assert!(std::fs::symlink_metadata(&held).is_ok());
    }
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

const LIMIT: &str = "passa de 1 MiB, o limite de leitura da IDE";

/// The read (INF-15): absent is `None` at any depth; the regular file is
/// read up to the limit; the folder in place of the file, the file in place
/// of a folder, the link of the `.kinein`, of a folder in the middle and of
/// the file say which and why, and nothing outside is read.
#[test]
fn the_read_checks_each_component_on_the_descriptor() {
    let root = base("leitura");
    assert_eq!(read(&root, "session.json"), Ok(None));
    std::fs::create_dir_all(root.join(".kinein").join("build")).unwrap();
    assert_eq!(read(&root, "session.json"), Ok(None));
    assert_eq!(read(&root, "build/debug/.kinein-kit"), Ok(None));
    std::fs::write(root.join(".kinein").join("session.json"), "{}").unwrap();
    assert_eq!(read(&root, "session.json"), Ok(Some(b"{}".to_vec())));
    assert_eq!(
        read(&root, "build"),
        Err(".kinein/build não é um arquivo regular".to_owned())
    );
    assert_eq!(
        read(&root, "session.json/x"),
        Err(".kinein/session.json não é uma pasta".to_owned())
    );

    let outside = base("leitura-fora");
    std::fs::write(outside.join(".kinein-kit"), "de fora").unwrap();
    link(&outside, &root.join(".kinein").join("build").join("debug"));
    assert_eq!(
        read(&root, "build/debug/.kinein-kit"),
        Err(".kinein/build/debug é um link simbólico, que a IDE não segue no projeto".to_owned())
    );
    let missing = outside.join("ausente");
    link(&missing, &root.join(".kinein").join("python.json"));
    assert_eq!(
        read(&root, "python.json"),
        Err(".kinein/python.json é um link simbólico, que a IDE não segue no projeto".to_owned())
    );
    if !cfg!(windows) {
        // A junção do Windows só liga pasta: o link para um arquivo de fora
        // com conteúdo é só do Unix.
        std::fs::write(outside.join("grafana.json"), "de fora").unwrap();
        link(
            &outside.join("grafana.json"),
            &root.join(".kinein").join("grafana.json"),
        );
        assert_eq!(
            read(&root, "grafana.json"),
            Err(
                ".kinein/grafana.json é um link simbólico, que a IDE não segue no projeto"
                    .to_owned()
            )
        );
    }

    let project = base("leitura-pasta-ligada");
    link(&outside, &project.join(".kinein"));
    assert_eq!(read(&project, ".kinein-kit"), Err(FOLDER_LINK.to_owned()));
    assert!(!missing.exists());
}

/// The limit: exactly 1 MiB is read, one byte more is refused, and the file
/// stays as it was.
#[test]
fn the_read_takes_up_to_the_limit() {
    let root = base("limite");
    std::fs::create_dir_all(root.join(".kinein")).unwrap();
    let path = root.join(".kinein").join("toolchain.json");
    std::fs::File::create(&path)
        .unwrap()
        .set_len(MAX_READ_BYTES)
        .unwrap();
    assert_eq!(
        read(&root, "toolchain.json").map(|bytes| bytes.map(|bytes| bytes.len())),
        Ok(Some(usize::try_from(MAX_READ_BYTES).unwrap()))
    );
    std::fs::File::create(&path)
        .unwrap()
        .set_len(MAX_READ_BYTES + 1)
        .unwrap();
    assert_eq!(
        read(&root, "toolchain.json"),
        Err(format!(".kinein/toolchain.json {LIMIT}"))
    );
    assert_eq!(std::fs::metadata(&path).unwrap().len(), MAX_READ_BYTES + 1);
}

/// The FIFO in place of the file, or of a folder in the middle, is refused
/// at once, without a writer on the other side; the band names the one right
/// inside the `.kinein`, before a link that comes later by name.
#[test]
fn a_fifo_is_refused_without_waiting_for_a_writer() {
    let root = base("fifo");
    std::fs::create_dir_all(root.join(".kinein")).unwrap();
    let file = root.join(".kinein").join("session.json");
    if !fifo(&file) {
        return;
    }
    let reader = root.clone();
    assert_eq!(
        unblocked(&file, move || read(&reader, "session.json")),
        Err(".kinein/session.json não é um arquivo regular".to_owned())
    );
    let folder = root.join(".kinein").join("build");
    assert!(fifo(&folder));
    let reader = root.clone();
    assert_eq!(
        unblocked(&folder, move || read(&reader, "build/.kinein-kit")),
        Err(".kinein/build não é uma pasta".to_owned())
    );
    link(
        &root.join("ausente"),
        &root.join(".kinein").join("toolchain.json"),
    );
    assert_eq!(
        summary(&root),
        Some(".kinein/build não é um arquivo regular nem uma pasta".to_owned())
    );
}

/// The project and the path under its `.kinein`, with `/`; the outermost
/// `.kinein` counts, so a preset named `.kinein` does not move the root.
#[test]
fn inside_finds_the_project_of_a_folder_in_its_state() {
    let root = base("dentro");
    let state = root.join(".kinein");
    assert_eq!(inside(&state), Some((root.as_path(), String::new())));
    assert_eq!(
        inside(&state.join("build").join("debug")),
        Some((root.as_path(), "build/debug".to_owned()))
    );
    let preset = state.join("presets").join(".kinein");
    assert_eq!(
        inside(&preset),
        Some((root.as_path(), "presets/.kinein".to_owned()))
    );
    assert_eq!(inside(&root.join("build")), None);
}
