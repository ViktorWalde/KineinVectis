//! 16.4a (`roadmaps/59` §7.1.4): o Remoto contra um projeto que tenta
//! quebrar. Cada `ssh`/`rsync`/`ssh-keygen` falso deixa um registro se for
//! executado; o teste confere que o hostil nunca chega a ele, que o legitimo
//! com espaco chega inteiro, e que o `known_hosts` de um host conhecido nao
//! ganha uma segunda chave.

use serde_json::json;

use super::remote::{scenario, write_tool};

/// O `remotes.json` de um projeto clonado, escrito a mao: nenhum dos alvos
/// hostis chega a um processo, por nenhum dos caminhos; a lista continua
/// mostrando todos (o arquivo da pessoa nao perde nada).
#[test]
fn a_cloned_remotes_json_with_hostile_values_never_reaches_a_process() {
    let _serial = crate::serialize_executables();
    let mut c = scenario("guarda-hostil");
    let ran = c.bin().join("rodou");
    for tool in ["ssh", "rsync", "scp"] {
        write_tool(
            &c.bin().join(tool),
            &format!("#!/bin/sh\necho {tool} >> '{}'\n", ran.display()),
        );
    }
    let marker = c.bin().join("injetado");
    let m = marker.display();
    let hostile = json!({ "schemaVersion": 1, "targets": [
        { "name": "opcao", "host": format!("-oProxyCommand=touch {m}") },
        { "name": "comando", "host": format!("pi;touch {m}") },
        { "name": "subshell", "host": format!("$(touch {m})") },
        { "name": "usuario", "host": "pi.local", "user": format!("-oProxyCommand=touch {m}") },
        { "name": "chave", "host": "pi.local", "identityFile": format!("-oProxyCommand=touch {m}") },
        { "name": "linha", "host": "pi.local", "deployDir": "/opt/app\ntouch /tmp/x" }
    ]});
    std::fs::create_dir_all(c.root.join(".kinein")).unwrap();
    std::fs::write(c.root.join(".kinein/remotes.json"), hostile.to_string()).unwrap();

    let listed = c.ok("remote.list", json!({}))["targets"].clone();
    assert_eq!(listed.as_array().unwrap().len(), 6, "{listed}");
    for name in ["opcao", "comando", "subshell", "usuario", "chave", "linha"] {
        for (method, params) in [
            ("remote.probe", json!({ "name": name })),
            ("remote.deploy", json!({ "name": name })),
            (
                "remote.command",
                json!({ "name": name, "kind": "run", "program": "app" }),
            ),
            ("remote.command", json!({ "name": name, "kind": "shell" })),
            ("remote.hostKey", json!({ "name": name })),
        ] {
            let response = c.rpc(method, params);
            let error = response
                .error
                .unwrap_or_else(|| panic!("{method} aceitou `{name}`"));
            assert!(
                error.message.contains("não pode ser usado"),
                "{method} {name}: {}",
                error.message
            );
        }
    }
    // Salvar pelo painel recusa o mesmo valor.
    let saved = c.rpc(
        "remote.save",
        json!({ "target": { "name": "x", "host": "-oProxyCommand=id" } }),
    );
    assert!(saved.error.unwrap().message.contains("não pode ir ao ssh"));
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !ran.exists(),
        "um processo ran: {:?}",
        std::fs::read_to_string(&ran)
    );
    assert!(!marker.exists());
}

/// O legitimo com espaco chega inteiro na linha do `sh -c` que o
/// `remote.command` arma: a chave e a pasta de deploy entre aspas.
#[test]
fn the_shell_line_quotes_a_key_and_a_folder_with_spaces() {
    let mut c = scenario("guarda-espaco");
    c.ok(
        "remote.save",
        json!({ "target": { "name": "pi", "host": "pi.local", "user": "pi", "port": 2222,
                            "identityFile": "/home/u/My Keys/pi", "deployDir": "/opt/minha app" } }),
    );
    let line = c.ok(
        "remote.command",
        json!({ "name": "pi", "kind": "run", "program": "app" }),
    );
    let command = line["command"].as_str().unwrap();
    assert!(
        command.starts_with("ssh -tt -p 2222 -i '/home/u/My Keys/pi' pi@pi.local "),
        "{command}"
    );
    assert!(command.contains("/opt/minha app/app"), "{command}");
}

/// O deploy com os mesmos valores: a chave entre aspas no `-e` do `rsync`, a
/// pasta entre aspas no `mkdir` remoto e no destino.
#[test]
#[cfg_attr(
    windows,
    ignore = "W9: o Remote no Windows (OpenSSH do sistema, sem rsync) e' da W9 (60 §3.3)"
)]
fn deploy_quotes_the_key_and_the_folder_with_spaces() {
    let _serial = crate::serialize_executables();
    let mut c = scenario("guarda-deploy");
    c.ok(
        "remote.save",
        json!({ "target": { "name": "pi", "host": "pi.local", "user": "pi", "port": 2222,
                            "identityFile": "/home/u/My Keys/pi", "deployDir": "/opt/minha app" } }),
    );
    let ssh_args = c.bin().join("ssh.args");
    let rsync_args = c.bin().join("rsync.args");
    write_tool(
        &c.bin().join("ssh"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            ssh_args.display()
        ),
    );
    write_tool(
        &c.bin().join("rsync"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n",
            rsync_args.display()
        ),
    );
    c.ok("remote.deploy", json!({ "name": "pi" }));
    let ev = c.event("event.remote.deployed");
    assert_eq!(ev["success"], true, "{ev}");
    let read_lines = |path: &std::path::Path| -> Vec<String> {
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
    };
    let mkdir = read_lines(&ssh_args);
    assert_eq!(
        mkdir.last().unwrap(),
        "mkdir -p '/opt/minha app'",
        "{mkdir:?}"
    );
    assert!(
        mkdir.contains(&"/home/u/My Keys/pi".to_owned()),
        "{mkdir:?}"
    );
    let rsync = read_lines(&rsync_args);
    assert_eq!(rsync[3], "ssh -p 2222 -i '/home/u/My Keys/pi'", "{rsync:?}");
    assert_eq!(rsync[5], "pi@pi.local:'/opt/minha app'/", "{rsync:?}");
}

/// Confiar e' so' o primeiro contato: com uma chave JA' gravada para o host,
/// o `trustHost` recusa e o `known_hosts` fica byte a byte igual; sem ela,
/// grava a linha vista.
#[test]
#[cfg_attr(
    windows,
    ignore = "W9: o Remote no Windows (OpenSSH do sistema, sem rsync) e' da W9 (60 §3.3)"
)]
fn trust_host_never_adds_a_second_key_for_a_known_host() {
    let _serial = crate::serialize_executables();
    let mut c = scenario("guarda-hostkey");
    let known = c.bin().join("known_hosts");
    write_tool(
        &c.bin().join("ssh"),
        &format!(
            "#!/bin/sh\nprintf 'hostname pi.local\\nport 22\\nuserknownhostsfile {}\\nhashknownhosts no\\n'\n",
            known.display()
        ),
    );
    write_tool(
        &c.bin().join("ssh-keyscan"),
        "#!/bin/sh\necho 'pi.local ssh-ed25519 AAAANOVA'\n",
    );
    // `-l -f -`: a impressao; `-F host -f arquivo`: as linhas do host.
    write_tool(
        &c.bin().join("ssh-keygen"),
        "#!/bin/sh\nif [ \"$1\" = -l ]; then cat >/dev/null; echo '256 SHA256:nova pi.local (ED25519)'; exit 0; fi\nif [ \"$1\" = -F ]; then grep \"^$2 \" \"$4\"; exit $?; fi\nexit 2\n",
    );
    c.ok(
        "remote.save",
        json!({ "target": { "name": "pi", "host": "pi.local" } }),
    );
    let before = "pi.local ssh-ed25519 AAAAANTIGA\n";
    std::fs::write(&known, before).unwrap();
    let key = c.ok("remote.hostKey", json!({ "name": "pi" }));
    assert_eq!(key["keys"][0]["fingerprint"], "SHA256:nova", "{key}");
    let refusal = c.rpc(
        "remote.trustHost",
        json!({ "name": "pi", "fingerprints": ["SHA256:nova"] }),
    );
    let error = refusal.error.expect("acrescentou uma segunda chave");
    assert!(
        error.message.contains("ssh-keygen -R pi.local"),
        "{}",
        error.message
    );
    assert_eq!(std::fs::read_to_string(&known).unwrap(), before);

    std::fs::write(&known, "").unwrap();
    let recorded = c.ok(
        "remote.trustHost",
        json!({ "name": "pi", "fingerprints": ["SHA256:nova"] }),
    );
    assert_eq!(recorded["recorded"], 1, "{recorded}");
    assert_eq!(
        std::fs::read_to_string(&known).unwrap(),
        "pi.local ssh-ed25519 AAAANOVA\n"
    );
}

/// Um marcador de espelho fora do cache da IDE (um repositorio clonado que o
/// traz) nao faz do projeto um espelho: nada sincroniza, e gravar um arquivo
/// nao chama o `rsync`.
#[test]
fn a_mirror_marker_outside_the_cache_is_not_a_mirror() {
    let _serial = crate::serialize_executables();
    let mut c = scenario("guarda-espelho");
    let ran = c.bin().join("rodou");
    write_tool(
        &c.bin().join("rsync"),
        &format!("#!/bin/sh\necho rsync >> '{}'\n", ran.display()),
    );
    std::fs::create_dir_all(c.root.join(".kinein")).unwrap();
    std::fs::write(
        c.root.join(".kinein/remote-mirror.json"),
        json!({ "schemaVersion": 1, "name": "pi", "host": "atacante.example", "path": "/srv" })
            .to_string(),
    )
    .unwrap();
    let synced = c.rpc("remote.sync", json!({ "direction": "push" }));
    assert!(synced.error.unwrap().message.contains("espelho"));
    let file = c.root.join("main.c");
    std::fs::write(&file, "int main(void){return 0;}\n").unwrap();
    c.ok(
        "fs.write",
        json!({ "path": file.to_str().unwrap(), "content": "int main(void){return 1;}\n",
                "expectedContent": "int main(void){return 0;}\n" }),
    );
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(!ran.exists(), "o rsync ran para o host do marcador");
}
