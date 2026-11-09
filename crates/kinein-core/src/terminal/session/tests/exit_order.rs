//! INF-04: the end of a session comes after its last output. The waiter used
//! to clear `running` and send `closed` as soon as the process exited, while
//! the reader could still be applying the output; a scroll made then moved
//! with the history that was still growing.
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use kinein_protocol::JsonRpcRequest;

use super::super::TerminalManager;
use super::lifetime::{ChildCleanup, process_running, wait_closed};
use super::temp_root;
use crate::terminal::render::tests::render_text;

const LAST_LINE: &str = "kv-ultima-linha";

/// `true` if `event` is the end of `id`.
fn is_closed(event: &JsonRpcRequest, id: &str) -> bool {
    event.method == "event.terminal.closed"
        && event
            .params
            .as_ref()
            .is_some_and(|params| params["id"] == id)
}

#[test]
fn a_session_ends_only_after_its_last_output() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    // Only the reader's EOF ends the wait here: no bound decides the result.
    manager.use_reader_drain(Duration::from_secs(3600));
    let root = temp_root("exit-order");
    // The waiter drops the resource as soon as it observes the exit.
    let (resource, exit_observed) = mpsc::channel::<()>();
    let script = format!("read line; printf '{LAST_LINE}\\n'");
    let id = manager
        .open_command_holding(&root, "sh", &["-c".to_owned(), script], &[], resource)
        .unwrap();

    // The reader stops at the state lock, and the process prints and exits.
    let state = Arc::clone(&manager.sessions[&id].state);
    let reader_stopped = state.lock().unwrap();
    manager.write(&id, "\n").unwrap();
    assert_eq!(
        exit_observed.recv_timeout(Duration::from_secs(10)),
        Err(mpsc::RecvTimeoutError::Disconnected),
        "the exit was not observed"
    );
    // With the reader stopped there is no end yet. The window only gives a
    // waiter that does not wait the time to show it.
    let window = Instant::now() + Duration::from_millis(500);
    while let Ok(event) = receiver.recv_timeout(window.saturating_duration_since(Instant::now())) {
        assert!(!is_closed(&event, &id), "closed before the last output");
    }
    assert!(
        manager.is_open(&id),
        "running cleared before the last output"
    );

    drop(reader_stopped);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last_output_seen = false;
    loop {
        let event = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("the session did not end");
        if is_closed(&event, &id) {
            break;
        }
        if event.method == "event.terminal.render" && render_text(&event).contains(LAST_LINE) {
            last_output_seen = true;
        }
    }
    assert!(
        last_output_seen,
        "closed came before the render of the last line"
    );
    assert!(!manager.is_open(&id));
}

#[test]
fn a_child_that_keeps_the_pty_open_delays_the_end_only_up_to_the_bound() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    manager.use_reader_drain(Duration::from_millis(200));
    let root = temp_root("exit-held-pty");
    let pid_file = root.join("holder.pid");
    // The shell exits; the background `sleep`, with SIGHUP ignored, keeps the
    // PTY open, and the reader sees no EOF.
    let script = format!(
        "trap '' HUP; sleep 30 & {}",
        crate::sh_save_background_pid(&pid_file)
    );
    let id = manager
        .open_command(&root, "sh", &["-c".to_owned(), script])
        .unwrap();
    wait_closed(&receiver, std::slice::from_ref(&id));
    let holder = ChildCleanup {
        pid: std::fs::read_to_string(pid_file).unwrap().trim().to_owned(),
        active: true,
    };
    assert!(!manager.is_open(&id));
    assert!(process_running(&holder.pid), "nothing kept the PTY open");
}
