//! Native `idf.py flash` (52 §14.40): fake activation and `idf.py` behind a
//! real `bash`, the ESP-IDF build fixture and a regular-file port.
use super::{Fixture, fixture};
use serde_json::{Value, json};
use std::{fs, time::Duration};

/// The fixture with an ESP-IDF home whose activation puts a recording
/// `idf.py` first in `PATH`; `version` is what `idf.py --version` answers.
fn idf_fixture(name: &str, version: &str) -> Fixture {
    let mut bench = fixture(name);
    let root = bench.root.clone();
    let quote =
        |path: std::path::PathBuf| crate::platform::program_invocation(&path.display().to_string());
    crate::write_executable(root.join("bin/bash"), "#!/bin/sh\nexec /bin/bash \"$@\"\n");
    let tools = root.join("home/.espressif/tools");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(root.join("idf-bin")).unwrap();
    fs::write(
        tools.join("activate_idf_v6.1.sh"),
        format!("export PATH={}:\"$PATH\"\n", quote(root.join("idf-bin"))),
    )
    .unwrap();
    let watched = "'^(ESPTOOL_|ESPPORT=|ESPBAUD=|IDF_FLASH_FULL=|IDF_TRUST_FLASH_CONTENT=|SERIAL_TOOL_EXTRA|IDF_EXTRA_ACTIONS_PATH=|IDF_EXTENSION_ALLOW)'";
    crate::write_executable(
        root.join("idf-bin/idf.py"),
        format!(
            "#!/bin/sh\nif [ \"$1\" = --version ]; then env | grep -E {watched} > {version_env}; pwd > {version_cwd}; echo 'ESP-IDF {version}'; exit 0; fi\n\
             printf '%s\\n' \"$@\" > {args}\npwd > {cwd}\nenv | grep -E {watched} > {env}\necho idf-fixture-complete\n",
            version_env = quote(root.join("idf-version.env")),
            version_cwd = quote(root.join("idf-version.cwd")),
            args = quote(root.join("idf.args")),
            cwd = quote(root.join("idf.cwd")),
            env = quote(root.join("idf.env")),
        ),
    );
    bench.core.home_override = Some(root.join("home"));
    bench
}

fn idf_proposal(bench: &mut Fixture, extra: &Value) -> kinein_protocol::JsonRpcResponse {
    let mut params = json!({"native":true, "engine":"idf.py", "device":bench.port});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    bench.rpc("runConfig.flashProposal", params)
}

fn message(response: &kinein_protocol::JsonRpcResponse) -> String {
    response
        .error
        .as_ref()
        .map(|error| error.message.clone())
        .unwrap_or_default()
}

#[test]
fn native_idf_py_builds_and_flashes_through_the_activated_wrapper() {
    let mut bench = idf_fixture("native-idf-run", "v6.1");
    let response = idf_proposal(&mut bench, &json!({}));
    assert!(response.error.is_none(), "{response:?}");
    let proposal = response.result.unwrap();
    let build = bench.root.join("build").display().to_string();
    let intent = &proposal["flash"];
    assert_eq!(intent["selection"]["engine"], "idf.py");
    let args: Vec<&str> = intent["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    let port = bench.port.display().to_string();
    assert_eq!(
        &args[args.len() - 7..],
        [
            "-B",
            build.as_str(),
            "-p",
            port.as_str(),
            "-b",
            "460800",
            "flash"
        ]
    );
    assert!(
        proposal["source"].to_string().contains("ESP-IDF v6.1"),
        "{proposal}"
    );
    assert!(
        proposal["warnings"]
            .to_string()
            .contains("compila antes de gravar"),
        "{proposal}"
    );
    let saved = bench.rpc(
        "runConfig.save",
        json!({"name":"Gravar (idf.py)", "command":proposal["command"], "flash":proposal["flash"]}),
    );
    assert!(saved.error.is_none(), "{saved:?}");
    // Code changes and the images a rebuild writes are what this engine
    // compiles and flashes: they do not invalidate the saved intent.
    fs::write(bench.root.join("main.c"), "void app_main(void) {}\n").unwrap();
    fs::write(bench.root.join("build/app.bin"), "rebuilt image").unwrap();
    let started = bench.rpc("run.start", json!({}));
    assert!(started.error.is_none(), "{started:?}");
    let id = started.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&bench.events, &id, Duration::from_secs(10));
    assert_eq!(exit, Some(0));
    let recorded = fs::read_to_string(bench.root.join("idf.args")).unwrap();
    assert_eq!(
        recorded.lines().collect::<Vec<_>>(),
        &args[args.len() - 7..]
    );
    assert_eq!(
        fs::read_to_string(bench.root.join("idf.cwd"))
            .unwrap()
            .trim(),
        bench.root.display().to_string()
    );
    // In the project, idf.py would import the project's idf_ext.py: the
    // version query runs in the activation's folder.
    assert_eq!(
        fs::read_to_string(bench.root.join("idf-version.cwd"))
            .unwrap()
            .trim(),
        bench
            .root
            .join("home/.espressif/tools")
            .display()
            .to_string()
    );
}

#[test]
fn native_idf_py_refuses_what_its_preview_cannot_promise() {
    for (case, extra) in [
        ("probe", json!({"probe":"0483:374b:SERIAL"})),
        ("target", json!({"target":"app"})),
        ("firmware", json!({"firmware":"fixture-catalog-entry"})),
    ] {
        let mut bench = idf_fixture(&format!("native-idf-{case}"), "v6.1");
        assert!(idf_proposal(&mut bench, &extra).error.is_some(), "{case}");
    }
    let mut bench = idf_fixture("native-idf-no-device", "v6.1");
    let response = bench.rpc(
        "runConfig.flashProposal",
        json!({"native":true, "engine":"idf.py"}),
    );
    assert!(response.error.is_some());

    // Never configured: idf.py would configure and build first, unpreviewed.
    let mut bench = idf_fixture("native-idf-unbuilt", "v6.1");
    fs::remove_file(bench.root.join("build/project_description.json")).unwrap();
    assert!(idf_proposal(&mut bench, &json!({})).error.is_some());

    let mut bench = idf_fixture("native-idf-v5", "v5.4.2");
    assert!(message(&idf_proposal(&mut bench, &json!({}))).contains("ESP-IDF 6"));

    let mut bench = idf_fixture("native-idf-esptool-config", "v6.1");
    fs::write(
        bench.root.join("build/esptool.cfg"),
        "[esptool]\nconnect_attempts = 1\n",
    )
    .unwrap();
    assert!(message(&idf_proposal(&mut bench, &json!({}))).contains("esptool.cfg"));

    // An idf.py extension in the project, or in a build component, can
    // redefine flash.
    let mut bench = idf_fixture("native-idf-project-extension", "v6.1");
    fs::write(
        bench.root.join("idf_ext.py"),
        "def action_extensions(base, path):\n    return {}\n",
    )
    .unwrap();
    assert!(message(&idf_proposal(&mut bench, &json!({}))).contains("idf_ext.py"));
    let mut bench = idf_fixture("native-idf-component-extension", "v6.1");
    let component = bench.root.join("components/extended");
    fs::create_dir_all(&component).unwrap();
    fs::write(
        component.join("idf_ext.py"),
        "def action_extensions(base, path):\n    return {}\n",
    )
    .unwrap();
    let description = bench.root.join("build/project_description.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&description).unwrap()).unwrap();
    value["build_component_info"] = json!({"extended": {"dir": component}});
    fs::write(&description, value.to_string()).unwrap();
    assert!(
        message(&idf_proposal(&mut bench, &json!({})))
            .contains(&component.join("idf_ext.py").display().to_string())
    );

    let mut bench = idf_fixture("native-idf-encrypted", "v6.1");
    fs::write(
        bench.root.join("build/config/sdkconfig.json"),
        r#"{"SECURE_BOOT": false, "SECURE_FLASH_ENC_ENABLED": true}"#,
    )
    .unwrap();
    assert!(message(&idf_proposal(&mut bench, &json!({}))).contains("eFuses"));
    assert!(!bench.root.join("idf.args").exists());
}

#[test]
fn native_idf_py_binds_configuration_by_content() {
    let mut bench = idf_fixture("native-idf-binding", "v6.1");
    let proposal = idf_proposal(&mut bench, &json!({})).result.unwrap();
    let saved = bench.rpc(
        "runConfig.save",
        json!({"name":"Gravar (idf.py)", "command":proposal["command"], "flash":proposal["flash"]}),
    );
    assert!(saved.error.is_none(), "{saved:?}");
    // A rebuild rewrites the description with the same content: still valid.
    let description = bench.root.join("build/project_description.json");
    let same = fs::read(&description).unwrap();
    fs::remove_file(&description).unwrap();
    fs::write(&description, same).unwrap();
    let again = idf_proposal(&mut bench, &json!({})).result.unwrap();
    assert_eq!(
        again["flash"]["fingerprint"],
        proposal["flash"]["fingerprint"]
    );
    // A new configuration default changes what the build would flash: refused.
    fs::write(
        bench.root.join("sdkconfig.defaults"),
        "CONFIG_ESPTOOLPY_FLASHSIZE_8MB=y\n",
    )
    .unwrap();
    let response = bench.rpc("run.start", json!({}));
    assert!(response.error.is_some(), "{response:?}");
    assert!(!bench.root.join("idf.args").exists());
}
