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
    path::{Component, Path},
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

pub(super) fn terminate_group(group: GroupId) -> bool {
    // ESRCH: o grupo ja' acabou, que e' o resultado esperado.
    kill_process_group(group, Signal::TERM).ok();
    true
}

pub(super) fn kill_group(group: GroupId) {
    kill_process_group(group, Signal::KILL).ok();
}
