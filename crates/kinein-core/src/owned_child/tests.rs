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
    Command::new(crate::write_executable(
        dir.join("filho.sh"),
        format!("#!/bin/sh\n{body}\n"),
    ))
}

/// No Windows, o `sh` do Git e' do MSYS: ele acrescenta variaveis ao ambiente
/// e fala pid do MSYS, nao do Windows. O que mede ambiente e pid e' Python.
#[cfg(windows)]
fn python(dir: &std::path::Path, body: &str) -> Command {
    Command::new(crate::write_executable(
        dir.join("filho.py"),
        format!("#!/usr/bin/env python3\n{body}\n"),
    ))
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

#[cfg(unix)]
fn alive(pid: &str) -> bool {
    std::path::Path::new(&format!("/proc/{}", pid.trim())).exists()
}

#[cfg(windows)]
fn alive(pid: &str) -> bool {
    let filter = format!("PID eq {}", pid.trim());
    let output = Command::new("tasklist")
        .args(["/FI", &filter, "/NH", "/FO", "CSV"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{}\"", pid.trim()))
}

#[test]
fn only_the_allowed_and_explicit_variables_reach_the_process() {
    let dir = dir("env");
    // Uma variavel de banco do processo da IDE nao pode vazar para o filho.
    // SAFETY nao se aplica: o teste nao usa set_var; o valor vem do comando.
    #[cfg(unix)]
    let mut command = script(&dir, "env | sort");
    #[cfg(windows)]
    let mut command = python(
        &dir,
        "import os\nfor name in sorted(os.environ):\n    print(name + '=' + os.environ[name])",
    );
    command.env("PGPASSWORD", "NAO_PODE_VAZAR");
    let (out, reader) = capture();
    let child = OwnedChild::spawn(
        command,
        Env::Allowlist(&[("KINEIN_INSTANCIA", "a1")]),
        None,
        reader,
    )
    .unwrap();
    let closed = child.close(Duration::from_secs(2));
    assert!(closed.collected(), "{closed:?}");
    let env = out.lock().unwrap().clone();
    assert!(env.contains("KINEIN_INSTANCIA=a1"), "{env}");
    assert!(!env.contains("PGPASSWORD"), "{env}");
    for line in env.lines() {
        let name = line.split('=').next().unwrap_or_default();
        // O Windows nao distingue maiuscula no nome (o Python devolve tudo em
        // maiusculas); o Unix distingue.
        let same = |allowed: &&str| {
            if cfg!(windows) {
                allowed.eq_ignore_ascii_case(name)
            } else {
                *allowed == name
            }
        };
        assert!(
            ALLOWED_ENV.iter().any(same)
                || name.eq_ignore_ascii_case("KINEIN_INSTANCIA")
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
    let child = OwnedChild::spawn(
        script(&dir, "cat; echo fim"),
        Env::Allowlist(&[]),
        None,
        reader,
    )
    .unwrap();
    let closed = child.close(Duration::from_secs(2));
    assert_eq!(closed.ending, Ending::Graceful);
    assert!(closed.collected());
    assert_eq!(out.lock().unwrap().as_str(), "fim\n");
}

#[test]
#[cfg(unix)]
fn a_process_that_ignores_eof_gets_term_and_then_kill() {
    let dir = dir("term");
    let (_out, reader) = capture();
    let child =
        OwnedChild::spawn(script(&dir, "sleep 60"), Env::Allowlist(&[]), None, reader).unwrap();
    let closed = child.close(Duration::from_millis(100));
    assert_eq!(closed.ending, Ending::Terminated);
    assert!(closed.collected(), "{closed:?}");

    // Ignorar o TERM e' herdado pelo `sleep`: so' o KILL resolve.
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(
        script(&dir, "trap '' TERM; sleep 60"),
        Env::Allowlist(&[]),
        None,
        reader,
    )
    .unwrap();
    let closed = child.close(Duration::from_millis(100));
    assert_eq!(closed.ending, Ending::Killed);
    assert!(closed.collected(), "{closed:?}");
}

/// O Windows nao tem pedido gentil ao grupo (o `platform::terminate_group`
/// devolve `false`): quem ignora o EOF vai direto para o fim do Job Object.
#[test]
#[cfg(windows)]
fn a_process_that_ignores_eof_is_killed_by_the_job() {
    let dir = dir("term");
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(
        python(&dir, "import time\ntime.sleep(60)"),
        Env::Allowlist(&[]),
        None,
        reader,
    )
    .unwrap();
    let closed = child.close(Duration::from_millis(100));
    assert_eq!(closed.ending, Ending::Killed);
    assert!(closed.collected(), "{closed:?}");
}

#[test]
fn a_descendant_in_the_group_does_not_survive_the_close() {
    let dir = dir("descendente");
    let pid_file = dir.join("auxiliar.pid");
    // O auxiliar herda o stdout: sem matar o grupo, o leitor nunca veria EOF.
    #[cfg(unix)]
    let command = script(
        &dir,
        &format!(
            "sleep 60 &\necho $! > '{}'\ncat > /dev/null",
            pid_file.display()
        ),
    );
    #[cfg(windows)]
    let command = python(
        &dir,
        &format!(
            "import subprocess, sys\n\
             helper = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])\n\
             open(r'{}', 'w').write(str(helper.pid))\n\
             sys.stdin.read()",
            pid_file.display()
        ),
    );
    let (_out, reader) = capture();
    let child = OwnedChild::spawn(command, Env::Allowlist(&[]), None, reader).unwrap();
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
        Env::Allowlist(&[]),
        None,
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
