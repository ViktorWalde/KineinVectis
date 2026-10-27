//! RP2040 through real dispatch and PTY; every USB interaction is simulated.
use super::Fixture;
use crate::flash::pico::{RESOURCE_KEY, fixture as data};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};
use std::{fs, sync::mpsc, time::Duration};

pub(super) fn fixture(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    let root = &fixture.root;
    fs::write(root.join("CMakeLists.txt"), "project(pico_fixture C)\n").unwrap();
    let build = root.join(".kinein/build");
    let reply = build.join(".cmake/api/v1/reply");
    fs::create_dir_all(&reply).unwrap();
    fs::write(reply.join("codemodel-v2-fixture.json"), serde_json::to_vec(&json!({
        "paths":{"source":root,"build":build},"configurations":[{"name":"","targets":[{"name":"firmware","jsonFile":"target-firmware.json"}]}]
    })).unwrap()).unwrap();
    fs::write(reply.join("target-firmware.json"), serde_json::to_vec(&json!({
        "name":"firmware","id":"firmware::@pico","type":"EXECUTABLE","artifacts":[{"path":"firmware.elf"}]
    })).unwrap()).unwrap();
    fs::write(build.join("firmware.elf"), data::elf()).unwrap();
    fs::write(build.join("firmware a'b.uf2"), data::image()).unwrap();
    fs::write(root.join("flash.release"), "").unwrap();
    let body = format!(
        r"#!/usr/bin/python3
from pathlib import Path
import json,signal,sys,time
R=Path({})
a=sys.argv[1:]
with (R/'pico.args').open('a') as f:f.write(json.dumps(a)+'\n')
mode=(R/'pico.mode').read_text() if (R/'pico.mode').exists() else ''
if a==['version']:
 print('picotool v'+('2.2.0' if mode=='legacy' else '9.0.0' if mode=='version' else '2.3.1')+' (Linux, GNU-15)');raise SystemExit(0)
if a and a[0]=='info':
 assert a==['info','--device','--vid','0x2e8a','--pid','0x0003','--ser','E660583883355B21'],a
 (R/'query.ready').touch()
 while (R/'query.wait').exists():time.sleep(.02)
 if mode=='partial':print('Warning: USB access denied')
 if mode=='exit':raise SystemExit(4)
 if mode=='duplicate':print('Multiple RP-series devices in BOOTSEL mode found:\n')
 print('Device Information\n type: RP2040\n flash size: '+('1024K' if mode=='capacity' else '2048K')+'\n flash id: 0x'+('E660583883355B22' if mode=='id' else 'E660583883355B21'))
 raise SystemExit(0)
assert a and a[0]=='load',a
signal.signal(signal.SIGHUP,signal.SIG_IGN)
(R/'flash.args').write_text(json.dumps(a));(R/'flash.ready').touch()
while not (R/'flash.release').exists():time.sleep(.02)
print('pico-fixture-complete')
",
        serde_json::to_string(&root.display().to_string()).unwrap()
    );
    crate::write_executable(root.join("bin/picotool"), body);
    fixture.core.refresh_tool_registry();
    fixture
}

pub(super) fn selection() -> Value {
    json!({"native":true,"engine":"picotool","target":"firmware","pico":{
        "image":".kinein/build/firmware a'b.uf2","destination":{"chip":"rp2040","serialSelector":"E660583883355B21","flashSizeBytes":2_097_152}}})
}

pub(super) fn proposal(fixture: &mut Fixture) -> Value {
    let response = fixture.rpc("runConfig.flashProposal", selection());
    assert!(response.error.is_none(), "{response:?}");
    response.result.unwrap()
}

#[test]
fn pico_native_save_replay_uses_literal_verified_uf2_without_force_restart_or_serial() {
    for mode in ["", "legacy"] {
        let mut f = fixture(&format!("pico-native-{mode}"));
        fs::write(f.root.join("pico.mode"), mode).unwrap();
        let p = proposal(&mut f);
        assert!(p["flash"]["selection"]["device"].is_null());
        let args = json!([
            "load",
            "--verify",
            "--family",
            "0xe48bff56",
            f.root.join(".kinein/build/firmware a'b.uf2"),
            "--type",
            "uf2",
            "--vid",
            "0x2e8a",
            "--pid",
            "0x0003",
            "--ser",
            "E660583883355B21"
        ]);
        assert_eq!(p["flash"]["args"], args);
        for evidence in ["firmware::@pico", "SHA256", "17 × 256", "0x10000000"] {
            assert!(p["source"].to_string().contains(evidence), "{p}");
        }
        assert!(
            f.rpc(
                "runConfig.save",
                json!({"name":"Pico verified","command":p["command"],"flash":p["flash"]})
            )
            .error
            .is_none()
        );
        let stored: Value =
            serde_json::from_slice(&fs::read(f.root.join(".kinein/runconfigs.json")).unwrap())
                .unwrap();
        assert_eq!(stored["schemaVersion"], 2);
        let started = f.rpc("run.start", json!({}));
        assert!(started.error.is_none(), "{started:?}");
        let id = started.result.unwrap()["terminalId"]
            .as_str()
            .unwrap()
            .to_owned();
        let (_, exit) =
            crate::tests::terminal_run_until_closed(&f.events, &id, Duration::from_secs(5));
        assert_eq!(exit, Some(0));
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(f.root.join("flash.args")).unwrap()).unwrap(),
            args
        );
    }
}

#[test]
fn pico_native_refuses_other_fields_bad_destination_and_unverified_tool_output() {
    let mut f = fixture("pico-native-refusals");
    for (field, value) in [
        ("device", json!(f.port)),
        ("probe", json!("1234:5678:S1")),
        ("firmware", json!("unknown")),
        ("buildDirectory", json!("build")),
        ("flashSizeBytes", json!(123)),
        ("dfu", super::dfu::selection()["dfu"].clone()),
    ] {
        let mut request = selection();
        request[field] = value;
        assert!(
            f.rpc("runConfig.flashProposal", request).error.is_some(),
            "{field}"
        );
    }
    for (field, value) in [
        ("chip", json!("rp2350")),
        ("serialSelector", json!("E660583883355b21")),
        ("serialSelector", json!("FFFFFFFFFFFFFFFF")),
        ("serialSelector", json!("0000000000000000")),
        ("serialSelector", json!("1")),
        ("flashSizeBytes", json!(0)),
        ("flashSizeBytes", json!(33_554_432)),
    ] {
        let mut request = selection();
        request["pico"]["destination"][field] = value;
        assert!(
            f.rpc("runConfig.flashProposal", request).error.is_some(),
            "{field}"
        );
    }
    for engine in ["esptool", "dfu-util", "probe-rs", "idf.py", "platformio"] {
        let mut request = selection();
        request["engine"] = json!(engine);
        assert!(
            f.rpc("runConfig.flashProposal", request).error.is_some(),
            "{engine}"
        );
    }
    for (field, value) in [
        ("native", json!(false)),
        ("target", json!("")),
        ("target", json!(" firmware")),
        ("target", json!("missing")),
    ] {
        let mut request = selection();
        request[field] = value;
        assert!(
            f.rpc("runConfig.flashProposal", request).error.is_some(),
            "{field}"
        );
    }
    for mode in ["version", "partial", "exit", "duplicate", "capacity", "id"] {
        fs::write(f.root.join("pico.mode"), mode).unwrap();
        assert!(
            f.rpc("runConfig.flashProposal", selection())
                .error
                .is_some(),
            "{mode}"
        );
        assert!(
            f.core
                .serial_usage
                .acquire_usb(RESOURCE_KEY, "test", "released", None)
                .is_ok()
        );
    }
    assert!(!f.root.join("flash.ready").exists());
}

#[test]
fn pico_native_replay_refuses_changed_uf2_elf_cmake_tool_or_destination_before_launch() {
    for change in [
        "uf2", "elf", "cmake", "tool", "capacity", "id", "version", "partial",
    ] {
        let mut f = fixture(&format!("pico-replay-{change}"));
        let p = proposal(&mut f);
        let build = f.root.join(".kinein/build");
        match change {
            "uf2" => {
                let mut bytes = data::image();
                bytes[512 + 35] ^= 1;
                fs::write(build.join("firmware a'b.uf2"), bytes).unwrap();
            }
            "elf" => {
                let mut bytes = data::elf();
                bytes[258] ^= 1;
                fs::write(build.join("firmware.elf"), bytes).unwrap();
            }
            "cmake" => {
                let path = build.join(".cmake/api/v1/reply/target-firmware.json");
                let text = fs::read_to_string(&path).unwrap();
                fs::write(path, text.replace("firmware::@pico", "firmware::@new")).unwrap();
            }
            "tool" => {
                crate::write_executable(f.root.join("bin/picotool"), "#!/bin/sh\necho changed\n");
            }
            mode => fs::write(f.root.join("pico.mode"), mode).unwrap(),
        }
        assert!(
            f.rpc("run.start", json!({"flash":p["flash"]}))
                .error
                .is_some(),
            "{change}"
        );
        assert!(!f.root.join("flash.ready").exists(), "{change}");
    }
}

#[test]
fn pico_native_family_lease_blocks_other_devices_and_discovery_but_leaves_other_resources_free() {
    let mut f = fixture("pico-family-lease");
    let p = proposal(&mut f);
    fs::remove_file(f.root.join("flash.release")).unwrap();
    let started = f.rpc("run.start", json!({"flash":p["flash"]}));
    assert!(started.error.is_none(), "{started:?}");
    assert_eq!(
        f.rpc("runConfig.flashProposal", selection())
            .error
            .unwrap()
            .code,
        JsonRpcErrorCode::SerialPortBusy
    );
    assert_eq!(
        f.rpc("picotool.list", json!({})).error.unwrap().code,
        JsonRpcErrorCode::SerialPortBusy
    );
    assert!(
        f.core
            .serial_usage
            .acquire_usb("0483:df11:S1", "test", "other USB", None)
            .is_ok()
    );
    assert!(
        f.core
            .serial_usage
            .acquire_probe("1234:5678:S1", "test", "other probe", None)
            .is_ok()
    );
    assert!(
        f.core
            .serial_usage
            .acquire(f.port.to_str().unwrap(), "test", "other serial", None)
            .is_ok()
    );
    fs::write(f.root.join("flash.release"), "").unwrap();
    let id = started.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) = crate::tests::terminal_run_until_closed(&f.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
    assert!(
        f.core
            .serial_usage
            .acquire_usb(RESOURCE_KEY, "test", "released", None)
            .is_ok()
    );
}

#[test]
fn pico_native_late_file_change_or_workspace_stop_refuses_before_pty() {
    for change in ["uf2", "elf", "tool", "workspace", "stop", "cmake"] {
        let mut f = fixture(&format!("pico-before-pty-{change}"));
        let p = proposal(&mut f);
        let (tx, rx) = mpsc::channel();
        f.core.enable_continuations(tx);
        assert!(
            f.core
                .handle_request(&JsonRpcRequest::new(
                    90_i64,
                    "run.start",
                    Some(json!({"flash":p["flash"]}))
                ))
                .is_deferred()
        );
        let finish = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        match change {
            "uf2" => {
                let path = f.root.join(".kinein/build/firmware a'b.uf2");
                fs::write(path, "changed").unwrap();
            }
            "elf" => {
                fs::write(f.root.join(".kinein/build/firmware.elf"), "changed").unwrap();
            }
            "tool" => {
                crate::write_executable(f.root.join("bin/picotool"), "#!/bin/sh\necho changed\n");
            }
            "cmake" => {
                let path = f
                    .root
                    .join(".kinein/build/.cmake/api/v1/reply/target-firmware.json");
                let text = fs::read_to_string(&path).unwrap();
                fs::write(path, text + " ").unwrap();
            }
            "workspace" => {
                f.rpc("workspace.open", json!({"path":f.root}));
            }
            _ => {
                f.rpc("run.stop", json!({}));
            }
        }
        assert!(finish(&mut f.core).error.is_some(), "{change}");
        assert!(!f.root.join("flash.ready").exists());
        assert!(
            f.core
                .serial_usage
                .acquire_usb(RESOURCE_KEY, "test", "released", None)
                .is_ok()
        );
    }
}
