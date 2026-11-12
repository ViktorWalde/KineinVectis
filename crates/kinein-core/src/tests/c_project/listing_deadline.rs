//! C1-43 (consolidação C §24): a listagem dos casos com prazo e o cancel,
//! com `CMake`, `ctest` e GCC reais.
//!
//! Até o 0.217.0 a listagem rodava o binário com `Command::output()`, sem
//! prazo e sem o cancel: um servidor posto como teste, que ignora
//! `--gtest_list_tests`, prendia o `test.discover` para sempre, e cancelar
//! não o parava. Aqui o servidor grava o pid a cada chamada e fica rodando;
//! o outro binário lista um caso como o gtest.

use super::Project;
use super::configurations::{build, configure};
use super::environment::project;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

const SERVER: &str = "#include <stdio.h>\n\
#include <stdlib.h>\n\
#include <unistd.h>\n\
int main(void) {\n\
    FILE *f = fopen(getenv(\"KV_PIDS\"), \"a\");\n\
    if (f) { fprintf(f, \"%d\\n\", (int)getpid()); fclose(f); }\n\
    for (;;) sleep(1);\n\
}\n";

const FAKE: &str = "#include <stdio.h>\n\
#include <string.h>\n\
int main(int argc, char **argv) {\n\
    if (argc > 1 && strcmp(argv[1], \"--gtest_list_tests\") == 0) {\n\
        puts(\"Math.\"); puts(\"  Adds\"); return 0;\n\
    }\n\
    return 0;\n\
}\n";

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"dev","generator":"Ninja","binaryDir":"${sourceDir}/build/dev",
   "cacheVariables":{"CMAKE_C_COMPILER":"gcc"}}]}"#;

/// The lines of the job until `finished`, the ones of the Tests tab (C1-45),
/// and its params.
fn until(p: &mut Project, job: &Value, finished: &str) -> (Vec<String>, Vec<String>, Value) {
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut lines = Vec::new();
    let mut tab = Vec::new();
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .expect("job deadline");
        let event = p.events.recv_timeout(left).unwrap();
        p.core.observe_notification(&event);
        let params = event.params.unwrap_or_default();
        if params["jobId"] != *job {
            continue;
        }
        match event.method.as_str() {
            "event.job.output" => lines.push(params["line"].as_str().unwrap().to_owned()),
            "event.test.output" => tab.push(params["line"].as_str().unwrap().to_owned()),
            method if method == finished => return (lines, tab, params),
            _ => {}
        }
    }
}

fn pids(file: &Path) -> Vec<String> {
    fs::read_to_string(file)
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// The listing asked the server again: a new pid is in the file.
fn asked(file: &Path, before: usize, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while pids(file).len() == before {
        assert!(
            Instant::now() < deadline,
            "o servidor não foi listado {what}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The process ended within 5 s; the test kills it either way.
fn gone(pid: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while crate::process_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    let survived = crate::process_alive(pid);
    crate::kill_process(pid);
    !survived
}

fn the_server_answered_none(lines: &[String]) {
    for call in ["--gtest_list_tests", "--libidentify"] {
        let said = format!("não respondeu a `{call}` em 5 s");
        assert!(
            lines.iter().any(|line| line.contains(&said)
                && (line.contains("/server`") || line.contains("/server.exe`"))),
            "{call}: {lines:?}"
        );
    }
}

#[test]
fn c1_43_a_binary_that_never_lists_stops_at_the_deadline_and_the_cancel_stops_the_listing() {
    let mut p = project("listing-deadline");
    // An ASCII folder: `fopen` of MinGW does not take the UTF-8 of `ação`.
    let file = std::env::temp_dir().join(format!("kinein-listing-pids-{}", std::process::id()));
    let _ = fs::remove_file(&file);
    let lists = format!(
        "cmake_minimum_required(VERSION 3.24)\n\
         project(listing LANGUAGES C)\n\
         enable_testing()\n\
         add_executable(server src/server.c)\n\
         add_test(NAME server COMMAND server)\n\
         set_tests_properties(server PROPERTIES ENVIRONMENT \"KV_PIDS={}\")\n\
         add_executable(fake src/fake.c)\n\
         add_test(NAME fake COMMAND fake)\n",
        file.display().to_string().replace('\\', "/")
    );
    fs::create_dir_all(p.root.join("src")).unwrap();
    for (name, text) in [
        ("src/server.c", SERVER),
        ("src/fake.c", FAKE),
        ("CMakeLists.txt", lists.as_str()),
        ("CMakePresets.json", PRESETS),
    ] {
        fs::write(p.root.join(name), text).unwrap();
    }
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    let (_, lines, success) = configure(&mut p, "dev");
    assert!(success, "{lines:?}");
    build(&mut p);

    // Each call of the listing of the server has its 5 s; the other binary
    // keeps its case.
    let started = Instant::now();
    let id = p.call("test.discover", json!({}))["jobId"].clone();
    let (lines, tab, discovered) = until(&mut p, &id, "event.test.discovered");
    let elapsed = started.elapsed();
    assert_eq!(discovered["success"], true, "{discovered}");
    let ids: Vec<_> = discovered["tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(ids, ["server", "fake", "fake::Math.Adds"], "{lines:?}");
    the_server_answered_none(&lines);
    // C1-45: the Tests tab has the same lines, the reasons among them, without
    // the `$` of the command and the count of the end.
    the_server_answered_none(&tab);
    assert_eq!(tab[..], lines[1..lines.len() - 1], "{lines:?}");
    assert!(elapsed >= Duration::from_secs(10), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(30), "{elapsed:?}");
    let listed = pids(&file);
    assert_eq!(listed.len(), 2, "{listed:?}");
    for pid in &listed {
        assert!(gone(pid), "o servidor {pid} sobreviveu ao prazo");
    }

    // The case of the server is looked for by the same listing, and the
    // reason of the refusal is the deadline.
    let id = p.call("test.run", json!({"testId":"server::Math.Adds"}))["jobId"].clone();
    let (lines, _, finished) = until(&mut p, &id, "event.test.finished");
    assert_eq!(finished["success"], false, "{finished}");
    let error = finished["error"].as_str().unwrap();
    assert!(
        error.contains("não respondeu a `--libidentify` em 5 s"),
        "{error}"
    );
    the_server_answered_none(&lines);
    for pid in pids(&file).iter().skip(listed.len()) {
        assert!(gone(pid), "o servidor {pid} sobreviveu ao prazo do run");
    }

    // The cancel while the run looks for the case says so.
    let before = pids(&file).len();
    let id = p.call("test.run", json!({"testId":"server::Math.Adds"}))["jobId"].clone();
    asked(&file, before, "no run");
    assert_eq!(p.call("job.cancel", json!({"jobId":id}))["cancelled"], true);
    let (_, _, finished) = until(&mut p, &id, "event.test.finished");
    assert_eq!(finished["error"], "listagem cancelada", "{finished}");
    for pid in pids(&file).iter().skip(before) {
        assert!(gone(pid), "o servidor {pid} sobreviveu ao cancel do run");
    }

    // The cancel in the middle of the listing gives no tree.
    let before = pids(&file).len();
    let id = p.call("test.discover", json!({}))["jobId"].clone();
    asked(&file, before, "no discover");
    let cancelled = Instant::now();
    assert_eq!(p.call("job.cancel", json!({"jobId":id}))["cancelled"], true);
    let (_, _, discovered) = until(&mut p, &id, "event.test.discovered");
    assert!(
        cancelled.elapsed() < Duration::from_secs(3),
        "{:?}",
        cancelled.elapsed()
    );
    assert_eq!(discovered["success"], false, "{discovered}");
    assert_eq!(discovered["error"], "listagem cancelada", "{discovered}");
    assert_eq!(discovered.get("tests"), None, "{discovered}");
    let (_, _, finished) = until(&mut p, &id, "event.job.finished");
    assert_eq!(finished["status"], "cancelled", "{finished}");
    for pid in pids(&file).iter().skip(before) {
        assert!(gone(pid), "o servidor {pid} sobreviveu ao cancel");
    }
    let _ = fs::remove_file(&file);
}
