//! A implementacao Windows do [`super`]: a biblioteca padrao e, para o que so'
//! a API do Windows faz (Job Object, rename sem sobrescrever), o `kinein-sys`,
//! que confina o `unsafe` (60 §3.1, D7). O limite de cada funcao esta' dito no
//! `mod.rs`, ao lado da API.

use std::{
    ffi::{OsStr, OsString},
    fs::{self, DirBuilder, File, OpenOptions, Permissions},
    io,
    os::windows::{
        fs::{MetadataExt, OpenOptionsExt},
        process::CommandExt,
    },
    path::{Component, Path, PathBuf},
    process::{Child, Command},
    sync::Arc,
};

use super::NoFollow;

/// Abre o proprio ponto de reparse, sem segui-lo.
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
/// Necessario para o `CreateFileW` abrir uma pasta.
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
/// O no' e' ponto de reparse: link simbolico, juncao ou outro.
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
/// O processo filho nao abre janela de console.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// O processo filho nasce parado, ate' ser retomado.
const CREATE_SUSPENDED: u32 = 0x0000_0004;

/// O job do filho, compartilhado entre o dono e quem so' mata
/// (`owned_child::GroupKiller`).
pub(super) type GroupId = Arc<kinein_sys::Job>;

pub(super) const INHERITED_ENV: &[&str] = &[
    "PATH",
    "SystemRoot",
    "windir",
    "ComSpec",
    "PATHEXT",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
];

/// A variavel de ambiente `name`, quando definida e nao vazia.
fn var_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub(super) fn home_dir() -> Option<PathBuf> {
    var_dir("USERPROFILE").or_else(|| var_dir("HOME"))
}

pub(super) fn config_home() -> PathBuf {
    var_dir("APPDATA")
        .or_else(|| home_dir().map(|home| home.join("AppData").join("Roaming")))
        .unwrap_or_default()
}

pub(super) fn state_home() -> PathBuf {
    var_dir("LOCALAPPDATA")
        .or_else(|| home_dir().map(|home| home.join("AppData").join("Local")))
        .unwrap_or_default()
}

pub(super) fn executable_names(binary: &str) -> Vec<String> {
    if Path::new(binary).extension().is_some() {
        return vec![binary.to_owned()];
    }
    let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned());
    pathext
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| format!("{binary}{}", ext.to_ascii_lowercase()))
        .collect()
}

fn in_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
    })
}

pub(super) fn default_shell() -> String {
    in_path("pwsh.exe")
        .or_else(|| in_path("powershell.exe"))
        .map(|shell| shell.display().to_string())
        .or_else(|| std::env::var("ComSpec").ok())
        .unwrap_or_else(|| "cmd.exe".to_owned())
}

/// O PowerShell do `PATH`; sem ele, o do sistema, que todo Windows tem.
fn powershell() -> String {
    in_path("pwsh.exe")
        .or_else(|| in_path("powershell.exe"))
        .or_else(|| {
            var_dir("SystemRoot").map(|root| {
                root.join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0")
                    .join("powershell.exe")
            })
        })
        .map_or_else(
            || "powershell.exe".to_owned(),
            |shell| shell.display().to_string(),
        )
}

pub(super) fn program_label(program: &Path) -> String {
    let executable = program.extension().is_some_and(|ext| {
        let ext = format!(".{}", ext.to_string_lossy());
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_owned())
            .split(';')
            .any(|known| known.eq_ignore_ascii_case(&ext))
    });
    let name = if executable {
        program.file_stem()
    } else {
        program.file_name()
    };
    name.map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

pub(super) fn shell_command(command: &str) -> (String, Vec<String>) {
    // `$?` e' lido logo depois do comando, antes de qualquer outra instrucao.
    let script = format!(
        "$global:LASTEXITCODE = 0\n{command}\n$kineinOk = $?\n\
         if ($LASTEXITCODE) {{ exit $LASTEXITCODE }}\nif (-not $kineinOk) {{ exit 1 }}\n"
    );
    let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    (
        powershell(),
        vec![
            "-NoLogo".to_owned(),
            "-EncodedCommand".to_owned(),
            base64(&utf16),
        ],
    )
}

/// O `bash.exe` do Git for Windows, achado a partir do `git.exe` do `PATH`
/// (`<git>\cmd\git.exe` ou `<git>\bin\git.exe`): a raiz do Git e' a pasta que
/// tem `bin\bash.exe` e `usr\bin`.
fn git_bash() -> Option<PathBuf> {
    let git = in_path("git.exe")?;
    git.ancestors().skip(1).find_map(|dir| {
        let bash = dir.join("bin").join("bash.exe");
        (bash.is_file() && dir.join("usr").join("bin").is_dir()).then_some(bash)
    })
}

pub(super) fn script_command(
    interpreter: &str,
    script: &Path,
) -> Result<(String, Vec<String>), String> {
    let script = script.display().to_string();
    match interpreter {
        "bash" => git_bash()
            .map(|bash| (bash.display().to_string(), vec!["--".to_owned(), script]))
            .ok_or_else(|| {
                "o script precisa do bash do Git for Windows, e o git.exe nao esta' no PATH"
                    .to_owned()
            }),
        "powershell" => Ok((
            powershell(),
            vec!["-NoLogo".to_owned(), "-File".to_owned(), script],
        )),
        "cmd" => Ok((
            std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_owned()),
            vec!["/D".to_owned(), "/C".to_owned(), script],
        )),
        other => Err(format!("interpretador sem equivalente no Windows: {other}")),
    }
}

/// Base64 padrao (RFC 4648, com `=`), o que o `-EncodedCommand` le.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let triple = chunk.iter().enumerate().fold(0_u32, |acc, (index, byte)| {
            acc | u32::from(*byte) << (16 - 8 * index)
        });
        for position in 0..4 {
            if position <= chunk.len() {
                let index = (triple >> (18 - 6 * position)) & 0x3F;
                text.push(char::from(ALPHABET[index as usize]));
            } else {
                text.push('=');
            }
        }
    }
    text
}

pub(super) fn portable_relative(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

pub(super) fn from_portable(text: &str) -> PathBuf {
    PathBuf::from(text.replace('/', "\\"))
}

/// O limite classico de caminho do Windows (`MAX_PATH`), contando o terminador.
const MAX_PATH: usize = 260;

// O unico uso permitido do `canonicalize` do std (clippy.toml).
#[allow(clippy::disallowed_methods)]
pub(super) fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    let verbatim = fs::canonicalize(path)?;
    Ok(without_verbatim(&verbatim).unwrap_or(verbatim))
}

/// `\\?\C:\x` vira `C:\x`, e `\\?\UNC\s\c\x` vira `\\s\c\x`, quando o caminho
/// cabe sem o prefixo. `None` quando ele precisa do prefixo, ou nao e' UTF-8.
fn without_verbatim(path: &Path) -> Option<PathBuf> {
    let text = path.to_str()?;
    let plain = if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else {
        let rest = text.strip_prefix(r"\\?\")?;
        let [letter, b':', b'\\', ..] = rest.as_bytes() else {
            return None;
        };
        if !letter.is_ascii_alphabetic() {
            return None;
        }
        rest.to_owned()
    };
    let fits = plain.len() < MAX_PATH
        && Path::new(&plain)
            .components()
            .all(|component| match component {
                Component::Normal(name) => plain_name(name),
                _ => true,
            });
    fits.then(|| PathBuf::from(plain))
}

/// Um nome que o Windows le igual com e sem o prefixo: sem ponto ou espaco no
/// fim e sem nome de dispositivo reservado (`CON`, `NUL`, `COM1`, ...).
fn plain_name(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    if name.ends_with('.') || name.ends_with(' ') {
        return false;
    }
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim_end()
        .to_ascii_uppercase();
    let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || matches!(
            stem.as_bytes(),
            [b'C', b'O', b'M', b'1'..=b'9'] | [b'L', b'P', b'T', b'1'..=b'9']
        );
    !device
}

/// Abre `path` sem seguir ponto de reparse e confere, no handle aberto, que ele
/// nao e' um. Um ponto de reparse que nao e' link (um arquivo de nuvem sem
/// copia local, por exemplo) tambem e' recusado: melhor recusar que seguir.
fn open_node(path: &Path) -> Result<File, NoFollow> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(NoFollow::Io)?;
    let attributes = file.metadata().map_err(NoFollow::Io)?.file_attributes();
    if attributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
        Ok(file)
    } else {
        Err(NoFollow::Link)
    }
}

pub(super) fn open_nofollow(path: &Path) -> Result<File, NoFollow> {
    let mut walked = PathBuf::new();
    let mut current = None;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => walked.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(name) => {
                walked.push(name);
                current = Some(open_node(&walked)?);
            }
            Component::ParentDir => return Err(NoFollow::InvalidPath),
        }
    }
    // So' a raiz do volume nao e' um no' copiavel.
    current.ok_or(NoFollow::InvalidPath)
}

pub(super) fn open_child_nofollow(
    _parent: &File,
    parent_path: &Path,
    name: &OsStr,
) -> Result<File, NoFollow> {
    open_node(&parent_path.join(name))
}

pub(super) fn child_names(_dir: &File, dir_path: &Path) -> io::Result<Vec<OsString>> {
    fs::read_dir(dir_path)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect()
}

pub(super) fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    kinein_sys::rename_noreplace(from, to)
}

// Nada a fazer no Windows (D3); `const` so' aqui, como no `terminate_group`.
#[allow(clippy::missing_const_for_fn)]
pub(super) fn owner_only_file(_options: &mut OpenOptions) {}

pub(super) fn owner_only_dir_builder() -> DirBuilder {
    DirBuilder::new()
}

pub(super) fn restrict_dir_to_owner(path: &Path) -> io::Result<()> {
    fs::metadata(path).map(drop)
}

// No Windows, `set_readonly(false)` so' tira o atributo somente-leitura; o
// lint fala do Unix, onde a mesma chamada abriria a escrita para todos.
#[allow(clippy::permissions_set_readonly_false)]
pub(super) fn owner_writable(mut permissions: Permissions) -> Permissions {
    permissions.set_readonly(false);
    permissions
}

/// O filho nasce SUSPENSO: ele entra no job antes da primeira instrucao, e
/// nenhum neto pode nascer fora dele (`group_of` o retoma).
pub(super) fn own_group(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
}

pub(super) fn group_of(child: &Child) -> Option<GroupId> {
    let job = kinein_sys::Job::new().and_then(|job| job.assign(child).map(|()| job));
    // O filho nasceu suspenso: ele e' retomado SEMPRE, com job ou sem, senao
    // nunca roda. Sem retomar, o job o mata, para a falha aparecer na hora
    // (o pipe fecha) em vez de um processo parado para sempre.
    if kinein_sys::resume_process(child.id()).is_err()
        && let Ok(job) = &job
    {
        drop(job.terminate());
    }
    job.ok().map(Arc::new)
}

// Poderia ser `const` so' aqui; no Unix ela manda um sinal, e o `mod.rs` que
// a chama e' o mesmo para os dois sistemas.
#[allow(clippy::missing_const_for_fn)]
pub(super) fn terminate_group(_group: &GroupId) -> bool {
    false
}

pub(super) fn kill_group(group: &GroupId) {
    drop(group.terminate());
}

pub(super) fn release_group(group: &GroupId) {
    // Se o sistema recusar, o job segue armado: o pior caso e' o de antes.
    drop(group.release());
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::without_verbatim;

    fn plain(text: &str) -> Option<PathBuf> {
        without_verbatim(Path::new(text))
    }

    #[test]
    fn a_drive_path_loses_the_prefix() {
        assert_eq!(
            plain(r"\\?\C:\dev\Kinein Vectis\src"),
            Some(PathBuf::from(r"C:\dev\Kinein Vectis\src"))
        );
    }

    #[test]
    fn a_network_share_loses_the_prefix() {
        assert_eq!(
            plain(r"\\?\UNC\servidor\pasta\x.rs"),
            Some(PathBuf::from(r"\\servidor\pasta\x.rs"))
        );
    }

    #[test]
    fn what_needs_the_prefix_keeps_it() {
        assert_eq!(plain(r"\\?\C:\dev\con.txt"), None, "nome reservado");
        assert_eq!(plain(r"\\?\C:\dev\COM1"), None, "nome reservado");
        assert_eq!(plain(r"\\?\C:\dev\fim."), None, "ponto no fim");
        assert_eq!(plain(r"\\?\C:\dev\fim "), None, "espaco no fim");
        assert_eq!(plain(r"\\?\Volume{1234}\x"), None, "volume sem letra");
        let long = format!(r"\\?\C:\{}", "a".repeat(300));
        assert_eq!(plain(&long), None, "passa do MAX_PATH");
    }

    #[test]
    fn a_name_that_only_looks_reserved_is_plain() {
        assert_eq!(
            plain(r"\\?\C:\dev\console\COM10\nul2"),
            Some(PathBuf::from(r"C:\dev\console\COM10\nul2"))
        );
    }

    #[test]
    fn the_canonical_form_of_a_real_folder_has_no_prefix() {
        let dir = std::env::temp_dir();
        let canonical = super::canonicalize(&dir).unwrap();
        assert!(
            !canonical.to_string_lossy().starts_with(r"\\?\"),
            "{canonical:?}"
        );
        assert!(canonical.is_absolute());
    }

    /// Uma juncao (o link de pasta que o Windows cria sem privilegio) no meio
    /// do caminho e' ponto de reparse: o `open_nofollow` recusa, e o mesmo
    /// arquivo pela pasta real abre. E' a prova, no Windows, do que os testes
    /// de symlink provam no Unix (60 §3.3, W2b).
    #[test]
    fn a_junction_in_the_path_is_refused_and_the_real_folder_opens() {
        let base = std::env::temp_dir().join(format!("kinein-juncao-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&base));
        let real = base.join("real");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("a.txt"), "x").unwrap();
        let link = base.join("juncao");
        let made = std::process::Command::new("cmd")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&link)
            .arg(&real)
            .output()
            .unwrap();
        assert!(
            made.status.success(),
            "{}",
            String::from_utf8_lossy(&made.stderr)
        );
        assert!(matches!(
            super::open_nofollow(&link.join("a.txt")),
            Err(super::NoFollow::Link)
        ));
        assert!(matches!(
            super::open_nofollow(&link),
            Err(super::NoFollow::Link)
        ));
        assert!(super::open_nofollow(&real.join("a.txt")).is_ok());
        // O `remove_dir_all` do `std` nao segue a juncao: so' ela sai.
        drop(std::fs::remove_dir_all(&base));
    }

    /// O nome que se digita: sem a extensao de executavel, em qualquer caixa;
    /// o que nao e' executavel fica com a extensao.
    #[test]
    fn the_program_label_is_what_the_user_types() {
        assert_eq!(
            super::program_label(Path::new(r"C:\Program Files\RedHat\Podman\podman.exe")),
            "podman"
        );
        assert_eq!(super::program_label(Path::new(r"C:\x\idf.CMD")), "idf");
        assert_eq!(
            super::program_label(Path::new(r"C:\x\esptool.py")),
            "esptool.py"
        );
    }

    /// Os vetores da RFC 4648 §10.
    #[test]
    fn base64_follows_the_rfc() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(super::base64(plain.as_bytes()), encoded);
        }
    }

    fn run(command: &str) -> std::process::Output {
        let (shell, args) = super::shell_command(command);
        std::process::Command::new(shell)
            .args(args)
            .output()
            .unwrap()
    }

    /// O comando chega intacto, com aspas duplas e simples, e o codigo de
    /// saida do programa nativo passa adiante.
    #[test]
    fn the_shell_command_carries_quotes_and_the_exit_code() {
        let output = run(r#"Write-Output "aspas ""duplas"" e 'simples'"; cmd /c exit 7"#);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            r#"aspas "duplas" e 'simples'"#
        );
        assert_eq!(output.status.code(), Some(7));
    }

    /// Um cmdlet que falha sai com 1; um comando que da' certo sai com 0.
    #[test]
    fn a_failing_cmdlet_exits_with_one_and_success_with_zero() {
        assert_eq!(
            run("Get-Item 'C:\\nao\\existe\\mesmo'").status.code(),
            Some(1)
        );
        assert_eq!(run("Write-Output ok").status.code(), Some(0));
    }
}
