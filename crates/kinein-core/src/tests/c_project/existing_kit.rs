//! C1: instalação do compilador C no kit de um projeto existente (§15).

use super::{Project, environment::project};
use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    time::Duration,
};

/// The search path of the core built by [`project`]: tools placed here are
/// what `tools.detect`, the catalog and the recipe suggestions see.
fn tools_dir(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-dispatch-{name}", std::process::id()))
}

fn real(name: &str) -> PathBuf {
    crate::tools::ToolDetector::from_environment()
        .find_in_path(name)
        .unwrap()
}

/// An alternate installation path that forwards to the real driver.
fn forwarder(path: &Path, driver: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    crate::write_executable(
        path,
        format!("#!/bin/sh\nexec '{}' \"$@\"\n", real(driver).display()),
    )
}

fn manual(name: &str, presets: Option<Value>) -> Project {
    let mut p = project(name);
    fs::create_dir(&p.root).unwrap();
    fs::write(
        p.root.join("main.c"),
        "#ifdef __cplusplus\n#error C only\n#endif\nint main(void) { return 0; }\n",
    )
    .unwrap();
    fs::write(
        p.root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(kit LANGUAGES C)\nadd_executable(kit main.c)\ntarget_compile_options(kit PRIVATE -std=gnu17)\n",
    )
    .unwrap();
    fs::write(
        p.root.join(".clangd"),
        "CompileFlags:\n  Add: [-DMANUAL_CLANGD]\n",
    )
    .unwrap();
    fs::write(p.root.join("compile_flags.txt"), "-DMANUAL_FLAGS\n").unwrap();
    if let Some(presets) = presets {
        fs::write(p.root.join("CMakePresets.json"), presets.to_string()).unwrap();
    }
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    p
}

fn request(p: &mut Project, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
    p.core
        .handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn cdb_compiler(p: &Project) -> String {
    let cdb: Value = serde_json::from_slice(
        &fs::read(p.root.join(".kinein/build/compile_commands.json")).unwrap(),
    )
    .unwrap();
    let command = cdb[0]["command"].as_str().unwrap();
    assert!(command.contains("-std=gnu17"), "{command}");
    command
        .split(" -")
        .next()
        .unwrap()
        .trim()
        .trim_matches('"')
        .to_owned()
}

/// Runs `cmake.configure` and returns the outcome with the job output.
fn configure(p: &mut Project, params: Value) -> (bool, Vec<String>) {
    let job = p.call("cmake.configure", params)["jobId"].clone();
    let mut output = Vec::new();
    loop {
        let event = p.events.recv_timeout(Duration::from_secs(60)).unwrap();
        p.core.observe_notification(&event);
        let params = event.params.unwrap_or_default();
        if params["jobId"] != job {
            continue;
        }
        if event.method == "event.job.output" {
            output.push(params["line"].as_str().unwrap_or_default().to_owned());
        }
        if event.method == "event.cmake.finished" {
            return (params["success"] == true, output);
        }
    }
}

/// `index.context` once the reloaded context describes `compiler`.
fn context_with(p: &mut Project, compiler: &str) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        while let Ok(event) = p.events.try_recv() {
            p.core.observe_notification(&event);
        }
        let value = p.call("index.context", json!({"path":p.root.join("main.c")}));
        // Before the reload the kit fallback may already name it (`kit`).
        if value["effective"]["compiler"]["value"] == compiler
            && value["effective"]["compiler"]["origin"] == "cdb"
            && value["staleBecause"].is_null()
        {
            assert_eq!(value["effective"]["standard"]["value"], "gnu17", "{value}");
            return value;
        }
        assert!(std::time::Instant::now() < deadline, "{value}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn preview(p: &mut Project, params: &Value) -> kinein_protocol::JsonRpcResponse {
    request(
        p,
        "configAction.preview",
        json!({"id":"cmake.selectCCompiler","params":params}),
    )
}

fn choose(p: &mut Project, params: &Value) -> Value {
    let plan = preview(p, params);
    assert!(plan.error.is_none(), "{plan:?}");
    let plan = plan.result.unwrap();
    let file = &plan["files"][0];
    assert_eq!(file["path"], ".kinein/toolchain.json");
    let applied = p.call(
        "configAction.apply",
        json!({"id":"cmake.selectCCompiler","params":params,
               "expected":[{"path":".kinein/toolchain.json","content":file["before"]}]}),
    );
    assert_eq!(applied["files"], json!([".kinein/toolchain.json"]));
    plan
}

fn notes(plan: &Value) -> String {
    plan["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|note| note.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn c_selection(p: &mut Project, preset: Option<&str>) -> Value {
    let result = p.call("toolchain.get", json!({"preset":preset}));
    result["selections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|selection| selection["role"] == "cCompiler")
        .unwrap()
        .clone()
}

#[test]
fn c1_existing_project_selects_installations_by_path_with_preview_and_recovery() {
    let name = "kit-installation";
    let detected = forwarder(&tools_dir(name).join("clang"), "clang");
    let mut p = manual(
        name,
        Some(
            json!({"version":6,"configurePresets":[{"name":"dev","generator":"Ninja",
            "cacheVariables":{"CMAKE_C_COMPILER":"cc"}}]}),
        ),
    );
    let gcc = forwarder(&p.parent.join("instalações C/gcc 16/bin/gcc"), "gcc");
    let clang = real("clang");
    let preserved: Vec<_> = [
        "main.c",
        "CMakeLists.txt",
        "CMakePresets.json",
        ".clangd",
        "compile_flags.txt",
    ]
    .map(|file| (file, fs::read(p.root.join(file)).unwrap()))
    .into();
    let kit_file = p.root.join(".kinein/toolchain.json");

    // Detectado: a receita sugere os drivers C das pastas do detector.
    let list = p.call("configAction.list", json!({}));
    let action = list["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["id"] == "cmake.selectCCompiler")
        .unwrap()
        .clone();
    assert_eq!(action["state"], "available", "{action}");
    assert_eq!(
        action["params"][0]["suggestions"],
        json!([detected]),
        "{action}"
    );
    assert_eq!(
        action["params"][1]["suggestions"],
        json!(["dev"]),
        "{action}"
    );

    // Relativo, ausente ou C++: recusado sem gravar o kit.
    for refused in [
        json!({"compiler":"bin/gcc"}),
        json!({"compiler":p.parent.join("sem gcc")}),
        json!({"compiler":real("g++")}),
    ] {
        let response = preview(&mut p, &refused);
        assert!(response.error.is_some(), "{refused}: {response:?}");
        assert!(!kit_file.exists(), "{refused}");
    }

    // Prévia não grava; aplicação explícita fixa o caminho com espaços/Unicode.
    let plan = preview(&mut p, &json!({"compiler":gcc})).result.unwrap();
    assert!(!kit_file.exists(), "a prévia não grava");
    let text = notes(&plan);
    for expected in [
        "Kit padrão",
        &format!("— {}", detected.display()),
        "Selecionado: automático →",
        "Efetivo: o build dir da IDE ainda não foi configurado.",
        "Divergência: o preset define CMAKE_C_COMPILER=cc",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
    choose(&mut p, &json!({"compiler":gcc}));
    let selection = c_selection(&mut p, None);
    assert_eq!(selection["id"], gcc.to_str().unwrap(), "{selection}");
    assert_eq!(
        selection["resolvedPath"],
        gcc.to_str().unwrap(),
        "{selection}"
    );
    assert!(configure(&mut p, json!({})).0);
    assert_eq!(cdb_compiler(&p), gcc.to_str().unwrap());
    context_with(&mut p, gcc.to_str().unwrap());

    // Trocar para Clang: o CMake recria o cache e a IDE repete o configure, sem
    // deixar a CDB do GCC como evidência atual (C1-14).
    let plan = choose(&mut p, &json!({"compiler":clang}));
    let text = notes(&plan);
    assert!(
        text.contains(&format!("O próximo configure troca {}", gcc.display())),
        "{text}"
    );
    let (success, output) = configure(&mut p, json!({}));
    assert!(success, "{output:?}");
    assert!(
        output
            .iter()
            .any(|line| line == crate::cmake::attempt::DEFINITIONS_LOST),
        "{output:?}"
    );
    assert_eq!(cdb_compiler(&p), clang.to_str().unwrap());
    context_with(&mut p, clang.to_str().unwrap());

    reopen_rebuild_and_release(&mut p, &gcc, &clang);
    for (file, bytes) in preserved {
        assert_eq!(fs::read(p.root.join(file)).unwrap(), bytes, "{file}");
    }
}

/// Second half of the journey: reopen, switch through `build.run` and go back
/// to automatic without touching the configured cache.
fn reopen_rebuild_and_release(p: &mut Project, gcc: &Path, clang: &Path) {
    // Reabrir conserva a escolha; o build usa a instalação escolhida.
    p.call("workspace.close", json!({}));
    p.call("workspace.open", json!({"path":p.root}));
    p.wait("event.index.finished", |_| true);
    assert_eq!(c_selection(p, None)["id"], clang.to_str().unwrap());
    p.build(true);
    assert_eq!(cdb_compiler(p), clang.to_str().unwrap());

    // O configure automático do build também repete quando o cache é recriado.
    choose(p, &json!({"compiler":gcc}));
    fs::remove_file(p.root.join(".kinein/build/.kinein-rigor")).unwrap();
    p.build(true);
    assert_eq!(cdb_compiler(p), gcc.to_str().unwrap());
    context_with(p, gcc.to_str().unwrap());

    // Voltar ao automático não troca o compilador de um cache existente.
    let plan = choose(p, &json!({"compiler":""}));
    assert!(notes(&plan).contains("conserva"), "{plan}");
    assert!(c_selection(p, None)["id"].is_null());
}

#[test]
fn c1_unavailable_kit_compiler_refuses_configure_until_restored() {
    for by_path in [false, true] {
        let name = if by_path {
            "kit-missing-path"
        } else {
            "kit-missing-id"
        };
        let mut p = manual(name, None);
        let driver = forwarder(&tools_dir(name).join("clang"), "clang");
        p.call("tools.detect", json!({}));
        if by_path {
            choose(&mut p, &json!({"compiler":driver}));
        } else {
            p.call("toolchain.set", json!({"role":"cCompiler","id":"clang"}));
        }
        assert!(configure(&mut p, json!({})).0);
        assert_eq!(cdb_compiler(&p), driver.to_str().unwrap());

        fs::remove_file(&driver).unwrap();
        p.call("tools.detect", json!({}));
        let selection = c_selection(&mut p, None);
        assert!(selection["id"].is_string(), "{selection}");
        assert!(selection["resolvedPath"].is_null(), "{selection}");
        assert!(selection["effectiveId"].is_null(), "{selection}");
        fs::remove_dir_all(p.root.join(".kinein/build")).unwrap();
        let refused = request(&mut p, "cmake.configure", json!({}));
        let message = refused.error.unwrap().message;
        assert!(message.contains("não está disponível"), "{message}");
        let job = p.call("build.run", json!({}))["jobId"].clone();
        let finished = p.wait("event.build.finished", |event| event["jobId"] == job);
        assert_eq!(finished["success"], false, "{finished}");
        assert!(
            finished["error"]
                .as_str()
                .unwrap()
                .contains("não está disponível"),
            "{finished}"
        );
        assert!(
            !p.root.join(".kinein/build/CMakeCache.txt").exists(),
            "no other compiler was configured"
        );

        forwarder(&driver, "clang");
        p.call("tools.detect", json!({}));
        assert!(configure(&mut p, json!({})).0);
        assert_eq!(cdb_compiler(&p), driver.to_str().unwrap());
    }
}

#[test]
fn c1_implicit_configure_keeps_the_kit_of_the_recorded_preset() {
    let name = "kit-recorded";
    let tools = tools_dir(name);
    let mut p = manual(
        name,
        Some(json!({"version":6,"configurePresets":[{"name":"dev","generator":"Ninja"}]})),
    );
    let gcc = forwarder(&tools.join("gcc"), "gcc");
    let clang = forwarder(&tools.join("clang"), "clang");
    p.call("tools.detect", json!({}));
    p.call("toolchain.set", json!({"role":"cCompiler","id":"gcc"}));
    p.call(
        "toolchain.set",
        json!({"role":"cCompiler","id":"clang","preset":"dev"}),
    );
    assert!(configure(&mut p, json!({"preset":"dev"})).0);
    assert_eq!(cdb_compiler(&p), clang.to_str().unwrap());
    // Sem preset pedido: o preset anotado conserva o kit dele, não o padrão.
    let (success, output) = configure(&mut p, json!({}));
    assert!(success, "{output:?}");
    assert!(
        !output
            .iter()
            .any(|line| line.contains("have changed variables")),
        "{output:?}"
    );
    assert_eq!(cdb_compiler(&p), clang.to_str().unwrap());
    // O configure automático do build (perfil de rigor a reaplicar) também.
    fs::remove_file(p.root.join(".kinein/build/.kinein-rigor")).unwrap();
    p.build(true);
    assert_eq!(cdb_compiler(&p), clang.to_str().unwrap());
    let context = context_with(&mut p, clang.to_str().unwrap());
    assert_eq!(context["effective"]["kit"]["preset"], "dev", "{context}");
    // A receita sem kit informado edita o kit que o último configure usou;
    // voltar esse kit ao automático não entrega o configure ao kit padrão.
    let plan = choose(&mut p, &json!({"compiler":""}));
    assert!(notes(&plan).contains("Kit dev"), "{plan}");
    assert!(c_selection(&mut p, Some("dev"))["id"].is_null());
    assert_eq!(c_selection(&mut p, None)["id"], "gcc");
    let (success, output) = configure(&mut p, json!({}));
    assert!(success, "{output:?}");
    assert_eq!(cdb_compiler(&p), clang.to_str().unwrap(), "{output:?}");
    assert!(gcc.is_file());
}

#[test]
fn c1_installation_check_runs_off_the_loop_and_refuses_a_changed_workspace() {
    let mut p = manual("kit-deferred", None);
    let gcc = real("gcc");
    let (sender, pending) = mpsc::channel::<crate::Continuation>();
    p.core.enable_continuations(sender);
    let deferred = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "configAction.preview",
        Some(json!({"id":"cmake.selectCCompiler","params":{"compiler":gcc}})),
    ));
    assert!(deferred.is_deferred(), "{deferred:?}");
    let finish = pending.recv_timeout(Duration::from_secs(8)).unwrap();
    let other = p.parent.join("other");
    fs::create_dir(&other).unwrap();
    p.call("workspace.open", json!({"path":other}));
    let refused = finish(&mut p.core);
    assert!(refused.error.unwrap().message.contains("workspace mudou"));
    assert!(!p.root.join(".kinein/toolchain.json").exists());
}
