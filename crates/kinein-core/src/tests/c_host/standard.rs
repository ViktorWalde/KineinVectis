//! Padrao C pelo dispatch, CMake/CDB reais e verificador existente.

use super::{Project, Value, json};

fn standard_context(p: &mut Project, standard: &str, gnu: bool) -> Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let prefix = if gnu { "gnu" } else { "c" };
    loop {
        let context = p.context("SAMPLE_BIAS=1");
        let observed = context["unit"]["standard"].as_str().unwrap_or_default();
        if observed == format!("{prefix}{standard}")
            || (standard == "23" && observed == format!("{prefix}2x"))
        {
            return context;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "standard context did not refresh: {context}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn preview(p: &mut Project, standard: &str, extensions: &str) -> Value {
    p.call(
        "configAction.preview",
        json!({"id":"cmake.setCStandard", "params":{
            "target":"c_host", "standard":standard, "extensions":extensions
        }}),
    )
}

fn apply(p: &mut Project, preview: &Value, standard: &str, extensions: &str) -> Value {
    p.call(
        "configAction.apply",
        json!({"id":"cmake.setCStandard", "params":{
        "target":"c_host", "standard":standard, "extensions":extensions
    }, "expected":[{"path":"CMakeLists.txt", "content":preview["files"][0]["before"]}]}),
    )
}

#[test]
fn c_standard_drives_gcc_clang_cdb_context_and_tests_without_changing_other_targets() {
    for compiler in ["gcc", "clang"] {
        for standard in ["11", "17", "23"] {
            let mut p = Project::new(&format!("standard-{compiler}-{standard}"));
            let presets = p.root.join("CMakePresets.json");
            let preset_content = std::fs::read_to_string(&presets)
                .unwrap()
                .replace("clang", compiler);
            std::fs::write(&presets, &preset_content).unwrap();
            let cmake = p.root.join("CMakeLists.txt");
            let original = std::fs::read_to_string(&cmake).unwrap();
            let plan = preview(&mut p, standard, "OFF");
            assert_eq!(std::fs::read_to_string(&cmake).unwrap(), original);
            apply(&mut p, &plan, standard, "OFF");
            assert!(
                std::fs::read_to_string(&cmake)
                    .unwrap()
                    .starts_with(&original)
            );
            assert_eq!(std::fs::read_to_string(&presets).unwrap(), preset_content);
            p.build(true);
            let context = standard_context(&mut p, standard, false);
            assert_eq!(context["language"], "c");
            let library = p.call("index.context", json!({"path":p.root.join("src/sum.c")}));
            assert_eq!(library["unit"]["standard"], "c11", "{library}");
            let model =
                crate::cmake::model::CmakeModel::load(&p.root.join(".kinein/build")).unwrap();
            let (_, group) = model.compile_group_for(&p.root.join("app/main.c")).unwrap();
            assert_eq!(group.language, "C");
            assert_eq!(group.standard.as_deref(), Some(standard));
            let driver = model.compiler_for("C").unwrap();
            assert_eq!(
                driver.id.as_deref(),
                Some(if compiler == "gcc" { "GNU" } else { "Clang" })
            );
            p.test();
            if standard == "23" {
                check_c23_verification(&mut p);
            }
        }
    }
}

fn check_c23_verification(p: &mut Project) {
    let source = format!(
        "{}\nstatic_assert(__STDC_VERSION__ >= 202311L, \"C23 required\");\n",
        p.before
    );
    std::fs::write(p.root.join("app/main.c"), &source).unwrap();
    p.before = source;
    p.build(true);
    let correction = p.preview(0);
    assert_eq!(
        correction["verification"]["status"], "noNewError",
        "{correction}"
    );
    let changed = preview(p, "17", "OFF");
    apply(p, &changed, "17", "OFF");
    assert!(
        p.response(
            "lsp.workspaceEdit.apply",
            json!({"transactionId":correction["transactionId"]})
        )
        .error
        .is_some()
    );
    p.build(false);
    let changed = preview(p, "23", "OFF");
    apply(p, &changed, "23", "OFF");
    p.build(true);
    standard_context(p, "23", false);
    let recovered = p.preview(0);
    assert_eq!(
        recovered["verification"]["status"], "noNewError",
        "{recovered}"
    );
    p.call(
        "lsp.workspaceEdit.apply",
        json!({"transactionId":recovered["transactionId"]}),
    );
    p.build(true);
    p.test();
}

#[test]
fn c_standard_can_switch_gnu_dialect_and_rejects_stale_apply() {
    let mut p = Project::new("standard-gnu");
    let cmake = p.root.join("CMakeLists.txt");
    let plan = preview(&mut p, "23", "ON");
    let original = std::fs::read_to_string(&cmake).unwrap();
    std::fs::write(&cmake, format!("{original}\n# external edit\n")).unwrap();
    let refused = p.response(
        "configAction.apply",
        json!({"id":"cmake.setCStandard", "params":{
        "target":"c_host", "standard":"23", "extensions":"ON"
    }, "expected":[{"path":"CMakeLists.txt", "content":plan["files"][0]["before"]}]}),
    );
    assert_eq!(
        refused.error.unwrap().code,
        kinein_protocol::JsonRpcErrorCode::FileChanged
    );
    assert!(
        !std::fs::read_to_string(&cmake)
            .unwrap()
            .contains("set_target_properties")
    );
    let plan = preview(&mut p, "23", "ON");
    apply(&mut p, &plan, "23", "ON");
    p.build(true);
    standard_context(&mut p, "23", true);
    let plan = preview(&mut p, "17", "OFF");
    apply(&mut p, &plan, "17", "OFF");
    p.build(true);
    standard_context(&mut p, "17", false);
    p.test();
}
