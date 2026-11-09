//! Shell integration for a command requested at `terminal.open` (INF-02, 62 §3.4).
//!
//! A command typed as soon as the shell process exists races its startup:
//! `zsh-newuser-install` ate the first character and a theme's configuration
//! wizard took the line as its answers. The shell itself says when it reads
//! input: these scripts make bash, zsh and fish print OSC 133;A at the prompt,
//! and the session writes the command only then. Each script reads the user's
//! own startup files first, so the shell is the one the user configured.
//!
//! Only a session opened with a command is integrated; a plain terminal starts
//! the shell exactly as before.

use std::{
    ffi::OsString,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

const BASH: &str = include_str!("integration/bash.sh");
const ZSHENV: &str = include_str!("integration/zshenv.zsh");
const ZSHRC: &str = include_str!("integration/zshrc.zsh");
/// Runs after `config.fish`; fish 4 also prints its own marker, later.
const FISH: &str = "function __kinein_prompt_marker --on-event fish_prompt; \
                    printf '\\e]133;A\\a'; end";

/// Files under the integration directory, by relative path.
const FILES: [(&str, &str); 3] = [
    ("kinein.bash", BASH),
    ("zsh/.zshenv", ZSHENV),
    ("zsh/.zshrc", ZSHRC),
];

/// How to start an integrated shell.
#[derive(Debug)]
pub(super) struct Launch {
    pub(super) args: Vec<String>,
    /// Variables to set; `None` removes one inherited from the IDE.
    pub(super) env: Vec<(&'static str, Option<OsString>)>,
}

/// A shell this module integrates.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// The shell `program` runs, by its file name; `None` for one this module
    /// does not know, which then receives the command right at open.
    pub(super) fn of(program: &str) -> Option<Self> {
        match Path::new(program).file_name()?.to_str()? {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }

    /// How to start it with the scripts `prepare` wrote in `directory`.
    /// `user_zdotdir` is the ZDOTDIR the user's zsh would have read.
    pub(super) fn launch(self, directory: &Path, user_zdotdir: Option<OsString>) -> Launch {
        match self {
            Self::Bash => Launch {
                args: vec![
                    "--init-file".to_owned(),
                    directory.join("kinein.bash").to_string_lossy().into_owned(),
                ],
                env: Vec::new(),
            },
            Self::Zsh => Launch {
                args: Vec::new(),
                env: vec![
                    ("ZDOTDIR", Some(directory.join("zsh").into_os_string())),
                    ("KINEIN_ZDOTDIR", user_zdotdir),
                ],
            },
            Self::Fish => Launch {
                args: vec!["--init-command".to_owned(), FISH.to_owned()],
                env: Vec::new(),
            },
        }
    }
}

/// Writes the scripts under `root`, in a private directory named by their
/// content, and returns it. Unchanged files are left alone, so concurrent IDE
/// instances agree; a changed one is replaced whole, never half-written.
pub(super) fn prepare(root: &Path) -> io::Result<PathBuf> {
    let mut joined = Vec::new();
    for (relative, contents) in FILES {
        joined.extend_from_slice(relative.as_bytes());
        joined.push(0);
        joined.extend_from_slice(contents.as_bytes());
        joined.push(0);
    }
    let mut name = String::new();
    for byte in &Sha256::digest(&joined)[..8] {
        let _ = write!(name, "{byte:02x}");
    }
    let directory = root.join(name);
    crate::platform::create_private_directory(&directory.join("zsh"))?;
    for (relative, contents) in FILES {
        let path = directory.join(relative);
        if fs::read(&path).is_ok_and(|current| current == contents.as_bytes()) {
            continue;
        }
        let partial = directory.join(format!(
            ".{}.{}",
            std::process::id(),
            relative.replace('/', "-")
        ));
        fs::write(&partial, contents)?;
        fs::rename(&partial, &path)?;
    }
    Ok(directory)
}

/// OSC 133;A, the prompt start, followed by `;` (options), BEL or ESC (ST).
const MARKER: &[u8] = b"\x1b]133;A";

/// Finds the first prompt marker in PTY output read in arbitrary chunks.
#[derive(Debug, Default)]
pub(super) struct PromptMarker {
    matched: usize,
}

impl PromptMarker {
    /// `true` when a marker ends inside `bytes`.
    pub(super) fn feed(&mut self, bytes: &[u8]) -> bool {
        for &byte in bytes {
            if self.matched == MARKER.len() {
                if matches!(byte, b';' | 0x07 | 0x1b) {
                    self.matched = 0;
                    return true;
                }
                self.matched = 0;
            }
            if byte == MARKER[self.matched] {
                self.matched += 1;
            } else {
                // ESC appears only at the start of the marker.
                self.matched = usize::from(byte == MARKER[0]);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(chunks: &[&[u8]]) -> bool {
        let mut marker = PromptMarker::default();
        chunks.iter().any(|chunk| marker.feed(chunk))
    }

    #[test]
    fn marker_is_found_whole_split_and_with_each_terminator() {
        assert!(found(&[b"prompt\x1b]133;A\x07$ "]));
        assert!(found(&[b"\x1b]133;A;click_events=1\x1b\\"]));
        assert!(found(&[b"\x1b]133;A\x1b\\"]));
        for split in 1..=MARKER.len() {
            let whole = b"x\x1b]133;A\x07y";
            assert!(
                found(&[&whole[..split], &whole[split..]]),
                "split at {split}"
            );
        }
        assert!(found(&[
            b"\x1b", b"]", b"1", b"3", b"3", b";", b"A", b"\x07"
        ]));
    }

    #[test]
    fn other_sequences_are_not_the_prompt_marker() {
        assert!(!found(&[b"\x1b]133;B\x07\x1b]133;C\x07\x1b]133;D;0\x07"]));
        assert!(!found(&[b"\x1b]133;AB\x07"]));
        assert!(!found(&[b"\x1b]1133;A\x07"]));
        assert!(!found(&[b"\x1b]133;A"]));
        assert!(!found(&[b"]133;A\x07"]));
        // An ESC that breaks a partial match starts the next one.
        assert!(found(&[b"\x1b]13\x1b]133;A\x07"]));
        assert!(found(&[b"\x1b]133;A\x1b]133;A\x07"]));
    }

    #[test]
    fn known_shells_by_file_name_only() {
        let directory = Path::new("/state/kinein");
        let bash = Shell::of("/usr/bin/bash").unwrap().launch(directory, None);
        assert_eq!(bash.args, ["--init-file", "/state/kinein/kinein.bash"]);
        let zsh = Shell::of("zsh")
            .unwrap()
            .launch(directory, Some("/home/u/.config/zsh".into()));
        assert!(zsh.args.is_empty());
        assert_eq!(
            zsh.env,
            [
                ("ZDOTDIR", Some(OsString::from("/state/kinein/zsh"))),
                (
                    "KINEIN_ZDOTDIR",
                    Some(OsString::from("/home/u/.config/zsh"))
                ),
            ]
        );
        assert_eq!(
            Shell::of("/bin/zsh").unwrap().launch(directory, None).env[1],
            ("KINEIN_ZDOTDIR", None)
        );
        assert_eq!(
            Shell::of("/usr/bin/fish")
                .unwrap()
                .launch(directory, None)
                .args[0],
            "--init-command"
        );
        for other in [
            "/bin/sh",
            "/usr/bin/dash",
            "/usr/bin/bash5",
            "/opt/zsh/bin/zsh-5.9",
            "nu",
            "",
        ] {
            assert_eq!(Shell::of(other), None, "{other}");
        }
    }

    #[test]
    fn prepare_writes_private_files_and_repairs_a_changed_one() {
        let root = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-integration-prepare", std::process::id()));
        drop(fs::remove_dir_all(&root));
        let directory = prepare(&root).unwrap();
        assert_eq!(
            directory,
            prepare(&root).unwrap(),
            "same content, same directory"
        );
        for (relative, contents) in FILES {
            assert_eq!(
                fs::read_to_string(directory.join(relative)).unwrap(),
                contents
            );
        }
        fs::write(directory.join("zsh/.zshrc"), "echo altered\n").unwrap();
        prepare(&root).unwrap();
        assert_eq!(
            fs::read_to_string(directory.join("zsh/.zshrc")).unwrap(),
            ZSHRC
        );
        let leftovers: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with('.'))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
        for private in [directory.clone(), directory.join("zsh")] {
            assert!(
                crate::platform::is_private_directory(&private),
                "{} must be private",
                private.display()
            );
        }
        drop(fs::remove_dir_all(&root));
    }
}
