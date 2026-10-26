//! Bounded dfu-util discovery. No detach, reset or firmware transfer.
//!
//! Format: official 0.11, `src/dfu_util.c::print_dfu_if` (52 §14.51).
//! `MODE_LIST` returns zero even after a USB access warning: unknown output
//! must invalidate the whole list rather than silently discard a device.
use kinein_protocol::{DfuInterface, DfuListResult, DfuMode};
use std::{
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

/// Query only --list through the existing bounded process runner.
#[must_use]
pub fn list(program: Option<&Path>, cancel: &Arc<AtomicBool>) -> DfuListResult {
    query(program, Duration::from_secs(5), cancel)
}

fn query(program: Option<&Path>, timeout: Duration, cancel: &Arc<AtomicBool>) -> DfuListResult {
    let Some(program) = program else {
        return failed(
            false,
            None,
            String::new(),
            "dfu-util não encontrado no PATH; instale a ferramenta antes de consultar USB DFU",
        );
    };
    let mut command = Command::new(program);
    command.arg("--list").env("LC_ALL", "C");
    let answer = crate::probe::query::run(command, timeout, cancel);
    let tool = Some(program.display().to_string());
    if let Some(failure) = answer.failure {
        let (available, reason) = match failure {
            crate::probe::query::Failure::Spawn(reason) => {
                (false, format!("não pôde iniciar: {reason}"))
            }
            crate::probe::query::Failure::Incomplete(reason) => (true, reason),
        };
        return failed(available, tool, answer.output, &reason);
    }
    match parse_listing(&answer.output) {
        Ok(interfaces) => {
            let hint = interfaces.is_empty().then(||
                "Nenhuma interface USB DFU foi listada nesta consulta. Confira a conexão e o modo USB da placa; uma porta serial não é uma interface DFU.".to_owned());
            DfuListResult {
                interfaces,
                tool_available: true,
                complete: true,
                tool,
                raw_output: answer.output,
                hint,
            }
        }
        Err(reason) => failed(true, tool, answer.output, &reason),
    }
}

fn failed(
    tool_available: bool,
    tool: Option<String>,
    raw_output: String,
    reason: &str,
) -> DfuListResult {
    DfuListResult {
        interfaces: Vec::new(),
        tool_available,
        complete: false,
        tool,
        raw_output,
        hint: Some(format!(
            "A lista USB DFU não foi verificada: {reason}. Confira a saída e solicite uma nova consulta."
        )),
    }
}

/// Strict discovery: no warnings, unknown lines or unverified version may
/// become a complete (especially empty) list.
fn parse_listing(output: &str) -> Result<Vec<DfuInterface>, String> {
    let mut banner = false;
    let mut interfaces = Vec::new();
    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line == "dfu-util 0.11" && !banner {
            banner = true;
            continue;
        }
        if matches!(
            line,
            "Copyright 2005-2009 Weston Schmidt, Harald Welte and OpenMoko Inc."
                | "Copyright 2010-2021 Tormod Volden and Stefan Schmidt"
                | "This program is Free Software and has ABSOLUTELY NO WARRANTY"
                | "Please report bugs to http://sourceforge.net/p/dfu-util/tickets/"
        ) {
            continue;
        }
        interfaces.push(
            parse_interface(line).ok_or_else(|| {
                format!("aviso, erro ou formato diferente do dfu-util 0.11: {line}")
            })?,
        );
    }
    if !banner {
        return Err("cabeçalho dfu-util 0.11 ausente; versão não comprovada".to_owned());
    }
    Ok(interfaces)
}

// The upstream printer does not escape USB strings. Ambiguous separators
// invalidate discovery instead of letting descriptor text become structure.
fn parse_interface(line: &str) -> Option<DfuInterface> {
    if line.matches("\", serial=\"").count() != 1 {
        return None;
    }
    let mut rest = line.strip_prefix("Found ")?;
    let mode = match field(&mut rest, ": [")? {
        "DFU" => DfuMode::Dfu,
        "Runtime" => DfuMode::Runtime,
        _ => return None,
    };
    let vid = hex(field(&mut rest, ":")?)?;
    let pid = hex(field(&mut rest, "] ver=")?)?;
    let release = hex(field(&mut rest, ", devnum=")?)?;
    let device_number = number(field(&mut rest, ", cfg=")?)?;
    let configuration = number(field(&mut rest, ", intf=")?)?;
    let interface = number(field(&mut rest, ", path=\"")?)?;
    let path = field(&mut rest, "\", alt=")?.to_owned();
    let alternate = number(field(&mut rest, ", name=\"")?)?;
    let name = field(&mut rest, "\", serial=\"")?.to_owned();
    let serial = rest.strip_suffix('"')?;
    Some(DfuInterface {
        mode,
        vid,
        pid,
        release,
        device_number,
        configuration,
        interface,
        path,
        alternate,
        name,
        serial: (!matches!(serial, "" | "UNKNOWN")).then(|| serial.to_owned()),
    })
}

fn field<'a>(rest: &mut &'a str, separator: &str) -> Option<&'a str> {
    let (value, tail) = rest.split_once(separator)?;
    *rest = tail;
    Some(value)
}

fn hex(value: &str) -> Option<String> {
    (value.len() == 4
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then(|| value.to_owned())
}

fn number(value: &str) -> Option<u8> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests;
