//! Native esptool intent: establish provenance, regenerate the existing plan, snapshot it.
//! Execution remains with run.start and the existing terminal/serial owners.
use super::{FirmwareImage, FlashCommand, FlashError, FlashPlan, ProposalTools, snapshot};
use kinein_protocol::{FlashIntent, FlashProposalParams, Framework, ProjectModel};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

/// Immutable inputs captured from the current Core selection before deferred IO.
#[derive(Debug)]
pub struct NativeRequest {
    /// Current project model from its existing owner.
    pub model: ProjectModel,
    /// Effective kit/preset and environment context, compared again before launch.
    pub context: Value,
    /// Explicit native proposal selection.
    pub selection: FlashProposalParams,
    /// Installed catalogue firmware, when requested.
    pub firmware: Option<FirmwareImage>,
    /// The executable selected by the existing detector.
    pub tool: PathBuf,
}

/// Native snapshot with the original provider plan and file observations.
#[derive(Debug)]
pub struct Prepared {
    /// Recomputed command; never a shell parser of received program/argv.
    pub plan: FlashPlan,
    /// Reviewable intent bound to the current inputs.
    pub intent: FlashIntent,
    files: Vec<snapshot::Observation>,
}

impl Prepared {
    /// Verify that files did not change between deferred validation and PTY handoff.
    #[must_use]
    pub fn unchanged(&self) -> bool {
        self.files.iter().all(snapshot::Observation::unchanged)
    }
}

/// Resolve native support before querying an unrelated executable.
///
/// # Errors
/// Refuses engines whose native destination validation is still pending.
pub fn engine(
    model: &ProjectModel,
    selection: &FlashProposalParams,
    firmware: Option<&FirmwareImage>,
) -> Result<String, FlashError> {
    let (engine, _) = super::select_engine(model, selection.engine.as_deref(), firmware)?;
    if engine != "esptool" {
        return Err(invalid(
            "este motor ainda não tem validação de destino nativa; a integração permanece pendente, sem converter a operação em shell",
        ));
    }
    Ok(engine)
}

/// Check static intent invariants and its display command without executing anything.
///
/// # Errors
/// Refuses unknown versions, malformed snapshots, foreign workspaces or edited previews.
pub fn validate_intent(
    root: &Path,
    intent: &FlashIntent,
    command: Option<&str>,
) -> Result<(), String> {
    if intent.version != 1
        || Path::new(&intent.workspace) != root
        || !intent.selection.native
        || intent.selection.engine.as_deref() != Some("esptool")
        || intent
            .selection
            .device
            .as_deref()
            .is_none_or(|device| device.trim().is_empty())
        || intent.device_key.is_empty()
        || !Path::new(&intent.program).is_absolute()
        || intent.args.is_empty()
        || intent.program.contains('\0')
        || intent.args.iter().any(|arg| arg.contains('\0'))
        || intent.fingerprint.len() != 64
        || !intent
            .fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("intenção de gravação inválida/sem suporte ou de outro workspace; gere outra prévia nativa".to_owned());
    }
    let preview = FlashCommand {
        program: intent.program.clone().into(),
        args: intent.args.clone(),
    }
    .preview();
    if command.is_some_and(|command| command != preview) {
        return Err(
            "o comando foi alterado sem uma nova intenção de gravação; gere outra prévia"
                .to_owned(),
        );
    }
    Ok(())
}

/// Produce a native snapshot using the existing version adapter and command planner.
///
/// # Errors
/// Refuses incomplete provenance, changed files/port, tool/cancellation failures or unsupported engines.
pub fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Prepared, FlashError> {
    let engine = engine(
        &request.model,
        &request.selection,
        request.firmware.as_ref(),
    )?;
    let device = super::chosen_port(request.selection.device.as_deref())?.to_owned();
    let device_key = crate::platform::serial_device_revision(Path::new(&device))
        .map_err(|error| invalid(&error.to_string()))?;
    let tool_revision = crate::platform::file_revision(&request.tool)
        .map_err(|error| invalid(&error.to_string()))?;
    let selected = super::esptool::probe_cancelable(
        request.tool.clone(),
        root,
        Duration::from_secs(5),
        cancel.clone(),
    )?;
    let sources = collect_sources(root, &mut request)?;
    request.selection.native = true;
    request.selection.engine = Some(engine);
    request.selection.device = Some(device);
    let tools = ProposalTools {
        find: &|_| None,
        esptool: Some(selected),
    };
    let mut plan = super::plan(
        &request.model,
        Some("esptool"),
        request.selection.device.as_deref(),
        request.selection.flash_size_bytes,
        request.firmware.as_ref(),
        None,
        &tools,
    )?;
    let captured = capture_files(&request, &sources, &tool_revision, cancel)?;
    let fingerprint = snapshot::digest(
        &serde_json::to_vec(&json!([
            root,
            request.context,
            request.selection,
            request.model.target,
            request.model.frameworks,
            request.model.sdks,
            plan.command.program,
            plan.command.args,
            plan.proposal.source,
            device_key,
            captured.contents
        ]))
        .map_err(|error| invalid(&error.to_string()))?,
    );
    let intent = FlashIntent {
        version: 1,
        workspace: root.display().to_string(),
        selection: request.selection,
        program: plan.command.program.display().to_string(),
        args: plan.command.args.clone(),
        device_key,
        fingerprint,
    };
    validate_intent(root, &intent, Some(&plan.proposal.command))
        .map_err(|message| invalid(&message))?;
    plan.proposal.source.push(
        "origem e conteúdo vinculados ao contexto; snapshot revalidado antes da execução"
            .to_owned(),
    );
    plan.proposal.flash = Some(intent.clone());
    let prepared = Prepared {
        plan,
        intent,
        files: captured.files,
    };
    if !prepared.unchanged() {
        return Err(invalid("arquivos mudaram ao preparar a prévia; gere outra"));
    }
    Ok(prepared)
}

struct SourceFiles {
    metadata: Vec<(PathBuf, String)>,
    images: Vec<PathBuf>,
}

fn collect_sources(root: &Path, request: &mut NativeRequest) -> Result<SourceFiles, FlashError> {
    let mut sources = Vec::new();
    let mut images = Vec::new();
    if let Some(image) = &request.firmware {
        if request.selection.build_directory.is_some() {
            return Err(invalid("firmware de catálogo não usa buildDirectory"));
        }
        if request
            .model
            .target
            .chip
            .as_ref()
            .zip(image.chip.as_ref())
            .is_some_and(|(left, right)| left != right)
        {
            return Err(invalid(
                "o firmware e o alvo atual têm chips diferentes; revise a seleção antes de gravar",
            ));
        }
        images.push(image.file.clone());
    } else {
        if request
            .model
            .frameworks
            .first()
            .is_none_or(|framework| framework.framework != Framework::EspIdf)
        {
            return Err(invalid(
                "a receita nativa requer um projeto ESP-IDF reconhecido; selecione o contexto correto",
            ));
        }
        let origin = crate::project::flash_origin::validate(
            root,
            request.selection.build_directory.as_deref(),
            &request.model,
        )
        .map_err(|message| invalid(&message))?;
        images.extend(
            origin
                .recipe
                .files
                .iter()
                .map(|image| PathBuf::from(&image.file)),
        );
        request.model.artifacts.flasher_args =
            Some(origin.build.join("flasher_args.json").display().to_string());
        request.model.artifacts.flash_recipe = Some(origin.recipe);
        request.selection.build_directory = Some(origin.build.display().to_string());
        sources = origin.sources;
        let project_file = root.join("CMakeLists.txt");
        sources.push((
            project_file.clone(),
            crate::platform::file_revision(&project_file)
                .map_err(|error| invalid(&error.to_string()))?,
        ));
    }
    Ok(SourceFiles {
        metadata: sources,
        images,
    })
}

struct CapturedFiles {
    contents: Vec<Value>,
    files: Vec<snapshot::Observation>,
}

fn capture_files(
    request: &NativeRequest,
    sources: &SourceFiles,
    tool_revision: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<CapturedFiles, FlashError> {
    let mut contents = Vec::new();
    let mut files = Vec::new();
    for (path, expected) in sources
        .metadata
        .iter()
        .map(|(path, revision)| (path, Some(revision.as_str())))
        .chain(sources.images.iter().map(|path| (path, None)))
        .chain(std::iter::once((&request.tool, Some(tool_revision))))
    {
        let (hash, observation) =
            snapshot::file(path, expected, cancel).map_err(|message| invalid(&message))?;
        if request
            .firmware
            .as_ref()
            .is_some_and(|image| image.file == *path)
        {
            let expected = request
                .firmware
                .as_ref()
                .and_then(|image| image.expected_sha256.as_deref())
                .ok_or_else(|| invalid("firmware sem checksum de origem do catálogo"))?;
            if hash != expected {
                return Err(invalid(
                    "checksum do firmware difere do catálogo; reinstale pelo catálogo antes de gravar",
                ));
            }
        }
        contents.push(json!([path, hash]));
        files.push(observation);
    }
    Ok(CapturedFiles { contents, files })
}

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!(
            "Não foi possível validar a gravação nativa: {message}. Nenhuma gravação foi iniciada."
        ),
    }
}
