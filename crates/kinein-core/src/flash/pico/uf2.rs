//! Canonical RP2040 elf2uf2 pages, compared byte for byte with bounded ELF segments.
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicBool, Ordering},
};

pub(super) const MAX_BYTES: usize = 32 * 1024 * 1024;
const BASE: u32 = 0x1000_0000;
const END: u32 = 0x1100_0000;

#[derive(Debug)]
pub(super) struct Image {
    pub start: u32,
    pub end: u32,
    pub pages: u32,
}

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        Err("preparação UF2 cancelada".into())
    } else {
        Ok(())
    }
}

/// Match exactly the page set produced by audited elf2uf2, including its sector padding.
pub(super) fn validate(elf: &[u8], uf2: &[u8], cancel: &AtomicBool) -> Result<Image, String> {
    cancelled(cancel)?;
    if uf2.is_empty() || uf2.len() > MAX_BYTES || !uf2.len().is_multiple_of(512) {
        return Err("UF2 precisa conter blocos completos de 512 bytes e ter até 32 MiB".into());
    }
    let segments = super::super::elf::segments(elf, BASE, END)?;
    let expected = pages(&segments);
    if expected.len() != uf2.len() / 512 {
        return Err("páginas UF2 diferem das páginas e padding do ELF escolhido".into());
    }
    let count = u32::try_from(expected.len()).map_err(|e| e.to_string())?;
    let mut start = 0;
    let mut end = 0;
    for (i, (block, address)) in uf2.chunks_exact(512).zip(expected).enumerate() {
        cancelled(cancel)?;
        if word(block, 0) != 0x0a32_4655
            || word(block, 4) != 0x9e5d_5157
            || word(block, 508) != 0x0ab1_6f30
            || word(block, 8) != 0x2000
            || word(block, 16) != 256
            || word(block, 20) != u32::try_from(i).unwrap_or(u32::MAX)
            || word(block, 24) != count
            || word(block, 28) != 0xe48b_ff56
            || word(block, 12) != address
            || block[288..508].iter().any(|byte| *byte != 0)
        {
            return Err(
                "UF2 RP2040 com cabeçalho, família, flags, ordem, endereço ou padding inválido"
                    .into(),
            );
        }
        if i == 0 {
            start = address;
        }
        end = address + 256;
        let mut data = [0_u8; 256];
        for segment in &segments {
            let from = address.max(segment.address);
            let to = end.min(segment.end);
            if from < to {
                let offset = segment.offset + (from - segment.address) as usize;
                data[(from - address) as usize..(to - address) as usize]
                    .copy_from_slice(&elf[offset..offset + (to - from) as usize]);
            }
        }
        if block[32..288] != data {
            return Err("bytes UF2 diferem dos segmentos ELF ou do padding zero conferido".into());
        }
    }
    Ok(Image {
        start,
        end,
        pages: count,
    })
}

fn pages(segments: &[super::super::elf::Segment]) -> BTreeSet<u32> {
    let mut result = BTreeSet::new();
    for segment in segments {
        for address in (segment.address & !255..segment.end).step_by(256) {
            result.insert(address);
        }
    }
    let last = result.last().copied().unwrap_or(BASE);
    let sectors: BTreeSet<_> = result.iter().map(|address| address & !4095).collect();
    for sector in sectors {
        for address in (sector..sector + 4096)
            .step_by(256)
            .take_while(|address| *address < last)
        {
            result.insert(address);
        }
    }
    result
}
