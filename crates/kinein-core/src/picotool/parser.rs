//! Audited `info --device` output for RP2040 in picotool 2.2.0/2.3.1.
use kinein_protocol::PicoDevice;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn version(output: &str) -> Result<String, String> {
    let line = output.trim();
    for version in ["2.2.0", "2.3.1"] {
        if line.starts_with(&format!("picotool v{version} ("))
            && line.ends_with(')')
            && line.lines().count() == 1
        {
            return Ok(version.into());
        }
    }
    Err("versão/formato sem suporte ou picotool sem suporte USB".into())
}

pub(super) fn devices(output: &str) -> Result<Vec<PicoDevice>, String> {
    let invalid =
        || "saída RP2040 incompleta, ambígua ou com campos/avisos desconhecidos".to_owned();
    let lines: Vec<_> = output
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    let multiple = lines.first() == Some(&"Multiple RP-series devices in BOOTSEL mode found:");
    let mut fields = None::<BTreeMap<&str, &str>>;
    let mut devices = Vec::new();
    let mut headers = BTreeSet::new();
    let mut separator = None;
    let mut waiting_header = multiple;
    for line in lines.into_iter().skip(usize::from(multiple)) {
        if let Some(length) = separator.take() {
            if line.len() != length || !line.bytes().all(|b| b == b'-') {
                return Err(invalid());
            }
            continue;
        }
        if line.starts_with("RP2040 device at bus ") {
            if !multiple || !waiting_header {
                return Err(invalid());
            }
            let value = line
                .strip_prefix("RP2040 device at bus ")
                .and_then(|s| s.strip_suffix(':'))
                .ok_or_else(invalid)?;
            let (bus, address) = value.split_once(", address ").ok_or_else(invalid)?;
            let numeric = |text: &str, max: u8| {
                text.parse::<u8>()
                    .ok()
                    .filter(|v| *v > 0 && *v <= max && v.to_string() == text)
            };
            let pair = (
                numeric(bus, 255).ok_or_else(invalid)?,
                numeric(address, 127).ok_or_else(invalid)?,
            );
            if !headers.insert(pair) {
                return Err(invalid());
            }
            separator = Some(line.len());
            waiting_header = false;
            continue;
        }
        if line == "Device Information" {
            if multiple && waiting_header {
                return Err(invalid());
            }
            if let Some(values) = fields.take() {
                devices.push(device(&values)?);
            }
            if devices.len() >= 16 || (!multiple && !devices.is_empty()) {
                return Err(invalid());
            }
            fields = Some(BTreeMap::new());
            waiting_header = multiple;
            continue;
        }
        let (key, value) = line.split_once(':').ok_or_else(invalid)?;
        let values = fields.as_mut().ok_or_else(invalid)?;
        if !matches!(key, "type" | "flash size" | "flash id")
            || values.insert(key, value.trim()).is_some()
        {
            return Err(invalid());
        }
    }
    if separator.is_some() {
        return Err(invalid());
    }
    if let Some(values) = fields {
        devices.push(device(&values)?);
    }
    if devices.is_empty() || (multiple && (devices.len() < 2 || headers.len() != devices.len())) {
        return Err(invalid());
    }
    let unique: BTreeSet<_> = devices.iter().map(|d| &d.serial_selector).collect();
    if unique.len() != devices.len() {
        return Err("flash IDs duplicados não identificam um destino".into());
    }
    Ok(devices)
}

fn device(fields: &BTreeMap<&str, &str>) -> Result<PicoDevice, String> {
    let invalid = || "tipo, flash size ou flash id RP2040 inválido/ausente".to_owned();
    if fields.get("type") != Some(&"RP2040") {
        return Err(invalid());
    }
    let raw = fields
        .get("flash id")
        .and_then(|s| s.strip_prefix("0x"))
        .ok_or_else(invalid)?;
    if raw.len() != 16
        || !raw.bytes().all(|c| c.is_ascii_hexdigit())
        || raw.bytes().all(|c| c == b'0')
        || raw.bytes().all(|c| c == b'F' || c == b'f')
    {
        return Err(invalid());
    }
    let size = fields
        .get("flash size")
        .and_then(|s| s.strip_suffix('K'))
        .ok_or_else(invalid)?;
    if !size.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalid());
    }
    let size = size
        .parse::<u32>()
        .ok()
        .and_then(|n| n.checked_mul(1024))
        .filter(|n| *n > 0 && *n <= 16 * 1024 * 1024)
        .ok_or_else(invalid)?;
    Ok(PicoDevice {
        chip: "rp2040".into(),
        serial_selector: raw.to_ascii_uppercase(),
        flash_size_bytes: size,
    })
}
