//! Adaptador EXTERNO do `SQLite` (passo 9a.1, `DocsPublic/arquitetura/39` §6.2).
//!
//! Construido neste repositorio e instalado ao lado do `kinein-core` (a
//! instalacao "da IDE", decisao do autor no 39 §5.2). Fala a API de adaptadores
//! 1.1 do 39 §4.4 e do 40: uma linha JSON por mensagem no stdin/stdout; o
//! stderr e' so' diagnostico, que o core guarda numa cauda limitada.
//!
//! O que ele NAO faz: decidir politica (o core ja' autorizou o texto),
//! interpretar o catalogo em instrucoes (o core gera `statements`), ou
//! prever/decidir (o `SQLite` nao tem a previa transacional do `PostgreSQL`).

pub mod budget;
pub mod engine;

use kinein_protocol::driver::operation::Request;
use kinein_protocol::driver::{
    ApiRange, InitializeParams, InitializeResult, Limits, Operation, Version,
};
use kinein_protocol::{JsonRpcId, JsonRpcRequest};
use serde_json::{Value, json};

/// A identidade que o core confere no handshake.
pub const ADAPTER_ID: &str = "kinein.sqlite";

/// A faixa que este adaptador fala: 1.1 traz a mensagem do banco.
pub const API: ApiRange = ApiRange {
    min: Version { major: 1, minor: 0 },
    max: Version { major: 1, minor: 1 },
};

/// O que este adaptador aguenta; o efetivo e' o menor de cada lado.
pub const LIMITS: Limits = Limits {
    message_bytes: 1_048_576,
    in_flight: 2,
    rows: 10_000,
    columns: 128,
    cell_bytes: 16_384,
    retained_bytes: 8_388_608,
    catalogue_items: 5_000,
};

/// As operacoes anunciadas; `preview`/`decide` nao existem no `SQLite`.
pub const OPERATIONS: [Operation; 8] = [
    Operation::Open,
    Operation::Test,
    Operation::Introspect,
    Operation::Query,
    Operation::Impact,
    Operation::Cancel,
    Operation::Close,
    Operation::Shutdown,
];

/// Uma linha do core vira as mensagens de resposta; `true` encerra.
#[must_use]
pub fn handle(adapter: &mut engine::Adapter, line: &str) -> (Vec<Value>, bool) {
    let Ok(request) = serde_json::from_str::<JsonRpcRequest>(line) else {
        return (
            vec![protocol_error(&JsonRpcId::Null, -32700, "linha invalida")],
            false,
        );
    };
    let id = request.id.clone().unwrap_or(JsonRpcId::Null);
    if request.method == "driver.initialize" {
        let Some(params) = request
            .params
            .and_then(|params| serde_json::from_value::<InitializeParams>(params).ok())
        else {
            return (
                vec![protocol_error(&id, -32602, "initialize invalido")],
                false,
            );
        };
        adapter.negotiate(&params, API, LIMITS);
        let result = InitializeResult {
            api: API,
            adapter_id: ADAPTER_ID.to_owned(),
            adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
            driver_version: Some(engine::sqlite_version()),
            engines: vec!["sqlite".to_owned()],
            operations: OPERATIONS.to_vec(),
            limits: LIMITS,
        };
        return (
            vec![json!({ "jsonrpc": "2.0", "id": id, "result": result })],
            false,
        );
    }
    let typed = json!({ "method": request.method, "params": request.params });
    let Ok(typed) = serde_json::from_value::<Request>(typed) else {
        return (
            vec![protocol_error(
                &id,
                -32601,
                "metodo ou parametros desconhecidos",
            )],
            false,
        );
    };
    let finished = matches!(typed, Request::Shutdown(_));
    (adapter.run(&id, typed), finished)
}

/// Erro do protocolo (nao do banco): codigo JSON-RPC padrao, sem `data`.
fn protocol_error(id: &JsonRpcId, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
