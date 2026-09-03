//! Toolchain como entidade (`toolchain.get` / `toolchain.set`).
//!
//! O que estes testes provam nao e "o JSON tem os campos certos" — e que a
//! escolha do usuario **muda o comando que roda**. Um seletor bonito que nao
//! altera o `cmake` seria a mesma classe de defeito do `cdbStale`, que o core
//! media desde o `0.62.0` e a UI descartava: dado medido que nao chega ao
//! efeito e indistinguivel de nao ter sido medido.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::{Core, toolchain::Toolchain};
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, ToolInfo, ToolStatus, ToolchainRole};

/// Um workspace `CMake` com um `PATH` FALSO que contem os binarios pedidos.
///
/// Criar executaveis de mentira (e nao usar os da maquina) e o que torna o
/// teste hermetico: ele passa igual numa maquina sem `gcc` instalado.
fn workspace_with_tools(name: &str, binarios: &[&str]) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-toolchain-{name}", std::process::id()));
    if base.exists() {
        std::fs::remove_dir_all(&base).unwrap();
    }
    let root = base.join("projeto");
    let bin = base.join("bin");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(
        root.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.24)\nproject(demo CXX)\n",
    )
    .unwrap();

    for binario in binarios {
        let caminho = bin.join(binario);
        std::fs::write(&caminho, "#!/bin/sh\necho 1.0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissoes = std::fs::metadata(&caminho).unwrap().permissions();
            permissoes.set_mode(0o755);
            std::fs::set_permissions(&caminho, permissoes).unwrap();
        }
    }
    (root.canonicalize().unwrap(), bin)
}

/// Um core cujo detector le APENAS o `bin` falso: a maquina real nao entra.
fn core_with_path(bin: &Path) -> Core {
    Core::with_detector(crate::tools::ToolDetector::with_search_path(bin))
}

fn open(core: &mut Core, root: &Path) {
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": root.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none(), "workspace.open falhou");
}

fn call(core: &mut Core, method: &str, params: &Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(5_i64, method, Some(params.clone())));
    let response = outcome.response();
    assert!(
        response.error.is_none(),
        "{method} {params} falhou: {:?}",
        response.error
    );
    response.result.clone().unwrap()
}

fn detectado(id: &str, caminho: &str) -> ToolInfo {
    ToolInfo {
        id: id.to_owned(),
        display_name: id.to_owned(),
        status: ToolStatus::Detected,
        path: Some(caminho.to_owned()),
        version: None,
        suggested_install: None,
        message: None,
    }
}

#[test]
fn toolchain_methods_require_an_open_workspace() {
    let mut core = core_with_empty_search_path("toolchain-guarda");
    for (method, params) in [
        ("toolchain.get", json!({})),
        ("toolchain.set", json!({ "role": "cxxCompiler" })),
    ] {
        let outcome = core.handle_request(&JsonRpcRequest::new(4_i64, method, Some(params)));
        assert_eq!(
            outcome.response().error.as_ref().unwrap().code,
            JsonRpcErrorCode::InvalidRequest,
            "{method}"
        );
    }
}

/// Sem escolha, tudo e automatico — o comportamento historico, intacto.
#[test]
fn a_fresh_workspace_is_fully_automatic() {
    let (root, bin) = workspace_with_tools("novo", &["clang", "clang++", "gcc", "g++", "cmake"]);
    let mut core = core_with_path(&bin);
    open(&mut core, &root);

    let resultado = call(&mut core, "toolchain.get", &json!({}));
    let selecoes = resultado["selections"].as_array().unwrap();

    // 6 desde 2026-09-03: `debugAdapter` entrou com a etapa 22 do roadmaps/35,
    // quando o adaptador DAP deixou de ser constante.
    assert_eq!(selecoes.len(), 6, "um por papel do protocolo");
    for selecao in selecoes {
        assert!(
            selecao.get("id").is_none(),
            "nada pode nascer fixado: {selecao}"
        );
    }
    // O `resolvedPath` do automatico e' informacao para a UI, nao fixacao.
    let cxx = selecoes
        .iter()
        .find(|selecao| selecao["role"] == "cxxCompiler")
        .unwrap();
    assert!(
        cxx["resolvedPath"].as_str().unwrap().ends_with("/clang++"),
        "o automatico descreve o primeiro detectado: {cxx}"
    );
}

/// So se oferece o que existe NESTA maquina.
#[test]
fn only_detected_candidates_are_offered() {
    let (root, bin) = workspace_with_tools("so-clang", &["clang", "clang++", "cmake", "ninja"]);
    let mut core = core_with_path(&bin);
    open(&mut core, &root);

    let resultado = call(&mut core, "toolchain.get", &json!({}));
    let cxx: Vec<&str> = resultado["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|candidato| candidato["role"] == "cxxCompiler")
        .map(|candidato| candidato["id"].as_str().unwrap())
        .collect();

    assert_eq!(cxx, ["clangxx"], "sem g++ instalado, sem G++ na lista");

    // E recusar e' de verdade: escolher o que nao existe nao "fica pendente".
    let recusa = core.handle_request(&JsonRpcRequest::new(
        6_i64,
        "toolchain.set",
        Some(json!({ "role": "cxxCompiler", "id": "gxx" })),
    ));
    let erro = recusa.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(
        erro.message.contains("nao foi detectado"),
        "{}",
        erro.message
    );
}

/// A escolha persiste no workspace e volta ao automatico quando liberada.
#[test]
fn a_choice_persists_in_the_workspace_and_can_be_released() {
    let (root, bin) =
        workspace_with_tools("persiste", &["clang", "clang++", "gcc", "g++", "cmake"]);
    let mut core = core_with_path(&bin);
    open(&mut core, &root);

    let fixado = call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "cxxCompiler", "id": "gxx" }),
    );
    let cxx = |resultado: &Value| -> Option<String> {
        resultado["selections"]
            .as_array()?
            .iter()
            .find(|selecao| selecao["role"] == "cxxCompiler")?
            .get("id")?
            .as_str()
            .map(str::to_owned)
    };
    assert_eq!(cxx(&fixado).as_deref(), Some("gxx"));
    assert!(root.join(".kinein/toolchain.json").is_file());

    // Um core NOVO le a mesma escolha: e estado do workspace, nao da sessao.
    let mut outro = core_with_path(&bin);
    open(&mut outro, &root);
    assert_eq!(
        cxx(&call(&mut outro, "toolchain.get", &json!({}))).as_deref(),
        Some("gxx")
    );

    let liberado = call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "cxxCompiler" }),
    );
    assert_eq!(cxx(&liberado), None);
}

/// **O teste que importa**: a escolha muda o COMANDO que vai rodar.
#[test]
fn the_choice_reaches_the_cmake_command_line() {
    let (root, bin) = workspace_with_tools(
        "comando",
        &["clang", "clang++", "gcc", "g++", "cmake", "ninja"],
    );
    let mut core = core_with_path(&bin);
    open(&mut core, &root);

    let ferramentas = vec![
        detectado("gxx", bin.join("g++").to_str().unwrap()),
        detectado("gcc", bin.join("gcc").to_str().unwrap()),
        detectado("ninja", bin.join("ninja").to_str().unwrap()),
        detectado("cmake", bin.join("cmake").to_str().unwrap()),
    ];

    // Automatico: o comando sai como sempre saiu.
    let automatico = Toolchain::resolve(&root, &ferramentas);
    let argumentos = |toolchain: &Toolchain| -> Vec<String> {
        crate::cmake::configure_command(&root, None, toolchain)
            .get_args()
            .map(|argumento| argumento.to_string_lossy().into_owned())
            .collect()
    };
    let padrao = argumentos(&automatico);
    assert!(
        !padrao
            .iter()
            .any(|argumento| argumento.starts_with("-DCMAKE_CXX_COMPILER")),
        "sem escolha, nada pode ser fixado no cache do CMake: {padrao:?}"
    );
    assert!(!padrao.contains(&"-G".to_owned()));

    // Escolhido: aparece no comando, com o caminho absoluto do binario.
    call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "cxxCompiler", "id": "gxx" }),
    );
    call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "generator", "id": "Ninja" }),
    );
    let escolhido = Toolchain::resolve(&root, &ferramentas);
    let com_escolha = argumentos(&escolhido);

    assert!(com_escolha.contains(&"-G".to_owned()));
    assert!(com_escolha.contains(&"Ninja".to_owned()));
    assert!(
        com_escolha
            .iter()
            .any(|argumento| argumento
                == &format!("-DCMAKE_CXX_COMPILER={}", bin.join("g++").display())),
        "a escolha tem que virar argumento: {com_escolha:?}"
    );

    // E o EXECUTAVEL do proprio cmake segue o papel `cmake`.
    call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "cmake", "id": "cmake" }),
    );
    let com_cmake = Toolchain::resolve(&root, &ferramentas);
    assert_eq!(
        crate::cmake::configure_command(&root, None, &com_cmake)
            .get_program()
            .to_string_lossy(),
        bin.join("cmake").to_string_lossy(),
        "o cmake escolhido tem que ser o que roda"
    );
    assert_eq!(
        com_cmake.program_for(ToolchainRole::Cmake),
        Some(bin.join("cmake"))
    );
}
