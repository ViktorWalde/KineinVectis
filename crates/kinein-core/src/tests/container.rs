//! O dominio `container` (roadmaps/28 §0, priorizado em 2026-09-12): a
//! deteccao do motor com um `PATH` FALSO, e os parsers com a saida REAL do
//! Podman 5.8.4 desta maquina e com a forma DOCUMENTADA do Docker.
//!
//! O que se prova: que o shim `podman-docker` nao e' tomado por Docker Engine
//! (erraria formato e diagnostico), que as duas formas de JSON viram a mesma
//! lista, que portas/nomes/estado saem certos de cada uma, e que a imagem do
//! Podman (que nao traz `repository`) ganha repo e tag do `Names`.

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

/// Devolve o caminho que o sistema acha: no Windows, o `.exe` de passagem.
fn fake_binary(dir: &Path, name: &str, corpo: &str) -> PathBuf {
    crate::write_executable(dir.join(name), format!("#!/bin/sh\n{corpo}\n"))
}

/// O que o `podman-docker` faz em 2026-09-12: aviso em stderr, versao do podman
/// em stdout.
const SHIM: &str = r#"echo "Emulate Docker CLI using podman. Create /etc/containers/nodocker to quiet msg." >&2
echo "podman version 5.8.4""#;

#[test]
fn the_podman_docker_shim_is_recognised_as_podman_and_the_real_podman_is_preferred() {
    let dir = temp_dir("shim");
    fake_binary(&dir, "docker", SHIM);
    let podman = fake_binary(&dir, "podman", r#"echo "podman version 5.8.4""#);
    let engine = detect_with(&ToolDetector::with_search_path(&dir)).unwrap();
    assert_eq!(engine.kind, ContainerEngine::Podman);
    assert!(engine.emulated);
    assert_eq!(engine.binary, podman);
}

#[test]
fn a_real_docker_is_docker_and_wins_over_podman_on_the_same_path() {
    let dir = temp_dir("docker");
    let docker = fake_binary(
        &dir,
        "docker",
        r#"echo "Docker version 28.3.2, build 578ccf6""#,
    );
    fake_binary(&dir, "podman", r#"echo "podman version 5.8.4""#);
    let engine = detect_with(&ToolDetector::with_search_path(&dir)).unwrap();
    assert_eq!(engine.kind, ContainerEngine::Docker);
    assert!(!engine.emulated);
    assert_eq!(engine.binary, docker);
}

#[test]
fn only_podman_or_nothing() {
    let dir = temp_dir("so-podman");
    fake_binary(&dir, "podman", r#"echo "podman version 5.8.4""#);
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
    let args_of = |command: std::process::Command| -> Vec<String> {
        command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    };
    let rm = action_command(
        &motor(ContainerEngine::Podman),
        ContainerAction::Remove,
        "pg",
        false,
    );
    assert_eq!(
        args_of(rm),
        vec!["rm", "pg"],
        "remover o que roda e' dois gestos, de proposito"
    );
    // 16b.3: "Remover com os volumes" e' `rm -v`, ainda sem `-f`.
    let with_volumes = action_command(
        &motor(ContainerEngine::Podman),
        ContainerAction::Remove,
        "pg",
        true,
    );
    assert_eq!(args_of(with_volumes), vec!["rm", "-v", "pg"]);

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
    for name in ["podman-compose.yml", "container-compose.yml"] {
        let dir = temp_dir(&format!("compose-{name}"));
        std::fs::write(dir.join(name), "services: {}\n").unwrap();
        assert_eq!(
            compose_file_in(&dir).as_deref(),
            Some(name),
            "{name}: o podman-compose acha"
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
    let error = recusa
        .response()
        .error
        .clone()
        .expect("sem arquivo, recusa");
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("compose.yaml"), "{}", error.message);

    std::fs::write(dir.join("compose.yml"), "services: {}\n").unwrap();
    let status = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "container.status",
        Some(json!({})),
    ));
    let result = status.response().result.clone().unwrap();
    assert_eq!(result["composeFile"], "compose.yml", "{result}");
}

/// Um `docker` falso que NEGA o socket, como o Docker CE 29.9.1 do Ubuntu
/// 26.04 com o usuario fora do grupo (medido em 2026-10-15): o `--version`
/// e a linha do cliente respondem, o resto sai 1 com "permission denied".
const DOCKER_DENYING: &str = r#"case "$1" in
  --version) echo "Docker version 29.9.1, build dfc520d" ;;
  version) echo "29.9.1"; echo "permission denied while trying to connect to the docker API at unix:///var/run/docker.sock" >&2; exit 1 ;;
  context) echo "unix:///var/run/docker.sock" ;;
  *) echo "permission denied while trying to connect to the docker API at unix:///var/run/docker.sock" >&2; exit 1 ;;
esac"#;

/// Um `podman` falso que responde, sem nenhum container.
const PODMAN_ANSWERING: &str = r#"case "$1" in
  --version) echo "podman version 5.7.0" ;;
  version) echo "5.7.0" ;;
  ps) echo "[]" ;;
  images) echo "[]" ;;
  info) echo "true" ;;
  *) exit 0 ;;
esac"#;

/// Defeito 2 (16b): o `docker` que nega nao ganha do `podman` que responde.
#[test]
fn a_denying_docker_does_not_win_over_a_podman_that_answers() {
    let dir = temp_dir("escolha");
    fake_binary(&dir, "docker", DOCKER_DENYING);
    let podman = fake_binary(&dir, "podman", PODMAN_ANSWERING);
    let (engine, check) =
        crate::container::select_with(&ToolDetector::with_search_path(&dir)).expect("ha' motor");
    assert_eq!(engine.kind, ContainerEngine::Podman);
    assert_eq!(engine.binary, podman);
    assert!(check.ok);
}

/// Defeitos 2 e 3: so' o `docker` que nega — o status diz por que e da' o
/// passo do grupo; o `list` vira ERRO com a dica, e nao "nenhum container".
#[test]
fn a_denying_docker_alone_is_reported_and_its_list_is_an_error() {
    let dir = temp_dir("negando");
    fake_binary(&dir, "docker", DOCKER_DENYING);
    let detector = ToolDetector::with_search_path(&dir);
    let selected = crate::container::select_with(&detector).expect("ha' motor");
    assert_eq!(selected.0.kind, ContainerEngine::Docker);
    assert!(!selected.1.ok);

    let status = crate::container::status(Some(&selected), None);
    assert!(!status.reachable);
    assert_eq!(status.client_version.as_deref(), Some("29.9.1"));
    assert_eq!(status.version, None, "sem servidor, sem versao de servidor");
    let hint = status.hint.expect("dica");
    assert!(hint.contains("usermod -aG docker"), "{hint}");
    assert!(
        status.raw_output.contains("permission denied"),
        "{}",
        status.raw_output
    );

    let failure = crate::container::list(&selected.0, true).expect_err("negado e' erro");
    assert!(failure.hint.contains("usermod"), "{}", failure.hint);
    assert!(failure.raw_output.contains("permission denied"));
    assert!(crate::container::images(&selected.0).is_err());
}

/// Defeito 2: o contexto do `docker` aponta o socket do Podman, que nao
/// responde. A dica e' a do socket do Podman — nunca `sudo systemctl`.
#[test]
fn a_docker_context_on_the_podman_socket_gets_the_podman_hint() {
    let dir = temp_dir("contexto-podman");
    fake_binary(
        &dir,
        "docker",
        r#"case "$1" in
  --version) echo "Docker version 29.8.2, build 1" ;;
  context) echo "unix:///run/user/1000/podman/podman.sock" ;;
  *) echo "Cannot connect to the Docker daemon at unix:///run/user/1000/podman/podman.sock" >&2; exit 1 ;;
esac"#,
    );
    let selected =
        crate::container::select_with(&ToolDetector::with_search_path(&dir)).expect("motor");
    let hint = crate::container::status(Some(&selected), None)
        .hint
        .expect("dica");
    assert!(hint.contains("podman.socket"), "{hint}");
    assert!(!hint.contains("sudo systemctl"), "{hint}");
}

/// Defeito 5: um `docker` cujo servidor e' o Podman diz o Podman, com a
/// versao DELE, e nao a do CLI.
#[test]
fn a_docker_cli_served_by_podman_is_labelled_as_podman() {
    let dir = temp_dir("servidor-podman");
    fake_binary(
        &dir,
        "docker",
        r#"case "$1" in
  --version) echo "Docker version 29.8.2, build 1" ;;
  version)
    case "$3" in
      *Server*) echo '{"Platform":{"Name":""},"Components":[{"Name":"Podman Engine","Version":"6.1.3"}],"Version":"6.1.3"}' ;;
      *) echo "29.8.2" ;;
    esac ;;
  ps) exit 0 ;;
  *) exit 0 ;;
esac"#,
    );
    let selected =
        crate::container::select_with(&ToolDetector::with_search_path(&dir)).expect("motor");
    let status = crate::container::status(Some(&selected), None);
    assert!(status.reachable);
    assert_eq!(status.server_name.as_deref(), Some("Podman Engine"));
    assert_eq!(status.version.as_deref(), Some("6.1.3"));
    assert_eq!(status.client_version.as_deref(), Some("29.8.2"));
}

/// O JSON do servidor do Docker Engine de verdade: o nome vem da plataforma.
#[test]
fn the_docker_engine_server_json_names_the_engine() {
    let (name, version) = parse::server_identity(
        r#"{"Platform":{"Name":"Docker Engine - Community"},"Components":[{"Name":"Engine","Version":"29.9.1"},{"Name":"containerd","Version":"v2.1.4"}],"Version":"29.9.1"}"#,
    );
    assert_eq!(name.as_deref(), Some("Docker Engine - Community"));
    assert_eq!(version.as_deref(), Some("29.9.1"));
    assert_eq!(parse::server_identity("null"), (None, None));
    assert_eq!(parse::server_identity("lixo"), (None, None));
}

/// A versao do CLI sai do `--version`, que nao fala com o servidor.
#[test]
fn the_cli_version_comes_from_the_version_flag_line() {
    assert_eq!(
        parse::cli_version("Docker version 29.9.1, build dfc520d").as_deref(),
        Some("29.9.1")
    );
    assert_eq!(
        parse::cli_version("podman version 5.7.0").as_deref(),
        Some("5.7.0")
    );
    assert_eq!(parse::cli_version("sem versao aqui"), None);
}

/// Defeito 4: a pergunta ao motor tem prazo. Um motor que dorme e' morto, e a
/// resposta diz que estourou — o core nao fica esperando.
#[test]
fn a_question_that_hangs_is_killed_at_the_deadline() {
    let dir = temp_dir("prazo");
    let slow = fake_binary(&dir, "docker", "sleep 30");
    let start = std::time::Instant::now();
    let reply = crate::container::probe::ask_command(
        std::process::Command::new(slow),
        std::time::Duration::from_millis(300),
    );
    assert!(reply.timed_out);
    assert!(!reply.ok);
    assert!(
        reply.reason().contains("nao respondeu"),
        "{}",
        reply.reason()
    );
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "voltou em {:?}",
        start.elapsed()
    );
}

/// Defeito 3 pelo despacho: `container.list` com o motor negando responde ERRO,
/// com a dica nos dados; o core usa o PATH dele, nao o da maquina.
#[test]
fn container_list_with_a_denying_engine_is_an_error_with_the_hint() {
    use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
    use serde_json::json;

    let dir = temp_dir("lista-negada");
    fake_binary(&dir, "docker", DOCKER_DENYING);
    let mut core = crate::Core::with_detector(ToolDetector::with_search_path(&dir));
    let reply = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "container.list",
        Some(json!({})),
    ));
    let error = reply.response().error.clone().expect("negado e' erro");
    assert_eq!(error.code, JsonRpcErrorCode::ToolNotFound);
    assert!(error.message.contains("usermod"), "{}", error.message);
    let details = error.details.expect("details");
    assert_eq!(details["reachable"], false);
    assert_eq!(details["engine"], "docker");
}

/// A saida REAL do `podman system df -v` (Podman 5.7.0, 2026-10-15), com um
/// volume anonimo de um `postgres:16` ligado por 8 s e outro vazio.
const PODMAN_DF_V: &str = concat!(
    "Images space usage:\n",
    "\n",
    "REPOSITORY                  TAG         IMAGE ID      CREATED     SIZE        SHARED SIZE  UNIQUE SIZE  CONTAINERS\n",
    "docker.io/library/postgres  16          275447c94b11  9 days      458.5MB     0B           458.5MB      1\n",
    "\n",
    "Containers space usage:\n",
    "\n",
    "CONTAINER ID  IMAGE         COMMAND     LOCAL VOLUMES  SIZE        CREATED             STATUS      NAMES\n",
    "f32c40955dbd  275447c94b11  postgres    1              0B          Less than a second  created     kv-measure\n",
    "\n",
    "Local Volumes space usage:\n",
    "\n",
    "VOLUME NAME                                                       LINKS       SIZE\n",
    "c145a933acd6f92f5cfb66ab56248af47950c62c2100fdce59764b004987fb4c  0           0B\n",
    "d2166cfeffc45c97b48fe9b7852ac84468a9e6f5be78e027be8f6153229f8834  1           40.08MB\n",
    "\n",
);

/// 16b.3: o tamanho de cada volume sai da tabela (o JSON verboso nao existe
/// no Podman 5.7), em unidades decimais; o anonimo e' o de 64 hexadecimais.
#[test]
fn volume_sizes_come_from_the_df_table_in_decimal_units() {
    let sizes = parse::volume_sizes(PODMAN_DF_V);
    assert_eq!(sizes.len(), 2, "{sizes:?}");
    assert_eq!(
        sizes.get("d2166cfeffc45c97b48fe9b7852ac84468a9e6f5be78e027be8f6153229f8834"),
        Some(&40_080_000)
    );
    assert_eq!(
        sizes.get("c145a933acd6f92f5cfb66ab56248af47950c62c2100fdce59764b004987fb4c"),
        Some(&0)
    );
    assert_eq!(parse::human_size("458.5MB"), Some(458_500_000));
    assert_eq!(parse::human_size("1.2GB"), Some(1_200_000_000));
    assert_eq!(parse::human_size("12kB"), Some(12_000));
    assert_eq!(parse::human_size("lixo"), None);
    assert!(parse::is_anonymous_volume(
        "d2166cfeffc45c97b48fe9b7852ac84468a9e6f5be78e027be8f6153229f8834"
    ));
    assert!(!parse::is_anonymous_volume("dados-do-banco"));
    assert!(
        !parse::is_anonymous_volume(&"z".repeat(64)),
        "64 letras quaisquer nao e' o nome que o motor gera"
    );
    let mounts = parse::mounted_volumes(
        r#"[{"Type":"bind","Source":"/home/x","Destination":"/x"},{"Type":"volume","Name":"c145a933acd6f92f5cfb66ab56248af47950c62c2100fdce59764b004987fb4c","Destination":"/var/lib/postgresql/data"}]"#,
    );
    assert_eq!(
        mounts,
        vec!["c145a933acd6f92f5cfb66ab56248af47950c62c2100fdce59764b004987fb4c"]
    );
}

/// 16b.3 pelo despacho: `container.volumes` diz o anonimo com o tamanho e o
/// nomeado sem `anonymous`; `volumes` com outra acao que nao `remove` e'
/// recusado antes de qualquer job.
#[test]
fn container_volumes_lists_the_anonymous_one_with_its_size() {
    use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
    use serde_json::json;

    let dir = temp_dir("volumes");
    fake_binary(
        &dir,
        "podman",
        r#"case "$1" in
  --version) echo "podman version 5.7.0" ;;
  ps) echo "[]" ;;
  inspect) echo '[{"Type":"volume","Name":"d2166cfeffc45c97b48fe9b7852ac84468a9e6f5be78e027be8f6153229f8834"},{"Type":"volume","Name":"dados-do-banco"}]' ;;
  system) printf 'Local Volumes space usage:\n\nVOLUME NAME  LINKS  SIZE\nd2166cfeffc45c97b48fe9b7852ac84468a9e6f5be78e027be8f6153229f8834  1  40.08MB\ndados-do-banco  1  2MB\n' ;;
  *) exit 0 ;;
esac"#,
    );
    let mut core = crate::Core::with_detector(ToolDetector::with_search_path(&dir));
    let reply = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "container.volumes",
        Some(json!({ "id": "kinein-pg" })),
    ));
    let result = reply.response().result.clone().expect("volumes");
    assert_eq!(result["volumes"][0]["anonymous"], true, "{result}");
    assert_eq!(result["volumes"][0]["sizeBytes"], 40_080_000, "{result}");
    assert_eq!(result["volumes"][1]["name"], "dados-do-banco", "{result}");
    assert_eq!(result["volumes"][1]["anonymous"], false, "{result}");

    let refused = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "container.action",
        Some(json!({ "id": "kinein-pg", "action": "stop", "volumes": true })),
    ));
    let error = refused.response().error.clone().expect("so' com remove");
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("remove"), "{}", error.message);
}
