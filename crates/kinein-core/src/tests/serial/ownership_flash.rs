//! Native flash dispatch and serial lifetime: fake esptool, regular-file port.
use super::ownership_fixture::Fixture;
use serde_json::{Value, json};
use std::{fs, time::Duration};
mod catalog;
mod pending;

fn fixture(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fs::write(
        fixture.root.join("CMakeLists.txt"),
        "include($ENV{IDF_PATH}/tools/cmake/project.cmake)\nproject(native)\n",
    )
    .unwrap();
    fs::write(
        fixture.root.join("sdkconfig"),
        "CONFIG_IDF_TARGET=\"esp32c3\"\n",
    )
    .unwrap();
    let build = fixture.root.join("build");
    fs::create_dir(&build).unwrap();
    fs::write(build.join("app.bin"), "compiled fixture").unwrap();
    fs::write(
        build.join("flasher_args.json"),
        serde_json::to_vec(&json!({
            "flash_files":{"0x0":"app.bin"},
            "extra_esptool_args":{"chip":"esp32c3", "stub":true}
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        build.join("project_description.json"),
        serde_json::to_vec(&json!({
            "version":"1.3", "project_path":fixture.root, "build_dir":build,
            "config_file":fixture.root.join("sdkconfig"), "target":"esp32c3", "app_bin":"app.bin"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(fixture.root.join("flash.release"), "").unwrap();
    let args =
        crate::platform::program_invocation(&fixture.root.join("flash.args").display().to_string());
    let ready = crate::platform::program_invocation(
        &fixture.root.join("flash.ready").display().to_string(),
    );
    let release = crate::platform::program_invocation(
        &fixture.root.join("flash.release").display().to_string(),
    );
    crate::write_executable(
        fixture.root.join("bin/esptool"),
        format!(
            "#!/bin/sh\nif [ \"$1\" = version ]; then touch {query_ready}; while [ -f {query_wait} ]; do sleep .02; done; echo 5.4.0; exit 0; fi\nprintf '%s\\n' \"$@\" > {args}\ntouch {ready}\nwhile [ ! -f {release} ]; do sleep .02; done\necho flash-fixture-complete\n",
            query_wait = crate::platform::program_invocation(
                &fixture.root.join("query.wait").display().to_string()
            ),
            query_ready = crate::platform::program_invocation(
                &fixture.root.join("query.ready").display().to_string()
            ),
        ),
    );
    fixture
}

fn proposal(fixture: &mut Fixture) -> kinein_protocol::JsonRpcResponse {
    fixture.rpc(
        "runConfig.flashProposal",
        json!({"native":true, "device":fixture.port}),
    )
}

#[test]
fn native_flash_refuses_a_recipe_without_build_origin() {
    let mut fixture = fixture("native-unproven-origin");
    fs::remove_file(fixture.root.join("build/project_description.json")).unwrap();
    let response = proposal(&mut fixture);
    assert!(
        response.error.is_some(),
        "native execution needs provenance: {response:?}"
    );
    assert!(!fixture.root.join("flash.args").exists());
}

fn native_proposal(fixture: &mut Fixture) -> Value {
    let response = proposal(fixture);
    assert!(response.error.is_none(), "{response:?}");
    let proposal = response.result.unwrap();
    assert!(proposal["flash"].is_object(), "{proposal:?}");
    proposal
}

#[test]
fn native_flash_is_saved_checked_and_executed_with_literal_arguments() {
    let mut fixture = fixture("native-save-run");
    let proposal = native_proposal(&mut fixture);
    assert!(
        proposal["source"]
            .as_array()
            .unwrap()
            .iter()
            .all(|line| { !line.as_str().unwrap().contains("escolhido na tela") }),
        "automatic engine resolution must not claim an explicit UI choice: {proposal}"
    );
    let saved = fixture.rpc(
        "runConfig.save",
        json!({
            "name":"Native flash", "command":proposal["command"], "flash":proposal["flash"]
        }),
    );
    assert!(saved.error.is_none(), "{saved:?}");
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
    assert!(args.contains("write-flash\n0x0\n"), "{args}");
    assert!(args.contains(fixture.port.to_str().unwrap()), "{args}");
}

fn update_description(fixture: &Fixture, field: &str, value: Value) {
    let path = fixture.root.join("build/project_description.json");
    let mut description: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    description[field] = value;
    fs::write(path, serde_json::to_vec(&description).unwrap()).unwrap();
}

#[test]
fn native_flash_refuses_wrong_project_target_configuration_and_app() {
    for field in [
        "project_path",
        "build_dir",
        "target",
        "config_file",
        "app_bin",
        "version",
    ] {
        let mut fixture = fixture(&format!("native-metadata-{field}"));
        update_description(&fixture, field, json!(fixture.root.join("other")));
        assert!(
            proposal(&mut fixture).error.is_some(),
            "accepted mismatched {field}"
        );
        assert!(!fixture.root.join("flash.args").exists());
    }
}

#[test]
fn native_flash_accepts_an_explicit_external_build_bound_to_this_workspace() {
    let mut fixture = fixture("native-external-selection");
    let external = super::temp_dir("native-external-build");
    for file in ["flasher_args.json", "app.bin", "project_description.json"] {
        fs::copy(fixture.root.join("build").join(file), external.join(file)).unwrap();
    }
    let path = external.join("project_description.json");
    let mut description: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    description["version"] = json!("1.2");
    description["build_dir"] = json!(external);
    fs::write(path, serde_json::to_vec(&description).unwrap()).unwrap();
    let response = fixture.rpc(
        "runConfig.flashProposal",
        json!({"native":true,"device":fixture.port,"buildDirectory":external}),
    );
    assert!(response.error.is_none(), "{response:?}");
    let intent = response.result.unwrap()["flash"].clone();
    assert_eq!(
        intent["selection"]["buildDirectory"],
        external.display().to_string()
    );
    assert!(
        intent["args"]
            .as_array()
            .unwrap()
            .contains(&json!(external.join("app.bin")))
    );
}

#[test]
fn native_flash_records_the_resolved_image_and_refuses_a_changed_alias() {
    let mut fixture = fixture("native-image-alias");
    let build = fixture.root.join("build");
    let actual = build.join("actual.bin");
    fs::rename(build.join("app.bin"), &actual).unwrap();
    std::os::unix::fs::symlink(&actual, build.join("app.bin")).unwrap();
    let intent = native_proposal(&mut fixture)["flash"].clone();
    assert!(intent["args"].as_array().unwrap().contains(&json!(actual)));
    let external = fixture.root.join("outside-build.bin");
    fs::write(&external, "other bytes").unwrap();
    fs::remove_file(build.join("app.bin")).unwrap();
    std::os::unix::fs::symlink(external, build.join("app.bin")).unwrap();
    assert!(
        fixture
            .rpc("run.start", json!({"flash":intent}))
            .error
            .is_some()
    );
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn native_flash_rejects_changed_files_tool_kit_device_and_forged_argv() {
    for change in [
        "image", "recipe", "metadata", "config", "tool", "kit", "device", "argv",
    ] {
        let mut fixture = fixture(&format!("native-changed-{change}"));
        let mut intent = native_proposal(&mut fixture)["flash"].clone();
        match change {
            "image" => fs::write(fixture.root.join("build/app.bin"), "changed image").unwrap(),
            "recipe" => fs::write(fixture.root.join("build/flasher_args.json"), "{}").unwrap(),
            "metadata" => {
                update_description(&fixture, "project_path", json!(fixture.root.join("other")));
            }
            "config" => fs::write(
                fixture.root.join("sdkconfig"),
                "CONFIG_IDF_TARGET=\"esp32\"\n",
            )
            .unwrap(),
            "tool" => {
                crate::write_executable(
                    fixture.root.join("bin/esptool"),
                    "#!/bin/sh\necho 6.0.0\n",
                );
            }
            "kit" => {
                assert!(
                    fixture
                        .rpc("toolchain.setKit", json!({"chip":"esp32","preset":""}))
                        .error
                        .is_none()
                );
            }
            "device" => {
                fs::remove_file(&fixture.port).unwrap();
                fs::write(&fixture.port, "new node").unwrap();
            }
            "argv" => {
                intent["args"] = json!(["version"]);
            }
            _ => unreachable!(),
        }
        let response = fixture.rpc("run.start", json!({"flash":intent}));
        assert!(
            response.error.is_some(),
            "accepted changed {change}: {response:?}"
        );
        assert!(
            !fixture.root.join("flash.args").exists(),
            "launched after {change}"
        );
        assert!(fixture.core.run_preflight.is_none());
    }
}

#[test]
fn native_flash_holds_its_serial_reservation_until_the_terminal_exits() {
    let mut fixture = fixture("native-busy-until-exit");
    let proposal = native_proposal(&mut fixture);
    fs::remove_file(fixture.root.join("flash.release")).unwrap();
    let started = fixture.rpc("run.start", json!({"flash":proposal["flash"]}));
    assert!(started.error.is_none(), "{started:?}");
    let busy = fixture.rpc("serial.monitor", json!({"device":fixture.port}));
    assert_eq!(
        busy.error.unwrap().code,
        kinein_protocol::JsonRpcErrorCode::SerialPortBusy
    );
    let other = fixture.root.join("other-port");
    fs::write(&other, "other node").unwrap();
    let monitor = fixture.rpc("serial.monitor", json!({"device":other}));
    fixture.close(monitor);
    assert!(fixture.rpc("run.stop", json!({})).error.is_none());
    let id = started.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    fixture.event("event.terminal.closed", "id", &id);
    let monitor = fixture.rpc("serial.monitor", json!({"device":fixture.port}));
    fixture.close(monitor);
}
