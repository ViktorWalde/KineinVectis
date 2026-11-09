//! C1-18: presets de arquivos incluídos (consolidação C §17).
//!
//! O `cmake` real configura FORA da IDE dois presets: um do arquivo incluído,
//! outro da raiz que herda dele o `binaryDir` com `${fileDir}`. A IDE lista os
//! dois e acha cada CDB exatamente onde o `CMake` a escreveu, com GCC e Clang
//! reais. A pasta vem da linha "Build files have been written to" do próprio
//! `CMake`, não da leitura da IDE.

use super::{Project, environment::project};
use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn fixture() -> Project {
    let p = project("include de presets");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::create_dir_all(p.root.join("cmake/presets")).unwrap();
    fs::write(p.root.join("src/main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(
        p.root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(inc LANGUAGES C)\nadd_executable(inc src/main.c)\n",
    )
    .unwrap();
    let root_presets = json!({"version":6,"include":["cmake/presets/compilers.json"],
        "configurePresets":[{"name":"clang-top","inherits":"base",
            "cacheVariables":{"CMAKE_C_COMPILER":"clang"}}]});
    fs::write(p.root.join("CMakePresets.json"), root_presets.to_string()).unwrap();
    let included = json!({"version":6,"configurePresets":[
        {"name":"base","hidden":true,"generator":"Ninja",
         "binaryDir":"${fileDir}/saídas/${presetName}",
         "cacheVariables":{"CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}},
        {"name":"gcc-inc","inherits":"base","cacheVariables":{"CMAKE_C_COMPILER":"gcc"}}]});
    fs::write(
        p.root.join("cmake/presets/compilers.json"),
        included.to_string(),
    )
    .unwrap();
    p
}

/// `cmake --preset` real; a pasta é a que o `CMake` diz ter escrito.
fn cli_configure(root: &Path, preset: &str) -> PathBuf {
    let output = Command::new("cmake")
        .arg("--preset")
        .arg(preset)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let written = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("-- Build files have been written to: "))
        .map(PathBuf::from)
        .unwrap();
    assert!(written.join("compile_commands.json").is_file());
    written
}

/// A mensagem de um pedido que o core recusa.
fn refusal(p: &mut Project, method: &str) -> String {
    let response = p
        .core
        .handle_request(&JsonRpcRequest::new(1_i64, method, Some(json!({}))))
        .response()
        .clone();
    response.error.expect(method).message
}

fn compiler_of(directory: &Path) -> String {
    let cdb: Value =
        serde_json::from_slice(&fs::read(directory.join("compile_commands.json")).unwrap())
            .unwrap();
    cdb[0]["command"]
        .as_str()
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

#[test]
fn c1_18_presets_of_an_included_file_are_found_where_cmake_configures_them() {
    let mut p = fixture();
    let gcc = cli_configure(&p.root, "gcc-inc");
    let clang = cli_configure(&p.root, "clang-top");
    assert!(compiler_of(&gcc).ends_with("gcc"), "{}", compiler_of(&gcc));
    assert!(
        compiler_of(&clang).ends_with("clang"),
        "{}",
        compiler_of(&clang)
    );
    let relative = |directory: &Path| {
        directory
            .strip_prefix(&p.root)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    };
    let (gcc_dir, clang_dir) = (relative(&gcc), relative(&clang));
    // `${fileDir}` do pai incluído: a pasta do arquivo de quem está em uso.
    assert_eq!(gcc_dir, "cmake/presets/saídas/gcc-inc");
    assert_eq!(clang_dir, "saídas/clang-top");

    p.call("workspace.open", json!({"path": p.root}));
    let listed = p.call("cmake.presets.list", json!({}));
    let names: Vec<&str> = listed["presets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|preset| preset["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["clang-top", "gcc-inc"]);

    let status = p.call("cmake.status", json!({}));
    assert_eq!(
        status["cdbCandidates"],
        json!([
            {"directory": clang_dir, "origin": "preset", "preset": "clang-top"},
            {"directory": gcc_dir, "origin": "preset", "preset": "gcc-inc"}
        ]),
        "{status}"
    );
    assert_eq!(status["cdbAbsence"]["kind"], "choiceRequired", "{status}");
    let chosen = p.call("cmake.selectConfiguration", json!({"preset": "gcc-inc"}));
    assert_eq!(chosen["cdbDirectory"], gcc_dir, "{chosen}");
    assert_eq!(chosen["cdbPreset"], "gcc-inc", "{chosen}");

    // Um `include` que some recusa tudo, como no `CMake`: nem candidata de
    // preset nem lista parcial.
    fs::remove_file(p.root.join("cmake/presets/compilers.json")).unwrap();
    let refused = refusal(&mut p, "cmake.presets.list");
    assert!(
        refused.contains("arquivo de presets não encontrado: cmake/presets/compilers.json"),
        "{refused}"
    );
    let cmake = Command::new("cmake")
        .arg("--list-presets")
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(!cmake.status.success(), "{cmake:?}");
    let status = p.call("cmake.status", json!({}));
    assert_eq!(
        status["cdbAbsence"]["kind"], "chosenConfiguration",
        "{status}"
    );
    assert!(
        status["cdbAbsence"]["reason"]
            .as_str()
            .unwrap()
            .contains("arquivo de presets não encontrado"),
        "{status}"
    );
}
