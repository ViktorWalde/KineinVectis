//! A base prova o que diz: ambiente, os tres finais, descendente e leitor.

use std::{
    io::Read,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

use super::*;

fn dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-owned-child")
        .join(format!("{}-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn script(dir: &std::path::Path, body: &str) -> Command {
    let path = dir.join("filho.sh");
    crate::write_executable(&path, format!("#!/bin/sh\n{body}\n"));
    Command::new(path)
}

/// Captura o stdout inteiro, como o dono de um protocolo faria.
fn capture() -> (
    Arc<Mutex<String>>,
    impl FnOnce(ChildStdout) + Send + 'static,
) {
    let out = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&out);
    (out, move |mut stdout: ChildStdout| {
        let mut text = String::new();
        drop(stdout.read_to_string(&mut text));
        *sink.lock().unwrap() = text;
    })
}

fn alive(pid: &str) -> bool {
    std::path::Path::new(&format!("/proc/{}", pid.trim())).exists()
}

#[test]
fn only_the_allowed_and_explicit_variables_reach_the_process() {
    let dir = dir("env");
    // Uma variavel de banco do processo da IDE nao pode vazar para o filho.
    // SAFETY nao se aplica: o teste nao usa set_var; o valor vem do comando.
    let mut command = script(&dir, "env | sort");
    command.env("PGPASSWORD", "NAO_PODE_VAZAR");
    let (out, reader) = capture();
    let child = OwnedChild::spawn(command, &[("KINEIN_INSTANCIA", "a1")], reader).unwrap();
    let closed = child.close(Duration::from_secs(2));
    assert!(closed.collected(), "{closed:?}");
    let env = out.lock().unwrap().clone();
    assert!(env.contains("KINEIN_INSTANCIA=a1"), "{env}");
    assert!(!env.contains("PGPASSWORD"), "{env}");
    for line in env.lines() {
        let name = line.split('=').next().unwrap_or_default();
        assert!(
            ALLOWED_ENV.contains(&name)
                || name == "KINEIN_INSTANCIA"
                || name == "PWD"
                || name == "SHLVL"
                || name == "_",
            "variavel nao permitida: {line}"
        );
    }
}

#[test]
fn eof_on_stdin_ends_a_cooperative_process_gracefully() {
    let dir = dir("eof");
    let (out, reader) = capture();
    let child = OwnedChild::spawn(script(&dir, "cat; echo fim"), &[], reader).unwrap();
    let closed = child.close(Duration::from_secs(2));
    assert_eq!(closed.ending, Ending::Graceful);
    assert!(closed.collected());
    assert_eq!(out.lock().unwrap().as_str(), "fim\n");
}

#[test]
fn a_process_that_ignores_eof_gets_term_and_then_kill() {
    let dir = dir("term");
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(script(&dir, "sleep 60"), &[], reader).unwrap();
    let closed = child.close(Duration::from_millis(100));
    assert_eq!(closed.ending, Ending::Terminated);
    assert!(closed.collected(), "{closed:?}");

    // Ignorar o TERM e' herdado pelo `sleep`: so' o KILL resolve.
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(script(&dir, "trap '' TERM; sleep 60"), &[], reader).unwrap();
    let closed = child.close(Duration::from_millis(100));
    assert_eq!(closed.ending, Ending::Killed);
    assert!(closed.collected(), "{closed:?}");
}

#[test]
fn a_descendant_in_the_group_does_not_survive_the_close() {
    let dir = dir("descendente");
    let pid_file = dir.join("auxiliar.pid");
    // O auxiliar herda o stdout: sem matar o grupo, o leitor nunca veria EOF.
    let body = format!(
        "sleep 60 &\necho $! > '{}'\ncat > /dev/null",
        pid_file.display()
    );
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(script(&dir, &body), &[], reader).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !pid_file.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    let helper = std::fs::read_to_string(&pid_file).unwrap();
    assert!(
        alive(&helper),
        "o auxiliar devia estar vivo antes do encerramento"
    );
    let closed = child.close(Duration::from_secs(2));
    assert_eq!(closed.ending, Ending::Graceful);
    assert!(
        closed.reader_joined,
        "o leitor ficou preso no stdout do auxiliar"
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while alive(&helper) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!alive(&helper), "o auxiliar sobreviveu ao encerramento");
}

#[test]
fn the_stderr_tail_is_kept_for_the_owner_to_decide() {
    let dir = dir("stderr");
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(
        script(&dir, "echo 'motivo da falha' >&2; cat > /dev/null"),
        &[],
        reader,
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !child.stderr_tail().contains("motivo") && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(child.stderr_tail().contains("motivo da falha"));
    assert!(child.close(Duration::from_secs(2)).collected());
}
