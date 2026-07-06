//! External tool detection.
//!
//! The core detects tools by searching an explicit search path and probing the
//! binary with `--version`. The core never installs anything: missing tools
//! only produce a suggested install command for the UI to display.

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{ToolInfo, ToolStatus};

/// Static description of one tool the core knows how to detect.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ToolSpec {
    /// Stable tool identifier used in the IPC protocol.
    pub id: &'static str,
    /// Human-readable name shown by the UI.
    pub display_name: &'static str,
    /// Binary name searched on the path.
    pub binary: &'static str,
    /// Alternative binary name used by some distributions.
    pub alternative_binary: Option<&'static str>,
    /// Package that provides the binary on CachyOS/Arch.
    pub pacman_package: &'static str,
}

/// Tools detected by MVP 0.2, as defined in `docs/10-mvp-plan.md`.
pub const KNOWN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        id: "cargo",
        display_name: "Cargo",
        binary: "cargo",
        alternative_binary: None,
        pacman_package: "rustup",
    },
    ToolSpec {
        id: "rustc",
        display_name: "Rust Compiler",
        binary: "rustc",
        alternative_binary: None,
        pacman_package: "rustup",
    },
    ToolSpec {
        id: "rustup",
        display_name: "rustup",
        binary: "rustup",
        alternative_binary: None,
        pacman_package: "rustup",
    },
    ToolSpec {
        id: "rust-analyzer",
        display_name: "rust-analyzer",
        binary: "rust-analyzer",
        alternative_binary: None,
        pacman_package: "rust-analyzer",
    },
    ToolSpec {
        id: "cmake",
        display_name: "CMake",
        binary: "cmake",
        alternative_binary: None,
        pacman_package: "cmake",
    },
    ToolSpec {
        id: "ninja",
        display_name: "Ninja",
        binary: "ninja",
        alternative_binary: None,
        pacman_package: "ninja",
    },
    ToolSpec {
        id: "git",
        display_name: "Git",
        binary: "git",
        alternative_binary: None,
        pacman_package: "git",
    },
    ToolSpec {
        id: "clangd",
        display_name: "clangd",
        binary: "clangd",
        alternative_binary: None,
        pacman_package: "clang",
    },
    ToolSpec {
        id: "clang",
        display_name: "clang",
        binary: "clang",
        alternative_binary: None,
        pacman_package: "clang",
    },
    ToolSpec {
        id: "clangxx",
        display_name: "clang++",
        binary: "clang++",
        alternative_binary: None,
        pacman_package: "clang",
    },
    ToolSpec {
        id: "gcc",
        display_name: "GCC",
        binary: "gcc",
        alternative_binary: None,
        pacman_package: "gcc",
    },
    ToolSpec {
        id: "gxx",
        display_name: "g++",
        binary: "g++",
        alternative_binary: None,
        pacman_package: "gcc",
    },
    ToolSpec {
        id: "gdb",
        display_name: "GDB",
        binary: "gdb",
        alternative_binary: None,
        pacman_package: "gdb",
    },
    ToolSpec {
        id: "lldb",
        display_name: "LLDB",
        binary: "lldb",
        alternative_binary: None,
        pacman_package: "lldb",
    },
    ToolSpec {
        id: "ripgrep",
        display_name: "ripgrep",
        binary: "rg",
        alternative_binary: None,
        pacman_package: "ripgrep",
    },
    ToolSpec {
        id: "fd",
        display_name: "fd",
        binary: "fd",
        alternative_binary: Some("fdfind"),
        pacman_package: "fd",
    },
];

/// Detects external tools on a configurable search path.
///
/// The default detector uses the `PATH` environment variable. Tests inject a
/// fixed search path so detection stays hermetic.
#[derive(Debug, Clone, Default)]
pub struct ToolDetector {
    search_path: Option<OsString>,
}

impl ToolDetector {
    /// Builds a detector that reads `PATH` from the environment.
    #[must_use]
    pub const fn from_environment() -> Self {
        Self { search_path: None }
    }

    /// Builds a detector with a fixed search path instead of `PATH`.
    #[must_use]
    pub fn with_search_path(search_path: impl Into<OsString>) -> Self {
        Self {
            search_path: Some(search_path.into()),
        }
    }

    /// Detects every tool in [`KNOWN_TOOLS`].
    #[must_use]
    pub fn detect_all(&self) -> Vec<ToolInfo> {
        KNOWN_TOOLS.iter().map(|spec| self.detect(spec)).collect()
    }

    /// Detects a single tool and reports its structured status.
    ///
    /// The pacman install suggestion only appears when `pacman` itself is
    /// available: on other distributions the suggested command would be
    /// useless noise.
    #[must_use]
    pub fn detect(&self, spec: &ToolSpec) -> ToolInfo {
        let suggestion = || self.suggested_install_for(spec);

        self.find_tool_binary(spec).map_or_else(
            || ToolInfo {
                id: spec.id.to_owned(),
                display_name: spec.display_name.to_owned(),
                status: ToolStatus::Missing,
                path: None,
                version: None,
                suggested_install: suggestion(),
                message: Some(format!("{} nao foi encontrado no PATH.", spec.display_name)),
            },
            |path| match probe_version(&path) {
                Ok(version) => ToolInfo {
                    id: spec.id.to_owned(),
                    display_name: spec.display_name.to_owned(),
                    status: ToolStatus::Detected,
                    path: Some(path.display().to_string()),
                    version: Some(version),
                    suggested_install: None,
                    message: None,
                },
                Err(reason) => ToolInfo {
                    id: spec.id.to_owned(),
                    display_name: spec.display_name.to_owned(),
                    status: ToolStatus::Failed,
                    path: Some(path.display().to_string()),
                    version: None,
                    suggested_install: suggestion(),
                    message: Some(format!(
                        "{} foi encontrado mas nao respondeu ao probe de versao: {reason}",
                        spec.display_name
                    )),
                },
            },
        )
    }

    fn suggested_install_for(&self, spec: &ToolSpec) -> Option<String> {
        self.find_in_path("pacman")
            .map(|_| format!("sudo pacman -S {}", spec.pacman_package))
    }

    fn find_in_path(&self, binary: &str) -> Option<PathBuf> {
        let search_path = self.search_path.clone().or_else(|| env::var_os("PATH"))?;

        env::split_paths(&search_path)
            .map(|directory| directory.join(binary))
            .find(|candidate| is_executable(candidate))
    }

    fn find_tool_binary(&self, spec: &ToolSpec) -> Option<PathBuf> {
        self.find_in_path(spec.binary).or_else(|| {
            spec.alternative_binary
                .and_then(|binary| self.find_in_path(binary))
        })
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn probe_version(path: &Path) -> Result<String, String> {
    match Command::new(path).arg("--version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| "a ferramenta nao informou versao".to_owned())
        }
        Ok(output) => Err(format!(
            "o comando de versao terminou com {}",
            output.status
        )),
        Err(error) => Err(format!("falha ao executar a ferramenta: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use kinein_protocol::ToolStatus;

    use super::{KNOWN_TOOLS, ToolDetector, ToolSpec};

    const FAKE_SPEC: ToolSpec = ToolSpec {
        id: "cargo",
        display_name: "Cargo",
        binary: "cargo",
        alternative_binary: None,
        pacman_package: "rustup",
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    fn temp_bin_dir(test_name: &str) -> PathBuf {
        let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-{test_name}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[cfg(unix)]
    fn write_fake_tool(dir: &std::path::Path, name: &str, script_body: &str) {
        use std::os::unix::fs::PermissionsExt;

        let path = dir.join(name);
        fs::write(&path, format!("#!/bin/sh\n{script_body}\n")).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }

    #[test]
    fn known_tools_cover_mvp_0_2_list() {
        let ids = KNOWN_TOOLS.iter().map(|spec| spec.id).collect::<Vec<_>>();

        assert_eq!(
            ids,
            [
                "cargo",
                "rustc",
                "rustup",
                "rust-analyzer",
                "cmake",
                "ninja",
                "git",
                "clangd",
                "clang",
                "clangxx",
                "gcc",
                "gxx",
                "gdb",
                "lldb",
                "ripgrep",
                "fd"
            ]
        );
    }

    #[test]
    fn missing_tool_without_pacman_has_no_suggestion() {
        let dir = temp_bin_dir("missing-no-pacman");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Missing);
        assert!(info.suggested_install.is_none());
        assert!(info.path.is_none());
        assert!(info.message.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn missing_tool_with_pacman_reports_suggestion() {
        let dir = temp_bin_dir("missing-with-pacman");
        write_fake_tool(&dir, "pacman", "exit 0");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Missing);
        assert_eq!(
            info.suggested_install.as_deref(),
            Some("sudo pacman -S rustup")
        );
    }

    #[cfg(unix)]
    #[test]
    fn fd_detection_accepts_fdfind_binary_name() {
        let _guard = EXEC_LOCK.lock().unwrap();
        let dir = temp_bin_dir("fd-fdfind");
        write_fake_tool(&dir, "fdfind", "echo 'fdfind 10.2.0'");
        let detector = ToolDetector::with_search_path(&dir);
        let fd_spec = KNOWN_TOOLS
            .iter()
            .find(|spec| spec.id == "fd")
            .expect("fd spec exists");

        let info = detector.detect(fd_spec);

        assert_eq!(info.status, ToolStatus::Detected);
        assert!(
            info.path
                .as_deref()
                .is_some_and(|path| path.ends_with("fdfind"))
        );
    }

    /// Serializes tests that write and execute fake tool scripts.
    ///
    /// Without this, one test can fork while another still holds the write
    /// descriptor of its script, and the exec fails with `ETXTBSY`.
    #[cfg(unix)]
    static EXEC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(unix)]
    #[test]
    fn detected_tool_reports_path_and_version() {
        let _guard = EXEC_LOCK.lock().unwrap();
        let dir = temp_bin_dir("detected");
        write_fake_tool(&dir, "cargo", "echo 'cargo 1.99.0 (fake)'");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Detected);
        assert_eq!(info.version.as_deref(), Some("cargo 1.99.0 (fake)"));
        assert_eq!(
            info.path.as_deref(),
            Some(dir.join("cargo").to_str().unwrap())
        );
        assert!(info.suggested_install.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn broken_tool_reports_failed_with_human_message() {
        let _guard = EXEC_LOCK.lock().unwrap();
        let dir = temp_bin_dir("broken");
        write_fake_tool(&dir, "cargo", "exit 3");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Failed);
        assert!(info.version.is_none());
        assert!(info.message.as_deref().unwrap().contains("Cargo"));
    }

    #[test]
    fn detect_all_returns_one_entry_per_known_tool() {
        let dir = temp_bin_dir("all");
        let detector = ToolDetector::with_search_path(&dir);

        let tools = detector.detect_all();

        assert_eq!(tools.len(), KNOWN_TOOLS.len());
        assert!(tools.iter().all(|tool| tool.status == ToolStatus::Missing));
    }
}
