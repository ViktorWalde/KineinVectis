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

pub(super) fn default_shell() -> String {
    let in_path = |name: &str| {
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join(name))
                .find(|candidate| candidate.is_file())
        })
    };
    in_path("pwsh.exe")
        .or_else(|| in_path("powershell.exe"))
        .map(|shell| shell.display().to_string())
        .or_else(|| std::env::var("ComSpec").ok())
        .unwrap_or_else(|| "cmd.exe".to_owned())
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
}
