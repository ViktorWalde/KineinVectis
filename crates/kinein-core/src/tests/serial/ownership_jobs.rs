//! Dispatch matrix with real jobs/PTYS and tools that never open the fixture port.
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use super::ownership_fixture::Fixture;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcResponse};
use serde_json::json;

fn busy(response: JsonRpcResponse, operation: &str) {
    let error = response.error.expect("conflict is refused before spawn");
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert_eq!(error.details.unwrap()["owner"]["operation"], operation);
}

fn wait_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !path.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        path.exists(),
        "fixture process did not start: {}",
        path.display()
    );
}

#[test]
fn monitor_excludes_both_jobs_and_releases_only_after_its_exit() {
    let mut fixture = Fixture::new("ownership-monitor-jobs");
    let monitor = fixture.monitor();
    wait_file(&fixture.root.join("tio.ready"));
    busy(fixture.identify(), "serial.monitor");
    busy(fixture.files(), "serial.monitor");
    assert!(!fixture.root.join("esptool.calls").exists());
    assert!(!fixture.root.join("mpremote.calls").exists());
    fixture.close(monitor);
    let identify = fixture.identify();
    assert_eq!(fixture.finish(identify)["status"], "success");
    let files = fixture.files();
    assert_eq!(fixture.finish(files)["status"], "success");
}

#[test]
fn identification_excludes_files_and_monitor_until_cancelled_process_is_collected() {
    let mut fixture = Fixture::new("ownership-identify-cancel");
    fs::remove_file(fixture.root.join("esptool.release")).unwrap();
    let identify = fixture.identify();
    let id = identify.result.as_ref().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_file(&fixture.root.join("esptool.ready"));
    busy(fixture.identify(), "serial.identify");
    busy(fixture.files(), "serial.identify");
    busy(fixture.monitor(), "serial.identify");
    assert!(!fixture.root.join("tio.calls").exists());
    assert!(!fixture.root.join("mpremote.calls").exists());
    assert!(
        fixture
            .rpc("job.cancel", json!({"jobId":id}))
            .error
            .is_none()
    );
    assert_eq!(fixture.finish(identify)["status"], "cancelled");
    let monitor = fixture.monitor();
    fixture.close(monitor);
}

#[test]
fn files_exclude_identification_and_monitor_until_the_job_completes() {
    let mut fixture = Fixture::new("ownership-files-complete");
    fs::remove_file(fixture.root.join("mpremote.release")).unwrap();
    let files = fixture.files();
    wait_file(&fixture.root.join("mpremote.ready"));
    busy(fixture.identify(), "serial.files");
    busy(fixture.files(), "serial.files");
    busy(fixture.monitor(), "serial.files");
    assert!(!fixture.root.join("esptool.calls").exists());
    assert!(!fixture.root.join("tio.calls").exists());
    fs::write(fixture.root.join("mpremote.release"), "release").unwrap();
    assert_eq!(fixture.finish(files)["status"], "success");
    let monitor = fixture.monitor();
    fixture.close(monitor);
}

#[test]
fn failed_tool_releases_the_channel_for_the_next_operation() {
    let mut fixture = Fixture::new("ownership-tool-failed");
    crate::write_executable(
        fixture.root.join("bin/mpremote"),
        "#!/bin/sh\necho 'fixture failure'\nexit 1\n",
    );
    let files = fixture.files();
    assert_eq!(fixture.finish(files)["status"], "failed");
    let identify = fixture.identify();
    assert_eq!(fixture.finish(identify)["status"], "success");
}

#[test]
fn two_ports_remain_independent_in_dispatch() {
    let mut fixture = Fixture::new("ownership-independent-ports");
    fs::remove_file(fixture.root.join("esptool.release")).unwrap();
    let first = fixture.identify();
    let other = fixture.root.join("other-port");
    fs::write(&other, "independent fixture").unwrap();
    let second = fixture.rpc(
        "serial.identify",
        json!({"device":other,"tool":fixture.root.join("bin/esptool")}),
    );
    assert!(first.error.is_none() && second.error.is_none());
    busy(fixture.identify(), "serial.identify");
    fs::write(fixture.root.join("esptool.release"), "release").unwrap();
    let first_id = first.result.unwrap()["jobId"].as_str().unwrap().to_owned();
    let second_id = second.result.unwrap()["jobId"].as_str().unwrap().to_owned();
    let mut finished = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(4);
    while finished.len() < 2 && Instant::now() < deadline {
        if let Ok(event) = fixture.events.recv_timeout(Duration::from_millis(30))
            && event.method == "event.job.finished"
        {
            let id = event.params.unwrap()["jobId"].as_str().unwrap().to_owned();
            if (id == first_id || id == second_id) && !finished.contains(&id) {
                finished.push(id);
            }
        }
    }
    assert!(finished.contains(&first_id) && finished.contains(&second_id));
}

#[test]
fn reopening_preserves_the_lease_and_changing_workspace_releases_it_after_exit() {
    let mut fixture = Fixture::new("ownership-workspace");
    let monitor = fixture.monitor();
    let id = monitor.result.unwrap()["id"].as_str().unwrap().to_owned();
    wait_file(&fixture.root.join("tio.ready"));
    assert!(
        fixture
            .rpc("workspace.open", json!({"path":fixture.root}))
            .error
            .is_none()
    );
    busy(fixture.identify(), "serial.monitor");
    let other = fixture.root.join("second");
    fs::create_dir(&other).unwrap();
    assert!(
        fixture
            .rpc("workspace.open", json!({"path":other}))
            .error
            .is_none()
    );
    fixture.event("event.terminal.closed", "id", &id);
    let identify = fixture.identify();
    assert_eq!(fixture.finish(identify)["status"], "success");
}

#[test]
fn a_serial_outcome_allows_an_immediate_followup_without_waiting_for_job_finished() {
    let mut fixture = Fixture::new("ownership-outcome-fence");
    for _ in 0..12 {
        let response = fixture.files();
        assert!(
            response.error.is_none(),
            "followup after outcome was refused: {response:?}"
        );
        let id = response.result.unwrap()["jobId"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            fixture.event("event.serial.files", "jobId", &id)["success"],
            true
        );
    }
}

#[test]
fn monitor_excludes_micropython_run_before_a_second_tool_starts() {
    for method in ["run.start", "run.script"] {
        let mut fixture = Fixture::new(&format!("monitor-mpy-{}", method.replace('.', "-")));
        fs::write(fixture.root.join("main.py"), "import machine\n").unwrap();
        let monitor = fixture.monitor();
        assert!(monitor.error.is_none());
        let params = if method == "run.start" {
            json!({"device": fixture.port})
        } else {
            json!({"path":fixture.root.join("main.py"), "device":fixture.port})
        };
        let refused = fixture.rpc(method, params);
        assert_eq!(
            refused.error.unwrap().code,
            JsonRpcErrorCode::SerialPortBusy
        );
        assert!(!fixture.root.join("mpremote.calls").exists());
    }
}
