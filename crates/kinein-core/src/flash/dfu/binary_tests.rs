use super::binary::validate;
use std::sync::atomic::AtomicBool;

const ELF: &[u8] = include_bytes!("fixtures/cmake-bin.elf");
const BIN: &[u8] = include_bytes!("fixtures/cmake-bin.bin");

fn valid(elf: &[u8], bin: &[u8]) -> Result<u32, String> {
    validate(elf, bin, &AtomicBool::new(false))
}

fn quad(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn bin_matches_real_arm_elf_load_addresses_data_and_bss() {
    assert_eq!(valid(ELF, BIN).unwrap(), 0x0800_4000);
    assert_eq!(BIN.len(), 32);
    assert!(
        validate(ELF, BIN, &AtomicBool::new(true))
            .unwrap_err()
            .contains("cancelada")
    );
    let mut changed = BIN.to_vec();
    changed[31] ^= 1;
    assert!(valid(ELF, &changed).unwrap_err().contains("bytes"));
    for bin in [&BIN[..31], &[]] {
        assert!(valid(ELF, bin).is_err());
    }
    let mut padded = BIN.to_vec();
    padded.push(0);
    assert!(valid(ELF, &padded).unwrap_err().contains("tamanho"));
}

#[test]
fn gaps_are_zero_and_physical_ranges_never_overlap_or_escape_flash() {
    let mut elf = ELF.to_vec();
    quad(&mut elf, 84 + 12, 0x0800_4020);
    let mut bin = BIN[..28].to_vec();
    bin.extend([0; 4]);
    bin.extend(&BIN[28..]);
    assert_eq!(valid(&elf, &bin).unwrap(), 0x0800_4000);
    bin[29] = 0xff;
    assert!(valid(&elf, &bin).unwrap_err().contains("gaps"));
    for address in [0x0800_4010, 0x2000_0000, u32::MAX - 1] {
        let mut elf = ELF.to_vec();
        quad(&mut elf, 84 + 12, address);
        assert!(valid(&elf, BIN).is_err());
    }
}

#[test]
fn elf_headers_dynamic_loading_sizes_and_alignment_are_checked() {
    for (offset, value) in [
        (0, b'X'),
        (4, 2),
        (5, 2),
        (6, 2),
        (16, 3),
        (18, 0x3e),
        (20, 2),
        (40, 0),
        (42, 0),
        (44, 0),
        (44, 129),
    ] {
        let mut elf = ELF.to_vec();
        elf[offset] = value;
        assert!(valid(&elf, BIN).is_err(), "header {offset}");
    }
    for (offset, value) in [
        (24, 0_u32),
        (28, u32::MAX),
        (52, 2),
        (52, 3),
        (56, u32::MAX),
        (68, u32::MAX),
        (72, 0),
        (76, 4),
        (80, 3),
    ] {
        let mut elf = ELF.to_vec();
        quad(&mut elf, offset, value);
        assert!(valid(&elf, BIN).is_err(), "field {offset}");
    }
    for end in [0, 51, 80, 16390] {
        assert!(valid(&ELF[..end], BIN).is_err());
    }
}

#[test]
fn raw_binary_never_allows_dfu_util_to_strip_prefixes_or_suffixes() {
    for prefix in [b"DfuSe".as_slice(), &[1, 0], &[0x1a, 0x3f], &[0x5a, 0x7f]] {
        let mut bin = BIN.to_vec();
        bin[..prefix.len()].copy_from_slice(prefix);
        let mut elf = ELF.to_vec();
        elf[0x4000..0x4000 + prefix.len()].copy_from_slice(prefix);
        assert!(valid(&elf, &bin).unwrap_err().contains("prefixo"));
    }
    let mut bin = BIN.to_vec();
    let end = bin.len();
    bin[end - 8..end - 5].copy_from_slice(b"UFD");
    let mut elf = ELF.to_vec();
    // The last eight BIN bytes span the tail of text and the data load image.
    elf[0x4000 + 24..0x4000 + 27].copy_from_slice(b"UFD");
    assert!(valid(&elf, &bin).unwrap_err().contains("sufixo"));
}
