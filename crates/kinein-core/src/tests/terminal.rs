//! Dispatch do domínio `terminal.*`.
//!
//! Escrito em 2026-08-29, ANTES do split do `terminal.rs` (1374 linhas, o maior
//! arquivo do core e o maior débito da catraca). O arquivo tinha 18 testes
//! unitários e **nenhum** de integração: mexer nele era mexer sem rede.
//!
//! O que estes testes cobrem, e o que deliberadamente NÃO cobrem:
//!
//! ```text
//! COBRE     roteamento por `Core::handle_request`, guardas (sem workspace,
//!           sem manager), validacao de params, ciclo de vida completo e o
//!           CONTRATO do `event.terminal.render` — os nomes de campo que a
//!           UI le.
//! NAO COBRE traducao VT, codificacao de mouse, roda e estilo de cursor: isso
//!           ja tem 18 testes unitarios em `terminal.rs`, onde ficam mais
//!           baratos e mais precisos. Duplicar aqui seria cerimonia.
//! ```
//!
//! O teste de contrato é o que dá licença para o split: se o corte renomear ou
//! perder um campo do render, a UI para de desenhar em silêncio (o Qt não
//! reclama de propriedade ausente num `QVariantMap`), e nenhum outro gate pega.

use std::{sync::mpsc, time::Duration};

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

/// Diretório de trabalho para uma sessão de terminal.
fn temp_root(nome: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-terminal-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    dir.canonicalize().unwrap()
}

/// Core com terminal habilitado e um workspace aberto. Devolve o receptor de
/// eventos porque o terminal é assíncrono: a resposta do `terminal.input` diz
/// "aceitei", não "já apareceu na tela".
fn core_com_terminal(
    nome: &str,
) -> (
    crate::Core,
    mpsc::Receiver<JsonRpcRequest>,
    std::path::PathBuf,
) {
    let raiz = temp_root(nome);
    let (emissor, receptor) = mpsc::channel();
    let mut core = core_with_empty_search_path(&format!("terminal-{nome}"));
    core.enable_lsp(emissor);
    let aberto = core.handle_request(&JsonRpcRequest::new(
        900_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
    (core, receptor, raiz)
}

/// Espera o primeiro `event.terminal.render` cujo texto contenha `agulha`.
fn render_com(receptor: &mpsc::Receiver<JsonRpcRequest>, agulha: &str) -> Option<JsonRpcRequest> {
    while let Ok(evento) = receptor.recv_timeout(Duration::from_secs(10)) {
        if evento.method != "event.terminal.render" {
            if evento.method == "event.terminal.closed" {
                return None;
            }
            continue;
        }
        let texto = evento
            .params
            .as_ref()
            .and_then(|p| p["lines"].as_array())
            .map(|linhas| {
                linhas
                    .iter()
                    .filter_map(|linha| linha.as_array())
                    .flatten()
                    .filter_map(|span| span["text"].as_str())
                    .collect::<String>()
            })
            .unwrap_or_default();
        if texto.contains(agulha) {
            return Some(evento);
        }
    }
    None
}

#[test]
fn terminal_open_requires_an_open_workspace() {
    let (emissor, _receptor) = mpsc::channel();
    let mut core = core_with_empty_search_path("terminal-sem-workspace");
    core.enable_lsp(emissor);

    let saida = core.handle_request(&JsonRpcRequest::new(901_i64, "terminal.open", None));
    let erro = saida.response().error.as_ref().unwrap();

    assert_eq!(erro.code, JsonRpcErrorCode::InvalidRequest);
    assert_eq!(erro.message, "nenhum workspace aberto");
}

/// Sem `enable_lsp` não há `TerminalManager`: o core tem de dizer isso, e não
/// fingir sucesso. É o caminho dos loops leves (`run_json_lines`).
#[test]
fn terminal_methods_report_an_unavailable_manager_instead_of_pretending() {
    let mut core = core_with_empty_search_path("terminal-sem-manager");
    for (metodo, params) in [
        ("terminal.input", json!({ "id": "t1", "data": "ls\n" })),
        (
            "terminal.resize",
            json!({ "id": "t1", "cols": 80, "rows": 24 }),
        ),
        ("terminal.scroll", json!({ "id": "t1", "offset": 3 })),
        ("terminal.close", json!({ "id": "t1" })),
    ] {
        let saida = core.handle_request(&JsonRpcRequest::new(902_i64, metodo, Some(params)));
        let erro = saida
            .response()
            .error
            .as_ref()
            .unwrap_or_else(|| panic!("{metodo} devolveu sucesso sem manager"));
        assert_eq!(erro.code, JsonRpcErrorCode::InternalError, "{metodo}");
    }
}

#[test]
fn terminal_methods_validate_their_params_before_touching_the_session() {
    let (mut core, _receptor, _raiz) = core_com_terminal("params");
    for (metodo, params) in [
        ("terminal.input", json!({ "id": "t1" })),
        ("terminal.resize", json!({ "id": "t1", "cols": 80 })),
        ("terminal.scroll", json!({ "id": "t1" })),
        ("terminal.close", json!({})),
        ("terminal.mouse", json!({ "id": "t1", "col": 1 })),
    ] {
        let saida = core.handle_request(&JsonRpcRequest::new(903_i64, metodo, Some(params)));
        let erro = saida
            .response()
            .error
            .as_ref()
            .unwrap_or_else(|| panic!("{metodo} aceitou params incompletos"));
        assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams, "{metodo}");
    }
}

/// Um id que não existe é erro, não pânico e não sucesso silencioso.
#[test]
fn operations_on_an_unknown_session_id_fail_cleanly() {
    let (mut core, _receptor, _raiz) = core_com_terminal("id-desconhecido");
    for (metodo, params) in [
        ("terminal.input", json!({ "id": "fantasma", "data": "x" })),
        (
            "terminal.resize",
            json!({ "id": "fantasma", "cols": 80, "rows": 24 }),
        ),
        ("terminal.scroll", json!({ "id": "fantasma", "offset": 1 })),
        ("terminal.close", json!({ "id": "fantasma" })),
    ] {
        let saida = core.handle_request(&JsonRpcRequest::new(904_i64, metodo, Some(params)));
        assert!(
            saida.response().error.is_some(),
            "{metodo} aceitou uma sessao inexistente"
        );
    }
}

/// Ciclo completo pelo dispatch: abre, escreve, o comando roda no PTY de
/// verdade e o resultado volta no grid.
#[test]
fn open_input_and_close_round_trip_through_dispatch() {
    let (mut core, receptor, _raiz) = core_com_terminal("ciclo");

    let aberto = core.handle_request(&JsonRpcRequest::new(905_i64, "terminal.open", None));
    let resultado = aberto.response().result.as_ref().unwrap();
    let id = resultado["id"].as_str().expect("id da sessao").to_owned();
    assert!(!id.is_empty());
    assert!(
        !resultado["shell"].as_str().unwrap_or_default().is_empty(),
        "terminal.open tem de dizer QUAL shell subiu"
    );

    let escrito = core.handle_request(&JsonRpcRequest::new(
        906_i64,
        "terminal.input",
        Some(json!({ "id": id, "data": "echo kinein-vectis-ok\n" })),
    ));
    assert!(escrito.response().error.is_none());
    assert!(
        render_com(&receptor, "kinein-vectis-ok").is_some(),
        "a saida do comando nao chegou em event.terminal.render"
    );

    let fechado = core.handle_request(&JsonRpcRequest::new(
        907_i64,
        "terminal.close",
        Some(json!({ "id": id })),
    ));
    assert!(fechado.response().error.is_none());

    // Depois de fechar, a mesma sessao nao aceita mais nada.
    let depois = core.handle_request(&JsonRpcRequest::new(
        908_i64,
        "terminal.input",
        Some(json!({ "id": id, "data": "x" })),
    ));
    assert!(depois.response().error.is_some());
}

/// **Teste de CONTRATO do `event.terminal.render`.**
///
/// Estes nomes de campo não são detalhe interno: são exatamente o que o
/// `ui/qml` lê do `QVariantMap`. Um campo renomeado ou perdido faz a UI parar
/// de desenhar **em silêncio** — o Qt não reclama de propriedade ausente num
/// `QVariantMap`, o build passa, o qmllint passa e o terminal fica preto.
///
/// É este teste que dá licença para o split do `terminal.rs`: ele prova que o
/// corte não mexeu no que atravessa a fronteira.
#[test]
fn render_event_keeps_every_field_the_ui_reads() {
    let (mut core, receptor, _raiz) = core_com_terminal("contrato-render");
    let aberto = core.handle_request(&JsonRpcRequest::new(909_i64, "terminal.open", None));
    let id = aberto.response().result.as_ref().unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let escrito = core.handle_request(&JsonRpcRequest::new(
        910_i64,
        "terminal.input",
        Some(json!({ "id": id, "data": "echo contrato-render\n" })),
    ));
    assert!(escrito.response().error.is_none());

    let evento = render_com(&receptor, "contrato-render").expect("render com a saida");
    let params = evento.params.as_ref().expect("render tem params");

    // Raiz do payload.
    assert_eq!(params["id"].as_str(), Some(id.as_str()));
    assert!(params["cols"].is_u64(), "cols");
    assert!(params["rows"].is_u64(), "rows");
    assert!(params["scrollback"].is_u64(), "scrollback");
    assert!(params["scrollbackMax"].is_u64(), "scrollbackMax");
    assert!(params["alternateScreen"].is_boolean(), "alternateScreen");
    assert!(
        params["applicationCursor"].is_boolean(),
        "applicationCursor"
    );
    assert!(params["bracketedPaste"].is_boolean(), "bracketedPaste");

    // Cursor: a UI le os cinco.
    let cursor = &params["cursor"];
    assert!(cursor["row"].is_u64(), "cursor.row");
    assert!(cursor["col"].is_u64(), "cursor.col");
    assert!(cursor["visible"].is_boolean(), "cursor.visible");
    assert!(cursor["shape"].is_string(), "cursor.shape");
    assert!(cursor["blinking"].is_boolean(), "cursor.blinking");

    // Linhas: lista de listas de spans; a UI le `text` e `cells` de cada span.
    let linhas = params["lines"].as_array().expect("lines e' lista");
    assert_eq!(
        linhas.len() as u64,
        params["rows"].as_u64().unwrap(),
        "lines tem de ter uma entrada por linha do viewport"
    );
    let span = linhas
        .iter()
        .filter_map(|linha| linha.as_array())
        .flatten()
        .next()
        .expect("ao menos um span");
    assert!(span["text"].is_string(), "span.text");
    assert!(span["cells"].is_u64(), "span.cells");
}

/// Fechar o workspace tem de derrubar as sessões: um PTY sobrevivente segura
/// processo filho e continua emitindo render de um projeto que a UI já fechou.
///
/// **A primeira versão deste teste não conseguia reprovar** — ela verificava
/// apenas que `terminal.open` era recusado depois, e isso é verdade só porque
/// não há workspace, independentemente de as sessões terem morrido. Transformar
/// `close_all()` em no-op deixava o teste verde. É o vício do `PONTO_ATUAL`
/// §0.2i, e foi o teste de mutação que o pegou.
///
/// A prova real é o `event.terminal.closed` de CADA sessão: ele nasce no
/// `spawn_waiter`, quando o processo filho de fato termina.
#[test]
fn closing_the_workspace_closes_every_terminal() {
    let (mut core, receptor, _raiz) = core_com_terminal("fecha-workspace");
    let mut abertos = Vec::new();
    for id_pedido in [911_i64, 912_i64] {
        let aberto = core.handle_request(&JsonRpcRequest::new(id_pedido, "terminal.open", None));
        assert!(aberto.response().error.is_none());
        abertos.push(
            aberto.response().result.as_ref().unwrap()["id"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }

    let fechado = core.handle_request(&JsonRpcRequest::new(913_i64, "workspace.close", None));
    assert!(fechado.response().error.is_none());

    // Cada sessao tem de anunciar a propria morte. Sem isto o shell fica orfao.
    let mut mortas = std::collections::BTreeSet::new();
    while mortas.len() < abertos.len() {
        let Ok(evento) = receptor.recv_timeout(Duration::from_secs(10)) else {
            break;
        };
        if evento.method == "event.terminal.closed" {
            if let Some(id) = evento.params.as_ref().and_then(|p| p["id"].as_str()) {
                mortas.insert(id.to_owned());
            }
        }
    }
    for id in &abertos {
        assert!(
            mortas.contains(id),
            "a sessao {id} sobreviveu ao workspace.close (vivas: {mortas:?})"
        );
    }

    // E o core recusa abrir de novo, porque nao ha mais workspace.
    let reabrir = core.handle_request(&JsonRpcRequest::new(914_i64, "terminal.open", None));
    assert_eq!(
        reabrir.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );
}

/// `terminal.resize` e `terminal.scroll` chegam à sessão viva e o render
/// seguinte reflete o pedido. Sem isto, o split poderia desligar o caminho e
/// só a tela contaria.
#[test]
fn resize_reaches_the_live_session_and_shows_up_in_the_next_render() {
    let (mut core, receptor, _raiz) = core_com_terminal("resize");
    let aberto = core.handle_request(&JsonRpcRequest::new(915_i64, "terminal.open", None));
    let id = aberto.response().result.as_ref().unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let redimensionado = core.handle_request(&JsonRpcRequest::new(
        916_i64,
        "terminal.resize",
        Some(json!({ "id": id, "cols": 100, "rows": 30 })),
    ));
    assert!(redimensionado.response().error.is_none());

    let escrito = core.handle_request(&JsonRpcRequest::new(
        917_i64,
        "terminal.input",
        Some(json!({ "id": id, "data": "echo depois-do-resize\n" })),
    ));
    assert!(escrito.response().error.is_none());
    let evento = render_com(&receptor, "depois-do-resize").expect("render apos resize");
    let params = evento.params.as_ref().unwrap();

    assert_eq!(params["cols"].as_u64(), Some(100));
    assert_eq!(params["rows"].as_u64(), Some(30));

    // Rolar sem historico e' aceito e clampado pelo emulador, nao um erro.
    let rolado = core.handle_request(&JsonRpcRequest::new(
        918_i64,
        "terminal.scroll",
        Some(json!({ "id": id, "offset": 5 })),
    ));
    assert!(rolado.response().error.is_none());
}
