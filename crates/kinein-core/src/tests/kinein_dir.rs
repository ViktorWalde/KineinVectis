//! INF-13 (`0.221.0`): o projeto com o `.kinein` ligado a uma pasta de fora
//! abre com o `stateProblem`, e nenhum dono que grava no `.kinein` cria ou
//! muda algo fora: o `workspace.json` e o banco ao abrir, a sessão, a
//! toolchain, os alvos remotos, o Grafana, a configuração ativa, a pasta de
//! build do configure e o LCOV da cobertura. Os links são de pasta ou para o
//! que não existe, com junção no Windows (60 §3.3).
//!
//! INF-15 (`0.222.0`): a reprodução do 62 §3.4. No lugar de cada arquivo que
//! o core lê do `.kinein`, o arquivo válido (o controle), um FIFO, um link
//! para o mesmo arquivo fora do projeto, uma pasta e um arquivo acima de
//! 1 MiB; o `workspace.open` e os pedidos do dono respondem, nada de fora é
//! usado, e o motivo aparece como o do INF-13. Os pedidos rodam com prazo, e
//! o teste abre o outro lado do FIFO se a leitura prender.
//!
//! INF-17: o catálogo do Banco, a cópia dele e os contatos remotos gravam
//! pelo mesmo guarda; o `datasource.save` e a sonda remota não criam nada
//! fora do projeto pelo link.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
};

use serde_json::{Value, json};

use super::{
    core_with_empty_search_path,
    remote::{scenario, write_tool},
};
use crate::kinein_dir::{fifo, link, unblocked};
use kinein_protocol::{JsonRpcRequest, ProjectKind};

const FOLDER_LINK: &str = ".kinein é um link simbólico, que a IDE não segue no projeto";

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-state-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

/// Um projeto C com o `.kinein` ligado a `outside`, vazia.
fn linked_project(name: &str) -> (PathBuf, PathBuf) {
    let dir = base(name);
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(
        root.join("src").join("main.c"),
        "int main(void) { return 0; }\n",
    )
    .unwrap();
    link(&outside, &root.join(".kinein"));
    (root, outside)
}

fn listing(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Pelo IPC: o `workspace.open` abre e diz por que, sem gravar o
/// `workspace.json` nem o banco fora; o `workspace.saveSession` falha com o
/// mesmo motivo. Trocado o link por uma pasta, o próximo abre sem o campo e
/// grava no projeto.
#[test]
fn the_project_with_a_linked_state_folder_opens_and_writes_nothing_outside() {
    let (root, outside) = linked_project("ipc");
    let mut core = core_with_empty_search_path("state-ipc");
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    let result = opened.response().result.as_ref().unwrap().clone();
    assert_eq!(result["stateProblem"], FOLDER_LINK);
    assert_eq!(result["root"], root.to_str().unwrap());

    let main = root.join("src").join("main.c");
    let saved = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.saveSession",
        Some(json!({ "openFiles": [main.to_str().unwrap()] })),
    ));
    let error = saved.response().error.as_ref().unwrap();
    assert!(error.message.contains(FOLDER_LINK), "{}", error.message);
    assert_eq!(listing(&outside), Vec::<String>::new());

    std::fs::remove_file(root.join(".kinein"))
        .or_else(|_| std::fs::remove_dir(root.join(".kinein")))
        .unwrap();
    let reopened = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    let result = reopened.response().result.as_ref().unwrap().clone();
    assert!(result.get("stateProblem").is_none(), "{result}");
    assert!(root.join(".kinein").join("workspace.json").is_file());
    assert_eq!(listing(&outside), Vec::<String>::new());
}

/// Cada dono, chamado direto, recusa com o motivo do guarda, e a pasta de
/// fora fica vazia.
#[test]
fn every_owner_refuses_the_linked_state_folder() {
    let (root, outside) = linked_project("donos");

    assert!(crate::db::DraftStore::open(&root).is_none());
    assert_eq!(
        crate::kits::store::save(&root, &BTreeMap::new()),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(
        crate::remote::remove(&root, "x"),
        Err(FOLDER_LINK.to_owned())
    );
    assert_eq!(crate::grafana::forget(&root), Err(FOLDER_LINK.to_owned()));
    assert_eq!(
        crate::cmake::select_configuration(&root, Some("debug"))
            .unwrap_err()
            .to_string(),
        FOLDER_LINK
    );
    let build = root.join(".kinein").join("build").join("debug");
    assert_eq!(
        crate::cmake::begin_configure(&root, &build, Some("debug"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        FOLDER_LINK
    );
    let coverage = crate::coverage::run(
        &root,
        ProjectKind::RustCargo,
        &crate::coverage::CoverageTools::default(),
        &Arc::new(AtomicBool::new(false)),
        &mut |_| {},
    );
    assert!(
        matches!(&coverage, Err(crate::coverage::CoverageError::Io(error)) if error.to_string() == FOLDER_LINK),
        "{coverage:?}"
    );
    assert_eq!(listing(&outside), Vec::<String>::new());
}

/// A pasta de build e os marcadores dela: com o `.kinein` de verdade, a
/// pasta de build ligada ou um marcador ligado para o que não existe param
/// o configure antes de criar algo, e o alvo segue sem existir.
#[test]
fn the_configure_refuses_a_linked_build_folder_or_marker() {
    let dir = base("build");
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(root.join(".kinein").join("build")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let build = root.join(".kinein").join("build").join("debug");
    link(&outside, &build);
    assert_eq!(
        crate::cmake::begin_configure(&root, &build, Some("debug"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        ".kinein/build/debug é um link simbólico, que a IDE não segue no projeto"
    );
    assert_eq!(listing(&outside), Vec::<String>::new());

    let release = root.join(".kinein").join("build").join("release");
    std::fs::create_dir_all(&release).unwrap();
    let missing = outside.join("trava");
    link(&missing, &release.join(".kinein-configure-lock"));
    assert_eq!(
        crate::cmake::begin_configure(&root, &release, Some("release"), "gcc")
            .err()
            .unwrap()
            .to_string(),
        ".kinein/build/release/.kinein-configure-lock é um link simbólico, que a IDE não segue no projeto"
    );
    assert!(!missing.exists());
    assert!(!root.join(".kinein").join("configuration.json").exists());
}

/// O banco dos rascunhos e os arquivos do `SQLite` ao lado dele: com o
/// `.kinein` de verdade, o banco, o `-wal`, o `-shm` ou o `-journal` ligados
/// para o que não existe desligam a persistência, e o alvo segue sem existir.
/// Os três de lado o `SQLite` recusa sozinho (medido em 2026-11-12); o teste
/// fica como guarda se ele mudar.
#[test]
fn the_draft_store_refuses_each_linked_database_file() {
    for name in [
        "kinein.db",
        "kinein.db-wal",
        "kinein.db-shm",
        "kinein.db-journal",
    ] {
        let dir = base(&format!("banco-{name}"));
        let root = dir.join("projeto");
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        let missing = dir.join("fora");
        link(&missing, &root.join(".kinein").join(name));
        assert!(crate::db::DraftStore::open(&root).is_none(), "{name}");
        assert!(!missing.exists(), "{name}");
    }
}

/// O que fica no lugar do arquivo lido.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Placed {
    /// O arquivo válido: o controle, que o dono usa.
    Regular,
    Fifo,
    /// Um link para o mesmo arquivo válido, fora do projeto; no Windows, a
    /// junção para o que não existe, porque ela só liga pasta.
    Link,
    Folder,
    /// Um byte acima do limite de leitura da IDE.
    Large,
}

const PLACED: [Placed; 5] = [
    Placed::Regular,
    Placed::Fifo,
    Placed::Link,
    Placed::Folder,
    Placed::Large,
];

/// Como o dono trata o arquivo que o guarda recusa.
enum Refused {
    /// Vale como ausente, sem erro.
    Absent,
    /// O erro do pedido traz o motivo do guarda.
    Reason,
    /// O erro do pedido é este texto.
    Text(&'static str),
}

/// Um arquivo que o core lê do `.kinein` e como se vê que o dono o usou.
struct Owner {
    /// O caminho sob o `.kinein`, com `/`.
    file: &'static str,
    /// O conteúdo válido; `%ROOT%` vira a raiz do projeto.
    valid: &'static str,
    /// Arquivos de antes, sob o `.kinein`: caminho e conteúdo.
    before: &'static [(&'static str, &'static str)],
    /// Os pedidos que leem o arquivo, depois do `workspace.open`; sem eles,
    /// vale a resposta do próprio `workspace.open`.
    requests: &'static [(&'static str, &'static str)],
    /// O texto que só aparece nas respostas quando o dono usou o arquivo.
    marker: Option<&'static str>,
    refused: Refused,
}

const LEGACY_CATALOGUE: &str = include_str!("../datasource/fixtures/profiles/v1-legacy.json");
const CURRENT_CATALOGUE: &str = include_str!("../datasource/fixtures/profiles/v2-mixed.json");
const BUILD_CACHE: &[(&str, &str)] = &[("build/CMakeCache.txt", "")];
/// O contexto de um arquivo C diz o kit que vale, o `cmake::active_kit`.
const ACTIVE_KIT: &str = r#"{"path": "src/sessao-marca.c"}"#;
const SAVE_PROFILE: &str = r#"{"profile": {"name": "copia-marca", "host": "localhost",
    "port": 5432, "database": "app", "user": "postgres",
    "secretSource": "environment", "secretVariable": "PGPASSWORD"}}"#;

/// O projeto do caso, com `placed` no lugar de `owner.file`; devolve a
/// raiz, a pasta de fora e o caminho do arquivo.
fn placed_project(owner: &Owner, placed: Placed) -> (PathBuf, PathBuf, PathBuf) {
    let dir = base(&format!(
        "leitura-{}-{placed:?}",
        owner.file.replace('/', "_")
    ));
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(root.join("CMakeLists.txt"), "project(p C)\n").unwrap();
    std::fs::write(root.join("src").join("sessao-marca.c"), "int x;\n").unwrap();
    for (file, body) in owner.before {
        let path = root.join(".kinein").join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    let quoted = serde_json::to_string(root.to_str().unwrap()).unwrap();
    let valid = owner.valid.replace("%ROOT%", &quoted[1..quoted.len() - 1]);
    let path = root.join(".kinein").join(owner.file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    match placed {
        Placed::Regular => std::fs::write(&path, valid).unwrap(),
        Placed::Fifo => assert!(fifo(&path)),
        Placed::Link if cfg!(windows) => link(&outside.join("ausente"), &path),
        Placed::Link => {
            let target = outside.join(path.file_name().unwrap());
            std::fs::write(&target, valid).unwrap();
            link(&target, &path);
        }
        Placed::Folder => std::fs::create_dir_all(&path).unwrap(),
        Placed::Large => std::fs::File::create(&path)
            .unwrap()
            .set_len(crate::fsops::MAX_READ_BYTES + 1)
            .unwrap(),
    }
    (root, outside, path)
}

/// O motivo do guarda para `placed` em `.kinein/<file>`.
fn reason(file: &str, placed: Placed) -> String {
    let why = match placed {
        Placed::Regular => unreachable!("o controle não é recusado"),
        Placed::Fifo | Placed::Folder => "não é um arquivo regular",
        Placed::Link => "é um link simbólico, que a IDE não segue no projeto",
        Placed::Large => "passa de 1 MiB, o limite de leitura da IDE",
    };
    format!(".kinein/{file} {why}")
}

/// O que a faixa diz: só a entrada logo dentro do `.kinein` que é link ou
/// não é arquivo regular nem pasta.
fn band(file: &str, placed: Placed) -> Option<String> {
    if file.contains('/') {
        return None;
    }
    match placed {
        Placed::Fifo => Some(format!(
            ".kinein/{file} não é um arquivo regular nem uma pasta"
        )),
        Placed::Link => Some(reason(file, placed)),
        _ => None,
    }
}

/// Cada caso de `owner`: abre o projeto e manda os pedidos com prazo, e
/// confere a faixa, o uso do arquivo, o motivo e a pasta de fora.
fn each_placement(owner: &'static Owner) {
    for placed in PLACED {
        if placed == Placed::Fifo && cfg!(windows) {
            continue;
        }
        let (root, outside, path) = placed_project(owner, placed);
        let project = root.clone();
        let responses = unblocked(&path, move || {
            let mut core = core_with_empty_search_path("kinein-dir-leitura");
            let mut requests = vec![("workspace.open", json!({ "path": project }))];
            for (method, params) in owner.requests {
                requests.push((method, serde_json::from_str(params).unwrap()));
            }
            requests
                .into_iter()
                .zip(1_i64..)
                .map(|((method, params), id)| {
                    let outcome =
                        core.handle_request(&JsonRpcRequest::new(id, method, Some(params)));
                    serde_json::to_value(outcome.response()).unwrap()
                })
                .collect::<Vec<Value>>()
        });
        let case = format!("{} {placed:?}", owner.file);
        let opened = &responses[0]["result"];
        assert!(opened.is_object(), "{case}: {}", responses[0]);
        assert_eq!(
            opened.get("stateProblem").and_then(Value::as_str),
            band(owner.file, placed).as_deref(),
            "{case}"
        );
        let observed = if owner.requests.is_empty() {
            &responses[..1]
        } else {
            &responses[1..]
        };
        let text = serde_json::to_string(observed).unwrap();
        if let Some(marker) = owner.marker {
            assert_eq!(
                text.contains(marker),
                placed == Placed::Regular,
                "{case}: {text}"
            );
        }
        if placed == Placed::Regular {
            assert!(
                observed
                    .iter()
                    .all(|response| response.get("error").is_none()),
                "{case}: {text}"
            );
        } else {
            match owner.refused {
                Refused::Absent => assert!(
                    observed
                        .iter()
                        .all(|response| response.get("error").is_none()),
                    "{case}: {text}"
                ),
                Refused::Reason => {
                    let expected = serde_json::to_string(&reason(owner.file, placed)).unwrap();
                    assert!(
                        text.contains(&expected[1..expected.len() - 1]),
                        "{case}: {text}"
                    );
                }
                Refused::Text(expected) => assert!(text.contains(expected), "{case}: {text}"),
            }
        }
        let mut left = Vec::new();
        if placed == Placed::Link && !cfg!(windows) {
            left.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
        assert_eq!(listing(&outside), left, "{case}");
    }
}

static WORKSPACE: Owner = Owner {
    file: "workspace.json",
    valid: "{}",
    before: &[],
    requests: &[],
    marker: None,
    refused: Refused::Absent,
};

static SESSION: Owner = Owner {
    file: "session.json",
    valid: r#"{"schemaVersion": 1, "openFiles": ["src/sessao-marca.c"]}"#,
    before: &[],
    requests: &[],
    marker: Some("sessao-marca.c"),
    refused: Refused::Absent,
};

static TOOLCHAIN: Owner = Owner {
    file: "toolchain.json",
    valid: r#"{"schemaVersion": 2, "kits": {"": {"selections": {}, "sysroot": "/fora-marca"}}}"#,
    before: &[],
    requests: &[("toolchain.get", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

static CONFIGURATION: Owner = Owner {
    file: "configuration.json",
    valid: r#"{"preset": "fora-marca"}"#,
    before: &[],
    requests: &[("cmake.status", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Reason,
};

static PYTHON: Owner = Owner {
    file: "python.json",
    valid: r#"{"schemaVersion": 1, "interpreter": "%ROOT%/src/sessao-marca.c"}"#,
    before: &[],
    requests: &[("python.status", "{}")],
    marker: Some("sessao-marca.c"),
    refused: Refused::Absent,
};

static SETTINGS: Owner = Owner {
    file: "settings.json",
    valid: r#"{"schemaVersion": 1, "explorerWidth": 287}"#,
    before: &[],
    requests: &[("settings.get", "{}")],
    marker: Some(r#""explorerWidth":287"#),
    refused: Refused::Reason,
};

static REMOTES: Owner = Owner {
    file: "remotes.json",
    valid: r#"{"schemaVersion": 1, "targets": [{"name": "fora-marca", "host": "h"}]}"#,
    before: &[],
    requests: &[("remote.list", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

static CONTACTS: Owner = Owner {
    file: "remote-contacts.json",
    valid: r#"{"schemaVersion": 1, "contacts": [{"name": "fora-marca", "ok": true, "at": 1}]}"#,
    before: &[],
    requests: &[("remote.list", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

static GRAFANA: Owner = Owner {
    file: "grafana.json",
    valid: r#"{"schemaVersion": 1, "profile": {"url": "http://fora-marca"}}"#,
    before: &[],
    requests: &[("grafana.get", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

static RUN_CONFIGS: Owner = Owner {
    file: "runconfigs.json",
    valid: r#"{"schemaVersion": 2, "configs": [{"id": "a", "name": "fora-marca", "command": "true"}]}"#,
    before: &[],
    requests: &[("runConfig.list", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Reason,
};

static DATA_SOURCES: Owner = Owner {
    file: "datasources.json",
    valid: CURRENT_CATALOGUE,
    before: &[],
    requests: &[("datasource.list", "{}")],
    marker: Some("bordo"),
    refused: Refused::Reason,
};

static LEGACY_COPY: Owner = Owner {
    file: "datasources.schema1.json",
    valid: LEGACY_CATALOGUE,
    before: &[("datasources.json", LEGACY_CATALOGUE)],
    requests: &[("datasource.save", SAVE_PROFILE)],
    marker: Some("copia-marca"),
    refused: Refused::Text(
        "Não foi possível conferir a cópia do catálogo anterior. Nada foi alterado.",
    ),
};

static PENDING: Owner = Owner {
    file: "build/.kinein-configure-pending",
    valid: r#"{"preset": null, "kit": "fora-marca"}"#,
    before: BUILD_CACHE,
    requests: &[("cmake.status", "{}"), ("index.context", ACTIVE_KIT)],
    // A tentativa pendente deixa o contexto desatualizado.
    marker: Some("configure não concluído"),
    refused: Refused::Reason,
};

static PRESET_MARK: Owner = Owner {
    file: "build/.kinein-preset",
    valid: "fora-marca\n",
    before: BUILD_CACHE,
    requests: &[("cmake.status", "{}")],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

static KIT_MARK: Owner = Owner {
    file: "build/.kinein-kit",
    valid: "fora-marca\n",
    before: BUILD_CACHE,
    requests: &[("index.context", ACTIVE_KIT)],
    marker: Some("fora-marca"),
    refused: Refused::Absent,
};

#[test]
fn the_workspace_file_in_each_placement() {
    each_placement(&WORKSPACE);
}

#[test]
fn the_session_is_read_by_the_guard() {
    each_placement(&SESSION);
}

#[test]
fn the_toolchain_is_read_by_the_guard() {
    each_placement(&TOOLCHAIN);
}

#[test]
fn the_chosen_configuration_is_read_by_the_guard() {
    each_placement(&CONFIGURATION);
}

#[test]
fn the_chosen_python_is_read_by_the_guard() {
    each_placement(&PYTHON);
}

#[test]
fn the_project_settings_are_read_by_the_guard() {
    each_placement(&SETTINGS);
}

#[test]
fn the_remote_targets_are_read_by_the_guard() {
    each_placement(&REMOTES);
}

#[test]
fn the_remote_contacts_are_read_by_the_guard() {
    each_placement(&CONTACTS);
}

#[test]
fn the_grafana_profile_is_read_by_the_guard() {
    each_placement(&GRAFANA);
}

#[test]
fn the_run_configs_are_read_by_the_guard() {
    each_placement(&RUN_CONFIGS);
}

#[test]
fn the_data_source_catalogue_is_read_by_the_guard() {
    each_placement(&DATA_SOURCES);
}

#[test]
fn the_legacy_catalogue_copy_is_read_by_the_guard() {
    each_placement(&LEGACY_COPY);
}

#[test]
fn the_pending_configure_is_read_by_the_guard() {
    each_placement(&PENDING);
}

#[test]
fn the_configured_preset_mark_is_read_by_the_guard() {
    each_placement(&PRESET_MARK);
}

#[test]
fn the_configured_kit_mark_is_read_by_the_guard() {
    each_placement(&KIT_MARK);
}

/// INF-17 pelo IPC: o `datasource.save` com o `.kinein` ligado a uma pasta de
/// fora, com o catálogo ligado ao que não existe, ou com a cópia do schema 1
/// ligada numa migração, falha com o motivo do guarda e não cria nada fora;
/// com o `.kinein` de verdade, grava no projeto.
#[test]
fn the_data_source_save_writes_nothing_through_a_link() {
    let save = |root: &Path| {
        let mut core = core_with_empty_search_path("kinein-dir-banco");
        let opened = core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": root })),
        ));
        assert!(opened.response().result.is_some(), "{opened:?}");
        let saved = core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "datasource.save",
            Some(serde_json::from_str(SAVE_PROFILE).unwrap()),
        ));
        saved.response().clone()
    };

    let root = base("banco-controle").join("projeto");
    std::fs::create_dir_all(&root).unwrap();
    let saved = save(&root);
    assert!(saved.error.is_none(), "{saved:?}");
    let catalogue = root.join(".kinein").join("datasources.json");
    assert!(
        std::fs::read_to_string(catalogue)
            .unwrap()
            .contains("copia-marca")
    );

    let (root, outside) = linked_project("banco-pasta");
    let message = save(&root).error.unwrap().message;
    assert!(message.contains(FOLDER_LINK), "{message}");
    assert_eq!(listing(&outside), Vec::<String>::new());

    for (file, before) in [
        ("datasources.json", None),
        ("datasources.schema1.json", Some(LEGACY_CATALOGUE)),
    ] {
        let dir = base(&format!("banco-{file}"));
        let root = dir.join("projeto");
        let state = root.join(".kinein");
        std::fs::create_dir_all(&state).unwrap();
        if let Some(body) = before {
            std::fs::write(state.join("datasources.json"), body).unwrap();
        }
        let missing = dir.join("fora");
        link(&missing, &state.join(file));
        let message = save(&root).error.unwrap().message;
        assert!(
            message.contains(&format!(
                ".kinein/{file} é um link simbólico, que a IDE não segue no projeto"
            )),
            "{message}"
        );
        assert!(!missing.exists(), "{file}");
        if let Some(body) = before {
            assert_eq!(
                std::fs::read_to_string(state.join("datasources.json")).unwrap(),
                body
            );
        }
    }
}

/// INF-17 pelo IPC: a sonda grava o último contato no projeto; com o
/// `remote-contacts.json` ligado ao que não existe, ela segue e responde, e
/// nada é criado fora. O `ssh` é o falso dos testes do `remote`.
#[test]
fn the_probe_records_no_contact_through_a_link() {
    let _serial = crate::serialize_executables();
    let mut c = scenario("contatos-ligados");
    c.ok(
        "remote.save",
        json!({ "target": { "name": "pi", "host": "192.168.0.42", "user": "pi" } }),
    );
    write_tool(
        &c.bin().join("ssh"),
        "#!/bin/sh\nprintf 'aarch64\\nLinux 6.6\\ngdbserver=\\npython3=\\nrsync=\\n'\n",
    );
    let contacts = c.root.join(".kinein").join("remote-contacts.json");
    c.ok("remote.probe", json!({ "name": "pi" }));
    assert_eq!(c.event("event.remote.probed")["success"], true);
    assert!(
        std::fs::read_to_string(&contacts)
            .unwrap()
            .contains("\"pi\"")
    );

    std::fs::remove_file(&contacts).unwrap();
    let outside = c.root.parent().unwrap().join("fora");
    std::fs::create_dir_all(&outside).unwrap();
    link(&outside.join("contatos"), &contacts);
    c.ok("remote.probe", json!({ "name": "pi" }));
    assert_eq!(c.event("event.remote.probed")["success"], true);
    assert_eq!(listing(&outside), Vec::<String>::new());
    assert!(
        std::fs::symlink_metadata(&contacts)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
