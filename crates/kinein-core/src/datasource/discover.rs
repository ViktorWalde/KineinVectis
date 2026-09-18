//! `datasource.discover`: o que responde NESTA maquina (`0.124.0`).
//!
//! Tres perguntas, cada uma MEDIDA e nunca deduzida:
//!
//! 1. ha' um servidor local? — o socket do `PostgreSQL` existe no diretorio
//!    de sempre, ou a porta padrao aceita conexao no loopback (`5432`,
//!    `27017`), com 200 ms de tolerancia;
//! 2. ha' um container de banco? — o motor lista (`ps -a`) e a imagem diz o
//!    que e' (`postgres`/`timescale`/`mongo`); a porta publicada no loopback
//!    e' o que o perfil usa; parado tambem aparece, dito como parado;
//! 3. ha' um arquivo `SQLite` no workspace? — extensao E o cabecalho
//!    `SQLite format 3\0`, ate' 4 niveis, pulando as pastas da maquina.
//!
//! Cada achado vem com o PERFIL que o alcanca — sem senha, como sempre.

use std::{
    fs,
    io::Read,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

use kinein_protocol::{
    ContainerInfo, DataSourceCandidate, DataSourceCandidateKind, DataSourceDiscoverResult,
    DataSourceEngine, DataSourceProfile, SecretSource,
};

use crate::container;
use crate::tools::ToolDetector;

/// Onde o `PostgreSQL` de distro poe o socket (Debian/Ubuntu/Fedora).
const SOCKET_DIRS: &[&str] = &["/var/run/postgresql", "/tmp"];
/// Quanto se espera por uma porta no loopback.
const PORT_TIMEOUT: Duration = Duration::from_millis(200);
/// Pastas que nao se percorrem procurando `.sqlite` — `.kinein` inclusa: o
/// `kinein.db` de la' e' a persistencia DA IDE (`crate::db`), nao um banco
/// do autor.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".kinein",
    ".idea",
    ".cache",
    "target",
    "build",
    "node_modules",
    ".venv",
];
/// Profundidade maxima da busca por arquivos.
const MAX_DEPTH: usize = 4;
/// Teto de arquivos listados.
const MAX_FILES: usize = 50;
/// Os 16 bytes que abrem todo arquivo `SQLite`.
const SQLITE_HEADER: &[u8] = b"SQLite format 3\0";

/// Tudo que respondeu, servidores primeiro, depois containers, depois arquivos.
#[must_use]
pub fn discover(root: &Path, detector: &ToolDetector) -> DataSourceDiscoverResult {
    let mut candidates = local_servers();
    let engine = container::detect_with(detector);
    if let Some(engine) = engine.as_ref() {
        let listed = container::list(engine, true);
        candidates.extend(listed.containers.iter().filter_map(container_candidate));
    }
    candidates.extend(sqlite_files(root));
    let hint = candidates.is_empty().then(|| {
        if engine.is_some() {
            "nenhum servidor no loopback, nenhum container de banco e nenhum .sqlite no \
             projeto. `Novo banco` cria um arquivo SQLite ou sobe um PostgreSQL/MongoDB em \
             container."
                .to_owned()
        } else {
            "nenhum servidor no loopback e nenhum .sqlite no projeto; sem Podman/Docker no \
             PATH nao ha' como subir um servidor em container — `Novo banco` cria um SQLite."
                .to_owned()
        }
    });
    DataSourceDiscoverResult {
        candidates,
        container_engine: engine.map(|e| e.kind),
        hint,
    }
}

/// O usuario desta sessao: e' o papel que o `peer` do socket aceita.
fn local_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "postgres".to_owned())
}

fn profile(
    engine: DataSourceEngine,
    name: &str,
    host: &str,
    port: u16,
    database: &str,
    user: &str,
) -> DataSourceProfile {
    DataSourceProfile {
        name: name.to_owned(),
        engine,
        host: host.to_owned(),
        port,
        database: database.to_owned(),
        user: user.to_owned(),
        secret_source: SecretSource::Automatic,
        secret_variable: None,
        sample_size: None,
        tls: None,
        ca_file: None,
    }
}

fn port_answers(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, PORT_TIMEOUT).is_ok()
}

/// O socket do `PostgreSQL` e as duas portas padrao no loopback.
fn local_servers() -> Vec<DataSourceCandidate> {
    let mut found = Vec::new();
    let socket_dir = SOCKET_DIRS
        .iter()
        .find(|dir| Path::new(dir).join(".s.PGSQL.5432").exists());
    if let Some(dir) = socket_dir {
        found.push(DataSourceCandidate {
            kind: DataSourceCandidateKind::LocalServer,
            label: "PostgreSQL local (socket)".to_owned(),
            detail: format!("{dir}/.s.PGSQL.5432 · sem senha (peer)"),
            running: true,
            profile: profile(
                DataSourceEngine::Postgres,
                "postgres-local",
                dir,
                5432,
                "postgres",
                &local_user(),
            ),
        });
    } else if port_answers(5432) {
        found.push(DataSourceCandidate {
            kind: DataSourceCandidateKind::LocalServer,
            label: "PostgreSQL local (porta)".to_owned(),
            detail: "127.0.0.1:5432 aceitou conexao".to_owned(),
            running: true,
            profile: profile(
                DataSourceEngine::Postgres,
                "postgres-local",
                "127.0.0.1",
                5432,
                "postgres",
                "postgres",
            ),
        });
    }
    if port_answers(27017) {
        found.push(DataSourceCandidate {
            kind: DataSourceCandidateKind::LocalServer,
            label: "MongoDB local".to_owned(),
            detail: "127.0.0.1:27017 aceitou conexao".to_owned(),
            running: true,
            profile: profile(
                DataSourceEngine::Mongo,
                "mongo-local",
                "127.0.0.1",
                27017,
                "test",
                "",
            ),
        });
    }
    found
}

/// O motor que a imagem entrega, pelo nome dela.
fn engine_of_image(image: &str) -> Option<(DataSourceEngine, u16)> {
    let lower = image.to_lowercase();
    if lower.contains("postgres") || lower.contains("timescale") {
        Some((DataSourceEngine::Postgres, 5432))
    } else if lower.contains("mongo") {
        Some((DataSourceEngine::Mongo, 27017))
    } else {
        None
    }
}

/// A porta do host publicada para `inner` (`0.0.0.0:5433->5432/tcp`,
/// `127.0.0.1:5433->5432/tcp`). Sem publicacao, `None`: nao ha' como alcancar.
fn published_port(ports: &[String], inner: u16) -> Option<u16> {
    let alvo = format!("->{inner}/");
    ports.iter().find_map(|line| {
        let (host, _) = line.split_once(&alvo)?;
        host.rsplit(':').next()?.parse().ok()
    })
}

/// Um container de banco vira candidato — parado tambem, dito como parado.
fn container_candidate(info: &ContainerInfo) -> Option<DataSourceCandidate> {
    let (engine, inner) = engine_of_image(&info.image)?;
    let name = info
        .names
        .first()
        .cloned()
        .unwrap_or_else(|| info.id.clone());
    let port = published_port(&info.ports, inner);
    let running = info.state == "running";
    let detail = port.map_or_else(
        || format!("{} · sem porta publicada · {}", info.status, info.image),
        |port| format!("{} · 127.0.0.1:{port} · {}", info.status, info.image),
    );
    let (database, user) = match engine {
        DataSourceEngine::Mongo => ("test", ""),
        _ => ("postgres", "postgres"),
    };
    Some(DataSourceCandidate {
        kind: DataSourceCandidateKind::Container,
        label: format!("container {name}"),
        detail,
        running: running && port.is_some(),
        profile: profile(
            engine,
            &name,
            "127.0.0.1",
            port.unwrap_or(inner),
            database,
            user,
        ),
    })
}

fn is_sqlite_file(path: &Path) -> bool {
    let mut header = [0_u8; 16];
    fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok()
        && header == SQLITE_HEADER
}

fn has_sqlite_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .as_deref(),
        Some("db" | "sqlite" | "sqlite3")
    )
}

/// Os arquivos `SQLite` do workspace, ate' `MAX_DEPTH`, ate' `MAX_FILES`.
fn sqlite_files(root: &Path) -> Vec<DataSourceCandidate> {
    let mut found = Vec::new();
    let mut pending: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if kind.is_dir() {
                if depth < MAX_DEPTH && !SKIP_DIRS.contains(&name.as_str()) {
                    pending.push((path, depth + 1));
                }
            } else if kind.is_file() && has_sqlite_extension(&path) && is_sqlite_file(&path) {
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                let size = fs::metadata(&path).map_or(0, |m| m.len());
                let stem = path
                    .file_stem()
                    .map_or_else(|| name.clone(), |s| s.to_string_lossy().to_string());
                found.push(DataSourceCandidate {
                    kind: DataSourceCandidateKind::File,
                    label: relative.clone(),
                    detail: format!("{} KB", size.div_ceil(1024)),
                    running: true,
                    profile: profile(
                        DataSourceEngine::Sqlite,
                        &stem,
                        "",
                        0,
                        &path.display().to_string(),
                        "",
                    ),
                });
                if found.len() >= MAX_FILES {
                    return found;
                }
            }
        }
    }
    found.sort_by(|a, b| a.label.cmp(&b.label));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_port_reads_the_host_side() {
        let ports = vec![
            "0.0.0.0:5433->5432/tcp".to_owned(),
            ":::5433->5432/tcp".to_owned(),
        ];
        assert_eq!(published_port(&ports, 5432), Some(5433));
        assert_eq!(published_port(&ports, 27017), None);
        assert_eq!(
            published_port(&["127.0.0.1:27018->27017/tcp".to_owned()], 27017),
            Some(27018)
        );
    }

    #[test]
    fn engine_comes_from_the_image_name() {
        assert_eq!(
            engine_of_image("docker.io/library/postgres:16"),
            Some((DataSourceEngine::Postgres, 5432))
        );
        assert_eq!(
            engine_of_image("timescale/timescaledb:latest-pg16"),
            Some((DataSourceEngine::Postgres, 5432))
        );
        assert_eq!(
            engine_of_image("docker.io/library/mongo:7"),
            Some((DataSourceEngine::Mongo, 27017))
        );
        assert_eq!(engine_of_image("docker.io/library/alpine:3.22"), None);
    }

    #[test]
    fn a_stopped_container_is_listed_but_not_running() {
        let info = ContainerInfo {
            id: "abc".to_owned(),
            names: vec!["kinein-pg".to_owned()],
            image: "docker.io/library/postgres:16".to_owned(),
            state: "exited".to_owned(),
            status: "Exited (0) 2 hours ago".to_owned(),
            ports: vec!["127.0.0.1:5433->5432/tcp".to_owned()],
            created: String::new(),
        };
        let candidate = container_candidate(&info).unwrap();
        assert_eq!(candidate.kind, DataSourceCandidateKind::Container);
        assert!(!candidate.running);
        assert_eq!(candidate.profile.port, 5433);
        assert_eq!(candidate.profile.host, "127.0.0.1");
        assert_eq!(candidate.profile.name, "kinein-pg");
    }

    #[test]
    fn sqlite_files_need_extension_and_header() {
        let dir = std::env::temp_dir().join(format!("kinein-discover-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("data")).unwrap();
        fs::create_dir_all(dir.join("target")).unwrap();
        let mut real = b"SQLite format 3\0".to_vec();
        real.extend(std::iter::repeat_n(0_u8, 100));
        fs::write(dir.join("data/app.sqlite"), &real).unwrap();
        fs::write(dir.join("notas.db"), b"nao e' sqlite").unwrap();
        fs::write(dir.join("target/cache.sqlite"), &real).unwrap();
        fs::create_dir_all(dir.join(".kinein")).unwrap();
        fs::write(dir.join(".kinein/kinein.db"), &real).unwrap();
        let found = sqlite_files(&dir);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].label, "data/app.sqlite");
        assert_eq!(found[0].profile.engine, DataSourceEngine::Sqlite);
        assert_eq!(found[0].profile.name, "app");
        let _ = fs::remove_dir_all(&dir);
    }
}
