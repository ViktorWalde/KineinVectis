//! INF-18 (`0.223.0`): a reprodução do 62 §3.4. No lugar de cada arquivo do
//! projeto ou de ferramenta que a abertura e o status do `CMake` leem, o
//! arquivo válido (o controle), um FIFO, uma pasta, um link para o mesmo
//! arquivo fora do projeto e o arquivo acima de 1 MiB. Pelo IPC, o
//! `workspace.open`, o `cmake.status` e os pedidos que a medida viu ler o
//! arquivo respondem; o FIFO e a pasta valem como ausentes, e o
//! `projectProblem` diz o caminho e o motivo; o link e o arquivo grande são
//! lidos como o sistema de build os lê; e nada do `.kinein` toma o lugar do
//! arquivo recusado. Os pedidos rodam com prazo, e o teste abre o outro lado
//! do FIFO se a leitura prender.
//!
//! No Windows não há FIFO, e a junção só liga pasta: lá valem a pasta e o
//! arquivo grande (63 §E.2).

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::fsops::{fifo, link, unblocked};
use kinein_protocol::JsonRpcRequest;

/// O que fica no lugar do arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placed {
    /// O arquivo válido: o controle, que o dono usa.
    Regular,
    Fifo,
    Folder,
    /// Um link para o mesmo arquivo válido, fora do projeto.
    Link,
    /// O arquivo válido com o começo acima de 1 MiB, o teto do `.kinein`.
    Large,
}

const PLACED: [Placed; 5] = [
    Placed::Regular,
    Placed::Fifo,
    Placed::Folder,
    Placed::Link,
    Placed::Large,
];

/// Como o arquivo cresce acima de 1 MiB sem mudar o que diz.
#[derive(Clone, Copy)]
enum Padding {
    /// Linhas de comentário com `#` antes do conteúdo.
    Comment,
    /// Espaço antes do conteúdo, para o JSON.
    Space,
}

/// Um arquivo que a abertura ou o status leem.
struct Owner {
    /// O caminho a partir da raiz, com `/`.
    file: &'static str,
    /// O conteúdo válido; `%ROOT%` vira a raiz do projeto.
    valid: &'static str,
    padding: Padding,
    /// Os outros arquivos do projeto, a partir da raiz: caminho e conteúdo.
    before: &'static [(&'static str, &'static str)],
    /// O texto que só aparece nas respostas quando o arquivo foi lido.
    marker: Option<&'static str>,
}

/// Os pedidos da medida que leram um arquivo do projeto (62 §3.4), depois do
/// `workspace.open`.
const REQUESTS: &[(&str, &str)] = &[
    ("settings.get", "{}"),
    ("toolchain.get", "{}"),
    ("cmake.status", "{}"),
    ("configAction.list", "{}"),
    ("project.model", "{}"),
    ("python.status", "{}"),
    ("build.size", "{}"),
    ("index.context", r#"{"path": "src/main.c"}"#),
];

/// O que um arquivo do `.kinein` diria se tomasse o lugar do recusado.
const DECOYS: [&str; 2] = ["kinein-marca", "KINEIN_MARCA"];
const CMAKE: (&str, &str) = ("CMakeLists.txt", "project(p C)\n");
const MAIN: (&str, &str) = ("src/main.c", "int main(void) { return 0; }\n");
/// O `.kinein/build` com o cache e a CDB que não são da configuração ativa:
/// nenhum deles pode aparecer no lugar do arquivo recusado. Sem configuração
/// registrada, a CDB do `.kinein/build` vale no automático (C1-20): ela só
/// entra com o registro.
const KINEIN_DECOY: [(&str, &str); 2] = [
    (
        ".kinein/build/CMakeCache.txt",
        "CMAKE_C_COMPILER:FILEPATH=/kinein-marca/cc\n",
    ),
    (
        ".kinein/build/compile_commands.json",
        r#"[{"directory": "%ROOT%", "file": "%ROOT%/src/main.c",
            "arguments": ["cc", "-DKINEIN_MARCA=1", "-c", "%ROOT%/src/main.c"]}]"#,
    ),
];
const DEV_PRESETS: (&str, &str) = (
    "CMakePresets.json",
    r#"{"version": 6, "configurePresets": [{"name": "dev", "generator": "Unix Makefiles",
        "binaryDir": "${sourceDir}/build/dev"}]}"#,
);
const DEV_CACHE: (&str, &str) = (
    "build/dev/CMakeCache.txt",
    "CMAKE_BUILD_TYPE:STRING=Debug\n",
);
const DEV_CHOSEN: (&str, &str) = (".kinein/configuration.json", r#"{"preset": "dev"}"#);
const SIMPLE: &[(&str, &str)] = &[CMAKE, MAIN];
const WITH_DEV: &[(&str, &str)] = &[
    CMAKE,
    MAIN,
    DEV_PRESETS,
    DEV_CACHE,
    DEV_CHOSEN,
    KINEIN_DECOY[0],
    KINEIN_DECOY[1],
];

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-project-file-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

fn with_root(text: &str, root: &Path) -> String {
    let quoted = serde_json::to_string(root.to_str().unwrap()).unwrap();
    text.replace("%ROOT%", &quoted[1..quoted.len() - 1])
}

fn write(path: &Path, body: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

/// O projeto do caso, com `placed` no lugar de `owner.file`; devolve a raiz,
/// a pasta de fora e o caminho do arquivo.
fn placed_project(owner: &Owner, placed: Placed) -> (PathBuf, PathBuf, PathBuf) {
    let dir = base(&format!("{}-{placed:?}", owner.file.replace('/', "_")));
    let root = dir.join("projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(&outside).unwrap();
    for (file, body) in owner.before {
        if *file != owner.file {
            write(&root.join(file), &with_root(body, &root));
        }
    }
    let valid = with_root(owner.valid, &root);
    let path = root.join(owner.file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    match placed {
        Placed::Regular => write(&path, &valid),
        Placed::Fifo => assert!(fifo(&path)),
        Placed::Folder => std::fs::create_dir_all(&path).unwrap(),
        Placed::Link => {
            let target = outside.join(path.file_name().unwrap());
            write(&target, &valid);
            link(&target, &path);
        }
        Placed::Large => {
            let size = usize::try_from(crate::fsops::MAX_READ_BYTES).unwrap() + 1;
            let padding = match owner.padding {
                Padding::Comment => "#".repeat(79) + "\n",
                Padding::Space => " ".repeat(79) + "\n",
            };
            write(&path, &(padding.repeat(size / 80 + 1) + &valid));
        }
    }
    (root, outside, path)
}

/// A abertura e os pedidos, com prazo; cada resposta em JSON.
fn responses(root: &Path, held: &Path) -> Vec<Value> {
    let project = root.to_owned();
    unblocked(held, move || {
        let mut core = core_with_empty_search_path("project-file");
        let mut requests = vec![("workspace.open", json!({ "path": project }))];
        for (method, params) in REQUESTS {
            requests.push((method, serde_json::from_str(params).unwrap()));
        }
        requests
            .into_iter()
            .zip(1_i64..)
            .map(|((method, params), id)| {
                let outcome = core.handle_request(&JsonRpcRequest::new(id, method, Some(params)));
                serde_json::to_value(outcome.response()).unwrap()
            })
            .collect()
    })
}

/// Cada caso de `owner`: abre o projeto e manda os pedidos, e confere o
/// `projectProblem`, o uso do arquivo e que nada do `.kinein` o substitui.
fn each_placement(owner: &'static Owner) {
    for placed in PLACED {
        if cfg!(windows) && matches!(placed, Placed::Fifo | Placed::Link) {
            continue;
        }
        let (root, outside, path) = placed_project(owner, placed);
        let answers = responses(&root, &path);
        let case = format!("{} {placed:?}", owner.file);
        let text = serde_json::to_string(&answers).unwrap();
        let opened = &answers[0]["result"];
        assert!(opened.is_object(), "{case}: {}", answers[0]);
        let problem = matches!(placed, Placed::Fifo | Placed::Folder)
            .then(|| format!("{} não é um arquivo regular", owner.file));
        assert_eq!(
            opened.get("projectProblem").and_then(Value::as_str),
            problem.as_deref(),
            "{case}"
        );
        // O `cmake.status` leva o mesmo campo quando o projeto é CMake.
        let status = &answers[1 + REQUESTS
            .iter()
            .position(|(method, _)| *method == "cmake.status")
            .unwrap()];
        if let Some(result) = status.get("result") {
            assert_eq!(
                result.get("projectProblem").and_then(Value::as_str),
                problem.as_deref(),
                "{case}: {status}"
            );
        } else {
            assert_eq!(owner.file, "CMakeLists.txt", "{case}: {status}");
        }
        if let Some(marker) = owner.marker {
            assert_eq!(text.contains(marker), problem.is_none(), "{case}: {text}");
        }
        for decoy in DECOYS {
            assert!(!text.contains(decoy), "{case}: {text}");
        }
        let left = if placed == Placed::Link {
            vec![path.file_name().unwrap().to_string_lossy().into_owned()]
        } else {
            Vec::new()
        };
        let mut names: Vec<String> = std::fs::read_dir(&outside)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, left, "{case}");
    }
}

/// O detector de frameworks lê o `CMakeLists.txt` inteiro, como o `CMake`.
static CMAKE_LISTS: Owner = Owner {
    file: "CMakeLists.txt",
    valid: "set(PICO_BOARD fora_marca)\npico_sdk_init()\nproject(p C)\n",
    padding: Padding::Comment,
    before: &[MAIN],
    marker: Some("PICO_BOARD fora_marca"),
};

static PLATFORMIO: Owner = Owner {
    file: "platformio.ini",
    valid: "[env:fora_marca]\nplatform = espressif32\n",
    padding: Padding::Comment,
    before: SIMPLE,
    marker: Some("fora_marca"),
};

static CARGO: Owner = Owner {
    file: "Cargo.toml",
    valid: "[package]\nname = \"fora-marca\"\nversion = \"0.1.0\"\n",
    padding: Padding::Comment,
    before: SIMPLE,
    marker: None,
};

static PYPROJECT: Owner = Owner {
    file: "pyproject.toml",
    valid: "[project]\nname = \"fora-marca\"\n",
    padding: Padding::Comment,
    before: SIMPLE,
    marker: None,
};

static PRESETS: Owner = Owner {
    file: "CMakePresets.json",
    valid: r#"{"version": 6, "configurePresets": [{"name": "fora-marca",
        "binaryDir": "${sourceDir}/build/fora-marca"}]}"#,
    padding: Padding::Space,
    before: &[CMAKE, MAIN, KINEIN_DECOY[0]],
    marker: Some("fora-marca"),
};

static USER_PRESETS: Owner = Owner {
    file: "CMakeUserPresets.json",
    valid: r#"{"version": 6, "configurePresets": [{"name": "fora-marca",
        "binaryDir": "${sourceDir}/build/fora-marca"}]}"#,
    padding: Padding::Space,
    before: &[CMAKE, MAIN, KINEIN_DECOY[0]],
    marker: Some("fora-marca"),
};

/// Um include dos presets: o `CMake` o lê como os da raiz.
static INCLUDED_PRESETS: Owner = Owner {
    file: "extra-presets.json",
    valid: r#"{"version": 6, "configurePresets": [{"name": "fora-marca",
        "binaryDir": "${sourceDir}/build/fora-marca"}]}"#,
    padding: Padding::Space,
    before: &[
        CMAKE,
        MAIN,
        (
            "CMakePresets.json",
            r#"{"version": 6, "include": ["extra-presets.json"]}"#,
        ),
        KINEIN_DECOY[0],
    ],
    marker: Some("fora-marca"),
};

static CLANGD: Owner = Owner {
    file: ".clangd",
    valid: "CompileFlags:\n  CompilationDatabase: fora-marca\n",
    padding: Padding::Comment,
    before: &[
        CMAKE,
        MAIN,
        (
            "fora-marca/compile_commands.json",
            r#"[{"directory": "%ROOT%", "file": "%ROOT%/src/main.c",
                "arguments": ["cc", "-c", "%ROOT%/src/main.c"]}]"#,
        ),
    ],
    marker: Some(r#""cdbDirectory":"fora-marca""#),
};

static IDE_CACHE: Owner = Owner {
    file: ".kinein/build/CMakeCache.txt",
    valid: "CMAKE_C_COMPILER:FILEPATH=/fora-marca/cc\n",
    padding: Padding::Comment,
    before: SIMPLE,
    marker: Some("fora-marca"),
};

static IDE_CDB: Owner = Owner {
    file: ".kinein/build/compile_commands.json",
    valid: r#"[{"directory": "%ROOT%", "file": "%ROOT%/src/main.c",
        "arguments": ["cc", "-DFORA_MARCA=1", "-c", "%ROOT%/src/main.c"]}]"#,
    padding: Padding::Space,
    before: &[CMAKE, MAIN, (".kinein/build/CMakeCache.txt", "")],
    marker: Some("FORA_MARCA"),
};

static PRESET_CACHE: Owner = Owner {
    file: "build/dev/CMakeCache.txt",
    valid: "CMAKE_C_COMPILER:FILEPATH=/fora-marca/cc\n",
    padding: Padding::Comment,
    before: WITH_DEV,
    marker: Some("fora-marca"),
};

static PRESET_CDB: Owner = Owner {
    file: "build/dev/compile_commands.json",
    valid: r#"[{"directory": "%ROOT%", "file": "%ROOT%/src/main.c",
        "arguments": ["cc", "-DFORA_MARCA=1", "-c", "%ROOT%/src/main.c"]}]"#,
    padding: Padding::Space,
    before: WITH_DEV,
    marker: Some("FORA_MARCA"),
};

static PRESET_KIT: Owner = Owner {
    file: "build/dev/.kinein-kit",
    valid: "fora-marca\n",
    padding: Padding::Comment,
    before: WITH_DEV,
    marker: Some("fora-marca"),
};

static PRESET_MARK: Owner = Owner {
    file: "build/dev/.kinein-preset",
    valid: "fora-marca\n",
    padding: Padding::Comment,
    before: WITH_DEV,
    marker: Some("fora-marca"),
};

static PENDING: Owner = Owner {
    file: "build/dev/.kinein-configure-pending",
    valid: r#"{"preset": "dev", "kit": "fora-marca"}"#,
    padding: Padding::Space,
    before: WITH_DEV,
    // A tentativa pendente deixa o contexto desatualizado.
    marker: Some("configure não concluído"),
};

static RIGOR: Owner = Owner {
    file: "build/dev/.kinein-rigor",
    valid: "fora-marca\n",
    padding: Padding::Comment,
    before: WITH_DEV,
    marker: None,
};

#[test]
fn the_cmake_lists_in_each_placement() {
    each_placement(&CMAKE_LISTS);
}

#[test]
fn the_platformio_ini_in_each_placement() {
    each_placement(&PLATFORMIO);
}

#[test]
fn the_cargo_manifest_in_each_placement() {
    each_placement(&CARGO);
}

#[test]
fn the_pyproject_in_each_placement() {
    each_placement(&PYPROJECT);
}

#[test]
fn the_presets_in_each_placement() {
    each_placement(&PRESETS);
}

#[test]
fn the_user_presets_in_each_placement() {
    each_placement(&USER_PRESETS);
}

#[test]
fn the_included_presets_in_each_placement() {
    each_placement(&INCLUDED_PRESETS);
}

#[test]
fn the_clangd_file_in_each_placement() {
    each_placement(&CLANGD);
}

#[test]
fn the_cache_of_the_ide_folder_in_each_placement() {
    each_placement(&IDE_CACHE);
}

#[test]
fn the_cdb_of_the_ide_folder_in_each_placement() {
    each_placement(&IDE_CDB);
}

#[test]
fn the_cache_of_the_preset_folder_in_each_placement() {
    each_placement(&PRESET_CACHE);
}

#[test]
fn the_cdb_of_the_preset_folder_in_each_placement() {
    each_placement(&PRESET_CDB);
}

#[test]
fn the_kit_mark_of_the_preset_folder_in_each_placement() {
    each_placement(&PRESET_KIT);
}

#[test]
fn the_preset_mark_of_the_preset_folder_in_each_placement() {
    each_placement(&PRESET_MARK);
}

#[test]
fn the_pending_configure_of_the_preset_folder_in_each_placement() {
    each_placement(&PENDING);
}

#[test]
fn the_rigor_mark_of_the_preset_folder_in_each_placement() {
    each_placement(&RIGOR);
}

/// A abertura e o `index.context` do cabeçalho, num core novo: o que o
/// contexto diz dele.
fn header_context(root: &Path) -> String {
    let mut core = core_with_empty_search_path("project-file-header");
    for (method, params) in [
        ("workspace.open", json!({ "path": root })),
        ("index.context", json!({ "path": "include/valor.h" })),
    ] {
        let outcome = core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)));
        if method == "index.context" {
            return serde_json::to_string(outcome.response()).unwrap();
        }
    }
    unreachable!()
}

/// O `.clangd` de uma pasta acima do cabeçalho, que o contexto do cabeçalho
/// lê para saber se a base é manual: o que não é arquivo regular vale como
/// ausente, sem esperar num FIFO. Fica fora do `projectProblem`, que só
/// diz o da raiz.
#[test]
fn the_clangd_file_above_a_header_is_read_as_absent() {
    let root = base("clangd-cabecalho");
    for (file, body) in [
        CMAKE,
        MAIN,
        ("include/valor.h", "#define VALOR 1\n"),
        (".kinein/build/CMakeCache.txt", ""),
        (
            ".kinein/build/compile_commands.json",
            r#"[{"directory": "%ROOT%", "file": "%ROOT%/src/main.c",
                "arguments": ["cc", "-I%ROOT%/include", "-c", "%ROOT%/src/main.c"]}]"#,
        ),
    ] {
        write(&root.join(file), &with_root(body, &root));
    }
    let project = root.clone();
    crate::fsops::reads_as_absent(
        &root.join("include").join(".clangd"),
        b"CompileFlags:\n  CompilationDatabase: fora\n",
        move || header_context(&project).contains("base de compilação configurada manualmente"),
    );
}

/// Os rascunhos da abertura e o arquivo de cada um: o que não é arquivo
/// regular vale como o que sumiu, sem esperar num FIFO; o rascunho sai, e a
/// abertura não o recupera.
#[test]
fn a_draft_whose_file_is_not_a_regular_file_is_not_recovered() {
    let root = base("rascunho");
    for (file, body) in SIMPLE {
        write(&root.join(file), body);
    }
    let project = root.clone();
    let draft = root.join("src").join("rascunho.c");
    crate::fsops::reads_as_absent(&draft.clone(), b"int disco;\n", move || {
        let store = crate::db::DraftStore::open(&project).unwrap();
        store
            .save(draft.to_str().unwrap(), "int rascunho;\n")
            .unwrap();
        drop(store);
        let mut core = core_with_empty_search_path("project-file-rascunho");
        core.enable_persistence(project.with_file_name("global"));
        let outcome = core.handle_request(&JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({ "path": project })),
        ));
        let opened = serde_json::to_value(outcome.response()).unwrap();
        opened["result"]["drafts"].clone()
    });
}
