//! The `python.json` of the created project goes through the guard of the
//! `.kinein` (INF-13), like every file the core writes there.

use std::path::PathBuf;

use crate::kinein_dir::link;

const FOLDER_LINK: &str = ".kinein é um link simbólico, que a IDE não segue no projeto";

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-project-options-tests")
        .join(format!("{}-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

/// The `python.json` of the created project pins the interpreter through
/// the guard too: the linked `.kinein` refuses it, and nothing goes outside.
#[test]
fn the_python_pin_refuses_the_linked_state_folder() {
    let outside = base("python-fora");
    let project = base("python");
    link(&outside, &project.join(".kinein"));
    let error = super::pin_python(&project, "/usr/bin/python3")
        .unwrap_err()
        .to_string();
    assert!(error.contains(FOLDER_LINK), "{error}");
    assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
}
