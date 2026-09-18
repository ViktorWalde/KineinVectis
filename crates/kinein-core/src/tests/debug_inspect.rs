//! P3 (2026-09-17): o que o depurador de embarcado MOSTRA — escopos inteiros
//! (Registers, os perifericos do SVD), memoria e disassembly como passagem
//! do DAP padrao, e o RTT do probe-rs com o handshake `rttWindowOpened` —
//! provado por despacho contra um adaptador FALSO em TCP que fala como o
//! `probe-rs dap-server` 0.32.0 (nomes de evento lidos na fonte).

use std::{io::BufReader, net::TcpListener, sync::mpsc, thread, time::Duration};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

use crate::lsp::framing::{read_message, write_message};

/// O adaptador falso: aceita o attach, para na thread 1 ao `configurationDone`
/// (com um `stackTrace` de um frame), anuncia um canal RTT e manda dados
/// nele, e responde `scopes`/`readMemory`/`disassemble` com corpos fixos.
fn fake_probe_rs(listener: TcpListener) -> thread::JoinHandle<Vec<Value>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut output = socket.try_clone().unwrap();
        let mut input = BufReader::new(socket);
        let mut requests = Vec::new();
        let mut attach_seq = Value::Null;
        let mut seq = 100;
        let mut resposta = |output: &mut std::net::TcpStream, req: &Value, body: Value| {
            seq += 1;
            write_message(output, &json!({ "seq": seq, "type": "response", "request_seq": req["seq"], "command": req["command"], "success": true, "body": body })).unwrap();
        };
        while let Some(request) = read_message(&mut input).unwrap() {
            let command = request["command"].as_str().unwrap().to_owned();
            match command.as_str() {
                "attach" => {
                    attach_seq = request["seq"].clone();
                    write_message(
                        &mut output,
                        &json!({ "seq": 2, "type": "event", "event": "initialized" }),
                    )
                    .unwrap();
                }
                "configurationDone" => {
                    resposta(&mut output, &request, json!({}));
                    write_message(&mut output, &json!({ "seq": 3, "type": "response", "request_seq": attach_seq, "command": "attach", "success": true })).unwrap();
                    // Como o probe-rs: o canal RTT anunciado, e a placa parada.
                    write_message(&mut output, &json!({ "seq": 4, "type": "event", "event": "probe-rs-rtt-channel-config", "body": { "channelNumber": 0, "channelName": "Terminal", "dataFormat": "String" } })).unwrap();
                    write_message(&mut output, &json!({ "seq": 5, "type": "event", "event": "probe-rs-rtt-data", "body": { "channelNumber": 0, "data": "boot ok\nled on\n" } })).unwrap();
                    write_message(&mut output, &json!({ "seq": 6, "type": "event", "event": "probe-rs-show-message", "body": { "severity": "warning", "message": "no RTT control block" } })).unwrap();
                    write_message(&mut output, &json!({ "seq": 7, "type": "event", "event": "stopped", "body": { "reason": "breakpoint", "threadId": 1 } })).unwrap();
                }
                "stackTrace" => resposta(
                    &mut output,
                    &request,
                    json!({ "stackFrames": [{ "id": 7, "name": "main", "line": 3, "column": 1, "source": { "path": "/w/main.c" }, "instructionPointerReference": "0x40080000" }] }),
                ),
                "scopes" => resposta(
                    &mut output,
                    &request,
                    json!({ "scopes": [
                    { "name": "Locals", "variablesReference": 11, "expensive": false },
                    { "name": "Registers", "variablesReference": 12, "expensive": false },
                    { "name": "Peripherals", "variablesReference": 13, "expensive": true }
                ] }),
                ),
                "readMemory" => resposta(
                    &mut output,
                    &request,
                    json!({ "address": "0x3ff00000", "data": "AQIDBA==", "unreadableBytes": 0 }),
                ),
                "disassemble" => resposta(
                    &mut output,
                    &request,
                    json!({ "instructions": [
                    { "address": "0x40080000", "instruction": "entry a1, 32", "instructionBytes": "36 41 00", "symbol": "main", "location": { "path": "/w/main.c" }, "line": 3 },
                    { "address": "0x40080003", "instruction": "movi a2, 1" }
                ] }),
                ),
                "disconnect" => {
                    resposta(&mut output, &request, json!({}));
                    requests.push(request);
                    break;
                }
                _ => resposta(&mut output, &request, json!({})),
            }
            requests.push(request);
        }
        requests
    })
}

fn espera(
    receiver: &mpsc::Receiver<JsonRpcRequest>,
    ate: &dyn Fn(&JsonRpcRequest) -> bool,
) -> JsonRpcRequest {
    let limite = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let evento = receiver
            .recv_timeout(limite.saturating_duration_since(std::time::Instant::now()))
            .expect("evento dentro do prazo");
        if ate(&evento) {
            return evento;
        }
    }
}

#[test]
fn scopes_memory_disassembly_and_rtt_come_through_the_dap_verbatim() {
    let root = std::env::temp_dir().join(format!("kinein-inspect-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("service.py"), "print(42)\n").unwrap();
    let (events, receiver) = mpsc::channel();
    let mut core = super::core_with_empty_search_path("debug-inspect");
    core.enable_lsp(events);
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": root }))
        ))
        .response()
        .error
        .is_none()
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = fake_probe_rs(listener);
    let started = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "debug.start",
        Some(json!({ "connect": { "host": "127.0.0.1", "port": port } })),
    ));
    assert!(
        started.response().error.is_none(),
        "{:?}",
        started.response()
    );

    // O RTT chega como saida de debug com categoria `rtt` e o canal; o aviso
    // do probe-rs vira console; a parada chega enriquecida com o frame.
    let canal = espera(&receiver, &|e| {
        e.method == "event.debug.output" && e.params.as_ref().unwrap()["category"] == "rtt"
    });
    let p = canal.params.unwrap();
    assert_eq!(p["channel"], 0);
    assert_eq!(p["channelName"], "Terminal");
    let dados = espera(&receiver, &|e| {
        e.method == "event.debug.output" && e.params.as_ref().unwrap()["line"] == "led on"
    });
    assert_eq!(dados.params.unwrap()["category"], "rtt");
    let aviso = espera(&receiver, &|e| {
        e.method == "event.debug.output"
            && e.params.as_ref().unwrap()["line"]
                .as_str()
                .unwrap()
                .contains("no RTT control block")
    });
    assert_eq!(aviso.params.unwrap()["category"], "console");
    let parada = espera(&receiver, &|e| e.method == "event.debug.stopped");
    assert_eq!(parada.params.unwrap()["line"], 3);

    // Os escopos inteiros, na ordem do adaptador.
    let r = core
        .handle_request(&JsonRpcRequest::new(
            4_i64,
            "debug.scopes",
            Some(json!({ "frameId": 7 })),
        ))
        .response()
        .result
        .clone()
        .expect("scopes");
    assert_eq!(r["frameId"], 7);
    let scopes = r["scopes"].as_array().unwrap();
    assert_eq!(scopes.len(), 3);
    assert_eq!(scopes[2]["name"], "Peripherals");
    assert_eq!(scopes[2]["ref"], 13);
    assert_eq!(scopes[2]["expensive"], true);
    assert_eq!(scopes[0]["expensive"], false);

    // Memoria e disassembly, verbatim.
    let r = core
        .handle_request(&JsonRpcRequest::new(
            5_i64,
            "debug.readMemory",
            Some(json!({ "memoryReference": "0x3ff00000", "offset": 4, "count": 4 })),
        ))
        .response()
        .result
        .clone()
        .expect("readMemory");
    assert_eq!(r["address"], "0x3ff00000");
    assert_eq!(r["data"], "AQIDBA==");
    let r = core
        .handle_request(&JsonRpcRequest::new(6_i64, "debug.disassemble", Some(json!({ "memoryReference": "0x40080000", "instructionOffset": -1, "instructionCount": 2 }))))
        .response()
        .result
        .clone()
        .expect("disassemble");
    let ins = r["instructions"].as_array().unwrap();
    assert_eq!(ins.len(), 2);
    assert_eq!(ins[0]["symbol"], "main");
    assert_eq!(ins[0]["file"], "/w/main.c");
    assert_eq!(ins[0]["line"], 3);
    assert_eq!(ins[1]["instruction"], "movi a2, 1");
    assert!(ins[1].get("symbol").is_none(), "{}", ins[1]);

    assert!(
        core.handle_request(&JsonRpcRequest::new(7_i64, "debug.stop", None))
            .response()
            .error
            .is_none()
    );
    conferir_pedidos(&server.join().unwrap());
}

/// O que o adaptador RECEBEU: o handshake do RTT e os argumentos dos tres
/// pedidos, como o DAP os define.
fn conferir_pedidos(requests: &[Value]) {
    // O handshake do RTT foi mandado ao adaptador, e os argumentos dos tres
    // pedidos foram passados como o DAP os define.
    let rtt = requests
        .iter()
        .find(|r| r["command"] == "rttWindowOpened")
        .expect("rttWindowOpened");
    assert_eq!(
        rtt["arguments"],
        json!({ "channelNumber": 0, "windowIsOpen": true })
    );
    let mem = requests
        .iter()
        .find(|r| r["command"] == "readMemory")
        .unwrap();
    assert_eq!(
        mem["arguments"],
        json!({ "memoryReference": "0x3ff00000", "offset": 4, "count": 4 })
    );
    let dis = requests
        .iter()
        .find(|r| r["command"] == "disassemble")
        .unwrap();
    assert_eq!(
        dis["arguments"],
        json!({ "memoryReference": "0x40080000", "instructionOffset": -1, "instructionCount": 2 })
    );
    let sc = requests.iter().find(|r| r["command"] == "scopes").unwrap();
    assert_eq!(sc["arguments"]["frameId"], 7);
    assert!(requests.iter().any(|r| r["command"] == "disconnect"));
}

/// Sem sessao os tres metodos recusam (`NotRunning`); parametros vazios sao
/// recusados antes.
#[test]
fn inspection_methods_refuse_without_a_session_and_validate_params() {
    let root = std::env::temp_dir().join(format!("kinein-inspect-recusa-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("service.py"), "print(42)\n").unwrap();
    let (events, _receiver) = mpsc::channel();
    let mut core = super::core_with_empty_search_path("debug-inspect-recusa");
    core.enable_lsp(events);
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": root }))
        ))
        .response()
        .error
        .is_none()
    );
    for (m, p) in [
        ("debug.scopes", json!({ "frameId": 7 })),
        (
            "debug.readMemory",
            json!({ "memoryReference": "0x3ff00000", "count": 4 }),
        ),
        (
            "debug.disassemble",
            json!({ "memoryReference": "0x40080000", "instructionCount": 2 }),
        ),
    ] {
        let e = core
            .handle_request(&JsonRpcRequest::new(2_i64, m, Some(p)))
            .response()
            .error
            .clone()
            .unwrap();
        assert_eq!(e.code, JsonRpcErrorCode::InvalidRequest, "{m}");
    }
    let e = core
        .handle_request(&JsonRpcRequest::new(
            2_i64,
            "debug.readMemory",
            Some(json!({ "memoryReference": "", "count": 0 })),
        ))
        .response()
        .error
        .clone()
        .unwrap();
    assert_eq!(e.code, JsonRpcErrorCode::InvalidParams);
}
