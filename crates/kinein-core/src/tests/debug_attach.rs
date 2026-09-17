//! TCP attach through the real Core, framing, handshake and teardown.
use std::{io::BufReader, net::TcpListener, sync::mpsc, thread, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

use crate::lsp::framing::{read_message, write_message};

fn core(name: &str) -> (crate::Core, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("kinein-attach-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("service.py"), "print(42)\n").unwrap();
    let (events, _) = mpsc::channel();
    let mut core = super::core_with_empty_search_path(name);
    core.enable_lsp(events);
    let response = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));
    assert!(response.response().error.is_none());
    (core, root)
}

fn fake_adapter(listener: TcpListener, refuse: bool) -> thread::JoinHandle<Vec<Value>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut output = socket.try_clone().unwrap();
        let mut input = BufReader::new(socket);
        let mut requests = Vec::new();
        let mut attach_seq = Value::Null;
        while let Some(request) = read_message(&mut input).unwrap() {
            let command = request["command"].as_str().unwrap();
            let response = json!({ "seq": 1, "type": "response", "request_seq": request["seq"], "command": command, "success": true, "body": {} });
            if command == "attach" {
                attach_seq = request["seq"].clone();
                if refuse {
                    write_message(&mut output, &json!({ "seq": 2, "type": "response", "request_seq": attach_seq, "command": "attach", "success": false, "message": "attach recusado pela prova" })).unwrap();
                    requests.push(request);
                    break;
                }
                write_message(
                    &mut output,
                    &json!({ "seq": 2, "type": "event", "event": "initialized" }),
                )
                .unwrap();
            } else {
                write_message(&mut output, &response).unwrap();
                if command == "configurationDone" {
                    write_message(&mut output, &json!({ "seq": 3, "type": "response", "request_seq": attach_seq, "command": "attach", "success": true })).unwrap();
                }
                if command == "disconnect" {
                    requests.push(request);
                    // No terminated event: EOF must release the session too.
                    break;
                }
            }
            requests.push(request);
        }
        requests
    })
}

#[test]
fn attach_replays_breakpoints_and_detaches_without_a_local_interpreter() {
    let (mut core, root) = core("flow");
    let file = root.join("service.py").canonicalize().unwrap();
    let stored = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": file, "breakpoints": [{ "line": 1 }] })),
    ));
    assert!(stored.response().error.is_none());
    for finish in ["debug.stop", "workspace.close"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = fake_adapter(listener, false);
        let params = json!({ "connect": { "host": " 127.0.0.1 ", "port": port } });
        let started = core.handle_request(&JsonRpcRequest::new(
            3_i64,
            "debug.start",
            Some(params.clone()),
        ));
        assert!(
            started.response().error.is_none(),
            "{:?}",
            started.response()
        );
        assert_eq!(
            started.response().result.as_ref().unwrap()["attached"],
            true
        );
        let duplicate =
            core.handle_request(&JsonRpcRequest::new(4_i64, "debug.start", Some(params)));
        assert_eq!(
            duplicate.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidRequest
        );
        assert!(
            core.handle_request(&JsonRpcRequest::new(5_i64, finish, None))
                .response()
                .error
                .is_none()
        );
        let requests = server.join().unwrap();
        let commands: Vec<_> = requests
            .iter()
            .map(|r| r["command"].as_str().unwrap())
            .collect();
        assert_eq!(
            commands,
            [
                "initialize",
                "attach",
                "setBreakpoints",
                "configurationDone",
                "disconnect"
            ]
        );
        assert_eq!(
            requests[1]["arguments"],
            json!({ "connect": { "host": "127.0.0.1", "port": port } })
        );
        assert_eq!(requests[2]["arguments"]["source"]["path"], json!(file));
        assert_eq!(requests[4]["arguments"]["terminateDebuggee"], false);
    }
}

#[test]
fn attach_validates_endpoint_and_conflicting_program_before_connecting() {
    let (mut core, _) = core("invalid");
    for params in [
        json!({ "connect": { "host": "", "port": 5678 } }),
        json!({ "connect": { "host": "host with spaces", "port": 5678 } }),
        json!({ "connect": { "host": "http://localhost", "port": 5678 } }),
        json!({ "connect": { "host": "localhost", "port": 0 } }),
        json!({ "connect": { "host": "localhost", "port": 65536 } }),
        json!({ "connect": { "host": "localhost", "port": -1 } }),
        json!({ "connect": { "host": "localhost", "port": "5678" } }),
        json!({ "connect": { "host": "localhost", "port": 5678, "extra": true } }),
        json!({ "connect": { "host": "localhost" } }),
        json!({ "connect": { "host": "localhost", "port": 5678 }, "program": "missing.py" }),
    ] {
        let result = core.handle_request(&JsonRpcRequest::new(2_i64, "debug.start", Some(params)));
        assert_eq!(
            result.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidParams,
            "{:?}",
            result.response()
        );
    }
}

#[test]
fn refused_attach_preserves_adapter_error_and_allows_retry() {
    let (mut core, _) = core("refusal");
    for refuse in [true, false] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = fake_adapter(listener, refuse);
        let response = core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "debug.start",
            Some(json!({ "connect": { "host": "127.0.0.1", "port": port } })),
        ));
        if refuse {
            assert!(
                response
                    .response()
                    .error
                    .as_ref()
                    .unwrap()
                    .message
                    .contains("attach recusado pela prova")
            );
        } else {
            assert!(
                response.response().error.is_none(),
                "{:?}",
                response.response()
            );
            drop(core.handle_request(&JsonRpcRequest::new(3_i64, "debug.stop", None)));
        }
        server.join().unwrap();
    }
}
