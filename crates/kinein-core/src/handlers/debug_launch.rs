//! Launching a debug session (`impl Core`): kit, adapter choice and the
//! existing `DebugManager`, plus `debug.start { probe }` (0.190.0, `roadmaps/52`
//! §14.36), where the probe-rs session uses and reserves the chosen probe.
//! Split from `debug.rs` (catraca de 500 linhas) by responsibility.
use super::debug::{invalid_debug_params, is_python_target};
use crate::rpc::{debug_error_response, debug_unavailable_response};
use crate::{Core, dap, flash};
use kinein_protocol::{
    DebugStartResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ToolchainRole,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

impl Core {
    /// Kit, adaptador e o `DebugManager` existente; `probe` (0.190.0) traz o
    /// seletor validado e a reserva que acompanha o adaptador.
    pub(super) fn debug_launch(
        &mut self,
        request_id: Option<Value>,
        root: &Path,
        target: &dap::DebugTarget,
        probe: Option<(String, crate::serial::ownership::Lease)>,
    ) -> JsonRpcResponse {
        let target = target.clone();
        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        let python = is_python_target(&target);
        let interpreter = if python && !target.is_attached() {
            match self.python_debug_interpreter(root) {
                Ok(path) => Some(path),
                Err(error) => return debug_error_response(request_id, &error),
            }
        } else {
            None
        };
        // Python never inherits a native kit's remoteTarget/debugServer/chip.
        // TCP attach uses the adapter that debugpy.listen already started.
        let native_path = toolchain.program_for(kinein_protocol::ToolchainRole::DebugAdapter);
        // O GDB certo para o ELF (pente-fino 2026-09-18): o `gdb` nativo nao
        // fala ARM/Xtensa/RISC-V e o attach ficaria mudo; um GDB de alvo
        // detectado entra no lugar, ou a recusa diz o pacote.
        let (gdb_id, gdb_path) = match self.gdb_for_target(&toolchain, &target) {
            Ok(picked) => picked,
            Err(error) => return debug_error_response(request_id, &error),
        };
        let choice = if python {
            dap::AdapterChoice {
                id: Some(dap::DEBUGPY),
                path: interpreter.as_deref(),
                ..dap::AdapterChoice::default()
            }
        } else {
            native_choice(
                &toolchain,
                gdb_id.as_deref(),
                gdb_path.as_deref(),
                native_path.as_deref(),
            )
        };
        let (selector, reservation) = probe.map_or((None, None), |(selector, lease)| {
            (Some(selector), Some(lease))
        });
        let choice = dap::AdapterChoice {
            probe: selector.as_deref(),
            ..choice
        };
        let Some(manager) = self.debug.as_mut() else {
            return debug_unavailable_response(request_id, "debug.start");
        };
        match manager.start(root, &target, &choice, reservation) {
            Ok(()) => JsonRpcResponse::success(
                request_id,
                json!(DebugStartResult {
                    program: target.display(),
                    attached: target.is_attached(),
                }),
            ),
            Err(error) => debug_error_response(request_id, &error),
        }
    }

    /// `Some((id, path))` quando o `gdb` nu deve dar lugar a um GDB de alvo
    /// (o ELF e' de outra maquina); `None` = a escolha do kit fica. A troca e'
    /// dita no console de debug; a falta e' erro com o pacote.
    fn gdb_for_target(
        &self,
        toolchain: &crate::toolchain::Toolchain,
        target: &dap::DebugTarget,
    ) -> Result<(Option<String>, Option<String>), dap::DebugError> {
        let Some(program) = target.program_path() else {
            return Ok((None, None));
        };
        let effective = toolchain
            .effective_program(kinein_protocol::ToolchainRole::DebugAdapter)
            .map(|(id, _)| id.to_owned())
            .or_else(|| {
                toolchain
                    .chosen(kinein_protocol::ToolchainRole::DebugAdapter)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| dap::default_adapter().to_owned());
        match dap::gdb_pick::pick_gdb(&effective, program, &self.detected_tools()) {
            dap::gdb_pick::GdbPick::Keep => Ok((None, None)),
            dap::gdb_pick::GdbPick::Replace { id, path, reason } => {
                if let Some(events) = self.events.as_ref() {
                    drop(events.send(kinein_protocol::JsonRpcRequest::notification(
                        "event.debug.output",
                        Some(json!({ "category": "console", "line": reason })),
                    )));
                }
                Ok((Some(id), Some(path)))
            }
            dap::gdb_pick::GdbPick::Unavailable { message } => {
                Err(dap::DebugError::NoTarget { message })
            }
        }
    }

    pub(super) fn debug_probe_start_response(
        &mut self,
        request_id: Option<Value>,
        root: PathBuf,
        target: dap::DebugTarget,
        probe: String,
    ) -> JsonRpcResponse {
        if target.is_attached() || target.program_path().is_none() || is_python_target(&target) {
            return invalid_debug_params(
                request_id,
                "probe vale só para depurar um programa nativo pelo probe-rs, não para Python ou connect",
            );
        }
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let Some(tool) = toolchain
            .program_for(ToolchainRole::DebugAdapter)
            .filter(|tool| tool.file_stem().and_then(|stem| stem.to_str()) == Some("probe-rs"))
        else {
            return invalid_debug_params(
                request_id,
                "probe vale só com o probe-rs como adaptador de depuração do kit",
            );
        };
        if let Err(message) = flash::probe_rs::Selector::parse(&probe) {
            return invalid_debug_params(request_id, &message);
        }
        if self
            .debug
            .as_ref()
            .is_some_and(dap::DebugManager::is_running)
        {
            return refusal(
                request_id,
                "Uma depuração já está em andamento; pare-a antes de iniciar outra.",
            );
        }
        let lease =
            match self
                .serial_usage
                .acquire_probe(&probe, "debug.start", "Depuração", Some(&root))
            {
                Ok(lease) => lease,
                Err(error) => return JsonRpcResponse::failure(request_id, error),
            };
        let (query_root, query_probe) = (root.clone(), probe.clone());
        self.defer_then(
            request_id,
            move || {
                let checked = flash::probe_rs::listed_probe(
                    &tool,
                    &query_root,
                    &query_probe,
                    &Arc::new(AtomicBool::new(false)),
                );
                (checked, lease)
            },
            move |core, request_id, (checked, lease)| {
                if let Err(message) = checked {
                    return refusal(
                        request_id,
                        &format!("Não foi possível validar a sonda da depuração: {message}"),
                    );
                }
                if core.workspace_root().as_ref() != Some(&root) {
                    return refusal(request_id, "O workspace mudou; inicie a depuração de novo.");
                }
                core.debug_launch(request_id, &root, &target, Some((probe, lease)))
            },
        )
    }
}

/// A escolha nativa: o GDB de alvo (quando o ELF pediu) vence o do kit.
fn native_choice<'a>(
    toolchain: &'a crate::toolchain::Toolchain,
    gdb_id: Option<&'a str>,
    gdb_path: Option<&'a str>,
    native_path: Option<&'a Path>,
) -> dap::AdapterChoice<'a> {
    dap::AdapterChoice {
        id: gdb_id.or_else(|| toolchain.chosen(kinein_protocol::ToolchainRole::DebugAdapter)),
        path: gdb_path.map(Path::new).or(native_path),
        chip: toolchain.chip(),
        remote_target: toolchain.remote_target(),
        debug_server: toolchain.debug_server(),
        svd_file: toolchain.svd_file(),
        probe: None,
    }
}

fn refusal(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, message, None),
    )
}
