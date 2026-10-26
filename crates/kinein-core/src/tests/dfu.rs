//! Real DFU dispatch and workspace lifecycle, using only a fake USB tool.
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::mpsc, time::Duration};

struct Fixture {
    core: crate::Core,
    root: PathBuf,
    tool: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kinein-dfu-dispatch-{}-{name}", std::process::id()));
        fs::create_dir_all(root.join("bin")).unwrap();
        let tool = crate::write_executable(
            root.join("bin/dfu-util"),
            "#!/bin/sh\n[ \"$#\" = 1 ] && [ \"$1\" = --list ] || exit 88\nsleep .2\necho 'dfu-util 0.11'\n",
        );
        let core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            root.join("bin"),
        ));
        let mut fixture = Self { core, root, tool };
        assert!(
            fixture
                .call("workspace.open", json!({"path":fixture.root}))
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
fn discovery_is_deferred_and_keeps_the_core_available() {
    let mut fixture = Fixture::new("deferred");
    let (sender, continuations) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(42_i64, "dfu.list", Some(json!({}))))
            .is_deferred()
    );
    assert!(fixture.call("runConfig.list", json!({})).error.is_none());
    let response = continuations.recv_timeout(Duration::from_secs(4)).unwrap()(&mut fixture.core);
    assert_eq!(response.id, Some(json!(42)));
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(response.result.unwrap()["complete"], true);
}

#[test]
fn lifecycle_cancels_discovery_even_when_reopening_the_same_root() {
    for change in ["close", "other", "same", "shutdown"] {
        let mut fixture = Fixture::new(change);
        let (sender, continuations) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        assert!(
            fixture
                .core
                .handle_request(&JsonRpcRequest::new(43_i64, "dfu.list", Some(json!({}))))
                .is_deferred()
        );
        let finish = continuations.recv_timeout(Duration::from_secs(4)).unwrap();
        let response = match change {
            "close" => fixture.call("workspace.close", json!({})),
            "shutdown" => fixture.call("core.shutdown", json!({})),
            "same" => fixture.call("workspace.open", json!({"path":fixture.root})),
            _ => {
                let other = fixture.root.join("other");
                fs::create_dir_all(&other).unwrap();
                fixture.call("workspace.open", json!({"path":other}))
            }
        };
        assert!(response.error.is_none(), "{response:?}");
        let response = finish(&mut fixture.core);
        assert_eq!(
            response.error.unwrap().code,
            JsonRpcErrorCode::InvalidRequest,
            "{change}"
        );
        assert!(response.result.is_none());
    }
}

#[test]
fn latest_query_result_and_stale_completion() {
    let mut fixture = Fixture::new("overlap");
    let (sender, continuations) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    for id in [50_i64, 51] {
        assert!(
            fixture
                .core
                .handle_request(&JsonRpcRequest::new(id, "dfu.list", Some(json!({}))))
                .is_deferred()
        );
    }
    let mut responses = Vec::new();
    for _ in 0..2 {
        responses.push(continuations.recv_timeout(Duration::from_secs(4)).unwrap()(
            &mut fixture.core,
        ));
    }
    let old = responses
        .iter()
        .find(|response| response.id == Some(json!(50)))
        .unwrap();
    let newest = responses
        .iter()
        .find(|response| response.id == Some(json!(51)))
        .unwrap();
    assert_eq!(
        old.error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
    assert!(newest.error.is_none(), "{newest:?}");
    assert_eq!(newest.result.as_ref().unwrap()["complete"], true);
}

#[test]
fn tool_changes_between_query_and_continuation_refuse_the_result() {
    let mut fixture = Fixture::new("tool-change");
    let (sender, continuations) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(44_i64, "dfu.list", Some(json!({}))))
            .is_deferred()
    );
    let finish = continuations.recv_timeout(Duration::from_secs(4)).unwrap();
    // Windows fake executables share a wrapper via hard links. Replace this
    // fixture's node; never overwrite the common wrapper's contents.
    let permissions = fs::metadata(&fixture.tool).unwrap().permissions();
    fs::remove_file(&fixture.tool).unwrap();
    fs::write(&fixture.tool, "replacement fixture executable").unwrap();
    fs::set_permissions(&fixture.tool, permissions).unwrap();
    let response = finish(&mut fixture.core);
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn strict_params_workspace_and_missing_tool_are_distinct_outcomes() {
    let mut fixture = Fixture::new("strict");
    let response = fixture.call("dfu.list", json!({"device":"auto"}));
    assert_eq!(
        response.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    fs::remove_file(&fixture.tool).unwrap();
    let response = fixture.call("dfu.list", json!({}));
    let result = response.result.unwrap();
    assert_eq!(result["toolAvailable"], false);
    assert_eq!(result["complete"], false);
    assert!(result["interfaces"].as_array().unwrap().is_empty());
    assert!(fixture.call("workspace.close", json!({})).error.is_none());
    assert_eq!(
        fixture.call("dfu.list", json!({})).error.unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}
