//! Native probe-rs intent (0.4.1h4b2b4b1, `roadmaps/52` §14.34).
//!
//! A `CMake` executable target, the kit's chip, one explicitly listed probe
//! and the 0.32 syntax, without options inherited from the environment or
//! presets. Execution stays with `run.start`, the PTY and the reservation.
mod tool;

pub use tool::inherited_variables;

use super::{
    FlashCommand, FlashError, FlashPlan,
    native::{NativeRequest, Prepared},
    snapshot,
};
use kinein_protocol::{FlashIntent, FlashProposalParams, ProbeInfo};
use serde_json::json;
use std::{
    fmt::{self, Write as _},
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

/// Explicit probe selector, canonical as probe-rs 0.32 prints a listed entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    vid: u16,
    pid: u16,
    interface: Option<u8>,
    serial: String,
}

impl Selector {
    /// Parse `vvvv:pppp[-IF]:SERIAL`, lowercase hex, serial required.
    ///
    /// # Errors
    /// Non-canonical text, missing serial or invalid hex/interface: a row
    /// index or VID/PID alone never names a destination.
    pub fn parse(text: &str) -> Result<Self, String> {
        let invalid = || {
            format!(
                "seletor de sonda inválido `{text}`: use vvvv:pppp[-IF]:SERIAL como a listagem mostra; sonda sem serial não identifica o destino"
            )
        };
        let mut parts = text.splitn(3, ':');
        let vid = parts.next().unwrap_or_default();
        let pid = parts.next().ok_or_else(invalid)?;
        let serial = parts.next().ok_or_else(invalid)?;
        let (pid, interface) = match pid.split_once('-') {
            Some((pid, interface)) => (pid, Some(interface.parse::<u8>().map_err(|_| invalid())?)),
            None => (pid, None),
        };
        let hex = |field: &str| {
            u16::from_str_radix(field, 16)
                .ok()
                .filter(|_| field.len() == 4)
        };
        let selector = Self {
            vid: hex(vid).ok_or_else(invalid)?,
            pid: hex(pid).ok_or_else(invalid)?,
            interface,
            serial: serial.to_owned(),
        };
        if serial.is_empty()
            || serial.trim() != serial
            || serial.contains('\0')
            || selector.to_string() != text
        {
            return Err(invalid());
        }
        Ok(selector)
    }

    /// The 0.32 match: interface optional, serial equal or its uppercase hex.
    fn matches(&self, probe: &ProbeInfo) -> bool {
        let id = |field: &str| u16::from_str_radix(field, 16).ok();
        id(&probe.vid) == Some(self.vid)
            && id(&probe.pid) == Some(self.pid)
            && self
                .interface
                .is_none_or(|interface| probe.interface == Some(interface))
            && probe.serial.as_deref().is_some_and(|serial| {
                serial == self.serial
                    || serial.bytes().fold(String::new(), |mut hex, byte| {
                        let _ = write!(hex, "{byte:02X}");
                        hex
                    }) == self.serial
            })
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:04x}:{:04x}", self.vid, self.pid)?;
        if let Some(interface) = self.interface {
            write!(formatter, "-{interface}")?;
        }
        write!(formatter, ":{}", self.serial)
    }
}

/// Selection invariants of a probe-rs native request or intent.
///
/// # Errors
/// `InvalidSelection` (`INVALID_PARAMS`) when target/probe are missing or a
/// field of another destination kind is present.
pub fn check_selection(selection: &FlashProposalParams) -> Result<(), FlashError> {
    let selection_error = |message: &str| FlashError::InvalidSelection {
        message: format!("probe-rs nativo: {message}"),
    };
    if selection.device.is_some()
        || selection.firmware.is_some()
        || selection.build_directory.is_some()
        || selection.flash_size_bytes.is_some()
    {
        return Err(selection_error(
            "device, firmware, buildDirectory e flashSizeBytes não se aplicam; o destino é a sonda",
        ));
    }
    if selection
        .target
        .as_deref()
        .is_none_or(|target| target.trim().is_empty() || target.trim() != target)
    {
        return Err(selection_error(
            "escolha o alvo executável do CMake (target) que gera o ELF",
        ));
    }
    let probe = selection
        .probe
        .as_deref()
        .ok_or_else(|| selection_error("escolha a sonda (probe) na listagem"))?;
    Selector::parse(probe)
        .map(drop)
        .map_err(|message| selection_error(&message))
}

/// What the kit, the tool, the codemodel and the listing proved.
struct Evidence {
    name: String,
    selector: Selector,
    chip: String,
    tool_revision: String,
    version: String,
    origin: crate::project::flash_origin::CmakeOrigin,
    cores: Vec<String>,
    probe: ProbeInfo,
}

/// Validate and snapshot a native probe-rs flash; nothing is written here.
///
/// # Errors
/// Any missing evidence: chip only deduced, presets, version, target/artifact,
/// architecture, probe absent/ambiguous/inaccessible, changed files or cancel.
pub(super) fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Prepared, FlashError> {
    let evidence = collect_evidence(root, &request, cancel)?;
    let mut plan = plan(&request.tool, &evidence);
    let (contents, files) = capture(&request.tool, &evidence, cancel)?;
    let device_key = snapshot::digest(
        &serde_json::to_vec(&json!([evidence.selector.to_string(), evidence.probe]))
            .map_err(|error| invalid(&error.to_string()))?,
    );
    request.selection.native = true;
    request.selection.engine = Some("probe-rs".to_owned());
    request.selection.probe = Some(evidence.selector.to_string());
    let fingerprint = snapshot::digest(
        &serde_json::to_vec(&json!([
            root,
            request.context,
            request.selection,
            request.model.target,
            plan.command.program,
            plan.command.args,
            plan.proposal.source,
            evidence.version,
            evidence.cores,
            device_key,
            contents
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
    super::native::validate_intent(root, &intent, Some(&plan.proposal.command))
        .map_err(|message| invalid(&message))?;
    plan.proposal.source.push(
        "origem e conteúdo vinculados ao contexto; snapshot revalidado antes da execução"
            .to_owned(),
    );
    plan.proposal.flash = Some(intent.clone());
    let prepared = Prepared::from_parts(plan, intent, files);
    if !prepared.unchanged() {
        return Err(invalid("arquivos mudaram ao preparar a prévia; gere outra"));
    }
    Ok(prepared)
}

fn collect_evidence(
    root: &Path,
    request: &NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Evidence, FlashError> {
    check_selection(&request.selection)?;
    let name = request.selection.target.clone().unwrap_or_default();
    let selector = Selector::parse(request.selection.probe.as_deref().unwrap_or_default())
        .map_err(|message| invalid(&message))?;
    let chip = request
        .kit_chip
        .clone()
        .filter(|chip| !chip.trim().is_empty() && request.model.target.chip.as_ref() == Some(chip))
        .ok_or_else(|| {
            invalid("o probe-rs precisa do chip fixado no kit (toolchain.setKit); um chip só deduzido do framework não basta")
        })?;
    let presets = tool::configuration_files(root, &request.tool);
    if !presets.is_empty() {
        return Err(invalid(&format!(
            "o probe-rs aplicaria presets de {}; a gravação nativa não os interpreta: renomeie o arquivo ou use o modo manual",
            presets
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let tool_revision = crate::platform::file_revision(&request.tool)
        .map_err(|error| invalid(&error.to_string()))?;
    let version = tool::version(&request.tool, root, cancel)?;
    let origin = crate::project::flash_origin::cmake_target(root, &name)
        .map_err(|message| invalid(&message))?;
    let cores = tool::cores(&request.tool, root, &chip, cancel)?;
    if !cores.iter().any(|core| tool::runs_on(origin.machine, core)) {
        return Err(invalid(&format!(
            "o ELF de `{name}` é {} e o chip {chip} tem núcleos {}; confira o kit e o alvo",
            origin.machine.label(),
            cores.join(", ")
        )));
    }
    let probe = select(&selector, tool::listed(&request.tool, root, cancel)?)?;
    Ok(Evidence {
        name,
        selector,
        chip,
        tool_revision,
        version,
        origin,
        cores,
        probe,
    })
}

/// The literal 0.32 argv and the evidence lines the preview shows.
fn plan(tool: &Path, evidence: &Evidence) -> FlashPlan {
    let artifact = evidence.origin.artifact.display().to_string();
    let command = FlashCommand::new(
        tool,
        [
            "download",
            "--chip",
            &evidence.chip,
            "--probe",
            &evidence.selector.to_string(),
            "--binary-format",
            "elf",
            "--verify",
            &artifact,
        ]
        .map(str::to_owned)
        .to_vec(),
    );
    let inherited = inherited_variables();
    let source = vec![
        "motor: probe-rs (resolvido para a intenção nativa)".to_owned(),
        format!(
            "ferramenta: {} (probe-rs {}, mi meta)",
            tool.display(),
            evidence.version
        ),
        format!(
            "alvo: {} ({}, codemodel do .kinein/build)",
            evidence.name, evidence.origin.id
        ),
        format!(
            "imagem: {artifact} (artefato único do alvo, ELF {})",
            evidence.origin.machine.label()
        ),
        format!(
            "chip: {} (kit; núcleos {} pelo chip info)",
            evidence.chip,
            evidence.cores.join(", ")
        ),
        format!(
            "sonda: {} — {} ({}), única entrada listada que casa",
            evidence.selector,
            evidence.probe.name,
            evidence.probe.kind.as_deref().unwrap_or("?")
        ),
        if inherited.is_empty() {
            "ambiente: nenhuma variável PROBE_RS_* herdada".to_owned()
        } else {
            format!(
                "ambiente: {} não chegam ao probe-rs",
                inherited
                    .iter()
                    .map(|name| name.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        "configuração: nenhum .probe-rs.* na raiz, na pasta do executável ou no home".to_owned(),
    ];
    let warnings = vec![
        "o chip não é reiniciado depois da gravação (sem --reset): reinicie a placa ou inicie a depuração".to_owned(),
        "apaga só os setores que recebem a imagem; o chip inteiro não é apagado".to_owned(),
    ];
    FlashPlan::new(
        "Gravar (probe-rs)".to_owned(),
        "probe-rs".to_owned(),
        source,
        warnings,
        command,
    )
}

/// Content hashes and change tokens: reply files, the ELF and the tool.
fn capture(
    tool: &Path,
    evidence: &Evidence,
    cancel: &Arc<AtomicBool>,
) -> Result<(Vec<serde_json::Value>, Vec<snapshot::Observation>), FlashError> {
    let mut contents = Vec::new();
    let mut files = Vec::new();
    for (path, expected) in evidence
        .origin
        .sources
        .iter()
        .map(|(path, revision)| (path.as_path(), Some(revision.as_str())))
        .chain(std::iter::once((evidence.origin.artifact.as_path(), None)))
        .chain(std::iter::once((
            tool,
            Some(evidence.tool_revision.as_str()),
        )))
    {
        let (hash, observation) =
            snapshot::file(path, expected, cancel).map_err(|message| invalid(&message))?;
        contents.push(json!([path, hash]));
        files.push(observation);
    }
    Ok((contents, files))
}

/// Exactly one accessible listed entry that probe-rs would select.
fn select(selector: &Selector, probes: Vec<ProbeInfo>) -> Result<ProbeInfo, FlashError> {
    let mut matching: Vec<ProbeInfo> = probes
        .into_iter()
        .filter(|probe| selector.matches(probe))
        .collect();
    match matching.len() {
        0 => Err(invalid(&format!(
            "a sonda {selector} não está na listagem; conecte-a e atualize a lista de sondas"
        ))),
        1 if matching[0].inaccessible => Err(invalid(&format!(
            "a sonda {selector} está listada como inacessível: falta permissão no dispositivo (regra de udev), não rode a IDE como root"
        ))),
        1 => Ok(matching.remove(0)),
        count => Err(invalid(&format!(
            "{count} sondas casam com {selector}; desconecte as duplicadas ou escolha a interface"
        ))),
    }
}

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!(
            "Não foi possível validar a gravação nativa por sonda: {message}. Nenhuma gravação foi iniciada."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{Selector, check_selection, select};
    use kinein_protocol::{FlashProposalParams, ProbeInfo};

    fn probe(vid: &str, pid: &str, interface: Option<u8>, serial: Option<&str>) -> ProbeInfo {
        ProbeInfo {
            name: "Fixture".to_owned(),
            vid: vid.to_owned(),
            pid: pid.to_owned(),
            serial: serial.map(str::to_owned),
            kind: Some("CMSIS-DAP".to_owned()),
            interface,
            inaccessible: false,
        }
    }

    #[test]
    fn selector_is_canonical_and_requires_a_serial() {
        for valid in [
            "0d28:0204:9900000031",
            "0403:6010-1:FT1234",
            "303a:1001:AA:BB:CC",
        ] {
            assert_eq!(Selector::parse(valid).unwrap().to_string(), valid);
        }
        for invalid in [
            "0d28:0204",
            "0d28:0204:",
            "0D28:0204:S1",
            "d28:0204:S1",
            "0d28:0204-x:S1",
            "0d28:0204: S1",
            "0",
            "1",
        ] {
            assert!(Selector::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn selection_mirrors_probe_rs_matching_and_refuses_ambiguity() {
        let selector = Selector::parse("0403:6010:FT1").unwrap();
        let channels = vec![
            probe("0403", "6010", Some(0), Some("FT1")),
            probe("0403", "6010", Some(1), Some("FT1")),
        ];
        assert!(
            select(&selector, channels.clone()).is_err(),
            "no interface matches both"
        );
        let one = Selector::parse("0403:6010-1:FT1").unwrap();
        assert_eq!(select(&one, channels).unwrap().interface, Some(1));
        let hex = Selector::parse("0483:374b:4142").unwrap();
        assert!(select(&hex, vec![probe("0483", "374b", None, Some("AB"))]).is_ok());
        let mut blocked = probe("0483", "374b", None, Some("S1"));
        blocked.inaccessible = true;
        let message = select(&Selector::parse("0483:374b:S1").unwrap(), vec![blocked])
            .unwrap_err()
            .to_string();
        assert!(message.contains("udev"), "{message}");
        assert!(select(&Selector::parse("0483:374b:S2").unwrap(), Vec::new()).is_err());
    }

    #[test]
    fn selection_fields_of_other_destinations_are_invalid_params() {
        let base = FlashProposalParams {
            native: true,
            engine: Some("probe-rs".to_owned()),
            target: Some("firmware".to_owned()),
            probe: Some("0483:374b:S1".to_owned()),
            ..FlashProposalParams::default()
        };
        assert!(check_selection(&base).is_ok());
        for broken in [
            FlashProposalParams {
                device: Some("/dev/ttyACM0".to_owned()),
                ..base.clone()
            },
            FlashProposalParams {
                target: None,
                ..base.clone()
            },
            FlashProposalParams {
                target: Some(" app".to_owned()),
                ..base.clone()
            },
            FlashProposalParams {
                probe: None,
                ..base.clone()
            },
            FlashProposalParams {
                probe: Some("0483:374b".to_owned()),
                ..base.clone()
            },
            FlashProposalParams {
                flash_size_bytes: Some(1),
                ..base
            },
        ] {
            assert!(
                matches!(
                    check_selection(&broken),
                    Err(crate::flash::FlashError::InvalidSelection { .. })
                ),
                "{broken:?}"
            );
        }
    }
}
