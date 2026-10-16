//! O contexto EFETIVO pelo despacho real (0.4.0a, `roadmaps/52` §14.1,
//! protocolo `0.173.0`): cada valor com a origem, as flags da CDB vencendo o
//! kit, o kit completando o que a unidade nao diz, o cache do `CMake` sem
//! CDB, e nada suposto quando ninguem diz.

use std::path::{Path, PathBuf};

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::core_with_empty_search_path;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-index-effective-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

fn put_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn call(core: &mut crate::Core, method: &str, params: &Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(9_i64, method, Some(params.clone())));
    let response = outcome.response();
    assert!(response.error.is_none(), "{method}: {:?}", response.error);
    response.result.clone().unwrap()
}

fn open(name: &str, root: &Path) -> crate::Core {
    let mut core = core_with_empty_search_path(name);
    call(
        &mut core,
        "workspace.open",
        &json!({ "path": root.to_str().unwrap() }),
    );
    core
}

fn effective(core: &mut crate::Core, path: &str) -> Value {
    call(core, "index.context", &json!({ "path": path }))["effective"].clone()
}

/// A CDB traz compilador, padrao, alvo e sysroot: eles vencem o kit, que
/// fixou outro alvo e outro sysroot; o kit que vale e' o padrao.
#[test]
fn the_cdb_flags_win_over_the_kit_and_each_value_says_where_it_came_from() {
    let root = temp_dir("cdb");
    let source = root.join("src/main.cpp");
    put_file(&source, "int main() { return 0; }\n");
    let cdb = json!([{
        "directory": root.join(".kinein/build").display().to_string(),
        "arguments": ["/usr/bin/clang++", "-std=c++20", "--target=aarch64-linux-gnu",
                      "--sysroot", "/srv/raiz", "-c", source.display().to_string()],
        "file": source.display().to_string()
    }]);
    put_file(
        &root.join(".kinein/build/compile_commands.json"),
        &serde_json::to_string(&cdb).unwrap(),
    );
    let mut core = open("efetivo-cdb", &root);
    call(
        &mut core,
        "toolchain.setKit",
        &json!({ "sysroot": "/opt/sysroot-do-kit", "targetTriple": "thumbv7em-none-eabihf" }),
    );

    let e = effective(&mut core, "src/main.cpp");
    assert_eq!(
        e["compiler"],
        json!({ "value": "/usr/bin/clang++", "origin": "cdb" }),
        "{e}"
    );
    assert_eq!(e["standard"], json!({ "value": "c++20", "origin": "cdb" }));
    assert_eq!(
        e["target"],
        json!({ "value": "aarch64-linux-gnu", "origin": "cdb" })
    );
    assert_eq!(
        e["sysroot"],
        json!({ "value": "/srv/raiz", "origin": "cdb" })
    );
    assert_eq!(e["flagsOrigin"], "cdb");
    assert_eq!(e["kit"], json!({ "preset": "" }));
}

/// Sem CDB: o compilador e' o que o cache do `CMake` registra, e o alvo e o
/// sysroot vem do kit; o Rust so' tem o alvo do kit; o Python, nada.
#[test]
fn without_a_cdb_the_cache_and_the_kit_complete_the_context() {
    let root = temp_dir("cache");
    put_file(&root.join("src/main.cpp"), "int main() { return 0; }\n");
    put_file(&root.join("src/lib.rs"), "pub fn f() {}\n");
    put_file(&root.join("tools/gera.py"), "x = 1\n");
    put_file(
        &root.join(".kinein/build/CMakeCache.txt"),
        "CMAKE_CXX_COMPILER:FILEPATH=/opt/arm/bin/arm-none-eabi-g++\n",
    );
    let mut core = open("efetivo-cache", &root);
    call(
        &mut core,
        "toolchain.setKit",
        &json!({ "sysroot": "/opt/sysroot-do-kit", "targetTriple": "thumbv7em-none-eabihf" }),
    );

    let e = effective(&mut core, "src/main.cpp");
    assert_eq!(
        e["compiler"],
        json!({ "value": "/opt/arm/bin/arm-none-eabi-g++", "origin": "cmakeCache" }),
        "{e}"
    );
    assert_eq!(
        e["target"],
        json!({ "value": "thumbv7em-none-eabihf", "origin": "kit" })
    );
    assert_eq!(
        e["sysroot"],
        json!({ "value": "/opt/sysroot-do-kit", "origin": "kit" })
    );
    assert!(
        e.get("standard").is_none() && e.get("flagsOrigin").is_none(),
        "{e}"
    );

    let rust = effective(&mut core, "src/lib.rs");
    assert_eq!(
        rust["target"],
        json!({ "value": "thumbv7em-none-eabihf", "origin": "kit" })
    );
    assert!(rust.get("compiler").is_none(), "{rust}");
    assert!(effective(&mut core, "tools/gera.py").is_null());
}

/// Sem CDB, sem cache e sem kit: nada suposto, so' o kit padrao dito.
#[test]
fn nobody_says_anything_and_nothing_is_assumed() {
    let root = temp_dir("nada");
    put_file(&root.join("src/main.c"), "int main(void) { return 0; }\n");
    let mut core = open("efetivo-nada", &root);
    assert_eq!(
        effective(&mut core, "src/main.c"),
        json!({ "kit": { "preset": "" } })
    );
}
