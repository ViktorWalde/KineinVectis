//! A implementacao Windows do [`super`]: o equivalente sem `unsafe`, so' com a
//! biblioteca padrao (60 §3.2, W1). O limite de cada funcao esta' dito no
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
    process::{Child, Command, Stdio},
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

pub(super) type GroupId = u32;

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
    if fs::symlink_metadata(from)?.is_dir() {
        match fs::symlink_metadata(to) {
            Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        return fs::rename(from, to);
    }
    // O `hard_link` falha com `AlreadyExists` se `to` existe: e' ele que
    // garante o "sem sobrescrever" para arquivo.
    fs::hard_link(from, to)?;
    fs::remove_file(from)
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

pub(super) fn own_group(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

// Sempre `Some` aqui; a assinatura e' a do Unix, onde o PID pode nao caber.
#[allow(clippy::unnecessary_wraps)]
pub(super) fn group_of(child: &Child) -> Option<GroupId> {
    Some(child.id())
}

// Poderia ser `const` so' aqui; no Unix ela manda um sinal, e o `mod.rs` que
// a chama e' o mesmo para os dois sistemas.
#[allow(clippy::missing_const_for_fn)]
pub(super) fn terminate_group(_group: GroupId) -> bool {
    false
}

pub(super) fn kill_group(group: GroupId) {
    let taskkill = std::env::var_os("SystemRoot").map_or_else(
        || PathBuf::from("taskkill.exe"),
        |root| Path::new(&root).join("System32").join("taskkill.exe"),
    );
    // Enquanto o `Child` do dono existe, ele segura o handle do processo e o
    // PID nao e' reaproveitado: o `/PID` acerta o processo certo, vivo ou nao.
    drop(
        Command::new(taskkill)
            .args(["/T", "/F", "/PID"])
            .arg(group.to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status(),
    );
}
