//! `remote.*` (P6 fatia 1 do `roadmaps/42`, 2026-09-17) por despacho: o
//! catalogo sem segredo; o probe com um `ssh` FALSO que responde como uma
//! Raspberry Pi (e outro que recusa a chave, para a dica do `ssh-copy-id`);
//! o deploy com `rsync` falso e, sem ele, `scp`; e o `remote.command` puro,
//! com o `host:porta` para o kit.

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
        .join(format!("{}-remote-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("ws/build")).unwrap();
    let base = base.canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ));
    core.enable_lsp(sender);
    let mut c = Cenario {
        core,
        root: base.join("ws"),
        events: receiver,
    };
    let r = c.rpc(
        "workspace.open",
        json!({ "path": c.root.to_str().unwrap() }),
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    c
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

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let r = self.rpc(method, params);
        assert!(r.error.is_none(), "{method}: {:?}", r.error);
        r.result.unwrap()
    }

    fn evento(&self, nome: &str) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < prazo {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == nome
            {
                return e.params.unwrap();
            }
        }
        panic!("{nome} nao chegou");
    }

    fn salvar_pi(&mut self) {
        self.ok(
            "remote.save",
            json!({ "target": { "name": "pi", "host": "192.168.0.42", "user": "pi", "port": 2222,
                                "identityFile": "/home/u/.ssh/pi" } }),
        );
    }
}

/// Salvar normaliza e ordena; a porta 22 some; um campo de senha e' recusado
/// pelo `deny_unknown_fields`; remover devolve o catalogo; o arquivo em disco
/// nao tem nada que pareca segredo.
#[test]
fn the_catalogue_saves_without_secrets_sorted_by_name() {
    let mut c = cenario("catalogo");
    assert_eq!(c.ok("remote.list", json!({}))["targets"], json!([]));
    c.ok(
        "remote.save",
        json!({ "target": { "name": " zeta ", "host": " bancada.local ", "port": 22, "user": "" } }),
    );
    c.salvar_pi();
    let lista = c.ok("remote.list", json!({}))["targets"].clone();
    assert_eq!(lista[0]["name"], "pi");
    assert_eq!(
        lista[1],
        json!({ "name": "zeta", "host": "bancada.local" }),
        "{lista}"
    );

    let senha = c.rpc(
        "remote.save",
        json!({ "target": { "name": "x", "host": "h", "password": "1234" } }),
    );
    assert_eq!(senha.error.unwrap().code, JsonRpcErrorCode::InvalidParams);
    let invalido = c.rpc(
        "remote.save",
        json!({ "target": { "name": "a b", "host": "h" } }),
    );
    assert!(invalido.error.unwrap().message.contains("barra nem espaco"));

    let texto = std::fs::read_to_string(c.root.join(".kinein/remotes.json")).unwrap();
    assert!(!texto.to_lowercase().contains("password") && !texto.contains("senha"));

    let depois = c.ok("remote.remove", json!({ "name": "zeta" }));
    assert_eq!(depois["targets"].as_array().unwrap().len(), 1);
    assert_eq!(
        c.ok("remote.remove", json!({ "name": "nunca" }))["targets"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

/// O probe: `ssh -o BatchMode=yes -o ConnectTimeout=5 -p 2222 -i <chave>
/// pi@host '<script>'`; o falso responde como uma Pi e o evento traz
/// arquitetura, kernel e as ferramentas (rsync ausente). Sem `ssh` no PATH a
/// recusa e' sincrona; alvo desconhecido idem.
#[test]
#[cfg(unix)]
fn the_probe_reads_the_target_through_a_batch_mode_ssh() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("probe");
    c.salvar_pi();
    let sem_ssh = c.rpc("remote.probe", json!({ "name": "pi" }));
    assert!(sem_ssh.error.unwrap().message.contains("openssh-client"));
    let desconhecido = c.rpc("remote.probe", json!({ "name": "outro" }));
    assert!(desconhecido.error.unwrap().message.contains("`outro`"));

    let registro = c.bin().join("ssh.args");
    executavel(
        &c.bin().join("ssh"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{r}'\nprintf 'aarch64\\nLinux 6.6.31+rpt-rpi-v8\\ngdbserver=/usr/bin/gdbserver\\npython3=/usr/bin/python3\\nrsync=\\n'\n",
            r = registro.display()
        ),
    );
    let aceito = c.ok("remote.probe", json!({ "name": "pi" }));
    assert!(aceito["command"].as_str().unwrap().contains(
        "-o BatchMode=yes -o ConnectTimeout=5 -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42"
    ));
    let ev = c.evento("event.remote.probed");
    assert_eq!(ev["success"], true, "{ev}");
    assert_eq!(ev["name"], "pi");
    assert_eq!(ev["arch"], "aarch64");
    assert_eq!(ev["kernel"], "Linux 6.6.31+rpt-rpi-v8");
    let tools = ev["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    assert_eq!(
        tools[0],
        json!({ "id": "gdbserver", "found": true, "path": "/usr/bin/gdbserver" })
    );
    assert_eq!(tools[2], json!({ "id": "rsync", "found": false }));
    let args: Vec<String> = std::fs::read_to_string(&registro)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        &args[..9],
        [
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-p",
            "2222",
            "-i",
            "/home/u/.ssh/pi",
            "pi@192.168.0.42"
        ]
    );
    assert!(args[9].starts_with("uname -m; uname -sr;"), "{}", args[9]);

    // A chave recusada: o evento diz `ssh-copy-id`.
    executavel(
        &c.bin().join("ssh"),
        "#!/bin/sh\necho 'pi@192.168.0.42: Permission denied (publickey).' >&2\nexit 255\n",
    );
    c.ok("remote.probe", json!({ "name": "pi" }));
    let ev = c.evento("event.remote.probed");
    assert_eq!(ev["success"], false);
    assert!(
        ev["error"]
            .as_str()
            .unwrap()
            .contains("ssh-copy-id pi@192.168.0.42"),
        "{ev}"
    );
    assert!(ev["raw"].as_str().unwrap().contains("Permission denied"));
    // A MESMA causa, tipada: e' disso que a UI decide qual gesto oferecer.
    assert_eq!(ev["failure"], "authentication", "{ev}");

    // Sonda que passa nao pode carregar falha nenhuma.
    executavel(
        &c.bin().join("ssh"),
        "#!/bin/sh\nprintf 'aarch64\\nLinux 6.6\\ngdbserver=\\npython3=\\nrsync=\\n'\n",
    );
    c.ok("remote.probe", json!({ "name": "pi" }));
    let ok = c.evento("event.remote.probed");
    assert_eq!(ok["success"], true);
    assert!(ok.get("failure").is_none(), "{ok}");
}

/// `remote.command { kind: copyId }` (`0.133.0`): a linha do `ssh-copy-id`
/// composta pelo core, com o `-p`/`-i` do perfil. PURA — nada roda aqui, e a
/// UI so' executa depois de mostrar a linha e receber um gesto explicito.
#[test]
#[cfg(unix)]
fn copy_id_composes_the_line_from_the_profile_and_runs_nothing() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("copy-id");
    c.salvar_pi();
    let registro = c.bin().parent().unwrap().join("copyid.argv");
    // Se alguem rodar `ssh` ou `ssh-copy-id` por engano, este arquivo aparece.
    for nome in ["ssh", "ssh-copy-id"] {
        executavel(
            &c.bin().join(nome),
            &format!("#!/bin/sh\nprintf 'rodou' >> {}\n", registro.display()),
        );
    }
    let fora = c.ok("remote.command", json!({ "name": "pi", "kind": "copyId" }));
    assert_eq!(
        fora["command"],
        "ssh-copy-id -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42"
    );
    assert_eq!(fora["name"], "Copiar chave para pi");
    assert!(fora["remoteTarget"].is_null(), "{fora}");
    // A procedencia tem de dizer de quem e' a chave.
    let origem = fora["source"].as_array().unwrap();
    assert!(
        origem.iter().any(|s| s.as_str().unwrap().contains("SUA")),
        "{fora}"
    );
    assert!(
        !registro.exists(),
        "compor a linha NAO pode executar processo nenhum"
    );
}

/// O deploy: `rsync -az --delete -e 'ssh -p 2222 -i <chave>' <root>/build
/// pi@host:~/kinein/<projeto>/`; sem rsync, `scp -P 2222 -i <chave> -r`.
/// Origem inexistente e' recusa sincrona.
#[test]
#[cfg(unix)]
fn deploy_prefers_rsync_and_falls_back_to_scp() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("deploy");
    c.salvar_pi();
    let sem_nada = c.rpc("remote.deploy", json!({ "name": "pi" }));
    assert!(
        sem_nada
            .error
            .unwrap()
            .message
            .contains("nem `rsync` nem `scp`")
    );

    let registro = c.bin().join("deploy.args");
    executavel(
        &c.bin().join("rsync"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{r}'\necho enviado\n",
            r = registro.display()
        ),
    );
    let aceito = c.ok("remote.deploy", json!({ "name": "pi" }));
    assert!(
        aceito["command"]
            .as_str()
            .unwrap()
            .contains("rsync -az --delete -e ssh -p 2222 -i /home/u/.ssh/pi")
    );
    let ev = c.evento("event.remote.deployed");
    assert_eq!(ev["success"], true, "{ev}");
    assert_eq!(ev["source"], c.root.join("build").display().to_string());
    assert_eq!(ev["dest"], "~/kinein/ws");
    let args: Vec<String> = std::fs::read_to_string(&registro)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(args[3], "ssh -p 2222 -i /home/u/.ssh/pi");
    assert_eq!(args[5], "pi@192.168.0.42:~/kinein/ws/");

    let ausente = c.rpc(
        "remote.deploy",
        json!({ "name": "pi", "source": "nao-existe" }),
    );
    assert!(ausente.error.unwrap().message.contains("compile antes"));

    // Sem rsync: scp, com a pasta de destino dada.
    std::fs::remove_file(c.bin().join("rsync")).unwrap();
    executavel(
        &c.bin().join("scp"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{r}'\necho 'ssh: connect to host 192.168.0.42 port 2222: Connection timed out' >&2\nexit 1\n",
            r = registro.display()
        ),
    );
    c.ok("remote.deploy", json!({ "name": "pi", "dest": "/opt/app" }));
    let ev = c.evento("event.remote.deployed");
    assert_eq!(ev["success"], false);
    assert!(ev["error"].as_str().unwrap().contains("5 s"), "{ev}");
    let args: Vec<String> = std::fs::read_to_string(&registro)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(
        args,
        [
            "-P",
            "2222",
            "-i",
            "/home/u/.ssh/pi",
            "-r",
            c.root.join("build").display().to_string().as_str(),
            "pi@192.168.0.42:/opt/app/"
        ]
    );
}

/// `remote.command` e' puro: run, gdbserver (com `remoteTarget` para o kit),
/// debugpy e shell; programa relativo vai para dentro do deployDir.
#[test]
fn remote_command_composes_the_ssh_line_and_the_kit_target() {
    let mut c = cenario("command");
    c.salvar_pi();
    let run = c.ok(
        "remote.command",
        json!({ "name": "pi", "kind": "run", "program": "app" }),
    );
    assert_eq!(
        run["command"],
        "ssh -tt -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42 '~/kinein/ws/app'"
    );
    assert_eq!(run["name"], "Rodar em pi");
    assert!(run.get("remoteTarget").is_none());

    let gdb = c.ok(
        "remote.command",
        json!({ "name": "pi", "kind": "debugServer", "program": "/opt/app" }),
    );
    assert_eq!(
        gdb["command"],
        "ssh -tt -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42 'gdbserver :2345 /opt/app'"
    );
    assert_eq!(gdb["remoteTarget"], "192.168.0.42:2345");

    let py = c.ok(
        "remote.command",
        json!({ "name": "pi", "kind": "debugpy", "program": "main.py", "port": 6000 }),
    );
    assert_eq!(
        py["command"],
        "ssh -tt -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42 'python3 -m debugpy --listen 0.0.0.0:6000 --wait-for-client ~/kinein/ws/main.py'"
    );
    assert_eq!(py["remoteTarget"], "192.168.0.42:6000");

    let shell = c.ok("remote.command", json!({ "name": "pi", "kind": "shell" }));
    assert_eq!(
        shell["command"],
        "ssh -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42"
    );
    assert_eq!(shell["name"], "Shell em pi");

    let sem_programa = c.ok("remote.command", json!({ "name": "pi", "kind": "run" }));
    assert!(
        sem_programa["command"]
            .as_str()
            .unwrap()
            .contains("<binario>")
    );
    assert!(
        sem_programa["source"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("edite"))
    );
}

/// Descoberta e explicacao (`0.132.0`, fatia R0.5): um `~/.ssh/config` falso e
/// um `ssh` falso que responde como `ssh -G`. Sem rede e sem alvo salvo.
fn cenario_descoberta(nome: &str) -> (crate::Core, PathBuf) {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-remote-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("home/.ssh")).unwrap();
    let base = base.canonicalize().unwrap();
    let core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ))
    .with_home(base.join("home"));
    (core, base)
}

#[test]
fn discover_lists_the_machines_aliases_without_a_workspace_open() {
    let (mut core, base) = cenario_descoberta("discover");
    std::fs::write(
        base.join("home/.ssh/config"),
        "Host *\n  ServerAliveInterval 30\nHost pi\n  HostName 192.168.0.42\nHost bancada\n",
    )
    .unwrap();
    // De proposito SEM workspace.open: a pergunta e' sobre a maquina, e o
    // primeiro uso precisa dela antes de existir alvo salvo.
    let r = core
        .handle_request(&JsonRpcRequest::new(
            7_i64,
            "remote.discover",
            Some(json!({})),
        ))
        .response()
        .clone();
    assert!(r.error.is_none(), "{:?}", r.error);
    let fora = r.result.unwrap();
    let nomes: Vec<&str> = fora["aliases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert_eq!(nomes, ["bancada", "pi"]);
    assert_eq!(fora["aliases"][0]["source"], "~/.ssh/config");
    assert_eq!(fora["sources"][0], "~/.ssh/config");

    let com_params = core
        .handle_request(&JsonRpcRequest::new(
            8_i64,
            "remote.discover",
            Some(json!({ "host": "pi" })),
        ))
        .response()
        .clone();
    assert_eq!(
        com_params.error.unwrap().code,
        JsonRpcErrorCode::InvalidParams
    );
}

#[test]
#[cfg(unix)]
fn resolve_asks_openssh_for_the_effective_summary_and_hides_the_proxy_command() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut core, base) = cenario_descoberta("resolve");
    let registro = base.join("ssh.argv");
    executavel(
        &base.join("bin/ssh"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > {}\n\
             echo 'user pi'\necho 'hostname 192.168.0.42'\necho 'port 2222'\n\
             echo 'identityfile ~/.ssh/pi'\n\
             echo 'proxycommand /usr/bin/secreto --token ABC %h'\n\
             echo 'sendenv LANG'\n",
            registro.display()
        ),
    );
    let r = core
        .handle_request(&JsonRpcRequest::new(
            9_i64,
            "remote.resolve",
            Some(json!({ "host": "pi" })),
        ))
        .response()
        .clone();
    assert!(r.error.is_none(), "{:?}", r.error);
    let fora = r.result.unwrap();
    // `ssh -G` EXPLICA sem conectar: nenhum outro argumento entra na linha.
    assert_eq!(
        std::fs::read_to_string(&registro).unwrap().trim(),
        "-G pi",
        "a linha do ssh mudou"
    );
    assert_eq!(fora["host"], "pi");
    assert_eq!(fora["user"], "pi");
    assert_eq!(fora["hostName"], "192.168.0.42");
    assert_eq!(fora["port"], 2222);
    assert_eq!(fora["identities"][0], "~/.ssh/pi");
    assert_eq!(fora["proxyCommand"], true);
    let texto = serde_json::to_string(&fora).unwrap();
    assert!(
        !texto.contains("ABC") && !texto.contains("secreto") && !texto.contains("sendenv"),
        "{texto}"
    );
}

#[test]
#[cfg(unix)]
fn resolve_refuses_a_host_that_ssh_would_read_as_an_option_before_spawning_anything() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut core, base) = cenario_descoberta("resolve-opcao");
    let registro = base.join("ssh.argv");
    executavel(
        &base.join("bin/ssh"),
        &format!("#!/bin/sh\nprintf 'rodou' > {}\n", registro.display()),
    );
    let r = core
        .handle_request(&JsonRpcRequest::new(
            10_i64,
            "remote.resolve",
            Some(json!({ "host": "-oProxyCommand=id" })),
        ))
        .response()
        .clone();
    let erro = r.error.unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains("opcao"), "{}", erro.message);
    assert!(
        !registro.exists(),
        "recusa tem de vir ANTES de rodar processo"
    );
}

#[test]
#[cfg(unix)]
fn resolve_repeats_what_ssh_complained_instead_of_inventing_a_summary() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut core, base) = cenario_descoberta("resolve-recusa");
    executavel(
        &base.join("bin/ssh"),
        "#!/bin/sh\necho 'Bad configuration option: naoexiste' >&2\nexit 255\n",
    );
    let r = core
        .handle_request(&JsonRpcRequest::new(
            11_i64,
            "remote.resolve",
            Some(json!({ "host": "pi" })),
        ))
        .response()
        .clone();
    let erro = r.error.unwrap();
    assert!(
        erro.message.contains("Bad configuration option"),
        "{}",
        erro.message
    );
}

/// `remote.parseCommand` (`0.134.0`): a linha colada vira proposta, e NADA e'
/// executado nem salvo. O "configurar servidor" sem formulario da R0.5.
#[test]
fn parse_command_proposes_a_profile_without_running_or_saving_anything() {
    let mut c = cenario("parse");
    let fora = c.ok(
        "remote.parseCommand",
        json!({ "command": "ssh -p 2222 -i ~/.ssh/pi pi@192.168.0.42" }),
    );
    assert_eq!(fora["target"]["host"], "192.168.0.42");
    assert_eq!(fora["target"]["user"], "pi");
    assert_eq!(fora["target"]["port"], 2222);
    assert_eq!(fora["target"]["identityFile"], "~/.ssh/pi");
    assert!(
        fora["source"].as_array().unwrap().len() >= 3,
        "a procedencia de cada campo tem de ser dita: {fora}"
    );
    // Propor NAO e' salvar: o catalogo continua vazio ate' o `remote.save`.
    let lista = c.ok("remote.list", json!({}));
    assert!(lista["targets"].as_array().unwrap().is_empty(), "{lista}");

    // O que um shell interpretaria e' recusado, nao limpo.
    let r = c.rpc(
        "remote.parseCommand",
        json!({ "command": "ssh pi@h; rm -rf /" }),
    );
    assert_eq!(
        r.error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
    assert!(r.error.unwrap().message.contains("shell"));

    // O que o perfil nao reproduz e' DITO, nao descartado em silencio.
    let r = c.rpc(
        "remote.parseCommand",
        json!({ "command": "ssh -J bastion pi@h" }),
    );
    assert!(r.error.unwrap().message.contains("~/.ssh/config"));
}
