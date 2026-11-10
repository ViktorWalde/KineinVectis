//! C1-25: os presets pelo `CMake` do projeto (consolidação C §18), de ponta a
//! ponta pelo core, com `CMake` 4.4.5, GCC e Ninja reais.
//!
//! Até o C1-25 a IDE listava, oferecia no menu e configurava um preset de
//! gerador que este `CMake` não tem, e aceitava um arquivo que o esquema dele
//! recusa; o configure falhava depois, no `CMake`. Agora a lista, o menu e o
//! configure seguem o `cmake --list-presets`, e a recusa dele vem antes de
//! qualquer job.

use super::{Project, environment::project};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};

const LISTS: &str = "cmake_minimum_required(VERSION 3.24)\nproject(esquema LANGUAGES C)\nadd_executable(esquema src/main.c)\n";

fn presets(extra: &str) -> String {
    format!(
        r#"{{"version":6,"configurePresets":[
          {{"name":"ok","generator":"Ninja","binaryDir":"${{sourceDir}}/build/ok","cacheVariables":{{"CMAKE_C_COMPILER":"gcc"}}{extra}}},
          {{"name":"vs","generator":"Visual Studio 17 2022","binaryDir":"${{sourceDir}}/build/vs"}},
          {{"name":"base","hidden":true,"generator":"Nope"}},
          {{"name":"heir","inherits":"base","binaryDir":"${{sourceDir}}/build/heir"}},
          {{"name":"rx","generator":"Ninja","binaryDir":"${{sourceDir}}/build/rx",
            "condition":{{"type":"matches","string":"a1","regex":"^a\\d$"}}}}]}}"#
    )
}

/// A resposta de um pedido que o core recusa.
fn refusal(p: &mut Project, method: &str, params: Value) -> (JsonRpcErrorCode, String) {
    let response = p
        .core
        .handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)))
        .response()
        .clone();
    let error = response.error.expect("o core recusa");
    (error.code, error.message)
}

/// Nenhum configure começa numa janela de observação.
fn no_configure_starts(p: &Project) {
    let deadline = Instant::now() + Duration::from_millis(300);
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        if let Ok(event) = p.events.recv_timeout(left) {
            assert_ne!(event.method, "event.cmake.started", "a recusa não roda job");
        }
    }
}

fn listed(p: &mut Project) -> Vec<String> {
    p.call("cmake.presets.list", json!({}))["presets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|preset| preset["name"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn c1_25_the_cmake_of_the_project_decides_the_list_the_menu_and_the_configure() {
    let mut p = project("esquema de presets");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(p.root.join("CMakeLists.txt"), LISTS).unwrap();
    fs::write(p.root.join("CMakePresets.json"), presets("")).unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);

    // Os geradores que este `CMake` não tem saem da lista e do menu, e o
    // `matches` que ele acha falso também (`\d` não é classe para ele).
    assert_eq!(listed(&mut p), ["ok"]);
    let status = p.call("cmake.status", json!({}));
    assert_eq!(
        status["configurations"],
        json!([{"preset":"ok","buildDir":"build/ok","configured":false}]),
        "{status}"
    );
    for (preset, reason) in [
        (
            "vs",
            "o CMake não tem o gerador \"Visual Studio 17 2022\" do preset vs",
        ),
        ("heir", "o CMake não tem o gerador \"Nope\" do preset heir"),
        ("rx", "a condition do preset rx é falsa para o CMake"),
        // C1-27: o `CMake` o recusaria com "Cannot use hidden configure
        // preset", depois de o job começar.
        (
            "base",
            "o preset base é oculto: o CMake não o usa pelo nome",
        ),
    ] {
        let (code, message) = refusal(&mut p, "cmake.configure", json!({"preset":preset}));
        assert_eq!(code, JsonRpcErrorCode::InvalidParams);
        assert_eq!(
            message,
            format!("a pasta do preset {preset} não é conhecida: {reason}")
        );
        no_configure_starts(&p);
        assert!(!p.root.join("build").join(preset).exists(), "{preset}");
    }

    // O preset que ele lista configura, sem preset pedido: é o único.
    let id = p.call("cmake.configure", json!({}))["jobId"].clone();
    let finished = p.wait("event.cmake.finished", |params| params["jobId"] == id);
    assert_eq!(finished["success"], true, "{finished}");
    assert!(p.root.join("build/ok/compile_commands.json").is_file());

    // Um campo que o esquema não tem: o `CMake` recusa o arquivo, e a IDE
    // também, com o motivo dele e sem job.
    fs::write(p.root.join("CMakePresets.json"), presets(r#","nope":1"#)).unwrap();
    let reason =
        "o CMake recusa os presets: CMakePresets.json:2: Invalid extra field \"nope\" in Preset";
    let (_, message) = refusal(&mut p, "cmake.presets.list", json!({}));
    assert_eq!(message, reason);
    let (code, message) = refusal(&mut p, "cmake.configure", json!({}));
    assert_eq!(code, JsonRpcErrorCode::InvalidParams);
    assert_eq!(message, reason);
    no_configure_starts(&p);
    let status = p.call("cmake.status", json!({}));
    assert!(status.get("configurations").is_none(), "{status}");

    // Corrigido o arquivo, a resposta muda sem reabrir nada.
    fs::write(p.root.join("CMakePresets.json"), presets("")).unwrap();
    assert_eq!(listed(&mut p), ["ok"]);
}
