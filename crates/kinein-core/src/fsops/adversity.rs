//! The adverse files of the tests: a link, a FIFO and the deadline that
//! opens the other side of a FIFO a read holds (INF-15, moved here by the
//! INF-18 so that the readers of the base use them too).

use std::{fmt, fs, path::Path};

/// A link at `link` to `target`, a folder or what does not exist yet, for the
/// tests: `ln -s` on Unix and the junction on Windows, which needs no
/// privilege (60 §3.3), so they run on both.
pub fn link(target: &Path, link: &Path) {
    let made = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
    } else {
        std::process::Command::new("ln")
            .arg("-s")
            .arg(target)
            .arg(link)
            .output()
    }
    .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
}

/// A FIFO at `path`, for the tests: `mkfifo` on Unix. `false` on Windows,
/// whose file system has no FIFO: there the tests prove the folder and the
/// link (63 §E.2).
#[must_use]
pub fn fifo(path: &Path) -> bool {
    if cfg!(windows) {
        return false;
    }
    let made = std::process::Command::new("mkfifo")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    true
}

/// What `work` returns, run in a thread.
///
/// A read held in the `open` of the FIFO `held` fails the test after 20 s,
/// where the normal path takes milliseconds: the other side is opened, so
/// the thread ends and the run never hangs (63 §E.2).
pub fn unblocked<T: Send + 'static>(held: &Path, work: impl FnOnce() -> T + Send + 'static) -> T {
    use std::sync::mpsc::RecvTimeoutError;

    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || drop(sender.send(work())));
    match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(value) => value,
        Err(RecvTimeoutError::Disconnected) => match worker.join() {
            Err(panic) => std::panic::resume_unwind(panic),
            Ok(()) => unreachable!("o trabalho terminou sem resposta"),
        },
        Err(RecvTimeoutError::Timeout) => {
            let other = held.to_owned();
            // O `open` de escrita volta assim que a leitura segura o FIFO.
            std::thread::spawn(move || drop(fs::OpenOptions::new().write(true).open(other)));
            panic!("a leitura prendeu no FIFO {}", held.display());
        }
    }
}

/// The reader `read` of the project file `path` (INF-18) takes what is not a
/// regular file as absent.
///
/// With `valid` there it says something else than without the file, so it
/// reads that file; the folder and the FIFO in its place say what the absent
/// file says, the FIFO run by [`unblocked`], so a read held in it fails the
/// test instead of hanging the run. The FIFO exists only on Unix.
pub fn reads_as_absent<T>(path: &Path, valid: &[u8], read: impl Fn() -> T + Send + 'static)
where
    T: PartialEq + fmt::Debug + Send + 'static,
{
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, valid).unwrap();
    let control = read();
    fs::remove_file(path).unwrap();
    let absent = read();
    assert_ne!(control, absent, "o leitor não leu {}", path.display());
    fs::create_dir(path).unwrap();
    assert_eq!(read(), absent, "a pasta no lugar de {}", path.display());
    fs::remove_dir(path).unwrap();
    if fifo(path) {
        assert_eq!(
            unblocked(path, read),
            absent,
            "o FIFO no lugar de {}",
            path.display()
        );
        fs::remove_file(path).unwrap();
    }
}
