//! A implementacao Unix do [`super`]: o comportamento de antes do porte
//! (2026-10-09), movido dos dominios sem mudar.

use std::{
    ffi::{OsStr, OsString},
    fs::{self, DirBuilder, File, OpenOptions, Permissions},
    io,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
        process::CommandExt,
    },
    path::{Component, Path, PathBuf},
    process::{Child, Command},
};

use rustix::{
    fs::{CWD, Dir, Mode, OFlags, RenameFlags, openat, renameat_with},
    io::Errno,
    process::{Pid, Signal, kill_process_group},
};

use super::NoFollow;

pub(super) type GroupId = Pid;

pub(super) const INHERITED_ENV: &[&str] = &[
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "TZ",
    "TMPDIR",
    "XDG_RUNTIME_DIR",
];

// O unico uso permitido do `canonicalize` do std (clippy.toml).
#[allow(clippy::disallowed_methods)]
pub(super) fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(path)
}

pub(super) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `$<xdg>` nao vazio, senao `$HOME/<fallback>`: o que o `settings.rs` fazia.
fn xdg_or_home(xdg: &str, fallback: &[&str]) -> PathBuf {
    if let Some(dir) = std::env::var_os(xdg)
        && !dir.is_empty()
    {
        return PathBuf::from(dir);
    }
    let mut path = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    path.extend(fallback);
    path
}

pub(super) fn config_home() -> PathBuf {
    xdg_or_home("XDG_CONFIG_HOME", &[".config"])
}

pub(super) fn state_home() -> PathBuf {
    xdg_or_home("XDG_STATE_HOME", &[".local", "state"])
}

pub(super) fn executable_names(binary: &str) -> Vec<String> {
    vec![binary.to_owned()]
}

pub(super) fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_absent| "/bin/bash".to_owned())
}

pub(super) fn program_label(program: &Path) -> String {
    program
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

pub(super) fn shell_command(command: &str) -> (String, Vec<String>) {
    ("sh".to_owned(), vec!["-lc".to_owned(), command.to_owned()])
}

#[allow(clippy::unnecessary_wraps)] // a assinatura e' a do Windows, que pode faltar o Git
pub(super) fn script_command(
    interpreter: &str,
    script: &Path,
) -> Result<(String, Vec<String>), String> {
    Ok((
        interpreter.to_owned(),
        vec!["--".to_owned(), script.display().to_string()],
    ))
}

pub(super) fn portable_relative(path: &Path) -> String {
    path.display().to_string()
}

pub(super) fn from_portable(text: &str) -> PathBuf {
    PathBuf::from(text)
}

pub(super) fn open_nofollow(path: &Path) -> Result<File, NoFollow> {
    let mut current = File::open("/").map_err(NoFollow::Io)?;
    for component in path.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(name) => current = open_child(&current, name)?,
            Component::Prefix(_) | Component::ParentDir => return Err(NoFollow::InvalidPath),
        }
    }
    Ok(current)
}

pub(super) fn open_child_nofollow(
    parent: &File,
    _parent_path: &Path,
    name: &OsStr,
) -> Result<File, NoFollow> {
    open_child(parent, name)
}

fn open_child(parent: &File, name: &OsStr) -> Result<File, NoFollow> {
    openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(|error| {
        if error == Errno::LOOP {
            NoFollow::Link
        } else {
            NoFollow::Io(error.into())
        }
    })
}

pub(super) fn child_names(dir: &File, _dir_path: &Path) -> io::Result<Vec<OsString>> {
    let mut names = Vec::new();
    for entry in Dir::read_from(dir)? {
        let entry = entry?;
        let name = OsStr::from_bytes(entry.file_name().to_bytes());
        if name != OsStr::new(".") && name != OsStr::new("..") {
            names.push(name.to_os_string());
        }
    }
    Ok(names)
}

pub(super) fn rename_noreplace(from: &Path, to: &Path) -> io::Result<()> {
    renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE).map_err(io::Error::from)
}

pub(super) fn owner_only_file(options: &mut OpenOptions) {
    options.mode(0o600);
}

pub(super) fn owner_only_dir_builder() -> DirBuilder {
    let mut builder = DirBuilder::new();
    builder.mode(0o700);
    builder
}

pub(super) fn restrict_dir_to_owner(path: &Path) -> io::Result<()> {
    fs::set_permissions(path, Permissions::from_mode(0o700))
}

pub(super) fn owner_writable(mut permissions: Permissions) -> Permissions {
    permissions.set_mode(permissions.mode() | 0o700);
    permissions
}

pub(super) fn own_group(command: &mut Command) {
    command.process_group(0);
}

pub(super) fn group_of(child: &Child) -> Option<GroupId> {
    i32::try_from(child.id()).ok().and_then(Pid::from_raw)
}

// Por referencia, e nao por valor: a assinatura e' a do Windows, onde o grupo
// e' um `Arc` do Job Object (W5), e o `mod.rs` que a chama e' um so'.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn terminate_group(group: &GroupId) -> bool {
    // ESRCH: o grupo ja' acabou, que e' o resultado esperado.
    kill_process_group(*group, Signal::TERM).ok();
    true
}

// Por referencia pelo mesmo motivo do `terminate_group`.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(super) fn kill_group(group: &GroupId) {
    kill_process_group(*group, Signal::KILL).ok();
}

// Por referencia, e nao `const`, pela assinatura do Windows, onde soltar o
// grupo e' uma chamada ao sistema; o `mod.rs` que a chama e' um so'.
#[allow(clippy::trivially_copy_pass_by_ref, clippy::missing_const_for_fn)]
pub(super) fn release_group(_group: &GroupId) {}
