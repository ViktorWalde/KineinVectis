//! Real dispatch and reservations; every picotool/USB operation is simulated.
use super::Fixture;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, atomic::AtomicBool, mpsc},
    time::{Duration, Instant},
};

fn fixture(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    let root = &fixture.root;
    let script = format!(
        r"#!/usr/bin/python3
from pathlib import Path
import json,sys,time
R=Path({})
args=sys.argv[1:]
with (R/'pico.args').open('a') as f: f.write(json.dumps(args)+'\n')
mode=(R/'pico.mode').read_text() if (R/'pico.mode').exists() else ''
if args==['version']:
 (R/'pico.ready').touch()
 while (R/'pico.hold').exists(): time.sleep(.02)
 if mode=='overflow': print('x'*70000);raise SystemExit(0)
 print('picotool v'+('9.0.0' if mode=='version' else '2.3.1')+' (Linux, GNU-15)');raise SystemExit(0)
base=['info','--device','--vid','0x2e8a','--pid','0x0003']
assert args==base or args==base+['--ser','E660583883355B21'],args
if mode=='exit': print('No accessible RP-series devices in BOOTSEL mode were found.');raise SystemExit(4)
if mode=='warning': print('Warning: USB access denied')
size='1024K' if mode=='changed' and '--ser' in args else '2048K'
print('Device Information\n type: RP2040\n flash size: '+size+'\n flash id: 0xE660583883355B21')
",
        serde_json::to_string(&root.display().to_string()).unwrap()
    );
    crate::write_executable(root.join("bin/picotool"), script);
    fixture.core.refresh_tool_registry();
    let _ = fs::remove_file(root.join("pico.args"));
    fixture
}

fn query(fixture: &mut Fixture) -> kinein_protocol::JsonRpcResponse {
    fixture.rpc("picotool.list", json!({}))
}

#[test]
fn picotool_query_confirms_literal_selector_without_enabling_native_flash() {
    let mut fixture = fixture("pico-query");
    let response = query(&mut fixture);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.unwrap();
    assert_eq!(result["complete"], true);
    assert_eq!(result["version"], "2.3.1");
    assert_eq!(
        result["devices"],
        json!([{"chip":"rp2040","serialSelector":"E660583883355B21","flashSizeBytes":2_097_152}])
    );
    let calls: Vec<Value> = fs::read_to_string(fixture.root.join("pico.args"))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(
        calls,
        vec![
            json!(["version"]),
            json!(["info", "--device", "--vid", "0x2e8a", "--pid", "0x0003"]),
            json!([
                "info",
                "--device",
                "--vid",
                "0x2e8a",
                "--pid",
                "0x0003",
                "--ser",
                "E660583883355B21"
            ])
        ]
    );
    assert!(
        fixture
            .rpc(
                "runConfig.flashProposal",
                json!({"native":true,"engine":"picotool"})
            )
            .error
            .is_some()
    );
    assert_eq!(
        query(&mut fixture).result.unwrap()["complete"],
        true,
        "lease released after completion"
    );
}

#[test]
fn picotool_partial_confirmation_version_and_access_failures_publish_no_devices() {
    for mode in ["changed", "version", "warning", "exit", "overflow"] {
        let mut fixture = fixture(&format!("pico-{mode}"));
        fs::write(fixture.root.join("pico.mode"), mode).unwrap();
        let result = query(&mut fixture).result.unwrap();
        assert_eq!(result["complete"], false, "{mode}: {result}");
        assert_eq!(result["devices"], json!([]));
        assert_eq!(result["toolAvailable"], true);
        assert!(result["rawOutput"].as_str().unwrap().len() <= 65536);
        assert!(result["hint"].is_string());
        assert!(
            fixture
                .core
                .serial_usage
                .acquire_automatic("test", "released", None)
                .is_ok()
        );
    }
}

#[test]
fn picotool_query_is_deferred_and_reserves_managed_channels_until_done() {
    let mut fixture = fixture("pico-deferred");
    fs::write(fixture.root.join("pico.hold"), "").unwrap();
    let (sender, receiver) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(
                41_i64,
                "picotool.list",
                Some(json!({}))
            ))
            .is_deferred()
    );
    assert!(fixture.rpc("runConfig.list", json!({})).error.is_none());
    assert!(
        fixture
            .core
            .serial_usage
            .acquire(fixture.port.to_str().unwrap(), "test", "busy", None)
            .is_err()
    );
    fs::remove_file(fixture.root.join("pico.hold")).unwrap();
    let result = receiver.recv_timeout(Duration::from_secs(3)).unwrap()(&mut fixture.core);
    assert!(result.error.is_none(), "{result:?}");
    assert!(
        fixture
            .core
            .serial_usage
            .acquire(fixture.port.to_str().unwrap(), "test", "released", None)
            .is_ok()
    );
}

#[test]
fn newest_picotool_query_waits_through_two_cancellations_and_old_results_are_stale() {
    let mut fixture = fixture("pico-latest");
    fs::write(fixture.root.join("pico.hold"), "").unwrap();
    let (sender, receiver) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(
                42_i64,
                "picotool.list",
                Some(json!({}))
            ))
            .is_deferred()
    );
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(
                43_i64,
                "picotool.list",
                Some(json!({}))
            ))
            .is_deferred()
    );
    assert!(
        fixture
            .core
            .handle_request(&JsonRpcRequest::new(
                44_i64,
                "picotool.list",
                Some(json!({}))
            ))
            .is_deferred()
    );
    fs::remove_file(fixture.root.join("pico.hold")).unwrap();
    for _ in 0..3 {
        let finish = receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        let result = finish(&mut fixture.core);
        if result.id == Some(json!(44)) {
            assert!(result.error.is_none(), "{result:?}");
            assert_eq!(result.result.unwrap()["complete"], true);
        } else {
            assert_eq!(result.error.unwrap().code, JsonRpcErrorCode::InvalidRequest);
        }
    }
    assert!(
        fixture
            .core
            .serial_usage
            .acquire_automatic("test", "released", None)
            .is_ok()
    );
}

#[test]
fn picotool_lifecycle_and_executable_changes_invalidate_queued_results() {
    for change in ["same", "close", "other", "shutdown", "tool"] {
        let mut fixture = fixture(&format!("pico-stale-{change}"));
        let (sender, receiver) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        assert!(
            fixture
                .core
                .handle_request(&JsonRpcRequest::new(
                    44_i64,
                    "picotool.list",
                    Some(json!({}))
                ))
                .is_deferred()
        );
        let finish = receiver.recv_timeout(Duration::from_secs(3)).unwrap();
        match change {
            "same" => {
                fixture.rpc("workspace.open", json!({"path":fixture.root}));
            }
            "close" => {
                fixture.rpc("workspace.close", json!({}));
            }
            "other" => {
                let other = fixture.root.join("other");
                fs::create_dir(&other).unwrap();
                fixture.rpc("workspace.open", json!({"path":other}));
            }
            "shutdown" => {
                fixture.rpc("core.shutdown", json!({}));
            }
            _ => {
                crate::write_executable(
                    fixture.root.join("bin/picotool"),
                    "#!/bin/sh\necho changed\n",
                );
            }
        }
        assert_eq!(
            finish(&mut fixture.core).error.unwrap().code,
            JsonRpcErrorCode::InvalidRequest,
            "{change}"
        );
        assert!(
            fixture
                .core
                .serial_usage
                .acquire_automatic("test", "released", None)
                .is_ok()
        );
    }
}

#[test]
fn picotool_deadline_cancellation_busy_missing_tool_and_strict_params_are_distinct() {
    let mut fixture = fixture("pico-bounds");
    assert_eq!(
        fixture
            .rpc("picotool.list", json!({"force":true}))
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::InvalidParams
    );
    let lease = fixture
        .core
        .serial_usage
        .acquire(fixture.port.to_str().unwrap(), "test", "occupied", None)
        .unwrap();
    assert_eq!(
        query(&mut fixture).error.unwrap().code,
        JsonRpcErrorCode::SerialPortBusy
    );
    drop(lease);
    fs::write(fixture.root.join("pico.hold"), "").unwrap();
    let tool = fixture.root.join("bin/picotool");
    let result = crate::picotool::list(
        Some(&tool),
        Instant::now() + Duration::from_millis(80),
        &Arc::new(AtomicBool::new(false)),
        None,
    );
    assert!(!result.complete && result.tool_available && result.devices.is_empty());
    let result = crate::picotool::list(
        Some(&tool),
        Instant::now() + Duration::from_secs(1),
        &Arc::new(AtomicBool::new(true)),
        None,
    );
    assert!(!result.complete && result.hint.unwrap().contains("cancelada"));
    fs::remove_file(tool).unwrap();
    assert_eq!(query(&mut fixture).result.unwrap()["toolAvailable"], false);
}
