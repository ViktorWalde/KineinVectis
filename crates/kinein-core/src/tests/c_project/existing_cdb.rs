//! C-05: a CDB de configurações existentes, a escolha e o clangd (consolidação C §16).
//!
//! Os presets são configurados pelo `cmake` real FORA da IDE, como num projeto
//! aberto depois do CLI ou de outra IDE. O clangd é o real, atrás de um
//! encaminhador que registra o argv; os diagnósticos dependem da CDB que ele
//! leu: `KV_CONFIG_DEBUG` e `KV_CONFIG_RELEASE` produzem cada um um `#warning`
//! próprio, e sem `KV_CONFIG_RELEASE` há um `#error`. Cada CDB tem assinatura
//! positiva; uma publicação vazia não prova nada (o core limpa os
//! diagnósticos do clangd que mata).

use super::{Project, environment::project};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

const MAIN: &str = "#ifdef KV_CONFIG_DEBUG\n#warning \"kv-debug-needle\"\n#endif\n\
#ifdef KV_CONFIG_RELEASE\n#warning \"kv-release-needle\"\n#else\n#error \"kv-release-missing\"\n#endif\n\
int main(void) { return 0; }\n";

fn fixture(name: &str) -> Project {
    let p = project(name);
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), MAIN).unwrap();
    fs::write(
        p.root.join("src/release.c"),
        "int release_only(void) { return 1; }\n",
    )
    .unwrap();
    fs::write(
        p.root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(cdbsel LANGUAGES C)\n\
         add_executable(cdbsel src/main.c)\n\
         target_compile_definitions(cdbsel PRIVATE KV_CONFIG_$<UPPER_CASE:$<CONFIG>>)\n\
         if(CMAKE_BUILD_TYPE STREQUAL \"Release\")\n  add_library(release_only STATIC src/release.c)\nendif()\n",
    )
    .unwrap();
    let presets = json!({"version":6,"configurePresets":[
        {"name":"base","hidden":true,"generator":"Ninja",
         "cacheVariables":{"CMAKE_EXPORT_COMPILE_COMMANDS":"ON"}},
        {"name":"gcc-debug","inherits":"base","binaryDir":"${sourceDir}/build/${presetName}",
         "cacheVariables":{"CMAKE_C_COMPILER":"gcc","CMAKE_BUILD_TYPE":"Debug"}},
        {"name":"clang-release","inherits":"base",
         "binaryDir":"${sourceParentDir}/${sourceDirName} saídas/$env{KV_KIND}",
         "environment":{"KV_KIND":"clang release"},
         "cacheVariables":{"CMAKE_C_COMPILER":"clang","CMAKE_BUILD_TYPE":"Release"}}
    ]});
    fs::write(p.root.join("CMakePresets.json"), presets.to_string()).unwrap();
    p
}

fn release_dir(p: &Project) -> PathBuf {
    p.parent.join("starter saídas/clang release")
}

/// O configure de outra ferramenta: o `cmake` real, com a query do file-api
/// que o `CLion` e o VS Code também deixam.
fn cli_configure(p: &Project, preset: &str, binary: &Path) {
    let query = binary.join(".cmake/api/v1/query");
    fs::create_dir_all(&query).unwrap();
    fs::write(query.join("codemodel-v2"), "").unwrap();
    let output = Command::new("cmake")
        .arg("--preset")
        .arg(preset)
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(binary.join("compile_commands.json").is_file());
}

/// Um clangd real que registra cada argv, separado por `---`.
fn recording_clangd(p: &mut Project) -> PathBuf {
    let log = p.parent.join("clangd argv.log");
    let wrapper = crate::write_executable(
        p.parent.join("clangd gravador"),
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\" >> '{}'; done\n\
             echo --- >> '{}'\nexec clangd \"$@\"\n",
            log.display(),
            log.display()
        ),
    );
    assert!(
        p.core
            .use_language_server_command("cpp", wrapper.to_str().unwrap(), &[])
    );
    log
}

fn last_argv(log: &Path) -> Vec<String> {
    let text = fs::read_to_string(log).unwrap_or_default();
    let blocks: Vec<&str> = text.split("---\n").filter(|b| !b.is_empty()).collect();
    blocks
        .last()
        .map(|block| block.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

fn spawns(log: &Path) -> usize {
    fs::read_to_string(log)
        .unwrap_or_default()
        .matches("---\n")
        .count()
}

fn open(p: &mut Project) {
    p.call("workspace.open", json!({"path":p.root}));
    p.call("workspace.trust", json!({}));
    p.wait("event.index.finished", |_| true);
}

fn status(p: &mut Project) -> Value {
    p.call("cmake.status", json!({}))
}

fn select(p: &mut Project, preset: Option<&str>) -> Value {
    p.call("cmake.selectConfiguration", json!({"preset":preset}))
}

/// Abre `main.c` no clangd (um `lsp.hover` com o buffer, como o editor) e
/// devolve a primeira publicação que `expected` aceita.
fn diagnostics(p: &mut Project, expected: impl Fn(&[String]) -> bool) -> Vec<String> {
    let path = p.root.join("src/main.c");
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = p.core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "lsp.hover",
            Some(json!({"path":path,"content":MAIN,"line":9,"column":5})),
        ));
        if response.response().result.is_some() {
            break;
        }
        assert!(Instant::now() < deadline, "{:?}", response.response());
        std::thread::sleep(Duration::from_millis(30));
    }
    let messages = |event: &Value| -> Vec<String> {
        event["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["message"].as_str().unwrap().to_owned())
            .collect()
    };
    let event = p.wait("event.lsp.diagnostics", |value| {
        value["path"] == json!(path) && expected(&messages(value))
    });
    messages(&event)
}

fn debug_signature(found: &[String]) -> bool {
    found.len() == 2
        && found.iter().any(|m| m.contains("kv-debug-needle"))
        && found.iter().any(|m| m.contains("kv-release-missing"))
}

fn release_signature(found: &[String]) -> bool {
    matches!(found, [only] if only.contains("kv-release-needle"))
}

fn context_after_reload(p: &mut Project, directory: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        while let Ok(event) = p.events.try_recv() {
            p.core.observe_notification(&event);
        }
        let context = p.call("index.context", json!({"path":p.root.join("src/main.c")}));
        if context["source"]
            .as_str()
            .is_some_and(|source| source.contains(directory))
        {
            return context;
        }
        assert!(Instant::now() < deadline, "{directory}: {context}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn defines(context: &Value) -> Vec<String> {
    context["unit"]["defines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn c05_preset_databases_configured_outside_the_ide_are_found_chosen_and_reach_clangd() {
    let mut p = fixture("cdb-presets");
    let debug = p.root.join("build/gcc-debug");
    cli_configure(&p, "gcc-debug", &debug);
    open(&mut p);
    let log = recording_clangd(&mut p);
    one_candidate_is_used(&mut p, &log, &debug);

    let release = release_dir(&p);
    cli_configure(&p, "clang-release", &release);
    open(&mut p);
    // O open reaplica o comando do clangd do workspace; o gravador volta.
    recording_clangd(&mut p);
    several_candidates_ask(&mut p, &release);
    choosing_restarts_clangd_without_configure(&mut p, &log, &release);
    an_unknown_choice_is_refused_and_null_asks_again(&mut p);
    the_chosen_database_goes_stale_and_the_ide_configure_returns(&mut p, &log, &debug);
}

fn one_candidate_is_used(p: &mut Project, log: &Path, debug: &Path) {
    // Uma candidata: usada, com origem, sem configure da IDE.
    let one = status(p);
    assert_eq!(one["configured"], false, "{one}");
    assert_eq!(one["cdbDirectory"], "build/gcc-debug", "{one}");
    assert_eq!(one["cdbOrigin"], "preset");
    assert_eq!(one["cdbPreset"], "gcc-debug");
    let context = context_after_reload(p, "build/gcc-debug");
    assert_eq!(
        context["source"],
        "compile_commands.json em build/gcc-debug (preset gcc-debug)"
    );
    assert!(
        context["unit"]["compiler"]
            .as_str()
            .unwrap()
            .ends_with("gcc"),
        "{context}"
    );
    assert!(
        defines(&context).contains(&"KV_CONFIG_DEBUG".to_owned()),
        "{context}"
    );
    diagnostics(p, debug_signature);
    assert!(
        last_argv(log).contains(&format!("--compile-commands-dir={}", debug.display())),
        "{:?}",
        last_argv(log)
    );
}

fn several_candidates_ask(p: &mut Project, release: &Path) {
    let two = status(p);
    assert!(two.get("cdbDirectory").is_none(), "{two}");
    assert_eq!(
        two["cdbAbsence"],
        json!({"kind":"choiceRequired","reason":
            "2 configurações de presets têm compile_commands.json (gcc-debug, clang-release): escolha uma"})
    );
    assert_eq!(
        two["cdbCandidates"],
        json!([
            {"directory":"build/gcc-debug","origin":"preset","preset":"gcc-debug"},
            {"directory":release.display().to_string(),"origin":"preset","preset":"clang-release"}
        ])
    );
    let hint = p.call("index.context", json!({"path":p.root.join("src/main.c")}));
    assert!(hint.get("unit").is_none(), "{hint}");
    assert!(
        hint["hint"]
            .as_str()
            .unwrap()
            .starts_with("sem compile_commands.json em uso: 2 configurações"),
        "{hint}"
    );
}

fn choosing_restarts_clangd_without_configure(p: &mut Project, log: &Path, release: &Path) {
    // Escolher ativa sem reconfigurar; o clangd reinicia na pasta nova.
    let before = spawns(log);
    let chosen = select(p, Some("clang-release"));
    assert_eq!(chosen["cdbOrigin"], "configuration", "{chosen}");
    assert_eq!(chosen["cdbDirectory"], release.display().to_string());
    assert_eq!(chosen["selectedConfiguration"], "clang-release");
    assert!(!p.root.join(".kinein/build").exists());
    p.wait("event.lsp.restarted", |value| value["language"] == "cpp");
    let context = context_after_reload(p, "saídas/clang release");
    assert!(
        context["unit"]["compiler"]
            .as_str()
            .unwrap()
            .ends_with("clang"),
        "{context}"
    );
    assert!(defines(&context).contains(&"KV_CONFIG_RELEASE".to_owned()));
    diagnostics(p, release_signature);
    assert_eq!(spawns(log), before + 1);
    assert!(last_argv(log).contains(&format!("--compile-commands-dir={}", release.display())));
    // O codemodel é o da mesma pasta (C1-17): só o Release tem `release_only`.
    let index = p.call("index.status", json!({}));
    assert_eq!(index["context"]["cmakeTargets"], 2, "{index}");
}

fn an_unknown_choice_is_refused_and_null_asks_again(p: &mut Project) {
    // Recusa sem gravar; voltar ao automático pede escolha de novo.
    let refused = p.core.handle_request(&JsonRpcRequest::new(
        9_i64,
        "cmake.selectConfiguration",
        Some(json!({"preset":"nao-existe"})),
    ));
    let error = refused.response().error.as_ref().expect("refused");
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert_eq!(status(p)["selectedConfiguration"], "clang-release");
    let automatic = select(p, None);
    assert_eq!(
        automatic["cdbAbsence"]["kind"], "choiceRequired",
        "{automatic}"
    );
    assert!(!p.root.join(".kinein/configuration.json").exists());
}

fn the_chosen_database_goes_stale_and_the_ide_configure_returns(
    p: &mut Project,
    log: &Path,
    debug: &Path,
) {
    // A CDB escolhida envelhece com o CMakeLists e nada a substitui.
    select(p, Some("gcc-debug"));
    diagnostics(p, debug_signature);
    let debug_argv = format!("--compile-commands-dir={}", debug.display());
    assert!(last_argv(log).contains(&debug_argv), "{:?}", last_argv(log));
    let cmake_lists = p.root.join("CMakeLists.txt");
    let text = fs::read_to_string(&cmake_lists).unwrap();
    fs::write(&cmake_lists, format!("{text}# depois do configure\n")).unwrap();
    let file = fs::File::options().append(true).open(&cmake_lists).unwrap();
    file.set_modified(std::time::SystemTime::now() + Duration::from_secs(5))
        .unwrap();
    let stale = status(p);
    assert_eq!(stale["cdbDirectory"], "build/gcc-debug", "{stale}");
    assert_eq!(stale["cdbStale"], true);
    assert_eq!(stale["cdbStaleBecause"], "CMakeLists.txt");

    // O configure da IDE usa a configuração escolhida, no binaryDir dela
    // (C1-19): a CDB continua em `build/gcc-debug`, e o clangd não muda de
    // pasta. Até o 0.208.0 ela voltava a `.kinein/build`.
    let before = spawns(log);
    let job = p.call("cmake.configure", json!({}))["jobId"].clone();
    let started = p.wait("event.cmake.started", |e| e["jobId"] == job);
    assert_eq!(started["presetSource"], "configuration", "{started}");
    assert_eq!(started["preset"], "gcc-debug");
    assert_eq!(started["buildDir"], "build/gcc-debug");
    let finished = p.wait("event.cmake.finished", |e| e["jobId"] == job);
    assert_eq!(finished["success"], true, "{finished}");
    let configured = status(p);
    assert_eq!(configured["cdbOrigin"], "configuration", "{configured}");
    assert_eq!(configured["cdbDirectory"], "build/gcc-debug");
    assert_eq!(configured["selectedConfiguration"], "gcc-debug");
    assert_eq!(configured["preset"], "gcc-debug", "a IDE anota a pasta");
    assert!(!p.root.join(".kinein/build").exists());
    diagnostics(p, debug_signature);
    assert_eq!(spawns(log), before, "a mesma pasta: o clangd não reinicia");
    assert!(last_argv(log).contains(&debug_argv));
    assert_eq!(fs::read_to_string(p.root.join("src/main.c")).unwrap(), MAIN);
}

/// O `.clangd` da raiz que fixa a base vence (autor, 2026-11-09): a IDE o usa
/// como CDB em uso e não passa a opção que o sobreporia.
#[test]
fn c05_a_root_clangd_file_decides_and_the_ide_does_not_override_it() {
    let mut p = fixture("cdb-clangd");
    let release = release_dir(&p);
    cli_configure(&p, "gcc-debug", &p.root.join("build/gcc-debug"));
    cli_configure(&p, "clang-release", &release);
    let clangd = format!(
        "CompileFlags:\n  CompilationDatabase: '{}'\n",
        release.display()
    );
    fs::write(p.root.join(".clangd"), &clangd).unwrap();
    open(&mut p);
    let log = recording_clangd(&mut p);

    let decided = status(&mut p);
    assert_eq!(decided["cdbOrigin"], "clangd", "{decided}");
    assert_eq!(decided["cdbDirectory"], release.display().to_string());
    assert_eq!(decided["cdbPreset"], "clang-release");
    diagnostics(&mut p, release_signature);
    let argv = last_argv(&log);
    assert!(
        !argv.iter().any(|a| a.starts_with("--compile-commands-dir")),
        "{argv:?}"
    );

    // A escolha fica registrada para o configure, mas o `.clangd` decide.
    let chosen = select(&mut p, Some("gcc-debug"));
    assert_eq!(chosen["cdbOrigin"], "clangd", "{chosen}");
    assert_eq!(chosen["selectedConfiguration"], "gcc-debug");

    fs::write(
        p.root.join(".clangd"),
        format!("If:\n  PathMatch: src/.*\n{clangd}"),
    )
    .unwrap();
    let manual = status(&mut p);
    assert!(manual.get("cdbDirectory").is_none(), "{manual}");
    assert_eq!(manual["cdbAbsence"]["kind"], "clangd", "{manual}");
    assert!(
        manual["cdbAbsence"]["reason"]
            .as_str()
            .unwrap()
            .contains("fragmento condicional"),
        "{manual}"
    );
    assert_eq!(
        fs::read_to_string(p.root.join(".clangd"))
            .unwrap()
            .matches("If:")
            .count(),
        1
    );
}

/// C1-16: a CDB de `builddir/` aparecia na IDE e não chegava ao clangd.
#[test]
fn c05_a_builddir_database_reaches_clangd() {
    let mut p = project("cdb-builddir");
    fs::create_dir_all(p.root.join("src")).unwrap();
    fs::write(p.root.join("src/main.c"), MAIN).unwrap();
    fs::write(
        p.root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(meson_like LANGUAGES C)\n\
         add_executable(meson_like src/main.c)\n\
         target_compile_definitions(meson_like PRIVATE KV_CONFIG_RELEASE)\n",
    )
    .unwrap();
    let output = Command::new("cmake")
        .args([
            "-S",
            ".",
            "-B",
            "builddir",
            "-G",
            "Ninja",
            "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
        ])
        .current_dir(&p.root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    open(&mut p);
    let log = recording_clangd(&mut p);

    let found = status(&mut p);
    assert_eq!(found["cdbDirectory"], "builddir", "{found}");
    assert_eq!(found["cdbOrigin"], "directory");
    diagnostics(&mut p, release_signature);
    assert!(last_argv(&log).contains(&format!(
        "--compile-commands-dir={}",
        p.root.join("builddir").display()
    )));
}

/// A configuração escolhida que sai dos presets é recusada, nunca trocada
/// pelo único preset que sobrou.
#[test]
fn c05_a_chosen_configuration_removed_from_the_presets_is_refused_not_replaced() {
    let mut p = fixture("cdb-removed");
    cli_configure(&p, "gcc-debug", &p.root.join("build/gcc-debug"));
    open(&mut p);
    select(&mut p, Some("gcc-debug"));
    let path = p.root.join("CMakePresets.json");
    let mut presets: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    presets["configurePresets"]
        .as_array_mut()
        .unwrap()
        .retain(|preset| preset["name"] != "gcc-debug");
    fs::write(&path, presets.to_string()).unwrap();

    let removed = status(&mut p);
    assert!(removed.get("cdbDirectory").is_none(), "{removed}");
    assert_eq!(
        removed["cdbAbsence"],
        json!({"kind":"chosenConfiguration","reason":
            "a configuração escolhida (preset gcc-debug) não existe mais entre os presets: escolha outra"})
    );
    let refused = p.core.handle_request(&JsonRpcRequest::new(
        9_i64,
        "cmake.configure",
        Some(json!({})),
    ));
    let error = refused.response().error.as_ref().expect("refused");
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert_eq!(
        error.message,
        "configuração escolhida removida dos presets: gcc-debug; escolha outra"
    );
    assert_eq!(status(&mut p)["selectedConfiguration"], "gcc-debug");
    assert!(!p.root.join(".kinein/build").exists());
}
