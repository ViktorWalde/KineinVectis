//! Real probe.list dispatch with a slow fake adapter; no hardware or global PATH changes.
use kinein_protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::mpsc, time::Duration};

struct Fixture {
    core: crate::Core,
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "kinein-probe-dispatch-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("CMakeLists.txt"), "project(probe_fixture C)\n").unwrap();
        crate::write_executable(
            root.join("bin/probe-rs"),
            "#!/bin/sh\nif [ \"$1\" != list ]; then echo 'probe-rs 0.32.0'; exit 0; fi\nsleep .25\necho '[0]: Fixture -- 1234:5678:S1 (CMSIS-DAP)'\n",
        );
        let root = crate::platform::canonicalize(&root).unwrap();
        let core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            root.join("bin"),
        ));
        let mut fixture = Self { core, root };
        let root = fixture.root.clone();
        assert!(
            fixture
                .call("workspace.open", json!({"path":root}))
                .error
                .is_none()
        );
        assert!(
            fixture
                .call(
                    "toolchain.set",
                    json!({"role":"debugAdapter", "id":"probe-rs"})
                )
                .error
                .is_none()
        );
        fixture
    }

    fn call(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(41_i64, method, Some(params)))
            .response()
            .clone()
    }
}

#[test]
fn slow_probe_query_leaves_the_core_loop_available() {
    let mut fixture = Fixture::new("deferred");
    let (sender, continuations) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    let response =
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(42_i64, "probe.list", Some(json!({}))));
    assert!(
        response.is_deferred(),
        "probe.list waited on the core loop: {response:?}"
    );
    assert!(fixture.call("runConfig.list", json!({})).error.is_none());
    let response = continuations.recv_timeout(Duration::from_secs(4)).unwrap()(&mut fixture.core);
    assert_eq!(response.id, Some(json!(42)));
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(response.result.unwrap()["probes"][0]["serial"], "S1");
}

#[test]
fn old_probe_result_is_refused_after_workspace_or_adapter_changes() {
    for change in ["workspace.close", "workspace.open", "toolchain.set"] {
        let mut fixture = Fixture::new(&format!("stale-{change}"));
        let (sender, continuations) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        assert!(
            fixture
                .core
                .handle_request(&JsonRpcRequest::new(43_i64, "probe.list", Some(json!({}))))
                .is_deferred()
        );
        let finish = continuations.recv_timeout(Duration::from_secs(4)).unwrap();
        let params = if change == "toolchain.set" {
            json!({"role":"debugAdapter"})
        } else if change == "workspace.open" {
            let other = fixture.root.join("other");
            fs::create_dir_all(&other).unwrap();
            json!({"path":other})
        } else {
            json!({})
        };
        assert!(fixture.call(change, params).error.is_none());
        let response = finish(&mut fixture.core);
        assert_eq!(response.id, Some(json!(43)));
        let error = response
            .error
            .expect("stale query should not update the panel");
        assert_eq!(
            error.code,
            kinein_protocol::JsonRpcErrorCode::InvalidRequest
        );
        assert!(error.message.contains("mudou"), "{change}: {error:?}");
        assert!(response.result.is_none());
    }
}
