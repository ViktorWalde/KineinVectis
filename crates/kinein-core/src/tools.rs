//! External tool detection.
//!
//! The core detects tools by searching an explicit search path and probing the
//! binary with `--version`. The core never installs anything, and never runs a
//! suggestion it emits.
//!
//! Detection is distribution-agnostic by construction: it reads the `PATH` and
//! checks the executable bit. Installation is NOT derived from the `PATH` — a
//! missing tool is precisely the one that is not there — so the core only
//! suggests an install when the command is canonical and independent of the
//! distribution. Guessing a package manager, or translating package names per
//! distribution, would be a guess dressed up as an instruction.

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
    /// Command that installs the tool, when a canonical one exists that does
    /// not depend on the distribution.
    ///
    /// `None` means "install it the way this machine installs things" — o core
    /// nao adivinha gerenciador de pacotes. Sugerir `pacman` numa Fedora, ou
    /// traduzir nome de pacote por distro (`g++` e `gcc-c++` na Fedora), seria
    /// palpite disfarcado de instrucao. Detectar e agnostico e le o PATH;
    /// instalar nao se deduz do PATH, porque a ferramenta ausente e justamente
    /// a que nao esta la.
    pub install_command: Option<&'static str>,
}

/// Tools detected by MVP 0.2, as defined in `docs/10-mvp-plan.md`.
pub const KNOWN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        id: "cargo",
        display_name: "Cargo",
        binary: "cargo",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rustc",
        display_name: "Rust Compiler",
        binary: "rustc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rustup",
        display_name: "rustup",
        binary: "rustup",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rust-analyzer",
        display_name: "rust-analyzer",
        binary: "rust-analyzer",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "cmake",
        display_name: "CMake",
        binary: "cmake",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "ninja",
        display_name: "Ninja",
        binary: "ninja",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "git",
        display_name: "Git",
        binary: "git",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clangd",
        display_name: "clangd",
        binary: "clangd",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clang",
        display_name: "clang",
        binary: "clang",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clangxx",
        display_name: "clang++",
        binary: "clang++",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gcc",
        display_name: "GCC",
        binary: "gcc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gxx",
        display_name: "g++",
        binary: "g++",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gdb",
        display_name: "GDB",
        binary: "gdb",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "lldb",
        display_name: "LLDB",
        binary: "lldb",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "lldb-dap",
        display_name: "lldb-dap",
        binary: "lldb-dap",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "ripgrep",
        display_name: "ripgrep",
        binary: "rg",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "fd",
        display_name: "fd",
        binary: "fd",
        alternative_binary: Some("fdfind"),
        install_command: None,
    },
    // CLIs de IA. Detectar `claude` e o MESMO que detectar `cargo`: e
    // CAPACIDADE ("existe no PATH?"), e capacidade pode viver no core. O que
    // NAO pode voltar e POLITICA — como o programa e executado. O core nao tem
    // ramo por programa: nao injeta flag, nao filtra saida e nao sabe que estas
    // entradas sao "de IA". Quem escolhe e roda e a UI.
    //
    // Aqui o `install_command` existe porque npm e canonico e independente de
    // distro — nao e palpite. (Claude Code tambem tem instalador nativo; a
    // sugestao aponta um caminho que funciona em qualquer distro, e o core
    // nunca a executa.)
    ToolSpec {
        id: "claude",
        display_name: "Claude Code",
        binary: "claude",
        alternative_binary: None,
        install_command: Some("npm install -g @anthropic-ai/claude-code"),
    },
    ToolSpec {
        id: "codex",
        display_name: "Codex",
        binary: "codex",
        alternative_binary: None,
        install_command: Some("npm install -g @openai/codex"),
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
    /// A suggestion only appears for tools whose install command is canonical
    /// and distribution-independent. For everything else the core reports
    /// `Missing` and stays quiet: the package manager is the user's business.
    #[must_use]
    pub fn detect(&self, spec: &ToolSpec) -> ToolInfo {
        let suggestion = || Self::suggested_install_for(spec);

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

    /// Resolves one executable on the detector's configured search path.
    ///
    /// This is used by opt-in launchers such as the AI CLI Bridge. It only
    /// resolves an executable path; it never installs or starts the tool.
    #[must_use]
    pub fn find_binary(&self, binary: &str) -> Option<PathBuf> {
        self.find_in_path(binary)
    }

    fn suggested_install_for(spec: &ToolSpec) -> Option<String> {
        spec.install_command.map(ToOwned::to_owned)
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
        install_command: None,
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
                "lldb-dap",
                "ripgrep",
                "fd",
                "claude",
                "codex"
            ]
        );
    }

    #[test]
    fn missing_distro_tool_suggests_nothing() {
        let dir = temp_bin_dir("missing-distro-tool");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Missing);
        // Ferramenta de distro nao ganha sugestao: o core nao adivinha
        // gerenciador de pacotes nem traduz nome por distro.
        assert!(info.suggested_install.is_none());
        assert!(info.path.is_none());
        assert!(info.message.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn install_suggestion_does_not_depend_on_the_distribution() {
        // Ter (ou nao ter) `pacman` no PATH nao pode mudar a sugestao: essa era
        // exatamente a dependencia de distro que saiu do core.
        let com_pacman = temp_bin_dir("suggestion-with-pacman");
        write_fake_tool(&com_pacman, "pacman", "exit 0");
        let sem_pacman = temp_bin_dir("suggestion-without-pacman");

        let claude = KNOWN_TOOLS
            .iter()
            .find(|spec| spec.id == "claude")
            .expect("claude spec exists");

        let a = ToolDetector::with_search_path(&com_pacman).detect(claude);
        let b = ToolDetector::with_search_path(&sem_pacman).detect(claude);

        assert_eq!(a.suggested_install, b.suggested_install);
        assert_eq!(
            a.suggested_install.as_deref(),
            Some("npm install -g @anthropic-ai/claude-code")
        );
        assert_eq!(
            ToolDetector::with_search_path(&com_pacman)
                .detect(&FAKE_SPEC)
                .suggested_install,
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn ai_clis_are_detected_exactly_like_any_other_tool() {
        // A fatia so esta certa enquanto o core NAO tiver ramo por programa.
        // Detectar `claude` tem de produzir o mesmo formato de resposta que
        // detectar `cargo`: mesma estrutura, mesmo probe, nenhum campo
        // especial. Se alguem escrever `if spec.id == "claude"` no detector
        // para injetar flag, filtrar saida ou mudar o probe, este teste cai.
        let _guard = EXEC_LOCK.lock().unwrap();
        let dir = temp_bin_dir("ai-cli-like-any-other");
        // Os dois fakes ecoam o MESMO texto de proposito: o que se compara e o
        // TRATAMENTO (mesmo probe, mesma estrutura), nao o conteudo.
        write_fake_tool(&dir, "claude", "echo 'ferramenta 9.9.9'");
        write_fake_tool(&dir, "cargo", "echo 'ferramenta 9.9.9'");
        let detector = ToolDetector::with_search_path(&dir);

        let claude = detector.detect(
            KNOWN_TOOLS
                .iter()
                .find(|spec| spec.id == "claude")
                .expect("claude spec exists"),
        );
        let cargo = detector.detect(&FAKE_SPEC);

        assert_eq!(claude.status, cargo.status);
        assert_eq!(claude.version, cargo.version);
        assert!(claude.suggested_install.is_none());
        assert_eq!(
            claude.path.as_deref(),
            Some(dir.join("claude").to_str().unwrap())
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
