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
    // quando o adaptador DAP deixou de ser constante. 7 desde 2026-09-12:
    // `serialMonitor`, o processo da aba de terminal (integracoes/38 §6, E3).
    assert_eq!(selecoes.len(), 7, "um por papel do protocolo");
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

/// Os alvos Rust instalados vem do `rustup` DETECTADO (`integracoes/39`):
/// sem rustup, `rustTargets` nao existe na resposta — a IDE nao sabe e diz;
/// com um rustup falso, e' a lista dele, linha a linha.
#[test]
fn rust_targets_come_from_the_detected_rustup_or_are_absent() {
    let (root, bin) = workspace_with_tools("rust-targets", &["cargo"]);
    let mut core = core_with_path(&bin);
    open(&mut core, &root);
    let sem = call(&mut core, "toolchain.get", &json!({}));
    assert!(sem.get("rustTargets").is_none(), "{sem}");

    std::fs::write(
        bin.join("rustup"),
        "#!/bin/sh\nif [ \"$1\" = target ] && [ \"$3\" = --installed ]; then printf 'x86_64-unknown-linux-gnu\\nthumbv7em-none-eabihf\\n'; elif [ \"$1\" = target ]; then printf 'x86_64-unknown-linux-gnu\\nthumbv7em-none-eabihf\\nriscv32imc-unknown-none-elf\\n'; else echo 1.29.0; fi\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(bin.join("rustup"), std::fs::Permissions::from_mode(0o755))
            .unwrap();
    }
    let mut core = core_with_path(&bin);
    open(&mut core, &root);
    let com = call(&mut core, "toolchain.get", &json!({}));
    assert_eq!(
        com["rustTargets"],
        json!(["x86_64-unknown-linux-gnu", "thumbv7em-none-eabihf"]),
        "{com}"
    );
}

/// O compilador cross de Linux que a distro empacota SEM o sistema alvo
/// (medido no Fedora 44: `gcc-aarch64-linux-gnu` com sys-root vazio) recebe a
/// dica de sysroot; com `usr/include` no sysroot que ele declara, ou com um
/// sysroot no kit, nao; e bare metal (`arm-none-eabi-gcc`) nunca entra.
#[test]
fn a_linux_cross_compiler_without_a_sysroot_gets_the_hint() {
    let (root, bin) = workspace_with_tools("sysroot-hint", &["cmake", "arm-none-eabi-gcc"]);
    let sysroot = root.join("sysroot-vazio");
    std::fs::create_dir_all(&sysroot).unwrap();
    std::fs::write(
        bin.join("aarch64-linux-gnu-gcc"),
        format!(
            "#!/bin/sh\nif [ \"$1\" = -print-sysroot ]; then echo {}; else echo 16.1.1; fi\n",
            sysroot.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            bin.join("aarch64-linux-gnu-gcc"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    let mut core = core_with_path(&bin);
    open(&mut core, &root);
    // Automatico: o primeiro candidato detectado e' o arm-none-eabi-gcc (bare
    // metal) — sem dica.
    let auto = call(&mut core, "toolchain.get", &json!({}));
    assert!(auto.get("sysrootHint").is_none(), "{auto}");
    // Fixado o cross de Linux: a dica aparece e diz de onde vem um sysroot.
    let fixado = call(
        &mut core,
        "toolchain.set",
        &json!({ "role": "cCompiler", "id": "aarch64-linux-gnu-gcc" }),
    );
    let dica = fixado["sysrootHint"].as_str().unwrap_or_default();
    assert!(
        dica.contains("aarch64-linux-gnu-gcc") && dica.contains("Bootlin"),
        "{fixado}"
    );
    assert!(dica.contains("sem usr/include"), "{dica}");
    // Com usr/include no sysroot declarado: nada a dizer.
    std::fs::create_dir_all(sysroot.join("usr/include")).unwrap();
    let ok = call(&mut core, "toolchain.get", &json!({}));
    assert!(ok.get("sysrootHint").is_none(), "{ok}");
    std::fs::remove_dir_all(sysroot.join("usr")).unwrap();
    // Com sysroot no kit: a escolha do usuario vale, sem dica.
    let com_kit = call(
        &mut core,
        "toolchain.setKit",
        &json!({ "sysroot": root.join("meu-sysroot").display().to_string() }),
    );
    assert!(com_kit.get("sysrootHint").is_none(), "{com_kit}");
}

/// O provedor de instalacao (integracoes/39 §5, 2026-09-13): o catalogo
/// chega INTEIRO com URL, tamanho, sha256, licenca e fonte VISIVEIS antes de
/// qualquer clique; o estado desta maquina (instalada ou nao, e onde) vem da
/// pasta da IDE; num projeto STM32 a familia `cortex-m` e' a recomendada; um
/// id fora do catalogo e uma toolchain ja' instalada sao recusas — e o `tar`
/// ausente e' recusa ANTES de baixar. Nada aqui toca a rede: o download
/// real e' o teste do dominio (`toolchain::install`), com um servidor local.
#[test]
#[cfg(unix)]
fn the_install_catalogue_is_visible_before_any_click_and_refuses_what_it_must() {
    let (root, bin) = workspace_with_tools("instalavel", &["cmake", "tar"]);
    // Um .ioc (a fixture real do STM32Cube): o project.model deduz a familia
    // stm32, que o catalogo traduz para cortex-m.
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scripts/fixtures/projetos/stm32cube/fixture.ioc"
        ),
        root.join("fixture.ioc"),
    )
    .unwrap();
    let raiz = bin.parent().unwrap().join("toolchains");
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = Core::with_detector(
        crate::tools::ToolDetector::with_search_path(&bin).with_install_root(&raiz),
    );
    core.enable_lsp(sender);
    open(&mut core, &root);

    let lista = core.handle_request(&JsonRpcRequest::new(30_i64, "toolchain.installable", None));
    let result = lista.response().result.clone().unwrap();
    assert_eq!(result["installRoot"], raiz.display().to_string());
    assert_eq!(result["projectFamily"], "stm32", "{result}");
    let toolchains = result["toolchains"].as_array().unwrap();
    assert_eq!(toolchains.len(), crate::toolchain::install::CATALOGO.len());
    let arm = toolchains
        .iter()
        .find(|t| t["id"] == "arm-gnu-arm-none-eabi")
        .expect("Arm GNU no catalogo");
    assert_eq!(arm["version"], "15.2.rel1");
    assert!(
        arm["url"]
            .as_str()
            .unwrap()
            .starts_with("https://developer.arm.com/")
    );
    assert_eq!(arm["sha256"].as_str().unwrap().len(), 64);
    assert_eq!(arm["sizeBytes"], 155_499_480_u64);
    assert!(arm["license"].as_str().unwrap().contains("GPL"));
    assert!(arm["source"].as_str().unwrap().contains("2026-09-13"));
    assert_eq!(arm["installed"], false);
    assert_eq!(arm["recommended"], true, "STM32 e' Cortex-M");
    assert_eq!(
        arm["installDir"],
        raiz.join("arm-gnu-arm-none-eabi/15.2.rel1")
            .display()
            .to_string()
    );
    let bootlin = toolchains
        .iter()
        .find(|t| t["id"] == "bootlin-aarch64-glibc-stable")
        .unwrap();
    assert_eq!(
        bootlin["recommended"], false,
        "Linux aarch64 nao e' o alvo de um STM32"
    );

    // Fora do catalogo: INVALID_PARAMS que aponta o metodo da lista.
    let fora = core.handle_request(&JsonRpcRequest::new(
        31_i64,
        "toolchain.install",
        Some(json!({ "id": "latest" })),
    ));
    let erro = fora.response().error.clone().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(erro.message.contains("toolchain.installable"));

    // Ja' instalada (a pasta com bin/ existe): recusa que diz onde esta'.
    std::fs::create_dir_all(raiz.join("arm-gnu-arm-none-eabi/15.2.rel1/bin")).unwrap();
    let lista = core.handle_request(&JsonRpcRequest::new(32_i64, "toolchain.installable", None));
    let result = lista.response().result.clone().unwrap();
    let arm = result["toolchains"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "arm-gnu-arm-none-eabi")
        .unwrap();
    assert_eq!(arm["installed"], true);
    let de_novo = core.handle_request(&JsonRpcRequest::new(
        33_i64,
        "toolchain.install",
        Some(json!({ "id": "arm-gnu-arm-none-eabi" })),
    ));
    let erro = de_novo.response().error.clone().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert!(erro.message.contains("ja' esta' instalada"), "{erro:?}");

    // Sem `tar` no PATH falso: TOOL_NOT_FOUND antes de qualquer download.
    std::fs::remove_file(bin.join("tar")).unwrap();
    let sem_tar = core.handle_request(&JsonRpcRequest::new(
        34_i64,
        "toolchain.install",
        Some(json!({ "id": "xpack-riscv-none-elf-gcc" })),
    ));
    let erro = sem_tar.response().error.clone().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::ToolNotFound, "{erro:?}");
    assert!(erro.message.contains("tar"));
    drop(receiver);
}

/// Depois de `event.toolchain.installed` com sucesso, o registro de
/// ferramentas e' refeito e o `toolchain.get` lista o compilador que nasceu
/// na pasta da IDE — sem reiniciar, sem `tools.detect` a mao.
#[test]
#[cfg(unix)]
fn an_installed_toolchain_becomes_a_candidate_on_the_next_toolchain_get() {
    use std::os::unix::fs::PermissionsExt;
    let (root, bin) = workspace_with_tools("instalada-vira-candidato", &["cmake"]);
    let raiz = bin.parent().unwrap().join("toolchains");
    let (sender, _receiver) = std::sync::mpsc::channel();
    let mut core = Core::with_detector(
        crate::tools::ToolDetector::with_search_path(&bin).with_install_root(&raiz),
    );
    core.enable_lsp(sender);
    open(&mut core, &root);
    let antes = core.handle_request(&JsonRpcRequest::new(
        40_i64,
        "toolchain.get",
        Some(json!({})),
    ));
    let candidatos = |r: &Value| -> Vec<String> {
        r["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap().to_owned())
            .collect()
    };
    assert!(
        !candidatos(&antes.response().result.clone().unwrap())
            .contains(&"arm-none-eabi-gcc".to_owned())
    );

    // A "instalacao": a pasta nasce com o binario (o que o job faria).
    let gcc = raiz.join("arm-gnu-arm-none-eabi/15.2.rel1/bin/arm-none-eabi-gcc");
    std::fs::create_dir_all(gcc.parent().unwrap()).unwrap();
    std::fs::write(&gcc, "#!/bin/sh\necho 15.2\n").unwrap();
    std::fs::set_permissions(&gcc, std::fs::Permissions::from_mode(0o755)).unwrap();
    // Sem o evento, o registro antigo ainda vale (o toolchain.get e' barato).
    let ainda = core.handle_request(&JsonRpcRequest::new(
        41_i64,
        "toolchain.get",
        Some(json!({})),
    ));
    assert!(
        !candidatos(&ainda.response().result.clone().unwrap())
            .contains(&"arm-none-eabi-gcc".to_owned())
    );
    core.observe_notification(&JsonRpcRequest::notification(
        "event.toolchain.installed",
        Some(json!({ "jobId": "j", "id": "arm-gnu-arm-none-eabi", "version": "15.2.rel1", "path": gcc.parent().unwrap().parent().unwrap().to_str().unwrap(), "success": true })),
    ));
    let depois = core.handle_request(&JsonRpcRequest::new(
        42_i64,
        "toolchain.get",
        Some(json!({})),
    ));
    let result = depois.response().result.clone().unwrap();
    assert!(
        candidatos(&result).contains(&"arm-none-eabi-gcc".to_owned()),
        "{result}"
    );
    let c = result["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "arm-none-eabi-gcc")
        .unwrap();
    assert_eq!(c["path"], gcc.display().to_string());
}
