//! Native RP2040 UF2 intent with CMake/ELF provenance and guarded BOOTSEL evidence.
mod artifact;
mod uf2;
use super::{
    FlashCommand, FlashError, FlashPlan,
    native::{NativeRequest, Prepared},
    snapshot,
};
use kinein_protocol::{FlashIntent, FlashProposalParams};
use serde_json::json;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

/// RP2040 serial matching claims interfaces before reading flash IDs in picotool.
/// Serializing this family leaves unrelated explicit ports/probes available.
pub(crate) const RESOURCE_KEY: &str = "picotool:2e8a:0003";

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!("Gravação RP2040 não validada: {message}. Nenhuma gravação foi iniciada."),
    }
}

/// Check explicit fields before reserving USB or invoking a tool.
///
/// # Errors
/// Missing/foreign selections, unsupported chip/capacity or unsafe selector syntax.
pub fn check_selection(selection: &FlashProposalParams) -> Result<(), FlashError> {
    let error = |message: &str| FlashError::InvalidSelection {
        message: format!("picotool nativo: {message}"),
    };
    if selection.device.is_some()
        || selection.probe.is_some()
        || selection.dfu.is_some()
        || selection.firmware.is_some()
        || selection.build_directory.is_some()
        || selection.flash_size_bytes.is_some()
    {
        return Err(error(
            "device, probe, dfu, firmware, buildDirectory e flashSizeBytes não se aplicam ao UF2 nativo",
        ));
    }
    if selection
        .target
        .as_deref()
        .is_none_or(|s| s.is_empty() || s.trim() != s || s.contains('\0'))
    {
        return Err(error(
            "escolha o executável CMake cujo ELF será conferido com o UF2",
        ));
    }
    let selected = selection
        .pico
        .as_ref()
        .ok_or_else(|| error("escolha UF2 e destino RP2040 BOOTSEL"))?;
    if selected.image.trim().is_empty() || selected.image.contains('\0') {
        return Err(error("caminho UF2 vazio ou inválido"));
    }
    let d = &selected.destination;
    if d.chip != "rp2040"
        || d.flash_size_bytes == 0
        || d.flash_size_bytes > 16 * 1024 * 1024
        || d.serial_selector.len() != 16
        || !d
            .serial_selector
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'A'..=b'F').contains(&c))
        || d.serial_selector == "0000000000000000"
        || d.serial_selector == "FFFFFFFFFFFFFFFF"
    {
        return Err(error(
            "exige RP2040, capacidade até 16 MiB e seletor --ser canônico confirmado, sem usar porta serial ou índice",
        ));
    }
    Ok(())
}

pub(super) fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
    lease: &crate::serial::ownership::Lease,
) -> Result<Prepared, FlashError> {
    check_selection(&request.selection)?;
    let selected = request
        .selection
        .pico
        .as_ref()
        .ok_or_else(|| invalid("seleção RP2040 ausente"))?;
    let target = request
        .selection
        .target
        .as_deref()
        .ok_or_else(|| invalid("alvo CMake ausente"))?;
    let tool_revision =
        crate::platform::file_revision(&request.tool).map_err(|e| invalid(&e.to_string()))?;
    let artifact = artifact::read(root, target, selected, cancel).map_err(|e| invalid(&e))?;
    let erased_end = (artifact.image.end + 4095) & !4095;
    if erased_end - 0x1000_0000 > selected.destination.flash_size_bytes {
        return Err(invalid(
            "o último setor apagável excede a capacidade do RP2040 escolhido",
        ));
    }
    let listing = crate::picotool::confirm(
        &request.tool,
        &selected.destination,
        Instant::now() + Duration::from_secs(5),
        cancel,
        lease,
    );
    if !listing.complete {
        return Err(invalid(
            listing.hint.as_deref().unwrap_or("destino não confirmado"),
        ));
    }
    let version = listing
        .version
        .as_deref()
        .ok_or_else(|| invalid("versão picotool ausente"))?;
    let (image_hash, image_observation) =
        snapshot::file(&artifact.path, Some(&artifact.revision), cancel)
            .map_err(|e| invalid(&e))?;
    let (tool_hash, tool_observation) =
        snapshot::file(&request.tool, Some(&tool_revision), cancel).map_err(|e| invalid(&e))?;
    let mut files = vec![image_observation, tool_observation];
    let mut contents = Vec::new();
    for (path, revision) in &artifact.sources {
        let (hash, observation) =
            snapshot::file(path, Some(revision), cancel).map_err(|e| invalid(&e))?;
        files.push(observation);
        contents.push(json!([path, hash]));
    }
    let mut plan = plan(
        &request.tool,
        &artifact,
        &selected.destination,
        version,
        &image_hash,
    );
    request.selection.engine = Some("picotool".into());
    if let Some(selection) = request.selection.pico.as_mut() {
        selection.image = artifact.path.display().to_string();
    }
    let device_key = snapshot::digest(
        &serde_json::to_vec(&json!([listing.devices, version]))
            .map_err(|e| invalid(&e.to_string()))?,
    );
    let fingerprint = snapshot::digest(
        &serde_json::to_vec(&json!([
            root,
            request.context,
            request.selection,
            plan.command.program,
            plan.command.args,
            plan.proposal.source,
            plan.proposal.warnings,
            device_key,
            contents,
            image_hash,
            tool_hash
        ]))
        .map_err(|e| invalid(&e.to_string()))?,
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
    super::native::validate_intent(root, &intent, Some(&plan.proposal.command))
        .map_err(|e| invalid(&e))?;
    plan.proposal.flash = Some(intent.clone());
    let prepared = Prepared::from_parts(plan, intent, files);
    if !prepared.unchanged() {
        return Err(invalid(
            "arquivos mudaram durante a preparação; gere outra prévia",
        ));
    }
    Ok(prepared)
}

fn plan(
    tool: &Path,
    artifact: &artifact::Artifact,
    destination: &kinein_protocol::PicoDevice,
    version: &str,
    image_hash: &str,
) -> FlashPlan {
    let command = FlashCommand::new(
        tool,
        vec![
            "load".into(),
            "--verify".into(),
            "--family".into(),
            "0xe48bff56".into(),
            artifact.path.display().to_string(),
            "--type".into(),
            "uf2".into(),
            "--vid".into(),
            "0x2e8a".into(),
            "--pid".into(),
            "0x0003".into(),
            "--ser".into(),
            destination.serial_selector.clone(),
        ],
    );
    let mut source = vec![
        format!("motor: picotool {version} (versão auditada confirmada)"),
        format!(
            "UF2 escolhido: {}; SHA256 {image_hash}",
            artifact.path.display()
        ),
        format!(
            "destino: RP2040 2e8a:0003 · seletor {} · {} bytes de flash confirmados",
            destination.serial_selector, destination.flash_size_bytes
        ),
    ];
    source.extend(artifact.evidence.iter().cloned());
    let warnings = vec!["Apaga setores de 4 KiB; bytes fora das páginas UF2 no mesmo setor podem ser apagados e zerados.".into(),
        "UF2 conferido com o ELF do build configurado; a prévia não recompila nem comprova atualização das fontes ou compatibilidade física da aplicação.".into(),
        "Verifica os dados gravados, sem force ou reinício; a aplicação não é iniciada.".into(),
        "Uma operação picotool RP2040 por vez: a reserva USB abrange essa família durante a consulta e gravação.".into()];
    FlashPlan::new(
        "Gravar (UF2/ELF)".into(),
        "picotool".into(),
        source,
        warnings,
        command,
    )
}

#[cfg(test)]
pub mod fixture;
#[cfg(test)]
mod tests;
