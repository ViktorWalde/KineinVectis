//! Automatic selection excludes device leases in memory and across cores.
use super::*;

#[test]
fn automatic_selection_excludes_every_port_and_device_owners_exclude_it() {
    for persistent in [false, true] {
        let (root, port) = fixture(&format!("automatic-matrix-{persistent}"));
        let second = root.join("other-port");
        fs::write(&second, "another regular fixture").unwrap();
        let usage = if persistent {
            registry(&root.join("locks"))
        } else {
            Registry::default()
        };
        let auto = usage
            .acquire_automatic("run.start", "MicroPython", None)
            .unwrap();
        for port in [&port, &second] {
            let denied = usage
                .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
                .unwrap_err();
            let details = denied.details.unwrap();
            assert_eq!(details["scope"], "device");
            assert_eq!(details["owner"]["automatic"], true);
            assert!(denied.message.contains("seleção automática"));
            assert!(denied.message.contains("Pare a execução"));
        }
        let denied = usage
            .acquire_automatic("run.script", "Other", None)
            .unwrap_err();
        assert_eq!(denied.details.unwrap()["device"], serde_json::Value::Null);
        drop(auto);
        let first = usage
            .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .unwrap();
        let other = usage
            .acquire(
                second.to_str().unwrap(),
                "serial.identify",
                "Identify",
                None,
            )
            .unwrap();
        assert_eq!(
            usage
                .acquire_automatic("run.start", "MicroPython", None)
                .unwrap_err()
                .code,
            JsonRpcErrorCode::SerialPortBusy
        );
        drop(first);
        assert!(
            usage
                .acquire_automatic("run.start", "MicroPython", None)
                .is_err()
        );
        drop(other);
        assert!(
            usage
                .acquire_automatic("run.start", "MicroPython", None)
                .is_ok()
        );
    }
}

#[test]
fn independent_registries_coordinate_automatic_launch_and_ignore_old_owner_data() {
    let (root, port) = fixture("automatic-independent");
    let first = registry(&root.join("locks"));
    let other = registry(&root.join("locks"));
    let auto = first
        .acquire_automatic("run.script", "MicroPython", None)
        .unwrap();
    assert!(other.acquire_automatic("run.start", "Other", None).is_err());
    let denied = other
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap_err();
    assert_eq!(denied.details.unwrap()["owner"]["automatic"], true);
    drop(auto);
    let device = other
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap();
    let denied = first
        .acquire_automatic("run.start", "MicroPython", None)
        .unwrap_err();
    // Shared device launch gates do not publish a unique owner. Never reuse
    // the previous automatic owner's metadata as evidence of a shared holder.
    assert_eq!(denied.details.unwrap()["owner"], serde_json::Value::Null);
    drop(device);
    assert!(
        first
            .acquire_automatic("run.start", "MicroPython", None)
            .is_ok()
    );
}

#[test]
fn automatic_execution_lock_survives_the_launch_guard_and_releases_after_tool_exit() {
    let (root, port) = fixture("automatic-tool-lifetime");
    let usage = registry(&root.join("locks"));
    let lease = usage
        .acquire_automatic("run.start", "MicroPython", None)
        .unwrap();
    let (_, arguments) = lease.invocation(&fake_tool(&root), &[]).unwrap();
    let child = worker_command(&root)
        .env(
            "KINEIN_SERIAL_EXEC_TEST_ARGS",
            serde_json::to_string(&arguments[1..]).unwrap(),
        )
        .spawn()
        .unwrap();
    let mut worker = Worker(child);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !root.join("tool-ready").exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        root.join("tool-ready").exists(),
        "automatic helper did not execute the tool"
    );
    drop(lease);
    let denied = usage
        .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
        .unwrap_err();
    let owner = &denied.details.unwrap()["owner"];
    assert_eq!(owner["processId"], worker.0.id());
    assert_eq!(owner["automatic"], true);
    assert!(
        usage
            .acquire_automatic("run.script", "Other", None)
            .is_err()
    );
    worker.0.kill().unwrap();
    worker.0.wait().unwrap();
    assert!(
        usage
            .acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
            .is_ok()
    );
    assert!(
        usage
            .acquire_automatic("run.start", "MicroPython", None)
            .is_ok()
    );
}

#[test]
fn a_device_launch_and_automatic_launch_cannot_both_win_the_kernel_race() {
    let (root, port) = fixture("automatic-launch-race");
    for _ in 0..8 {
        let barrier = Arc::new(Barrier::new(2));
        let directory = root.join("locks");
        let mut threads = Vec::new();
        for automatic in [false, true] {
            let barrier = Arc::clone(&barrier);
            let port = port.clone();
            let usage = registry(&directory);
            threads.push(thread::spawn(move || {
                barrier.wait();
                let result = if automatic {
                    usage.acquire_automatic("run.start", "MicroPython", None)
                } else {
                    usage.acquire(port.to_str().unwrap(), "serial.monitor", "Monitor", None)
                };
                barrier.wait();
                result
            }));
        }
        let results: Vec<_> = threads
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(
            results.iter().filter(|result| result.is_ok()).count(),
            1,
            "device and automatic launches both acquired the channel"
        );
    }
}
