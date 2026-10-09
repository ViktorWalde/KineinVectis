//! A ponte contra o adaptador FALSO (`scripts/fake_driver_adapter.py`): cada
//! modo reproduz um risco do 39 §6.1, e o teste olha o efeito (desfecho, slot,
//! coleta), nao uma resposta montada.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::*;
use crate::owned_child::Ending;

fn adapter(mode: &str, extra: Option<&Path>) -> Command {
    let script = crate::platform::canonicalize(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/fake_driver_adapter.py"),
    )
    .expect("scripts/fake_driver_adapter.py");
    // No Windows o interpretador e' `python` (60 §3.1, D6).
    let mut command = Command::new(if cfg!(windows) { "python" } else { "python3" });
    command.arg(script).arg(mode);
    if let Some(path) = extra {
        command.arg(path);
    }
    command
}

fn start(mode: &str) -> DriverProcess {
    DriverProcess::start(adapter(mode, None), &[], "native.fake", "postgres").expect("handshake")
}

fn context(operation: &str) -> Value {
    json!({ "instanceId": "i", "sessionId": "s", "generation": 1, "operationId": operation })
}

fn query(driver: &DriverProcess, operation: &str) -> Pending {
    driver
        .send(
            "driver.query",
            json!({ "context": context(operation), "text": "select 1", "maxRows": 10 }),
            operation,
            Kind::Write,
        )
        .expect("send")
}

fn next(pending: &Pending) -> Event {
    pending
        .events
        .recv_timeout(Duration::from_secs(5))
        .expect("evento no prazo")
}

#[test]
fn handshake_negotiates_streams_and_shutdown_collects() {
    let driver = start("normal");
    assert_eq!(driver.negotiated().limits.in_flight, 2);
    assert_eq!(driver.negotiated().limits.message_bytes, 4096);
    let pending = query(&driver, "q1");
    assert!(matches!(next(&pending), Event::Chunk(chunk) if chunk.get().contains("\"rows\"")));
    assert!(matches!(next(&pending), Event::Terminal(_)));
    let (confirmed, closed) = driver.shutdown(&context("shutdown"));
    assert!(confirmed);
    assert_eq!(closed.ending, Ending::Graceful);
    assert!(closed.collected(), "{closed:?}");
}

#[test]
fn a_refused_or_silent_handshake_collects_only_that_instance() {
    let refused = DriverProcess::start(adapter("normal", None), &[], "outro.adaptador", "postgres");
    assert!(matches!(
        refused,
        Err(StartFailure::Refused(HandshakeFailure::InvalidMetadata))
    ));
    let begin = Instant::now();
    let silent = DriverProcess::start(adapter("slow_init", None), &[], "native.fake", "postgres");
    assert!(matches!(silent, Err(StartFailure::Timeout)));
    // 5 s de prazo do initialize e o encerramento com TERM: bem antes dos 10 s.
    assert!(
        begin.elapsed() < Duration::from_secs(9),
        "{:?}",
        begin.elapsed()
    );
}

#[test]
fn a_line_over_the_negotiated_limit_kills_the_transport_with_unknown_outcome() {
    let driver = start("big_line");
    let pending = query(&driver, "q1");
    assert!(matches!(
        next(&pending),
        Event::Lost(OperationOutcome::Unknown)
    ));
    assert_eq!(
        driver
            .send(
                "driver.test",
                json!({ "context": context("t1") }),
                "t1",
                Kind::Read
            )
            .err(),
        Some(SendRefusal::Closed)
    );
    assert!(driver.shutdown(&context("shutdown")).1.collected());
}

#[test]
fn a_crash_after_a_write_is_unknown_and_never_retried() {
    let driver = start("crash_after_write");
    let pending = query(&driver, "q1");
    assert!(matches!(
        next(&pending),
        Event::Lost(OperationOutcome::Unknown)
    ));
    // O mesmo canal nao recebe mais nada: a ponte nao reenvia a escrita.
    assert!(
        pending
            .events
            .recv_timeout(Duration::from_millis(300))
            .is_err()
    );
    let (confirmed, closed) = driver.shutdown(&context("shutdown"));
    assert!(!confirmed);
    assert!(closed.collected(), "{closed:?}");
}

#[test]
fn a_pending_query_leaves_the_control_slot_for_its_cancellation() {
    let driver = Arc::new(start("late"));
    let slow = query(&driver, "q1");
    // O unico slot comum esta' ocupado: um segundo pedido comum espera...
    let waiting = {
        let driver = Arc::clone(&driver);
        thread::spawn(move || {
            driver
                .send(
                    "driver.test",
                    json!({ "context": context("t1") }),
                    "t1",
                    Kind::Read,
                )
                .map(|pending| next(&pending))
        })
    };
    thread::sleep(Duration::from_millis(50));
    assert!(
        !waiting.is_finished(),
        "o pedido comum nao devia ter passado"
    );
    // ...mas o cancelamento sai na hora, pelo slot reservado. (O adaptador
    // falso responde em ordem; a propriedade do core e' o ENVIO sem espera.)
    let begin = Instant::now();
    let cancel = driver
        .send(
            "driver.cancel",
            json!({ "context": context("c1"), "target": "q1" }),
            "c1",
            Kind::Control,
        )
        .unwrap();
    assert!(
        begin.elapsed() < Duration::from_millis(200),
        "o cancelamento esperou slot"
    );
    assert!(!waiting.is_finished(), "o pedido comum continua esperando");
    assert!(matches!(next(&slow), Event::Chunk(_)));
    assert!(matches!(next(&slow), Event::Terminal(_)));
    assert!(matches!(next(&cancel), Event::Terminal(_)));
    assert!(matches!(waiting.join().unwrap(), Ok(Event::Terminal(_))));
    let driver = Arc::into_inner(driver).unwrap();
    assert!(driver.shutdown(&context("shutdown")).1.collected());
}

#[test]
fn the_local_queue_refuses_beyond_its_capacity() {
    let driver = Arc::new(start("late"));
    let held = query(&driver, "q0");
    let waiters: Vec<_> = (0..QUEUE_CAPACITY)
        .map(|index| {
            let driver = Arc::clone(&driver);
            thread::spawn(move || {
                let operation = format!("t{index}");
                driver
                    .send(
                        "driver.test",
                        json!({ "context": context(&operation) }),
                        &operation,
                        Kind::Read,
                    )
                    .map(|pending| next(&pending))
            })
        })
        .collect();
    let deadline = Instant::now() + Duration::from_secs(5);
    while driver.shared.lock().waiting < QUEUE_CAPACITY && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        driver
            .send(
                "driver.test",
                json!({ "context": context("demais") }),
                "demais",
                Kind::Read
            )
            .err(),
        Some(SendRefusal::Busy)
    );
    drop(held);
    for waiter in waiters {
        assert!(matches!(waiter.join().unwrap(), Ok(Event::Terminal(_))));
    }
    let driver = Arc::into_inner(driver).unwrap();
    assert!(driver.shutdown(&context("shutdown")).1.collected());
}

#[test]
fn a_chunk_for_an_operation_nobody_asked_is_a_protocol_violation() {
    let driver = start("stray_chunk");
    let pending = query(&driver, "q1");
    assert!(matches!(
        next(&pending),
        Event::Lost(OperationOutcome::Unknown)
    ));
    assert!(driver.shutdown(&context("shutdown")).1.collected());
}

#[test]
fn a_noisy_stderr_does_not_block_and_stays_bounded() {
    let driver = start("noisy");
    let pending = query(&driver, "q1");
    assert!(matches!(next(&pending), Event::Chunk(_)));
    assert!(matches!(next(&pending), Event::Terminal(_)));
    let tail = driver.child.lock().unwrap().stderr_tail();
    assert!(
        tail.lines().count() <= crate::stderr_tail::DEFAULT_CAPACITY,
        "{}",
        tail.lines().count()
    );
    assert!(driver.shutdown(&context("shutdown")).1.collected());
}

#[test]
fn an_adapter_that_ignores_shutdown_is_killed_and_collected() {
    let driver = start("ignore_shutdown");
    let begin = Instant::now();
    let (confirmed, closed) = driver.shutdown(&context("shutdown"));
    assert!(!confirmed);
    assert_eq!(closed.ending, Ending::Killed);
    assert!(closed.collected(), "{closed:?}");
    assert!(
        begin.elapsed() < Duration::from_secs(13),
        "{:?}",
        begin.elapsed()
    );
}

#[test]
fn a_helper_in_the_adapter_group_does_not_outlive_the_shutdown() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("kinein-driver-helper-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    let pid_file = dir.join("helper.pid");
    let driver = DriverProcess::start(
        adapter("helper", Some(&pid_file)),
        &[],
        "native.fake",
        "postgres",
    )
    .expect("handshake");
    let pid = std::fs::read_to_string(&pid_file).unwrap();
    let proc_dir = PathBuf::from(format!("/proc/{}", pid.trim()));
    assert!(proc_dir.exists(), "o auxiliar devia estar vivo");
    let (confirmed, closed) = driver.shutdown(&context("shutdown"));
    assert!(confirmed && closed.collected(), "{closed:?}");
    let deadline = Instant::now() + Duration::from_secs(2);
    while proc_dir.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!proc_dir.exists(), "o auxiliar sobreviveu ao encerramento");
}
