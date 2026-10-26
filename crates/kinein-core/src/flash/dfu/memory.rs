//! Erasable/writeable ranges reported by the selected Internal Flash interface.
use super::image::Element;

#[derive(Debug)]
struct Region {
    start: u32,
    end: u32,
    writable: bool,
}

pub(super) fn validate(description: &str, elements: &[Element]) -> Result<(), String> {
    let regions = parse(description)?;
    for element in elements {
        let end = element
            .address
            .checked_add(element.size)
            .ok_or("overflow de endereço DfuSe")?;
        let mut cursor = element.address;
        for region in &regions {
            if cursor >= region.start && cursor < region.end {
                if !region.writable {
                    return Err("elemento DfuSe alcança memória protegida/não apagável".to_owned());
                }
                cursor = end.min(region.end);
            }
            if cursor == end {
                break;
            }
        }
        if cursor != end {
            return Err(
                "elemento DfuSe está fora da memória interna gravável anunciada".to_owned(),
            );
        }
    }
    Ok(())
}

fn parse(description: &str) -> Result<Vec<Region>, String> {
    let invalid = || "mapa DfuSe Internal Flash ausente, ambíguo ou sem suporte".to_owned();
    let mut parts = description.split('/');
    if parts.next().map(str::trim) != Some("@Internal Flash") {
        return Err(invalid());
    }
    let mut regions = Vec::new();
    while let Some(address) = parts.next() {
        let mut start =
            u32::from_str_radix(address.trim().strip_prefix("0x").ok_or_else(invalid)?, 16)
                .map_err(|_| invalid())?;
        let groups = parts.next().ok_or_else(invalid)?;
        for group in groups.split(',') {
            if regions.len() >= 128 {
                return Err(invalid());
            }
            let (count, size) = group.trim().split_once('*').ok_or_else(invalid)?;
            let bytes = size.as_bytes();
            if bytes.len() < 3 {
                return Err(invalid());
            }
            let unit = match bytes[bytes.len() - 2] {
                b'B' => 1,
                b'K' => 1024,
                b'M' => 1024 * 1024,
                _ => return Err(invalid()),
            };
            let access = bytes[bytes.len() - 1];
            if !(b'a'..=b'g').contains(&access) {
                return Err(invalid());
            }
            let numeric = |text: &str| -> Result<u32, String> {
                if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(invalid());
                }
                text.parse::<u32>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(invalid)
            };
            let length = numeric(count)?
                .checked_mul(numeric(&size[..size.len() - 2])?)
                .and_then(|value| value.checked_mul(unit))
                .ok_or_else(invalid)?;
            let end = start.checked_add(length).ok_or_else(invalid)?;
            if start < 0x0800_0000
                || end > 0x0900_0000
                || regions
                    .last()
                    .is_some_and(|region: &Region| region.end > start)
            {
                return Err(invalid());
            }
            regions.push(Region {
                start,
                end,
                writable: matches!(access, b'f' | b'g'),
            });
            start = end;
        }
    }
    if regions.is_empty() {
        return Err(invalid());
    }
    Ok(regions)
}
