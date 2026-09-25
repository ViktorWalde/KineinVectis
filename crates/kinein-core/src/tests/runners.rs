//! `test.run` / `quality.run` dispatch — now async cancelable jobs.
//!
//! Both mirror `build.run` (see `tests/build.rs`); here we cover their
//! pre-flight validation and one real async test run.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::JsonRpcRequest;

/// Rust/Cargo (clippy), Python (ruff) e C/C++ (clang-tidy pela CDB, D6)
/// tem linter no `quality.run`; um workspace Maven continua recusado com
/// "tipo nao suportado".
#[test]
fn quality_run_rejects_kinds_without_a_linter() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-quality-unsupported", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Maven: sem linter. (Um CMake tem o clang-tidy pela CDB desde o D6,
    // 2026-09-17 — ver `tests/quality_tidy.rs`.)
    std::fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();
    let mut core = core_with_empty_search_path("quality-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(71_i64, "quality.run", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("maven"), "{}", error.message);
}

/// Rust, `CMake` e (desde a fatia 3 da cadeia Python) Python tem runner de
/// testes; um projeto Maven continua recusado com "tipo nao suportado".
#[test]
fn test_run_rejects_unsupported_kind() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-test-unsupported", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();
    let mut core = core_with_empty_search_path("test-unsupported");

    let opened = core.handle_request(&JsonRpcRequest::new(
        72_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outcome = core.handle_request(&JsonRpcRequest::new(73_i64, "test.run", Some(json!({}))));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("maven"), "{error:?}");
}

#[test]
fn test_run_starts_a_job_and_reports_a_passing_case() {
    use std::time::Duration;

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("test-job");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-test-job", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "#[test]\nfn soma() {\n    assert_eq!(1 + 1, 2);\n}\n",
    )
    .unwrap();

    let opened = core.handle_request(&JsonRpcRequest::new(
        74_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let started = core.handle_request(&JsonRpcRequest::new(75_i64, "test.run", Some(json!({}))));
    let job_id = started.response().result.as_ref().unwrap()["jobId"]
        .as_str()
        .expect("test.run deve retornar jobId")
        .to_owned();

    let mut saw_created = false;
    let mut saw_job_output = false;
    let mut saw_passing_case = false;
    let mut saw_test_finished = false;
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(90))
            .expect("eventos de teste dentro do timeout");
        match event.method.as_str() {
            // Desde 2026-09-12 o workspace.open sobe TAMBEM o job do indice do
            // projeto (roadmaps/42 P0): os eventos de job aqui sao filtrados
            // pelo id do test.run — o do indice nao e' o que se prova.
            "event.job.created" => {
                if event.params.as_ref().unwrap()["id"] == job_id.as_str() {
                    saw_created = true;
                }
            }
            "event.job.output" => {
                let params = event.params.as_ref().unwrap();
                if params["jobId"] == job_id.as_str() && params["line"].is_string() {
                    saw_job_output = true;
                }
            }
            "event.test.case" => {
                let params = event.params.as_ref().unwrap();
                if params["name"] == "soma" && params["status"] == "passed" {
                    saw_passing_case = true;
                }
            }
            "event.test.finished" => {
                let params = event.params.as_ref().unwrap();
                assert_eq!(params["jobId"], job_id.as_str());
                assert_eq!(params["success"], true);
                assert_eq!(params["passed"], 1);
                saw_test_finished = true;
            }
            "event.job.finished" => {
                let params = event.params.as_ref().unwrap();
                if params["jobId"] != job_id.as_str() {
                    continue;
                }
                assert_eq!(params["status"], "success");
                break;
            }
            _ => {}
        }
    }
    assert!(saw_created, "faltou event.job.created");
    assert!(saw_job_output, "faltou event.job.output");
    assert!(saw_passing_case, "faltou event.test.case passed");
    assert!(saw_test_finished, "faltou event.test.finished");
}

/// Workspace Python (pyproject + app.py com um import solto) e um core com
/// jobs cuja busca de ferramentas e' SO' a pasta `bin` do workspace.
fn python_quality_fixture(
    name: &str,
) -> (
    std::path::PathBuf,
    crate::Core,
    std::sync::mpsc::Receiver<JsonRpcRequest>,
) {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-quality-python-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(dir.join("app.py"), "import os\nx=1\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        80_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    (dir, core, receiver)
}

/// Pede `quality.run` e devolve o `jobId` e, do fluxo de eventos, o
/// `event.quality.finished` desse job com todos os `event.quality.diagnostic`
/// que vieram antes dele.
fn run_quality_and_collect(
    core: &mut crate::Core,
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
) -> (Vec<serde_json::Value>, serde_json::Value) {
    use std::time::{Duration, Instant};

    let started = core.handle_request(&JsonRpcRequest::new(81_i64, "quality.run", Some(json!({}))));
    let job_id = started.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut diagnosticos = Vec::new();
    let limite = Instant::now() + Duration::from_secs(20);
    loop {
        let e = receiver
            .recv_timeout(limite.saturating_duration_since(Instant::now()))
            .expect("evento");
        let do_job = e
            .params
            .as_ref()
            .is_some_and(|p| p["jobId"] == job_id.as_str());
        match e.method.as_str() {
            "event.quality.diagnostic" if do_job => diagnosticos.push(e.params.clone().unwrap()),
            "event.quality.finished" if do_job => return (diagnosticos, e.params.clone().unwrap()),
            _ => {}
        }
    }
}

/// Python (cadeia do roadmaps/41 bloco B, fatia 2): sem ruff detectado, o
/// `quality.run` falha NOMEANDO a ferramenta e o passo oficial — nao com o
/// "tipo de projeto nao suportado" de antes.
#[test]
fn quality_run_without_ruff_names_the_tool() {
    // Escreve um executavel e o roda: sem este lock corre com os
    // outros iguais e o `exec` volta ETXTBSY (ver lib.rs).
    let _serial = crate::serializar_executaveis();
    let (_dir, mut core, receiver) = python_quality_fixture("sem-ruff");
    let (diagnosticos, finished) = run_quality_and_collect(&mut core, &receiver);
    assert!(diagnosticos.is_empty());
    assert_eq!(finished["success"], false);
    let erro = finished["error"].as_str().unwrap();
    assert!(
        erro.contains("ruff") && erro.contains("pipx"),
        "erro nomeia a ferramenta e o passo: {erro}"
    );
}

/// Com ruff detectado: `check --output-format concise --no-fix` NO root, com o
/// `--select` do perfil de rigor quando o projeto nao declara regras; cada
/// linha vira diagnostico com arquivo/linha/coluna e severidade (E9/SyntaxError
/// = erro; o resto aviso, com "corrigivel" quando o ruff marca `[*]`).
#[test]
fn quality_run_lints_python_with_the_detected_ruff() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, mut core, receiver) = python_quality_fixture("com-ruff");
    // O workspace pede perfil strict e NAO declara regras: o `--select` do
    // perfil tem de ir na linha de comando.
    std::fs::create_dir_all(dir.join(".kinein")).unwrap();
    std::fs::write(
        dir.join(".kinein/settings.json"),
        r#"{"schemaVersion":1,"rigorProfile":"strict"}"#,
    )
    .unwrap();
    let registro = dir.join("pedido.txt");
    std::fs::write(
        dir.join("bin/ruff"),
        format!(
            "#!/bin/sh\necho \"$@\" > {reg}\npwd >> {reg}\nprintf 'app.py:1:8: F401 [*] `os` imported but unused\\napp.py:2:1: E999 SyntaxError: nao\\nFound 2 errors.\\n'\nexit 1\n",
            reg = registro.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(dir.join("bin/ruff"), std::fs::Permissions::from_mode(0o755)).unwrap();

    let (diagnosticos, finished) = run_quality_and_collect(&mut core, &receiver);

    let pedido = std::fs::read_to_string(&registro).unwrap();
    assert!(
        pedido.contains("check --output-format concise --no-fix"),
        "{pedido}"
    );
    assert!(
        pedido.contains("--select E,F,W,I,UP,B,N"),
        "perfil strict sem regras do projeto: {pedido}"
    );
    let mut linhas = pedido.lines();
    assert!(
        linhas.next().unwrap().ends_with(" ."),
        "o check e' no root: {pedido}"
    );
    assert_eq!(
        std::path::Path::new(linhas.next().unwrap()),
        dir,
        "o ruff corre NO root (caminhos relativos e ruff.toml dependem disso)"
    );
    assert_eq!(diagnosticos.len(), 2, "{diagnosticos:?}");
    let f401 = &diagnosticos[0];
    assert_eq!(f401["file"], "app.py");
    assert_eq!(
        (f401["line"].as_u64(), f401["column"].as_u64()),
        (Some(1), Some(8))
    );
    assert_eq!(f401["severity"], "warning");
    let mensagem = f401["message"].as_str().unwrap();
    assert!(
        mensagem.contains("F401") && mensagem.contains("corrigivel"),
        "{f401}"
    );
    assert_eq!(diagnosticos[1]["severity"], "error");
    assert_eq!(finished["success"], false, "achou problemas = nao passou");
}

/// Workspace Python com um teste, e um `.venv/bin/python` falso com `corpo`
/// quando dado.
#[cfg(unix)]
fn pytest_workspace(nome: &str, corpo_do_python: Option<&str>) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-test-python-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(
        dir.join("tests/test_a.py"),
        "def test_soma():\n    assert 1 + 1 == 2\n",
    )
    .unwrap();
    if let Some(corpo) = corpo_do_python {
        let py = dir.join(".venv/bin/python");
        std::fs::create_dir_all(py.parent().unwrap()).unwrap();
        std::fs::write(&py, corpo).unwrap();
        std::fs::set_permissions(&py, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir.canonicalize().unwrap()
}

/// Abre `dir`, pede `test.run` (com `filter`) e devolve os eventos do job ate
/// o `event.test.finished`, e este.
fn run_tests_and_collect(
    dir: &std::path::Path,
    filter: Option<&str>,
) -> (Vec<JsonRpcRequest>, serde_json::Value) {
    use std::time::{Duration, Instant};

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        90_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let params = filter.map_or(json!({}), |f| json!({ "filter": f }));
    let started = core.handle_request(&JsonRpcRequest::new(91_i64, "test.run", Some(params)));
    let job_id = started.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut eventos = Vec::new();
    let limite = Instant::now() + Duration::from_secs(20);
    loop {
        let e = receiver
            .recv_timeout(limite.saturating_duration_since(Instant::now()))
            .expect("evento");
        if e.params
            .as_ref()
            .is_none_or(|p| p["jobId"] != job_id.as_str())
        {
            continue;
        }
        if e.method == "event.test.finished" {
            return (eventos, e.params.unwrap());
        }
        eventos.push(e);
    }
}

/// Fatia 3 da cadeia Python: sem interpretador o `test.run` erra orientando a
/// criar o ambiente; com interpretador mas sem o modulo pytest NAQUELE
/// ambiente, diz como instalar nele (nao num Python qualquer).
#[test]
#[cfg(unix)]
fn test_run_without_python_or_pytest_says_how_to_install() {
    // Escreve um executavel e o roda: sem este lock corre com os
    // outros iguais e o `exec` volta ETXTBSY (ver lib.rs).
    let _serial = crate::serializar_executaveis();
    let dir = pytest_workspace("sem-python", None);
    let (_eventos, finished) = run_tests_and_collect(&dir, None);
    assert_eq!(finished["success"], false);
    assert!(
        finished["error"].as_str().unwrap().contains(".venv"),
        "{finished}"
    );

    let dir = pytest_workspace(
        "sem-pytest",
        Some("#!/bin/sh\necho \"$0: No module named pytest\" >&2\nexit 1\n"),
    );
    let (_eventos, finished) = run_tests_and_collect(&dir, None);
    assert_eq!(finished["success"], false);
    let erro = finished["error"].as_str().unwrap();
    assert!(
        erro.contains("pytest") && erro.contains("uv add --dev pytest"),
        "{erro}"
    );
}

/// `test.run` num workspace Python roda `python -m pytest -v` com o
/// interpretador DO PROJETO (cwd no root; `-k` com o filtro), cada linha `-v`
/// vira `event.test.case` — o resumo curto (estado na frente) nao conta duas
/// vezes — e o total vai no `finished`.
#[test]
#[cfg(unix)]
fn test_run_runs_pytest_with_the_project_interpreter() {
    let dir = pytest_workspace(
        "com-pytest",
        Some(concat!(
            "#!/bin/sh\n",
            "echo \"args: $*\"\n",
            "echo \"cwd: $(pwd)\"\n",
            "echo 'tests/test_a.py::test_soma PASSED [ 33%]'\n",
            "echo 'tests/test_a.py::test_lento SKIPPED (rede) [ 66%]'\n",
            "echo 'tests/test_a.py::test_quebra FAILED [100%]'\n",
            "echo 'FAILED tests/test_a.py::test_quebra - assert 1 == 2'\n",
            "exit 1\n"
        )),
    );
    let (eventos, finished) = run_tests_and_collect(&dir, Some("soma"));
    let started = eventos
        .iter()
        .find(|e| e.method == "event.test.started")
        .unwrap();
    assert_eq!(
        started.params.as_ref().unwrap()["command"],
        ".venv/bin/python -m pytest -v -k soma"
    );
    let saidas: Vec<&str> = eventos
        .iter()
        .filter(|e| e.method == "event.test.output")
        .map(|e| e.params.as_ref().unwrap()["line"].as_str().unwrap())
        .collect();
    assert!(saidas.contains(&"args: -m pytest -v -k soma"), "{saidas:?}");
    let cwd = format!("cwd: {}", dir.display());
    assert!(saidas.contains(&cwd.as_str()), "{saidas:?}");
    let casos: Vec<(String, String)> = eventos
        .iter()
        .filter(|e| e.method == "event.test.case")
        .map(|e| {
            let p = e.params.as_ref().unwrap();
            (
                p["name"].as_str().unwrap().to_owned(),
                p["status"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        casos,
        vec![
            ("tests/test_a.py::test_soma".to_owned(), "passed".to_owned()),
            (
                "tests/test_a.py::test_lento".to_owned(),
                "ignored".to_owned()
            ),
            (
                "tests/test_a.py::test_quebra".to_owned(),
                "failed".to_owned()
            ),
        ]
    );
    assert_eq!(finished["success"], false);
    assert_eq!(
        (
            finished["passed"].as_u64(),
            finished["failed"].as_u64(),
            finished["ignored"].as_u64()
        ),
        (Some(1), Some(1), Some(1))
    );
}

/// Num projeto `MicroPython` o Executar vai para a placa, mas o pytest fica no
/// HOST: `test.run` usa o interpretador do projeto, nunca o mpremote.
#[test]
#[cfg(unix)]
fn test_run_in_a_micropython_project_stays_on_the_host() {
    use std::os::unix::fs::PermissionsExt;

    let dir = pytest_workspace(
        "micropython",
        Some("#!/bin/sh\necho 'tests/test_a.py::test_soma PASSED [100%]'\n"),
    );
    std::fs::write(dir.join("main.py"), "import machine\n").unwrap();
    std::fs::write(
        dir.join("bin/mpremote"),
        "#!/bin/sh\necho NAO-DEVERIA\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(
        dir.join("bin/mpremote"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let (eventos, finished) = run_tests_and_collect(&dir, None);
    let started = eventos
        .iter()
        .find(|e| e.method == "event.test.started")
        .unwrap();
    assert_eq!(
        started.params.as_ref().unwrap()["command"],
        ".venv/bin/python -m pytest -v"
    );
    assert_eq!(finished["passed"], 1, "{finished}");
}

/// Pede `test.discover` e espera o `event.test.discovered` desse job.
fn discover_and_wait(
    core: &mut crate::Core,
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
) -> serde_json::Value {
    use std::time::{Duration, Instant};

    let started = core.handle_request(&JsonRpcRequest::new(
        96_i64,
        "test.discover",
        Some(json!({})),
    ));
    let job_id = started.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let limite = Instant::now() + Duration::from_secs(20);
    loop {
        let e = receiver
            .recv_timeout(limite.saturating_duration_since(Instant::now()))
            .expect("evento");
        if e.method == "event.test.discovered"
            && e.params
                .as_ref()
                .is_some_and(|p| p["jobId"] == job_id.as_str())
        {
            break e.params.unwrap();
        }
    }
}

/// Descobre e roda um so' (41 B6, o que faltou; 2026-09-13): `test.discover`
/// e' um job que lista pelo `--collect-only -q` e termina em
/// `event.test.discovered` com o node id de cada caso; `test.run { testId }`
/// passa esse id POSICIONAL ao pytest (nao `-k`), e o `testId` vence o
/// `filter`. Sem pytest no ambiente, o discover diz como instalar nele.
#[test]
#[cfg(unix)]
fn tests_are_discovered_and_one_runs_by_its_exact_id() {
    use std::time::{Duration, Instant};

    let dir = pytest_workspace(
        "descobrir",
        Some(concat!(
            "#!/bin/sh\n",
            "echo \"args: $*\"\n",
            "case \"$*\" in\n",
            "  *--collect-only*) printf 'tests/test_a.py::test_soma\\ntests/test_a.py::test_x[a b]\\n\\n2 tests collected in 0.00s\\n'; exit 0;;\n",
            "  *) echo 'tests/test_a.py::test_soma PASSED [100%]'; exit 0;;\n",
            "esac\n"
        )),
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        95_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let descoberto = discover_and_wait(&mut core, &receiver);
    assert_eq!(descoberto["runner"], "pytest");
    assert_eq!(descoberto["success"], true, "{descoberto}");
    assert_eq!(
        descoberto["command"],
        ".venv/bin/python -m pytest --collect-only -q"
    );
    let tests = descoberto["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 2, "{tests:?}");
    assert_eq!(tests[1]["id"], "tests/test_a.py::test_x[a b]");
    assert_eq!(tests[1]["name"], "test_x[a b]");
    assert_eq!(tests[1]["file"], "tests/test_a.py");

    // Rodar UM: o id posicional, e o filter e' ignorado quando ha' testId.
    let (eventos, finished) = {
        let params = json!({ "testId": "tests/test_a.py::test_x[a b]", "filter": "soma" });
        let started = core.handle_request(&JsonRpcRequest::new(97_i64, "test.run", Some(params)));
        let job_id = started.response().result.clone().unwrap()["jobId"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut eventos = Vec::new();
        let limite = Instant::now() + Duration::from_secs(20);
        loop {
            let e = receiver
                .recv_timeout(limite.saturating_duration_since(Instant::now()))
                .expect("evento");
            if e.params
                .as_ref()
                .is_none_or(|p| p["jobId"] != job_id.as_str())
            {
                continue;
            }
            if e.method == "event.test.finished" {
                break (eventos, e.params.unwrap());
            }
            eventos.push(e);
        }
    };
    let started = eventos
        .iter()
        .find(|e| e.method == "event.test.started")
        .unwrap();
    assert_eq!(
        started.params.as_ref().unwrap()["command"],
        ".venv/bin/python -m pytest -v tests/test_a.py::test_x[a b]"
    );
    let args: Vec<&str> = eventos
        .iter()
        .filter(|e| e.method == "event.test.output")
        .filter_map(|e| e.params.as_ref().unwrap()["line"].as_str())
        .filter(|l| l.starts_with("args:"))
        .collect();
    assert_eq!(
        args,
        vec!["args: -m pytest -v tests/test_a.py::test_x[a b]"],
        "sem -k"
    );
    assert_eq!(finished["passed"], 1);
}

/// Workspace `CMake` com a CDB em `build/` e um core cuja busca de
/// ferramentas e' so' a pasta `bin` do workspace (D6, clang-tidy).
fn cpp_quality_fixture(
    name: &str,
    com_cdb: bool,
) -> (
    std::path::PathBuf,
    crate::Core,
    std::sync::mpsc::Receiver<JsonRpcRequest>,
) {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-quality-cpp-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("CMakeLists.txt"), "project(x CXX)\n").unwrap();
    std::fs::write(dir.join("a.cpp"), "int main() { int x; return x; }\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    if com_cdb {
        std::fs::create_dir_all(dir.join("build")).unwrap();
        std::fs::write(
            dir.join("build/compile_commands.json"),
            format!(
                r#"[{{"directory":"{d}","file":"{d}/a.cpp","command":"c++ -c a.cpp"}}]"#,
                d = dir.display()
            ),
        )
        .unwrap();
    }
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        90_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    (dir, core, receiver)
}

/// C/C++ (D6 do roadmaps/41, 2026-09-17): o `quality.run` num `CMake` roda o
/// clang-tidy pela CDB — `run-clang-tidy -p build -quiet` quando o script
/// existe, senao `clang-tidy -p build <arquivos da CDB>`; os avisos
/// `arquivo:linha:coluna: warning: … [check]` viram diagnosticos. Sem CDB,
/// a falha diz "configure"; sem clang-tidy, nomeia a ferramenta.
#[test]
#[cfg(unix)]
fn quality_run_on_cpp_runs_clang_tidy_over_the_cdb() {
    use std::os::unix::fs::PermissionsExt;

    let _serial = crate::serializar_executaveis();

    // Sem CDB: o motor existe, mas diz o que falta.
    let (_dir, mut core, receiver) = cpp_quality_fixture("sem-cdb", false);
    let (_, finished) = run_quality_and_collect(&mut core, &receiver);
    assert_eq!(finished["success"], false);
    assert!(
        finished["error"]
            .as_str()
            .unwrap()
            .contains("compilation database"),
        "{finished}"
    );

    // Com CDB e sem clang-tidy: a ferramenta, nomeada.
    let (dir, mut core, receiver) = cpp_quality_fixture("sem-tidy", true);
    let (_, finished) = run_quality_and_collect(&mut core, &receiver);
    assert!(
        finished["error"].as_str().unwrap().contains("clang-tidy"),
        "{finished}"
    );

    // clang-tidy falso (sem o script): recebe -p build e os arquivos da CDB
    // e avisa como o real.
    let executavel = |nome: &str, corpo: &str| {
        let p = dir.join("bin").join(nome);
        std::fs::write(&p, corpo).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    executavel(
        "clang-tidy",
        &format!(
            "#!/bin/sh\necho \"tidy-falso $*\"\necho \"{d}/a.cpp:1:18: warning: variable 'x' is uninitialized [cppcoreguidelines-init-variables]\"\nexit 0\n",
            d = dir.display()
        ),
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            91_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() }))
        ))
        .response()
        .error
        .is_none()
    );
    let (diagnosticos, finished) = run_quality_and_collect(&mut core, &receiver);
    assert_eq!(finished["success"], true, "{finished}");
    assert_eq!(diagnosticos.len(), 1, "{diagnosticos:?}");
    assert_eq!(diagnosticos[0]["severity"], "warning");
    assert_eq!(diagnosticos[0]["line"], 1);
    assert!(
        diagnosticos[0]["message"]
            .as_str()
            .unwrap()
            .contains("[cppcoreguidelines-init-variables]"),
        "{}",
        diagnosticos[0]
    );
}

/// Com o script `run-clang-tidy` (o do LLVM), ele vence: `-p build -quiet`,
/// sem lista de arquivos.
#[test]
#[cfg(unix)]
fn quality_run_on_cpp_prefers_run_clang_tidy() {
    use std::os::unix::fs::PermissionsExt;

    let _serial = crate::serializar_executaveis();

    let (dir, _core, _receiver) = cpp_quality_fixture("run-tidy", true);
    let executavel = |nome: &str, corpo: &str| {
        let p = dir.join("bin").join(nome);
        std::fs::write(&p, corpo).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    executavel("clang-tidy", "#!/bin/sh\nexit 0\n");
    executavel(
        "run-clang-tidy",
        "#!/bin/sh\necho \"run-tidy-falso $*\"\nexit 0\n",
    );
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    assert!(
        core.handle_request(&JsonRpcRequest::new(
            92_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() }))
        ))
        .response()
        .error
        .is_none()
    );
    let started = core.handle_request(&JsonRpcRequest::new(93_i64, "quality.run", Some(json!({}))));
    let job_id = started.response().result.clone().unwrap()["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut linhas = Vec::new();
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let e = receiver
            .recv_timeout(limite.saturating_duration_since(std::time::Instant::now()))
            .expect("evento");
        let p = e.params.clone().unwrap_or_default();
        if p["jobId"] != job_id.as_str() {
            continue;
        }
        match e.method.as_str() {
            "event.quality.output" => linhas.push(p["line"].as_str().unwrap().to_owned()),
            "event.quality.finished" => break,
            _ => {}
        }
    }
    assert!(
        linhas
            .iter()
            .any(|l| l == &format!("run-tidy-falso -p {} -quiet", dir.join("build").display())),
        "{linhas:?}"
    );
}
