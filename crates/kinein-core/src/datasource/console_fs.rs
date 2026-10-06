//! Descriptor-relative console creation and inspection; no symlink traversal.
use rustix::fs::{
    AtFlags, Mode, OFlags, RenameFlags, mkdirat, open, openat, renameat_with, unlinkat,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs::File, io::Write, path::Path};

pub(super) fn directory(root: &Path, versioned: bool, create: bool) -> Result<File, String> {
    let mut current = File::from(
        open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| "O projeto não está disponível para o console.")?,
    );
    let parts: &[&str] = if versioned {
        &[".kinein", "consoles", "v1"]
    } else {
        &[".kinein", "consoles"]
    };
    for &part in parts {
        if create {
            match mkdirat(&current, part, Mode::RUSR | Mode::WUSR | Mode::XUSR) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(_) => return Err("Não foi possível criar a pasta dos consoles.".into()),
            }
        }
        current = File::from(
            openat(
                &current,
                part,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| "A pasta dos consoles precisa ser um diretório real, sem links.")?,
        );
    }
    Ok(current)
}

pub(super) fn inspect(directory: &File, name: &str) -> Result<bool, String> {
    let descriptor = match openat(
        directory,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(descriptor) => descriptor,
        Err(rustix::io::Errno::NOENT) => return Ok(false),
        Err(_) => return Err("O console não pode ser aberto; links não são permitidos.".into()),
    };
    let file = File::from(descriptor);
    if !file
        .metadata()
        .map_err(|_| "Não foi possível conferir o console.")?
        .is_file()
    {
        return Err("O console precisa ser um arquivo regular, sem links.".into());
    }
    Ok(true)
}

pub(super) fn create(directory: &File, name: &str, header: &str) -> Result<bool, String> {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let sequence = SERIAL.fetch_add(1, Ordering::Relaxed);
    let temporary = format!(".console-{}-{sequence}.tmp", std::process::id());
    let descriptor = openat(
        directory,
        temporary.as_str(),
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|_| "Não foi possível preparar o console.")?;
    let written = File::from(descriptor).write_all(header.as_bytes());
    let outcome = if written.is_err() {
        Err("Não foi possível gravar o cabeçalho do console.".into())
    } else {
        match renameat_with(
            directory,
            temporary.as_str(),
            directory,
            name,
            RenameFlags::NOREPLACE,
        ) {
            Ok(()) => Ok(true),
            Err(rustix::io::Errno::EXIST) => match inspect(directory, name) {
                Ok(true) => Ok(false),
                Ok(false) => Err("O console mudou durante a criação; tente novamente.".into()),
                Err(error) => Err(error),
            },
            Err(_) => Err("Não foi possível publicar o console.".into()),
        }
    };
    match unlinkat(directory, temporary.as_str(), AtFlags::empty()) {
        Ok(()) | Err(rustix::io::Errno::NOENT) => outcome,
        Err(_) => Err("Não foi possível limpar o arquivo temporário do console.".into()),
    }
}
