//! O executavel falso dos testes no Windows (DocsPublic/roadmaps/60 §3.3, W2b).
//!
//! No Unix, o teste grava um script com shebang e o kernel o executa. O
//! `CreateProcess` do Windows so' abre executavel PE, entao aqui o script e'
//! gravado como esta' e ganha, ao lado, um `<nome>.exe` de passagem, que roda o
//! script com o interpretador do shebang. O script e' o mesmo nos dois
//! sistemas. O `.exe` e' achado como a ferramenta de verdade seria: pelo
//! `PATHEXT` do `find_in_path`, e pelo `.exe` que o `Command` acrescenta a um
//! caminho sem extensao.
//!
//! O `sh` e o `bash` sao os do Git for Windows, achados pelo `git --exec-path`.
//! Nunca o `bash` do `PATH`: no Windows ele e' o lancador do WSL. O lancador do
//! Git (`<git>\bin\sh.exe`) poe o `/usr/bin` do MSYS no `PATH`, e o script acha
//! `cat`, `sort` e `env` como no Linux.
//!
//! O programa de passagem e' compilado pelo `rustc` uma vez por processo de
//! teste, numa pasta temporaria so' dele.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

/// O programa de passagem. As constantes `SH`, `BASH` e `PYTHON` entram na
/// frente dele, com os caminhos desta maquina.
const PROGRAM: &str = r##"
use std::{env, fs, process::{exit, Command}};

fn fail(message: &str) -> ! {
    eprintln!("kinein-fake-exe: {message}");
    exit(127)
}

fn main() {
    let me = env::current_exe().unwrap_or_else(|error| fail(&format!("current_exe: {error}")));
    let script = me.with_extension("");
    let text = fs::read_to_string(&script)
        .unwrap_or_else(|error| fail(&format!("{}: {error}", script.display())));
    let shebang = text.lines().next().and_then(|line| line.strip_prefix("#!")).unwrap_or("");
    let mut words = shebang.split_whitespace();
    let mut name = words.next().unwrap_or("sh").rsplit('/').next().unwrap_or("sh");
    if name == "env" {
        name = words.next().unwrap_or("sh");
    }
    let interpreter = match name {
        "sh" => SH,
        "bash" => BASH,
        "python" | "python3" => PYTHON,
        other => fail(&format!("interpretador sem equivalente no Windows: {other}")),
    };
    let status = Command::new(interpreter)
        .args(words)
        .arg(&script)
        .args(env::args_os().skip(1))
        .status()
        .unwrap_or_else(|error| fail(&format!("{interpreter}: {error}")));
    exit(status.code().unwrap_or(1));
}
"##;

/// A raiz do Git for Windows: o `--exec-path` e' `<raiz>\mingw64\libexec\git-core`.
fn git_root() -> PathBuf {
    let output = Command::new("git")
        .arg("--exec-path")
        .output()
        .expect("os testes com executavel falso pedem o Git for Windows no PATH");
    let exec_path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    exec_path
        .ancestors()
        .nth(3)
        .expect("o --exec-path do Git tem tres niveis acima")
        .to_path_buf()
}

/// O `python.exe` do `PATH`, sem o atalho da Microsoft Store (`WindowsApps`).
fn python() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .filter(|dir| !dir.to_string_lossy().contains("WindowsApps"))
                .map(|dir| dir.join("python.exe"))
                .find(|candidate| candidate.is_file())
        })
        .unwrap_or_else(|| PathBuf::from("python"))
}

fn rust_string(path: &Path) -> String {
    format!("{:?}", path.display().to_string())
}

/// Compila o programa de passagem uma vez por processo.
fn program() -> &'static Path {
    static PROGRAM_EXE: OnceLock<PathBuf> = OnceLock::new();
    PROGRAM_EXE.get_or_init(|| {
        let git = git_root();
        let sh = git.join("bin").join("sh.exe");
        let bash = git.join("bin").join("bash.exe");
        assert!(
            sh.is_file(),
            "sem o sh do Git for Windows em {}",
            sh.display()
        );
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("fake-exe-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("cria a pasta do programa de passagem");
        let source = dir.join("main.rs");
        let exe = dir.join("kinein-fake-exe.exe");
        std::fs::write(
            &source,
            format!(
                "const SH: &str = {};\nconst BASH: &str = {};\nconst PYTHON: &str = {};\n{PROGRAM}",
                rust_string(&sh),
                rust_string(&bash),
                rust_string(&python()),
            ),
        )
        .expect("grava o fonte do programa de passagem");
        let output = Command::new("rustc")
            .args(["--edition", "2021", "-C", "opt-level=1", "-D", "warnings"])
            .arg("-o")
            .arg(&exe)
            .arg(&source)
            .output()
            .expect("os testes com executavel falso pedem o rustc no PATH");
        assert!(
            output.status.success(),
            "o rustc nao compilou o programa de passagem: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        exe
    })
}

/// Poe o `.exe` de passagem ao lado do script `path` e devolve o caminho dele.
// `pub` seria reprovado pelo `unreachable_pub` do workspace; o conflito com o
// clippy esta' resolvido assim no `platform/mod.rs`.
#[allow(clippy::redundant_pub_crate)]
pub(super) fn beside(path: &Path) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".exe");
    let exe = PathBuf::from(name);
    drop(std::fs::remove_file(&exe));
    if std::fs::hard_link(program(), &exe).is_err() {
        std::fs::copy(program(), &exe).expect("copia o programa de passagem");
    }
    exe
}
