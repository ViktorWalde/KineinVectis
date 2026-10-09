//! Recoverable trash and explicit permanent removal share workspace confinement.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::FsError;
use super::confine::new_child_path;

fn removable_path(root: &Path, path: &Path) -> Result<(PathBuf, fs::FileType), FsError> {
    // Canonicalize only the parent. Canonicalizing the entry itself follows a
    // symlink and would remove its target rather than the link selected by the user.
    let target = if path == root {
        root.to_path_buf()
    } else {
        new_child_path(root, path)?
    };
    if target == root {
        return Err(FsError::WorkspaceRoot {
            path: target.display().to_string(),
        });
    }
    let kind = fs::symlink_metadata(&target)
        .map_err(|source| FsError::InvalidPath {
            path: target.display().to_string(),
            source,
        })?
        .file_type();
    Ok((target, kind))
}

/// Moves an existing workspace entry to the system trash. On failure the
/// caller keeps the source and may offer explicit permanent removal.
pub fn move_to_trash(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let (target, _) = removable_path(root, path)?;
    trash::delete(&target).map_err(|error| FsError::Io {
        path: target.display().to_string(),
        source: io::Error::other(error.to_string()),
    })?;
    Ok(target)
}

/// Permanently removes a file or directory inside the workspace root.
pub fn delete(root: &Path, path: &Path) -> Result<PathBuf, FsError> {
    let (target, kind) = removable_path(root, path)?;
    if kind.is_dir() {
        fs::remove_dir_all(&target)
    } else {
        fs::remove_file(&target)
    }
    .map_err(|source| FsError::Io {
        path: target.display().to_string(),
        source,
    })?;
    Ok(target)
}

// A lixeira dos testes e' a do freedesktop (`XDG_DATA_HOME` isolado) e os
// links sao do Unix; no Windows a lixeira e' a do sistema, sem estes testes.
// Dois atributos: o clippy so' reconhece modulo de teste pelo `#[cfg(test)]`.
#[cfg(test)]
#[cfg(unix)]
mod tests {
    use std::{
        env, fs,
        os::unix::fs::symlink,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{FsError, delete, move_to_trash};

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("kinein-{label}-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn run_isolated_child(test_name: &str, root_env: &str, root: &Path, data_home: &Path) {
        let output = Command::new(env::current_exe().unwrap())
            .arg("--exact")
            .arg(test_name)
            .env(root_env, root)
            .env("XDG_DATA_HOME", data_home)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn permanent_remove_deletes_link_not_target() {
        let base = temp_root("delete-link");
        fs::create_dir_all(base.join("workspace")).unwrap();
        let root = crate::platform::canonicalize(&base.join("workspace")).unwrap();
        let target = base.join("outside.txt");
        let link = root.join("link.txt");
        fs::write(&target, "preservar").unwrap();
        symlink(&target, &link).unwrap();
        assert_eq!(delete(&root, &link).unwrap(), link);
        assert!(fs::symlink_metadata(&link).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "preservar");
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn trash_rejects_root_and_outside() {
        let base = temp_root("trash-confine");
        let root = crate::platform::canonicalize(&base).unwrap();
        let outside = root.parent().unwrap();
        assert!(matches!(
            move_to_trash(&root, &root),
            Err(FsError::WorkspaceRoot { .. })
        ));
        assert!(matches!(
            move_to_trash(&root, outside),
            Err(FsError::OutsideRoot { .. })
        ));
        fs::remove_dir(&root).unwrap();
    }

    // O processo filho recebe XDG_DATA_HOME isolado; o teste nunca toca a
    // lixeira real do usuário nem altera variáveis de ambiente globais.
    #[test]
    fn trash_in_isolated_home_child() {
        let Some(root) = env::var_os("KINEIN_TRASH_TEST_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let source = root.join("ação com espaço.txt");
        fs::write(&source, "recuperável").unwrap();
        let moved = move_to_trash(&root, &source).unwrap();
        assert_eq!(moved, source);
        assert!(!source.exists());
        let target = root.parent().unwrap().join("preservar.txt");
        let link = root.join("atalho.txt");
        fs::write(&target, "alvo intacto").unwrap();
        symlink(&target, &link).unwrap();
        assert_eq!(move_to_trash(&root, &link).unwrap(), link);
        assert!(fs::symlink_metadata(&link).is_err());
        assert_eq!(fs::read_to_string(target).unwrap(), "alvo intacto");
    }

    #[test]
    fn trash_unavailable_child_preserves_source() {
        let Some(root) = env::var_os("KINEIN_TRASH_FAILURE_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let source = root.join("manter.txt");
        fs::write(&source, "intacto").unwrap();
        assert!(move_to_trash(&root, &source).is_err());
        assert_eq!(fs::read_to_string(source).unwrap(), "intacto");
    }

    #[test]
    fn trash_preserves_recoverable_payload_and_metadata() {
        let base = temp_root("trash-test");
        let root = base.join("workspace");
        let data = base.join("data");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&data).unwrap();
        run_isolated_child(
            "fsops::remove_ops::tests::trash_in_isolated_home_child",
            "KINEIN_TRASH_TEST_ROOT",
            &root,
            &data,
        );
        assert!(!root.join("ação com espaço.txt").exists());
        let files = data.join("Trash/files");
        let info = data.join("Trash/info");
        let payload = files.join("ação com espaço.txt");
        let link = files.join("atalho.txt");
        assert_eq!(fs::read_to_string(payload).unwrap(), "recuperável");
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
        assert_eq!(
            fs::read_to_string(base.join("preservar.txt")).unwrap(),
            "alvo intacto"
        );
        assert!(info.join("ação com espaço.txt.trashinfo").exists());
        assert!(info.join("atalho.txt.trashinfo").exists());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn trash_unavailable_does_not_delete_source() {
        let base = temp_root("trash-failure");
        let root = base.join("workspace");
        fs::create_dir_all(&root).unwrap();
        let invalid_data_home = base.join("data-home-is-a-file");
        fs::write(&invalid_data_home, "ocupado").unwrap();
        run_isolated_child(
            "fsops::remove_ops::tests::trash_unavailable_child_preserves_source",
            "KINEIN_TRASH_FAILURE_ROOT",
            &root,
            &invalid_data_home,
        );
        assert_eq!(
            fs::read_to_string(root.join("manter.txt")).unwrap(),
            "intacto"
        );
        fs::remove_dir_all(base).unwrap();
    }
}
