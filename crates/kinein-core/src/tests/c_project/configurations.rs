//! C-04 (C1-19 a C1-21, consolidação C §17): cada configuração configura,
//! compila, roda, testa e dá contexto na pasta dela, pelo `cmake` real.
//!
//! Até o 0.208.0 todo configure rodava em `.kinein/build` com `-B`, que
//! sobrepõe o `binaryDir` do preset (`CMake` 4.4.5): a IDE e o terminal
//! configuravam pastas diferentes para o mesmo preset, e trocar de preset
//! reconfigurava a mesma pasta. Aqui o executável imprime o tipo de build e o
//! compilador com que foi gerado, e o teste do `ctest` tem o tipo de build no
//! nome: cada agulha diz de que pasta veio.

use super::{Project, environment::project};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, ProjectKind};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const MAIN: &str = "#include <stdio.h>\nint main(void) { puts(\"kv-run-\" KV_KIND); return 0; }\n";

const LISTS: &str = r#"cmake_minimum_required(VERSION 3.24)
project(confs LANGUAGES C)
if(KV_FAIL)
  message(FATAL_ERROR "kv-configure-failure")
endif()
add_executable(confs src/main.c)
target_compile_definitions(confs PRIVATE "KV_KIND=\"${CMAKE_BUILD_TYPE}-${CMAKE_C_COMPILER_ID}\"")
enable_testing()
add_test(NAME kv_smoke_${CMAKE_BUILD_TYPE} COMMAND confs)
"#;

fn presets(release_fails: bool) -> String {
    json!({"version":6,"configurePresets":[
        {"name":"base","hidden":true,"generator":"Ninja",
         "cacheVariables":{"CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}},
        {"name":"gcc-debug","inherits":"base","binaryDir":"${sourceDir}/build/${presetName}",
         "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_BUILD_TYPE":"Debug"}},
        {"name":"clang-release","inherits":"base",
         "binaryDir":"${sourceParentDir}/${sourceDirName} saídas/$env{KV_KIND}",
         "environment":{"KV_KIND":"clang release"},
         "cacheVariables":{"CMAKE_C_COMPILER":"clang","CMAKE_BUILD_TYPE":"Release",
                           "KV_FAIL":if release_fails {"ON"} else {"OFF"}}},
        {"name":"plain","inherits":"base",
         "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_BUILD_TYPE":"MinSizeRel"}},
        {"name":"a/b","inherits":"base"}
    ]})
    .to_string()
}

fn fixture(name: &str, presets: &str) -> Project {
    let mut p = project(name);
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), MAIN).unwrap();
    fs::write(p.root.join("CMakeLists.txt"), LISTS).unwrap();
    fs::write(p.root.join("CMakePresets.json"), presets).unwrap();
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
    p
}

fn release_dir(p: &Project) -> PathBuf {
    p.parent.join("starter saídas/clang release")
}

/// O ciclo de um job: o evento de início, as linhas e o sucesso.
pub(super) fn job(
    p: &mut Project,
    job: &Value,
    started: &str,
    finished: &str,
) -> (Value, Vec<String>, bool) {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut start = Value::Null;
    let mut lines = Vec::new();
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .expect("job deadline");
        let event = p.events.recv_timeout(left).unwrap();
        p.core.observe_notification(&event);
        let params = event.params.unwrap_or_default();
        if params["jobId"] != *job {
            continue;
        }
        match event.method.as_str() {
            "event.job.output" => lines.push(params["line"].as_str().unwrap().to_owned()),
            method if method == started => start = params,
            method if method == finished => return (start, lines, params["success"] == true),
            _ => {}
        }
    }
}

pub(super) fn configure(p: &mut Project, preset: &str) -> (Value, Vec<String>, bool) {
    let id = p.call("cmake.configure", json!({"preset":preset}))["jobId"].clone();
    job(p, &id, "event.cmake.started", "event.cmake.finished")
}

/// `build.run`: as linhas do job, com as do configure implícito.
pub(super) fn build(p: &mut Project) -> Vec<String> {
    let id = p.call("build.run", json!({}))["jobId"].clone();
    let (_, lines, success) = job(p, &id, "event.build.started", "event.build.finished");
    assert!(success, "{lines:?}");
    lines
}

/// `run.start` sem comando: o executável da configuração ativa.
pub(super) fn run(p: &mut Project, expected: &Path) -> String {
    let result = p.call("run.start", json!({}));
    assert_eq!(
        result["command"],
        crate::platform::program_invocation(expected.to_str().unwrap()),
        "{result}"
    );
    let id = result["terminalId"].as_str().unwrap().to_owned();
    let (lines, code) =
        super::super::terminal_run_until_closed(&p.events, &id, Duration::from_secs(20));
    assert_eq!(code, Some(0), "{lines:?}");
    let printed: Vec<_> = lines.iter().filter(|l| l.starts_with("kv-run-")).collect();
    assert_eq!(printed.len(), 1, "{lines:?}");
    printed[0].clone()
}

/// Os casos que o `ctest` rodou, com o estado.
pub(super) fn ctest(p: &mut Project) -> Vec<String> {
    let id = p.call("test.run", json!({}))["jobId"].clone();
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut cases = Vec::new();
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .expect("ctest deadline");
        let event = p.events.recv_timeout(left).unwrap();
        p.core.observe_notification(&event);
        let params = event.params.unwrap_or_default();
        if params["jobId"] != id {
            continue;
        }
        match event.method.as_str() {
            "event.test.case" => cases.push(format!("{} {}", params["name"], params["status"])),
            "event.test.finished" => {
                assert_eq!(params["success"], true, "{params} {cases:?}");
                return cases;
            }
            _ => {}
        }
    }
}

/// O contexto de `main.c` depois que o core recarregou a CDB de `directory`.
pub(super) fn context_from(p: &mut Project, directory: &Path) -> Value {
    let cdb = directory.join("compile_commands.json");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        while let Ok(event) = p.events.try_recv() {
            p.core.observe_notification(&event);
        }
        let context = p.call("index.context", json!({"path":p.root.join("src/main.c")}));
        if context["cdbLocation"]["path"] == cdb.to_str().unwrap()
            && !context["effective"].is_null()
        {
            return context;
        }
        assert!(Instant::now() < deadline, "{}: {context}", cdb.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(super) fn kind_define(context: &Value) -> String {
    context["unit"]["defines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|define| define.starts_with("KV_KIND="))
        .unwrap()
        .to_owned()
}

fn select(p: &mut Project, preset: Option<&str>) {
    p.call("cmake.selectConfiguration", json!({"preset":preset}));
}

pub(super) fn status(p: &mut Project) -> Value {
    p.call("cmake.status", json!({}))
}

fn cache(directory: &Path) -> Vec<u8> {
    fs::read(directory.join("CMakeCache.txt")).unwrap()
}

#[test]
fn c04_each_configuration_builds_runs_tests_and_gives_context_in_its_own_folder() {
    let mut p = fixture("configurations", &presets(false));
    let debug = p.root.join("build/gcc-debug");
    let release = release_dir(&p);
    let plain = p.root.join(".kinein/presets/plain");

    // Quatro presets e nenhuma escolha: não há configuração ativa.
    let before = status(&mut p);
    assert!(before.get("buildDir").is_none(), "{before}");
    assert_eq!(
        before["presetChoices"],
        json!(["gcc-debug", "clang-release", "plain", "a/b"])
    );
    // Um nome que não serve de pasta é recusado antes de qualquer processo.
    let refused = p.core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "cmake.configure",
        Some(json!({"preset":"a/b"})),
    ));
    let error = refused.response().error.clone().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert_eq!(
        error.message,
        "o preset \"a/b\" não declara binaryDir, e o nome não serve de pasta em .kinein/presets: \
         declare binaryDir no preset ou mude o nome"
    );
    assert_eq!(crate::cmake::selected_configuration(&p.root), Ok(None));
    assert!(!p.root.join(".kinein/presets").exists());

    // O configure da IDE roda no binaryDir, e diz isso na saída.
    let (started, lines, success) = configure(&mut p, "gcc-debug");
    assert!(success, "{lines:?}");
    assert_eq!(started["buildDir"], "build/gcc-debug", "{started}");
    assert_eq!(
        lines[..2],
        [
            "configurando em build/gcc-debug".to_owned(),
            "build/gcc-debug é o binaryDir do preset gcc-debug: o cache recebe as definições da \
             IDE (-D da linha de comando), e `cmake --preset gcc-debug` no terminal as herda"
                .to_owned()
        ]
    );
    let written = format!("-- Build files have been written to: {}", debug.display());
    assert!(lines.contains(&written), "{lines:?}");
    build(&mut p);
    assert_eq!(run(&mut p, &debug.join("confs")), "kv-run-Debug-GNU");
    assert_eq!(ctest(&mut p), ["\"kv_smoke_Debug\" \"passed\""]);
    let context = context_from(&mut p, &debug);
    assert_eq!(kind_define(&context), "KV_KIND=\"Debug-GNU\"");
    assert_eq!(
        crate::dap::resolve_program(ProjectKind::Cmake, &p.root)
            .unwrap()
            .program_path(),
        Some(debug.join("confs").as_path())
    );
    let debug_cache = cache(&debug);

    // Outra configuração, fora da raiz, com espaço e acento no caminho.
    let (started, lines, success) = configure(&mut p, "clang-release");
    assert!(success, "{lines:?}");
    assert_eq!(started["buildDir"], release.to_str().unwrap(), "{started}");
    assert_eq!(lines[0], format!("configurando em {}", release.display()));
    build(&mut p);
    assert_eq!(run(&mut p, &release.join("confs")), "kv-run-Release-Clang");
    assert_eq!(ctest(&mut p), ["\"kv_smoke_Release\" \"passed\""]);
    assert_eq!(
        kind_define(&context_from(&mut p, &release)),
        "KV_KIND=\"Release-Clang\""
    );
    assert_eq!(cache(&debug), debug_cache, "a outra pasta fica como estava");

    // Voltar não reconfigura: o build segue direto na pasta que já existe.
    select(&mut p, Some("gcc-debug"));
    let lines = build(&mut p);
    assert!(
        !lines.iter().any(|line| line.starts_with("configurando em")),
        "{lines:?}"
    );
    assert_eq!(run(&mut p, &debug.join("confs")), "kv-run-Debug-GNU");
    assert_eq!(
        kind_define(&context_from(&mut p, &debug)),
        "KV_KIND=\"Debug-GNU\""
    );
    assert_eq!(status(&mut p)["preset"], "gcc-debug");

    // Sem binaryDir, a pasta é da IDE, e a nota do binaryDir não aparece.
    let (started, lines, success) = configure(&mut p, "plain");
    assert!(success, "{lines:?}");
    assert_eq!(started["buildDir"], ".kinein/presets/plain");
    assert_eq!(lines[0], "configurando em .kinein/presets/plain");
    assert!(!lines[1].contains("binaryDir"), "{lines:?}");
    build(&mut p);
    assert_eq!(run(&mut p, &plain.join("confs")), "kv-run-MinSizeRel-GNU");

    // A pasta de uma configuração escolhida que sumiu é refeita pelo build,
    // no mesmo lugar.
    select(&mut p, Some("clang-release"));
    fs::remove_dir_all(&release).unwrap();
    let lines = build(&mut p);
    assert_eq!(lines[0], format!("configurando em {}", release.display()));
    assert_eq!(run(&mut p, &release.join("confs")), "kv-run-Release-Clang");

    assert_eq!(cache(&debug), debug_cache);
    assert!(!p.root.join(".kinein/build").exists());
}

#[test]
fn c04_a_failed_configure_and_an_old_ide_folder_stay_in_their_folders() {
    let mut p = fixture("configurations-failure", &presets(true));
    let debug = p.root.join("build/gcc-debug");
    let release = release_dir(&p);
    assert!(configure(&mut p, "gcc-debug").2);
    build(&mut p);
    let debug_cache = cache(&debug);

    // A tentativa que falhou fica na pasta dela; a outra configuração
    // continua atual e compila sem reconfigurar.
    let (_, lines, success) = configure(&mut p, "clang-release");
    assert!(!success, "{lines:?}");
    assert!(release.join(".kinein-configure-pending").is_file());
    let failed = status(&mut p);
    assert_eq!(failed["configurationPreset"], "clang-release", "{failed}");
    select(&mut p, Some("gcc-debug"));
    let current = status(&mut p);
    assert_ne!(current["cdbStale"], true, "{current}");
    assert_eq!(current["cdbDirectory"], "build/gcc-debug", "{current}");
    context_from(&mut p, &debug);
    let lines = build(&mut p);
    assert!(
        !lines.iter().any(|line| line.starts_with("configurando em")),
        "{lines:?}"
    );
    assert_eq!(cache(&debug), debug_cache);
    assert!(release.join(".kinein-configure-pending").is_file());

    // Um projeto que a IDE configurou antes do C-04: `.kinein/build` anotado,
    // sem registro, continua o ativo no automático para rodar e depurar.
    select(&mut p, None);
    fs::remove_dir_all(&debug).unwrap();
    fs::remove_dir_all(&release).unwrap();
    let ide = p.root.join(".kinein/build");
    for args in [
        vec!["--preset", "gcc-debug", "-B", ide.to_str().unwrap()],
        vec!["--build", ide.to_str().unwrap()],
    ] {
        let output = Command::new("cmake")
            .args(args)
            .current_dir(&p.root)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }
    fs::write(ide.join(".kinein-preset"), "gcc-debug\n").unwrap();
    let legacy = status(&mut p);
    assert_eq!(legacy["configurationSource"], "buildDir", "{legacy}");
    assert_eq!(legacy["configurationPreset"], "gcc-debug");
    assert_eq!(legacy["buildDir"], ide.to_str().unwrap());
    assert_eq!(run(&mut p, &ide.join("confs")), "kv-run-Debug-GNU");
    assert_eq!(
        crate::dap::resolve_program(ProjectKind::Cmake, &p.root)
            .unwrap()
            .program_path(),
        Some(ide.join("confs").as_path())
    );
    // O próximo configure (aqui o do build, sem o registro do rigor) vai
    // para a pasta do preset e a registra.
    let lines = build(&mut p);
    assert_eq!(lines[0], "configurando em build/gcc-debug", "{lines:?}");
    assert_eq!(status(&mut p)["configurationSource"], "configuration");
    assert_eq!(run(&mut p, &debug.join("confs")), "kv-run-Debug-GNU");
    assert!(ide.join("CMakeCache.txt").is_file(), "a pasta antiga fica");
}

/// Reparar remove só uma pasta cujo cache é deste projeto (decisão do autor
/// de 2026-11-09).
#[test]
fn c04_repair_removes_a_preset_folder_only_with_this_project_cache() {
    let presets = json!({"version":6,"configurePresets":[
        {"name":"gcc-debug","generator":"Ninja","binaryDir":"${sourceDir}/build/${presetName}",
         "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_BUILD_TYPE":"Debug"}},
        {"name":"in-source","generator":"Ninja","binaryDir":"${sourceDir}",
         "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_BUILD_TYPE":"Debug"}}
    ]});
    let mut p = fixture("configurations-repair", &presets.to_string());
    let debug = p.root.join("build/gcc-debug");
    let preview = |p: &mut Project| {
        p.core
            .handle_request(&JsonRpcRequest::new(
                3_i64,
                "configAction.preview",
                Some(json!({"id":"cmake.repairBuildDir","params":{}})),
            ))
            .response()
            .clone()
    };

    // O cache de outro projeto na pasta do preset: recusado.
    let other = p.parent.join("outro projeto");
    fs::create_dir_all(&other).unwrap();
    fs::write(
        other.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(other NONE)\n",
    )
    .unwrap();
    let output = Command::new("cmake")
        .arg("-S")
        .arg(&other)
        .arg("-B")
        .arg(&debug)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    select(&mut p, Some("gcc-debug"));
    assert_eq!(
        preview(&mut p).error.unwrap().message,
        format!(
            "{} é a pasta de build do preset e o cache dela é de outro projeto ({}): a IDE não a \
             remove",
            debug.display(),
            other.display()
        )
    );
    assert!(debug.join("CMakeCache.txt").is_file());

    // Com o cache deste projeto, a prévia mostra o caminho absoluto.
    fs::remove_dir_all(&debug).unwrap();
    assert!(configure(&mut p, "gcc-debug").2);
    let plan = preview(&mut p).result.unwrap();
    assert!(
        plan.to_string()
            .contains(&format!("Remove {} inteiro", debug.display())),
        "{plan}"
    );
    p.call(
        "configAction.apply",
        json!({"id":"cmake.repairBuildDir","params":{},"expected":[]}),
    );
    assert!(!debug.exists());
    assert!(p.root.join("build").is_dir() && p.root.join("src/main.c").is_file());

    // O build in-source nunca é removido: a pasta é a raiz.
    assert!(configure(&mut p, "in-source").2);
    assert!(p.root.join("CMakeCache.txt").is_file());
    assert_eq!(
        preview(&mut p).error.unwrap().message,
        format!(
            "{} é a pasta de build do preset e contém a raiz do projeto (build in-source): a IDE \
             não a remove",
            p.root.display()
        )
    );
    assert_eq!(fs::read_to_string(p.root.join("src/main.c")).unwrap(), MAIN);
}
