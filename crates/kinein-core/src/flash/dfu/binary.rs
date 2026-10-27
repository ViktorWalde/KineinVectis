//! A bounded BIN/ELF comparison, not an ELF converter or loader (52 §14.53).
use std::sync::atomic::{AtomicBool, Ordering};

struct Segment {
    address: u32,
    end: u32,
    offset: usize,
    size: usize,
}

fn invalid() -> String {
    "BIN exige ELF32 little endian ARM ET_EXEC válido, sem carregamento dinâmico".into()
}

fn word(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|v| v.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(invalid)
}

fn quad(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|v| v.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(invalid)
}

fn cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        Err("preparação BIN cancelada".into())
    } else {
        Ok(())
    }
}

/// Compare every file-backed load byte and return its lowest physical address.
pub(super) fn validate(elf: &[u8], binary: &[u8], cancel: &AtomicBool) -> Result<u32, String> {
    cancelled(cancel)?;
    if binary.is_empty() || binary.len() > super::image::MAX_BYTES {
        return Err("BIN vazio ou maior que 32 MiB".into());
    }
    if binary.starts_with(b"DfuSe")
        || (binary.len() >= 16 && &binary[binary.len() - 8..binary.len() - 5] == b"UFD")
        || (binary.len() >= 2
            && ((binary[0] == 1 && binary[1] == 0)
                || (binary[0] & 0x3f == 0x1a && binary[1] & 0x3f == 0x3f)))
    {
        return Err(
            "BIN precisa ser cru, sem prefixo TI/LPC/DfuSe ou sufixo UFD que o dfu-util removeria"
                .into(),
        );
    }
    let segments = segments(elf)?;
    let start = segments.first().ok_or_else(invalid)?.address;
    let end = segments.last().ok_or_else(invalid)?.end;
    if usize::try_from(end - start).ok() != Some(binary.len()) {
        return Err(
            "tamanho do BIN difere do intervalo de carga do ELF; padding extra não é aceito".into(),
        );
    }
    let mut cursor = 0;
    for segment in segments {
        cancelled(cancel)?;
        let offset = usize::try_from(segment.address - start).map_err(|_| invalid())?;
        for gap in binary[cursor..offset].chunks(65_536) {
            cancelled(cancel)?;
            if gap.iter().any(|byte| *byte != 0) {
                return Err("gaps do BIN entre segmentos ELF precisam conter somente zeros".into());
            }
        }
        let source = &elf[segment.offset..segment.offset + segment.size];
        let target = &binary[offset..offset + segment.size];
        for (source, target) in source.chunks(65_536).zip(target.chunks(65_536)) {
            cancelled(cancel)?;
            if source != target {
                return Err("bytes do BIN diferem dos segmentos de carga do ELF escolhido".into());
            }
        }
        cursor = offset + segment.size;
    }
    Ok(start)
}

fn segments(elf: &[u8]) -> Result<Vec<Segment>, String> {
    if elf.len() < 52
        || elf.len() > 64 * 1024 * 1024
        || &elf[..7] != b"\x7fELF\x01\x01\x01"
        || word(elf, 16)? != 2
        || word(elf, 18)? != 0x28
        || quad(elf, 20)? != 1
        || word(elf, 40)? != 52
        || word(elf, 42)? != 32
    {
        return Err(invalid());
    }
    let count = usize::from(word(elf, 44)?);
    let offset = usize::try_from(quad(elf, 28)?).map_err(|_| invalid())?;
    if !(1..=128).contains(&count)
        || offset < 52
        || offset
            .checked_add(count * 32)
            .is_none_or(|end| end > elf.len())
    {
        return Err(invalid());
    }
    let entry = quad(elf, 24)? & !1;
    let mut executable_entry = false;
    let mut result = Vec::new();
    for header in elf[offset..offset + count * 32].chunks_exact(32) {
        match quad(header, 0)? {
            2 | 3 => return Err(invalid()),
            1 => {}
            _ => continue,
        }
        let file_offset = quad(header, 4)?;
        let virtual_address = quad(header, 8)?;
        let address = quad(header, 12)?;
        let size = quad(header, 16)?;
        let memory_size = quad(header, 20)?;
        let alignment = quad(header, 28)?;
        if size > memory_size
            || virtual_address.checked_add(memory_size).is_none()
            || (alignment > 1
                && (!alignment.is_power_of_two()
                    || virtual_address % alignment != file_offset % alignment))
        {
            return Err(invalid());
        }
        let offset = usize::try_from(file_offset).map_err(|_| invalid())?;
        let length = usize::try_from(size).map_err(|_| invalid())?;
        if offset.checked_add(length).is_none_or(|end| end > elf.len()) {
            return Err(invalid());
        }
        if size == 0 {
            continue;
        }
        executable_entry |= quad(header, 24)? & 1 != 0
            && entry >= virtual_address
            && entry < virtual_address.checked_add(size).ok_or_else(invalid)?;
        let end = address.checked_add(size).ok_or_else(invalid)?;
        if address < 0x0800_0000 || end > 0x0900_0000 {
            return Err("ELF tem bytes carregáveis fora de flash interna; BIN não é validável neste recorte".into());
        }
        result.push(Segment {
            address,
            end,
            offset,
            size: length,
        });
    }
    result.sort_by_key(|segment| segment.address);
    if !executable_entry || result.is_empty() {
        return Err(invalid());
    }
    if result.windows(2).any(|pair| pair[0].end > pair[1].address) {
        return Err("segmentos de carga ELF sobrepostos".into());
    }
    Ok(result)
}
