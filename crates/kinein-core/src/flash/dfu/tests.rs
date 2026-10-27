use super::{check_selection, image, memory, selected_interface};
use kinein_protocol::{
    DfuDestination, DfuFlashSelection, DfuInterface, DfuMode, FlashProposalParams,
};
use std::sync::atomic::AtomicBool;

// Produced by the official dfu-util 0.11 dfuse-pack.py, not this parser.
const VALID: &[u8] = include_bytes!("fixtures/valid.dfu");
const MAP: &str = "@Internal Flash /0x08000000/04*016Kg,01*064Kg,07*128Kg";

fn crc(bytes: &mut [u8]) {
    let end = bytes.len() - 4;
    let value = image::checksum(&bytes[..end], &AtomicBool::new(false)).unwrap();
    bytes[end..].copy_from_slice(&value.to_le_bytes());
}

fn destination() -> DfuDestination {
    DfuDestination {
        vid: "0483".into(),
        pid: "df11".into(),
        serial: "FIXTURE-1".into(),
        path: "1-2".into(),
        configuration: 1,
        interface: 0,
        alternate: 0,
    }
}

fn entry() -> DfuInterface {
    DfuInterface {
        mode: DfuMode::Dfu,
        vid: "0483".into(),
        pid: "df11".into(),
        serial: Some("FIXTURE-1".into()),
        path: "1-2".into(),
        configuration: 1,
        interface: 0,
        alternate: 0,
        release: "2200".into(),
        device_number: 2,
        name: MAP.into(),
    }
}

#[test]
fn native_dfu_file_has_valid_crc_target_and_address() {
    let parsed = image::parse(VALID, &AtomicBool::new(false)).unwrap();
    assert_eq!(parsed.alternate, 0);
    assert_eq!(parsed.release, 0);
    assert_eq!(
        parsed.elements,
        vec![image::Element {
            address: 0x0800_0000,
            size: 24
        }]
    );
    memory::validate(MAP, &parsed.elements).unwrap();
    let cancelled = AtomicBool::new(true);
    assert!(
        image::parse(VALID, &cancelled)
            .unwrap_err()
            .contains("cancelada")
    );
}

#[test]
fn corrupted_crc_and_truncation_are_never_accepted() {
    let mut bytes = VALID.to_vec();
    bytes[293] ^= 1;
    assert!(
        image::parse(&bytes, &AtomicBool::new(false))
            .unwrap_err()
            .contains("CRC")
    );
    for end in [0, 10, 285, 293, VALID.len() - 1] {
        assert!(
            image::parse(&VALID[..end], &AtomicBool::new(false)).is_err(),
            "truncation at {end}"
        );
    }
}

#[test]
fn multiple_elements_require_ordered_ranges_and_exact_target_size() {
    let suffix = VALID.len() - 16;
    let mut bytes = VALID[..suffix].to_vec();
    let second = bytes.len();
    bytes.extend_from_slice(&VALID[285..suffix]);
    bytes[second..second + 4].copy_from_slice(&0x0800_0020_u32.to_le_bytes());
    bytes.extend_from_slice(&VALID[suffix..]);
    bytes[281..285].copy_from_slice(&2_u32.to_le_bytes());
    bytes[277..281].copy_from_slice(&64_u32.to_le_bytes());
    let length = u32::try_from(bytes.len()).unwrap();
    bytes[6..10].copy_from_slice(&length.to_le_bytes());
    crc(&mut bytes);
    let image = image::parse(&bytes, &AtomicBool::new(false)).unwrap();
    assert_eq!(image.elements.len(), 2);
    memory::validate(MAP, &image.elements).unwrap();
    for address in [0x0800_0010_u32, 0x07ff_ffff] {
        let mut invalid = bytes.clone();
        invalid[second..second + 4].copy_from_slice(&address.to_le_bytes());
        crc(&mut invalid);
        assert!(
            image::parse(&invalid, &AtomicBool::new(false))
                .unwrap_err()
                .contains("sobrepostos")
        );
    }
    let mut trailing = VALID[..suffix].to_vec();
    trailing.push(0x99);
    trailing.extend_from_slice(&VALID[suffix..]);
    let length = u32::try_from(trailing.len()).unwrap();
    trailing[6..10].copy_from_slice(&length.to_le_bytes());
    trailing[277..281].copy_from_slice(&33_u32.to_le_bytes());
    crc(&mut trailing);
    assert!(
        image::parse(&trailing, &AtomicBool::new(false))
            .unwrap_err()
            .contains("excedentes")
    );
}

#[test]
fn malformed_target_sizes_suffix_and_addresses_are_refused_with_valid_crc() {
    for (offset, value) in [
        (5, 2),
        (10, 2),
        (11, b'X'),
        (17, 1),
        (18, 2),
        (281, 0),
        (VALID.len() - 8, b'X'),
        (VALID.len() - 5, 17),
    ] {
        let mut bytes = VALID.to_vec();
        bytes[offset] = value;
        crc(&mut bytes);
        // Alternate is structurally valid; it is compared with USB in prepare.
        if offset == 17 {
            assert_eq!(
                image::parse(&bytes, &AtomicBool::new(false))
                    .unwrap()
                    .alternate,
                1
            );
        } else {
            assert!(
                image::parse(&bytes, &AtomicBool::new(false)).is_err(),
                "field {offset}"
            );
        }
    }
    for (offset, value) in [
        (6, 0_u32),
        (277, 0),
        (281, 129),
        (285, u32::MAX),
        (289, 0),
        (289, u32::MAX),
    ] {
        let mut bytes = VALID.to_vec();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        crc(&mut bytes);
        assert!(
            image::parse(&bytes, &AtomicBool::new(false)).is_err(),
            "field {offset}={value}"
        );
    }
    let mut bytes = VALID.to_vec();
    bytes[22..277].fill(b'X');
    crc(&mut bytes);
    assert!(
        image::parse(&bytes, &AtomicBool::new(false))
            .unwrap_err()
            .contains("terminador")
    );
    for offset in [VALID.len() - 12, VALID.len() - 14, VALID.len() - 10] {
        let mut bytes = VALID.to_vec();
        bytes[offset] ^= 1;
        crc(&mut bytes);
        assert!(image::parse(&bytes, &AtomicBool::new(false)).is_err());
    }
}

#[test]
fn memory_map_requires_internal_erase_and_write_ranges() {
    let element = vec![image::Element {
        address: 0x0800_1000,
        size: 0x1000,
    }];
    for map in [
        "@Option Bytes /0x1fff7800/01*016Kg",
        "@Internal Flash /0x08000000/01*016Ka",
        "@Internal Flash /0x08000000/01*016Ke",
        "@Internal Flash /0x08000000/01*000Kg",
        "@Internal Flash /0x08000000/999999999*999999999Mg",
        "@Internal Flash /0x90000000/01*016Kg",
        "@Internal Flash /0x08000000/01*016Kg/0x08000000/01*016Kg",
        "@Internal Flash /0x08000000/01*016Kz",
    ] {
        assert!(memory::validate(map, &element).is_err(), "{map}");
    }
    memory::validate("@Internal Flash /0x08000000/01*004Ka,03*004Kg", &element).unwrap();
    let outside = vec![image::Element {
        address: 0x07ff_ffff,
        size: 2,
    }];
    assert!(memory::validate(MAP, &outside).is_err());
    let crossing = vec![image::Element {
        address: 0x0800_3000,
        size: 0x2000,
    }];
    assert!(memory::validate("@Internal Flash /0x08000000/01*016Kg,01*016Ka", &crossing).is_err());
}

#[test]
fn destination_must_name_one_alternative_of_one_dfu_instance() {
    let selected = destination();
    let entry = entry();
    assert_eq!(
        selected_interface(&selected, std::slice::from_ref(&entry)).unwrap(),
        entry
    );
    for entries in [
        vec![],
        vec![entry.clone(), entry.clone()],
        vec![DfuInterface {
            mode: DfuMode::Runtime,
            ..entry.clone()
        }],
        vec![DfuInterface {
            path: "1-3".into(),
            ..entry.clone()
        }],
        vec![
            entry.clone(),
            DfuInterface {
                path: "1-3".into(),
                device_number: 3,
                ..entry.clone()
            },
        ],
    ] {
        assert!(selected_interface(&selected, &entries).is_err());
    }
    assert!(
        selected_interface(
            &selected,
            &[
                entry.clone(),
                DfuInterface {
                    alternate: 1,
                    ..entry
                }
            ]
        )
        .is_ok()
    );
}

#[test]
fn selection_refuses_foreign_fields_and_selector_separators() {
    let base = FlashProposalParams {
        native: true,
        engine: Some("dfu-util".into()),
        dfu: Some(DfuFlashSelection {
            image: "firmware.dfu".into(),
            binary_target: None,
            destination: destination(),
        }),
        ..FlashProposalParams::default()
    };
    assert!(check_selection(&base).is_ok());
    for selection in [
        FlashProposalParams {
            device: Some("/dev/ttyUSB0".into()),
            ..base.clone()
        },
        FlashProposalParams {
            probe: Some("probe".into()),
            ..base.clone()
        },
        FlashProposalParams {
            target: Some("app".into()),
            ..base.clone()
        },
        FlashProposalParams {
            firmware: Some("catalog".into()),
            ..base.clone()
        },
        FlashProposalParams {
            build_directory: Some("build".into()),
            ..base.clone()
        },
        FlashProposalParams {
            flash_size_bytes: Some(123),
            ..base.clone()
        },
        FlashProposalParams {
            dfu: None,
            ..base.clone()
        },
    ] {
        assert!(check_selection(&selection).is_err());
    }
    for value in ["", "UNKNOWN", "a,b", "a\n", " a"] {
        let mut selection = base.clone();
        selection.dfu.as_mut().unwrap().destination.serial = value.into();
        assert!(check_selection(&selection).is_err(), "serial {value:?}");
    }
    for path in ["/dev/ttyUSB0", "1-02", "0-1", "1-0", "1-1..2", "1-1:2"] {
        let mut selection = base.clone();
        selection.dfu.as_mut().unwrap().destination.path = path.into();
        assert!(check_selection(&selection).is_err(), "path {path}");
    }
}
