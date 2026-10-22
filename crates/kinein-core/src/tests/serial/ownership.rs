//! Private regular-file fixtures: never open a tty or act on hardware.
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcErrorCode;

use crate::serial::ownership::Registry;
mod automatic;

fn fixture(name: &str) -> (PathBuf, PathBuf) {
    let root = super::temp_dir(name);
    let port = root.join("port");
    fs::write(&port, "regular fixture, never a serial device").unwrap();
    (root, port)
}

fn registry(directory: &Path) -> Registry {
    let mut registry = Registry::default();
    registry.use_directory(directory.to_path_buf());
    registry
}

#[test]
fn same_channel_and_aliases_share_the_owner_until_drop() {
    let (root, port) = fixture("lease-aliases");
    let link = root.join("by-id");
    let hard = root.join("hard-link");
    std::os::unix::fs::symlink(&port, &link).unwrap();
    fs::hard_link(&port, &hard).unwrap();
    let registry = Registry::default();
    let lease = registry
        .acquire(
            port.to_str().unwrap(),
            "serial.identify",
            "Identificar",
            Some(&root),
        )
        .unwrap();
    for alias in [&port, &link, &hard] {
        let error = registry
            .acquire(alias.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .unwrap_err();
        assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
        let details = error.details.unwrap();
        assert_eq!(details["owner"]["operation"], "serial.identify");
        assert_eq!(details["owner"]["corePid"], std::process::id());
        assert_eq!(details["owner"]["workspace"], root.display().to_string());
        assert!(error.message.contains("Jobs"));
    }
    let other = root.join("other-port");
    fs::write(&other, "another independent channel").unwrap();
    let _independent = registry
        .acquire(other.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap();
    drop(lease);
    assert!(
        registry
            .acquire(port.to_str().unwrap(), "serial.files", "Files", None)
            .is_ok()
    );
    assert!(
        !root.join("serial-usage").exists(),
        "default registry writes no global state"
    );
}

#[test]
fn concurrent_requests_have_exactly_one_owner() {
    let (_root, port) = fixture("lease-threads");
    let registry = Arc::new(Registry::default());
    let barrier = Arc::new(Barrier::new(8));
    let mut workers = Vec::with_capacity(8);
    for _ in 0..8 {
        let registry = Arc::clone(&registry);
        let barrier = Arc::clone(&barrier);
        let port = port.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            let lease =
                registry.acquire(port.to_str().unwrap(), "serial.identify", "Identify", None);
            let accepted = lease.is_ok();
            barrier.wait();
            drop(lease);
            accepted
        }));
    }
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|accepted| *accepted)
            .count(),
        1
    );
}

#[test]
fn independent_registries_use_kernel_locks_and_ignore_unlocked_stale_metadata() {
    let (root, port) = fixture("lease-registry-locks");
    let directory = root.join("locks");
    let first = registry(&directory);
    let second = registry(&directory);
    let lease = first
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap();
    let error = second
        .acquire(port.to_str().unwrap(), "serial.files", "Files", None)
        .unwrap_err();
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert!(error.message.contains("Terminal"));
    let before: std::collections::BTreeMap<_, _> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.path(), entry.metadata().unwrap().ino())
        })
        .collect();
    drop(lease);
    let files: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        files.len(),
        before.len(),
        "lock files must never be unlinked"
    );
    for path in &files {
        assert_eq!(
            Some(&fs::metadata(path).unwrap().ino()),
            before.get(path),
            "coordination inode was replaced"
        );
    }
    for path in &files {
        fs::write(path, "stale invalid owner data").unwrap();
    }
    let _next = second
        .acquire(port.to_str().unwrap(), "serial.files", "Files", None)
        .unwrap();
    let permission = fs::metadata(&directory).unwrap();
    assert_eq!(permission.permissions().mode() & 0o777, 0o700);
    for path in files {
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn unavailable_coordination_is_refused_without_leaking_a_memory_reservation() {
    let (root, port) = fixture("lease-storage-failure");
    let directory = root.join("locks");
    fs::write(&directory, "not a directory").unwrap();
    let registry = registry(&directory);
    let error = registry
        .acquire(port.to_str().unwrap(), "serial.identify", "Identify", None)
        .unwrap_err();
    assert_eq!(error.code, JsonRpcErrorCode::InternalError);
    fs::remove_file(&directory).unwrap();
    assert!(
        registry
            .acquire(port.to_str().unwrap(), "serial.identify", "Identify", None)
            .is_ok()
    );
}

#[test]
fn coordination_rejects_a_link_without_changing_its_target() {
    let (root, port) = fixture("lease-storage-link");
    let target = root.join("untouched");
    fs::create_dir(&target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    let directory = root.join("locks");
    std::os::unix::fs::symlink(&target, &directory).unwrap();
    let registry = registry(&directory);
    assert_eq!(
        registry
            .acquire(port.to_str().unwrap(), "serial.identify", "Identify", None)
            .unwrap_err()
            .code,
        JsonRpcErrorCode::InternalError
    );
    assert_eq!(
        fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o755
    );
}

struct Worker(Child);
impl Drop for Worker {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

fn worker_command(root: &Path) -> Command {
    let output = fs::File::create(root.join("worker.log")).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "tests::serial::ownership::kernel_lock_worker",
            "--nocapture",
        ])
        .env("KINEIN_SERIAL_RESERVATION_TEST", root)
        .stdout(output.try_clone().unwrap())
        .stderr(output);
    command
}

#[test]
fn kernel_lock_worker() {
    let Some(root) = std::env::var_os("KINEIN_SERIAL_RESERVATION_TEST").map(PathBuf::from) else {
        return;
    };
    if let Ok(arguments) = std::env::var("KINEIN_SERIAL_EXEC_TEST_ARGS") {
        let arguments: Vec<String> = serde_json::from_str(&arguments).unwrap();
        let arguments: Vec<_> = arguments
            .into_iter()
            .map(std::ffi::OsString::from)
            .collect();
        let error = crate::run_serial_tool(&arguments).unwrap_err();
        fs::write(root.join("tool-result"), error.to_string()).unwrap();
        return;
    }
    let registry = registry(&root.join("locks"));
    let port = root.join("port");
    let _lease = registry
        .acquire(
            port.to_str().unwrap(),
            "serial.identify",
            "Worker",
            Some(&root),
        )
        .unwrap();
    fs::write(root.join("ready"), std::process::id().to_string()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !root.join("release").exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        root.join("release").exists(),
        "parent did not release its owned fixture worker"
    );
}

#[test]
fn another_process_is_refused_and_a_crashed_owner_is_recovered() {
    let (root, port) = fixture("lease-crash-recovery");
    let child = worker_command(&root).spawn().unwrap();
    let mut worker = Worker(child);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !root.join("ready").exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        root.join("ready").exists(),
        "owned worker did not acquire its lock"
    );
    let registry = registry(&root.join("locks"));
    let error = registry
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap_err();
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert!(error.message.contains("outra instância"));
    worker.0.kill().unwrap();
    worker.0.wait().unwrap();
    assert!(
        registry
            .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .is_ok(),
        "a dead process must not leave a persistent busy flag"
    );
}

#[test]
fn failed_terminal_spawn_releases_its_reservation() {
    let (root, port) = fixture("lease-failed-spawn");
    let registry = Registry::default();
    let lease = registry
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap();
    let (sender, _) = std::sync::mpsc::channel();
    let mut terminal = crate::terminal::TerminalManager::new(sender);
    assert!(
        terminal
            .open_command_holding(&root, "/nonexistent-kinein-fixture", &[], &[], lease)
            .is_err()
    );
    assert!(
        registry
            .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .is_ok()
    );
}

fn fake_tool(root: &Path) -> PathBuf {
    let tool = root.join("tool");
    let ready = crate::platform::program_invocation(&root.join("tool-ready").display().to_string());
    crate::write_executable(
        &tool,
        format!("#!/bin/sh\nprintf 'ready' > {ready}\nexec sleep 30\n"),
    );
    tool
}

#[test]
fn execution_lock_stays_with_the_tool_after_the_launch_reservation_drops() {
    let (root, port) = fixture("lease-tool-lifetime");
    let registry = registry(&root.join("locks"));
    let lease = registry
        .acquire(port.to_str().unwrap(), "serial.identify", "Identify", None)
        .unwrap();
    let (_, arguments) = lease.invocation(&fake_tool(&root), &[]).unwrap();
    let child = worker_command(&root)
        .env(
            "KINEIN_SERIAL_EXEC_TEST_ARGS",
            serde_json::to_string(&arguments[1..]).unwrap(),
        )
        .spawn()
        .unwrap();
    let mut worker = Worker(child);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !root.join("tool-ready").exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        root.join("tool-ready").exists(),
        "helper did not execute the fixture tool"
    );
    drop(lease);
    let busy = registry
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap_err();
    assert_eq!(
        busy.code,
        JsonRpcErrorCode::SerialPortBusy,
        "core guard dropped while tool is still alive"
    );
    assert_eq!(
        busy.details.unwrap()["owner"]["processId"],
        worker.0.id(),
        "live tool identity was erased"
    );
    worker.0.kill().unwrap();
    worker.0.wait().unwrap();
    assert!(
        registry
            .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .is_ok()
    );
}

#[test]
fn an_obsolete_launch_token_cannot_start_the_tool() {
    let (root, port) = fixture("lease-obsolete-launch");
    let registry = registry(&root.join("locks"));
    let old = registry
        .acquire(port.to_str().unwrap(), "serial.identify", "Old", None)
        .unwrap();
    let (_, arguments) = old.invocation(&fake_tool(&root), &[]).unwrap();
    drop(old);
    let _new = registry
        .acquire(port.to_str().unwrap(), "serial.monitor", "New", None)
        .unwrap();
    let child = worker_command(&root)
        .env(
            "KINEIN_SERIAL_EXEC_TEST_ARGS",
            serde_json::to_string(&arguments[1..]).unwrap(),
        )
        .spawn()
        .unwrap();
    let mut worker = Worker(child);
    assert!(worker.0.wait().unwrap().success());
    assert!(
        fs::read_to_string(root.join("tool-result"))
            .unwrap()
            .contains("expirou")
    );
    assert!(
        !root.join("tool-ready").exists(),
        "obsolete launch executed a tool"
    );
}

#[test]
fn a_completed_reservation_is_retried_until_the_fork_window_closes() {
    let (root, port) = fixture("lease-fork-window");
    let directory = root.join("locks");
    let registry = registry(&directory);
    drop(
        registry
            .acquire(port.to_str().unwrap(), "serial.identify", "Identify", None)
            .unwrap(),
    );
    let pending = fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "pending")
        })
        .unwrap();
    let inherited = crate::platform::serial_lock_file(&pending).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if crate::platform::try_lock_file(&inherited, false).unwrap() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "fixture lock setup did not settle"
        );
        thread::sleep(Duration::from_millis(1));
    }
    let releaser = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        drop(inherited);
    });
    let lease = registry.acquire(port.to_str().unwrap(), "serial.identify", "Identify", None);
    releaser.join().unwrap();
    assert!(
        lease.is_ok(),
        "a transient inherited descriptor was reported as a real operation"
    );
}
