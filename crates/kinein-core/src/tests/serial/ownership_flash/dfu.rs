//! `DfuSe` through real dispatch and PTY; USB and firmware transfer are simulated.
use super::Fixture;
use kinein_protocol::{DfuDestination, JsonRpcErrorCode};
use serde_json::{Value, json};
use std::{fs, sync::mpsc, time::Duration};

const IMAGE: &[u8] = include_bytes!("../../../flash/dfu/fixtures/valid.dfu");
const LIST: &str = "dfu-util 0.11\nFound DFU: [0483:df11] ver=2200, devnum=2, cfg=1, intf=0, path=\"1-2\", alt=0, name=\"@Internal Flash /0x08000000/04*016Kg,01*064Kg,07*128Kg\", serial=\"FIXTURE-1\"\n";

pub(super) fn fixture(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    let root = &fixture.root;
    fs::write(root.join("firmware a'b.dfu"), IMAGE).unwrap();
    fs::write(root.join("dfu.list"), LIST).unwrap();
    fs::write(root.join("flash.release"), "").unwrap();
    let quoted = crate::platform::program_invocation(&root.display().to_string());
    crate::write_executable(
        root.join("bin/dfu-util"),
        format!(
            "#!/bin/sh\nR={quoted}\ncase \"$1\" in\n--list) : > \"$R/query.ready\"; while [ -f \"$R/query.wait\" ]; do sleep .02; done; cat \"$R/dfu.list\"; exit 0;;\n--version|-V) echo 'dfu-util 0.11'; exit 0;;\n--device) printf '%s\\n' \"$@\" > \"$R/flash.args\"; : > \"$R/flash.ready\"; while [ ! -f \"$R/flash.release\" ]; do sleep .02; done; echo dfu-fixture-complete; exit 0;;\nesac\nexit 88\n"
        ),
    );
    fixture.core.refresh_tool_registry();
    fixture
}

pub(super) fn selection() -> Value {
    json!({"native":true,"engine":"dfu-util","dfu":{"image":"firmware a'b.dfu","destination":{
        "vid":"0483","pid":"df11","serial":"FIXTURE-1","path":"1-2","configuration":1,"interface":0,"alternate":0}}})
}

fn proposal(fixture: &mut Fixture) -> Value {
    let response = fixture.rpc("runConfig.flashProposal", selection());
    assert!(response.error.is_none(), "{response:?}");
    response.result.unwrap()
}

fn destination() -> DfuDestination {
    serde_json::from_value(selection()["dfu"]["destination"].clone()).unwrap()
}

#[test]
fn dfu_native_intent_is_saved_and_runs_literal_argv_without_serial_fallback() {
    let mut fixture = fixture("dfu-native-save-run");
    let proposal = proposal(&mut fixture);
    assert!(proposal["flash"]["selection"]["device"].is_null());
    assert_eq!(
        proposal["flash"]["selection"]["dfu"]["image"],
        json!(fixture.root.join("firmware a'b.dfu"))
    );
    let response = fixture.rpc(
        "runConfig.save",
        json!({"name":"DfuSe", "command":proposal["command"], "flash":proposal["flash"]}),
    );
    assert!(response.error.is_none(), "{response:?}");
    let stored: Value =
        serde_json::from_slice(&fs::read(fixture.root.join(".kinein/runconfigs.json")).unwrap())
            .unwrap();
    assert_eq!(stored["schemaVersion"], 2);
    let response = fixture.rpc("run.start", json!({}));
    assert!(response.error.is_none(), "{response:?}");
    let id = response.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
    let args = fs::read_to_string(fixture.root.join("flash.args")).unwrap();
    assert_eq!(
        args.lines().collect::<Vec<_>>(),
        vec![
            "--device",
            ",0483:df11",
            "--serial",
            ",FIXTURE-1",
            "--path",
            "1-2",
            "--cfg",
            "1",
            "--intf",
            "0",
            "--alt",
            "0",
            "--download",
            fixture.root.join("firmware a'b.dfu").to_str().unwrap()
        ]
    );
}

#[test]
fn dfu_native_refuses_changed_image_tool_or_usb_evidence_before_launch() {
    for change in [
        "image", "deleted", "tool", "path", "release", "memory", "serial", "partial",
    ] {
        let mut fixture = fixture(&format!("dfu-stale-{change}"));
        let intent = proposal(&mut fixture)["flash"].clone();
        match change {
            "image" => {
                let mut bytes = IMAGE.to_vec();
                bytes[293] ^= 1;
                fs::write(fixture.root.join("firmware a'b.dfu"), bytes).unwrap();
            }
            "deleted" => {
                fs::remove_file(fixture.root.join("firmware a'b.dfu")).unwrap();
            }
            "tool" => {
                crate::write_executable(
                    fixture.root.join("bin/dfu-util"),
                    "#!/bin/sh\necho 'dfu-util 0.12'\n",
                );
            }
            "path" => fs::write(fixture.root.join("dfu.list"), LIST.replace("1-2", "1-3")).unwrap(),
            "release" => fs::write(
                fixture.root.join("dfu.list"),
                LIST.replace("ver=2200", "ver=2201"),
            )
            .unwrap(),
            "memory" => fs::write(
                fixture.root.join("dfu.list"),
                LIST.replace("04*016Kg", "04*016Ka"),
            )
            .unwrap(),
            "serial" => fs::write(
                fixture.root.join("dfu.list"),
                LIST.replace("FIXTURE-1", "FIXTURE-2"),
            )
            .unwrap(),
            _ => fs::write(
                fixture.root.join("dfu.list"),
                format!("{LIST}dfu-util: Cannot open DFU device (LIBUSB_ERROR_ACCESS)\n"),
            )
            .unwrap(),
        }
        let response = fixture.rpc("run.start", json!({"flash":intent}));
        assert!(response.error.is_some(), "{change}: {response:?}");
        assert!(!fixture.root.join("flash.args").exists(), "{change}");
    }
}

#[test]
fn dfu_native_refuses_foreign_fields_unsafe_images_and_runtime_or_ambiguous_usb() {
    let mut fixture = fixture("dfu-selection-refusals");
    for (field, value) in [
        ("device", json!(fixture.port)),
        ("probe", json!("0483:374b:S1")),
        ("target", json!("app")),
        ("buildDirectory", json!("build")),
        ("firmware", json!("missing")),
        ("flashSizeBytes", json!(123)),
    ] {
        let mut request = selection();
        request[field] = value;
        assert!(
            fixture
                .rpc("runConfig.flashProposal", request)
                .error
                .is_some(),
            "{field}"
        );
    }
    for native in [false, true] {
        let mut request = selection();
        request["native"] = json!(native);
        request["engine"] = json!("esptool");
        assert_eq!(
            fixture
                .rpc("runConfig.flashProposal", request)
                .error
                .unwrap()
                .code,
            JsonRpcErrorCode::InvalidParams
        );
    }
    for output in [
        LIST.replace("Found DFU:", "Found Runtime:"),
        LIST.replace("alt=0", "alt=1"),
        format!(
            "{LIST}{}",
            LIST.replace("devnum=2", "devnum=3").replace("1-2", "1-3")
        ),
    ] {
        fs::write(fixture.root.join("dfu.list"), output).unwrap();
        assert!(
            fixture
                .rpc("runConfig.flashProposal", selection())
                .error
                .is_some()
        );
    }
    fs::write(fixture.root.join("dfu.list"), LIST).unwrap();
    let mut request = selection();
    request["dfu"]["image"] = json!(fixture.port);
    assert!(
        fixture
            .rpc("runConfig.flashProposal", request)
            .error
            .is_some()
    );
    let outside = fixture.root.with_extension("external.dfu");
    fs::write(&outside, IMAGE).unwrap();
    let mut request = selection();
    request["dfu"]["image"] = json!(outside);
    assert!(
        fixture
            .rpc("runConfig.flashProposal", request)
            .error
            .is_some()
    );
    std::os::unix::fs::symlink(&outside, fixture.root.join("outside-alias.dfu")).unwrap();
    let mut request = selection();
    request["dfu"]["image"] = json!("outside-alias.dfu");
    assert!(
        fixture
            .rpc("runConfig.flashProposal", request)
            .error
            .is_some()
    );
    fs::remove_file(outside).unwrap();
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn dfu_usb_reservation_conflicts_across_alternatives_and_with_automatic_selection() {
    let mut fixture = fixture("dfu-usb-busy");
    let intent = proposal(&mut fixture)["flash"].clone();
    let key = crate::flash::dfu::resource_key(&destination());
    let mut other = destination();
    other.alternate = 1;
    other.path = "1-4".into();
    assert_eq!(key, crate::flash::dfu::resource_key(&other));
    let lease = fixture
        .core
        .serial_usage
        .acquire_usb(&key, "test", "USB fixture", Some(&fixture.root))
        .unwrap();
    let response = fixture.rpc("run.start", json!({"flash":intent}));
    let error = response.error.unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert_eq!(error.details.unwrap()["scope"], "usb");
    assert!(!fixture.root.join("flash.args").exists());
    assert!(
        fixture
            .core
            .serial_usage
            .acquire_automatic("test", "auto", None)
            .is_err()
    );
    assert!(
        fixture
            .core
            .serial_usage
            .acquire(&fixture.port.display().to_string(), "test", "serial", None)
            .is_ok()
    );
    assert!(
        fixture
            .core
            .serial_usage
            .acquire_probe("0483:374b:S2", "test", "probe", None)
            .is_ok()
    );
    drop(lease);
    let automatic = fixture
        .core
        .serial_usage
        .acquire_automatic("test", "auto", None)
        .unwrap();
    assert!(
        fixture
            .core
            .serial_usage
            .acquire_usb(&key, "test", "USB", None)
            .is_err()
    );
    drop(automatic);
    assert!(
        fixture
            .core
            .serial_usage
            .acquire_usb(&key, "test", "USB", None)
            .is_ok()
    );
}

#[test]
fn dfu_pending_preparation_is_cancelled_by_stop_or_same_workspace_activation() {
    for change in ["stop", "same", "close"] {
        let mut fixture = fixture(&format!("dfu-cancel-{change}"));
        let intent = proposal(&mut fixture)["flash"].clone();
        fs::remove_file(fixture.root.join("query.ready")).unwrap();
        fs::write(fixture.root.join("query.wait"), "").unwrap();
        let (sender, continuation) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        let started = fixture.rpc("run.start", json!({"flash":intent}));
        assert_eq!(started.result.unwrap()["kineinDeferred"], true);
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while !fixture.root.join("query.ready").exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(fixture.root.join("query.ready").exists());
        assert!(fixture.rpc("runConfig.list", json!({})).error.is_none());
        assert!(
            fixture
                .rpc("run.start", json!({"command":"echo no-fallback"}))
                .error
                .is_some()
        );
        let response = match change {
            "stop" => fixture.rpc("run.stop", json!({})),
            "same" => fixture.rpc("workspace.open", json!({"path":fixture.root})),
            _ => fixture.rpc("workspace.close", json!({})),
        };
        assert!(response.error.is_none(), "{response:?}");
        assert!(
            continuation.recv_timeout(Duration::from_secs(3)).unwrap()(&mut fixture.core)
                .error
                .is_some()
        );
        assert!(!fixture.root.join("flash.args").exists());
        assert!(
            fixture
                .core
                .serial_usage
                .acquire_usb(
                    &crate::flash::dfu::resource_key(&destination()),
                    "test",
                    "released",
                    None
                )
                .is_ok()
        );
    }
}

#[test]
fn dfu_file_changed_after_worker_validation_is_refused_before_pty() {
    let mut fixture = fixture("dfu-last-check");
    let intent = proposal(&mut fixture)["flash"].clone();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    fixture.rpc("run.start", json!({"flash":intent}));
    let finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
    fs::write(
        fixture.root.join("firmware a'b.dfu"),
        "replaced after validation",
    )
    .unwrap();
    let response = finish(&mut fixture.core);
    assert!(response.error.is_some());
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn dfu_usb_reservation_is_shared_by_persisted_registries() {
    let fixture = fixture("dfu-persisted-reservation");
    let mut first = crate::serial::ownership::Registry::default();
    let mut second = crate::serial::ownership::Registry::default();
    first.use_directory(fixture.root.join("usage"));
    second.use_directory(fixture.root.join("usage"));
    let key = crate::flash::dfu::resource_key(&destination());
    let lease = first
        .acquire_usb(&key, "test", "first USB", Some(&fixture.root))
        .unwrap();
    let error = second
        .acquire_usb(&key, "test", "second USB", Some(&fixture.root))
        .unwrap_err();
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert_eq!(error.details.unwrap()["scope"], "usb");
    assert!(
        second
            .acquire_usb("0483:df11:OTHER", "test", "other", None)
            .is_ok()
    );
    drop(lease);
    assert!(second.acquire_usb(&key, "test", "released", None).is_ok());
}
