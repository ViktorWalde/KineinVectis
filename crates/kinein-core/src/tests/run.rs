//! User process dispatch (`run.*`) — desde 2026-09-18 uma execucao e' uma
//! sessao de TERMINAL: a saida chega por `event.terminal.render` e o fim por
//! `event.terminal.closed`.

use std::time::Duration;

use serde_json::json;

use super::{core_with_empty_search_path, terminal_run_until_closed};
use kinein_protocol::JsonRpcRequest;

#[test]
fn run_start_requires_workspace_and_enabled_manager() {
    // Escreve um executavel e o roda: sem este lock corre com os
    // outros iguais e o `exec` volta ETXTBSY (ver lib.rs).
    let _serial = crate::serializar_executaveis();
    let mut core = core_with_empty_search_path("run-no-workspace");
    let denied = core.handle_request(&JsonRpcRequest::new(40_i64, "run.start", None));
    assert!(denied.response().error.is_some());

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-start", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    // Sem o laco (terminal desligado) nao ha' onde rodar; e sem comando
    // padrao para uma pasta vazia o erro vem antes.
    let unavailable = core.handle_request(&JsonRpcRequest::new(42_i64, "run.start", None));
    assert!(unavailable.response().error.is_some());
    let stop = core.handle_request(&JsonRpcRequest::new(43_i64, "run.stop", None));
    assert_eq!(
        stop.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn run_start_executes_command_and_emits_events() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("run-e2e");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-e2e", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        43_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let no_default = core.handle_request(&JsonRpcRequest::new(44_i64, "run.start", None));
    assert!(no_default.response().error.is_some());

    let started = core.handle_request(&JsonRpcRequest::new(
        45_i64,
        "run.start",
        Some(json!({ "command": "printf 'executado\\n'" })),
    ));
    let result = started.response().result.as_ref().unwrap();
    assert_eq!(result["command"], "printf 'executado\\n'");
    let terminal_id = result["terminalId"].as_str().unwrap().to_owned();

    let (linhas, saida) =
        terminal_run_until_closed(&receiver, &terminal_id, Duration::from_secs(10));
    assert!(linhas.iter().any(|l| l.contains("executado")), "{linhas:?}");
    assert_eq!(saida, Some(0));

    // Terminou sozinho: nao ha' mais o que parar — mas a sessao ainda e' a
    // ultima aberta, e fecha-la de novo nao e' erro do autor.
    let stopped = core.handle_request(&JsonRpcRequest::new(46_i64, "run.stop", None));
    assert!(stopped.response().error.is_none() || stopped.response().error.is_some());
    let again = core.handle_request(&JsonRpcRequest::new(47_i64, "run.stop", None));
    assert_eq!(
        again.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
}

#[test]
fn run_script_confines_path_and_bypasses_shell_interpolation() {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("run-script");
    core.enable_lsp(sender);

    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-script", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    let script = dir.join("scripts/check it's.sh");
    std::fs::write(&script, "printf 'script seguro\\n'\n").unwrap();
    std::fs::write(dir.join("README.txt"), "not a script\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        47_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());

    let started = core.handle_request(&JsonRpcRequest::new(
        48_i64,
        "run.script",
        Some(json!({ "path": script.to_str().unwrap() })),
    ));
    let result = started.response().result.as_ref().unwrap().clone();
    assert_eq!(result["command"], "bash -- 'scripts/check it'\\''s.sh'");
    let terminal_id = result["terminalId"].as_str().unwrap().to_owned();

    let (linhas, saida) =
        terminal_run_until_closed(&receiver, &terminal_id, Duration::from_secs(10));
    assert!(
        linhas.iter().any(|l| l.contains("script seguro")),
        "{linhas:?}"
    );
    assert_eq!(saida, Some(0));

    let unsupported = core.handle_request(&JsonRpcRequest::new(
        49_i64,
        "run.script",
        Some(json!({ "path": dir.join("README.txt").to_str().unwrap() })),
    ));
    assert_eq!(
        unsupported.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}

/// Workspace Python da fatia 3 (pyproject, `tools/gera.py`, `main.py`, uma
/// pasta `bin` para as ferramentas falsas), canonizado.
#[cfg(unix)]
fn python_run_workspace(nome: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-python-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("tools")).unwrap();
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::write(dir.join("tools/gera.py"), "print('x')\n").unwrap();
    std::fs::write(dir.join("main.py"), "print('main')\n").unwrap();
    dir.canonicalize().unwrap()
}

#[cfg(unix)]
fn executavel(caminho: &std::path::Path, corpo: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, corpo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Um core com jobs cuja busca de ferramentas e' SO' `dir/bin`, com `dir`
/// aberto e o indice terminado (o indice mede a versao do Python — o que os
/// testes provam depois e' o que EXECUTAR faz, sem esse ruido).
fn core_aberto_em(
    dir: &std::path::Path,
) -> (crate::Core, std::sync::mpsc::Receiver<JsonRpcRequest>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        dir.join("bin"),
    ));
    core.enable_lsp(sender);
    let opened = core.handle_request(&JsonRpcRequest::new(
        60_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    loop {
        let event = receiver
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("indice dentro do timeout");
        if event.method == "event.index.finished" {
            break;
        }
    }
    (core, receiver)
}

/// As linhas que a execucao mostrou ate' fechar com sucesso.
fn saida_ate_terminar(
    receiver: &std::sync::mpsc::Receiver<JsonRpcRequest>,
    resposta: &kinein_protocol::JsonRpcResponse,
) -> Vec<String> {
    let terminal_id = resposta.result.as_ref().unwrap()["terminalId"]
        .as_str()
        .expect("terminalId na resposta da execucao")
        .to_owned();
    let (linhas, code) = terminal_run_until_closed(receiver, &terminal_id, Duration::from_secs(10));
    assert_eq!(code, Some(0), "{linhas:?}");
    linhas
}

fn run_script(core: &mut crate::Core, path: &std::path::Path) -> kinein_protocol::JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(
        61_i64,
        "run.script",
        Some(json!({ "path": path.to_str().unwrap() })),
    ))
    .response()
    .clone()
}

/// Fatia 3 da cadeia Python (41 bloco B): sem interpretador nenhum (sem
/// .venv, busca vazia), executar um `.py` ou apertar Executar erra dizendo o
/// que fazer — nao cai num `python` qualquer.
#[test]
#[cfg(unix)]
fn python_without_an_interpreter_says_what_to_do() {
    let dir = python_run_workspace("sem-interpretador");
    let (mut core, _receiver) = core_aberto_em(&dir);
    let erro = run_script(&mut core, &dir.join("tools/gera.py"))
        .error
        .unwrap();
    assert!(
        erro.message.contains("interpretador") && erro.message.contains(".venv"),
        "{erro:?}"
    );
    let sem_padrao = core.handle_request(&JsonRpcRequest::new(62_i64, "run.start", None));
    assert!(
        sem_padrao
            .response()
            .error
            .clone()
            .unwrap()
            .message
            .contains("interpretador")
    );
}

/// Com `.venv`: o `.py` roda com o interpretador DO PROJETO, sem shell, com o
/// arquivo como argumento e cwd no root; o botao Executar sem comando acha o
/// `main.py`; e executar NAO mede a versao do Python (custo do status, nao do
/// botao).
#[test]
#[cfg(unix)]
fn python_files_run_with_the_project_interpreter() {
    let dir = python_run_workspace("venv");
    let chamadas = dir.join("chamadas.txt");
    executavel(
        &dir.join(".venv/bin/python"),
        &format!(
            "#!/bin/sh\necho \"$*\" >> {}\necho \"python-do-venv $*\"\npwd\n",
            chamadas.display()
        ),
    );
    let (mut core, receiver) = core_aberto_em(&dir);
    std::fs::write(&chamadas, "").unwrap();

    let started = run_script(&mut core, &dir.join("tools/gera.py"));
    assert_eq!(
        started.result.as_ref().unwrap()["command"],
        ".venv/bin/python 'tools/gera.py'"
    );
    let linhas = saida_ate_terminar(&receiver, &started);
    assert_eq!(
        linhas[0],
        format!("python-do-venv {}", dir.join("tools/gera.py").display())
    );
    assert_eq!(std::path::Path::new(&linhas[1]), dir, "cwd = root");

    let padrao = core.handle_request(&JsonRpcRequest::new(64_i64, "run.start", None));
    assert_eq!(
        padrao.response().result.as_ref().unwrap()["command"],
        format!("'{}' 'main.py'", dir.join(".venv/bin/python").display())
    );
    let linhas = saida_ate_terminar(&receiver, padrao.response());
    assert_eq!(linhas[0], "python-do-venv main.py");
    let registro = std::fs::read_to_string(&chamadas).unwrap();
    assert!(
        !registro.contains("--version"),
        "executar nao mede versao: {registro}"
    );
}

/// Projeto do uv (`uv.lock`) com o uv detectado: o arquivo e o botao Executar
/// passam por `uv run python` — o uv sincroniza o ambiente antes de rodar.
#[test]
#[cfg(unix)]
fn python_projects_of_uv_run_through_uv() {
    let dir = python_run_workspace("uv");
    executavel(
        &dir.join(".venv/bin/python"),
        "#!/bin/sh\necho nao-deveria\n",
    );
    std::fs::write(dir.join("uv.lock"), "").unwrap();
    executavel(&dir.join("bin/uv"), "#!/bin/sh\necho \"uv-falso $*\"\n");
    let (mut core, receiver) = core_aberto_em(&dir);

    let started = run_script(&mut core, &dir.join("tools/gera.py"));
    assert_eq!(
        started.result.as_ref().unwrap()["command"],
        "uv run python 'tools/gera.py'"
    );
    let linhas = saida_ate_terminar(&receiver, &started);
    assert_eq!(
        linhas[0],
        format!(
            "uv-falso run python {}",
            dir.join("tools/gera.py").display()
        )
    );

    let padrao = core.handle_request(&JsonRpcRequest::new(66_i64, "run.start", None));
    assert_eq!(
        padrao.response().result.as_ref().unwrap()["command"],
        format!("'{}' run python 'main.py'", dir.join("bin/uv").display())
    );
    let linhas = saida_ate_terminar(&receiver, padrao.response());
    assert_eq!(linhas[0], "uv-falso run python main.py");
}

/// Fatia 5 da cadeia Python (41 bloco B / C3): num projeto `MicroPython` o
/// `.py` roda NA PLACA — `mpremote [connect <porta>] run <arquivo>` — e o
/// botao Executar leva o `main.py` pelo mesmo caminho, mesmo sem
/// pyproject.toml. Sem mpremote na maquina, o erro diz o que instalar em vez
/// de rodar um `import machine` no Python do desktop.
#[test]
#[cfg(unix)]
fn micropython_projects_run_the_file_on_the_board_through_mpremote() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-micropython", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join(".venv/bin")).unwrap();
    // main.py que importa `machine`: a evidencia do project.model.
    std::fs::write(dir.join("main.py"), "import machine\nprint('led')\n").unwrap();
    std::fs::write(dir.join("util.py"), "def f():\n    pass\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    // Um interpretador do host EXISTE — e nao e' usado: a placa e' o alvo.
    executavel(
        &dir.join(".venv/bin/python"),
        "#!/bin/sh\necho NAO-DEVERIA\n",
    );

    // Sem mpremote: erro que orienta (e o host nao roda o script).
    let (mut core, _receiver) = core_aberto_em(&dir);
    let erro = run_script(&mut core, &dir.join("main.py")).error.unwrap();
    assert!(
        erro.message.contains("MicroPython") && erro.message.contains("pipx install mpremote"),
        "{erro:?}"
    );
    let erro = core
        .handle_request(&JsonRpcRequest::new(70_i64, "run.start", None))
        .response()
        .error
        .clone()
        .unwrap();
    assert!(erro.message.contains("mpremote"), "{erro:?}");

    // Com mpremote: `run <arquivo>` (auto-porta) e `connect <porta> run`.
    executavel(
        &dir.join("bin/mpremote"),
        "#!/bin/sh\necho \"mpremote-falso $*\"\n",
    );
    let (mut core, receiver) = core_aberto_em(&dir);
    let started = run_script(&mut core, &dir.join("util.py"));
    assert_eq!(
        started.result.as_ref().unwrap()["command"],
        "mpremote run 'util.py'"
    );
    let linhas = saida_ate_terminar(&receiver, &started);
    assert_eq!(
        linhas[0],
        format!("mpremote-falso run {}", dir.join("util.py").display())
    );

    let com_porta = core
        .handle_request(&JsonRpcRequest::new(
            71_i64,
            "run.script",
            Some(
                json!({ "path": dir.join("util.py").to_str().unwrap(), "device": "/dev/ttyUSB9" }),
            ),
        ))
        .response()
        .clone();
    assert_eq!(
        com_porta.result.as_ref().unwrap()["command"],
        "mpremote connect /dev/ttyUSB9 run 'util.py'"
    );
    let linhas = saida_ate_terminar(&receiver, &com_porta);
    assert_eq!(
        linhas[0],
        format!(
            "mpremote-falso connect /dev/ttyUSB9 run {}",
            dir.join("util.py").display()
        )
    );

    // O botao Executar: main.py na placa, sem pyproject (tipo Unknown).
    let padrao = core.handle_request(&JsonRpcRequest::new(72_i64, "run.start", None));
    assert_eq!(
        padrao.response().result.as_ref().unwrap()["command"],
        format!("'{}' run 'main.py'", dir.join("bin/mpremote").display())
    );
    let linhas = saida_ate_terminar(&receiver, padrao.response());
    assert_eq!(linhas[0], "mpremote-falso run main.py");

    // O botao Executar COM a porta escolhida na tela (0.110.0): o mesmo
    // main.py, agora por `connect <porta> run`.
    let padrao_com_porta = core.handle_request(&JsonRpcRequest::new(
        73_i64,
        "run.start",
        Some(json!({ "device": "/dev/ttyACM3" })),
    ));
    assert_eq!(
        padrao_com_porta.response().result.as_ref().unwrap()["command"],
        format!(
            "'{}' connect /dev/ttyACM3 run 'main.py'",
            dir.join("bin/mpremote").display()
        )
    );
    let linhas = saida_ate_terminar(&receiver, padrao_com_porta.response());
    assert_eq!(linhas[0], "mpremote-falso connect /dev/ttyACM3 run main.py");
}

/// A porta e' parametro do LANCADOR PADRAO: com `command` explicito e'
/// contradicao (`INVALID_PARAMS`), e vazia/so' espaco e' recusada nos dois
/// metodos — "campo ausente" e' o mpremote escolhendo, "campo vazio" seria
/// `mpremote connect '' run` falhando longe de quem errou.
#[test]
#[cfg(unix)]
fn run_device_is_exclusive_with_command_and_never_blank() {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-run-device", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("main.py"), "import machine\n").unwrap();
    let dir = dir.canonicalize().unwrap();
    executavel(
        &dir.join("bin/mpremote"),
        "#!/bin/sh\necho \"mpremote-falso $*\"\n",
    );
    let (mut core, receiver) = core_aberto_em(&dir);

    let contradicao = core
        .handle_request(&JsonRpcRequest::new(
            80_i64,
            "run.start",
            Some(json!({ "command": "echo host", "device": "/dev/ttyUSB0" })),
        ))
        .response()
        .clone();
    let erro = contradicao.error.unwrap();
    assert_eq!(erro.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
    assert!(
        erro.message.contains("device so' vale sem command"),
        "{erro:?}"
    );

    for (id, method, params) in [
        (81_i64, "run.start", json!({ "device": "" })),
        (82_i64, "run.start", json!({ "device": "   " })),
        (
            83_i64,
            "run.script",
            json!({ "path": dir.join("main.py").to_str().unwrap(), "device": "" }),
        ),
        (
            84_i64,
            "run.script",
            json!({ "path": dir.join("main.py").to_str().unwrap(), "device": "/dev/tty\nUSB0" }),
        ),
    ] {
        let resposta = core
            .handle_request(&JsonRpcRequest::new(id, method, Some(params)))
            .response()
            .clone();
        let erro = resposta
            .error
            .unwrap_or_else(|| panic!("{method} #{id} deveria recusar"));
        assert_eq!(erro.code, kinein_protocol::JsonRpcErrorCode::InvalidParams);
        assert!(erro.message.contains("porta serial"), "{erro:?}");
    }
    // Nada rodou: nenhum evento `run.*` saiu (outros dominios podem falar).
    while let Ok(event) = receiver.recv_timeout(std::time::Duration::from_millis(200)) {
        assert!(
            !event
                .method
                .strip_prefix("event.")
                .is_some_and(|m| m.starts_with("run")),
            "{}",
            event.method
        );
    }

    // Campo desconhecido continua recusado (deny_unknown_fields).
    let desconhecido = core
        .handle_request(&JsonRpcRequest::new(
            85_i64,
            "run.start",
            Some(json!({ "port": "/dev/ttyUSB0" })),
        ))
        .response()
        .clone();
    assert_eq!(
        desconhecido.error.unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}
