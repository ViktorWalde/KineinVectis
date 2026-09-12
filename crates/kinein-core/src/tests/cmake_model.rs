//! O MODELO POR ALVO do `CMake` (pilar 0 do `roadmaps/42`; a "CDB em memoria"
//! do §8 item 4): o file-api `codemodel-v2` + `toolchains-v1` lidos do build
//! dir da IDE.
//!
//! O que se prova: que cada target vem com fontes (geradas marcadas),
//! artefatos absolutos, grupos de compilacao (linguagem, padrao, includes
//! absolutos, defines, fragmentos, sysroot) e dependencias RESOLVIDAS para
//! nome; que o inverso funciona (arquivo -> targets, cabecalho incluso); que
//! `cmake.targets.list` carrega o modelo e filtra utilitario; e que o
//! `index.context` de um arquivo SEM `compile_commands.json` vem do file-api
//! com o compilador da toolchains-v1 — e, COM a CDB, a CDB vence e os targets
//! continuam.

use std::path::{Path, PathBuf};

use kinein_protocol::JsonRpcRequest;
use serde_json::json;

use super::core_with_empty_search_path;
use crate::cmake::model::CmakeModel;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-cmake-model-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}

fn escrever(caminho: &Path, conteudo: &str) {
    std::fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    std::fs::write(caminho, conteudo).unwrap();
}

/// Um reply minimo e fiel a forma do cmake-file-api(7): dois targets reais,
/// um utilitario, e a toolchains-v1 (quando `com_toolchains`).
fn projeto(name: &str, com_toolchains: bool) -> PathBuf {
    let raiz = temp_dir(name);
    let build = raiz.join(".kinein/build");
    escrever(&raiz.join("CMakeLists.txt"), "project(app)\n");
    escrever(&raiz.join("src/main.cpp"), "int main() { return 0; }\n");
    escrever(&raiz.join("src/u.c"), "int u(void) { return 1; }\n");
    escrever(&raiz.join("src/u2.c"), "int u2(void) { return 2; }\n");
    escrever(&raiz.join("include/app.h"), "int main();\n");
    escrever(&raiz.join("tools/solto.cpp"), "int solto() { return 2; }\n");
    let reply = build.join(".cmake/api/v1/reply");
    let r = raiz.display();
    escrever(
        &reply.join("codemodel-v2-abc.json"),
        &json!({
            "kind": "codemodel", "version": {"major": 2, "minor": 8},
            "paths": {"source": r.to_string(), "build": build.display().to_string()},
            "configurations": [{"name": "", "targets": [
                {"name": "app", "id": "app::@1", "jsonFile": "target-app-1.json"},
                {"name": "util", "id": "util::@2", "jsonFile": "target-util-2.json"},
                {"name": "all_gen", "id": "all_gen::@3", "jsonFile": "target-all_gen-3.json"}
            ]}]
        })
        .to_string(),
    );
    escrever(
        &reply.join("target-app-1.json"),
        &json!({
            "name": "app", "id": "app::@1", "type": "EXECUTABLE",
            "paths": {"source": ".", "build": "."},
            "artifacts": [{"path": "app"}],
            "link": {"language": "CXX"},
            "dependencies": [{"id": "util::@2"}, {"id": "all_gen::@3"}, {"id": "Qt6::Core::@imported"}],
            "sources": [
                {"path": "src/main.cpp", "compileGroupIndex": 0, "sourceGroupIndex": 0},
                {"path": "include/app.h", "sourceGroupIndex": 1},
                {"path": ".kinein/build/gen/moc.cpp", "compileGroupIndex": 0, "isGenerated": true}
            ],
            "compileGroups": [{
                "language": "CXX",
                "languageStandard": {"standard": "20", "backtraces": []},
                "includes": [{"path": format!("{r}/include")}, {"path": "/usr/include/qt6"}],
                "defines": [{"define": "APP=1"}],
                "compileCommandFragments": [{"fragment": "-std=gnu++20"}, {"fragment": "-Wall"}],
                "sysroot": {"path": "/opt/sysroot"},
                "sourceIndexes": [0, 2]
            }]
        })
        .to_string(),
    );
    escrever(
        &reply.join("target-util-2.json"),
        &json!({
            "name": "util", "id": "util::@2", "type": "STATIC_LIBRARY",
            "paths": {"source": "src", "build": "src"},
            "artifacts": [{"path": "src/libutil.a"}],
            "sources": [{"path": "src/u.c", "compileGroupIndex": 0}, {"path": "src/u2.c", "compileGroupIndex": 1}],
            "compileGroups": [{
                "language": "C",
                "includes": [{"path": format!("{r}/src")}], "defines": [{"define": "UTIL"}, {"define": "APP=1"}],
                "compileCommandFragments": [{"fragment": "-O2"}],
                "sourceIndexes": [0]
            }, {
                "language": "C",
                "includes": [{"path": format!("{r}/src")}], "defines": [{"define": "UTIL"}, {"define": "SEGUNDO"}],
                "compileCommandFragments": [{"fragment": "-O0"}],
                "sourceIndexes": [1]
            }]
        })
        .to_string(),
    );
    escrever(
        &reply.join("target-all_gen-3.json"),
        &json!({"name": "all_gen", "id": "all_gen::@3", "type": "UTILITY", "sources": []})
            .to_string(),
    );
    if com_toolchains {
        escrever(
            &reply.join("toolchains-v1-abc.json"),
            &json!({"kind": "toolchains", "version": {"major": 1, "minor": 1}, "toolchains": [
                {"language": "C", "compiler": {"path": "/usr/bin/cc", "id": "GNU", "version": "15.1.0",
                    "implicit": {"includeDirectories": ["/usr/lib/gcc/include"]}}},
                {"language": "CXX", "compiler": {"path": "/usr/bin/c++", "id": "GNU", "version": "15.1.0",
                    "implicit": {"includeDirectories": ["/usr/include/c++/15", "/usr/lib/gcc/include"]}}}
            ]})
            .to_string(),
        );
    }
    raiz
}

#[test]
fn the_model_reads_targets_sources_groups_artifacts_and_resolves_dependencies() {
    let raiz = projeto("modelo", true);
    let modelo = CmakeModel::load(&raiz.join(".kinein/build")).expect("reply legivel");
    assert_eq!(modelo.source_dir, raiz);
    assert_eq!(modelo.build_dir, raiz.join(".kinein/build"));
    let nomes: Vec<&str> = modelo.targets.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(nomes, vec!["app", "util", "all_gen"]);

    let app = &modelo.targets[0];
    assert_eq!(app.kind, "EXECUTABLE");
    assert_eq!(
        app.artifacts,
        vec![raiz.join(".kinein/build/app")],
        "absoluto ao build dir"
    );
    assert_eq!(app.source_dir, raiz);
    assert_eq!(
        app.dependencies,
        vec!["util", "all_gen"],
        "ids resolvidos para nomes; o importado (fora do codemodel) nao entra"
    );
    assert_eq!(app.link_language.as_deref(), Some("CXX"));
    assert_eq!(app.sources.len(), 3);
    let gerada = app.sources.iter().find(|s| s.generated).unwrap();
    assert_eq!(gerada.path, raiz.join(".kinein/build/gen/moc.cpp"));
    let cabecalho = app
        .sources
        .iter()
        .find(|s| s.path.ends_with("app.h"))
        .unwrap();
    assert_eq!(cabecalho.compile_group, None, "listado, nao compilado");
    let grupo = &app.compile_groups[0];
    assert_eq!(grupo.language, "CXX");
    assert_eq!(grupo.standard.as_deref(), Some("20"));
    assert_eq!(
        grupo.includes,
        vec![
            raiz.join("include").display().to_string(),
            "/usr/include/qt6".to_owned()
        ]
    );
    assert_eq!(grupo.defines, vec!["APP=1"]);
    assert_eq!(grupo.fragments, vec!["-std=gnu++20", "-Wall"]);
    assert_eq!(grupo.sysroot.as_deref(), Some("/opt/sysroot"));

    let util = &modelo.targets[1];
    assert_eq!(
        util.source_dir,
        raiz.join("src"),
        "paths.source relativo a raiz"
    );
    assert_eq!(
        util.artifacts,
        vec![raiz.join(".kinein/build/src/libutil.a")]
    );

    // O inverso: arquivo -> targets; cabecalho incluso; fora de todo target = vazio.
    let donos = |rel: &str| -> Vec<String> {
        modelo
            .targets_for(&raiz.join(rel))
            .iter()
            .map(|t| t.name.clone())
            .collect()
    };
    assert_eq!(donos("src/main.cpp"), vec!["app"]);
    assert_eq!(donos("include/app.h"), vec!["app"]);
    assert_eq!(donos("src/u.c"), vec!["util"]);
    assert!(donos("tools/solto.cpp").is_empty());
    let (t, g) = modelo.compile_group_for(&raiz.join("src/u.c")).unwrap();
    assert_eq!((t.name.as_str(), g.language.as_str()), ("util", "C"));
    assert!(
        modelo
            .compile_group_for(&raiz.join("include/app.h"))
            .is_none(),
        "cabecalho nao tem grupo"
    );
    let cxx = modelo.compiler_for("CXX").unwrap();
    assert_eq!(
        (cxx.path.as_str(), cxx.id.as_deref(), cxx.version.as_deref()),
        ("/usr/bin/c++", Some("GNU"), Some("15.1.0"))
    );
    assert_eq!(cxx.implicit_includes.len(), 2);
    assert!(modelo.compiler_for("ASM").is_none());

    // Sem reply: None, sem panico.
    assert!(CmakeModel::load(&raiz.join("build-inexistente")).is_none());
}

#[test]
fn cmake_targets_list_carries_the_model_and_hides_utilities() {
    let raiz = projeto("targets", true);
    let mut core = core_with_empty_search_path("cmake-model-targets");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let resposta = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "cmake.targets.list",
        Some(json!({})),
    ));
    let r = resposta.response().result.clone().unwrap();
    assert_eq!(r["origin"], "fileApi");
    let alvos = r["targets"].as_array().unwrap();
    assert_eq!(alvos.len(), 2, "o utilitario fica de fora: {alvos:?}");
    let app = &alvos[0];
    assert_eq!(app["name"], "app");
    assert_eq!(app["kind"], "executable");
    assert_eq!(
        app["artifacts"][0],
        raiz.join(".kinein/build/app").display().to_string()
    );
    assert_eq!(app["sources"], 2, "main.cpp e app.h; a gerada nao conta");
    assert_eq!(app["generatedSources"], 1);
    assert_eq!(app["languages"], json!(["CXX"]));
    assert_eq!(app["standard"], "20");
    assert_eq!(app["includes"], 2);
    assert_eq!(app["defines"], 1);
    assert_eq!(app["sysroot"], "/opt/sysroot");
    assert_eq!(app["dependencies"], json!(["util", "all_gen"]));
    assert_eq!(app["sourceDir"], raiz.display().to_string());
    let util = &alvos[1];
    assert_eq!(util["kind"], "staticLibrary");
    assert_eq!(
        util["languages"],
        json!(["C"]),
        "dois grupos, uma linguagem"
    );
    assert_eq!(util["sources"], 2);
    assert_eq!(
        util["includes"], 1,
        "o mesmo include nos dois grupos conta uma vez"
    );
    assert_eq!(
        util["defines"], 3,
        "UTIL, APP=1, SEGUNDO — UTIL repetido conta uma vez"
    );
    assert!(util.get("sysroot").is_none());
    assert!(util.get("standard").is_none());
}

#[test]
fn index_context_falls_back_to_the_file_api_unit_and_always_names_the_targets() {
    let raiz = projeto("contexto", true);
    let mut core = core_with_empty_search_path("cmake-model-contexto");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let contexto = |core: &mut crate::Core, rel: &str| {
        core.handle_request(&JsonRpcRequest::new(
            2_i64,
            "index.context",
            Some(json!({ "path": rel })),
        ))
        .response()
        .result
        .clone()
        .unwrap()
    };
    let status = core
        .handle_request(&JsonRpcRequest::new(3_i64, "index.status", Some(json!({}))))
        .response()
        .result
        .clone()
        .unwrap();
    assert_eq!(status["context"]["cmakeTargets"], 3);
    assert_eq!(status["context"]["cdbEntries"], 0);

    // Sem CDB: a unidade vem do file-api, com o compilador da toolchains-v1
    // e o -std= do fragmento (mais fiel que o "20" do languageStandard).
    let main = contexto(&mut core, "src/main.cpp");
    assert_eq!(main["targets"], json!(["app"]), "{main}");
    let unidade = &main["unit"];
    assert_eq!(unidade["compiler"], "/usr/bin/c++");
    assert_eq!(unidade["standard"], "gnu++20");
    assert_eq!(
        unidade["directory"],
        raiz.join(".kinein/build").display().to_string()
    );
    assert_eq!(
        unidade["includes"],
        json!([
            raiz.join("include").display().to_string(),
            "/usr/include/qt6"
        ])
    );
    assert_eq!(unidade["defines"], json!(["APP=1"]));
    assert_eq!(unidade["arguments"], json!(["-std=gnu++20", "-Wall"]));
    assert!(unidade.get("output").is_none());
    assert!(
        main["source"]
            .as_str()
            .unwrap()
            .starts_with("file-api codemodel-v2 (target app)"),
        "{main}"
    );
    assert!(main.get("hint").is_none());
    // C: o compilador de C.
    let u = contexto(&mut core, "src/u.c");
    assert_eq!(u["unit"]["compiler"], "/usr/bin/cc");
    assert_eq!(u["targets"], json!(["util"]));
    assert!(
        u["unit"].get("standard").is_none(),
        "sem -std= e sem languageStandard"
    );
    // Cabecalho: listado pelo target (targets), mas sem unidade — com a dica.
    let h = contexto(&mut core, "include/app.h");
    assert_eq!(h["targets"], json!(["app"]));
    assert!(h.get("unit").is_none());
    assert!(h["hint"].as_str().unwrap().contains("cabecalho"), "{h}");
    // Fora de todo target e sem CDB: a dica de configurar, sem targets.
    let solto = contexto(&mut core, "tools/solto.cpp");
    assert!(solto.get("targets").is_none(), "{solto}");
    assert!(
        solto["hint"]
            .as_str()
            .unwrap()
            .starts_with("sem compile_commands.json")
    );

    // COM a CDB no mesmo build dir: a CDB vence a unidade; os targets ficam.
    escrever(
        &raiz.join(".kinein/build/compile_commands.json"),
        &json!([{
            "directory": raiz.join(".kinein/build").display().to_string(),
            "arguments": ["/usr/bin/c++", "-std=c++17", "-c", raiz.join("src/main.cpp").display().to_string()],
            "file": raiz.join("src/main.cpp").display().to_string()
        }])
        .to_string(),
    );
    core.observe_notification(&JsonRpcRequest::notification(
        "event.cmake.finished",
        Some(json!({ "success": true })),
    ));
    let main = contexto(&mut core, "src/main.cpp");
    assert_eq!(main["unit"]["standard"], "c++17");
    assert!(
        main["source"]
            .as_str()
            .unwrap()
            .starts_with("compile_commands.json")
    );
    assert_eq!(main["targets"], json!(["app"]));
}

/// Sem a `toolchains-v1` (`CMake` < 3.20, ou query antiga): a unidade do file-api
/// ainda vem, e o compilador e' DITO como desconhecido — nao inventado.
#[test]
fn without_toolchains_the_file_api_unit_says_the_compiler_is_the_kits() {
    let raiz = projeto("sem-toolchains", false);
    let mut core = core_with_empty_search_path("cmake-model-sem-toolchains");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let main = core
        .handle_request(&JsonRpcRequest::new(
            2_i64,
            "index.context",
            Some(json!({ "path": "src/main.cpp" })),
        ))
        .response()
        .result
        .clone()
        .unwrap();
    assert_eq!(main["unit"]["compiler"], "(CXX do kit)");
    assert_eq!(main["unit"]["standard"], "gnu++20");
}
