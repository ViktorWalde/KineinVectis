//! esptool configuration files a native flash would load (52 §14.38).
//!
//! esptool 4.8 (`esptool/config.py`) and 5.x (esp-pylib 1.2 `ToolConfig`)
//! look at `ESPTOOL_CFGFILE`, then the working directory, the user
//! configuration folder and the home folder, trying `esptool.cfg`,
//! `setup.cfg` and `tox.ini` in each, and load the first file that parses
//! with an `[esptool]` section. That file changes timeouts, attempts and the
//! reset sequences outside the previewed line. The native flow removes
//! `ESPTOOL_*` (so `ESPTOOL_CFGFILE` never applies), runs in the workspace
//! root and refuses while any remaining candidate declares the section.
use std::path::{Path, PathBuf};

/// Candidate names, in esptool's order inside each folder.
const NAMES: [&str; 3] = ["esptool.cfg", "setup.cfg", "tox.ini"];

/// The first candidate that declares `[esptool]`, in esptool's search order.
///
/// A header line is enough (a superset of esp-pylib, which also requires the
/// whole file to parse). A file that is not UTF-8 text is skipped, as esptool
/// skips it.
///
/// # Errors
/// A candidate that exists but cannot be checked: unreadable, a broken link
/// or above the editor's read limit.
pub(in crate::flash) fn first_declared(
    cwd: &Path,
    home: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    for folder in folders(cwd, home) {
        for name in NAMES {
            let path = folder.join(name);
            if std::fs::symlink_metadata(&path).is_err() || path.is_dir() {
                continue;
            }
            let text = crate::platform::canonicalize(&path)
                .map_err(|error| error.to_string())
                .and_then(|resolved| {
                    crate::fsops::read_external_file(&resolved).map_err(|error| match error {
                        crate::fsops::FsError::NotText { .. } => String::new(),
                        other => other.to_string(),
                    })
                });
            match text {
                Ok((_, text)) if declares_section(&text) => return Ok(Some(path)),
                Ok(_) => {}
                Err(reason) if reason.is_empty() => {}
                Err(reason) => return Err(format!("{}: {reason}", path.display())),
            }
        }
    }
    Ok(None)
}

/// The working directory, the user configuration folder, then home.
fn folders(cwd: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let mut folders = vec![cwd.to_path_buf()];
    if let Some(home) = home {
        folders.push(if cfg!(windows) {
            home.join("AppData").join("Local").join("esptool")
        } else {
            home.join(".config").join("esptool")
        });
        folders.push(home.to_path_buf());
    }
    folders
}

/// configparser's header: the stripped line is `[name]` up to its last `]`.
fn declares_section(text: &str) -> bool {
    text.lines().any(|line| {
        line.trim()
            .strip_prefix('[')
            .and_then(|rest| rest.rsplit_once(']'))
            .is_some_and(|(name, _)| name == "esptool")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> (PathBuf, PathBuf) {
        let base =
            std::env::temp_dir().join(format!("{}-esptool-config-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let (cwd, home) = (base.join("project"), base.join("home"));
        fs::create_dir_all(&cwd).unwrap();
        fs::create_dir_all(home.join(".config/esptool")).unwrap();
        (cwd, home)
    }

    #[test]
    fn only_a_header_line_for_esptool_counts() {
        for (text, declared) in [
            ("[esptool]\nconnect_attempts = 9\n", true),
            ("  [esptool]   \n", true),
            ("[esptool] ; reset\n", true),
            ("[flake8]\nmax-line-length = 99\n", false),
            ("# [esptool]\n", false),
            ("; [esptool]\n", false),
            ("[esptool.extra]\n", false),
            ("[esptool] ]\n", false),
            ("esptool = 1\n", false),
        ] {
            assert_eq!(declares_section(text), declared, "{text:?}");
        }
    }

    #[test]
    fn the_search_follows_esptool_order_and_skips_unrelated_files() {
        let (cwd, home) = scratch("order");
        assert_eq!(first_declared(&cwd, Some(&home)), Ok(None));
        fs::write(cwd.join("setup.cfg"), "[flake8]\n").unwrap();
        fs::write(cwd.join("esptool.cfg"), "# nothing for esptool\n").unwrap();
        fs::write(cwd.join("tox.ini"), [0xff_u8, 0xfe, b'['].as_slice()).unwrap();
        assert_eq!(
            first_declared(&cwd, Some(&home)),
            Ok(None),
            "no section, not text"
        );
        fs::write(home.join("setup.cfg"), "[esptool]\n").unwrap();
        assert_eq!(
            first_declared(&cwd, Some(&home)),
            Ok(Some(home.join("setup.cfg")))
        );
        let user = if cfg!(windows) {
            let folder = home.join("AppData/Local/esptool");
            fs::create_dir_all(&folder).unwrap();
            folder
        } else {
            home.join(".config/esptool")
        };
        fs::write(user.join("tox.ini"), "[esptool]\n").unwrap();
        assert_eq!(
            first_declared(&cwd, Some(&home)),
            Ok(Some(user.join("tox.ini")))
        );
        fs::write(cwd.join("tox.ini"), "[esptool]\nreset_delay = 1\n").unwrap();
        assert_eq!(
            first_declared(&cwd, Some(&home)),
            Ok(Some(cwd.join("tox.ini")))
        );
        fs::write(cwd.join("esptool.cfg"), "[esptool]\n").unwrap();
        assert_eq!(
            first_declared(&cwd, Some(&home)),
            Ok(Some(cwd.join("esptool.cfg")))
        );
        assert_eq!(
            first_declared(&cwd, None),
            Ok(Some(cwd.join("esptool.cfg")))
        );
    }

    #[test]
    fn a_candidate_that_cannot_be_checked_refuses() {
        let (cwd, home) = scratch("unchecked");
        let limit = usize::try_from(crate::fsops::MAX_READ_BYTES).unwrap();
        fs::write(cwd.join("esptool.cfg"), vec![b'#'; limit + 1]).unwrap();
        let refused = first_declared(&cwd, Some(&home)).unwrap_err();
        assert!(refused.contains("esptool.cfg"), "{refused}");
    }
}
