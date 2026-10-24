//! The PIO Home server and its IDE agent against a fake `pio` that serves the
//! session page, the shutdown and a minimal JSON-RPC WebSocket (52 §14.45,
//! §14.46).

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::home::{Home, http};

/// A fake `pio` that behaves like `PlatformIO` 6.2 where the tests look
/// (`fixtures/fake_pio.py` says how).
const FAKE: &str = include_str!("fixtures/fake_pio.py");

fn bench(name: &str) -> (PathBuf, PathBuf) {
    let bin = std::env::temp_dir()
        .join("kinein-platformio-home")
        .join(format!("{}-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&bin));
    std::fs::create_dir_all(&bin).unwrap();
    let pio = crate::write_executable(bin.join("pio"), FAKE);
    (bin, pio)
}

fn launches(bin: &Path) -> Vec<Vec<String>> {
    std::fs::read_to_string(bin.join("home.launches"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn status(url: &str) -> Option<u16> {
    http()
        .get(url)
        .call()
        .ok()
        .map(|response| response.status().as_u16())
}

const DEADLINE: Duration = Duration::from_secs(20);

#[test]
fn the_session_page_is_served_reused_and_the_root_refused() {
    let (bin, pio) = bench("session");
    let home = Home::default();
    let session = home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    let launched = launches(&bin);
    assert_eq!(launched.len(), 1);
    let id = &launched[0][launched[0]
        .iter()
        .position(|arg| arg == "--session-id")
        .unwrap()
        + 1];
    assert_eq!(id.len(), 32, "128 bits in hex");
    assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let port = session.port.to_string();
    assert_eq!(
        launched[0],
        [
            "home",
            "--host",
            "127.0.0.1",
            "--port",
            &port,
            "--no-open",
            "--session-id",
            id,
            "--shutdown-timeout",
            "300"
        ]
    );
    assert_eq!(
        session.url,
        format!("http://127.0.0.1:{port}/session/{id}/")
    );
    assert_eq!(
        std::fs::read_to_string(bin.join("home.cwd")).unwrap(),
        crate::platform::canonicalize(&bin)
            .unwrap()
            .display()
            .to_string(),
        "beside the executable, outside any project"
    );
    assert_eq!(status(&session.url), Some(200));
    assert_eq!(status(&format!("http://127.0.0.1:{port}/")), Some(403));
    let again = home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    assert_eq!(again, session);
    assert_eq!(launches(&bin).len(), 1, "the live server is reused");
}

#[test]
fn stopping_closes_the_agent_then_the_server_and_frees_its_port() {
    let (bin, pio) = bench("stop");
    let home = Home::default();
    let (sender, exits) = mpsc::channel();
    let session = home
        .start(&pio, DEADLINE, None, move |code| {
            let _ = sender.send(code);
        })
        .unwrap();
    assert!(home.stop());
    assert_eq!(
        std::fs::read_to_string(bin.join("ws.closed"))
            .ok()
            .as_deref(),
        Some("frame"),
        "the agent closed its WebSocket first, with a close frame"
    );
    assert!(
        bin.join("home.shutdown").exists(),
        "then the shutdown request"
    );
    assert_eq!(status(&session.url), None, "nothing listens any more");
    assert!(!home.stop(), "nothing left to stop");
    assert!(
        exits.recv_timeout(Duration::from_secs(1)).is_err(),
        "a requested stop is not an exit on its own"
    );
    let next = home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    let id = |url: &str| {
        url.trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap()
            .to_owned()
    };
    assert_ne!(
        id(&next.url),
        id(&session.url),
        "a new server, a new random session"
    );
    drop(home);
    assert_eq!(status(&next.url), None, "dropping the owner stops it");
}

#[test]
fn an_exit_on_its_own_is_reported_once_and_frees_the_slot() {
    let (bin, pio) = bench("short");
    std::fs::write(bin.join("mode-short"), "").unwrap();
    let home = Home::default();
    let (sender, exits) = mpsc::channel();
    home.start(&pio, DEADLINE, None, move |code| {
        let _ = sender.send(code);
    })
    .unwrap();
    assert_eq!(exits.recv_timeout(Duration::from_secs(10)), Ok(Some(7)));
    assert!(
        exits.recv_timeout(Duration::from_millis(600)).is_err(),
        "once"
    );
    std::fs::remove_file(bin.join("mode-short")).unwrap();
    home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    assert_eq!(launches(&bin).len(), 2, "the next start launches again");
}

#[test]
fn a_server_that_ends_or_never_answers_is_an_error_with_its_output() {
    let (bin, pio) = bench("failures");
    std::fs::write(bin.join("mode-exit"), "").unwrap();
    let home = Home::default();
    let error = home.start(&pio, DEADLINE, None, |_| {}).unwrap_err();
    assert!(error.contains("fixture refuses to start"), "{error}");
    std::fs::remove_file(bin.join("mode-exit")).unwrap();
    std::fs::write(bin.join("mode-silent"), "").unwrap();
    let error = home
        .start(&pio, Duration::from_secs(2), None, |_| {})
        .unwrap_err();
    assert!(error.contains("não respondeu em 2 s"), "{error}");
    assert!(!home.stop(), "a failed start leaves nothing running");
    // A server that answers, but not 200 on this session, is not ready.
    std::fs::remove_file(bin.join("mode-silent")).unwrap();
    std::fs::write(bin.join("mode-wrong"), "").unwrap();
    let error = home
        .start(&pio, Duration::from_secs(2), None, |_| {})
        .unwrap_err();
    assert!(error.contains("não respondeu em 2 s"), "{error}");
}

#[test]
fn concurrent_starts_share_one_launch() {
    let (bin, pio) = bench("concurrent");
    let home = std::sync::Arc::new(Home::default());
    // All three start before any is joined: that is the race under test.
    let mut threads = Vec::new();
    for _ in 0..3 {
        let home = std::sync::Arc::clone(&home);
        let pio = pio.clone();
        threads.push(std::thread::spawn(move || {
            home.start(&pio, DEADLINE, None, |_| {})
        }));
    }
    let sessions: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap().unwrap())
        .collect();
    assert!(sessions.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(launches(&bin).len(), 1);
}

/// The page's commands, as the fake WebSocket hands them to the agent.
fn inject(bin: &Path, commands: &[Value]) {
    let lines: Vec<String> = commands.iter().map(Value::to_string).collect();
    std::fs::write(bin.join("commands.jsonl"), lines.join("\n") + "\n").unwrap();
}

fn results(bin: &Path, count: usize) -> Vec<Value> {
    let read = || -> Vec<Value> {
        std::fs::read_to_string(bin.join("results.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while read().len() < count && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    // A repeated answer would come right behind: give it the time.
    std::thread::sleep(Duration::from_millis(400));
    read()
}

/// The agent answers every command the page sends, keeps listening between
/// them, and turns the two actions into the UI event (52 §14.46).
#[test]
fn the_agent_answers_the_page_and_tells_the_ui_what_to_open() {
    let (bin, pio) = bench("agent");
    let project = bin.join("created");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("platformio.ini"), "[env:x]\n").unwrap();
    let home = Home::default();
    home.set_workspace(Some(&project));
    let (events, received) = mpsc::channel();
    home.start(&pio, DEADLINE, Some(events), |_| {}).unwrap();
    let file = project.join("src/main.cpp");
    inject(
        &bin,
        &[
            json!({"id": "c1", "method": "get_pio_project_dirs", "params": null}),
            json!({"id": "c2", "method": "open_project", "params": project}),
            json!({"id": "c3", "method": "open_text_document",
                   "params": {"path": file, "line": 3, "column": 2}}),
            json!({"id": "c4", "method": "open_project", "params": "relative/dir"}),
            json!({"id": "c5", "method": "unknown_command", "params": null}),
        ],
    );
    let answered = results(&bin, 5);
    assert_eq!(
        answered,
        [
            json!(["c1", [project]]),
            json!(["c2", true]),
            json!(["c3", true]),
            json!(["c4", false]),
            json!(["c5", null]),
        ],
        "every command answered, in order: the agent listened again each time"
    );
    let first = received.recv_timeout(Duration::from_secs(5)).unwrap();
    let second = received.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(first.method, "event.platformio.homeCommand");
    assert_eq!(
        first.params,
        Some(json!({"method": "open_project", "params": {"path": project}}))
    );
    assert_eq!(
        second.params,
        Some(json!({"method": "open_text_document",
                    "params": {"path": file, "line": 3, "column": 2}}))
    );
    assert!(
        received.recv_timeout(Duration::from_millis(500)).is_err(),
        "no event for the query, the relative path or an unknown command"
    );
    // Not a PlatformIO workspace: no project dirs.
    home.set_workspace(Some(&bin));
    inject(
        &bin,
        &[json!({"id": "c6", "method": "get_pio_project_dirs", "params": null})],
    );
    assert_eq!(results(&bin, 6)[5], json!(["c6", []]));
    assert_eq!(
        std::fs::read_to_string(bin.join("listens")).unwrap().len(),
        7,
        "one listen at the start and one after each answer, never a spin"
    );
}

/// PIO Home hands a pending command to every listen until its result is in,
/// and a listen may overtake the result: the agent passes each id on once.
#[test]
fn a_listen_that_overtakes_the_result_does_not_repeat_the_command() {
    let (bin, pio) = bench("overtake");
    std::fs::write(bin.join("mode-overtake"), "").unwrap();
    let home = Home::default();
    let (events, received) = mpsc::channel();
    home.start(&pio, DEADLINE, Some(events), |_| {}).unwrap();
    let project = bin.join("created");
    inject(
        &bin,
        &[
            json!({"id": "c1", "method": "open_project", "params": project}),
            json!({"id": "c2", "method": "open_project", "params": project}),
        ],
    );
    assert_eq!(results(&bin, 2), [json!(["c1", true]), json!(["c2", true])]);
    for _ in 0..2 {
        received.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    assert!(
        received.recv_timeout(Duration::from_millis(500)).is_err(),
        "the command delivered again is not a second event"
    );
}

#[test]
fn an_agent_that_cannot_connect_fails_the_start_and_stops_the_server() {
    let (bin, pio) = bench("no-ws");
    std::fs::write(bin.join("mode-no-ws"), "").unwrap();
    let home = Home::default();
    let error = home.start(&pio, DEADLINE, None, |_| {}).unwrap_err();
    assert!(error.contains("agente do PIO Home"), "{error}");
    assert!(!home.stop(), "nothing left running");
    let launched = &launches(&bin)[0];
    let port = &launched[launched.iter().position(|arg| arg == "--port").unwrap() + 1];
    assert_eq!(
        status(&format!("http://127.0.0.1:{port}/")),
        None,
        "the server it started is gone too"
    );
}

/// Server and agent are one unit: the agent leaving ends the server too,
/// and the exit is reported once.
#[test]
fn an_agent_that_leaves_ends_the_unit() {
    let (bin, pio) = bench("ws-close");
    std::fs::write(bin.join("mode-ws-close"), "").unwrap();
    let home = Home::default();
    let (sender, exits) = mpsc::channel();
    let session = home
        .start(&pio, DEADLINE, None, move |code| {
            let _ = sender.send(code);
        })
        .unwrap();
    assert_eq!(exits.recv_timeout(Duration::from_secs(10)), Ok(Some(0)));
    assert_eq!(status(&session.url), None, "the server went with it");
    assert!(!home.stop(), "the slot is free");
}

/// A release stops the server on its deadline, once, as `idle`: not as an
/// exit on its own.
#[test]
fn a_release_stops_the_server_when_its_deadline_passes() {
    let (_, pio) = bench("release");
    let home = Home::default();
    assert!(!home.release(Duration::ZERO, || {}), "nothing to release");
    let (sender, exits) = mpsc::channel();
    let session = home
        .start(&pio, DEADLINE, None, move |code| {
            let _ = sender.send(code);
        })
        .unwrap();
    let (idle, stopped) = mpsc::channel();
    let released = Instant::now();
    assert!(home.release(Duration::from_millis(400), move || {
        let _ = idle.send(Instant::now());
    }));
    let at = stopped.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(at - released >= Duration::from_millis(400), "not before");
    assert_eq!(status(&session.url), None, "the server is gone");
    assert!(
        stopped.recv_timeout(Duration::from_millis(500)).is_err(),
        "once"
    );
    assert!(
        exits.recv_timeout(Duration::from_millis(500)).is_err(),
        "not an exit"
    );
    assert!(!home.stop(), "the slot is free");
}

/// The page coming back (a start) or another release cancels the pending
/// deadline; after a stop, it finds no server.
#[test]
fn a_start_or_another_release_cancels_the_deadline() {
    let (_, pio) = bench("release-cancel");
    let home = Home::default();
    let session = home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    let (idle, stopped) = mpsc::channel();
    let watch = move || {
        let idle = idle.clone();
        move || {
            let _ = idle.send(());
        }
    };
    let short = Duration::from_millis(500);
    assert!(home.release(short, watch()));
    assert_eq!(home.start(&pio, DEADLINE, None, |_| {}).unwrap(), session);
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(status(&session.url), Some(200), "the start cancelled it");
    assert!(home.release(Duration::from_millis(800), watch()));
    std::thread::sleep(Duration::from_millis(500));
    assert!(home.release(Duration::from_millis(800), watch()));
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        status(&session.url),
        Some(200),
        "the second one restarted it"
    );
    stopped.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(
        stopped.recv_timeout(Duration::from_millis(500)).is_err(),
        "once"
    );
    let next = home.start(&pio, DEADLINE, None, |_| {}).unwrap();
    assert!(home.release(short, watch()));
    assert!(home.stop());
    std::thread::sleep(Duration::from_millis(900));
    assert!(
        stopped.try_recv().is_err(),
        "after a stop there is nothing to stop"
    );
    assert_eq!(status(&next.url), None);
}

fn rpc(core: &mut crate::Core, method: &str) -> kinein_protocol::JsonRpcResponse {
    rpc_with(core, method, serde_json::json!({}))
}

fn rpc_with(
    core: &mut crate::Core,
    method: &str,
    params: Value,
) -> kinein_protocol::JsonRpcResponse {
    core.handle_request(&kinein_protocol::JsonRpcRequest::new(
        7_i64,
        method,
        Some(params),
    ))
    .response()
    .clone()
}

/// Through the protocol: a missing `pio`, start and stop, and the event of a
/// server that ends on its own.
#[test]
fn the_methods_answer_and_an_exit_on_its_own_becomes_the_event() {
    let empty = std::env::temp_dir()
        .join("kinein-platformio-home")
        .join(format!("{}-empty", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(empty));
    let missing = rpc(&mut core, "platformio.homeStart");
    assert_eq!(
        missing.error.map(|error| error.code),
        Some(kinein_protocol::JsonRpcErrorCode::ToolNotFound)
    );

    let (bin, _) = bench("rpc");
    let mut core =
        crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(bin.clone()));
    let (sender, events) = mpsc::channel();
    core.enable_lsp(sender);
    let started = rpc(&mut core, "platformio.homeStart");
    let url = started.result.as_ref().unwrap()["url"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(status(&url), Some(200));
    let stopped = rpc(&mut core, "platformio.homeStop");
    assert_eq!(stopped.result.unwrap()["stopped"], true);

    std::fs::write(bin.join("mode-short"), "").unwrap();
    assert!(rpc(&mut core, "platformio.homeStart").error.is_none());
    let deadline = Instant::now() + Duration::from_secs(10);
    let exited = loop {
        let event = events.recv_timeout(Duration::from_millis(200));
        if let Ok(event) = event
            && event.method == "event.platformio.homeExited"
        {
            break event;
        }
        assert!(Instant::now() < deadline, "no exit event");
    };
    assert_eq!(exited.params, Some(json!({"code": 7, "reason": "exited"})));
    assert_eq!(
        rpc(&mut core, "platformio.homeStop").result.unwrap()["stopped"],
        false
    );

    // The release: its bounds, nothing to release, and the idle event.
    for after in [json!(3601), json!(-1), json!("300")] {
        let refused = rpc_with(
            &mut core,
            "platformio.homeRelease",
            json!({ "after": after }),
        );
        assert_eq!(
            refused.error.map(|error| error.code),
            Some(kinein_protocol::JsonRpcErrorCode::InvalidParams),
            "{after}"
        );
    }
    let nothing = rpc_with(&mut core, "platformio.homeRelease", json!({}));
    assert_eq!(nothing.result.unwrap()["releasing"], false);
    std::fs::remove_file(bin.join("mode-short")).unwrap();
    let again = rpc(&mut core, "platformio.homeStart").result.unwrap();
    let defaulted = rpc_with(&mut core, "platformio.homeRelease", json!({}));
    assert_eq!(defaulted.result.unwrap()["releasing"], true);
    std::thread::sleep(Duration::from_secs(1));
    assert_eq!(
        status(again["url"].as_str().unwrap()),
        Some(200),
        "without `after`, 300 s"
    );
    assert_eq!(
        rpc(&mut core, "platformio.homeStart").result.unwrap(),
        again
    );
    let released = rpc_with(&mut core, "platformio.homeRelease", json!({ "after": 0 }));
    assert_eq!(released.result.unwrap()["releasing"], true);
    let deadline = Instant::now() + Duration::from_secs(10);
    let idle = loop {
        let event = events.recv_timeout(Duration::from_millis(200));
        if let Ok(event) = event
            && event.method == "event.platformio.homeExited"
        {
            break event;
        }
        assert!(Instant::now() < deadline, "no idle event");
    };
    assert_eq!(idle.params, Some(json!({"code": null, "reason": "idle"})));
}
