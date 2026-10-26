use super::{parse_listing, query};
use kinein_protocol::DfuMode;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const ROW: &str = "Found DFU: [0483:df11] ver=2200, devnum=2, cfg=1, intf=0, path=\"1-2\", alt=0, name=\"@Internal Flash /0x08000000/04*016Kg\", serial=\"STM-1\"";
const BANNER: &str = "dfu-util 0.11\n\nCopyright 2005-2009 Weston Schmidt, Harald Welte and OpenMoko Inc.\nCopyright 2010-2021 Tormod Volden and Stefan Schmidt\nThis program is Free Software and has ABSOLUTELY NO WARRANTY\nPlease report bugs to http://sourceforge.net/p/dfu-util/tickets/\n\n";

fn tool(name: &str, body: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("kinein-dfu-query-{}-{name}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    crate::write_executable(
        root.join("dfu-util"),
        format!("#!/bin/sh\n[ \"$#\" = 1 ] && [ \"$1\" = --list ] || exit 88\n{body}\n"),
    )
}

#[test]
fn alternatives_and_runtime_keep_their_descriptors() {
    let runtime = ROW
        .replace("Found DFU:", "Found Runtime:")
        .replace("alt=0", "alt=1");
    let entries = parse_listing(&format!("{BANNER}{ROW}\n{runtime}\n")).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].mode, DfuMode::Dfu);
    assert_eq!(entries[0].vid, "0483");
    assert_eq!(entries[0].pid, "df11");
    assert_eq!(entries[0].release, "2200");
    assert_eq!(entries[0].device_number, 2);
    assert_eq!(entries[0].configuration, 1);
    assert_eq!(entries[0].interface, 0);
    assert_eq!(entries[0].path, "1-2");
    assert_eq!(entries[0].alternate, 0);
    assert!(entries[0].name.starts_with("@Internal Flash"));
    assert_eq!(entries[0].serial.as_deref(), Some("STM-1"));
    assert_eq!(entries[1].mode, DfuMode::Runtime);
    assert_eq!(entries[1].alternate, 1);
    for serial in ["UNKNOWN", ""] {
        assert!(
            parse_listing(&format!("{BANNER}{}", ROW.replace("STM-1", serial))).unwrap()[0]
                .serial
                .is_none()
        );
    }
    let quoted = ROW
        .replace("@Internal Flash", "@Flash \"quoted\"")
        .replace("STM-1", "serial\"quoted");
    let entries = parse_listing(&format!("{BANNER}{quoted}")).unwrap();
    assert!(entries[0].name.contains("\"quoted\""));
    assert_eq!(entries[0].serial.as_deref(), Some("serial\"quoted"));
}

#[test]
fn warnings_unknown_formats_and_unsupported_versions_refuse_partial_results() {
    for output in [
        format!(
            "{BANNER}{ROW}\ndfu-util: Cannot open DFU device 0483:df11 found on devnum 2 (LIBUSB_ERROR_ACCESS)"
        ),
        format!("{BANNER}{ROW}\nwarning: incomplete USB discovery"),
        format!("{BANNER}{}", ROW.replace("alt=0", "alt=256")),
        format!("{BANNER}{}", ROW.replace("cfg=1", "cfg=+1")),
        format!("{BANNER}{}", ROW.replace("[0483:df11]", "[483:df11]")),
        format!("{BANNER}{}", ROW.replace("serial=", "unknown=")),
        format!(
            "{BANNER}{}",
            ROW.replace("STM-1", "x\", serial=\"ambiguous")
        ),
        format!("{}{ROW}", BANNER.replace("0.11", "0.12")),
        ROW.to_owned(),
        String::new(),
    ] {
        assert!(
            parse_listing(&output).is_err(),
            "partial/unverified output accepted: {output}"
        );
    }
    assert!(parse_listing(BANNER).unwrap().is_empty());
}

#[test]
fn list_uses_only_the_literal_discovery_option() {
    let tool = tool("argv", &format!("cat <<'OUTPUT'\n{BANNER}{ROW}\nOUTPUT"));
    let result = query(Some(&tool), Duration::from_secs(2), &Arc::default());
    assert!(result.tool_available && result.complete, "{result:?}");
    assert_eq!(result.interfaces.len(), 1);
    assert_eq!(result.tool.as_deref(), tool.to_str());
    assert!(result.raw_output.contains(ROW));
}

#[test]
fn zero_exit_with_access_warning_is_not_a_verified_empty_or_partial_list() {
    let tool = tool(
        "warning",
        &format!(
            "echo 'dfu-util 0.11'\necho '{ROW}'\necho 'dfu-util: Cannot open DFU device (LIBUSB_ERROR_ACCESS)' >&2\nexit 0"
        ),
    );
    let result = query(Some(&tool), Duration::from_secs(2), &Arc::default());
    assert!(result.tool_available && !result.complete);
    assert!(result.interfaces.is_empty());
    assert!(result.raw_output.contains("LIBUSB_ERROR_ACCESS"));
    assert!(result.hint.unwrap().contains("não foi verificada"));
}

#[test]
fn exit_failure_never_publishes_recognized_rows() {
    let tool = tool(
        "failure",
        &format!("echo 'dfu-util 0.11'\necho '{ROW}'\nexit 7"),
    );
    let result = query(Some(&tool), Duration::from_secs(2), &Arc::default());
    assert!(result.tool_available && !result.complete && result.interfaces.is_empty());
    assert!(result.hint.unwrap().contains('7'));
}

#[test]
fn missing_tool_is_distinct_from_an_empty_complete_listing() {
    for program in [None, Some(std::path::Path::new("/missing/kinein/dfu-util"))] {
        let result = query(program, Duration::from_secs(1), &Arc::default());
        assert!(!result.tool_available && !result.complete && result.interfaces.is_empty());
        assert!(result.hint.is_some());
    }
    let tool = tool("empty", "echo 'dfu-util 0.11'");
    let result = query(Some(&tool), Duration::from_secs(1), &Arc::default());
    assert!(result.complete && result.tool_available && result.interfaces.is_empty());
    assert!(result.hint.unwrap().contains("Nenhuma interface"));
}

#[test]
fn deadline_and_cancellation_stop_the_query_and_discard_partial_results() {
    for cancelled in [false, true] {
        let tool = tool(
            if cancelled { "cancel" } else { "timeout" },
            &format!("echo 'dfu-util 0.11'\necho '{ROW}'\nsleep 30"),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        if cancelled {
            let token = cancel.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(100));
                token.store(true, Ordering::SeqCst);
            });
        }
        let start = Instant::now();
        let result = query(
            Some(&tool),
            Duration::from_millis(if cancelled { 2000 } else { 100 }),
            &cancel,
        );
        assert!(start.elapsed() < Duration::from_secs(3));
        assert!(result.tool_available && !result.complete && result.interfaces.is_empty());
        assert!(
            result
                .hint
                .unwrap()
                .contains(if cancelled { "cancelada" } else { "prazo" })
        );
    }
}

#[test]
fn overflow_bounds_retained_output_without_publishing_partial_interfaces() {
    let tool = tool(
        "overflow",
        "echo 'dfu-util 0.11'\ni=0; while [ $i -lt 3000 ]; do echo '0123456789012345678901234567890123456789'; i=$((i+1)); done\nsleep 30",
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let result = query(Some(&tool), Duration::from_secs(3), &cancel);
    assert!(result.tool_available && !result.complete && result.interfaces.is_empty());
    assert!(result.raw_output.len() <= 65_536);
    assert!(result.hint.unwrap().contains("64 KiB"));
    assert!(!cancel.load(Ordering::SeqCst));
}
