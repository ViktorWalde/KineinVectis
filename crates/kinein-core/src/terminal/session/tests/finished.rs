//! 52 §14.38: a finished command keeps its output readable, within a bound.
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kinein_protocol::{JsonRpcRequest, TerminalMouseEvent, TerminalMouseModifiers};
use serde_json::Value;

use super::super::{MAX_SESSIONS, TerminalError, TerminalManager};
use super::lifetime::wait_closed;
use super::temp_root;
use crate::terminal::render::tests::render_text;

/// Runs `script` to its end; `None` without a PTY in this environment.
fn finished(
    manager: &mut TerminalManager,
    receiver: &mpsc::Receiver<JsonRpcRequest>,
    name: &str,
    script: &str,
) -> Option<String> {
    // One folder per test: `temp_root` recreates it, and tests run in parallel.
    let root = temp_root(name);
    let id = manager
        .open_command(&root, "sh", &["-c".to_owned(), script.to_owned()])
        .ok()?;
    wait_closed(receiver, std::slice::from_ref(&id));
    let deadline = Instant::now() + Duration::from_secs(3);
    while manager.is_open(&id) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!manager.is_open(&id));
    Some(id)
}

/// The first render of `id` that `expected` accepts. The emitter may still
/// publish an earlier frame after the exit, so a render is not an answer by
/// position; an action that did nothing never produces the expected frame.
fn render_where(
    receiver: &mpsc::Receiver<JsonRpcRequest>,
    id: &str,
    expected: impl Fn(&Value, &str) -> bool,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    while let Ok(event) = receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
    {
        let params = event.params.clone().unwrap_or_default();
        if event.method == "event.terminal.render"
            && params["id"] == id
            && expected(&params, &render_text(&event))
        {
            return params;
        }
    }
    panic!("no expected render of {id}");
}

#[test]
fn a_finished_session_stays_readable_but_takes_no_input() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    // Mouse reporting is left on: a live process would receive the wheel.
    let Some(id) = finished(
        &mut manager,
        &receiver,
        "finished-readable",
        "seq 1 60; printf '\\033[?1000h'",
    ) else {
        return;
    };
    manager.scroll(&id, 10).unwrap();
    render_where(&receiver, &id, |params, _| params["scrollback"] == 10);
    let wheel = TerminalMouseEvent::Wheel { lines: 3 };
    manager
        .mouse(&id, 0, 0, wheel, TerminalMouseModifiers::default())
        .unwrap();
    // The wheel scrolls the history of a finished session.
    render_where(&receiver, &id, |params, _| params["scrollback"] == 13);
    manager.resize(&id, 100, 30).unwrap();
    render_where(&receiver, &id, |params, _| {
        params["cols"] == 100 && params["rows"] == 30
    });
    manager.select_all(&id, "finished-1").unwrap();
    let copied = manager.copy_selection(&id, "finished-1").unwrap().unwrap();
    assert!(
        copied.lines().any(|line| line == "1") && copied.lines().any(|line| line == "60"),
        "{copied}"
    );
    manager.clear_scrollback(&id).unwrap();
    render_where(&receiver, &id, |params, _| params["scrollbackMax"] == 0);
    assert!(matches!(
        manager.write(&id, "x"),
        Err(TerminalError::NotOpen)
    ));
}

#[test]
fn finished_sessions_are_kept_up_to_the_bound_and_closed_ones_leave() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    let Some(first) = finished(
        &mut manager,
        &receiver,
        "finished-bound",
        "printf first-output",
    ) else {
        return;
    };
    let mut ids = vec![first];
    for index in 0..=MAX_SESSIONS {
        ids.push(
            finished(
                &mut manager,
                &receiver,
                "finished-bound",
                &format!("printf output-{index}"),
            )
            .unwrap(),
        );
    }
    // The open that followed MAX_SESSIONS + 1 finished sessions dropped the oldest.
    assert!(matches!(
        manager.scroll(&ids[0], 1),
        Err(TerminalError::NotOpen)
    ));
    for id in &ids[1..] {
        manager.scroll(id, 1).unwrap();
        render_where(&receiver, id, |_, text| text.contains("output-"));
    }
    let root = temp_root("finished-closed");
    let closed = manager
        .open_command(&root, "sh", &["-c".to_owned(), "sleep 30".to_owned()])
        .unwrap();
    manager.close(&closed).unwrap();
    wait_closed(&receiver, std::slice::from_ref(&closed));
    assert!(matches!(
        manager.scroll(&closed, 1),
        Err(TerminalError::NotOpen)
    ));
    // The cap counts live processes only: twelve retained outputs do not block.
    let live = manager
        .open_command(&root, "sh", &["-c".to_owned(), "sleep 30".to_owned()])
        .unwrap();
    assert!(!manager.sessions.contains_key(&closed));
    manager.close(&live).unwrap();
}
