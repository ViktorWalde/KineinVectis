//! C1-26: `matches` e `notMatches` (consolidação C §19), de ponta a ponta
//! pelo core, com `CMake` 4.4.5, GCC e Ninja reais.
//!
//! Até o C1-26 a leitura nativa seguia avaliando depois de uma forma que ela
//! não decide e recusava os presets pela macro inválida que o `CMake` nem lê:
//! com o C1-25, o configure dizia "o `CMake` aceita os presets, mas a IDE não os
//! lê". Agora a lista do `CMake` decide, e sem ele o item diz que a
//! `condition` fica verdadeira.

use super::{Project, environment::project};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};
use std::fs;

const LISTS: &str = "cmake_minimum_required(VERSION 3.24)\nproject(condicao LANGUAGES C)\nadd_executable(condicao src/main.c)\n";

const NOTE: &str = "a condition usa matches ou notMatches, que só o CMake decide; sem ele, a IDE a dá por verdadeira";

const PRESETS: &str = r#"{"version":6,"configurePresets":[
  {"name":"ok","generator":"Ninja","binaryDir":"${sourceDir}/build/ok","cacheVariables":{"CMAKE_C_COMPILER":"gcc"}},
  {"name":"rx","generator":"Ninja","binaryDir":"${sourceDir}/build/rx",
   "condition":{"type":"matches","string":"a1","regex":"^a\\d$"}},
  {"name":"anybad","generator":"Ninja","binaryDir":"${sourceDir}/build/anybad","cacheVariables":{"CMAKE_C_COMPILER":"gcc"},
   "condition":{"type":"anyOf","conditions":[
     {"type":"matches","string":"a1","regex":"^a1$"},
     {"type":"equals","lhs":"${nope}","rhs":"x"}]}}]}"#;

/// Cada preset com a nota dele, se tiver.
type Notes = Vec<(String, Value)>;

/// A lista e a `configurations`, cada preset com a nota dele.
fn notes(p: &mut Project) -> (Notes, Notes) {
    let pair = |item: &Value, key: &str| {
        (
            item[key].as_str().unwrap().to_owned(),
            item.get("conditionNote").cloned().unwrap_or(Value::Null),
        )
    };
    let list = p.call("cmake.presets.list", json!({}))["presets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| pair(item, "name"))
        .collect();
    let menu = p.call("cmake.status", json!({}))["configurations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| pair(item, "preset"))
        .collect();
    (list, menu)
}

#[test]
fn c1_26_the_cmake_decides_matches_and_without_it_the_menu_says_so() {
    let mut p = project("condicao de presets");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(p.root.join("CMakeLists.txt"), LISTS).unwrap();
    fs::write(p.root.join("CMakePresets.json"), PRESETS).unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);

    // Com o `CMake`: a lista dele, sem nota. O `anyOf` que ele para no
    // `matches` verdadeiro configura, e o `matches` falso sai.
    let with = (
        vec![
            ("ok".to_owned(), Value::Null),
            ("anybad".to_owned(), Value::Null),
        ],
        vec![
            ("ok".to_owned(), Value::Null),
            ("anybad".to_owned(), Value::Null),
        ],
    );
    assert_eq!(notes(&mut p), with);
    let id = p.call("cmake.configure", json!({"preset":"anybad"}))["jobId"].clone();
    let finished = p.wait("event.cmake.finished", |params| params["jobId"] == id);
    assert_eq!(finished["success"], true, "{finished}");
    assert!(p.root.join("build/anybad/compile_commands.json").is_file());
    let response = p
        .core
        .handle_request(&JsonRpcRequest::new(
            9_i64,
            "cmake.configure",
            Some(json!({"preset":"rx"})),
        ))
        .response()
        .clone();
    let error = response.error.expect("o rx sai da lista");
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert_eq!(
        error.message,
        "a pasta do preset rx não é conhecida: a condition do preset rx é falsa para o CMake"
    );

    // Sem resposta do `CMake`: a leitura nativa lista os dois que ela não
    // decide, e a lista e o menu dizem por quê.
    let native = crate::cmake_presets::native_reading();
    let without: Notes = vec![
        ("ok".to_owned(), Value::Null),
        ("rx".to_owned(), json!(NOTE)),
        ("anybad".to_owned(), json!(NOTE)),
    ];
    assert_eq!(notes(&mut p), (without.clone(), without));
    drop(native);
    assert_eq!(notes(&mut p), with);
}
