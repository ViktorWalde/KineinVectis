//! probe-rs debugging on the chosen probe (0.190.0, `roadmaps/52` §14.36):
//! real dispatch, the native flash fixture and a stdio `dap-server` stand-in.
use super::Fixture;
use super::probe_rs::{LISTED, PROBE, fixture, propose, request};
use kinein_protocol::JsonRpcErrorCode;
use serde_json::{Value, json};
use std::{
    fs, thread,
    time::{Duration, Instant},
};

/// Speaks just enough DAP over stdio; records the launch and its environment,
/// and ends by itself (terminated + exit) when `dap.exit` appears.
const FAKE_DAP: &str = r#"import json, os, sys, threading, time
log = sys.argv[1]
def watch():
    while not os.path.exists(os.path.join(log, "dap.exit")):
        time.sleep(0.02)
    send({"type": "event", "event": "terminated"})
    os._exit(0)
threading.Thread(target=watch, daemon=True).start()
with open(os.path.join(log, "dap.env"), "w") as out:
    out.write("".join(name + "\n" for name in os.environ if name.startswith("PROBE_RS_")))
seq = 0
def read():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode().strip()
        if not line:
            break
        key, value = line.split(":", 1)
        headers[key.strip().lower()] = value.strip()
    return json.loads(sys.stdin.buffer.read(int(headers["content-length"])))
def send(message):
    global seq
    seq += 1
    message["seq"] = seq
    data = json.dumps(message).encode()
    sys.stdout.buffer.write(b"Content-Length: %d\r\n\r\n" % len(data) + data)
    sys.stdout.buffer.flush()
def respond(request, body=None):
    send({"type": "response", "request_seq": request["seq"], "command": request["command"], "success": True, "body": body or {}})
while True:
    request = read()
    if request is None:
        break
    command = request.get("command")
    if command == "initialize":
        respond(request, {"supportsConfigurationDoneRequest": True})
        send({"type": "event", "event": "initialized"})
    elif command == "launch":
        with open(os.path.join(log, "dap.launch"), "w") as out:
            json.dump(request.get("arguments"), out)
        respond(request)
    elif command == "disconnect":
        respond(request)
        sys.exit(0)
    else:
        respond(request)
"#;

fn debug_fixture(name: &str) -> Fixture {
    let fixture = fixture(name);
    fs::write(fixture.root.join("fake_dap.py"), FAKE_DAP).unwrap();
    fixture
}

fn start(fixture: &mut Fixture, probe: &str) -> kinein_protocol::JsonRpcResponse {
    let program = fixture.root.join(".kinein/build/firmware.elf");
    fixture.rpc("debug.start", json!({"program": program, "probe": probe}))
}

/// Gravar a mesma sonda devolve busy enquanto `busy` for verdade; senao executa.
fn flash_until(
    fixture: &mut Fixture,
    intent: &Value,
    busy: bool,
) -> kinein_protocol::JsonRpcResponse {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let response = fixture.rpc("run.start", json!({"flash": intent}));
        let is_busy = response
            .error
            .as_ref()
            .is_some_and(|error| error.code == JsonRpcErrorCode::SerialPortBusy);
        if is_busy == busy || Instant::now() >= deadline {
            return response;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn debugging_uses_the_chosen_probe_and_reserves_it_until_the_adapter_ends() {
    let mut fixture = debug_fixture("probe-rs-debug-reserve");
    let intent = propose(&mut fixture, request()).result.unwrap()["flash"].clone();
    let started = start(&mut fixture, PROBE);
    assert!(started.error.is_none(), "{started:?}");
    let launch: Value =
        serde_json::from_slice(&fs::read(fixture.root.join("dap.launch")).unwrap()).unwrap();
    assert_eq!(launch["probe"], PROBE);
    assert_eq!(launch["chip"], "STM32F411RETx");
    assert!(launch.get("flashingConfig").is_none(), "{launch}");
    assert_eq!(
        fs::read_to_string(fixture.root.join("dap.env")).unwrap(),
        ""
    );

    let busy = flash_until(&mut fixture, &intent, true);
    let error = busy
        .error
        .expect("flashing the probe being debugged must refuse");
    assert_eq!(error.code, JsonRpcErrorCode::SerialPortBusy);
    assert!(error.message.contains("Depuração"), "{error:?}");
    assert!(error.message.contains("Pare a depuração"), "{error:?}");
    assert!(!fixture.root.join("probe.args").exists());

    // The adapter ends by itself; the dead session stays in the slot, the
    // probe does not.
    fs::write(fixture.root.join("dap.exit"), "").unwrap();
    let flashed = flash_until(&mut fixture, &intent, false);
    assert!(flashed.error.is_none(), "{flashed:?}");
    let id = flashed.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, exit) =
        crate::tests::terminal_run_until_closed(&fixture.events, &id, Duration::from_secs(5));
    assert_eq!(exit, Some(0));
}

#[test]
fn debugging_refuses_wrong_absent_or_reserved_probes() {
    let mut fixture = debug_fixture("probe-rs-debug-refusals");
    let root = fixture.root.clone();
    let connect = fixture.rpc(
        "debug.start",
        json!({"connect": {"host": "127.0.0.1", "port": 5678}, "probe": PROBE}),
    );
    assert_eq!(connect.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
    let malformed = start(&mut fixture, "0483:374b");
    assert_eq!(
        malformed.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    fs::write(root.join("probe.list"), "No debug probes were found.\n").unwrap();
    let absent = start(&mut fixture, PROBE).error.unwrap();
    assert_eq!(absent.code, JsonRpcErrorCode::InvalidRequest);
    assert!(
        absent.message.contains("não está na listagem"),
        "{absent:?}"
    );
    fs::write(root.join("probe.list"), LISTED).unwrap();

    let held = fixture
        .core
        .serial_usage
        .acquire_probe(PROBE, "run.start", "Gravação nativa por sonda", Some(&root))
        .unwrap();
    let reserved = start(&mut fixture, PROBE).error.unwrap();
    assert_eq!(reserved.code, JsonRpcErrorCode::SerialPortBusy);
    drop(held);

    // Another adapter in the kit: the selector belongs to probe-rs only.
    crate::write_executable(
        root.join("bin/gdb"),
        "#!/bin/sh\necho 'GNU gdb (fixture) 16.2'\n",
    );
    fixture.core.refresh_tool_registry();
    let gdb = fixture.rpc("toolchain.set", json!({"role":"debugAdapter", "id":"gdb"}));
    assert!(gdb.error.is_none(), "{gdb:?}");
    let other = start(&mut fixture, PROBE).error.unwrap();
    assert_eq!(other.code, JsonRpcErrorCode::InvalidParams);
    assert!(other.message.contains("probe-rs"), "{other:?}");
    assert!(
        !root.join("dap.launch").exists(),
        "no refusal may start the adapter"
    );
}
