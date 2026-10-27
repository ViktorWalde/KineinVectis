use super::uf2;
use std::sync::atomic::AtomicBool;

use super::fixture::{elf, image};

fn validate(elf: &[u8], image: &[u8]) -> Result<uf2::Image, String> {
    uf2::validate(elf, image, &AtomicBool::new(false))
}

#[test]
fn uf2_matches_load_bytes_and_official_touched_sector_padding_without_writing_bss() {
    let result = validate(&elf(), &image()).unwrap();
    assert_eq!(
        (result.start, result.end, result.pages),
        (0x1000_0000, 0x1000_2100, 17)
    );
    assert!(
        uf2::validate(&elf(), &image(), &AtomicBool::new(true))
            .unwrap_err()
            .contains("cancelada")
    );
    let mut changed = image();
    changed[512 + 35] ^= 1;
    assert!(validate(&elf(), &changed).unwrap_err().contains("bytes"));
    let mut changed = image();
    changed[32] = 1;
    assert!(validate(&elf(), &changed).unwrap_err().contains("padding"));
    let mut changed = image();
    changed[16 * 512 + 40] = 1;
    assert!(
        validate(&elf(), &changed).is_err(),
        "BSS is not initialized data"
    );
}

#[test]
fn uf2_rejects_other_families_flags_addressing_order_counts_and_partial_blocks() {
    for (offset, value) in [
        (0, 0_u32),
        (4, 0),
        (508, 0),
        (8, 0x2001),
        (8, 0x6000),
        (8, 0xa000),
        (8, 0x3000),
        (16, 128),
        (20, 1),
        (24, 18),
        (28, 0xe48b_ff59),
        (12, 0x2000_0000),
        (12, 0x1000_0001),
        (512 + 12, 0x1000_0000),
        (512 + 20, 0),
    ] {
        let mut bytes = image();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(validate(&elf(), &bytes).is_err(), "header {offset}={value}");
    }
    let mut bytes = image();
    bytes[288] = 1;
    assert!(validate(&elf(), &bytes).is_err());
    for end in [0, 1, 511, 512, image().len() - 1] {
        assert!(validate(&elf(), &image()[..end]).is_err());
    }
}

#[test]
fn uf2_rejects_extra_missing_or_invalid_padding_pages_and_elf_ranges() {
    let mut extra = image();
    extra.extend_from_slice(&image()[..512]);
    assert!(validate(&elf(), &extra).is_err());
    let mut missing = image();
    missing.drain(..512);
    assert!(validate(&elf(), &missing).is_err());
    for (offset, value) in [
        (18, 0x3e_u32),
        (24, 0x2000_0101),
        (28, 0xffff_ffff),
        (52, 2),
        (52 + 12, 0x2000_0100),
        (52 + 16, 17),
        (52 + 20, 8),
        (84 + 12, 0x1000_0104),
    ] {
        let mut bytes = elf();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(validate(&bytes, &image()).is_err(), "ELF {offset}");
    }
    let mut sparse = image();
    sparse[16 * 512 + 12..16 * 512 + 16].copy_from_slice(&0x1000_1000_u32.to_le_bytes());
    assert!(
        validate(&elf(), &sparse).is_err(),
        "untouched whole sectors stay absent"
    );
}
