use super::parser;
use std::fmt::Write;

const ONE: &str =
    "Device Information\n type: RP2040\n flash size: 2048K\n flash id: 0xE660583883355B21\n";

fn multiple(ids: &[&str]) -> String {
    let mut text = "Multiple RP-series devices in BOOTSEL mode found:\n".to_owned();
    for (i, id) in ids.iter().enumerate() {
        let header = format!("RP2040 device at bus 1, address {}:", i + 2);
        write!(
            text,
            "\n{header}\n{}\n{}",
            "-".repeat(header.len()),
            ONE.replace("E660583883355B21", id)
        )
        .unwrap();
    }
    text
}

#[test]
fn audited_versions_are_explicit_and_usb_disabled_builds_are_not_accepted() {
    for version in ["2.2.0", "2.3.1"] {
        assert_eq!(
            parser::version(&format!("picotool v{version} (Linux, GNU-15)\n")).unwrap(),
            version
        );
    }
    for value in [
        "2.3.1",
        "picotool v2.3.0 (Linux)",
        "picotool v9.0.0 (Linux)",
        "picotool v2.2.0 (Linux)\nThis version of picotool was compiled without USB support.",
    ] {
        assert!(parser::version(value).is_err());
    }
}

#[test]
fn single_and_multiple_reports_keep_id_and_capacity_without_invented_addresses() {
    let devices = parser::devices(ONE).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].serial_selector, "E660583883355B21");
    assert_eq!(devices[0].flash_size_bytes, 2 * 1024 * 1024);
    let many = parser::devices(&multiple(&["E660583883355B21", "E660583883355B22"])).unwrap();
    assert_eq!(many.len(), 2);
    let lower = ONE.replace("E660583883355B21", "e660583883355b21");
    assert_eq!(parser::devices(&lower).unwrap(), devices);
}

#[test]
fn incomplete_ambiguous_unknown_and_protected_output_never_becomes_devices() {
    for value in [
        String::new(),
        ONE.replace(" type: RP2040\n", ""),
        ONE.replace("RP2040", "RP2350"),
        ONE.replace("2048K", "not determined due to access permissions"),
        ONE.replace("2048K", "4294967295K"),
        ONE.replace("2048K", "0K"),
        ONE.replace("E660583883355B21", "0000000000000000"),
        ONE.replace("E660583883355B21", "FFFFFFFFFFFFFFFF"),
        ONE.replace("E660583883355B21", "E660!"),
        format!("{ONE}Warning: cannot access other device\n"),
        format!("{ONE} type: RP2040\n"),
        format!("{ONE} chipid: 123\n"),
        multiple(&["E660583883355B21", "E660583883355B21"]),
        format!("{ONE}{ONE}"),
    ] {
        assert!(parser::devices(&value).is_err(), "{value}");
    }
}

#[test]
fn multiple_headers_and_device_count_are_bounded() {
    let ids: Vec<_> = (1..=17).map(|n| format!("E66058388335{n:04X}")).collect();
    let refs: Vec<_> = ids.iter().map(String::as_str).collect();
    assert_eq!(parser::devices(&multiple(&refs[..16])).unwrap().len(), 16);
    assert!(parser::devices(&multiple(&refs)).is_err());
    let valid = multiple(&refs[..2]);
    for value in [
        valid.replace("address 3:", "address 128:"),
        valid.replace("address 3:", "address 2:"),
        valid.replace("---", "-x-"),
        valid.replace("Multiple RP-series devices in BOOTSEL mode found:\n", ""),
    ] {
        assert!(parser::devices(&value).is_err());
    }
}

#[test]
fn cancelled_waiter_does_not_release_the_active_usb_section_to_a_third_query() {
    use std::{
        sync::{Arc, Mutex, atomic::AtomicBool, atomic::Ordering},
        time::{Duration, Instant},
    };
    let gate = Arc::new(Mutex::new(()));
    let active = gate.lock().unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let waiter = {
        let (gate, cancelled) = (gate.clone(), cancelled.clone());
        std::thread::spawn(move || {
            super::exclusive(&gate, &cancelled, Instant::now() + Duration::from_secs(1)).is_none()
        })
    };
    cancelled.store(true, Ordering::SeqCst);
    assert!(waiter.join().unwrap());
    let deadline = Instant::now() + Duration::from_millis(60);
    assert!(super::exclusive(&gate, &AtomicBool::new(false), deadline).is_none());
    drop(active);
    assert!(
        super::exclusive(
            &gate,
            &AtomicBool::new(false),
            Instant::now() + Duration::from_secs(1)
        )
        .is_some()
    );
}
