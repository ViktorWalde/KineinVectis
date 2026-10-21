//! Run and serial operations share the existing PTY lifetime, without hardware.
use super::ownership_fixture::Fixture;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcResponse};
use serde_json::json;
use std::{fs, time::Duration};

fn micropython(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fs::write(fixture.root.join("main.py"), "import machine\n").unwrap();
    fixture
}

fn assert_busy(response: JsonRpcResponse, operation: &str) {
    let error = response
        .error
        .expect("managed serial operation must be refused before tool spawn");
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert_eq!(error.details.unwrap()["owner"]["operation"], operation);
}

fn stop(fixture: &mut Fixture, response: &JsonRpcResponse) {
    let id = response.result.as_ref().unwrap()["terminalId"]
        .as_str()
        .unwrap();
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    fixture.event("event.terminal.closed", "id", id);
}

#[test]
fn both_micropython_run_methods_exclude_jobs_and_monitor_until_stopped() {
    for method in ["run.start", "run.script"] {
        let mut fixture = micropython(&format!("mpy-owner-{}", method.replace('.', "-")));
        fs::remove_file(fixture.root.join("mpremote.release")).unwrap();
        let mut params = json!({"device":fixture.port});
        if method == "run.script" {
            params["path"] = json!(fixture.root.join("main.py"));
        }
        let run = fixture.rpc(method, params);
        assert!(run.error.is_none(), "{run:?}");
        assert_busy(fixture.identify(), method);
        assert_busy(fixture.files(), method);
        assert_busy(fixture.monitor(), method);
        stop(&mut fixture, &run);
        let monitor = fixture.monitor();
        fixture.close(monitor);
    }
}

#[test]
fn an_automatic_run_refuses_an_existing_port_owner_and_then_excludes_other_ports() {
    let mut fixture = micropython("mpy-automatic-run");
    let monitor = fixture.monitor();
    let denied = fixture.rpc("run.start", json!({}));
    assert_busy(denied, "serial.monitor");
    assert!(!fixture.root.join("mpremote.calls").exists());
    fixture.close(monitor);
    fs::remove_file(fixture.root.join("mpremote.release")).unwrap();
    let run = fixture.rpc("run.start", json!({}));
    assert!(run.error.is_none(), "{run:?}");
    let other = fixture.root.join("second-port");
    fs::write(&other, "fixture").unwrap();
    let denied = fixture.rpc("serial.monitor", json!({"device":other}));
    assert_busy(denied, "run.start");
    stop(&mut fixture, &run);
    let monitor = fixture.rpc("serial.monitor", json!({"device":other}));
    fixture.close(monitor);
}

#[test]
fn another_explicit_port_remains_available_during_micropython_run() {
    let mut fixture = micropython("mpy-distinct-port");
    fs::remove_file(fixture.root.join("mpremote.release")).unwrap();
    let run = fixture.rpc("run.start", json!({"device":fixture.port}));
    assert!(run.error.is_none(), "{run:?}");
    let other = fixture.root.join("other");
    fs::write(&other, "fixture").unwrap();
    let monitor = fixture.rpc("serial.monitor", json!({"device":other}));
    assert!(monitor.error.is_none(), "{monitor:?}");
    stop(&mut fixture, &run);
    fixture.close(monitor);
}

#[test]
fn missing_ports_are_rejected_but_host_python_and_manual_config_ignore_the_selection() {
    let mut fixture = micropython("mpy-native-only");
    let missing = fixture.root.join("missing");
    let denied = fixture.rpc("run.start", json!({"device":missing}));
    assert_eq!(denied.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
    assert!(!fixture.root.join("mpremote.calls").exists());
    let monitor = fixture.monitor();
    let saved = fixture.rpc(
        "runConfig.save",
        json!({"name":"Host", "command":"printf manual"}),
    );
    assert!(saved.error.is_none());
    let run = fixture.rpc("run.start", json!({"device":fixture.port}));
    assert!(run.error.is_none(), "{run:?}");
    let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
    fixture.event("event.terminal.closed", "id", id);
    fs::write(fixture.root.join("main.py"), "print('host')\n").unwrap();
    let interpreter = fixture.root.join(".venv/bin/python");
    fs::create_dir_all(interpreter.parent().unwrap()).unwrap();
    crate::write_executable(&interpreter, "#!/bin/sh\nprintf host-python\n");
    let run = fixture.rpc(
        "run.script",
        json!({"path":fixture.root.join("main.py"),"device":missing}),
    );
    assert!(run.error.is_none(), "{run:?}");
    let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
    let (lines, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, id, Duration::from_secs(4));
    assert_eq!(exit, Some(0), "{lines:?}");
    assert!(lines.join("\n").contains("host-python"));
    fixture.close(monitor);
}

#[test]
fn a_finished_micropython_run_releases_before_the_terminal_closed_event() {
    let mut fixture = micropython("mpy-natural-exit");
    for status in [0, 7] {
        crate::write_executable(
            fixture.root.join("bin/mpremote"),
            format!("#!/bin/sh\nexit {status}\n"),
        );
        let run = fixture.rpc("run.start", json!({"device":fixture.port}));
        assert!(run.error.is_none(), "{run:?}");
        let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
        fixture.event("event.terminal.closed", "id", id);
        let monitor = fixture.monitor();
        fixture.close(monitor);
    }
}

#[test]
fn port_and_script_names_remain_literal_in_both_micropython_run_methods() {
    let mut fixture = micropython("mpy-literal-arguments");
    let script = fixture.root.join("file it's safe.py");
    fs::write(&script, "print('fixture')\n").unwrap();
    crate::write_executable(
        fixture.root.join("bin/mpremote"),
        "#!/bin/sh\nprintf '%s\\n' \"$@\"\n",
    );
    for method in ["run.start", "run.script"] {
        for label in ["my board", "board's adapter"] {
            let device = fixture.root.join(label);
            fs::write(&device, "regular fixture").unwrap();
            let mut params = json!({"device": device});
            if method == "run.script" {
                params["path"] = json!(script);
            }
            let run = fixture.rpc(method, params);
            assert!(run.error.is_none(), "{run:?}");
            let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
            let (lines, exit) = crate::tests::terminal_run_until_closed(
                &fixture.events,
                id,
                Duration::from_secs(4),
            );
            assert_eq!(exit, Some(0), "{lines:?}");
            let path = if method == "run.script" {
                script.display().to_string()
            } else {
                "main.py".to_owned()
            };
            assert_eq!(
                lines,
                vec![
                    "connect".to_owned(),
                    device.display().to_string(),
                    "run".to_owned(),
                    path
                ]
            );
        }
    }
}

#[test]
fn a_failed_program_releases_the_port_after_the_terminal_reports_its_exit() {
    let mut fixture = micropython("mpy-spawn-failure");
    crate::write_executable(
        fixture.root.join("bin/mpremote"),
        "#!/missing-fixture-interpreter\n",
    );
    let denied = fixture.rpc(
        "run.script",
        json!({"path":fixture.root.join("main.py"), "device":fixture.port}),
    );
    if let Some(result) = denied.result {
        let id = result["terminalId"].as_str().unwrap();
        let (lines, exit) =
            crate::tests::terminal_run_until_closed(&fixture.events, id, Duration::from_secs(4));
        assert!(
            exit.is_some_and(|code| code != 0),
            "invalid program reported success: {lines:?}"
        );
    } else {
        assert!(denied.error.is_some());
    }
    let monitor = fixture.monitor();
    fixture.close(monitor);
}

#[test]
fn a_workspace_transition_stops_micropython_and_releases_the_port() {
    let mut fixture = micropython("mpy-workspace-change");
    fs::remove_file(fixture.root.join("mpremote.release")).unwrap();
    let run = fixture.rpc("run.start", json!({}));
    assert!(run.error.is_none(), "{run:?}");
    let other = fixture.root.join("other-project");
    fs::create_dir(&other).unwrap();
    fs::write(other.join("main.py"), "print('host')\n").unwrap();
    assert!(
        fixture
            .rpc("workspace.open", json!({"path": other}))
            .error
            .is_none()
    );
    let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
    fixture.event("event.terminal.closed", "id", id);
    let monitor = fixture.monitor();
    fixture.close(monitor);
}

#[test]
fn explicit_micropython_access_is_checked_before_any_tool_runs() {
    use std::os::unix::fs::PermissionsExt;
    let mut fixture = micropython("mpy-port-permission");
    fs::set_permissions(&fixture.port, fs::Permissions::from_mode(0o000)).unwrap();
    for method in ["run.start", "run.script"] {
        let mut params = json!({"device":fixture.port});
        if method == "run.script" {
            params["path"] = json!(fixture.root.join("main.py"));
        }
        let denied = fixture.rpc(method, params);
        if rustix::process::geteuid().is_root() {
            // Root can access a regular file with mode 000; assert actual access,
            // rather than expecting an unprivileged denial in this environment.
            assert!(denied.error.is_none(), "{denied:?}");
            let id = denied.result.as_ref().unwrap()["terminalId"]
                .as_str()
                .unwrap();
            fixture.event("event.terminal.closed", "id", id);
        } else {
            let error = denied.error.unwrap();
            assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
            assert!(!fixture.root.join("mpremote.calls").exists());
        }
    }
    fs::set_permissions(&fixture.port, fs::Permissions::from_mode(0o600)).unwrap();
    let run = fixture.rpc("run.start", json!({"device":fixture.port}));
    assert!(run.error.is_none());
    let id = run.result.as_ref().unwrap()["terminalId"].as_str().unwrap();
    fixture.event("event.terminal.closed", "id", id);
}
