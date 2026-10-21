//! Dispatch matrix with real jobs/PTYS and tools that never open the fixture port.
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

struct Fixture {
    core: crate::Core,
    root: PathBuf,
    port: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = super::temp_dir(name);
        let bin = root.join("bin");
        fs::create_dir(&bin).unwrap();
        let port = root.join("port");
        fs::write(&port, "regular fixture; tools never open it").unwrap();
        fs::write(root.join("main.py"), "print('fixture')\n").unwrap();
        for (tool, output) in [
            ("esptool", "Chip type: ESP32-D0WD-V3 (revision v3.1)"),
            ("mpremote", "ls :\n       15 main.py"),
            ("tio", "monitor fixture"),
        ] {
            let calls = crate::platform::program_invocation(
                &root.join(format!("{tool}.calls")).display().to_string(),
            );
            let ready = crate::platform::program_invocation(
                &root.join(format!("{tool}.ready")).display().to_string(),
            );
            let release = root.join(format!("{tool}.release"));
            fs::write(&release, "release fixture").unwrap();
            let release = crate::platform::program_invocation(&release.display().to_string());
            let body = if tool == "tio" {
                "trap '' HUP\nexec sleep 30".to_owned()
            } else {
                format!("while [ ! -f {release} ]; do sleep 0.02; done\nprintf '%s\\n' '{output}'")
            };
            crate::write_executable(
                bin.join(tool),
                format!(
                    "#!/bin/sh\ncase \"$*\" in *version*|*--help*|*-V*) echo '{tool} v5.4.0'; exit 0;; esac\nprintf 'called\\n' >> {calls}\n: > {ready}\n{body}\n"
                ),
            );
        }
        let mut core =
            crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(bin));
        let (sender, events) = mpsc::channel();
        core.enable_lsp(sender);
        let mut fixture = Self {
            core,
            root,
            port,
            events,
        };
        let root = fixture.root.clone();
        assert!(
            fixture
                .rpc("workspace.open", json!({"path":root}))
                .error
                .is_none()
        );
        fixture
    }

    fn rpc(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(31_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn identify(&mut self) -> JsonRpcResponse {
        self.rpc(
            "serial.identify",
            json!({"device": self.port, "tool": self.root.join("bin/esptool")}),
        )
    }

    fn files(&mut self) -> JsonRpcResponse {
        self.rpc(
            "serial.files",
            json!({"device": self.port,"action":"list","tool":self.root.join("bin/mpremote")}),
        )
    }

    fn monitor(&mut self) -> JsonRpcResponse {
        self.rpc("serial.monitor", json!({"device": self.port}))
    }

    fn event(&self, method: &str, key: &str, value: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(4);
        while Instant::now() < deadline {
            if let Ok(event) = self.events.recv_timeout(Duration::from_millis(30))
                && event.method == method
                && let Some(params) = event.params
                && params[key] == value
            {
                return params;
            }
        }
        panic!("missing fixture event {method} {key}={value}");
    }

    fn finish(&self, response: JsonRpcResponse) -> Value {
        assert!(response.error.is_none(), "{response:?}");
        let id = response.result.unwrap()["jobId"]
            .as_str()
            .unwrap()
            .to_owned();
        self.event("event.job.finished", "jobId", &id)
    }

    fn close(&mut self, response: JsonRpcResponse) {
        assert!(response.error.is_none(), "{response:?}");
        let id = response.result.unwrap()["id"].as_str().unwrap().to_owned();
        assert!(self.rpc("terminal.close", json!({"id":id})).error.is_none());
        self.event("event.terminal.closed", "id", &id);
    }
}

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
            finished.push(event.params.unwrap()["jobId"].as_str().unwrap().to_owned());
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
