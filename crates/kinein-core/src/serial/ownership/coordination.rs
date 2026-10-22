//! Launch reservation in the core; execution reservation in the tool itself.
use std::{
    ffi::OsString,
    fmt::Write as _,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use kinein_protocol::JsonRpcError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{Owner, Resource, occupied, unavailable};

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Deserialize)]
struct Record {
    owner: Owner,
    token: String,
    #[serde(default)]
    completed: bool,
}

#[derive(Debug)]
pub(super) struct Coordination {
    pending: File,
    _global: Option<File>,
    _launch: Option<File>,
    automatic: bool,
    global_path: PathBuf,
    execution_path: PathBuf,
    pending_path: PathBuf,
    token: String,
}

impl Coordination {
    pub(super) fn acquire(
        directory: &Path,
        key: Option<&str>,
        device: Resource<'_>,
        owner: &Owner,
    ) -> Result<Self, JsonRpcError> {
        prepare(directory).map_err(|error| unavailable(device, &error))?;
        // Parent launch gate stays held while the child takes its execution lock.
        // Automatic selection never holds the same exclusive lock as its child.
        let automatic = key.is_none();
        let launch_path = directory.join("launch.lock");
        let launch = crate::platform::serial_lock_file(&launch_path)
            .map_err(|error| unavailable(device, &error))?;
        if !lock(&launch, !automatic, &launch).map_err(|error| unavailable(device, &error))? {
            let record = read_record(&launch).filter(|record| !automatic && !record.completed);
            return Err(occupied(
                device,
                record.as_ref().map(|record| &record.owner),
            ));
        }
        let global_path = directory.join("global.lock");
        let global = crate::platform::serial_lock_file(&global_path)
            .map_err(|error| unavailable(device, &error))?;
        if !lock(&global, !automatic, &launch).map_err(|error| unavailable(device, &error))? {
            // A failed shared lock identifies an automatic tool. An exclusive
            // probe can conflict with several device owners: do not invent one.
            let record = read_record(&launch).filter(|record| !automatic && record.owner.automatic);
            return Err(occupied(
                device,
                record.as_ref().map(|record| &record.owner),
            ));
        }
        let (mut pending, pending_path, execution_path, launch_guard) = if let Some(key) = key {
            let mut digest = String::with_capacity(64);
            for byte in Sha256::digest(key.as_bytes()) {
                let _ = write!(&mut digest, "{byte:02x}");
            }
            let pending_path = directory.join(format!("{digest}.pending"));
            let execution_path = directory.join(format!("{digest}.lock"));
            let pending = crate::platform::serial_lock_file(&pending_path)
                .map_err(|error| unavailable(device, &error))?;
            if !lock(&pending, false, &pending).map_err(|error| unavailable(device, &error))? {
                let record = read_record(&pending).filter(|record| !record.completed);
                return Err(occupied(
                    device,
                    record.as_ref().map(|record| &record.owner),
                ));
            }
            let execution = crate::platform::serial_lock_file(&execution_path)
                .map_err(|error| unavailable(device, &error))?;
            if !lock(&execution, false, &pending).map_err(|error| unavailable(device, &error))? {
                return Err(occupied(
                    device,
                    read_record(&pending).as_ref().map(|record| &record.owner),
                ));
            }
            drop(execution);
            (pending, pending_path, execution_path, Some(launch))
        } else {
            (launch, launch_path, global_path.clone(), None)
        };
        let token = format!(
            "{}:{}:{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)
        );
        write_record(
            &mut pending,
            &Record {
                owner: owner.clone(),
                token: token.clone(),
                completed: false,
            },
        )
        .map_err(|error| unavailable(device, &error))?;
        let global_guard = if automatic {
            drop(global);
            None
        } else {
            Some(global)
        };
        Ok(Self {
            pending,
            _global: global_guard,
            _launch: launch_guard,
            automatic,
            global_path,
            execution_path,
            pending_path,
            token,
        })
    }

    pub(super) fn invocation(
        &self,
        program: &Path,
        args: &[String],
    ) -> io::Result<(PathBuf, Vec<String>)> {
        let mut wrapped = vec![
            "--serial-tool".to_owned(),
            if self.automatic {
                "automatic"
            } else {
                "device"
            }
            .to_owned(),
        ];
        for path in [&self.global_path, &self.execution_path, &self.pending_path] {
            wrapped.push(
                path.to_str()
                    .ok_or_else(|| io::Error::other("caminho de coordenação não é UTF-8"))?
                    .to_owned(),
            );
        }
        wrapped.extend([
            self.token.clone(),
            "--".to_owned(),
            program.display().to_string(),
        ]);
        wrapped.extend_from_slice(args);
        Ok((crate::platform::self_executable()?, wrapped))
    }
}

impl Drop for Coordination {
    fn drop(&mut self) {
        // Never unlink: another process may have this inode open already.
        // Keep identity while descendants may still own the execution lease.
        if let Some(mut record) = read_record(&self.pending)
            && record.token == self.token
        {
            record.completed = true;
            drop(write_record(&mut self.pending, &record));
        }
    }
}

fn prepare(directory: &Path) -> io::Result<()> {
    crate::platform::owner_only_dir_builder()
        .recursive(true)
        .create(directory)?;
    if !std::fs::symlink_metadata(directory)?.is_dir() {
        return Err(io::Error::other(
            "a pasta de coordenação não é um diretório regular",
        ));
    }
    crate::platform::restrict_dir_to_owner(directory)
}

fn lock(file: &File, shared: bool, metadata: &File) -> io::Result<bool> {
    let deadline = Instant::now() + Duration::from_millis(50);
    loop {
        if crate::platform::try_lock_file(file, shared)? {
            return Ok(true);
        }
        if read_record(metadata).is_some_and(|record| !record.completed)
            || Instant::now() >= deadline
        {
            return Ok(false);
        }
        // A concurrent fork can briefly retain a CLOEXEC descriptor after its
        // parent's drop. Do not turn that scheduling window into a false owner.
        thread::sleep(Duration::from_millis(1));
    }
}

fn read_record(mut file: &File) -> Option<Record> {
    let mut body = Vec::new();
    file.seek(SeekFrom::Start(0)).ok()?;
    file.take(16 * 1024).read_to_end(&mut body).ok()?;
    serde_json::from_slice(&body).ok()
}

fn write_record(file: &mut File, record: &Record) -> io::Result<()> {
    let body = serde_json::to_vec(record)?;
    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    file.write_all(&body)?;
    file.flush()
}

pub(super) fn run(arguments: &[OsString]) -> io::Result<()> {
    if arguments.len() < 7
        || arguments[5] != "--"
        || (arguments[0] != "automatic" && arguments[0] != "device")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "lançamento serial interno inválido",
        ));
    }
    let automatic = arguments[0] == "automatic";
    let global = crate::platform::serial_lock_file(Path::new(&arguments[1]))?;
    if !crate::platform::try_lock_file(&global, !automatic)? {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "o canal serial já está reservado por outra operação",
        ));
    }
    let mut files = vec![global];
    if !automatic {
        let execution = crate::platform::serial_lock_file(Path::new(&arguments[2]))?;
        if !crate::platform::try_lock_file(&execution, false)? {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "o canal serial já está reservado por outra operação",
            ));
        }
        files.push(execution);
    }
    let mut pending = crate::platform::serial_lock_file(Path::new(&arguments[3]))?;
    let mut record = read_record(&pending)
        .ok_or_else(|| io::Error::other("reserva de lançamento indisponível"))?;
    if record.completed
        || arguments[4] != record.token.as_str()
        || record.owner.automatic != automatic
        || !crate::platform::is_parent_process(record.owner.core_pid)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "a reserva de lançamento expirou; a ferramenta não foi iniciada",
        ));
    }
    record.owner.process_id = Some(std::process::id());
    write_record(&mut pending, &record)?;
    drop(pending);
    crate::platform::exec_keep_files(files, &arguments[6], &arguments[7..])
}
