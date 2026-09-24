//! Debug dispatch (`debug.*`): guardas, confinamento e store de breakpoints.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

#[test]
fn debug_methods_require_workspace_or_manager() {
    // Escreve um executavel e o roda: sem este lock corre com os
    // outros iguais e o `exec` volta ETXTBSY (ver lib.rs).
    let _serial = crate::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut core = core_with_empty_search_path("debug-guards");

    let start = core.handle_request(&JsonRpcRequest::new(1_i64, "debug.start", None));
    assert_eq!(
        start.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
    let breakpoints = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": "/x/main.c", "breakpoints": [{ "line": 1 }] })),
    ));
    assert_eq!(
        breakpoints.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    // Sem enable_lsp nao ha manager: controles de sessao indisponiveis.
    let resume = core.handle_request(&JsonRpcRequest::new(3_i64, "debug.continue", None));
    assert_eq!(
        resume.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InternalError
    );
}

#[test]
fn breakpoint_flow_and_session_guards_through_dispatch() {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-flow", std::process::id()));
    let root = base.join("ws");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(base.join("fora.rs"), "fn main() {}\n").unwrap();

    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-flow");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let outside = core.handle_request(&JsonRpcRequest::new(
        11_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": base.join("fora.rs").to_str().unwrap(), "breakpoints": [{ "line": 1 }] })),
    ));
    assert_eq!(
        outside.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let missing = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("nao-existe.rs").to_str().unwrap(), "breakpoints": [{ "line": 1 }] })),
    ));
    assert_eq!(
        missing.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let stored = core.handle_request(&JsonRpcRequest::new(
        13_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("main.rs").to_str().unwrap(), "breakpoints": [{ "line": 7 }, { "line": 3 }, { "line": 7 }] })),
    ));
    let result = stored.response().result.as_ref().unwrap().clone();
    assert_eq!(result["breakpoints"][0]["line"], 3);
    assert_eq!(result["breakpoints"][1]["line"], 7);
    assert_eq!(result["breakpoints"][0]["verified"], false);

    let cleared = core.handle_request(&JsonRpcRequest::new(
        14_i64,
        "debug.setBreakpoints",
        Some(json!({ "file": root.join("main.rs").to_str().unwrap(), "breakpoints": [] })),
    ));
    let result = cleared.response().result.as_ref().unwrap().clone();
    assert_eq!(result["breakpoints"].as_array().unwrap().len(), 0);

    // Nenhuma sessao viva: controles respondem estado invalido, nao panico.
    let resume = core.handle_request(&JsonRpcRequest::new(15_i64, "debug.continue", None));
    assert_eq!(
        resume.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    let ghost_program = core.handle_request(&JsonRpcRequest::new(
        16_i64,
        "debug.start",
        Some(json!({ "program": root.join("fantasma").to_str().unwrap() })),
    ));
    assert_eq!(
        ghost_program.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // Automatico num cargo sem binario compilado: erro claro, nao spawn.
    let no_target = core.handle_request(&JsonRpcRequest::new(17_i64, "debug.start", None));
    let error = no_target.response().error.as_ref().unwrap().clone();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert!(error.message.contains("compile antes"));
}

#[test]
fn inspection_methods_validate_params_and_session_state() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-inspect", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-inspect");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Sem sessao viva: INVALID_REQUEST com a mensagem de dominio.
    let stack = core.handle_request(&JsonRpcRequest::new(31_i64, "debug.stackTrace", None));
    assert_eq!(
        stack.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    // frameId e ref juntos (ou ausentes) e INVALID_PARAMS antes de tudo.
    let both = core.handle_request(&JsonRpcRequest::new(
        32_i64,
        "debug.variables",
        Some(json!({ "frameId": 1, "ref": 2 })),
    ));
    assert_eq!(
        both.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
    let neither = core.handle_request(&JsonRpcRequest::new(33_i64, "debug.variables", None));
    assert_eq!(
        neither.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    let by_ref = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "debug.variables",
        Some(json!({ "ref": 5 })),
    ));
    assert_eq!(
        by_ref.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn command_list_includes_debug_start() {
    let mut core = core_with_empty_search_path("debug-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(20_i64, "command.list", None));
    let result = outcome.response().result.as_ref().unwrap().clone();
    let has_debug_start = result["commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|command| command["id"] == "debug.start");
    assert!(has_debug_start);
}

/// `debug.evaluate` (watch) recusa antes de chegar ao adapter o que o adapter
/// responderia mal: expressao vazia. E sem sessao viva, diz que nao ha sessao
/// em vez de estourar.
#[test]
fn evaluate_guards_empty_expression_and_missing_session() {
    let mut core = core_with_empty_search_path("debug-evaluate");

    // Sem enable_lsp nao ha manager de debug.
    let sem_sessao = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "debug.evaluate",
        Some(json!({ "expression": "conta" })),
    ));
    // `InternalError` e a convencao ja existente do `debug_unavailable_response`
    // para "debug nao habilitado neste loop" — nao invento codigo novo aqui.
    assert_eq!(
        sem_sessao.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InternalError
    );

    // Expressao so' de espacos: recusada como params invalidos, e a mensagem
    // vem do core — a do lldb para string vazia nao ajuda ninguem.
    let vazia = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "debug.evaluate",
        Some(json!({ "expression": "   " })),
    ));
    assert_eq!(
        vazia.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );

    // Sem `expression` nenhuma: tambem params invalidos, nao panico.
    let sem_campo = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "debug.evaluate",
        Some(json!({})),
    ));
    assert_eq!(
        sem_campo.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
}

/// Fatia 4 da cadeia Python (41 bloco B): um alvo `.py` — mesmo num
/// workspace `CMake` — vai para o debugpy DO interpretador do projeto, e o
/// `debug.start` recusa com o passo certo quando nao ha' interpretador ou
/// quando o interpretador nao tem o modulo. O adaptador real e' provado pelo
/// gate (verificar-python-debug.sh); aqui sao os desvios que o precedem.
#[test]
#[cfg(unix)]
fn a_python_target_needs_the_project_interpreter_with_debugpy() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-python", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tools")).unwrap();
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    // Workspace CMake com um script Python ao lado — como o projeto da
    // exercitacao do gate.
    std::fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(x CXX)\n",
    )
    .unwrap();
    std::fs::write(dir.join("tools/gera.py"), "print('x')\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    let gera = dir.join("tools/gera.py");
    let abrir = || {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            dir.join("bin"),
        ));
        core.enable_lsp(sender);
        let opened = core.handle_request(&JsonRpcRequest::new(
            30_i64,
            "workspace.open",
            Some(json!({ "path": dir.to_str().unwrap() })),
        ));
        assert!(opened.response().error.is_none());
        core
    };
    let iniciar = |core: &mut crate::Core| {
        core.handle_request(&JsonRpcRequest::new(
            31_i64,
            "debug.start",
            Some(json!({ "program": gera.to_str().unwrap() })),
        ))
        .response()
        .clone()
    };

    // Sem interpretador nenhum: orienta a criar o ambiente.
    let mut core = abrir();
    let erro = iniciar(&mut core).error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains(".venv"), "{erro:?}");

    // Interpretador SEM o modulo: TOOL_NOT_FOUND com o passo para instalar
    // NO ambiente — e a sonda e' o `import` isolado, nao o adaptador subindo.
    let python = dir.join(".venv/bin/python");
    std::fs::create_dir_all(python.parent().unwrap()).unwrap();
    let registro = dir.join("chamadas.txt");
    std::fs::write(
        &python,
        format!(
            "#!/bin/sh\necho \"$*\" >> {reg}\necho 'No module named debugpy' >&2\nexit 1\n",
            reg = registro.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&python, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut core = abrir();
    let erro = iniciar(&mut core).error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound, "{erro:?}");
    assert!(
        erro.message.contains("debugpy ausente") && erro.message.contains("uv add --dev debugpy"),
        "{erro:?}"
    );
    let chamadas = std::fs::read_to_string(&registro).unwrap();
    assert!(chamadas.contains("-I -c import debugpy"), "{chamadas}");
    assert!(
        !chamadas.contains("debugpy.adapter"),
        "nao sobe o adaptador sem o modulo: {chamadas}"
    );
}

/// O adaptador que MORRE sem responder deixa o motivo (2026-09-17): o que ele
/// escreveu no stderr sai ao vivo como `event.debug.output { category:
/// "adapter" }` e entra na mensagem do `debug.start` que falhou. Antes o
/// stderr ia para /dev/null e o gate viu "o adapter nao respondeu a
/// `initialize`" sem causa. O "adaptador" e' o interpretador falso: passa no
/// `import debugpy` e, chamado como `-m debugpy.adapter`, reclama e sai.
#[test]
#[cfg(unix)]
fn a_dying_adapter_leaves_its_stderr_in_the_error_and_as_events() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-adapter-stderr", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".venv/bin")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(dir.join("main.py"), "print('x')\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    let python = dir.join(".venv/bin/python");
    std::fs::write(
        &python,
        "#!/bin/sh\ncase \"$*\" in\n  *'import debugpy'*) exit 0 ;;\n  *debugpy.adapter*) \
         echo 'Traceback (most recent call last):' >&2; echo 'ImportError: boom no adaptador' >&2; \
         exit 1 ;;\nesac\nexit 0\n",
    )
    .unwrap();
    std::fs::set_permissions(&python, std::fs::Permissions::from_mode(0o755)).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-adapter-stderr");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        50_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let erro = core
        .handle_request(&JsonRpcRequest::new(
            51_i64,
            "debug.start",
            Some(json!({ "program": dir.join("main.py").to_str().unwrap() })),
        ))
        .response()
        .error
        .clone()
        .unwrap();
    assert!(
        erro.message.contains("--- stderr do adaptador ---")
            && erro.message.contains("ImportError: boom no adaptador"),
        "{erro:?}"
    );
    // As mesmas linhas sairam ao vivo, com a categoria propria.
    let mut vivas = Vec::new();
    while let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(300)) {
        if event.method == "event.debug.output" {
            let params = event.params.unwrap();
            assert_eq!(params["category"], "adapter");
            vivas.push(params["line"].as_str().unwrap().to_owned());
        }
    }
    assert_eq!(
        vivas,
        [
            "Traceback (most recent call last):",
            "ImportError: boom no adaptador"
        ]
    );
}

/// Num workspace Python, `debug.start {}` acha o mesmo ponto de entrada do
/// Executar; um pacote com `__main__.py` e' o alvo `-m pacote` (2026-09-13).
#[test]
fn debug_start_without_program_uses_the_python_entry_point() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-debug-python-entrada", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("pacote")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(dir.join("pacote/__main__.py"), "").unwrap();
    let dir = dir.canonicalize().unwrap();
    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("debug-python-entrada");
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let erro = core
        .handle_request(&JsonRpcRequest::new(41_i64, "debug.start", None))
        .response()
        .error
        .clone()
        .unwrap();
    // O pacote E' um alvo (`-m pacote`, desde 2026-09-13): passou pela
    // resolucao e chegou ao caminho do Python — a recusa e' a do interpretador.
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains("interpretador"), "{erro:?}");
    assert_eq!(
        crate::dap::resolve_program(kinein_protocol::ProjectKind::Python, &dir).unwrap(),
        crate::dap::DebugTarget::Module("pacote".to_owned())
    );

    // Com main.py, o alvo e' ele — e a recusa seguinte continua a do
    // interpretador (o alvo foi resolvido).
    std::fs::write(dir.join("main.py"), "").unwrap();
    let erro = core
        .handle_request(&JsonRpcRequest::new(42_i64, "debug.start", None))
        .response()
        .error
        .clone()
        .unwrap();
    assert!(erro.message.contains("interpretador"), "{erro:?}");
}
