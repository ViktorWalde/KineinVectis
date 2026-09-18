//! O workspace ESPELHADO (P6 fatia 2, `roadmaps/42` §P6, 2026-09-18) por
//! despacho, com um `rsync` FALSO que ecoa argv e "transfere" copiando o que
//! puder: `remote.open` puxa e grava o marcador; `workspace.open` do espelho
//! devolve `remote`; `fs.write` no espelho EMPURRA o arquivo sozinho;
//! `remote.sync` com e sem `paths` (e a recusa de `..`/`.kinein`);
//! `remote.status` fora e dentro de um espelho.

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

fn executavel(caminho: &Path, corpo: &str) {
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, corpo).unwrap();
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

struct Cenario {
    core: crate::Core,
    base: PathBuf,
    events: mpsc::Receiver<JsonRpcRequest>,
}

fn cenario(nome: &str) -> Cenario {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-mirror-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("bin")).unwrap();
    std::fs::create_dir_all(base.join("ws")).unwrap();
    std::fs::create_dir_all(base.join("home")).unwrap();
    // A "Pi": uma pasta local que o rsync falso le e escreve.
    std::fs::create_dir_all(base.join("pi/sensor/src")).unwrap();
    std::fs::write(
        base.join("pi/sensor/src/main.c"),
        "int main(void) { return 0; }\n",
    )
    .unwrap();
    std::fs::write(base.join("pi/sensor/CMakeLists.txt"), "project(sensor C)\n").unwrap();
    let base = base.canonicalize().unwrap();
    let (sender, receiver) = mpsc::channel();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        base.join("bin"),
    ))
    .with_home(base.join("home"));
    core.enable_lsp(sender);
    let mut c = Cenario {
        core,
        base: base.clone(),
        events: receiver,
    };
    let r = c.rpc(
        "workspace.open",
        json!({ "path": base.join("ws").to_str().unwrap() }),
    );
    assert!(r.error.is_none(), "{:?}", r.error);
    c.ok(
        "remote.save",
        json!({ "target": { "name": "pi", "host": "192.168.0.42", "user": "pi", "port": 2222 } }),
    );
    c
}

impl Cenario {
    fn rpc(&mut self, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(3_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let r = self.rpc(method, params);
        assert!(r.error.is_none(), "{method}: {:?}", r.error);
        r.result.unwrap()
    }

    fn synced(&self) -> Value {
        let prazo = std::time::Instant::now() + Duration::from_secs(20);
        while std::time::Instant::now() < prazo {
            if let Ok(e) = self.events.recv_timeout(Duration::from_millis(50))
                && e.method == "event.remote.synced"
            {
                return e.params.unwrap();
            }
        }
        panic!("event.remote.synced nao chegou");
    }

    /// O rsync falso: registra argv, e copia `<origem>` para `<destino>`
    /// traduzindo `pi@192.168.0.42:/home/pi/...` para a pasta local da "Pi".
    /// Ecoa uma linha itemizada por caminho, como o `-i` faria.
    fn rsync_falso(&self) {
        let pi = self.base.join("pi").display().to_string();
        let registro = self.base.join("bin/rsync.args").display().to_string();
        executavel(
            &self.base.join("bin/rsync"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{registro}'\n\
                 eval src=\\${{$(($# - 1))}}; eval dst=\\${{$#}}\n\
                 src=$(printf '%s' \"$src\" | sed 's#^pi@192.168.0.42:/home/pi#{pi}#')\n\
                 dst=$(printf '%s' \"$dst\" | sed 's#^pi@192.168.0.42:/home/pi#{pi}#')\n\
                 case \"$src\" in */) mkdir -p \"$dst\"; cp -r \"$src\". \"$dst\"; (cd \"$src\" && find . -type f | sed 's#^\\./#<f+++++++++ #');;\n\
                 *) mkdir -p \"$(dirname \"$dst\")\"; cp \"$src\" \"$dst\"; echo \"<f+++++++++ $(basename \"$src\")\";; esac\n"
            ),
        );
    }
}

/// Abre `/home/pi/sensor` como espelho e devolve a raiz dele.
fn abrir_espelho(c: &mut Cenario) -> PathBuf {
    c.rsync_falso();
    let aceito = c.ok(
        "remote.open",
        json!({ "name": "pi", "path": "/home/pi/sensor/" }),
    );
    let espelho = PathBuf::from(aceito["mirror"].as_str().unwrap());
    let ev = c.synced();
    assert_eq!(ev["success"], true, "{ev}");
    let aberto = c.ok(
        "workspace.open",
        json!({ "path": espelho.to_str().unwrap() }),
    );
    assert_eq!(aberto["remote"]["name"], "pi");
    espelho
}

#[test]
#[cfg(unix)]
fn a_remote_folder_becomes_a_local_mirror_with_a_marker() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("abrir");

    // Fora de um espelho: status vazio; sync recusado; sem rsync, open recusado.
    assert!(c.ok("remote.status", json!({})).get("mirror").is_none());
    assert!(
        c.rpc("remote.sync", json!({ "direction": "pull" }))
            .error
            .unwrap()
            .message
            .contains("nao e' um espelho")
    );
    assert!(
        c.rpc(
            "remote.open",
            json!({ "name": "pi", "path": "/home/pi/sensor" })
        )
        .error
        .unwrap()
        .message
        .contains("rsync")
    );
    c.rsync_falso();
    assert!(
        c.rpc(
            "remote.open",
            json!({ "name": "outro", "path": "/home/pi/sensor" })
        )
        .error
        .is_some()
    );
    assert!(
        c.rpc("remote.open", json!({ "name": "pi", "path": " " }))
            .error
            .is_some()
    );

    // Abrir: o job puxa a arvore para o cache e grava o marcador.
    let aceito = c.ok(
        "remote.open",
        json!({ "name": "pi", "path": "/home/pi/sensor/" }),
    );
    let espelho = PathBuf::from(aceito["mirror"].as_str().unwrap());
    assert!(
        espelho.starts_with(c.base.join("home/.cache/kinein-vectis/remote/pi")),
        "{}",
        espelho.display()
    );
    assert_eq!(espelho.file_name().unwrap(), "sensor");
    let comando = aceito["command"].as_str().unwrap();
    assert!(
        comando.contains(
            "-az -i --exclude .kinein -e ssh -p 2222 -o ControlMaster=auto -o ControlPath="
        ),
        "{comando}"
    );
    assert!(
        comando.contains("pi@192.168.0.42:/home/pi/sensor/ "),
        "{comando}"
    );
    let ev = c.synced();
    assert_eq!(ev["success"], true, "{ev}");
    assert_eq!(ev["direction"], "pull");
    assert_eq!(ev["name"], "pi");
    assert_eq!(ev["mirror"], espelho.display().to_string());
    let changed: Vec<&str> = ev["changed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(
        changed.contains(&"src/main.c") && changed.contains(&"CMakeLists.txt"),
        "{changed:?}"
    );
    assert!(espelho.join("src/main.c").is_file());
    assert!(espelho.join(".kinein/remote-mirror.json").is_file());

    // O workspace.open do espelho diz de quem e'; remote.status idem.
    let aberto = c.ok(
        "workspace.open",
        json!({ "path": espelho.to_str().unwrap() }),
    );
    assert_eq!(aberto["remote"]["name"], "pi");
    assert_eq!(aberto["remote"]["path"], "/home/pi/sensor");
    assert_eq!(aberto["remote"]["host"], "192.168.0.42");
    assert_eq!(aberto["kind"], "cmake");
    assert_eq!(
        c.ok("remote.status", json!({}))["mirror"]["mirrorRoot"],
        espelho.display().to_string()
    );
    // O alvo foi copiado para o catalogo do espelho (usuario/porta/chave).
    assert_eq!(c.ok("remote.list", json!({}))["targets"][0]["port"], 2222);
}

#[test]
#[cfg(unix)]
fn saving_in_a_mirror_pushes_and_sync_moves_both_ways() {
    let _serial = super::EXECUTAVEIS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut c = cenario("sync");
    let espelho = abrir_espelho(&mut c);

    // Salvar no espelho EMPURRA so' esse arquivo.
    let registro = c.base.join("bin/rsync.args");
    std::fs::write(&registro, "").unwrap();
    c.ok(
        "fs.write",
        json!({ "path": espelho.join("src/main.c").to_str().unwrap(), "content": "int main(void) { return 1; }\n", "expectedContent": "int main(void) { return 0; }\n" }),
    );
    let ev = c.synced();
    assert_eq!(ev["direction"], "push", "{ev}");
    assert_eq!(ev["success"], true);
    assert_eq!(ev["changed"], json!(["main.c"]));
    let args = std::fs::read_to_string(&registro).unwrap();
    assert!(
        args.contains(&format!(
            "{}/src/main.c\npi@192.168.0.42:/home/pi/sensor/src/main.c\n",
            espelho.display()
        )),
        "{args}"
    );
    assert_eq!(
        std::fs::read_to_string(c.base.join("pi/sensor/src/main.c")).unwrap(),
        "int main(void) { return 1; }\n"
    );

    // remote.sync: paths invalidos recusados; push de uma pasta; pull da arvore.
    assert!(
        c.rpc(
            "remote.sync",
            json!({ "direction": "push", "paths": ["../x"] })
        )
        .error
        .is_some()
    );
    assert!(
        c.rpc(
            "remote.sync",
            json!({ "direction": "push", "paths": [".kinein/remotes.json"] })
        )
        .error
        .is_some()
    );
    std::fs::write(c.base.join("pi/sensor/NOVO.md"), "da pi\n").unwrap();
    c.ok("remote.sync", json!({ "direction": "pull" }));
    let ev = c.synced();
    assert_eq!(ev["direction"], "pull");
    assert!(
        ev["changed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "NOVO.md"),
        "{ev}"
    );
    assert!(espelho.join("NOVO.md").is_file());
    // O .kinein do espelho nunca vai para a Pi (o exclude esta' na linha; o
    // rsync falso copia tudo — o que se prova aqui e' a LINHA).
    c.ok(
        "remote.sync",
        json!({ "direction": "push", "paths": ["src/"] }),
    );
    let ev = c.synced();
    assert!(
        ev["command"]
            .as_str()
            .unwrap()
            .contains("--exclude .kinein")
    );
    assert!(
        ev["command"]
            .as_str()
            .unwrap()
            .ends_with("pi@192.168.0.42:/home/pi/sensor/src"),
        "{}",
        ev["command"]
    );

    // Falha do rsync: o evento diz.
    executavel(
        &c.base.join("bin/rsync"),
        "#!/bin/sh\necho 'rsync: connection unexpectedly closed' >&2\nexit 12\n",
    );
    c.ok("remote.sync", json!({ "direction": "pull" }));
    let ev = c.synced();
    assert_eq!(ev["success"], false);
    assert!(
        ev["error"]
            .as_str()
            .unwrap()
            .contains("connection unexpectedly closed"),
        "{ev}"
    );
}
