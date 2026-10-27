//! Synthetic ARM/UF2 bytes shared by format and real-dispatch tests only.
/// ARM ELF with XIP code, RAM data loaded from flash and separate BSS.
#[must_use]
pub fn elf() -> Vec<u8> {
    let mut bytes = vec![0_u8; 280];
    bytes[..7].copy_from_slice(b"\x7fELF\x01\x01\x01");
    for (offset, value) in [(16, 2_u16), (18, 0x28), (40, 52), (42, 32), (44, 3)] {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    for (offset, value) in [(20, 1_u32), (24, 0x1000_0101), (28, 52)] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (i, values) in [
        [1_u32, 256, 0x1000_0100, 0x1000_0100, 16, 16, 5, 4],
        [1, 272, 0x2000_0000, 0x1000_2000, 8, 16, 6, 4],
        [1, 280, 0x2000_0010, 0x1000_2008, 0, 32, 6, 4],
    ]
    .into_iter()
    .enumerate()
    {
        for (j, value) in values.into_iter().enumerate() {
            let offset = 52 + i * 32 + j * 4;
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    for (i, byte) in bytes[256..272].iter_mut().enumerate() {
        *byte = u8::try_from(i).unwrap();
    }
    bytes[272..280].copy_from_slice(&[0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7]);
    bytes
}

/// Audited canonical UF2 pages for the ELF fixture, including sector padding.
#[must_use]
pub fn image() -> Vec<u8> {
    let mut bytes = vec![0_u8; 17 * 512];
    for (i, block) in bytes.chunks_exact_mut(512).enumerate() {
        let address = if i == 16 {
            0x1000_2000
        } else {
            0x1000_0000 + u32::try_from(i).unwrap() * 256
        };
        for (offset, value) in [
            (0, 0x0a32_4655_u32),
            (4, 0x9e5d_5157),
            (8, 0x2000),
            (12, address),
            (16, 256),
            (20, u32::try_from(i).unwrap()),
            (24, 17),
            (28, 0xe48b_ff56),
            (508, 0x0ab1_6f30),
        ] {
            block[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        if i == 1 {
            block[32..48].copy_from_slice(&elf()[256..272]);
        }
        if i == 16 {
            block[32..40].copy_from_slice(&elf()[272..280]);
        }
    }
    bytes
}
