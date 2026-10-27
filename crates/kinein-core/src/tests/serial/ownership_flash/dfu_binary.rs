//! Native BIN/ELF proof through the real dispatcher and the existing fake USB tool.
use super::{Fixture, dfu};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, sync::mpsc, time::Duration};

const ELF: &[u8] = include_bytes!("../../../flash/dfu/fixtures/cmake-bin.elf");
const BIN: &[u8] = include_bytes!("../../../flash/dfu/fixtures/cmake-bin.bin");

fn fixture(name: &str) -> Fixture {
    let fixture = dfu::fixture(name);
    let root = &fixture.root;
    let build = root.join(".kinein/build");
    let reply = build.join(".cmake/api/v1/reply");
    fs::create_dir_all(&reply).unwrap();
    fs::write(reply.join("codemodel-v2-fixture.json"), serde_json::to_vec(&json!({
        "paths":{"source":root,"build":build},"configurations":[{"name":"","targets":[{"name":"firmware","jsonFile":"target-firmware.json"}]}]
    })).unwrap()).unwrap();
    fs::write(reply.join("target-firmware.json"), serde_json::to_vec(&json!({
        "name":"firmware","id":"firmware::@bin","type":"EXECUTABLE","artifacts":[{"path":"firmware.elf"}]
    })).unwrap()).unwrap();
    fs::write(build.join("firmware.elf"), ELF).unwrap();
    fs::write(build.join("firmware a'b.bin"), BIN).unwrap();
    fixture
}

fn selection() -> Value {
    let mut value = dfu::selection();
    value["dfu"]["image"] = json!(".kinein/build/firmware a'b.bin");
    value["dfu"]["binaryTarget"] = json!("firmware");
    value
}

fn proposal(fixture: &mut Fixture) -> Value {
    let response = fixture.rpc("runConfig.flashProposal", selection());
    assert!(response.error.is_none(), "{response:?}");
    response.result.unwrap()
}

fn source(fixture: &Fixture, name: &str) -> PathBuf {
    fixture.root.join(".kinein/build").join(name)
}

#[test]
fn dfu_bin_cmake_intent_saves_replays_and_uses_the_verified_load_address() {
    let mut fixture = fixture("dfu-bin-save-run");
    let proposed = proposal(&mut fixture);
    let evidence = proposed["source"].to_string();
    for value in [
        "firmware::@bin",
        "firmware.elf",
        "SHA256",
        "0x08004000",
        "32 bytes",
    ] {
        assert!(evidence.contains(value), "{evidence}");
    }
    assert_eq!(
        proposed["flash"]["selection"]["dfu"]["binaryTarget"],
        "firmware"
    );
    let args = proposed["flash"]["args"].as_array().unwrap();
    assert_eq!(args[12], "--dfuse-address");
    assert_eq!(args[13], "0x08004000");
    assert_eq!(args[14], "--download");
    assert!(fixture.rpc("runConfig.save",json!({"name":"Verified BIN","command":proposed["command"],"flash":proposed["flash"]})).error.is_none());
    let response = fixture.rpc("run.start", json!({}));
    assert!(response.error.is_none(), "{response:?}");
    let id = response.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
    let launched = fs::read_to_string(fixture.root.join("flash.args")).unwrap();
    let expected: Vec<_> = args.iter().map(|arg| arg.as_str().unwrap()).collect();
    assert_eq!(launched.lines().collect::<Vec<_>>(), expected);
}

#[test]
fn dfu_bin_refuses_unproven_or_changed_content_and_cmake_origin() {
    for change in [
        "bin",
        "elf",
        "model",
        "target",
        "outside",
        "no-target",
        "multi-config",
        "multi-artifact",
        "wrong-root",
        "architecture",
    ] {
        let mut fixture = fixture(&format!("dfu-bin-refuse-{change}"));
        let mut params = selection();
        let reply = source(&fixture, ".cmake/api/v1/reply");
        match change {
            "bin" => {
                let mut bytes = BIN.to_vec();
                bytes[31] ^= 1;
                fs::write(source(&fixture, "firmware a'b.bin"), bytes).unwrap();
            }
            "elf" => {
                let mut bytes = ELF.to_vec();
                bytes[0] = 0;
                fs::write(source(&fixture, "firmware.elf"), bytes).unwrap();
            }
            "model" => fs::remove_file(reply.join("codemodel-v2-fixture.json")).unwrap(),
            "target" => params["dfu"]["binaryTarget"] = json!("missing"),
            "outside" => {
                fs::write(fixture.root.join("copy.bin"), BIN).unwrap();
                params["dfu"]["image"] = json!("copy.bin");
            }
            "no-target" => params["dfu"]["binaryTarget"] = json!(""),
            "architecture" => {
                let mut bytes = ELF.to_vec();
                bytes[18] = 0x3e;
                fs::write(source(&fixture, "firmware.elf"), bytes).unwrap();
            }
            _ => {
                let path = reply.join(if change == "multi-artifact" {
                    "target-firmware.json"
                } else {
                    "codemodel-v2-fixture.json"
                });
                let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                match change {
                    "multi-config" => {
                        value["configurations"]
                            .as_array_mut()
                            .unwrap()
                            .push(json!({"targets":[]}));
                    }
                    "multi-artifact" => {
                        value["artifacts"]
                            .as_array_mut()
                            .unwrap()
                            .push(json!({"path":"extra.elf"}));
                    }
                    _ => value["paths"]["source"] = json!(fixture.root.parent().unwrap()),
                }
                fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
        }
        let response = fixture.rpc("runConfig.flashProposal", params);
        assert!(response.error.is_some(), "{change}: {response:?}");
        assert!(!fixture.root.join("flash.args").exists());
    }
}

#[test]
fn dfu_bin_rechecks_elf_and_reply_during_replay_and_before_pty() {
    for change in ["elf-before", "reply-before", "elf-late", "reply-late"] {
        let mut fixture = fixture(&format!("dfu-bin-stale-{change}"));
        let intent = proposal(&mut fixture)["flash"].clone();
        let path = if change.starts_with("elf") {
            source(&fixture, "firmware.elf")
        } else {
            source(&fixture, ".cmake/api/v1/reply/target-firmware.json")
        };
        let mut bytes = fs::read(&path).unwrap();
        // ELF non-loaded bytes and JSON whitespace preserve content checks,
        // but still invalidate the snapshot chosen in the original preview.
        bytes.extend(b" ");
        let (sender, continuation) = mpsc::channel();
        fixture.core.enable_continuations(sender);
        if change.ends_with("before") {
            fs::write(&path, &bytes).unwrap();
        }
        fixture.rpc("run.start", json!({"flash":intent}));
        let finish = continuation.recv_timeout(Duration::from_secs(3)).unwrap();
        if change.ends_with("late") {
            fs::write(&path, &bytes).unwrap();
        }
        assert!(finish(&mut fixture.core).error.is_some(), "{change}");
        assert!(!fixture.root.join("flash.args").exists());
    }
}

#[test]
fn dfu_bin_metadata_and_artifacts_never_block_on_fifo_or_unbounded_files() {
    for name in [
        "firmware.elf",
        "firmware a'b.bin",
        ".cmake/api/v1/reply/target-firmware.json",
    ] {
        let mut fixture = fixture(&format!("dfu-bin-fifo-{}", name.replace('/', "-")));
        let path = source(&fixture, name);
        fs::remove_file(&path).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &path,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        let response = fixture.rpc("runConfig.flashProposal", selection());
        assert!(response.error.is_some(), "{name}");
    }
    for (name, size) in [
        ("firmware.elf", 64 * 1024 * 1024 + 1),
        ("firmware a'b.bin", 32 * 1024 * 1024 + 1),
        (
            ".cmake/api/v1/reply/target-firmware.json",
            16 * 1024 * 1024 + 1,
        ),
    ] {
        let mut fixture = fixture(&format!("dfu-bin-limit-{}", name.replace('/', "-")));
        fs::File::create(source(&fixture, name))
            .unwrap()
            .set_len(size)
            .unwrap();
        assert!(
            fixture
                .rpc("runConfig.flashProposal", selection())
                .error
                .is_some(),
            "{name}"
        );
    }
}
