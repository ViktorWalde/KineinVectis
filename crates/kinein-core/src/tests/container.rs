//! O dominio `container` (roadmaps/28 §0, priorizado em 2026-09-12): a
//! deteccao do motor com um `PATH` FALSO, e os parsers com a saida REAL do
//! Podman 5.8.4 desta maquina e com a forma DOCUMENTADA do Docker.
//!
//! O que se prova: que o shim `podman-docker` nao e' tomado por Docker Engine
//! (erraria formato e diagnostico), que as duas formas de JSON viram a mesma
//! lista, que portas/nomes/estado saem certos de cada uma, e que a imagem do
//! Podman (que nao traz `repository`) ganha repo e tag do `Names`.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use kinein_protocol::{ComposeAction, ContainerAction, ContainerEngine, ContainerOpenMode};

use crate::container::{
    Engine, action_command, compose_command, compose_file_in, detect_with, open_program_args, parse,
};
use crate::tools::ToolDetector;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-container-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn binario_falso(dir: &Path, nome: &str, corpo: &str) {
    let caminho = dir.join(nome);
    std::fs::write(&caminho, format!("#!/bin/sh\n{corpo}\n")).unwrap();
    std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// O que o `podman-docker` faz em 2026-09-12: aviso em stderr, versao do podman
/// em stdout.
const SHIM: &str = r#"echo "Emulate Docker CLI using podman. Create /etc/containers/nodocker to quiet msg." >&2
echo "podman version 5.8.4""#;

#[test]
fn the_podman_docker_shim_is_recognised_as_podman_and_the_real_podman_is_preferred() {
    let dir = temp_dir("shim");
    binario_falso(&dir, "docker", SHIM);
    binario_falso(&dir, "podman", r#"echo "podman version 5.8.4""#);
    let engine = detect_with(&ToolDetector::with_search_path(&dir)).unwrap();
    assert_eq!(engine.kind, ContainerEngine::Podman);
    assert!(engine.emulated);
    assert_eq!(engine.binary, dir.join("podman"));
}

#[test]
fn a_real_docker_is_docker_and_wins_over_podman_on_the_same_path() {
    let dir = temp_dir("docker");
    binario_falso(
        &dir,
        "docker",
        r#"echo "Docker version 28.3.2, build 578ccf6""#,
    );
    binario_falso(&dir, "podman", r#"echo "podman version 5.8.4""#);
    let engine = detect_with(&ToolDetector::with_search_path(&dir)).unwrap();
    assert_eq!(engine.kind, ContainerEngine::Docker);
    assert!(!engine.emulated);
    assert_eq!(engine.binary, dir.join("docker"));
}

#[test]
fn only_podman_or_nothing() {
    let dir = temp_dir("so-podman");
    binario_falso(&dir, "podman", r#"echo "podman version 5.8.4""#);
    let engine = detect_with(&ToolDetector::with_search_path(&dir)).unwrap();
    assert_eq!(
        (engine.kind, engine.emulated),
        (ContainerEngine::Podman, false)
    );

    let vazio = temp_dir("nada");
    assert!(detect_with(&ToolDetector::with_search_path(&vazio)).is_none());
}

/// Dois containers da saida REAL de `podman ps -a --format json` em
/// 2026-09-12, reduzidos as chaves que importam.
const PODMAN_PS: &str = r#"[
 {"Id":"fdd8de359c22","Image":"docker.io/timescale/timescaledb:latest-pg16","Names":["timescaledb"],
  "State":"exited","Status":"Exited (0) 2 weeks ago","Created":1787775954,"CreatedAt":"2 weeks ago",
  "Ports":[{"host_ip":"","container_port":5432,"host_port":5432,"range":1,"protocol":"tcp"}]},
 {"Id":"27b17e15d40c","Image":"docker.io/library/postgres:16-alpine","Names":["postgres-dev","pg"],
  "State":"running","Status":"Up 3 minutes","Created":1787775959,"CreatedAt":"2 weeks ago",
  "Ports":[{"host_ip":"127.0.0.1","container_port":5432,"host_port":5433,"range":1,"protocol":"tcp"},
           {"container_port":9187,"protocol":"tcp"}]}
]"#;

/// A forma documentada de `docker ps --format '{{json .}}'`: um objeto por
/// linha, `ID`, `Names` como string, `Ports` como string. NAO verificada
/// contra um Docker Engine real (esta maquina nao tem um).
const DOCKER_PS: &str = r#"{"Command":"\"docker-entrypoint.s…\"","CreatedAt":"2026-09-10 10:00:00 -0300 -03","ID":"a1b2c3d4e5f6","Image":"postgres:16","Labels":"","LocalVolumes":"1","Mounts":"pgdata","Names":"/pg,/pg-alias","Networks":"bridge","Ports":"0.0.0.0:5432->5432/tcp, :::5432->5432/tcp","RunningFor":"2 days ago","Size":"0B","State":"running","Status":"Up 2 days"}
lixo que nao e json
{"ID":"ffff00001111","Image":"redis:7","Names":"cache","Ports":"","State":"exited","Status":"Exited (0) 1 hour ago","CreatedAt":"2026-09-11 09:00:00 -0300 -03"}"#;

#[test]
fn podman_array_form_is_parsed_with_names_ports_and_state() {
    let lista = parse::containers(PODMAN_PS);
    assert_eq!(lista.len(), 2);
    let ts = &lista[0];
    assert_eq!(ts.names, vec!["timescaledb"]);
    assert_eq!(ts.state, "exited");
    assert_eq!(ts.status, "Exited (0) 2 weeks ago");
    assert_eq!(ts.ports, vec!["0.0.0.0:5432->5432/tcp"]);
    assert_eq!(ts.created, "2 weeks ago");
    let pg = &lista[1];
    assert_eq!(pg.names, vec!["postgres-dev", "pg"]);
    // host_ip declarado e' respeitado; porta exposta sem host vira `porta/proto`.
    assert_eq!(pg.ports, vec!["127.0.0.1:5433->5432/tcp", "9187/tcp"]);
    assert_eq!(pg.state, "running");
}

#[test]
fn docker_json_lines_form_is_parsed_and_garbage_lines_are_ignored() {
    let lista = parse::containers(DOCKER_PS);
    assert_eq!(
        lista.len(),
        2,
        "a linha de lixo nao vira container nem derruba as outras"
    );
    let pg = &lista[0];
    assert_eq!(pg.id, "a1b2c3d4e5f6");
    // A barra inicial do Docker antigo cai; a virgula separa nomes.
    assert_eq!(pg.names, vec!["pg", "pg-alias"]);
    assert_eq!(
        pg.ports,
        vec!["0.0.0.0:5432->5432/tcp", ":::5432->5432/tcp"]
    );
    assert_eq!(pg.created, "2026-09-10 10:00:00 -0300 -03");
    assert!(lista[1].ports.is_empty());
    assert_eq!(lista[1].state, "exited");
}

#[test]
fn state_is_derived_from_exited_when_the_engine_leaves_it_blank() {
    let lista = parse::containers(
        r#"{"Id":"x","Exited":true,"State":""}
{"Id":"y","Exited":false}"#,
    );
    assert_eq!(lista[0].state, "exited");
    assert_eq!(lista[1].state, "running");
}

#[test]
fn podman_images_get_repo_and_tag_from_names_and_docker_from_fields() {
    // Podman `--format json`: sem repository/tag; Names ["repo:tag"]; registro
    // com porta para provar que a divisao e' no ultimo `:` depois da `/`.
    let podman = r#"[{"Id":"681c2edef562","Names":["localhost:5000/kinein/builder:bookworm"],"Size":3035568151,"Created":1788379930,"CreatedAt":"2026-09-02T20:12:10Z"},
{"Id":"deadbeef0000","Names":null,"RepoTags":null,"Size":10,"CreatedAt":"2026-09-01T00:00:00Z"},
{"Id":"cafe00000000","Names":["docker.io/library/mongo"],"Size":1,"CreatedAt":"2026-09-01T00:00:00Z"}]"#;
    let imagens = parse::images(podman);
    assert_eq!(imagens.len(), 3);
    assert_eq!(imagens[0].repository, "localhost:5000/kinein/builder");
    assert_eq!(imagens[0].tag, "bookworm");
    assert_eq!(imagens[0].size, "3035568151");
    assert_eq!(imagens[0].created, "2026-09-02T20:12:10Z");
    assert_eq!(
        (imagens[1].repository.as_str(), imagens[1].tag.as_str()),
        ("<none>", "<none>")
    );
    assert_eq!(
        (imagens[2].repository.as_str(), imagens[2].tag.as_str()),
        ("docker.io/library/mongo", "latest")
    );

    let docker = r#"{"Containers":"N/A","CreatedAt":"2026-09-01 10:00:00 -0300 -03","CreatedSince":"11 days ago","Digest":"<none>","ID":"9c7a54a9a43c","Repository":"postgres","SharedSize":"N/A","Size":"431MB","Tag":"16","UniqueSize":"N/A","VirtualSize":"431.2MB"}"#;
    let imagens = parse::images(docker);
    assert_eq!(imagens[0].repository, "postgres");
    assert_eq!(imagens[0].tag, "16");
    assert_eq!(imagens[0].size, "431MB");
}

fn motor(kind: ContainerEngine) -> Engine {
    Engine {
        kind,
        binary: PathBuf::from("/usr/bin/motor"),
        emulated: false,
    }
}

#[test]
fn commands_are_built_as_the_cli_expects_and_rm_is_never_forced() {
    let rm = action_command(
        &motor(ContainerEngine::Podman),
        ContainerAction::Remove,
        "pg",
    );
    let args: Vec<String> = rm
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        args,
        vec!["rm", "pg"],
        "remover o que roda e' dois gestos, de proposito"
    );

    let (programa, args) = open_program_args(
        &motor(ContainerEngine::Docker),
        ContainerOpenMode::Logs,
        "pg",
    );
    assert_eq!(programa, "/usr/bin/motor");
    assert_eq!(args, vec!["logs", "-f", "--tail", "200", "pg"]);
    let (_, args) = open_program_args(
        &motor(ContainerEngine::Docker),
        ContainerOpenMode::Shell,
        "pg",
    );
    assert_eq!(args, vec!["exec", "-it", "pg", "/bin/sh"]);

    // `<bin> compose` vira programa + subcomando; binario avulso, so' programa.
    let root = Path::new("/tmp/projeto");
    let up = compose_command(
        "/usr/bin/podman compose",
        ComposeAction::Up,
        Some("dev.yaml"),
        root,
    );
    assert_eq!(up.get_program(), "/usr/bin/podman");
    let args: Vec<String> = up
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, vec!["compose", "-f", "dev.yaml", "up", "-d"]);
    assert_eq!(up.get_current_dir(), Some(root));
    let down = compose_command("podman-compose", ComposeAction::Down, None, root);
    assert_eq!(down.get_program(), "podman-compose");
    let args: Vec<String> = down
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, vec!["down"]);
}

/// O arquivo de compose e' do PROJETO: o primeiro dos nomes que a ferramenta
/// procura sozinha, na ordem em que ela prefere; pasta com esse nome nao vale.
#[test]
fn the_compose_file_is_the_first_the_tool_would_pick_up() {
    let dir = temp_dir("compose-file");
    assert_eq!(compose_file_in(&dir), None, "pasta vazia");
    std::fs::write(dir.join("docker-compose.yml"), "services: {}\n").unwrap();
    assert_eq!(
        compose_file_in(&dir).as_deref(),
        Some("docker-compose.yml"),
        "o nome classico ainda conta"
    );
    std::fs::write(dir.join("compose.yml"), "services: {}\n").unwrap();
    std::fs::write(dir.join("compose.yaml"), "services: {}\n").unwrap();
    assert_eq!(
        compose_file_in(&dir).as_deref(),
        Some("compose.yaml"),
        "compose.yaml ganha de compose.yml e do docker-compose.yml, como nas duas ferramentas"
    );
    for nome in ["podman-compose.yml", "container-compose.yml"] {
        let dir = temp_dir(&format!("compose-{nome}"));
        std::fs::write(dir.join(nome), "services: {}\n").unwrap();
        assert_eq!(
            compose_file_in(&dir).as_deref(),
            Some(nome),
            "{nome}: o podman-compose acha"
        );
    }
    let so_pasta = temp_dir("compose-pasta");
    std::fs::create_dir_all(so_pasta.join("compose.yaml")).unwrap();
    assert_eq!(
        compose_file_in(&so_pasta),
        None,
        "uma PASTA compose.yaml nao e' arquivo"
    );
}

/// `container.status` diz o arquivo de compose do workspace aberto (e nada sem
/// workspace); `container.compose` sem arquivo recusa ANTES de subir um job
/// que so' poderia falhar — o motivo diz o que criar.
#[test]
fn status_reports_the_compose_file_and_compose_refuses_without_one() {
    use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
    use serde_json::json;

    let mut core = super::core_with_empty_search_path("container-compose");
    let status = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "container.status",
        Some(json!({})),
    ));
    let result = status.response().result.clone().unwrap();
    assert!(
        result.get("composeFile").is_none(),
        "sem workspace nao ha' arquivo: {result}"
    );

    let dir = temp_dir("compose-workspace");
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let recusa = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "container.compose",
        Some(json!({ "action": "up" })),
    ));
    let erro = recusa
        .response()
        .error
        .clone()
        .expect("sem arquivo, recusa");
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(erro.message.contains("compose.yaml"), "{}", erro.message);

    std::fs::write(dir.join("compose.yml"), "services: {}\n").unwrap();
    let status = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "container.status",
        Some(json!({})),
    ));
    let result = status.response().result.clone().unwrap();
    assert_eq!(result["composeFile"], "compose.yml", "{result}");
}
