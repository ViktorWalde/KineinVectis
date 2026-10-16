//! `workspace.trust` e o que a confianca liga (16.4b, `roadmaps/59` §7.1.4).
//!
//! Sem confianca, nada do projeto roda sozinho: o configure automatico do
//! `CMake` e' da UI (ela le `trusted`), e o rust-analyzer sobe aqui sem build
//! scripts, sem macros procedurais e sem o `cargo check` ao salvar — o
//! `cargo check` tambem compila e roda os build scripts. Realce, navegacao e
//! completar continuam; o resto volta no clique em "Confiar neste projeto".

use kinein_protocol::{JsonRpcErrorCode, JsonRpcResponse, WorkspaceTrustResult};
use serde_json::{Value, json};

use crate::rpc::no_workspace_response;
use crate::{Core, workspace};

/// A configuracao do rust-analyzer: o alvo do kit (`cargo.target`) e, sem a
/// confianca, `cargo.buildScripts.enable`, `procMacro.enable` e `checkOnSave`
/// falsos (manual do rust-analyzer, "Configuration"). Devolve os `settings`
/// (so' quando o kit empurra um alvo, como antes) e as `initializationOptions`
/// (sempre que ha' algo a dizer: e' o que ele le' antes de carregar o projeto).
pub(crate) fn rust_analyzer_config(target_triple: Option<&str>, trusted: bool) -> (Value, Value) {
    let target = target_triple.filter(|triple| !triple.trim().is_empty());
    let mut config = json!({});
    if let Some(triple) = target {
        config["cargo"] = json!({ "target": triple });
    }
    if !trusted {
        if config.get("cargo").is_none() {
            config["cargo"] = json!({});
        }
        config["cargo"]["buildScripts"] = json!({ "enable": false });
        config["procMacro"] = json!({ "enable": false });
        config["checkOnSave"] = json!(false);
    }
    let settings = target.map_or(Value::Null, |_| json!({ "rust-analyzer": config.clone() }));
    let options = if config.as_object().is_some_and(serde_json::Map::is_empty) {
        Value::Null
    } else {
        config
    };
    (settings, options)
}

impl Core {
    /// Roteia `workspace.trust`.
    pub(crate) fn workspace_trust_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "workspace.trust" => Some(self.trust_workspace(request_id)),
            _ => None,
        }
    }

    /// `workspace.trust {}` -> `{ trusted: true }`: grava a raiz aberta fora do
    /// projeto e sobe o rust-analyzer de novo, completo.
    fn trust_workspace(&mut self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(root) = self.workspace.as_ref().map(|open| open.root.clone()) else {
            return no_workspace_response(request_id, "workspace.trust");
        };
        if let Some(global_storage) = self.global_storage.as_deref()
            && let Err(reason) = workspace::trust_in(global_storage, &root)
        {
            return JsonRpcResponse::failure(
                request_id,
                kinein_protocol::JsonRpcError::new(JsonRpcErrorCode::InternalError, reason, None),
            );
        }
        if let Some(open) = self.workspace.as_mut() {
            open.trusted = true;
        }
        // O mesmo caminho do kit: com o alvo dele, agora sem as travas; o
        // servidor vivo e' reiniciado.
        let toolchain = crate::toolchain::Toolchain::resolve(
            std::path::Path::new(&root),
            &self.detected_tools(),
        );
        let target = toolchain.target_triple().map(str::to_owned);
        self.configure_rust_lsp_from_toolchain(target.as_deref());
        JsonRpcResponse::success(request_id, json!(WorkspaceTrustResult { trusted: true }))
    }

    /// A pessoa ja' confiou em `root`? So' o estado global responde; sem ele
    /// (os testes do core), nenhuma raiz e' confiavel ate' o `workspace.trust`.
    pub(crate) fn root_is_trusted(&self, root: &str) -> bool {
        self.global_storage
            .as_deref()
            .is_some_and(|global_storage| workspace::is_trusted_in(global_storage, root))
    }

    /// Grava a confianca em `root` sem resposta (o projeto que a IDE criou).
    pub(crate) fn remember_trust(&self, root: &str) {
        if let Some(global_storage) = self.global_storage.as_deref() {
            drop(workspace::trust_in(global_storage, root));
        }
    }
}
