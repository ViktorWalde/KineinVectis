//! `DfuSe` 1 + DFU suffix, from the public file format (52 §14.52).
//! No firmware bytes are generated or interpreted as executable host code.
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) const MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct Image {
    pub alternate: u8,
    pub release: u16,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Element {
    pub address: u32,
    pub size: u32,
}

pub(super) fn parse(bytes: &[u8], cancel: &AtomicBool) -> Result<Image, String> {
    if bytes.len() < 11 + 274 + 8 + 1 + 16 || bytes.len() > MAX_BYTES {
        return Err("DfuSe inválido: arquivo vazio/truncado ou maior que 32 MiB".to_owned());
    }
    let end = bytes.len() - 16;
    let suffix = &bytes[end..];
    if &suffix[8..11] != b"UFD"
        || suffix[11] != 16
        || word(suffix, 6)? != 0x011a
        || word(suffix, 4)? != 0x0483
        || word(suffix, 2)? != 0xdf11
    {
        return Err(
            "DfuSe exige sufixo UFD 1.1a de 16 bytes e destino explícito 0483:df11".to_owned(),
        );
    }
    if checksum(&bytes[..bytes.len() - 4], cancel)? != quad(suffix, 12)? {
        return Err(
            "CRC do arquivo DfuSe difere do sufixo; nenhuma gravação foi iniciada".to_owned(),
        );
    }
    if &bytes[..5] != b"DfuSe"
        || bytes[5] != 1
        || bytes[10] != 1
        || usize::try_from(quad(bytes, 6)?).ok() != Some(bytes.len())
    {
        return Err(
            "DfuSe exige versão 1, tamanho total correto e exatamente um Target".to_owned(),
        );
    }
    let target = &bytes[11..285];
    if &target[..6] != b"Target" || quad(target, 7)? > 1 {
        return Err("prefixo Target DfuSe inválido".to_owned());
    }
    if quad(target, 7)? == 1 && !target[11..266].contains(&0) {
        return Err("nome Target DfuSe sem terminador".to_owned());
    }
    let count = quad(target, 270)?;
    if !(1..=128).contains(&count) || usize::try_from(quad(target, 266)?).ok() != Some(end - 285) {
        return Err("tamanho/contagem de elementos DfuSe inválidos (1–128)".to_owned());
    }
    let mut offset = 285;
    let mut previous_end = 0_u32;
    let mut elements = Vec::new();
    for _ in 0..count {
        if cancel.load(Ordering::SeqCst) {
            return Err("preparação DfuSe cancelada".to_owned());
        }
        if offset + 8 > end {
            return Err("cabeçalho de elemento DfuSe truncado".to_owned());
        }
        let address = quad(bytes, offset)?;
        let size = quad(bytes, offset + 4)?;
        let range_end = address
            .checked_add(size)
            .filter(|_| size > 0)
            .ok_or("elemento DfuSe vazio ou com overflow de endereço")?;
        if address < previous_end {
            return Err("elementos DfuSe fora de ordem ou sobrepostos".to_owned());
        }
        offset = offset
            .checked_add(8)
            .and_then(|offset| {
                usize::try_from(size)
                    .ok()
                    .and_then(|size| offset.checked_add(size))
            })
            .filter(|offset| *offset <= end)
            .ok_or("dados de elemento DfuSe truncados")?;
        previous_end = range_end;
        elements.push(Element { address, size });
    }
    if offset != end {
        return Err("bytes excedentes após os elementos DfuSe".to_owned());
    }
    Ok(Image {
        alternate: target[6],
        release: word(suffix, 0)?,
        elements,
    })
}

fn word(bytes: &[u8], offset: usize) -> Result<u16, String> {
    bytes
        .get(offset..offset + 2)
        .and_then(|value| value.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "campo DfuSe truncado".to_owned())
}

fn quad(bytes: &[u8], offset: usize) -> Result<u32, String> {
    bytes
        .get(offset..offset + 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "campo DfuSe truncado".to_owned())
}

const fn table() -> [u32; 256] {
    let mut values = [0; 256];
    let mut index = 0;
    let mut seed = 0_u32;
    while index < 256 {
        let mut value = seed;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 1 {
                (value >> 1) ^ 0xedb8_8320
            } else {
                value >> 1
            };
            bit += 1;
        }
        values[index] = value;
        index += 1;
        seed += 1;
    }
    values
}

pub(super) fn checksum(bytes: &[u8], cancel: &AtomicBool) -> Result<u32, String> {
    const TABLE: [u32; 256] = table();
    let mut crc = u32::MAX;
    for chunk in bytes.chunks(65_536) {
        if cancel.load(Ordering::SeqCst) {
            return Err("preparação DfuSe cancelada".to_owned());
        }
        for byte in chunk {
            crc = (crc >> 8) ^ TABLE[usize::from(crc.to_le_bytes()[0] ^ byte)];
        }
    }
    Ok(crc)
}
