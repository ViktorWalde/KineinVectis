//! Native `pio run -t upload` (52 §14.44): a fake `pio` answers
//! `system info` and records the run; a fake `PlatformIO` interpreter answers
//! the configuration query from `query.json`, which each case edits.
use super::Fixture;
use serde_json::{Value, json};
use std::{fs, time::Duration};

fn quote(path: &std::path::Path) -> String {
    crate::platform::program_invocation(&path.display().to_string())
}

/// What the real query answers for the bench, pointing at fixture files.
fn accepted_query(root: &std::path::Path) -> Value {
    let platform = root.join("platform");
    json!({
        "core": "6.2.0",
        "configPath": root.join("platformio.ini"),
        "envs": ["esp32dev"], "defaultEnvs": [], "candidates": ["esp32dev"],
        "platformio": {},
        "env": "esp32dev",
        "options": {"platform": "espressif32", "board": "esp32dev", "framework": ["arduino"]},
        "platform": {"name": "espressif32", "version": "8.0.0", "files": [
            platform.join("platform.json"), platform.join("platform.py"), platform.join("builder/main.py")
        ]},
        "board": {"id": "esp32dev", "manifest": platform.join("boards/esp32dev.json"),
                  "upload": {"speed": 460_800, "protocol": "esptool"}},
        "protocol": "esptool",
        "uploader": {"dir": root.join("packages/tool-esptool"), "version": "2.50400.0"}
    })
}

fn pio_fixture(name: &str, core: &str) -> Fixture {
    let bench = Fixture::new(name);
    let root = bench.root.clone();
    fs::write(
        root.join("platformio.ini"),
        "[env:esp32dev]\nplatform = espressif32\nboard = esp32dev\nframework = arduino\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/main.cpp"),
        "void setup() {}\nvoid loop() {}\n",
    )
    .unwrap();
    for file in [
        "platform/platform.json",
        "platform/platform.py",
        "platform/builder/main.py",
        "platform/boards/esp32dev.json",
        "packages/tool-esptool/package.json",
    ] {
        fs::create_dir_all(root.join(file).parent().unwrap()).unwrap();
        fs::write(root.join(file), format!("{file} fixture\n")).unwrap();
    }
    write_query(&root, &accepted_query(&root));
    fs::write(
        root.join("pio-info.json"),
        json!({"core_version": {"value": core}, "python_exe": {"value": root.join("bin/pio-python")}})
            .to_string(),
    )
    .unwrap();
    let watched = "'^(ESPTOOL_|PLATFORMIO_(UPLOAD|BUILD|EXTRA|DEFAULT|SRC|CORE))'";
    crate::write_executable(
        root.join("bin/pio-python"),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" \"$2\" > {args}\npwd > {cwd}\nenv | grep -E {watched} > {env}\ncat {query}\n",
            args = quote(&root.join("python.args")),
            cwd = quote(&root.join("python.cwd")),
            env = quote(&root.join("python.env")),
            query = quote(&root.join("query.json")),
        ),
    );
    crate::write_executable(
        root.join("bin/pio"),
        format!(
            "#!/bin/sh\nif [ \"$1\" = system ]; then env | grep -E {watched} > {info_env}; pwd > {info_cwd}; cat {info}; exit 0; fi\n\
             printf '%s\\n' \"$@\" > {args}\npwd > {cwd}\nenv | grep -E {watched} > {env}\necho pio-fixture-complete\n",
            info_env = quote(&root.join("pio-info.env")),
            info_cwd = quote(&root.join("pio-info.cwd")),
            info = quote(&root.join("pio-info.json")),
            args = quote(&root.join("pio.args")),
            cwd = quote(&root.join("pio.cwd")),
            env = quote(&root.join("pio.env")),
        ),
    );
    bench
}

fn write_query(root: &std::path::Path, query: &Value) {
    fs::write(root.join("query.json"), format!("{query}\n")).unwrap();
}

fn pio_proposal(bench: &mut Fixture, extra: &Value) -> kinein_protocol::JsonRpcResponse {
    let mut params = json!({"native":true, "engine":"platformio", "device":bench.port});
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    bench.rpc("runConfig.flashProposal", params)
}

/// A refusal case: its name, how it edits the accepted query and what the
/// refusal must name.
type Case<'a> = (&'a str, &'a dyn Fn(&mut Value), &'a str);

fn message(response: &kinein_protocol::JsonRpcResponse) -> String {
    response
        .error
        .as_ref()
        .map(|error| error.message.clone())
        .unwrap_or_default()
}

#[test]
fn native_platformio_uploads_the_single_env_through_pio() {
    let mut bench = pio_fixture("native-pio-run", "6.2.0");
    let response = pio_proposal(&mut bench, &json!({}));
    assert!(response.error.is_none(), "{response:?}");
    let proposal = response.result.unwrap();
    let intent = &proposal["flash"];
    assert_eq!(intent["selection"]["engine"], "platformio");
    let root = bench.root.display().to_string();
    let port = bench.port.display().to_string();
    let args: Vec<&str> = intent["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    let expected = [
        "run",
        "-d",
        root.as_str(),
        "-e",
        "esp32dev",
        "-t",
        "upload",
        "--upload-port",
        port.as_str(),
    ];
    assert_eq!(args, expected);
    let source = proposal["source"].to_string();
    for needle in [
        "env: esp32dev (o único env do platformio.ini",
        "tool-esptool 2.50400.0",
        "baud: 460800 (placa)",
    ] {
        assert!(source.contains(needle), "{needle}: {proposal}");
    }
    assert!(
        proposal["warnings"]
            .to_string()
            .contains("compila antes de gravar"),
        "{proposal}"
    );
    // `pio system info` needs no project: it runs beside the executable.
    assert_eq!(
        fs::read_to_string(bench.root.join("pio-info.cwd"))
            .unwrap()
            .trim(),
        bench.root.join("bin").display().to_string()
    );
    let python = fs::read_to_string(bench.root.join("python.args")).unwrap();
    assert_eq!(python.lines().collect::<Vec<_>>(), ["-I", "-c"]);
    assert_eq!(
        fs::read_to_string(bench.root.join("python.cwd"))
            .unwrap()
            .trim(),
        root
    );
    let saved = bench.rpc(
        "runConfig.save",
        json!({"name":"Gravar (PlatformIO)", "command":proposal["command"], "flash":proposal["flash"]}),
    );
    assert!(saved.error.is_none(), "{saved:?}");
    // The code is what this engine builds and uploads: changing it keeps the
    // saved intent valid.
    fs::write(bench.root.join("src/main.cpp"), "void setup() {}\n").unwrap();
    let started = bench.rpc("run.start", json!({}));
    assert!(started.error.is_none(), "{started:?}");
    let id = started.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&bench.events, &id, Duration::from_secs(10));
    assert_eq!(exit, Some(0));
    let recorded = fs::read_to_string(bench.root.join("pio.args")).unwrap();
    assert_eq!(recorded.lines().collect::<Vec<_>>(), expected);
    assert_eq!(
        fs::read_to_string(bench.root.join("pio.cwd"))
            .unwrap()
            .trim(),
        root
    );
}

#[test]
fn native_platformio_refuses_what_its_preview_cannot_promise() {
    let cases: [Case<'_>; 12] = [
        (
            "two-envs",
            &|q| {
                q["candidates"] = json!(["esp32dev", "esp32s3"]);
            },
            "default_envs",
        ),
        (
            "rejected",
            &|q| *q = json!({"error": "UnknownEnvNamesError: x"}),
            "recusou a configuração",
        ),
        (
            "scripts",
            &|q| q["options"]["extra_scripts"] = json!(["pre:version.py"]),
            "extra_scripts",
        ),
        (
            "command",
            &|q| q["options"]["upload_command"] = json!("cp $SOURCE x"),
            "upload_command",
        ),
        (
            "packages",
            &|q| q["options"]["platform_packages"] = json!(["x"]),
            "platform_packages",
        ),
        (
            "configs",
            &|q| q["platformio"]["extra_configs"] = json!(["more.ini"]),
            "extra_configs",
        ),
        (
            "espidf",
            &|q| q["options"]["framework"] = json!(["arduino", "espidf"]),
            "espidf",
        ),
        (
            "avr",
            &|q| q["platform"]["name"] = json!("atmelavr"),
            "atmelavr",
        ),
        ("ota", &|q| q["protocol"] = json!("espota"), "espota"),
        (
            "touch",
            &|q| q["board"]["upload"]["use_1200bps_touch"] = json!(true),
            "use_1200bps_touch",
        ),
        // A board_upload.* key the manifest lacks stays a string, and the
        // platform takes any non-empty string as true, even "no".
        (
            "wait-string",
            &|q| q["board"]["upload"]["wait_for_upload_port"] = json!("no"),
            "wait_for_upload_port",
        ),
        (
            "uploader",
            &|q| q["uploader"]["dir"] = Value::Null,
            "tool-esptool",
        ),
    ];
    for (case, edit, needle) in cases {
        let mut bench = pio_fixture(&format!("native-pio-{case}"), "6.2.0");
        let mut query = accepted_query(&bench.root);
        edit(&mut query);
        write_query(&bench.root, &query);
        let text = message(&pio_proposal(&mut bench, &json!({})));
        assert!(text.contains(needle), "{case}: {text}");
        assert!(!bench.root.join("pio.args").exists(), "{case}");
    }
    for (case, extra) in [
        ("target", json!({"target":"app"})),
        ("probe", json!({"probe":"0483:374b:SERIAL"})),
        ("build", json!({"buildDirectory":"build"})),
    ] {
        let mut bench = pio_fixture(&format!("native-pio-{case}"), "6.2.0");
        assert!(pio_proposal(&mut bench, &extra).error.is_some(), "{case}");
    }
    let mut bench = pio_fixture("native-pio-core5", "5.2.5");
    assert!(message(&pio_proposal(&mut bench, &json!({}))).contains("Core 6"));
    let mut bench = pio_fixture("native-pio-esptool-config", "6.2.0");
    fs::write(
        bench.root.join("esptool.cfg"),
        "[esptool]\nconnect_attempts = 1\n",
    )
    .unwrap();
    assert!(message(&pio_proposal(&mut bench, &json!({}))).contains("esptool.cfg"));
}

#[test]
fn native_platformio_binds_configuration_by_content() {
    let mut bench = pio_fixture("native-pio-binding", "6.2.0");
    let proposal = pio_proposal(&mut bench, &json!({})).result.unwrap();
    let saved = bench.rpc(
        "runConfig.save",
        json!({"name":"Gravar (PlatformIO)", "command":proposal["command"], "flash":proposal["flash"]}),
    );
    assert!(saved.error.is_none(), "{saved:?}");
    // A different computed option the preview does not display, with the
    // same files, is another intent.
    let mut query = accepted_query(&bench.root);
    query["options"]["build_flags"] = json!(["-DKINEIN=1"]);
    write_query(&bench.root, &query);
    let flagged = pio_proposal(&mut bench, &json!({})).result.unwrap();
    assert_ne!(
        flagged["flash"]["fingerprint"],
        proposal["flash"]["fingerprint"]
    );
    write_query(&bench.root, &accepted_query(&bench.root));
    // Rewriting the board manifest after saving refuses the saved intent.
    fs::write(
        bench.root.join("platform/boards/esp32dev.json"),
        "{\"upload\": {\"speed\": 921600}}\n",
    )
    .unwrap();
    let response = bench.rpc("run.start", json!({}));
    assert!(response.error.is_some(), "{response:?}");
    assert!(!bench.root.join("pio.args").exists());
}
