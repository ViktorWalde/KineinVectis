//! Native `DfuSe` file selection and USB evidence (52 §14.52).
mod artifact;
mod binary;
mod image;
mod memory;

use super::{
    FlashCommand, FlashError, FlashPlan,
    native::{NativeRequest, Prepared},
    snapshot,
};
use kinein_protocol::{DfuDestination, DfuInterface, DfuMode, FlashIntent, FlashProposalParams};
use serde_json::json;
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!(
            "Gravação USB DFU não validada: {message}. Nenhuma gravação foi iniciada."
        ),
    }
}

/// Validate the explicit selection before querying or reserving any resource.
///
/// # Errors
/// Missing/foreign fields, unsupported USB device or unsafe selector syntax.
pub fn check_selection(selection: &FlashProposalParams) -> Result<(), FlashError> {
    let selection_error = |message: &str| FlashError::InvalidSelection {
        message: format!("dfu-util nativo: {message}"),
    };
    if selection.device.is_some()
        || selection.probe.is_some()
        || selection.target.is_some()
        || selection.firmware.is_some()
        || selection.build_directory.is_some()
        || selection.flash_size_bytes.is_some()
    {
        return Err(selection_error(
            "device, probe, target, firmware, buildDirectory e flashSizeBytes não se aplicam ao arquivo DfuSe",
        ));
    }
    let selected = selection
        .dfu
        .as_ref()
        .ok_or_else(|| selection_error("escolha o arquivo DfuSe e uma alternativa USB DFU"))?;
    if selected
        .binary_target
        .as_deref()
        .is_some_and(|name| name.trim().is_empty() || name.trim() != name || name.contains('\0'))
    {
        return Err(selection_error(
            "escolha o alvo CMake do BIN, sem espaços externos",
        ));
    }
    let destination = &selected.destination;
    if selected.image.trim().is_empty() || selected.image.contains('\0') {
        return Err(selection_error(
            "caminho do arquivo DfuSe vazio ou inválido",
        ));
    }
    if destination.vid != "0483"
        || destination.pid != "df11"
        || destination.configuration != 1
        || destination.interface != 0
    {
        return Err(selection_error(
            "este recorte cobre DfuSe 0483:df11, configuração 1 e interface 0",
        ));
    }
    if destination.serial.is_empty()
        || destination.serial == "UNKNOWN"
        || destination.serial.trim() != destination.serial
        || destination.serial.contains(',')
        || destination.serial.chars().any(char::is_control)
        || !usb_path(&destination.path)
    {
        return Err(selection_error(
            "serial real sem separadores e caminho USB canônico são obrigatórios; porta serial e índice de linha não são destinos",
        ));
    }
    Ok(())
}

fn usb_path(value: &str) -> bool {
    let Some((bus, ports)) = value.split_once('-') else {
        return false;
    };
    let number = |value: &str| {
        value
            .parse::<u8>()
            .ok()
            .is_some_and(|number| number > 0 && number.to_string() == value)
    };
    number(bus) && ports.split('.').count() <= 8 && ports.split('.').all(number)
}

/// One reservation for all alternatives of the explicitly selected USB device.
#[must_use]
pub fn resource_key(destination: &DfuDestination) -> String {
    // Canonical fixed VID/PID + unambiguous serial. Path binds the snapshot,
    // while a replugged instance with the same serial still conflicts.
    format!(
        "{}:{}:{}",
        destination.vid, destination.pid, destination.serial
    )
}

fn selected_interface(
    destination: &DfuDestination,
    interfaces: &[DfuInterface],
) -> Result<DfuInterface, FlashError> {
    let same_serial = |entry: &&DfuInterface| {
        entry.vid == destination.vid
            && entry.pid == destination.pid
            && entry.serial.as_deref() == Some(destination.serial.as_str())
    };
    let instances: std::collections::BTreeSet<_> = interfaces
        .iter()
        .filter(same_serial)
        .map(|entry| (&entry.path, entry.device_number))
        .collect();
    if instances.len() != 1 {
        return Err(invalid(
            "destino USB ausente ou serial ambíguo em mais de uma instância",
        ));
    }
    let mut matches = interfaces.iter().filter(same_serial).filter(|entry| {
        entry.mode == DfuMode::Dfu
            && entry.path == destination.path
            && entry.configuration == destination.configuration
            && entry.interface == destination.interface
            && entry.alternate == destination.alternate
    });
    let selected = matches.next().ok_or_else(|| invalid("alternativa escolhida ausente ou dispositivo ainda em Runtime; entre em DFU e consulte novamente"))?;
    if matches.next().is_some() {
        return Err(invalid(
            "mais de uma entrada corresponde à alternativa USB escolhida",
        ));
    }
    Ok(selected.clone())
}

pub(super) fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Prepared, FlashError> {
    check_selection(&request.selection)?;
    let selected = request
        .selection
        .dfu
        .as_ref()
        .ok_or_else(|| invalid("seleção DfuSe ausente"))?;
    let tool_revision = crate::platform::file_revision(&request.tool)
        .map_err(|error| invalid(&error.to_string()))?;
    let artifact = artifact::read(root, selected, cancel).map_err(|message| invalid(&message))?;
    let image = &artifact.image;
    let path = &artifact.path;
    let listing = crate::dfu::list(Some(&request.tool), cancel);
    if !listing.complete {
        return Err(invalid(
            listing
                .hint
                .as_deref()
                .unwrap_or("lista USB não verificada"),
        ));
    }
    let entry = selected_interface(&selected.destination, &listing.interfaces)?;
    if image.alternate != entry.alternate {
        return Err(invalid(
            "alternativa no arquivo DfuSe difere da escolhida na listagem",
        ));
    }
    let release =
        u16::from_str_radix(&entry.release, 16).map_err(|_| invalid("release USB inválido"))?;
    if ![0, u16::MAX, release].contains(&image.release) {
        return Err(invalid(
            "bcdDevice do arquivo DfuSe difere do release USB observado",
        ));
    }
    memory::validate(&entry.name, &image.elements).map_err(|message| invalid(&message))?;
    let (image_hash, image_observation) = snapshot::file(path, Some(&artifact.revision), cancel)
        .map_err(|message| invalid(&message))?;
    let (tool_hash, tool_observation) = snapshot::file(&request.tool, Some(&tool_revision), cancel)
        .map_err(|message| invalid(&message))?;
    let mut contents = Vec::new();
    let mut files = vec![image_observation, tool_observation];
    for (source, revision) in &artifact.sources {
        let (hash, observed) =
            snapshot::file(source, Some(revision), cancel).map_err(|message| invalid(&message))?;
        contents.push(json!([source, hash]));
        files.push(observed);
    }
    let destination = &selected.destination;
    let mut plan = plan(&request.tool, &artifact, &entry, &image_hash, destination);
    request.selection.engine = Some("dfu-util".into());
    if let Some(selected) = request.selection.dfu.as_mut() {
        selected.image = path.display().to_string();
    }
    let device_key =
        snapshot::digest(&serde_json::to_vec(&entry).map_err(|error| invalid(&error.to_string()))?);
    let mut fingerprint_parts = json!([
        root,
        request.context,
        request.selection,
        plan.command.program,
        plan.command.args,
        plan.proposal.source,
        plan.proposal.warnings,
        device_key,
        image_hash,
        tool_hash
    ]);
    if !contents.is_empty()
        && let Some(parts) = fingerprint_parts.as_array_mut()
    {
        parts.push(json!(contents));
    }
    let fingerprint = snapshot::digest(
        &serde_json::to_vec(&fingerprint_parts).map_err(|error| invalid(&error.to_string()))?,
    );
    let intent = FlashIntent {
        version: 1,
        workspace: root.display().to_string(),
        selection: request.selection,
        program: request.tool.display().to_string(),
        args: plan.command.args.clone(),
        device_key,
        fingerprint,
    };
    super::native::validate_intent(root, &intent, Some(&plan.proposal.command))
        .map_err(|message| invalid(&message))?;
    plan.proposal.flash = Some(intent.clone());
    let prepared = Prepared::from_parts(plan, intent, files);
    if !prepared.unchanged() {
        return Err(invalid("arquivos mudaram ao preparar a prévia; gere outra"));
    }
    Ok(prepared)
}

fn plan(
    tool: &Path,
    artifact: &artifact::Artifact,
    entry: &DfuInterface,
    image_hash: &str,
    destination: &DfuDestination,
) -> FlashPlan {
    let path = &artifact.path;
    let image = &artifact.image;
    let mut args = vec![
        "--device".into(),
        format!(",{}:{}", destination.vid, destination.pid),
        "--serial".into(),
        format!(",{}", destination.serial),
        "--path".into(),
        destination.path.clone(),
        "--cfg".into(),
        destination.configuration.to_string(),
        "--intf".into(),
        destination.interface.to_string(),
        "--alt".into(),
        destination.alternate.to_string(),
    ];
    if let Some(address) = artifact.address {
        args.extend(["--dfuse-address".into(), format!("0x{address:08x}")]);
    }
    args.extend(["--download".into(), path.display().to_string()]);
    let command = FlashCommand::new(tool, args);
    let mut source = vec![
        "motor: dfu-util 0.11 (confirmado na listagem completa)".to_owned(),
        format!(
            "origem: arquivo {} explicitamente escolhido: {}; SHA256 {image_hash}",
            if artifact.address.is_some() {
                "BIN"
            } else {
                "DfuSe"
            },
            path.display()
        ),
        format!(
            "destino: {}:{} · serial {} · caminho {} · cfg {} · interface {} · alternativa {}",
            entry.vid,
            entry.pid,
            destination.serial,
            entry.path,
            entry.configuration,
            entry.interface,
            entry.alternate
        ),
        format!("memória anunciada: {}", entry.name),
    ];
    source.extend(artifact.evidence.iter().cloned());
    source.extend(image.elements.iter().map(|element| {
        format!(
            "elemento: endereço 0x{:08x}, {} bytes",
            element.address, element.size
        )
    }));
    let mut warnings = vec!["O dfu-util apaga as páginas cobertas pela gravação; bytes fora dos elementos na mesma página também podem ser apagados.".to_owned(),
        "Arquivo escolhido pelo usuário; a IDE não deduz que veio do build ativo nem comprova a compatibilidade da aplicação com o chip.".to_owned(),
        "Não são pedidos reset, leave, unprotect, mass-erase ou force; a aplicação não é iniciada por esta prévia.".to_owned()];
    if artifact.address.is_some() {
        warnings[1] = "BIN conferido com o ELF do último build configurado; a prévia não recompila fontes nem comprova a compatibilidade física da aplicação com o chip.".into();
    }
    FlashPlan::new(
        if artifact.address.is_some() {
            "Gravar (BIN/ELF)"
        } else {
            "Gravar (DfuSe)"
        }
        .into(),
        "dfu-util".into(),
        source,
        warnings,
        command,
    )
}

#[cfg(test)]
mod binary_tests;
#[cfg(test)]
mod tests;
