use super::pico_native::{fixture, proposal, selection};
use crate::flash::pico::RESOURCE_KEY;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::json;
use std::{
    fs,
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn pico_artifact_bounds_path_format_and_capacity_refuse_before_usb_queries() {
    for change in [
        "outside-build",
        "external-alias",
        "large-uf2",
        "large-elf",
        "uf2-fifo",
        "elf-fifo",
        "capacity",
        "family",
    ] {
        let mut f = fixture(&format!("pico-bounds-{change}"));
        let build = f.root.join(".kinein/build");
        let image = build.join("firmware a'b.uf2");
        let elf = build.join("firmware.elf");
        let mut request = selection();
        match change {
            "outside-build" => {
                let path = f.root.join("elsewhere.uf2");
                fs::copy(&image, &path).unwrap();
                request["pico"]["image"] = json!(path);
            }
            "external-alias" => {
                let outside = f.root.with_extension("outside.uf2");
                fs::copy(&image, &outside).unwrap();
                fs::remove_file(&image).unwrap();
                std::os::unix::fs::symlink(outside, &image).unwrap();
            }
            "large-uf2" => fs::OpenOptions::new()
                .write(true)
                .open(&image)
                .unwrap()
                .set_len(32 * 1024 * 1024 + 512)
                .unwrap(),
            "large-elf" => fs::OpenOptions::new()
                .write(true)
                .open(&elf)
                .unwrap()
                .set_len(64 * 1024 * 1024 + 1)
                .unwrap(),
            "capacity" => request["pico"]["destination"]["flashSizeBytes"] = json!(8192),
            "family" => {
                let mut bytes = fs::read(&image).unwrap();
                bytes[28..32].copy_from_slice(&0xe48b_ff59_u32.to_le_bytes());
                fs::write(&image, bytes).unwrap();
            }
            kind => {
                let path = if kind == "uf2-fifo" { &image } else { &elf };
                fs::remove_file(path).unwrap();
                assert!(
                    std::process::Command::new("mkfifo")
                        .arg(path)
                        .status()
                        .unwrap()
                        .success()
                );
            }
        }
        let start = Instant::now();
        let result = f.rpc("runConfig.flashProposal", request);
        assert!(result.error.is_some(), "{change}");
        assert!(start.elapsed() < Duration::from_secs(2), "{change}");
        assert!(!f.root.join("query.ready").exists(), "{change}");
        assert!(
            f.core
                .serial_usage
                .acquire_usb(RESOURCE_KEY, "test", "released", None)
                .is_ok()
        );
    }
}

#[test]
fn pico_preparation_is_deferred_guarded_and_run_stop_cancels_before_launch() {
    for method in ["runConfig.flashProposal", "run.start"] {
        let mut f = fixture(&format!("pico-deferred-{method}"));
        let params = if method == "run.start" {
            json!({"flash":proposal(&mut f)["flash"]})
        } else {
            selection()
        };
        fs::remove_file(f.root.join("query.ready")).ok();
        fs::write(f.root.join("query.wait"), "").unwrap();
        let (tx, rx) = mpsc::channel();
        f.core.enable_continuations(tx);
        let (response_tx, responses) = mpsc::channel();
        f.core.enable_deferred_responses(response_tx);
        assert!(
            f.core
                .handle_request(&JsonRpcRequest::new(101_i64, method, Some(params)))
                .is_deferred()
        );
        let limit = Instant::now() + Duration::from_secs(2);
        while !f.root.join("query.ready").exists() && Instant::now() < limit {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(f.root.join("query.ready").exists());
        assert!(f.rpc("runConfig.list", json!({})).error.is_none());
        assert_eq!(
            f.rpc("runConfig.flashProposal", selection())
                .error
                .unwrap()
                .code,
            JsonRpcErrorCode::SerialPortBusy
        );
        assert!(
            f.core
                .serial_usage
                .acquire(f.port.to_str().unwrap(), "test", "serial free", None)
                .is_ok()
        );
        if method == "run.start" {
            assert!(f.rpc("run.stop", json!({})).error.is_none());
        } else {
            fs::remove_file(f.root.join("query.wait")).unwrap();
        }
        let result = if method == "run.start" {
            rx.recv_timeout(Duration::from_secs(3)).unwrap()(&mut f.core)
        } else {
            responses.recv_timeout(Duration::from_secs(3)).unwrap()
        };
        assert_eq!(result.error.is_some(), method == "run.start", "{result:?}");
        assert!(!f.root.join("flash.ready").exists());
        assert!(
            f.core
                .serial_usage
                .acquire_usb(RESOURCE_KEY, "test", "released", None)
                .is_ok()
        );
    }
}
