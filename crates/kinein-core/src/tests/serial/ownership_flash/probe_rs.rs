//! Native probe-rs through real dispatch: `CMake` reply fixture, fake 0.32 CLI
//! (`mi meta`, `chip info`, `list`, `download`), real PTY and reservation.
use super::Fixture;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcResponse};
use serde_json::{Value, json};
use std::{fs, path::Path, sync::mpsc, thread, time::Duration};

pub(super) const PROBE: &str = "0483:374b:066DFF485153";
pub(super) const LISTED: &str = "The following debug probes were found:\n[0]: STLink V2-1 -- 0483:374b:066DFF485153 (ST-LINK)\n";

fn elf(machine: u16) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[..4].copy_from_slice(b"\x7fELF");
    bytes[4..7].copy_from_slice(&[1, 1, 1]);
    bytes[18..20].copy_from_slice(&machine.to_le_bytes());
    bytes
}

/// The canonical build's file-api reply: an executable and a library.
fn reply(root: &Path, firmware_artifacts: &[&str]) {
    let build = root.join(".kinein/build");
    let reply = build.join(".cmake/api/v1/reply");
    fs::create_dir_all(&reply).unwrap();
    let write = |name: &str, value: &Value| {
        fs::write(reply.join(name), serde_json::to_vec(value).unwrap()).unwrap();
    };
    write(
        "codemodel-v2-fixture.json",
        &json!({"paths":{"source":root, "build":build}, "configurations":[{"name":"", "targets":[
            {"name":"firmware", "jsonFile":"target-firmware.json"},
            {"name":"support", "jsonFile":"target-support.json"}
        ]}]}),
    );
    write(
        "target-firmware.json",
        &json!({"name":"firmware", "id":"firmware::@fixture", "type":"EXECUTABLE",
            "artifacts": firmware_artifacts.iter().map(|path| json!({"path":path})).collect::<Vec<_>>()}),
    );
    write(
        "target-support.json",
        &json!({"name":"support", "id":"support::@fixture", "type":"STATIC_LIBRARY",
            "artifacts":[{"path":"libsupport.a"}]}),
    );
    fs::write(build.join("firmware.elf"), elf(0x28)).unwrap();
    fs::write(build.join("libsupport.a"), "!<arch>\n").unwrap();
}

pub(super) fn fixture(name: &str) -> Fixture {
    let mut fixture = Fixture::new(name);
    let root = fixture.root.clone();
    reply(&root, &["firmware.elf"]);
    fs::write(
        root.join("probe.meta"),
        r#"{"version":"0.32.0","commit":"fixture","arch":"x86_64","os":"linux"}"#,
    )
    .unwrap();
    fs::write(root.join("probe.list"), LISTED).unwrap();
    let quoted = crate::platform::program_invocation(&root.display().to_string());
    crate::write_executable(
        root.join("bin/probe-rs"),
        format!(
            "#!/bin/sh\nR={quoted}\ncase \"$1\" in\n\
             --version|-V) echo 'probe-rs 0.32.0 (git commit: fixture)'; exit 0;;\n\
             mi) : > \"$R/query.ready\"; while [ -f \"$R/query.wait\" ]; do sleep .02; done; cat \"$R/probe.meta\"; exit 0;;\n\
             chip) if [ \"$3\" = STM32F411RETx ]; then printf 'STM32F411RETx\\nCores (1):\\n    - main (Armv7em)\\nNVM: 0x08000000..0x08080000 (512.0 KiB)\\n'; exit 0; fi\n\
             echo \"Error: The requested chip '$3' was not found in the list of known targets.\" >&2; exit 1;;\n\
             list) cat \"$R/probe.list\"; exit 0;;\n\
             download) printf '%s\\n' \"$@\" > \"$R/probe.args\"; env | grep '^PROBE_RS_' > \"$R/probe.env\"; echo download-fixture-complete; exit 0;;\n\
             dap-server) exec python3 \"$R/fake_dap.py\" \"$R\";;\n\
             esac\nexit 2\n"
        ),
    );
    fixture.core.refresh_tool_registry();
    for (method, params) in [
        (
            "toolchain.set",
            json!({"role":"debugAdapter", "id":"probe-rs"}),
        ),
        ("toolchain.setKit", json!({"chip":"STM32F411RETx"})),
    ] {
        let response = fixture.rpc(method, params);
        assert!(response.error.is_none(), "{method}: {response:?}");
    }
    fixture
}

pub(super) fn request() -> Value {
    json!({"native":true, "engine":"probe-rs", "target":"firmware", "probe":PROBE})
}

pub(super) fn propose(fixture: &mut Fixture, params: Value) -> JsonRpcResponse {
    fixture.rpc("runConfig.flashProposal", params)
}

fn refusal(fixture: &mut Fixture, params: Value) -> (JsonRpcErrorCode, String) {
    let response = propose(fixture, params);
    let error = response.error.expect("refusal expected");
    (error.code, error.message)
}

#[test]
fn probe_rs_intent_binds_target_probe_and_launches_its_literal_arguments() {
    let mut fixture = fixture("probe-rs-native-run");
    let response = propose(&mut fixture, request());
    assert!(response.error.is_none(), "{response:?}");
    let proposal = response.result.unwrap();
    let intent = proposal["flash"].clone();
    let artifact = fixture.root.join(".kinein/build/firmware.elf");
    let expected = [
        "download",
        "--chip",
        "STM32F411RETx",
        "--probe",
        PROBE,
        "--binary-format",
        "elf",
        "--verify",
        &artifact.display().to_string(),
    ];
    assert_eq!(intent["args"], json!(expected), "{proposal}");
    assert_eq!(intent["selection"]["target"], "firmware");
    assert!(intent["selection"].get("device").is_none());
    let source = proposal["source"].to_string();
    for evidence in [
        "firmware::@fixture",
        "Armv7em",
        "STLink V2-1",
        "PROBE_RS_",
        ".probe-rs.*",
    ] {
        assert!(source.contains(evidence), "{evidence}: {source}");
    }
    assert!(proposal["warnings"].to_string().contains("--reset"));
    let response = fixture.rpc("run.start", json!({"flash":intent}));
    assert!(response.error.is_none(), "{response:?}");
    let id = response.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
    let launched = fs::read_to_string(fixture.root.join("probe.args")).unwrap();
    assert_eq!(launched.lines().collect::<Vec<_>>(), expected);
}

#[test]
fn probe_rs_refuses_every_missing_or_conflicting_piece_of_evidence() {
    let mut fixture = fixture("probe-rs-native-refusals");
    let root = fixture.root.clone();
    for (params, code) in [
        (
            json!({"engine":"probe-rs", "target":"firmware", "probe":PROBE}),
            JsonRpcErrorCode::InvalidParams,
        ),
        (
            json!({"native":true, "engine":"probe-rs", "probe":PROBE}),
            JsonRpcErrorCode::InvalidParams,
        ),
        (
            json!({"native":true, "engine":"probe-rs", "target":"firmware", "probe":"0483:374b"}),
            JsonRpcErrorCode::InvalidParams,
        ),
        (
            json!({"native":true, "engine":"probe-rs", "target":"firmware", "probe":PROBE, "device":fixture.port}),
            JsonRpcErrorCode::InvalidParams,
        ),
        (
            json!({"native":true, "engine":"esptool", "probe":PROBE, "device":fixture.port}),
            JsonRpcErrorCode::InvalidParams,
        ),
    ] {
        assert_eq!(refusal(&mut fixture, params.clone()).0, code, "{params}");
    }
    let refused = |fixture: &mut Fixture, params: Value, needle: &str| {
        let (code, message) = refusal(fixture, params);
        assert_eq!(code, JsonRpcErrorCode::InvalidRequest, "{message}");
        assert!(message.contains(needle), "{needle}: {message}");
    };
    let with_target =
        |target: &str| json!({"native":true, "engine":"probe-rs", "target":target, "probe":PROBE});
    refused(&mut fixture, with_target("support"), "não um executável");
    refused(&mut fixture, with_target("missing"), "não existe");
    reply(&root, &["firmware.elf", "firmware.map"]);
    refused(&mut fixture, request(), "exatamente um");
    reply(&root, &["firmware.elf"]);
    fs::write(root.join(".kinein/build/firmware.elf"), elf(0xF3)).unwrap();
    refused(&mut fixture, request(), "RISC-V");
    fs::write(root.join(".kinein/build/firmware.elf"), "not an elf").unwrap();
    refused(&mut fixture, request(), "não é ELF");
    fs::remove_file(root.join(".kinein/build/firmware.elf")).unwrap();
    refused(&mut fixture, request(), "firmware.elf");
    reply(&root, &["firmware.elf"]);
    for (listing, needle) in [
        ("No debug probes were found.\n", "não está na listagem"),
        (
            "[0]: STLink V2-1 -- 0483:374b:066DFF485153 (ST-LINK) (inaccessible)\n",
            "udev",
        ),
        (
            &format!("{LISTED}[1]: Clone -- 0483:374b:066DFF485153 (ST-LINK)\n") as &str,
            "2 sondas",
        ),
        (
            "[0]: STLink V2-1 -- 0483:374b:066DFF485153\n",
            "fora do formato",
        ),
    ] {
        fs::write(root.join("probe.list"), listing).unwrap();
        refused(&mut fixture, request(), needle);
    }
    fs::write(root.join("probe.list"), LISTED).unwrap();
    fs::write(root.join("probe.meta"), r#"{"version":"0.31.0"}"#).unwrap();
    refused(&mut fixture, request(), "0.32.x");
    fs::write(
        root.join("probe.meta"),
        r#"{"version":"0.32.0","commit":"fixture","arch":"x86_64","os":"linux"}"#,
    )
    .unwrap();
    fs::write(
        root.join(".probe-rs.toml"),
        "[presets.default]\nchip-erase = true\n",
    )
    .unwrap();
    refused(&mut fixture, request(), "presets");
    fs::remove_file(root.join(".probe-rs.toml")).unwrap();
    assert!(
        fixture
            .rpc("toolchain.setKit", json!({"chip":"NOTACHIP"}))
            .error
            .is_none()
    );
    refused(&mut fixture, request(), "NOTACHIP");
    // The same chip, but only deduced from CubeMX: probe-rs suggests, the user confirms.
    fs::write(root.join("board.ioc"), "Mcu.Name=STM32F411RETx\n").unwrap();
    assert!(
        fixture
            .rpc("toolchain.setKit", json!({"chip":""}))
            .error
            .is_none()
    );
    let model = fixture.core.compute_project_model(&root);
    assert_eq!(model.target.chip.as_deref(), Some("STM32F411RETx"));
    refused(&mut fixture, request(), "chip fixado no kit");
    assert!(
        !root.join("probe.args").exists(),
        "a refusal launched probe-rs"
    );
}

#[test]
fn launch_checks_artifact_and_probe_again_and_reserves_the_probe() {
    let mut fixture = fixture("probe-rs-native-revalidation");
    let root = fixture.root.clone();
    let intent = propose(&mut fixture, request()).result.unwrap()["flash"].clone();
    fs::write(
        root.join(".kinein/build/firmware.elf"),
        [elf(0x28), vec![1]].concat(),
    )
    .unwrap();
    assert!(
        fixture
            .rpc("run.start", json!({"flash":intent}))
            .error
            .is_some()
    );
    fs::write(root.join(".kinein/build/firmware.elf"), elf(0x28)).unwrap();
    let intent = propose(&mut fixture, request()).result.unwrap()["flash"].clone();
    fs::write(root.join("probe.list"), "No debug probes were found.\n").unwrap();
    assert!(
        fixture
            .rpc("run.start", json!({"flash":intent}))
            .error
            .is_some()
    );
    fs::write(root.join("probe.list"), LISTED).unwrap();
    let held = fixture
        .core
        .serial_usage
        .acquire_probe(PROBE, "debug.start", "Depuração", Some(&root))
        .unwrap();
    let busy = fixture.rpc("run.start", json!({"flash":intent}));
    let error = busy.error.unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert!(error.message.contains("a sonda"), "{error:?}");
    assert_eq!(error.details.unwrap()["scope"], "probe");
    drop(held);
    // An explicit serial port (the probe's VCP monitor) does not block it.
    let port = fixture.port.display().to_string();
    let monitor = fixture
        .core
        .serial_usage
        .acquire(&port, "serial.monitor", "Monitor", Some(&root))
        .unwrap();
    let response = fixture.rpc("run.start", json!({"flash":intent}));
    assert!(response.error.is_none(), "{response:?}");
    let id = response.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
    drop(monitor);
    assert!(root.join("probe.args").exists());
}

#[test]
fn stopping_probe_rs_preparation_cancels_the_query_without_launching() {
    let mut fixture = fixture("probe-rs-native-cancel");
    let root = fixture.root.clone();
    let intent = propose(&mut fixture, request()).result.unwrap()["flash"].clone();
    fs::remove_file(root.join("query.ready")).unwrap();
    fs::write(root.join("query.wait"), "").unwrap();
    let (sender, continuation) = mpsc::channel();
    fixture.core.enable_continuations(sender);
    let started = fixture.rpc("run.start", json!({"flash":intent}));
    assert_eq!(started.result.unwrap()["kineinDeferred"], true);
    for _ in 0..300 {
        if root.join("query.ready").exists() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(root.join("query.ready").exists());
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    let finish = continuation
        .recv_timeout(Duration::from_secs(2))
        .expect("cancel must stop the version query before its deadline");
    let response = finish(&mut fixture.core);
    assert!(response.error.unwrap().message.contains("cancelada"));
    fs::remove_file(root.join("query.wait")).unwrap();
    assert!(!root.join("probe.args").exists());
}
