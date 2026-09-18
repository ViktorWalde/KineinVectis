//! `coverage.run` / `coverage.lines` (D8 do `roadmaps/41`, 2026-09-17) por
//! despacho, com as ferramentas FALSAS escrevendo um LCOV: o Rust pelo
//! `cargo llvm-cov` (o cargo do kit, fixado), o Python pelo `coverage.py` do
//! interpretador do projeto; a recusa do que nao tem ferramenta; e as linhas
//! de um arquivo do relatorio, para a calha do editor.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};

fn executavel(caminho: &Path, corpo: &str) {
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, corpo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

struct Cenario {
    core: crate::Core,
    root: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str) -> Cenario {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-coverage-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("ws/src")).unwrap();
    let base = base.canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ));
    core.enable_lsp(sender);
    Cenario {
        core,
        root: base.join("ws"),
        events: receiver,
    }
}

impl Cenario {
    fn bin(&self) -> PathBuf {
        self.root.parent().unwrap().join("bin")
    }

    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(2_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn abrir(&mut self) {
        let r = self.rpc(
            "workspace.open",
            json!({ "path": self.root.to_str().unwrap() }),
        );
        assert!(r.error.is_none(), "{:?}", r.error);
    }

    fn finished(&self) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < prazo {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == "event.coverage.finished"
            {
                return e.params.unwrap();
            }
        }
        panic!("event.coverage.finished nao chegou");
    }
}

fn lcov(arquivo: &Path) -> String {
    format!(
        "SF:{f}\\nDA:1,2\\nDA:2,0\\nDA:3,1\\nend_of_record\\n",
        f = arquivo.display()
    )
}

/// Rust: sem cargo-llvm-cov, a recusa diz o passo; com o falso, o `cargo` do
/// kit recebe `llvm-cov --lcov --output-path <root>/.kinein/coverage.lcov`, o
/// resumo por arquivo sai no evento e `coverage.lines` da' as linhas.
#[test]
#[cfg(unix)]
fn rust_coverage_runs_cargo_llvm_cov_and_the_lines_reach_the_editor() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("rust");
    std::fs::write(
        c.root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(c.root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    let lib = c.root.join("src/lib.rs");
    // O cargo falso (fixado no kit): escreve o LCOV no --output-path.
    executavel(
        &c.bin().join("cargo"),
        &format!(
            "#!/bin/sh\necho \"cargo-falso $*\"\nwhile [ $# -gt 0 ]; do if [ \"$1\" = --output-path ]; then printf '{l}' > \"$2\"; fi; shift; done\n",
            l = lcov(&lib)
        ),
    );
    c.abrir();
    let fixado = c.rpc("toolchain.set", json!({ "role": "cargo", "id": "cargo" }));
    assert!(fixado.error.is_none(), "{:?}", fixado.error);

    // Sem cargo-llvm-cov: o job falha dizendo o passo.
    assert!(c.rpc("coverage.run", json!({})).error.is_none());
    let fim = c.finished();
    assert_eq!(fim["success"], false);
    assert!(
        fim["error"]
            .as_str()
            .unwrap()
            .contains("cargo install cargo-llvm-cov"),
        "{fim}"
    );

    executavel(&c.bin().join("cargo-llvm-cov"), "#!/bin/sh\nexit 0\n");
    let mut c2 = cenario("rust2");
    std::fs::write(
        c2.root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(c2.root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    let lib = c2.root.join("src/lib.rs");
    executavel(
        &c2.bin().join("cargo"),
        &format!(
            "#!/bin/sh\necho \"cargo-falso $*\"\nwhile [ $# -gt 0 ]; do if [ \"$1\" = --output-path ]; then printf '{l}' > \"$2\"; fi; shift; done\n",
            l = lcov(&lib)
        ),
    );
    executavel(&c2.bin().join("cargo-llvm-cov"), "#!/bin/sh\nexit 0\n");
    c2.abrir();
    assert!(
        c2.rpc("toolchain.set", json!({ "role": "cargo", "id": "cargo" }))
            .error
            .is_none()
    );
    assert!(c2.rpc("coverage.run", json!({})).error.is_none());
    let fim = c2.finished();
    assert_eq!(fim["success"], true, "{fim}");
    assert_eq!(fim["tool"], "cargo llvm-cov");
    assert_eq!(
        fim["path"],
        c2.root.join(".kinein/coverage.lcov").display().to_string()
    );
    let files = fim["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["file"], lib.display().to_string());
    assert_eq!(
        (
            files[0]["linesFound"].as_u64(),
            files[0]["linesHit"].as_u64()
        ),
        (Some(3), Some(2))
    );

    let linhas = c2
        .rpc(
            "coverage.lines",
            json!({ "file": lib.display().to_string() }),
        )
        .result
        .unwrap();
    assert_eq!(linhas["known"], true);
    assert_eq!(linhas["covered"], json!([1, 3]));
    assert_eq!(linhas["missed"], json!([2]));
    let outro = c2
        .rpc("coverage.lines", json!({ "file": "/nao/existe.rs" }))
        .result
        .unwrap();
    assert_eq!(outro["known"], false);
}

/// Python: o `coverage run -m pytest` e o `coverage lcov -o` do interpretador
/// do projeto; sem o modulo, o passo de instalar no ambiente. C/C++: recusa
/// antes do job, dizendo por que.
#[test]
#[cfg(unix)]
fn python_coverage_uses_the_project_interpreter_and_cpp_is_refused() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("python");
    std::fs::write(
        c.root.join("pyproject.toml"),
        "[project]\nname = \"demo\"\n",
    )
    .unwrap();
    std::fs::write(c.root.join("app.py"), "x = 1\n").unwrap();
    let app = c.root.join("app.py");
    // O .venv com um python falso que "roda" o coverage e escreve o LCOV no -o.
    executavel(
        &c.root.join(".venv/bin/python"),
        &format!(
            "#!/bin/sh\necho \"python-falso $*\"\nif [ \"$1 $2 $3\" = \"-m coverage lcov\" ]; then printf '{l}' > \"$5\"; fi\n",
            l = lcov(&app)
        ),
    );
    c.abrir();
    assert!(c.rpc("coverage.run", json!({})).error.is_none());
    let fim = c.finished();
    assert_eq!(fim["success"], true, "{fim}");
    assert_eq!(fim["tool"], "coverage.py");
    assert_eq!(fim["files"][0]["file"], app.display().to_string());

    // O python que nao tem o modulo: falha no `coverage run` -> o passo.
    executavel(
        &c.root.join(".venv/bin/python"),
        "#!/bin/sh\necho 'No module named coverage' >&2\nexit 1\n",
    );
    assert!(c.rpc("coverage.run", json!({})).error.is_none());
    let fim = c.finished();
    assert_eq!(fim["success"], false);
    assert!(
        fim["error"]
            .as_str()
            .unwrap()
            .contains("uv add --dev coverage"),
        "{fim}"
    );

    let mut cpp = cenario("cpp");
    std::fs::write(cpp.root.join("CMakeLists.txt"), "project(x)\n").unwrap();
    cpp.abrir();
    let erro = cpp.rpc("coverage.run", json!({})).error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains("--coverage"), "{}", erro.message);
}
