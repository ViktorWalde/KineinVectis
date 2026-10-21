//! One child handle shared by closing and nonblocking exit observation.
//! Never signal a PID after the waiter has reaped it.

use std::{
    io,
    sync::{Arc, Mutex, MutexGuard},
};

use portable_pty::{Child, ExitStatus};

type ChildHandle = Box<dyn Child + Send + Sync>;
pub(super) type SharedChild = Arc<Mutex<ChildHandle>>;

fn lock(child: &SharedChild) -> io::Result<MutexGuard<'_, ChildHandle>> {
    child
        .lock()
        .map_err(|_| io::Error::other("o controle do processo do terminal está indisponível"))
}

pub(super) fn poll(child: &SharedChild) -> io::Result<Option<ExitStatus>> {
    lock(child)?.try_wait()
}

pub(super) fn stop(child: &SharedChild) -> io::Result<()> {
    let mut child = lock(child)?;
    if child.try_wait()?.is_some() {
        return Ok(());
    }
    // portable-pty creates the Unix session/group with setsid. The child is
    // still unreaped under this lock, so its PID cannot have been recycled.
    crate::platform::terminal_group(child.process_id()).map_or_else(
        || child.kill(),
        |group| crate::platform::try_kill_group(&group),
    )
}
