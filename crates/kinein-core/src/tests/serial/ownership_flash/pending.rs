//! Real deferred continuations: cancellation, transitions and last-moment checks.
use super::{fixture, native_proposal};
use serde_json::json;
use std::{
    fs,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn await_query(fixture: &super::Fixture) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !fixture.root.join("query.ready").exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(fixture.root.join("query.ready").exists());
}

#[test]
fn stopping_native_preparation_cancels_the_query_and_cannot_launch_later() {
    let mut fixture = fixture("native-cancel-pending");
    let intent = native_proposal(&mut fixture)["flash"].clone();
    fs::remove_file(fixture.root.join("query.ready")).unwrap();
    fs::write(fixture.root.join("query.wait"), "").unwrap();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    let started = fixture.rpc("run.start", json!({"flash":intent}));
    assert_eq!(started.result.unwrap()["kineinDeferred"], true);
    await_query(&fixture);
    assert!(fixture.rpc("runConfig.list", json!({})).error.is_none());
    assert!(
        fixture
            .rpc("run.start", json!({"command":"echo unrelated"}))
            .error
            .is_some()
    );
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    let finish = continuation
        .recv_timeout(Duration::from_secs(2))
        .expect("cancel must stop the version query before its deadline");
    let response = finish(&mut fixture.core);
    assert!(response.error.unwrap().message.contains("cancelada"));
    assert!(!fixture.root.join("flash.args").exists());
    fs::remove_file(fixture.root.join("query.wait")).unwrap();
    let monitor = fixture.rpc("serial.monitor", json!({"device":fixture.port}));
    fixture.close(monitor);
}

#[test]
fn reopening_the_same_workspace_cancels_a_native_attempt() {
    for action in ["same", "other", "close"] {
        let mut fixture = fixture(&format!("native-transition-{action}"));
        let intent = native_proposal(&mut fixture)["flash"].clone();
        fs::remove_file(fixture.root.join("query.ready")).unwrap();
        fs::write(fixture.root.join("query.wait"), "").unwrap();
        let (sender, continuation) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        fixture.rpc("run.start", json!({"flash":intent}));
        await_query(&fixture);
        let response = match action {
            "close" => fixture.rpc("workspace.close", json!({})),
            "other" => {
                let other = fixture.root.join("other");
                fs::create_dir(&other).unwrap();
                fixture.rpc("workspace.open", json!({"path":other}))
            }
            _ => fixture.rpc("workspace.open", json!({"path":fixture.root})),
        };
        assert!(response.error.is_none(), "{response:?}");
        let response =
            continuation.recv_timeout(Duration::from_secs(2)).unwrap()(&mut fixture.core);
        assert!(response.error.is_some());
        assert!(!fixture.root.join("flash.args").exists());
    }
}

#[test]
fn stopping_after_validation_still_prevents_a_delayed_launch() {
    let mut fixture = fixture("native-cancel-before-finish");
    let intent = native_proposal(&mut fixture)["flash"].clone();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    fixture.rpc("run.start", json!({"flash":intent}));
    let finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    let response = finish(&mut fixture.core);
    assert!(response.error.is_some());
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn an_old_cancelled_reply_does_not_discard_the_new_native_attempt() {
    let mut fixture = fixture("native-stale-finish");
    let first = native_proposal(&mut fixture)["flash"].clone();
    let other = fixture.root.join("other-port");
    fs::write(&other, "other node").unwrap();
    let second = fixture
        .rpc(
            "runConfig.flashProposal",
            json!({"native":true,"device":other}),
        )
        .result
        .unwrap()["flash"]
        .clone();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    fixture.rpc("run.start", json!({"flash":first}));
    let first_finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    fixture.rpc("run.start", json!({"flash":second}));
    assert!(first_finish(&mut fixture.core).error.is_some());
    assert!(fixture.core.run_preflight.is_some());
    let second_response =
        continuation.recv_timeout(Duration::from_secs(3)).unwrap()(&mut fixture.core);
    assert!(second_response.error.is_none(), "{second_response:?}");
    let id = second_response.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
}

#[test]
fn a_changed_artifact_after_worker_validation_is_refused_before_pty_start() {
    let mut fixture = fixture("native-changed-before-finish");
    let intent = native_proposal(&mut fixture)["flash"].clone();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    fixture.rpc("run.start", json!({"flash":intent}));
    let finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
    fs::write(
        fixture.root.join("build/app.bin"),
        "replaced after verification",
    )
    .unwrap();
    let response = finish(&mut fixture.core);
    assert!(response.error.is_some());
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn a_changed_active_configuration_does_not_launch_the_previous_flash() {
    let mut fixture = fixture("native-active-changed");
    let proposal = native_proposal(&mut fixture);
    assert!(
        fixture
            .rpc(
                "runConfig.save",
                json!({"name":"Native", "command":proposal["command"], "flash":proposal["flash"]})
            )
            .error
            .is_none()
    );
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    fixture.rpc("run.start", json!({}));
    let finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(
        fixture
            .rpc("runConfig.setActive", json!({}))
            .error
            .is_none()
    );
    let response = finish(&mut fixture.core);
    assert!(response.error.is_some());
    assert!(!fixture.root.join("flash.args").exists());
}
