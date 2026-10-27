//! A bounded BIN/ELF comparison, not an ELF converter or loader (52 §14.53).
use std::sync::atomic::{AtomicBool, Ordering};

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
    let segments = super::super::elf::segments(elf, 0x0800_0000, 0x0900_0000)?;
    let start = segments
        .first()
        .ok_or("ELF sem segmentos carregáveis")?
        .address;
    let end = segments.last().ok_or("ELF sem segmentos carregáveis")?.end;
    if usize::try_from(end - start).ok() != Some(binary.len()) {
        return Err(
            "tamanho do BIN difere do intervalo de carga do ELF; padding extra não é aceito".into(),
        );
    }
    let mut cursor = 0;
    for segment in segments {
        cancelled(cancel)?;
        let offset = usize::try_from(segment.address - start).map_err(|_| "offset BIN inválido")?;
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
