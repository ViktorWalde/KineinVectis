//! Shared bounded ARM load segments for DFU BIN and RP2040 UF2 evidence.
pub(super) struct Segment {
    pub address: u32,
    pub end: u32,
    pub offset: usize,
    pub size: usize,
}

fn invalid() -> String {
    "exige ELF32 little endian ARM ET_EXEC válido, sem carregamento dinâmico".into()
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

pub(super) fn segments(elf: &[u8], lower: u32, upper: u32) -> Result<Vec<Segment>, String> {
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
        if address < lower || end > upper {
            return Err("ELF tem bytes carregáveis fora da flash permitida neste recorte".into());
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
