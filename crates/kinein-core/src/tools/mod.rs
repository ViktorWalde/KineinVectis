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

mod known;
pub mod search_dirs;

pub use search_dirs::install_root;
use search_dirs::{extra_search_dirs, installed_bin_dirs};

use std::{
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
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

pub use known::KNOWN_TOOLS;

/// Detects external tools on a configurable search path.
///
/// The default detector uses the `PATH` environment variable. Tests inject a
/// fixed search path so detection stays hermetic. A pasta onde a IDE instala
/// toolchains (`install_root`) e' enumerada A CADA busca — o que o provedor
/// de instalacao acabou de desempacotar entra na proxima deteccao.
#[derive(Debug, Clone, Default)]
pub struct ToolDetector {
    /// O `PATH` (do ambiente ou fixado) mais os diretorios extras, na ordem.
    search_path: Option<OsString>,
    /// Os diretorios que entram logo DEPOIS do `PATH` (os extras de
    /// `search_dirs`), quando o detector veio do ambiente.
    extra_dirs: Vec<PathBuf>,
    /// A pasta da IDE (`toolchains/<id>/<versao>/bin`), lida a cada busca.
    install_root: Option<PathBuf>,
}

impl ToolDetector {
    /// Builds a detector that reads `PATH` from the environment — and, desde
    /// 2026-09-12 (`integracoes/39`), os diretorios onde os distribuidores de
    /// toolchain instalam por padrao (`~/.local/xPacks`, `~/.espressif/tools`,
    /// a pasta da IDE, `/opt/*/bin`): o `arm-none-eabi-gcc` de um tarball em
    /// `/opt` e' achado sem o usuario editar o PATH. O `PATH` vem PRIMEIRO,
    /// para o que o usuario escolheu vencer o que a IDE encontrou; a pasta da
    /// IDE vem logo depois dele.
    #[must_use]
    pub fn from_environment() -> Self {
        let home = env::var_os("HOME").map(PathBuf::from);
        let extra_dirs = home.as_ref().map_or_else(Vec::new, |home| {
            let xpacks = env::var_os("XPACKS_STORE_FOLDER").map(PathBuf::from);
            let idf = env::var_os("IDF_TOOLS_PATH").map(PathBuf::from);
            extra_search_dirs(home, xpacks.as_deref(), idf.as_deref())
        });
        Self {
            search_path: env::var_os("PATH"),
            extra_dirs,
            install_root: home.map(|home| install_root(&home)),
        }
    }

    /// Builds a detector with a fixed search path instead of `PATH`.
    #[must_use]
    pub fn with_search_path(search_path: impl Into<OsString>) -> Self {
        Self {
            search_path: Some(search_path.into()),
            extra_dirs: Vec::new(),
            install_root: None,
        }
    }

    /// A pasta onde a IDE instala toolchains — para um teste apontar uma
    /// pasta temporaria e para o provedor saber onde desempacotar.
    #[must_use]
    pub fn with_install_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.install_root = Some(root.into());
        self
    }

    /// A pasta da IDE deste detector, se ha' uma.
    #[must_use]
    pub fn install_root(&self) -> Option<&Path> {
        self.install_root.as_deref()
    }

    /// Todos os diretorios onde se procura, na ordem: `PATH`, a pasta da IDE
    /// (o que existe AGORA), os extras dos distribuidores.
    fn search_dirs(&self) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = self
            .search_path
            .as_ref()
            .map(|p| env::split_paths(p).collect())
            .unwrap_or_default();
        let instaladas = self
            .install_root
            .as_deref()
            .map(installed_bin_dirs)
            .unwrap_or_default();
        for extra in instaladas
            .into_iter()
            .chain(self.extra_dirs.iter().cloned())
        {
            if !dirs.contains(&extra) {
                dirs.push(extra);
            }
        }
        dirs
    }

    /// Detects every tool in [`KNOWN_TOOLS`].
    #[must_use]
    pub fn detect_all(&self) -> Vec<ToolInfo> {
        KNOWN_TOOLS.iter().map(|spec| self.detect(spec)).collect()
    }

    /// Só a PRESENÇA de cada ferramenta (o caminho), sem o probe de versão
    /// (Etapa 2 F6-b, 2026-09-18). Medido: `detect_all` custa ~1,4 s (62
    /// processos `--version`) e era chamado SINCRONO no `workspace.open` de
    /// um registro vazio — o laço parado antes de a primeira tela aparecer.
    /// Este e' o que a toolchain precisa para escolher; a versão vem do scan
    /// de ambiente, que roda como job e substitui o registro.
    #[must_use]
    pub fn detect_all_presence(&self) -> Vec<ToolInfo> {
        KNOWN_TOOLS
            .iter()
            .map(|spec| {
                self.find_tool_binary(spec).map_or_else(
                    || ToolInfo {
                        id: spec.id.to_owned(),
                        display_name: spec.display_name.to_owned(),
                        status: ToolStatus::Missing,
                        path: None,
                        version: None,
                        suggested_install: Self::suggested_install_for(spec),
                        message: Some(format!("{} nao foi encontrado no PATH.", spec.display_name)),
                    },
                    |path| ToolInfo {
                        id: spec.id.to_owned(),
                        display_name: spec.display_name.to_owned(),
                        status: ToolStatus::Detected,
                        path: Some(path.display().to_string()),
                        version: None,
                        suggested_install: None,
                        message: None,
                    },
                )
            })
            .collect()
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

    fn suggested_install_for(spec: &ToolSpec) -> Option<String> {
        spec.install_command.map(ToOwned::to_owned)
    }

    /// Where `binary` lives on the search path, if anywhere. `pub(crate)` for
    /// the domains that pick between binaries (`container`: docker vs podman).
    pub(crate) fn find_in_path(&self, binary: &str) -> Option<PathBuf> {
        self.search_dirs()
            .into_iter()
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

/// Tentativas extras de exec quando o binario responde `ETXTBSY`.
///
/// `ETXTBSY` ("Text file busy") nao diz que a ferramenta esta quebrada: diz que
/// alguem ainda segura um descritor de ESCRITA para aquele arquivo no instante
/// do `execve` — o linker terminando de gravar `target/debug/foo`, um
/// gerenciador de pacotes atualizando o binario, ou um `fork` concorrente que
/// herdou o descritor antes de fechar no `exec`. E' transitorio por definicao e
/// some sozinho em milissegundos; tratar como "ferramenta falhou" e' reportar
/// erro por uma corrida. Referencia: `execve(2)`, secao ERRORS (Linux man-pages
/// 6.9) — "ETXTBSY: The specified executable was open for writing by one or
/// more processes."
const EXEC_BUSY_ATTEMPTS: u32 = 20;

/// Espera entre duas tentativas de exec apos `ETXTBSY` (total <= 200 ms).
const EXEC_BUSY_BACKOFF: Duration = Duration::from_millis(10);

fn probe_version(path: &Path) -> Result<String, String> {
    for remaining in (0..EXEC_BUSY_ATTEMPTS).rev() {
        match Command::new(path).arg("--version").output() {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| "a ferramenta nao informou versao".to_owned());
            }
            Ok(output) => {
                return Err(format!(
                    "o comando de versao terminou com {}",
                    output.status
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::ExecutableFileBusy && remaining > 0 => {
                thread::sleep(EXEC_BUSY_BACKOFF);
            }
            Err(error) => return Err(format!("falha ao executar a ferramenta: {error}")),
        }
    }
    Err("a ferramenta seguiu ocupada para execucao (ETXTBSY)".to_owned())
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
                "make",
                "bear",
                "git",
                "clangd",
                "clang",
                "clangxx",
                "gcc",
                "gxx",
                "gdb",
                "lldb",
                "lldb-dap",
                // Cross-compiladores e adaptador de embarcado, 2026-09-03
                // (roadmaps/35 etapas 22 e 23).
                "arm-none-eabi-gcc",
                "arm-none-eabi-gxx",
                // As toolchains por alvo e as meta-ferramentas, 2026-09-12
                // (integracoes/39).
                "riscv-none-elf-gcc",
                "riscv-none-elf-gxx",
                "riscv32-esp-elf-gcc",
                "riscv32-esp-elf-gxx",
                "xtensa-esp-elf-gcc",
                "xtensa-esp-elf-gxx",
                "aarch64-linux-gnu-gcc",
                "aarch64-linux-gnu-gxx",
                "arm-linux-gnueabihf-gcc",
                "arm-linux-gnueabihf-gxx",
                "riscv64-linux-gnu-gcc",
                "riscv64-linux-gnu-gxx",
                "gdb-multiarch",
                "arm-none-eabi-gdb",
                "xtensa-esp-elf-gdb",
                "riscv32-esp-elf-gdb",
                "west",
                "pio",
                "picotool",
                "dfu-util",
                "openocd",
                "espup",
                // Python, 2026-09-12 (bloco B do roadmaps/41).
                "python3",
                "uv",
                "pipx",
                "ruff",
                "basedpyright",
                "pytest",
                "mypy",
                "poetry",
                "mpremote",
                "probe-rs",
                // Containers como dominio nativo, 2026-09-12 (roadmaps/28 §0).
                "docker",
                "podman",
                "podman-compose",
                // Monitores seriais, 2026-09-12 (E3 do integracoes/38 §6).
                "tio",
                "picocom",
                "minicom",
                "espflash",
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
        let _guard = exec_lock();
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
        let _guard = exec_lock();
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
    ///
    /// O lock estreita a janela ENTRE ESTES testes; ele nao a fecha, porque
    /// qualquer outro teste da suite que forke no instante errado herda o
    /// descritor e produz o mesmo `ETXTBSY` (`PONTO_ATUAL` §0.2h). Quem fecha a
    /// corrida e' o retry de [`EXEC_BUSY_ATTEMPTS`] no `probe_version`; o
    /// escopo deste lock NAO deve crescer para tapar o buraco — isso esconderia
    /// o defeito em vez de corrigi-lo.
    #[cfg(unix)]
    static EXEC_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Toma o [`EXEC_LOCK`] ignorando envenenamento.
    ///
    /// Sem isto, o primeiro teste que falha segurando o mutex derruba TODOS os
    /// seguintes com `PoisonError` — a cascata que fez uma falha virar duas em
    /// 2026-07-16 e escondeu qual teste era o real. O dado protegido e' `()`:
    /// nao ha estado corrompido a preservar.
    #[cfg(unix)]
    fn exec_lock() -> std::sync::MutexGuard<'static, ()> {
        EXEC_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[cfg(unix)]
    #[test]
    fn detected_tool_reports_path_and_version() {
        let _guard = exec_lock();
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
        let _guard = exec_lock();
        let dir = temp_bin_dir("broken");
        write_fake_tool(&dir, "cargo", "exit 3");
        let detector = ToolDetector::with_search_path(&dir);

        let info = detector.detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Failed);
        assert!(info.version.is_none());
        assert!(info.message.as_deref().unwrap().contains("Cargo"));
    }

    /// O caso do §0.2h, tornado DETERMINISTICO.
    ///
    /// A flake original dependia de outro teste forkar no microssegundo errado.
    /// Aqui a corrida e' reproduzida de proposito: um escritor segura o
    /// descritor de escrita do script enquanto a deteccao tenta executa-lo, que
    /// e' exatamente o estado que o `execve` recusa com `ETXTBSY`. Sem o retry
    /// do `probe_version` este teste reprova SEMPRE (status `Failed`) — foi
    /// assim que ele foi verificado.
    #[cfg(unix)]
    #[test]
    fn detection_survives_a_writer_still_holding_the_script() {
        use std::time::Duration;

        let _guard = exec_lock();
        let dir = temp_bin_dir("etxtbsy");
        write_fake_tool(&dir, "cargo", "echo 'cargo 1.99.0 (fake)'");

        // Segura o descritor de ESCRITA por menos que a janela total do retry
        // (20 x 10 ms), e solta numa thread — o `execve` so passa a valer
        // depois disso.
        let holder = fs::OpenOptions::new()
            .write(true)
            .open(dir.join("cargo"))
            .unwrap();
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            drop(holder);
        });

        let info = ToolDetector::with_search_path(&dir).detect(&FAKE_SPEC);
        releaser.join().unwrap();

        assert_eq!(info.status, ToolStatus::Detected);
        assert_eq!(info.version.as_deref(), Some("cargo 1.99.0 (fake)"));
    }

    /// O `EXEC_LOCK` envenenado nao pode derrubar os testes seguintes.
    ///
    /// Era a segunda metade do §0.2h: a primeira falha morria segurando o
    /// mutex, e a proxima virava `PoisonError` — uma falha real virava duas, e
    /// a cascata escondia qual era a verdadeira.
    #[cfg(unix)]
    #[test]
    fn poisoned_exec_lock_does_not_cascade() {
        let poisoner = std::thread::spawn(|| {
            let _guard = exec_lock();
            panic!("simula um teste que falha segurando o lock");
        });
        assert!(poisoner.join().is_err());
        assert!(EXEC_LOCK.is_poisoned());

        // O proximo a chegar continua trabalhando: o dado protegido e' `()`.
        let _guard = exec_lock();
        let dir = temp_bin_dir("poisoned-lock");
        write_fake_tool(&dir, "cargo", "echo 'cargo 1.99.0 (fake)'");

        let info = ToolDetector::with_search_path(&dir).detect(&FAKE_SPEC);

        assert_eq!(info.status, ToolStatus::Detected);
    }

    #[test]
    fn detect_all_returns_one_entry_per_known_tool() {
        let dir = temp_bin_dir("all");
        let detector = ToolDetector::with_search_path(&dir);

        let tools = detector.detect_all();

        assert_eq!(tools.len(), KNOWN_TOOLS.len());
        assert!(tools.iter().all(|tool| tool.status == ToolStatus::Missing));
    }

    /// Os diretorios de toolchain alem do PATH: o xpm, o ESP-IDF, o
    /// cargo/pipx do usuario — so' os que EXISTEM, na ordem, sem repetir; e
    /// as duas variaveis de ambiente (lidas por quem chama) mandam no lugar
    /// do padrao. (A pasta da IDE e' lida a parte, a cada busca: teste
    /// `the_install_root_is_read_on_every_lookup`.)
    #[test]
    fn extra_search_dirs_finds_the_toolchain_homes_that_exist() {
        let home = temp_bin_dir("extra-dirs-home");
        let mk = |rel: &str| std::fs::create_dir_all(home.join(rel)).unwrap();
        mk(".local/xPacks/@xpack-dev-tools/riscv-none-elf-gcc/15.2.0-1/.content/bin");
        mk(".espressif/tools/xtensa-esp-elf/esp-16.1.0_20260609/xtensa-esp-elf/bin");
        mk(".cargo/bin");
        // Sem `bin`: nao entra.
        mk(".local/xPacks/@xpack-dev-tools/openocd/0.12.0-7/.content/share");
        let dirs = super::extra_search_dirs(&home, None, None);
        let rel: Vec<String> = dirs
            .iter()
            .filter(|d| d.starts_with(&home))
            .map(|d| d.strip_prefix(&home).unwrap().display().to_string())
            .collect();
        assert_eq!(
            rel,
            vec![
                ".local/xPacks/@xpack-dev-tools/riscv-none-elf-gcc/15.2.0-1/.content/bin",
                ".espressif/tools/xtensa-esp-elf/esp-16.1.0_20260609/xtensa-esp-elf/bin",
                ".cargo/bin",
            ]
        );
        // XPACKS_STORE_FOLDER e IDF_TOOLS_PATH sobrescrevem o padrao.
        let outro = home.join("outro-store");
        std::fs::create_dir_all(outro.join("@xpack-dev-tools/gcc/1.0/.content/bin")).unwrap();
        let idf = home.join("idf-tools");
        std::fs::create_dir_all(idf.join("riscv32-esp-elf/v1/riscv32-esp-elf/bin")).unwrap();
        let dirs = super::extra_search_dirs(&home, Some(&outro), Some(&idf));
        assert!(dirs.contains(&outro.join("@xpack-dev-tools/gcc/1.0/.content/bin")));
        assert!(dirs.contains(&idf.join("riscv32-esp-elf/v1/riscv32-esp-elf/bin")));
        assert!(
            !dirs
                .iter()
                .any(|d| d.starts_with(home.join(".local/xPacks"))),
            "com a variavel, o padrao do xpm nao e' lido"
        );
    }

    /// A pasta da IDE nao e' congelada na construcao: uma toolchain que o
    /// provedor desempacotou depois de o detector existir aparece na busca
    /// seguinte — sem isso, "Instalar" exigiria reiniciar a IDE para valer.
    /// E ela vem DEPOIS do PATH: o que o usuario escolheu vence.
    #[test]
    #[cfg(unix)]
    fn the_install_root_is_read_on_every_lookup() {
        use std::os::unix::fs::PermissionsExt;

        // Escreve um executavel e o roda: sem este lock corre com os
        // outros iguais e o `exec` volta ETXTBSY (ver lib.rs).
        let _serial = crate::serializar_executaveis();
        let base = temp_bin_dir("install-root");
        let path_dir = base.join("path");
        let raiz = base.join("toolchains");
        std::fs::create_dir_all(&path_dir).unwrap();
        let detector = ToolDetector::with_search_path(&path_dir).with_install_root(&raiz);
        assert_eq!(detector.install_root(), Some(raiz.as_path()));
        assert_eq!(detector.find_in_path("arm-none-eabi-gcc"), None);

        // A toolchain nasce DEPOIS: a mesma instancia a encontra.
        let bin = raiz.join("arm-gnu-arm-none-eabi/15.2.rel1/bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("arm-none-eabi-gcc"), "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(
            bin.join("arm-none-eabi-gcc"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_eq!(
            detector.find_in_path("arm-none-eabi-gcc"),
            Some(bin.join("arm-none-eabi-gcc"))
        );

        // O PATH vence a pasta da IDE.
        std::fs::write(path_dir.join("arm-none-eabi-gcc"), "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(
            path_dir.join("arm-none-eabi-gcc"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_eq!(
            detector.find_in_path("arm-none-eabi-gcc"),
            Some(path_dir.join("arm-none-eabi-gcc"))
        );
    }
}
