//! Explicit compiler context differences: actual readers and RPC, no tools
//! from the host and no execution of a compiler from a project's CDB.
use super::core_with_empty_search_path;
use crate::index::context::{
    CompileContext, ContextTools, KitFacts, compile_drift, complete_with_kit,
};
use kinein_protocol::{ContextOrigin, JsonRpcRequest};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn fixture(name: &str, arguments: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("kinein-drift-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("build")).unwrap();
    std::fs::write(root.join("main.cpp"), "int main() {}\n").unwrap();
    let root = crate::platform::canonicalize(&root).unwrap();
    database(&root, arguments);
    root
}
fn database(root: &Path, arguments: &[&str]) {
    let entries = json!([{"directory":root.join("build"),"file":root.join("main.cpp"),"arguments":arguments}]);
    std::fs::write(root.join("compile_commands.json"), entries.to_string()).unwrap();
}
fn context(root: &Path) -> kinein_protocol::FileContext {
    CompileContext::load(root, &ContextTools::default()).for_file(
        root,
        Path::new("main.cpp"),
        "cpp",
    )
}
fn kit(root: &Path) -> KitFacts {
    KitFacts {
        target_triple: Some("aarch64-linux-gnu".into()),
        sysroot: Some(root.join("kit-sdk").display().to_string()),
        ..KitFacts::default()
    }
}
fn call(core: &mut crate::Core, method: &str, params: Value) -> Value {
    let reply = core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)));
    assert!(
        reply.response().error.is_none(),
        "{method}: {:?}",
        reply.response()
    );
    reply.response().result.clone().unwrap()
}

#[test]
fn explicit_options_differ_but_absent_values_and_kit_values_do_not() {
    let root = fixture(
        "explicit",
        &["clang++", "--target=arm-none-eabi", "--sysroot=/different"],
    );
    let mut answer = context(&root);
    let drift = compile_drift(&answer, &kit(&root));
    assert_eq!(serde_json::to_value(&drift).unwrap()[0]["kind"], "target");
    assert_eq!(drift.len(), 2);
    assert_eq!(drift[0].configured, "aarch64-linux-gnu");
    assert_eq!(drift[0].effective.value, "arm-none-eabi");
    assert_eq!(drift[0].effective.origin, ContextOrigin::Cdb);
    assert_eq!(
        serde_json::to_value(&drift).unwrap()[1]["action"],
        "inspectKit"
    );
    answer
        .effective
        .as_mut()
        .unwrap()
        .target
        .as_mut()
        .unwrap()
        .origin = ContextOrigin::Kit;
    answer
        .effective
        .as_mut()
        .unwrap()
        .sysroot
        .as_mut()
        .unwrap()
        .origin = ContextOrigin::Kit;
    assert!(compile_drift(&answer, &kit(&root)).is_empty());
    answer.unit = None;
    assert!(compile_drift(&answer, &kit(&root)).is_empty());
    database(&root, &["clang++", "-O2"]);
    let mut unknown = context(&root);
    complete_with_kit(unknown.effective.as_mut().unwrap(), &kit(&root));
    assert!(compile_drift(&unknown, &kit(&root)).is_empty());
    let header = CompileContext::load(&root, &ContextTools::default()).for_file(
        &root,
        Path::new("header.hpp"),
        "cpp",
    );
    assert!(compile_drift(&header, &kit(&root)).is_empty());
}

#[test]
fn the_last_repeated_option_is_the_one_compared_in_both_forms() {
    let root = fixture("repeated", &["clang++"]);
    let first = root.join("first").display().to_string();
    let last = root.join("last").display().to_string();
    let attached_first = format!("--sysroot={first}");
    database(
        &root,
        &[
            "clang++",
            "--target=arm-none-eabi",
            "-target",
            "aarch64-linux-gnu",
            &attached_first,
            "--sysroot",
            &last,
        ],
    );
    let mut expected = kit(&root);
    expected.sysroot = Some(last.clone());
    assert!(compile_drift(&context(&root), &expected).is_empty());
    database(
        &root,
        &[
            "clang++",
            "-target",
            "aarch64-linux-gnu",
            "--target=arm-none-eabi",
            "--sysroot",
            &last,
            &attached_first,
        ],
    );
    let actual = context(&root);
    let drift = compile_drift(&actual, &expected);
    assert_eq!(drift.len(), 2);
    assert_eq!(drift[0].effective.value, "arm-none-eabi");
    assert_eq!(drift[1].effective.value, first);
}

#[test]
fn relative_sdk_paths_and_resolved_aliases_do_not_produce_false_drift() {
    let root = fixture("paths", &["clang++", "--sysroot=../kit-sdk"]);
    std::fs::create_dir(root.join("kit-sdk")).unwrap();
    assert!(compile_drift(&context(&root), &kit(&root)).is_empty());
    let compiler = root.join("build/compiler");
    std::fs::write(&compiler, "fixture").unwrap();
    database(&root, &["./compiler", "--sysroot=../kit-sdk"]);
    let mut expected = kit(&root);
    expected.pinned_compiler = Some(compiler.display().to_string());
    assert!(compile_drift(&context(&root), &expected).is_empty());
    let other = root.join("other-compiler");
    std::fs::write(&other, "different").unwrap();
    expected.pinned_compiler = Some(other.display().to_string());
    let drift = compile_drift(&context(&root), &expected);
    assert_eq!(drift.len(), 1);
    assert_eq!(serde_json::to_value(&drift).unwrap()[0]["kind"], "compiler");
    // A bare driver requires PATH knowledge; an unavailable driver has no identity.
    for driver in ["compiler", "./missing-compiler"] {
        database(&root, &[driver]);
        assert!(compile_drift(&context(&root), &expected).is_empty());
    }
    database(&root, &["clang++", "--sysroot=../missing-sdk"]);
    assert!(compile_drift(&context(&root), &expected).is_empty());
    expected.sysroot = Some("relative-kit-sdk".into());
    database(&root, &["clang++", "--sysroot=/other-sdk"]);
    assert!(compile_drift(&context(&root), &expected).is_empty());
}

#[test]
fn file_api_values_are_explicit_evidence_but_cache_and_unknown_values_are_not() {
    let root = fixture("origins", &["clang++", "--target=arm-none-eabi"]);
    let mut answer = context(&root);
    answer
        .effective
        .as_mut()
        .unwrap()
        .target
        .as_mut()
        .unwrap()
        .origin = ContextOrigin::CmakeFileApi;
    let drift = compile_drift(&answer, &kit(&root));
    assert_eq!(drift.len(), 1);
    assert_eq!(drift[0].effective.origin, ContextOrigin::CmakeFileApi);
    answer
        .effective
        .as_mut()
        .unwrap()
        .target
        .as_mut()
        .unwrap()
        .origin = ContextOrigin::CmakeCache;
    assert!(compile_drift(&answer, &kit(&root)).is_empty());
    answer.effective = None;
    assert!(compile_drift(&answer, &kit(&root)).is_empty());
}

#[test]
fn rpc_uses_the_last_configured_kit_and_updates_without_changing_the_build() {
    let root = fixture("rpc", &["clang++"]);
    let build_sdk = root.join("build-sdk").display().to_string();
    let kit_sdk = root.join("kit-sdk").display().to_string();
    let new_sdk = format!("--sysroot={}", root.join("new-build-sdk").display());
    database(
        &root,
        &[
            "clang++",
            "--target=arm-none-eabi",
            &format!("--sysroot={build_sdk}"),
        ],
    );
    let build = root.join(".kinein/build");
    std::fs::create_dir_all(&build).unwrap();
    std::fs::write(
        build.join("CMakeCache.txt"),
        "CMAKE_CXX_COMPILER:FILEPATH=/cached/cxx\n",
    )
    .unwrap();
    std::fs::write(build.join(".kinein-preset"), "cross\n").unwrap();
    std::fs::write(
        root.join(".kinein/toolchain.json"),
        json!({"schemaVersion":2,"kits":{
        "":{"targetTriple":"arm-none-eabi","sysroot":build_sdk},
        "cross":{"targetTriple":"aarch64-linux-gnu","sysroot":kit_sdk}}})
        .to_string(),
    )
    .unwrap();
    let original = std::fs::read(root.join("compile_commands.json")).unwrap();
    let mut core = core_with_empty_search_path("drift-rpc");
    call(&mut core, "workspace.open", json!({"path":root}));
    call(&mut core, "toolchain.get", json!({}));
    let answer = call(&mut core, "index.context", json!({"path":"main.cpp"}));
    assert_eq!(answer["effective"]["kit"]["preset"], "cross");
    assert_eq!(answer["drift"].as_array().unwrap().len(), 2);
    assert_eq!(answer["drift"][0]["configured"], "aarch64-linux-gnu");
    assert_eq!(answer["drift"][1]["effective"]["value"], build_sdk);
    let mut legacy = answer;
    legacy.as_object_mut().unwrap().remove("drift");
    assert!(
        serde_json::from_value::<kinein_protocol::FileContext>(legacy)
            .unwrap()
            .drift
            .is_empty()
    );
    call(
        &mut core,
        "toolchain.setKit",
        json!({"preset":"cross","targetTriple":"arm-none-eabi","sysroot":build_sdk}),
    );
    assert!(
        call(&mut core, "index.context", json!({"path":"main.cpp"}))
            .get("drift")
            .is_none()
    );
    assert_eq!(
        std::fs::read(root.join("compile_commands.json")).unwrap(),
        original
    );
    database(&root, &["clang++", "--target=aarch64-linux-gnu", &new_sdk]);
    core.reload_index_context();
    assert_eq!(
        call(&mut core, "index.context", json!({"path":"main.cpp"}))["drift"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
