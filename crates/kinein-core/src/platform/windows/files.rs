//! Windows filesystem handles, publication and configuration coordination.
use crate::platform::NoFollow;
use std::{
    ffi::{OsStr, OsString},
    fs::{self, DirBuilder, File, OpenOptions, Permissions},
    io,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};

const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

pub(in crate::platform) fn configuration_lock(path: &Path) -> io::Result<File> {
    use std::{
        thread,
        time::{Duration, Instant},
    };
    let deadline = Instant::now() + Duration::from_millis(50);
    let file = loop {
        let result = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path);
        match result {
            Ok(file) => break file,
            Err(error) if error.raw_os_error() == Some(32) => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "outra instância está alterando as configurações; aguarde e tente novamente",
                    ));
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => return Err(error),
        }
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::other("o arquivo de coordenação não é regular"));
    }
    Ok(file)
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

pub(in crate::platform) fn open_nofollow(path: &Path) -> Result<File, NoFollow> {
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

pub(in crate::platform) fn open_child_nofollow(
    _parent: &File,
    parent_path: &Path,
    name: &OsStr,
) -> Result<File, NoFollow> {
    open_node(&parent_path.join(name))
}

pub(in crate::platform) fn child_names(_dir: &File, dir_path: &Path) -> io::Result<Vec<OsString>> {
    fs::read_dir(dir_path)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect()
}

pub(in crate::platform) fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    kinein_sys::rename_noreplace(from, to)
}

// Nada a fazer no Windows (D3); `const` so' aqui, como no `terminate_group`.
#[allow(clippy::missing_const_for_fn)]
pub(in crate::platform) fn owner_only_file(_options: &mut OpenOptions) {}

pub(in crate::platform) fn owner_only_dir_builder() -> DirBuilder {
    DirBuilder::new()
}

pub(in crate::platform) fn restrict_dir_to_owner(path: &Path) -> io::Result<()> {
    fs::metadata(path).map(drop)
}

// No Windows, `set_readonly(false)` so' tira o atributo somente-leitura; o
// lint fala do Unix, onde a mesma chamada abriria a escrita para todos.
#[allow(clippy::permissions_set_readonly_false)]
pub(in crate::platform) fn owner_writable(mut permissions: Permissions) -> Permissions {
    permissions.set_readonly(false);
    permissions
}
