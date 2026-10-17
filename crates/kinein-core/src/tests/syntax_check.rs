//! A checagem do compilador ao salvar pelo despacho real (C1, 0.4.0c,
//! `roadmaps/52` §14.3), com um compilador FALSO no padrao do `rsync_falso`:
//! ele grava o argv e acusa um erro quando o arquivo tem "ERRO". Prova a
//! linha vinda da CDB (`-fsyntax-only`, sem `-o`), o erro chegando fundido
//! como `compiler`, o erro corrigido apagado, dois salvamentos seguidos
//! virando uma checagem so', a configuracao desligada e o cabecalho sem
//! unidade.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-syntax-check-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

fn put_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

struct Project {
    root: PathBuf,
    log: PathBuf,
    core: crate::Core,
    events: Receiver<JsonRpcRequest>,
    disk: String,
}

impl Project {
    fn new(name: &str) -> Self {
        let root = temp_dir(name);
        let log = root.join("argv.log");
        let compiler = root.join("bin/g++-falso");
        std::fs::create_dir_all(root.join("bin")).unwrap();
        crate::write_executable(
            &compiler,
            format!(
                "#!/bin/sh\necho \"$*\" >> '{}'\nif grep -q DEMORA src/a.cpp; then sleep 5; fi\n\
                 if grep -q ERRO src/a.cpp; then \
                 echo 'src/a.cpp:2:5: error: erro falso' >&2; exit 1; fi\nexit 0\n",
                log.display()
            ),
        );
        put_file(&root.join("src/a.cpp"), "int main() {}\n");
        put_file(&root.join("inc/b.h"), "#pragma once\n");
        let cdb = json!([{
            "directory": root.display().to_string(),
            "arguments": [compiler.display().to_string(), "-std=c++20", "-Iinc",
                          "-o", "obj/a.o", "-c", "src/a.cpp"],
            "file": "src/a.cpp"
        }]);
        put_file(
            &root.join(".kinein/build/compile_commands.json"),
            &serde_json::to_string(&cdb).unwrap(),
        );
        let (sender, events) = std::sync::mpsc::channel();
        let mut core = super::core_with_empty_search_path(&format!("syntax-{name}"));
        core.enable_lsp(sender);
        let mut project = Self {
            root,
            log,
            core,
            events,
            disk: "int main() {}\n".to_owned(),
        };
        let root = project.root.display().to_string();
        project.call("workspace.open", &json!({ "path": root }));
        // O contexto (com a CDB) carrega no job do indice: salvar antes dele
        // nao acha a unidade.
        let deadline = Instant::now() + Duration::from_secs(10);
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            let event = project
                .events
                .recv_timeout(left)
                .expect("o indice nao terminou");
            if event.method == "event.index.finished" {
                break;
            }
        }
        // Sem confianca, nada roda: a CDB pode ter vindo no repositorio.
        project.save("src/a.cpp", "int main() {}\n");
        std::thread::sleep(Duration::from_millis(700));
        assert!(
            project.runs().is_empty(),
            "projeto nao confiavel: nada roda"
        );
        project.call("workspace.trust", &json!({}));
        project
    }

    fn call(&mut self, method: &str, params: &Value) -> Value {
        let outcome =
            self.core
                .handle_request(&JsonRpcRequest::new(1_i64, method, Some(params.clone())));
        let response = outcome.response();
        assert!(response.error.is_none(), "{method}: {:?}", response.error);
        response.result.clone().unwrap_or(Value::Null)
    }

    fn save(&mut self, relative: &str, content: &str) {
        let path = self.root.join(relative).display().to_string();
        let expected = std::fs::read_to_string(self.root.join(relative)).unwrap();
        self.call(
            "fs.write",
            &json!({ "path": path, "content": content, "expectedContent": expected }),
        );
        if relative == "src/a.cpp" {
            self.disk = content.to_owned();
        }
    }

    /// Os diagnosticos do COMPILADOR de `a.cpp` no primeiro evento fundido
    /// que satisfaz `wanted`, ou `None` (sem servidor, o core tambem publica
    /// ali um diagnostico de fonte `lsp`).
    fn compiler_diagnostics(
        &self,
        within: Duration,
        wanted: impl Fn(&[Value]) -> bool,
    ) -> Option<Vec<Value>> {
        let source = self.root.join("src/a.cpp").display().to_string();
        let deadline = Instant::now() + within;
        while let Some(left) = deadline.checked_duration_since(Instant::now()) {
            let Ok(event) = self.events.recv_timeout(left) else {
                break;
            };
            let params = event.params.unwrap_or_default();
            if event.method != "event.lsp.diagnostics" || params["path"] != source.as_str() {
                continue;
            }
            let compiler: Vec<Value> = params["diagnostics"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|d| d["source"] == "compiler")
                .cloned()
                .collect();
            if wanted(&compiler) {
                return Some(compiler);
            }
        }
        None
    }

    fn runs(&self) -> Vec<String> {
        std::fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

#[test]
fn a_saved_file_is_checked_by_the_cdb_compiler_and_the_fixed_error_goes_away() {
    let _serial = crate::serialize_executables();
    let mut p = Project::new("ciclo");

    p.save("src/a.cpp", "int main() {\n    ERRO;\n}\n");
    let found = p
        .compiler_diagnostics(Duration::from_secs(10), |d| !d.is_empty())
        .expect("o erro chega");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["message"], "erro falso");
    assert_eq!(
        (found[0]["line"].clone(), found[0]["column"].clone()),
        (json!(2), json!(5))
    );
    let runs = p.runs();
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert!(
        runs[0].contains("-fsyntax-only") && runs[0].contains("-Iinc") && !runs[0].contains("-o "),
        "a linha da CDB, sem gravar: {}",
        runs[0]
    );

    p.save("src/a.cpp", "int main() {}\n");
    p.compiler_diagnostics(Duration::from_secs(10), <[Value]>::is_empty)
        .expect("corrigido, o erro some");
}

#[test]
fn two_quick_saves_check_once_and_headers_and_the_setting_are_respected() {
    let _serial = crate::serialize_executables();
    let mut p = Project::new("debounce");

    p.save("src/a.cpp", "int main() {\n    ERRO;\n}\n");
    p.save("src/a.cpp", "int main() {\n    ERRO; // de novo\n}\n");
    p.compiler_diagnostics(Duration::from_secs(10), |d| !d.is_empty())
        .expect("a checagem do segundo");
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(
        p.runs().len(),
        1,
        "o segundo cancelou o primeiro: {:?}",
        p.runs()
    );

    p.save("inc/b.h", "#pragma once\nint x;\n");
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(p.runs().len(), 1, "cabecalho sem unidade: so' o LSP");

    p.call(
        "settings.set",
        &json!({ "scope": "workspace", "values": { "compilerCheckOnSave": false } }),
    );
    p.save("src/a.cpp", "int main() {}\n");
    std::thread::sleep(Duration::from_millis(900));
    assert_eq!(p.runs().len(), 1, "desligada, nada roda");
    assert!(p.disk.ends_with("{}\n"));
}

/// O prazo (1 s nos testes, 10 s no produto): o compilador que nao volta faz
/// o job FALHAR, dizendo o porque; o erro de antes fica (nunca "sem
/// problemas" por engano).
#[test]
fn a_compiler_past_the_deadline_fails_the_job_and_keeps_the_last_result() {
    let _serial = crate::serialize_executables();
    let mut p = Project::new("prazo");
    p.save("src/a.cpp", "int main() {\n    ERRO;\n}\n");
    p.compiler_diagnostics(Duration::from_secs(10), |d| !d.is_empty())
        .expect("o erro chega");

    p.save("src/a.cpp", "int main() {\n    ERRO; DEMORA;\n}\n");
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut output = Vec::new();
    let mut status = Value::Null;
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let Ok(event) = p.events.recv_timeout(left) else {
            break;
        };
        let params = event.params.unwrap_or_default();
        match event.method.as_str() {
            "event.job.output" => {
                output.push(params["line"].as_str().unwrap_or_default().to_owned());
            }
            // O job do primeiro salvamento termina como `warning` (achou o
            // erro); o que importa e' o do segundo.
            "event.job.finished" if params["status"] == "failed" => {
                status = params["status"].clone();
                break;
            }
            "event.lsp.diagnostics" => {
                let compiler = params["diagnostics"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|d| d["source"] == "compiler")
                    .count();
                assert_eq!(compiler, 1, "o prazo nao apaga o erro de antes: {params}");
            }
            _ => {}
        }
    }
    assert_eq!(status, "failed", "{output:?}");
    assert!(
        output.iter().any(|l| l.contains("passou de 1 s")),
        "{output:?}"
    );
}
