//! Shared dispatch fixture: real jobs/PTYS, fake tools, regular-file ports.
use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
pub(super) struct Fixture {
    pub(super) core: crate::Core,
    pub(super) root: PathBuf,
    pub(super) port: PathBuf,
    pub(super) events: mpsc::Receiver<JsonRpcRequest>,
}

impl Fixture {
    pub(super) fn new(name: &str) -> Self {
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
        assert!(
            fixture
                .rpc("toolchain.set", json!({"role":"serialMonitor","id":"tio"}))
                .error
                .is_none()
        );
        fixture
    }

    pub(super) fn rpc(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(31_i64, method, Some(params)))
            .response()
            .clone()
    }

    pub(super) fn identify(&mut self) -> JsonRpcResponse {
        self.rpc(
            "serial.identify",
            json!({"device": self.port, "tool": self.root.join("bin/esptool")}),
        )
    }

    pub(super) fn files(&mut self) -> JsonRpcResponse {
        self.rpc(
            "serial.files",
            json!({"device": self.port,"action":"list","tool":self.root.join("bin/mpremote")}),
        )
    }

    pub(super) fn monitor(&mut self) -> JsonRpcResponse {
        self.rpc("serial.monitor", json!({"device": self.port}))
    }

    pub(super) fn event(&self, method: &str, key: &str, value: &str) -> Value {
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

    pub(super) fn finish(&self, response: JsonRpcResponse) -> Value {
        assert!(response.error.is_none(), "{response:?}");
        let id = response.result.unwrap()["jobId"]
            .as_str()
            .unwrap()
            .to_owned();
        self.event("event.job.finished", "jobId", &id)
    }

    pub(super) fn close(&mut self, response: JsonRpcResponse) {
        assert!(response.error.is_none(), "{response:?}");
        let id = response.result.unwrap()["id"].as_str().unwrap().to_owned();
        assert!(self.rpc("terminal.close", json!({"id":id})).error.is_none());
        self.event("event.terminal.closed", "id", &id);
    }
}
