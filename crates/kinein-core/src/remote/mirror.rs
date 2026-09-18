//! O workspace ESPELHADO (P6 fatia 2, `roadmaps/42` §P6, 2026-09-18).
//!
//! A pasta remota vira um espelho local por `rsync`, e a IDE abre o espelho
//! como workspace comum — `fs.*`, indice, busca, git e LSP nao mudam; o que
//! muda e' a SINCRONIA, que e' o `rsync` do sistema nos dois sentidos.
//!
//! ```text
//! espelho    <cache>/remote/<alvo>/<hash do path>/<basename>
//! marcador   <espelho>/.kinein/remote-mirror.json  { schemaVersion, name, host, path }
//! pull       rsync -az -i --exclude .kinein -e 'ssh …' [user@]host:<path>/ <espelho>/
//! push       rsync -az -i --exclude .kinein -e 'ssh …' <espelho>/ [user@]host:<path>/
//! por arquivo  o mesmo, com <espelho>/<rel> e <path>/<rel> — o `fs.write` num
//!            espelho empurra o arquivo sozinho
//! nunca --delete: apagar e' gesto explicito, fora desta fatia
//! ```
//!
//! O `-i` (itemize) do `rsync` diz o que foi transferido, uma linha por
//! caminho: e' o `changed` do evento, lido sem adivinhar.

use std::{
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use kinein_protocol::{RemoteMirror, RemoteSyncDirection, RemoteTarget};
use serde::{Deserialize, Serialize};

use super::destination;

const SCHEMA_VERSION: u32 = 1;
/// O que nunca sincroniza: o estado da IDE (dos dois lados).
pub const EXCLUDES: [&str; 1] = [".kinein"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MirrorFile {
    schema_version: u32,
    name: String,
    host: String,
    path: String,
}

/// A raiz de todos os espelhos: `<home>/.cache/kinein-vectis/remote`.
#[must_use]
pub fn cache_root(home: &Path) -> PathBuf {
    home.join(".cache/kinein-vectis/remote")
}

/// Onde o espelho de `path` no alvo mora: `<cache>/<alvo>/<hash>/<basename>`.
/// O basename e' o nome do workspace que a IDE mostra.
#[must_use]
pub fn mirror_root(home: &Path, target: &RemoteTarget, path: &str) -> PathBuf {
    let path = path.trim().trim_end_matches('/');
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let base = path
        .rsplit('/')
        .next()
        .filter(|b| !b.is_empty())
        .unwrap_or("raiz");
    cache_root(home)
        .join(&target.name)
        .join(format!("{:016x}", hasher.finish()))
        .join(base)
}

fn marker_path(mirror: &Path) -> PathBuf {
    mirror.join(".kinein").join("remote-mirror.json")
}

/// Grava o marcador do espelho (cria a pasta).
///
/// # Errors
/// Disco.
pub fn write_marker(mirror: &Path, target: &RemoteTarget, path: &str) -> Result<(), String> {
    let marcador = marker_path(mirror);
    if let Some(parent) = marcador.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("falha criando {}: {e}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(&MirrorFile {
        schema_version: SCHEMA_VERSION,
        name: target.name.clone(),
        host: target.host.clone(),
        path: path.trim().trim_end_matches('/').to_owned(),
    })
    .map_err(|e| e.to_string())?;
    fs::write(&marcador, body).map_err(|e| format!("falha gravando {}: {e}", marcador.display()))
}

/// O espelho que `root` e', se for um (marcador legivel e do schema atual).
#[must_use]
pub fn read_marker(root: &Path) -> Option<RemoteMirror> {
    let body = fs::read_to_string(marker_path(root)).ok()?;
    let file: MirrorFile = serde_json::from_str(&body).ok()?;
    (file.schema_version == SCHEMA_VERSION).then(|| RemoteMirror {
        name: file.name,
        host: file.host,
        path: file.path,
        mirror_root: root.display().to_string(),
    })
}

/// `ssh [-p] [-i] -o ControlMaster=auto -o ControlPath=<cache>/ssh-%C -o
/// ControlPersist=60` — uma conexao so' para o push arquivo a arquivo.
#[must_use]
pub fn ssh_transport(target: &RemoteTarget, control_dir: &Path) -> String {
    let mut e = vec!["ssh".to_owned()];
    if let Some(port) = target.port {
        e.push(format!("-p {port}"));
    }
    if let Some(key) = &target.identity_file {
        e.push(format!("-i {key}"));
    }
    e.push("-o ControlMaster=auto".to_owned());
    e.push(format!("-o ControlPath={}/ssh-%C", control_dir.display()));
    e.push("-o ControlPersist=60".to_owned());
    e.join(" ")
}

/// Uma linha de `rsync` para `rel` (`""` = a arvore inteira), no sentido dado.
#[must_use]
pub fn rsync_args(
    target: &RemoteTarget,
    remote_path: &str,
    mirror: &Path,
    rel: &str,
    direction: RemoteSyncDirection,
    transport: &str,
) -> Vec<String> {
    let rel = rel.trim_matches('/');
    let remoto = if rel.is_empty() {
        format!(
            "{}:{}/",
            destination(target),
            remote_path.trim_end_matches('/')
        )
    } else {
        format!(
            "{}:{}/{rel}",
            destination(target),
            remote_path.trim_end_matches('/')
        )
    };
    let local = if rel.is_empty() {
        format!("{}/", mirror.display())
    } else {
        mirror.join(rel).display().to_string()
    };
    let mut args = vec!["-az".to_owned(), "-i".to_owned()];
    for exclude in EXCLUDES {
        args.push("--exclude".to_owned());
        args.push(exclude.to_owned());
    }
    args.push("-e".to_owned());
    args.push(transport.to_owned());
    match direction {
        RemoteSyncDirection::Pull => {
            args.push(remoto);
            args.push(local);
        }
        RemoteSyncDirection::Push => {
            args.push(local);
            args.push(remoto);
        }
    }
    args
}

/// Os caminhos transferidos, da saida `-i` do rsync (`<f+++++++++ src/a.c`,
/// `>f.st...... b.py`, `cd+++++++++ dir/`). Linhas que nao sao itemizacao
/// (mensagens, vazio) ficam de fora.
#[must_use]
pub fn parse_itemized(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|linha| {
            let (flags, caminho) = linha.split_once(' ')?;
            let caminho = caminho.trim();
            let itemizado = flags.len() >= 11
                && matches!(flags.as_bytes()[0], b'<' | b'>' | b'c' | b'h' | b'.' | b'*')
                && matches!(flags.as_bytes()[1], b'f' | b'd' | b'L' | b'D' | b'S');
            (itemizado && !caminho.is_empty() && caminho != "./" && caminho != ".")
                .then(|| caminho.trim_end_matches('/').to_owned())
        })
        .collect()
}

/// `rel` e' seguro para sincronizar: relativo, sem `..`, e nao e' o `.kinein`.
#[must_use]
pub fn safe_relative(rel: &str) -> bool {
    let rel = rel.trim_matches('/');
    !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.split('/').any(|p| p == "..")
        && rel != ".kinein"
        && !rel.starts_with(".kinein/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pi() -> RemoteTarget {
        RemoteTarget {
            name: "pi".to_owned(),
            host: "192.168.0.42".to_owned(),
            user: Some("pi".to_owned()),
            port: Some(2222),
            identity_file: None,
            deploy_dir: None,
        }
    }

    #[test]
    fn the_mirror_lives_under_the_cache_named_after_the_remote_folder() {
        let home = Path::new("/home/u");
        let m = mirror_root(home, &pi(), "/home/pi/projetos/sensor/");
        assert!(m.starts_with("/home/u/.cache/kinein-vectis/remote/pi/"));
        assert_eq!(m.file_name().unwrap(), "sensor");
        assert_eq!(m, mirror_root(home, &pi(), "/home/pi/projetos/sensor"));
        assert_ne!(m, mirror_root(home, &pi(), "/home/pi/outro/sensor"));
        assert_eq!(mirror_root(home, &pi(), "/").file_name().unwrap(), "raiz");
    }

    #[test]
    fn marker_round_trips_and_is_read_only_from_its_schema() {
        let dir = std::env::temp_dir()
            .join("kinein-remote-mirror")
            .join(format!("{}-marcador", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert!(read_marker(&dir).is_none());
        write_marker(&dir, &pi(), "/home/pi/sensor/").unwrap();
        let m = read_marker(&dir).unwrap();
        assert_eq!(
            (m.name.as_str(), m.host.as_str(), m.path.as_str()),
            ("pi", "192.168.0.42", "/home/pi/sensor")
        );
        assert_eq!(m.mirror_root, dir.display().to_string());
        fs::write(
            marker_path(&dir),
            r#"{"schemaVersion": 9, "name": "x", "host": "h", "path": "/p"}"#,
        )
        .unwrap();
        assert!(read_marker(&dir).is_none());
    }

    #[test]
    fn rsync_lines_carry_excludes_transport_and_direction() {
        let transport = ssh_transport(&pi(), Path::new("/home/u/.cache/kinein-vectis/remote"));
        assert_eq!(
            transport,
            "ssh -p 2222 -o ControlMaster=auto -o ControlPath=/home/u/.cache/kinein-vectis/remote/ssh-%C -o ControlPersist=60"
        );
        let m = Path::new("/home/u/.cache/kinein-vectis/remote/pi/abc/sensor");
        let pull = rsync_args(
            &pi(),
            "/home/pi/sensor",
            m,
            "",
            RemoteSyncDirection::Pull,
            &transport,
        );
        assert_eq!(&pull[..5], ["-az", "-i", "--exclude", ".kinein", "-e"]);
        assert_eq!(pull[6], "pi@192.168.0.42:/home/pi/sensor/");
        assert_eq!(
            pull[7],
            "/home/u/.cache/kinein-vectis/remote/pi/abc/sensor/"
        );
        let push = rsync_args(
            &pi(),
            "/home/pi/sensor/",
            m,
            "src/main.c",
            RemoteSyncDirection::Push,
            &transport,
        );
        assert_eq!(
            push[6],
            "/home/u/.cache/kinein-vectis/remote/pi/abc/sensor/src/main.c"
        );
        assert_eq!(push[7], "pi@192.168.0.42:/home/pi/sensor/src/main.c");
    }

    #[test]
    fn itemized_output_becomes_the_changed_list() {
        let raw = "sending incremental file list\n.d..t...... ./\n<f+++++++++ src/main.c\n>f.st...... README.md\ncd+++++++++ include/\n\nsent 1,234 bytes  received 35 bytes\n";
        assert_eq!(parse_itemized(raw), ["src/main.c", "README.md", "include"]);
        assert!(parse_itemized("").is_empty());
        assert!(safe_relative("src/main.c") && safe_relative("/src/"));
        assert!(
            !safe_relative("")
                && !safe_relative("../x")
                && !safe_relative(".kinein/remotes.json")
                && !safe_relative("a/../b")
        );
    }
}
