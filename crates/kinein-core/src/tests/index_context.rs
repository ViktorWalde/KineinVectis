//! O CONTEXTO DE COMPILADOR por arquivo (pilar 0 do `roadmaps/42`, segunda
//! metade da decisao do autor em 2026-09-12: "integracao profunda de leitura
//! do contexto do codigo/compilador").
//!
//! O que se prova: que a `compile_commands.json` e' lida nas DUAS formas do
//! padrao (`arguments` e `command`), com caminho relativo resolvido contra o
//! `directory`, `-I`/`-isystem`/`-iquote` colados ou separados, `-D`, `-std=`
//! e `-o`; que cabecalho e arquivo fora da CDB recebem a dica certa; que um
//! `CMakeLists.txt` de SUBPASTA mais novo que a CDB a envelhece (a falha
//! silenciosa medida neste repositorio em 2026-09-12); que o arquivo Rust cai
//! no alvo do cargo por `src_path` exato, depois pelo diretorio mais longo,
//! `lib` em empate; que o interpretador Python segue a precedencia do
//! `roadmaps/29` §4.1; e, pelo despacho real, que `index.context` responde.
//!
//! Nada aqui roda ferramenta da maquina: o cargo, o poetry e o python sao
//! scripts escritos pelo teste, ou `None`.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use kinein_protocol::JsonRpcRequest;
use serde_json::json;

use super::core_with_empty_search_path;
use crate::index::context::{CompileContext, Ferramentas};
use crate::python::env::PythonTools;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-index-context-tests")
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

/// Um executavel que imprime `saida` e sai com 0.
fn script(caminho: &Path, saida: &str) -> PathBuf {
    escrever(caminho, &format!("#!/bin/sh\ncat <<'FIM'\n{saida}\nFIM\n"));
    std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
    caminho.to_path_buf()
}

fn sem_ferramentas() -> Ferramentas {
    Ferramentas::default()
}

fn datar(caminho: &Path, quando: SystemTime) {
    std::fs::File::options()
        .write(true)
        .open(caminho)
        .unwrap()
        .set_modified(quando)
        .unwrap();
}

#[test]
fn cdb_units_are_read_in_both_forms_with_flags_split_and_paths_resolved() {
    let raiz = temp_dir("cdb");
    escrever(&raiz.join("CMakeLists.txt"), "project(x)\n");
    escrever(&raiz.join("src/a.cpp"), "int a() { return 1; }\n");
    escrever(&raiz.join("src/b.c"), "int b(void) { return 2; }\n");
    escrever(&raiz.join("src/b.h"), "int b(void);\n");
    escrever(&raiz.join("src/solto.cpp"), "int solto() { return 3; }\n");
    let build = raiz.join("build");
    let cdb = json!([
        {
            "directory": build.display().to_string(),
            "arguments": ["/usr/bin/c++", "-DQT_CORE_LIB", "-D", "NIVEL=2", "-I", "../src",
                          "-isystem", "/usr/include/qt6", "-std=gnu++23", "-o",
                          "a.o", "-c", "../src/a.cpp"],
            "file": "../src/a.cpp",
            "output": "CMakeFiles/x.dir/src/a.cpp.o"
        },
        {
            "directory": build.display().to_string(),
            "command": format!("/usr/bin/cc -DNOME=\"a b\" -I{r}/src -iquote inc -std=c11 -c {r}/src/b.c -o b.o",
                               r = raiz.display()),
            "file": raiz.join("src/b.c").display().to_string()
        }
    ]);
    escrever(
        &build.join("compile_commands.json"),
        &serde_json::to_string(&cdb).unwrap(),
    );
    let ctx = CompileContext::load(&raiz, &sem_ferramentas());
    let resumo = ctx.summary();
    assert_eq!(resumo.cdb_directory.as_deref(), Some("build"));
    assert_eq!(resumo.cdb_entries, 2);
    assert!(!resumo.cdb_stale, "{resumo:?}");
    assert_eq!(resumo.cargo_packages, 0, "sem cargo injetado, nada roda");

    // A forma `arguments`, com `file` RELATIVO ao directory.
    let a = ctx.for_file(&raiz, Path::new("src/a.cpp"), "cpp");
    let unidade = a.unit.expect("a.cpp esta' na CDB");
    assert_eq!(unidade.compiler, "/usr/bin/c++");
    assert_eq!(unidade.directory, build.display().to_string());
    assert_eq!(unidade.standard.as_deref(), Some("gnu++23"));
    assert_eq!(
        unidade.includes,
        vec![
            build.join("../src").display().to_string(),
            "/usr/include/qt6".to_owned()
        ],
        "-I separado resolve contra o directory; -isystem absoluto fica"
    );
    assert_eq!(unidade.defines, vec!["QT_CORE_LIB", "NIVEL=2"]);
    assert_eq!(
        unidade.output.as_deref(),
        Some("CMakeFiles/x.dir/src/a.cpp.o"),
        "o campo output vence o -o (aqui diferentes de proposito)"
    );
    assert_eq!(unidade.arguments.len(), 13);
    assert_eq!(a.source.as_deref(), Some("compile_commands.json em build"));
    assert!(a.hint.is_none());

    // A forma `command`: aspas agrupam, -D colado, -I colado, -o vira output.
    let b = ctx.for_file(&raiz, &raiz.join("src/b.c"), "c");
    let unidade = b.unit.expect("b.c esta' na CDB");
    assert_eq!(unidade.compiler, "/usr/bin/cc");
    assert_eq!(unidade.defines, vec!["NOME=a b"]);
    assert_eq!(unidade.standard.as_deref(), Some("c11"));
    assert_eq!(
        unidade.includes,
        vec![
            raiz.join("src").display().to_string(),
            build.join("inc").display().to_string()
        ]
    );
    assert_eq!(unidade.output.as_deref(), Some("b.o"));

    // Cabecalho: nao e' unidade, e a dica diz por que.
    let h = ctx.for_file(&raiz, Path::new("src/b.h"), "c");
    assert!(h.unit.is_none());
    assert!(h.hint.as_deref().unwrap().contains("cabecalho"), "{h:?}");
    // Fonte fora da CDB, CDB em dia: nenhum alvo o compila.
    let solto = ctx.for_file(&raiz, Path::new("src/solto.cpp"), "cpp");
    assert!(solto.unit.is_none());
    assert!(
        solto
            .hint
            .as_deref()
            .unwrap()
            .contains("nenhum alvo o compila"),
        "{solto:?}"
    );
    // Outra linguagem: nada a dizer.
    let sh = ctx.for_file(&raiz, Path::new("go.sh"), "other");
    assert!(sh.unit.is_none() && sh.hint.is_none() && sh.source.is_none());
}

#[test]
fn without_a_cdb_every_c_file_gets_the_configure_hint() {
    let raiz = temp_dir("sem-cdb");
    escrever(&raiz.join("main.c"), "int main(void) { return 0; }\n");
    let ctx = CompileContext::load(&raiz, &sem_ferramentas());
    assert_eq!(ctx.summary().cdb_entries, 0);
    let m = ctx.for_file(&raiz, Path::new("main.c"), "c");
    assert!(
        m.hint
            .as_deref()
            .unwrap()
            .starts_with("sem compile_commands.json"),
        "{m:?}"
    );
}

/// A falha silenciosa medida em 2026-09-12: a CDB de 04/09 sem cinco fontes
/// que o `ui/CMakeLists.txt` de 12/09 acrescentou, e o `cdb::status` dizendo
/// "nao envelheceu" porque so' olha a raiz.
#[test]
fn a_newer_cmakelists_in_a_subfolder_marks_the_cdb_stale() {
    let raiz = temp_dir("subpasta");
    let ontem = SystemTime::now() - Duration::from_secs(86_400);
    let anteontem = ontem - Duration::from_secs(86_400);
    escrever(&raiz.join("CMakeLists.txt"), "add_subdirectory(ui)\n");
    escrever(
        &raiz.join("ui/CMakeLists.txt"),
        "add_executable(ui src/main.cpp src/novo.cpp)\n",
    );
    escrever(&raiz.join("ui/src/main.cpp"), "int main() {}\n");
    escrever(&raiz.join("ui/src/novo.cpp"), "int novo() { return 1; }\n");
    let build = raiz.join("build");
    let cdb = json!([{
        "directory": build.join("ui").display().to_string(),
        "arguments": ["c++", "-c", raiz.join("ui/src/main.cpp").display().to_string()],
        "file": raiz.join("ui/src/main.cpp").display().to_string()
    }]);
    escrever(
        &build.join("compile_commands.json"),
        &serde_json::to_string(&cdb).unwrap(),
    );
    datar(&raiz.join("CMakeLists.txt"), anteontem);
    datar(&build.join("compile_commands.json"), ontem);
    datar(&raiz.join("ui/CMakeLists.txt"), SystemTime::now());

    let ctx = CompileContext::load(&raiz, &sem_ferramentas());
    let resumo = ctx.summary();
    assert!(resumo.cdb_stale, "{resumo:?}");
    assert_eq!(
        resumo.cdb_stale_because.as_deref(),
        Some("ui/CMakeLists.txt")
    );
    let novo = ctx.for_file(&raiz, Path::new("ui/src/novo.cpp"), "cpp");
    assert!(
        novo.hint
            .as_deref()
            .unwrap()
            .contains("mais velha que ui/CMakeLists.txt"),
        "{novo:?}"
    );
    let main = ctx.for_file(&raiz, Path::new("ui/src/main.cpp"), "cpp");
    assert!(main.unit.is_some(), "a unidade velha ainda e' mostrada");
    assert!(
        main.hint.as_deref().unwrap().contains("reconfigure"),
        "{main:?}"
    );

    // Com o ui/CMakeLists.txt mais velho que a CDB, nada envelhece — nem um
    // CMakeLists.txt novo ACIMA da raiz (a subida para na raiz do workspace).
    datar(&raiz.join("ui/CMakeLists.txt"), anteontem);
    escrever(
        &raiz.parent().unwrap().join("CMakeLists.txt"),
        "project(alheio)\n",
    );
    let ctx = CompileContext::load(&raiz, &sem_ferramentas());
    assert!(!ctx.summary().cdb_stale, "{:?}", ctx.summary());
}

fn metadata_de_dois_pacotes(raiz: &Path) -> String {
    let r = raiz.display();
    json!({
        "packages": [
            {
                "name": "app", "edition": "2024",
                "manifest_path": format!("{r}/Cargo.toml"),
                "features": {"serial": [], "async": [], "default": []},
                "targets": [
                    {"name": "app", "kind": ["lib"], "src_path": format!("{r}/src/lib.rs")},
                    {"name": "app-cli", "kind": ["bin"], "src_path": format!("{r}/src/main.rs")},
                    {"name": "tool", "kind": ["bin"], "src_path": format!("{r}/src/bin/tool/main.rs")},
                    {"name": "build-script-build", "kind": ["custom-build"], "src_path": format!("{r}/build.rs")}
                ]
            },
            {
                "name": "gen", "edition": "2021",
                "manifest_path": format!("{r}/tools/gen/Cargo.toml"),
                "features": {},
                "targets": [
                    {"name": "gen", "kind": ["bin"], "src_path": format!("{r}/tools/gen/src/main.rs")}
                ]
            }
        ],
        "workspace_members": ["app", "gen"]
    })
    .to_string()
}

#[test]
fn rust_files_belong_to_the_cargo_target_exact_then_longest_dir_then_lib() {
    let _serial = crate::serializar_executaveis();
    let raiz = temp_dir("cargo");
    escrever(
        &raiz.join("Cargo.toml"),
        "[workspace]\nmembers = [\"tools/gen\"]\n",
    );
    let cargo = script(&raiz.join("bin/cargo"), &metadata_de_dois_pacotes(&raiz));
    let ferramentas = Ferramentas {
        cargo: Some(cargo),
        ..Ferramentas::default()
    };
    let ctx = CompileContext::load(&raiz, &ferramentas);
    let resumo = ctx.summary();
    assert_eq!((resumo.cargo_packages, resumo.cargo_targets), (2, 5));

    let alvo = |rel: &str| {
        let c = ctx.for_file(&raiz, Path::new(rel), "rust");
        (
            c.cargo.map(|k| (k.package, k.target, k.kind)),
            c.hint,
            c.source,
        )
    };
    let (main, _, fonte) = alvo("src/main.rs");
    assert_eq!(
        main,
        Some(("app".into(), "app-cli".into(), "bin".into())),
        "src_path exato do bin"
    );
    assert_eq!(fonte.as_deref(), Some("cargo metadata --no-deps"));
    assert_eq!(
        alvo("src/lib.rs").0,
        Some(("app".into(), "app".into(), "lib".into()))
    );
    assert_eq!(
        alvo("src/motor/mod.rs").0,
        Some(("app".into(), "app".into(), "lib".into())),
        "lib e bin dividem src/: o lib vence o empate"
    );
    assert_eq!(
        alvo("tools/gen/src/util.rs").0,
        Some(("gen".into(), "gen".into(), "bin".into())),
        "o pacote aninhado e' dono do que esta' embaixo dele"
    );
    assert_eq!(
        alvo("src/bin/tool/helpers.rs").0,
        Some(("app".into(), "tool".into(), "bin".into())),
        "src/bin/tool/ esta' dentro de src/, mas o diretorio mais longo vence o lib"
    );
    assert_eq!(
        alvo("build.rs").0,
        Some((
            "app".into(),
            "build-script-build".into(),
            "custom-build".into()
        )),
        "o build.rs casa exato"
    );
    let (fora, dica, _) = alvo("scripts/fora.rs");
    assert!(
        fora.is_none(),
        "o build.rs mora na raiz e NAO e' dono do resto por prefixo: {fora:?}"
    );
    assert!(
        dica.as_deref().unwrap().contains("fora de todo alvo"),
        "{dica:?}"
    );

    let detalhe = ctx
        .for_file(&raiz, Path::new("src/lib.rs"), "rust")
        .cargo
        .unwrap();
    assert_eq!(detalhe.edition, "2024");
    assert_eq!(
        detalhe.features,
        vec!["async", "default", "serial"],
        "ordenadas"
    );
    assert_eq!(
        detalhe.manifest,
        raiz.join("Cargo.toml").display().to_string()
    );
    assert_eq!(
        detalhe.src_path,
        raiz.join("src/lib.rs").display().to_string()
    );

    // Sem cargo injetado: o Cargo.toml existe, mas a IDE nao roda nada e diz.
    let sem = CompileContext::load(&raiz, &sem_ferramentas());
    assert_eq!(sem.summary().cargo_targets, 0);
    let (nada, dica, _) = {
        let c = sem.for_file(&raiz, Path::new("src/main.rs"), "rust");
        (c.cargo, c.hint, c.source)
    };
    assert!(nada.is_none());
    assert!(
        dica.as_deref().unwrap().starts_with("sem `cargo metadata`"),
        "{dica:?}"
    );

    // Cargo que falha: sem panico, sem alvos.
    escrever(&raiz.join("bin/cargo"), "#!/bin/sh\nexit 101\n");
    let quebrado = CompileContext::load(&raiz, &ferramentas);
    assert_eq!(quebrado.summary().cargo_targets, 0);
}

#[test]
fn the_python_interpreter_follows_virtual_env_then_venv_then_poetry_then_system() {
    let _serial = crate::serializar_executaveis();
    let raiz = temp_dir("python");
    escrever(&raiz.join("tools/gera.py"), "def gera():\n    pass\n");
    let sistema = script(&raiz.join("bin/python3"), "Python 9.9.9");
    let py = |rel: &str| {
        let caminho = raiz.join(rel).join("bin/python");
        script(&caminho, "Python 3.99.0");
        caminho
    };

    // 1. Nada: None, e o arquivo Python recebe a dica.
    let ctx = CompileContext::load(&raiz, &sem_ferramentas());
    assert!(ctx.summary().python_interpreter.is_none());
    let g = ctx.for_file(&raiz, Path::new("tools/gera.py"), "python");
    assert!(g.python.is_none());
    assert!(
        g.hint.as_deref().unwrap().contains("nenhum interpretador"),
        "{g:?}"
    );

    // 2. So' o sistema: vem com aviso e com a versao medida.
    let so_sistema = Ferramentas {
        python: PythonTools {
            python_sistema: Some(sistema),
            medir_versao: true,
            ..PythonTools::default()
        },
        ..Ferramentas::default()
    };
    let ctx = CompileContext::load(&raiz, &so_sistema);
    let g = ctx.for_file(&raiz, Path::new("tools/gera.py"), "python");
    let env = g.python.expect("sistema");
    assert_eq!(env.origin, "sistema");
    assert_eq!(env.version.as_deref(), Some("Python 9.9.9"));
    assert!(env.warning.as_deref().unwrap().contains("quebra a distro"));
    assert_eq!(g.source.as_deref(), Some("interpretador por sistema"));

    // 3. poetry.lock + poetry que responde um ambiente existente: vence o sistema.
    escrever(&raiz.join("poetry.lock"), "");
    let ambiente_poetry = raiz.join("caches/virtualenvs/app-py3.12");
    py("caches/virtualenvs/app-py3.12");
    let poetry = script(
        &raiz.join("bin/poetry"),
        &ambiente_poetry.display().to_string(),
    );
    let com_poetry = Ferramentas {
        python: PythonTools {
            poetry: Some(poetry),
            ..so_sistema.python.clone()
        },
        ..so_sistema.clone()
    };
    let ctx = CompileContext::load(&raiz, &com_poetry);
    let env = ctx.summary();
    assert_eq!(env.python_origin.as_deref(), Some("poetry"));
    assert_eq!(
        env.python_interpreter.as_deref(),
        Some(ambiente_poetry.join("bin/python").to_str().unwrap())
    );
    // poetry.lock sem o poetry injetado: cai para o sistema, sem rodar nada.
    let ctx = CompileContext::load(&raiz, &so_sistema);
    assert_eq!(ctx.summary().python_origin.as_deref(), Some("sistema"));

    // 4. venv/ e .venv/ na raiz vencem o poetry; .venv vence venv.
    py("venv");
    let ctx = CompileContext::load(&raiz, &com_poetry);
    assert_eq!(ctx.summary().python_origin.as_deref(), Some("venv"));
    let ponto = py(".venv");
    let ctx = CompileContext::load(&raiz, &com_poetry);
    let resumo = ctx.summary();
    assert_eq!(resumo.python_origin.as_deref(), Some(".venv"));
    assert_eq!(resumo.python_interpreter.as_deref(), ponto.to_str());
    assert!(
        ctx.for_file(&raiz, Path::new("tools/gera.py"), "python")
            .python
            .unwrap()
            .warning
            .is_none(),
        "so' o sistema avisa"
    );

    // 5. VIRTUAL_ENV ativo vence tudo — mas so' se o interpretador existir.
    let ativo = py("ativos/outro");
    let com_ativo = Ferramentas {
        python: PythonTools {
            virtual_env: Some(raiz.join("ativos/outro").display().to_string()),
            ..com_poetry.python.clone()
        },
        ..com_poetry.clone()
    };
    let ctx = CompileContext::load(&raiz, &com_ativo);
    let resumo = ctx.summary();
    assert_eq!(resumo.python_origin.as_deref(), Some("VIRTUAL_ENV"));
    assert_eq!(resumo.python_interpreter.as_deref(), ativo.to_str());
    let fantasma = Ferramentas {
        python: PythonTools {
            virtual_env: Some(raiz.join("ativos/apagado").display().to_string()),
            ..com_poetry.python.clone()
        },
        ..com_poetry
    };
    let ctx = CompileContext::load(&raiz, &fantasma);
    assert_eq!(
        ctx.summary().python_origin.as_deref(),
        Some(".venv"),
        "VIRTUAL_ENV apontando para o nada nao vale"
    );
}

/// Pelo despacho real: `index.context` exige workspace, classifica a
/// linguagem como o indice (Python incluido) e responde mesmo sem CDB nem
/// cargo — com a dica, nao com erro. O `index.status` carrega o resumo.
#[test]
fn index_context_answers_through_the_dispatcher() {
    let raiz = temp_dir("despacho");
    escrever(&raiz.join("src/main.c"), "int main(void) { return 0; }\n");
    escrever(&raiz.join("tools/gera.py"), "def gera():\n    pass\n");
    let mut core = core_with_empty_search_path("index-context");
    let sem = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "index.context",
        Some(json!({ "path": "src/main.c" })),
    ));
    assert!(
        sem.response().error.is_some(),
        "sem workspace nao ha' contexto"
    );
    let aberto = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let status = core.handle_request(&JsonRpcRequest::new(3_i64, "index.status", Some(json!({}))));
    let contexto = status.response().result.clone().unwrap()["context"].clone();
    assert_eq!(contexto["cdbEntries"], 0, "{contexto}");
    assert_eq!(contexto["cargoPackages"], 0);
    assert!(
        contexto.get("pythonInterpreter").is_none(),
        "PATH vazio: sem python"
    );

    let c = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "index.context",
        Some(json!({ "path": "src/main.c" })),
    ));
    let r = c.response().result.clone().unwrap();
    assert_eq!(r["language"], "c");
    assert_eq!(r["path"], raiz.join("src/main.c").display().to_string());
    assert!(
        r["hint"]
            .as_str()
            .unwrap()
            .starts_with("sem compile_commands.json"),
        "{r}"
    );

    let p = core.handle_request(&JsonRpcRequest::new(
        5_i64,
        "index.context",
        Some(json!({ "path": raiz.join("tools/gera.py").to_str().unwrap() })),
    ));
    let r = p.response().result.clone().unwrap();
    assert_eq!(r["language"], "python");
    assert!(
        r["hint"].as_str().unwrap().contains("nenhum interpretador"),
        "{r}"
    );

    let ruim = core.handle_request(&JsonRpcRequest::new(
        6_i64,
        "index.context",
        Some(json!({})),
    ));
    assert!(ruim.response().error.is_some(), "path e' obrigatorio");
}

/// O contexto SEGUE o build: um configure que escreve a CDB e um `Cargo.toml`
/// salvo recarregam o contexto sem reconstruir o indice dos arquivos.
#[test]
fn the_context_is_reloaded_after_configure_and_after_a_manifest_change() {
    let raiz = temp_dir("recarga");
    escrever(&raiz.join("src/main.c"), "int main(void) { return 0; }\n");
    let mut core = core_with_empty_search_path("index-context-recarga");
    let aberto = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    let contexto = |core: &mut crate::Core, id: i64| {
        core.handle_request(&JsonRpcRequest::new(id, "index.status", Some(json!({}))))
            .response()
            .result
            .clone()
            .unwrap()["context"]
            .clone()
    };
    assert_eq!(contexto(&mut core, 2)["cdbEntries"], 0);

    // O configure escreve a CDB; o evento chega; a unidade passa a existir.
    let cdb = json!([{
        "directory": raiz.join("build").display().to_string(),
        "arguments": ["cc", "-std=c17", "-c", raiz.join("src/main.c").display().to_string()],
        "file": raiz.join("src/main.c").display().to_string()
    }]);
    escrever(
        &raiz.join("build/compile_commands.json"),
        &serde_json::to_string(&cdb).unwrap(),
    );
    core.observe_notification(&JsonRpcRequest::notification(
        "event.cmake.finished",
        Some(json!({ "success": true })),
    ));
    assert_eq!(contexto(&mut core, 3)["cdbEntries"], 1);
    let unidade = core
        .handle_request(&JsonRpcRequest::new(
            4_i64,
            "index.context",
            Some(json!({ "path": "src/main.c" })),
        ))
        .response()
        .result
        .clone()
        .unwrap();
    assert_eq!(unidade["unit"]["standard"], "c17", "{unidade}");

    // Um evento que nao e' de build nao recarrega: a CDB reescrita fica velha
    // no contexto ate' o proximo configure ou Cargo.toml.
    let cdb2 = json!([]);
    escrever(
        &raiz.join("build/compile_commands.json"),
        &serde_json::to_string(&cdb2).unwrap(),
    );
    core.observe_notification(&JsonRpcRequest::notification(
        "event.fs.changed",
        Some(json!({ "changes": [{ "path": raiz.join("src/main.c").display().to_string(), "kind": "modified" }] })),
    ));
    assert_eq!(
        contexto(&mut core, 5)["cdbEntries"],
        1,
        "so' o arquivo foi reindexado"
    );
    core.observe_notification(&JsonRpcRequest::notification(
        "event.fs.changed",
        Some(json!({ "changes": [{ "path": raiz.join("Cargo.toml").display().to_string(), "kind": "created" }] })),
    ));
    assert_eq!(
        contexto(&mut core, 6)["cdbEntries"],
        0,
        "o Cargo.toml recarrega tudo"
    );
}
