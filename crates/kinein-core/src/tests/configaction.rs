//! Configuration Actions (`configAction.list` / `preview` / `apply`).
//!
//! **Toda acao e verificada contra ARQUIVO REAL**, nunca contra mock: o teste
//! abre um workspace de verdade num diretorio temporario, manda a requisicao
//! pelo dispatch do `Core` e le o disco depois. Foi assim que a etapa 1 de
//! 2026-08-30 descobriu que uma sonda podia ficar verde com o produto quebrado
//! — olhando a resposta em vez do efeito.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::{Core, RequestOutcome};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-configaction-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

const CMAKELISTS: &str = "cmake_minimum_required(VERSION 3.24)\n\
                          \n\
                          project(demo CXX)\n\
                          \n\
                          add_executable(demo\n    src/main.cpp\n)\n";

const CARGO_TOML: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
                          \n[dependencies]\nserde = \"1.0\"\n";

fn cmake_workspace(name: &str) -> PathBuf {
    let dir = temp_dir(name);
    std::fs::write(dir.join("CMakeLists.txt"), CMAKELISTS).unwrap();
    dir
}

fn cargo_workspace(name: &str) -> PathBuf {
    let dir = temp_dir(name);
    std::fs::write(dir.join("Cargo.toml"), CARGO_TOML).unwrap();
    dir
}

fn open(core: &mut Core, dir: &Path) {
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");
}

fn call(core: &mut Core, method: &str, params: Value) -> RequestOutcome {
    core.handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
}

fn ok(core: &mut Core, method: &str, params: &Value) -> Value {
    let outcome = call(core, method, params.clone());
    let response = outcome.response();
    assert!(
        response.error.is_none(),
        "{method} {params} falhou: {:?}",
        response.error
    );
    response.result.clone().unwrap()
}

fn error_code(core: &mut Core, method: &str, params: Value) -> JsonRpcErrorCode {
    let outcome = call(core, method, params);
    outcome.response().error.as_ref().unwrap().code
}

/// Aplica uma acao devolvendo a lista de arquivos escritos.
fn apply(core: &mut Core, id: &str, params: &Value) -> Value {
    let preview = ok(
        core,
        "configAction.preview",
        &json!({ "id": id, "params": params }),
    );
    let expected: Vec<Value> = preview["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let mut entry = json!({ "path": file["path"] });
            if let Some(before) = file.get("before") {
                entry["content"] = before.clone();
            }
            entry
        })
        .collect();
    ok(
        core,
        "configAction.apply",
        &json!({ "id": id, "params": params, "expected": expected }),
    )
}

fn action<'list>(list: &'list Value, id: &str) -> &'list Value {
    list["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["id"] == id)
        .unwrap_or_else(|| panic!("acao {id} ausente da lista"))
}

#[test]
fn configaction_methods_require_an_open_workspace() {
    let mut core = core_with_empty_search_path("configaction-guarda");
    for method in [
        "configAction.list",
        "configAction.preview",
        "configAction.apply",
    ] {
        assert_eq!(
            error_code(&mut core, method, json!({ "id": "cargo.check" })),
            JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }
}

/// Spec 9.2 §1: a lista e filtrada pelo build system ATIVO, nao pelo compilador.
#[test]
fn the_list_is_filtered_by_the_active_build_system() {
    let dir = cmake_workspace("escopo");
    let mut core = core_with_empty_search_path("configaction-escopo");
    open(&mut core, &dir);

    let list = ok(&mut core, "configAction.list", &json!({}));
    let ids: Vec<&str> = list["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|action| action["id"].as_str().unwrap())
        .collect();
    // Eram as 10 da spec de MVP §12.1; `findPackage` e `fetchContent`
    // entraram em 2026-09-03 com o dominio `library` (roadmaps/35 etapa 20).
    assert_eq!(ids.len(), 12, "as 12 acoes CMake");
    assert!(
        ids.iter().all(|id| id.starts_with("cmake.")),
        "projeto CMake nao pode ver acao Cargo: {ids:?}"
    );
    assert_eq!(list["activeBuildSystems"], json!(["cmake"]));

    // O usuario pode pedir para ver o que o escopo escondeu (spec 9.2 §25).
    let todas = ok(
        &mut core,
        "configAction.list",
        &json!({ "includeHiddenByScope": true }),
    );
    assert_eq!(todas["actions"].as_array().unwrap().len(), 18);
    assert_eq!(
        action(&todas, "cargo.addDependency")["state"],
        "hiddenByScope"
    );

    // E fora do escopo continua sendo recusa de verdade, nao so ocultacao.
    assert_eq!(
        error_code(
            &mut core,
            "configAction.preview",
            json!({ "id": "cargo.addDependency", "params": { "name": "x", "version": "1" } })
        ),
        JsonRpcErrorCode::InvalidParams
    );
}

#[test]
fn a_mixed_workspace_sees_both_toolboxes() {
    let dir = cmake_workspace("misto");
    std::fs::write(dir.join("Cargo.toml"), CARGO_TOML).unwrap();
    let mut core = core_with_empty_search_path("configaction-misto");
    open(&mut core, &dir);

    let list = ok(&mut core, "configAction.list", &json!({}));
    assert_eq!(list["actions"].as_array().unwrap().len(), 18);
    assert_eq!(action(&list, "cargo.addDependency")["state"], "available");
    assert_eq!(action(&list, "cmake.addExecutable")["state"], "available");
}

/// O estado nao e decorativo: ele muda com o que esta no disco.
#[test]
fn the_state_of_each_action_is_measured_in_the_workspace() {
    let dir = cmake_workspace("estado");
    let mut core = core_with_empty_search_path("configaction-estado");
    open(&mut core, &dir);

    let antes = ok(&mut core, "configAction.list", &json!({}));
    // Sem CDB nenhuma, habilitar compile_commands e RECOMENDADO.
    assert_eq!(
        action(&antes, "cmake.enableCompileCommands")["state"],
        "recommended"
    );
    // Sem configure, inspecionar o cache nao e possivel ainda.
    assert_eq!(
        action(&antes, "cmake.inspectCache")["state"],
        "partiallyAvailable"
    );
    assert_eq!(
        action(&antes, "cmake.repairBuildDir")["state"],
        "unavailable"
    );

    apply(&mut core, "cmake.enableCompileCommands", &json!({}));

    let depois = ok(&mut core, "configAction.list", &json!({}));
    let acao = action(&depois, "cmake.enableCompileCommands");
    assert_eq!(acao["state"], "unavailable");
    assert!(
        acao["reason"]
            .as_str()
            .unwrap()
            .contains("CMAKE_EXPORT_COMPILE_COMMANDS")
    );
}

/// Preview e um PLANO: ele nao pode encostar no disco.
#[test]
fn preview_shows_the_diff_without_writing_anything() {
    let dir = cmake_workspace("preview");
    let mut core = core_with_empty_search_path("configaction-preview");
    open(&mut core, &dir);

    let preview = ok(
        &mut core,
        "configAction.preview",
        &json!({
            "id": "cmake.addExecutable",
            "params": { "name": "extra", "sources": "src/extra.cpp" },
        }),
    );

    assert_eq!(preview["files"][0]["path"], "CMakeLists.txt");
    assert_eq!(preview["files"][0]["before"], CMAKELISTS);
    assert!(
        preview["files"][0]["after"]
            .as_str()
            .unwrap()
            .contains("add_executable(extra")
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap(),
        CMAKELISTS,
        "o preview escreveu no disco"
    );
}

/// A mesma barreira do `fs.write` (`ARCHITECTURE` §7.1), aplicada ao consentimento.
#[test]
fn apply_refuses_when_the_file_moved_after_the_preview() {
    let dir = cmake_workspace("corrida");
    let mut core = core_with_empty_search_path("configaction-corrida");
    open(&mut core, &dir);

    let preview = ok(
        &mut core,
        "configAction.preview",
        &json!({
            "id": "cmake.addExecutable",
            "params": { "name": "extra", "sources": "src/extra.cpp" },
        }),
    );
    let snapshot = preview["files"][0]["before"].clone();

    // Alguem editou o arquivo entre o preview e o Apply.
    std::fs::write(dir.join("CMakeLists.txt"), "outro conteudo\n").unwrap();

    let codigo = error_code(
        &mut core,
        "configAction.apply",
        json!({
            "id": "cmake.addExecutable",
            "params": { "name": "extra", "sources": "src/extra.cpp" },
            "expected": [{ "path": "CMakeLists.txt", "content": snapshot }],
        }),
    );

    assert_eq!(codigo, JsonRpcErrorCode::FileChanged);
    assert_eq!(
        std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap(),
        "outro conteudo\n",
        "recusar tem que significar NAO ESCREVER"
    );
}

/// As 10 acoes `CMake` da spec de MVP §12.1, cada uma contra o arquivo real.
#[test]
fn every_cmake_action_has_a_verified_effect_on_disk() {
    let dir = cmake_workspace("cmake-todas");
    let mut core = core_with_empty_search_path("configaction-cmake-todas");
    open(&mut core, &dir);

    // 1. compile_commands.json — antes do primeiro target, como a doc do CMake
    //    exige (a variavel inicializa a propriedade de CADA target criado).
    apply(&mut core, "cmake.enableCompileCommands", &json!({}));
    let texto = std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap();
    let posicao_da_variavel = texto.find("set(CMAKE_EXPORT_COMPILE_COMMANDS ON)").unwrap();
    assert!(posicao_da_variavel < texto.find("add_executable(demo").unwrap());
    assert!(texto.starts_with("cmake_minimum_required(VERSION 3.24)\n"));

    // 2 e 3. Presets Debug e Release, no arquivo que ainda nem existia.
    apply(&mut core, "cmake.createDebugPreset", &json!({}));
    apply(&mut core, "cmake.createReleasePreset", &json!({}));
    let presets: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("CMakePresets.json")).unwrap())
            .unwrap();
    assert_eq!(presets["version"], 6);
    assert_eq!(presets["configurePresets"][0]["name"], "debug");
    assert_eq!(
        presets["configurePresets"][0]["cacheVariables"]["CMAKE_BUILD_TYPE"],
        "Debug"
    );
    assert_eq!(presets["configurePresets"][1]["name"], "release");

    // 4 e 5. Targets novos.
    apply(
        &mut core,
        "cmake.addExecutable",
        &json!({ "name": "ferramenta", "sources": "src/tool.cpp" }),
    );
    apply(
        &mut core,
        "cmake.addStaticLibrary",
        &json!({ "name": "nucleo", "sources": "src/nucleo.cpp src/aux.cpp" }),
    );

    // 6, 7 e 8. Comandos sobre um target existente.
    apply(
        &mut core,
        "cmake.addSourceToTarget",
        &json!({ "target": "nucleo", "sources": "src/extra.cpp" }),
    );
    apply(
        &mut core,
        "cmake.addIncludeDirectory",
        &json!({ "target": "nucleo", "directories": "include", "visibility": "PUBLIC" }),
    );
    apply(
        &mut core,
        "cmake.addTargetLinkLibraries",
        &json!({ "target": "ferramenta", "libraries": "nucleo" }),
    );

    let texto = std::fs::read_to_string(dir.join("CMakeLists.txt")).unwrap();
    assert!(texto.contains("add_executable(ferramenta\n    src/tool.cpp\n)\n"));
    assert!(texto.contains("add_library(nucleo STATIC\n    src/nucleo.cpp\n    src/aux.cpp\n)\n"));
    assert!(texto.contains("target_sources(nucleo PRIVATE\n    src/extra.cpp\n)\n"));
    assert!(texto.contains("target_include_directories(nucleo PUBLIC\n    include\n)\n"));
    assert!(texto.contains("target_link_libraries(ferramenta PRIVATE\n    nucleo\n)\n"));

    // 9. Inspecionar o cache: le o CMakeCache.txt do build dir da IDE.
    let build = dir.join(".kinein/build");
    std::fs::create_dir_all(&build).unwrap();
    std::fs::write(
        build.join("CMakeCache.txt"),
        "CMAKE_BUILD_TYPE:STRING=Debug\nCMAKE_HOME_DIRECTORY:INTERNAL=/tmp\n",
    )
    .unwrap();
    let inspecao = ok(
        &mut core,
        "configAction.preview",
        &json!({ "id": "cmake.inspectCache" }),
    );
    assert_eq!(inspecao["report"], json!(["CMAKE_BUILD_TYPE:STRING=Debug"]));
    assert!(inspecao["files"].as_array().unwrap().is_empty());

    // 10. Reparar o build dir: some do disco de verdade.
    assert!(build.is_dir());
    apply(&mut core, "cmake.repairBuildDir", &json!({}));
    assert!(!build.exists(), "o build dir continuou la");
}

/// As 6 acoes Cargo da spec de MVP §12.2, cada uma contra o arquivo real.
#[test]
fn every_cargo_action_has_a_verified_effect_on_disk() {
    let dir = cargo_workspace("cargo-todas");
    let mut core = core_with_empty_search_path("configaction-cargo-todas");
    open(&mut core, &dir);

    // 1, 2 e 3. Dependencia, dev-dependency e feature.
    apply(
        &mut core,
        "cargo.addDependency",
        &json!({ "name": "anyhow", "version": "1.0" }),
    );
    apply(
        &mut core,
        "cargo.addDevDependency",
        &json!({ "name": "proptest", "version": "1.5" }),
    );
    apply(
        &mut core,
        "cargo.addFeature",
        &json!({ "name": "extra", "enables": "serde/derive" }),
    );
    // 4. Edition.
    apply(&mut core, "cargo.setEdition", &json!({ "edition": "2024" }));

    let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("\nserde = \"1.0\"\nanyhow = \"1.0\"\n"),
        "{manifest}"
    );
    assert!(
        manifest.contains("[dev-dependencies]\nproptest = \"1.5\"\n"),
        "{manifest}"
    );
    assert!(
        manifest.contains("[features]\nextra = [\"serde/derive\"]\n"),
        "{manifest}"
    );
    assert!(manifest.contains("edition = \"2024\""), "{manifest}");
    assert!(!manifest.contains("edition = \"2021\""), "{manifest}");
    assert!(
        manifest.starts_with("[package]\nname = \"demo\""),
        "{manifest}"
    );

    // 5. cargo check REUSA o job que ja existe; sem JobManager neste Core, o
    //    erro vem do dono do metodo — que e a prova de que nao ha executor novo.
    let outcome = call(
        &mut core,
        "configAction.apply",
        json!({ "id": "cargo.check" }),
    );
    let erro = outcome.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InternalError);
    assert!(erro.message.contains("jobs"), "{}", erro.message);

    // 6. Run config: escrita pelo dominio dono do .kinein/runconfigs.json.
    apply(
        &mut core,
        "cargo.createRunConfig",
        &json!({ "name": "Servidor", "command": "cargo run --bin server" }),
    );
    let configs = ok(&mut core, "runConfig.list", &json!({}));
    assert_eq!(configs["configs"][0]["name"], "Servidor");
    assert_eq!(configs["activeId"], "cfg-1");
    assert!(dir.join(".kinein/runconfigs.json").is_file());
}

/// Recusar e o comportamento CERTO: a IDE nao adivinha manifest do usuario.
#[test]
fn actions_refuse_instead_of_corrupting_the_project() {
    let dir = cargo_workspace("recusa");
    let mut core = core_with_empty_search_path("configaction-recusa");
    open(&mut core, &dir);

    for (id, params) in [
        // Dependencia que ja esta no arquivo.
        (
            "cargo.addDependency",
            json!({ "name": "serde", "version": "1" }),
        ),
        // Nome que nao e identificador de crate.
        (
            "cargo.addDependency",
            json!({ "name": "ser de", "version": "1" }),
        ),
        // Versao ausente: a IDE nao resolve versao na rede.
        ("cargo.addDependency", json!({ "name": "anyhow" })),
        // Edition que nao existe.
        ("cargo.setEdition", json!({ "edition": "2030" })),
        // Id inexistente.
        ("cargo.naoExiste", json!({})),
    ] {
        assert_eq!(
            error_code(
                &mut core,
                "configAction.preview",
                json!({ "id": id, "params": params })
            ),
            JsonRpcErrorCode::InvalidParams,
            "{id} {params}"
        );
    }

    assert_eq!(
        std::fs::read_to_string(dir.join("Cargo.toml")).unwrap(),
        CARGO_TOML,
        "uma recusa nao pode ter escrito nada"
    );
}

/// Alvo inexistente e nome reservado pela CMP0037 sao recusados com motivo.
#[test]
fn cmake_actions_validate_target_names_and_existence() {
    let dir = cmake_workspace("validacao");
    let mut core = core_with_empty_search_path("configaction-validacao");
    open(&mut core, &dir);

    let inexistente = call(
        &mut core,
        "configAction.preview",
        json!({
            "id": "cmake.addTargetLinkLibraries",
            "params": { "target": "fantasma", "libraries": "m" },
        }),
    );
    let erro = inexistente.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(erro.message.contains("demo"), "diz quais targets existem");

    let reservado = call(
        &mut core,
        "configAction.preview",
        json!({
            "id": "cmake.addExecutable",
            "params": { "name": "install", "sources": "src/a.cpp" },
        }),
    );
    assert!(
        reservado
            .response()
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("CMP0037")
    );

    let fuga = call(
        &mut core,
        "configAction.preview",
        json!({
            "id": "cmake.addExecutable",
            "params": { "name": "app", "sources": "../fora/main.cpp" },
        }),
    );
    assert!(
        fuga.response()
            .error
            .as_ref()
            .unwrap()
            .message
            .contains("sai da raiz")
    );
}
