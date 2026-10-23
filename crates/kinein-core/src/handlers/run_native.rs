//! Native run validation returns to the existing PTY owner through `defer_then`.
use super::run::Invocation;
use crate::{Core, flash, runconfig};
use kinein_protocol::{
    FlashIntent, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ProjectModel, RunConfigInfo,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

impl Core {
    pub(super) fn explicit_flash_response(
        &mut self,
        request_id: Option<Value>,
        params: kinein_protocol::RunStartParams,
    ) -> JsonRpcResponse {
        if params.command.is_some() || params.device.is_some() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "run.start: flash é exclusivo com command/device",
                    None,
                ),
            );
        }
        let Some(flash) = params.flash else {
            return refusal(request_id, "Intenção nativa ausente.");
        };
        self.run_flash_response(request_id, flash, None)
    }

    pub(super) fn native_flash_proposal_response(
        &self,
        request_id: Option<Value>,
        root: std::path::PathBuf,
        model: ProjectModel,
        selection: kinein_protocol::FlashProposalParams,
        firmware: Option<flash::FirmwareImage>,
    ) -> JsonRpcResponse {
        let engine = match flash::native::engine(&model, &selection, firmware.as_ref()) {
            Ok(engine) => engine,
            Err(error) => return super::runconfig::flash_error_response(request_id, &error),
        };
        if engine == "esptool"
            && let Some(device) = selection.device.as_deref()
            && let Err(response) =
                super::serial::ready_port(request_id.as_ref(), "runConfig.flashProposal", device)
        {
            return *response;
        }
        let tool = match self.native_tool(&root, &engine) {
            Ok(tool) => tool,
            Err(error) => return super::runconfig::flash_error_response(request_id, &error),
        };
        let input = self.native_request(&root, model, selection, firmware, tool);
        self.defer_work(request_id, move |request_id| {
            match flash::native::prepare(&root, input, &Arc::new(AtomicBool::new(false))) {
                Ok(prepared) => JsonRpcResponse::success(request_id, json!(prepared.plan.proposal)),
                Err(error) => super::runconfig::flash_error_response(request_id, &error),
            }
        })
    }

    fn native_kit(&self, root: &Path) -> (String, crate::toolchain::Toolchain) {
        let preset = crate::cmake::active_kit(root);
        let kit = crate::toolchain::Toolchain::resolve_kit(root, &self.detected_tools(), &preset);
        (preset, kit)
    }

    pub(super) fn native_flash_context(&self, root: &Path, model: &ProjectModel) -> Value {
        let (preset, kit) = self.native_kit(root);
        json!([
            preset,
            kit.to_result(),
            model.target,
            model.frameworks,
            model.sdks
        ])
    }

    /// esptool from the detector; probe-rs from the kit's debugAdapter role,
    /// the same executable `probe.list` asks.
    fn native_tool(&self, root: &Path, engine: &str) -> Result<PathBuf, flash::FlashError> {
        let missing = |tool: &str, hint: &str| flash::FlashError::MissingTool {
            tool: tool.to_owned(),
            hint: hint.to_owned(),
        };
        if engine == "probe-rs" {
            let tool = self.probe_program(root).ok_or_else(|| {
                missing(
                    "probe-rs",
                    "instale pelo catálogo ou fixe o adaptador de depuração no kit",
                )
            })?;
            if tool.file_stem().and_then(|stem| stem.to_str()) != Some("probe-rs") {
                return Err(missing(
                    "probe-rs",
                    &format!(
                        "o adaptador de depuração do kit é {}; fixe o probe-rs para gravar por sonda",
                        tool.display()
                    ),
                ));
            }
            return Ok(tool);
        }
        self.detector
            .find_in_path("esptool")
            .or_else(|| self.detector.find_in_path("esptool.py"))
            .ok_or_else(|| missing("esptool", "instale pelo catálogo de ferramentas"))
    }

    fn native_request(
        &self,
        root: &Path,
        model: ProjectModel,
        selection: kinein_protocol::FlashProposalParams,
        firmware: Option<flash::FirmwareImage>,
        tool: PathBuf,
    ) -> flash::native::NativeRequest {
        let context = self.native_flash_context(root, &model);
        flash::native::NativeRequest {
            kit_chip: self.native_kit(root).1.chip().map(str::to_owned),
            model,
            context,
            selection,
            firmware,
            tool,
        }
    }

    pub(crate) fn cancel_run_preflight(&mut self) -> bool {
        self.run_preflight.take().is_some_and(|cancel| {
            cancel.store(true, Ordering::SeqCst);
            true
        })
    }

    pub(super) fn run_flash_response(
        &mut self,
        request_id: Option<Value>,
        intent: FlashIntent,
        active: Option<RunConfigInfo>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return crate::rpc::no_workspace_response(request_id, "run.start");
        };
        if let Err(message) = flash::native::validate_intent(&root, &intent, None) {
            return refusal(request_id, &message);
        }
        let Some(terminal) = &self.terminal else {
            return crate::rpc::terminal_unavailable_response(request_id, "run.start");
        };
        if self
            .run_terminal
            .as_ref()
            .is_some_and(|id| terminal.is_open(id))
        {
            return refusal(
                request_id,
                "Uma execução já está em andamento; pare-a antes de gravar.",
            );
        }
        let probe = intent.selection.probe.clone();
        let reservation = if let Some(probe) = probe.as_deref() {
            match self.serial_usage.acquire_probe(
                probe,
                "run.start",
                "Gravação nativa por sonda",
                Some(&root),
            ) {
                Ok(reservation) => reservation,
                Err(error) => return JsonRpcResponse::failure(request_id, error),
            }
        } else {
            match self.claim_serial_destination(request_id.as_ref(), &intent) {
                Ok(reservation) => reservation,
                Err(response) => return *response,
            }
        };
        let firmware = match intent
            .selection
            .firmware
            .as_deref()
            .map(|id| self.firmware_image(id))
        {
            Some(Err(message)) => return refusal(request_id, &message),
            Some(Ok(image)) => Some(image),
            None => None,
        };
        let model = self.compute_project_model(&root);
        let context = self.native_flash_context(&root, &model);
        let engine = intent.selection.engine.clone().unwrap_or_default();
        let Ok(tool) = self.native_tool(&root, &engine) else {
            return refusal(
                request_id,
                "A ferramenta de gravação selecionada não está disponível; revise as ferramentas e gere outra prévia.",
            );
        };
        let input = self.native_request(&root, model, intent.selection.clone(), firmware, tool);
        let cancel = Arc::new(AtomicBool::new(false));
        self.run_preflight = Some(cancel.clone());
        let wait_cancel = cancel.clone();
        let wait_root = root.clone();
        self.defer_then(request_id, move || (flash::native::prepare(&wait_root, input, &wait_cancel), reservation),
            move |core, request_id, (prepared, reservation)| {
                if cancel.load(Ordering::SeqCst)
                    || !core.run_preflight.as_ref().is_some_and(|pending| Arc::ptr_eq(pending, &cancel)) {
                    return refusal(request_id, "A preparação foi cancelada; nenhuma gravação foi iniciada.");
                }
                core.run_preflight = None;
                let prepared = match prepared { Ok(prepared) => prepared,
                    Err(error) => return super::runconfig::flash_error_response_for("run.start", request_id, &error), };
                if core.workspace_root().as_ref() != Some(&root) { return refusal(request_id, "O workspace mudou; gere outra prévia."); }
                let model = core.compute_project_model(&root);
                if core.native_flash_context(&root, &model) != context || !prepared.unchanged() || prepared.intent != intent {
                    return refusal(request_id, "Contexto, ferramenta, porta ou arquivos mudaram desde a prévia; revise e gere outra antes de gravar.");
                }
                if let Err(message) = check_active(&root, active.as_ref()) {
                    return refusal(request_id, &message);
                }
                if probe.is_none() {
                    let device = intent.selection.device.as_deref().unwrap_or_default();
                    if let Err(response) = super::serial::ready_port(request_id.as_ref(), "run.start", device) { return *response; }
                    if crate::platform::serial_device_revision(Path::new(device)).ok().as_ref() != Some(&intent.device_key) {
                        return refusal(request_id, "A identidade da porta mudou; gere outra prévia.");
                    }
                }
                let identity = active.as_ref().map_or_else(|| json!(["flash", root, intent.fingerprint]), |config| json!(["config", root, config.id]));
                let removed = if probe.is_some() { flash::probe_rs::inherited_variables() } else { flash::esptool::inherited_variables() };
                core.start_in_terminal(request_id, &root, Invocation {
                    program: prepared.plan.command.program.display().to_string(), args: prepared.plan.command.args,
                    command: prepared.plan.proposal.command, identity, reservation: Some(reservation), removed,
                })
            })
    }
}

impl Core {
    /// esptool: the bound port must be ready and keep its identity, then reserved.
    fn claim_serial_destination(
        &self,
        request_id: Option<&Value>,
        intent: &FlashIntent,
    ) -> Result<crate::serial::ownership::Lease, Box<JsonRpcResponse>> {
        let device = intent.selection.device.as_deref().unwrap_or_default();
        super::serial::ready_port(request_id, "run.start", device)?;
        if crate::platform::serial_device_revision(Path::new(device))
            .ok()
            .as_ref()
            != Some(&intent.device_key)
        {
            return Err(Box::new(refusal(
                request_id.cloned(),
                "A porta mudou desde a prévia; selecione o dispositivo e gere outra.",
            )));
        }
        self.claim_serial_port(device, "run.start", "Gravação nativa")
            .map_err(|error| Box::new(JsonRpcResponse::failure(request_id.cloned(), error)))
    }
}

pub(super) fn refusal(request_id: Option<Value>, message: &str) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(JsonRpcErrorCode::InvalidRequest, message, None),
    )
}

fn check_active(root: &Path, active: Option<&RunConfigInfo>) -> Result<(), String> {
    let Some(active) = active else {
        return Ok(());
    };
    let state = runconfig::load(root).map_err(|error| error.to_string())?;
    if state.active_id.as_ref() != Some(&active.id)
        || !state.configs.iter().any(|config| {
            config.id == active.id
                && config.flash == active.flash
                && config.command == active.command
        })
    {
        return Err(
            "A configuração ativa mudou durante a preparação; confirme a seleção antes de gravar."
                .to_owned(),
        );
    }
    Ok(())
}
