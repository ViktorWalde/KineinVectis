//! A instancia de Grafana (`grafana.*`) pelo despacho JSON-RPC.
//!
//! O que estes testes provam nao e' "o CRUD funciona" — e' que a mesma decisao
//! que guarda o perfil de banco sem senha vale aqui para o TOKEN, e que ela
//! sobrevive ao caminho REAL, o que a UI de fato chama. Um perfil que nasce
//! limpo no dominio e aceita um token pelo handler seria a mesma classe de
//! defeito que a regra existe para impedir.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

/// Workspace vazio com um `Cargo.toml`, o bastante para `workspace.open`.
fn workspace(nome: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-grafana-{nome}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    dir
}

fn abrir(core: &mut crate::Core, dir: &std::path::Path) {
    let aberto = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());
}

#[test]
fn grafana_salva_le_e_esquece_pelo_despacho() {
    let dir = workspace("crud");
    let mut core = core_with_empty_search_path("grafana-crud");

    // Sem workspace aberto nao ha' onde guardar a instancia.
    let sem_workspace = core.handle_request(&JsonRpcRequest::new(10_i64, "grafana.get", None));
    assert_eq!(
        sem_workspace.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    abrir(&mut core, &dir);

    // AUSENCIA E' UM ESTADO, e ela responde com sucesso e sem perfil — nao com
    // erro. A tela desenha um convite; um erro faria a UI mostrar falha para
    // quem simplesmente ainda nao configurou nada.
    let vazio = core.handle_request(&JsonRpcRequest::new(11_i64, "grafana.get", None));
    let resposta = vazio.response();
    assert!(resposta.error.is_none());
    assert!(resposta.result.as_ref().unwrap().get("profile").is_none());

    let salvo = core.handle_request(&JsonRpcRequest::new(
        12_i64,
        "grafana.save",
        Some(json!({ "profile": {
            "url": "http://localhost:3000/",
            "tokenSource": "environment",
            "tokenVariable": "  GRAFANA_TOKEN  "
        }})),
    ));
    let resultado = salvo.response().result.as_ref().unwrap().clone();
    // A barra final some e a variavel e' aparada: a UI manda o que o autor
    // digitou, e normalizar no core evita `//api/health` na primeira sonda.
    assert_eq!(resultado["profile"]["url"], "http://localhost:3000");
    assert_eq!(resultado["profile"]["tokenVariable"], "GRAFANA_TOKEN");

    let lido = core.handle_request(&JsonRpcRequest::new(13_i64, "grafana.get", None));
    assert_eq!(
        lido.response().result.as_ref().unwrap()["profile"]["url"],
        "http://localhost:3000"
    );

    let esquecido = core.handle_request(&JsonRpcRequest::new(14_i64, "grafana.forget", None));
    assert!(
        esquecido
            .response()
            .result
            .as_ref()
            .unwrap()
            .get("profile")
            .is_none()
    );
}

/// O TOKEN NAO PODE ENTRAR PELO HANDLER.
///
/// `deny_unknown_fields` no perfil e' o que faz esta recusa acontecer, e sem o
/// teste ninguem notaria se alguem o removesse: a chamada continuaria
/// respondendo sucesso, e o campo sumiria em silencio — o autor acreditaria ter
/// guardado uma credencial que nunca foi guardada.
#[test]
fn perfil_com_token_e_recusado() {
    let dir = workspace("token");
    let mut core = core_with_empty_search_path("grafana-token");
    abrir(&mut core, &dir);

    let resposta = core.handle_request(&JsonRpcRequest::new(
        20_i64,
        "grafana.save",
        Some(json!({ "profile": {
            "url": "http://localhost:3000",
            "tokenSource": "none",
            "token": "glsa_segredo"
        }})),
    ));
    let erro = resposta.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
}

/// Endereco sem esquema e' recusado APONTANDO O CONSERTO, e nada e' gravado.
#[test]
fn endereco_invalido_nao_grava_nada() {
    let dir = workspace("invalido");
    let mut core = core_with_empty_search_path("grafana-invalido");
    abrir(&mut core, &dir);

    let resposta = core.handle_request(&JsonRpcRequest::new(
        30_i64,
        "grafana.save",
        Some(json!({ "profile": { "url": "localhost:3000" }})),
    ));
    let erro = resposta.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(
        erro.message.contains("http://"),
        "mensagem: {}",
        erro.message
    );

    let lido = core.handle_request(&JsonRpcRequest::new(31_i64, "grafana.get", None));
    assert!(
        lido.response()
            .result
            .as_ref()
            .unwrap()
            .get("profile")
            .is_none()
    );
}

/// Sondar sem instancia configurada recusa com uma frase, nao com um panic.
#[test]
fn sondar_sem_instancia_recusa() {
    let dir = workspace("sem-instancia");
    let mut core = core_with_empty_search_path("grafana-sem-instancia");
    abrir(&mut core, &dir);

    let resposta = core.handle_request(&JsonRpcRequest::new(40_i64, "grafana.probe", None));
    let erro = resposta.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(erro.message.contains("nenhum Grafana"));
}

/// Politica `prompt` sem token recusa com `SECRET_REQUIRED`, e e' por ESSE
/// codigo que a UI abre o campo — nunca lendo o texto da mensagem.
#[test]
fn politica_prompt_sem_token_pede_o_token_por_codigo() {
    let dir = workspace("prompt");
    let mut core = core_with_empty_search_path("grafana-prompt");
    abrir(&mut core, &dir);

    let _ = core.handle_request(&JsonRpcRequest::new(
        50_i64,
        "grafana.save",
        Some(json!({ "profile": { "url": "http://localhost:3000", "tokenSource": "prompt" }})),
    ));

    let resposta = core.handle_request(&JsonRpcRequest::new(51_i64, "grafana.probe", None));
    assert_eq!(
        resposta.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::SecretRequired
    );
}

/// Politica `environment` com a variavel ausente tambem pede — e diz QUAL
/// variavel esta' faltando, senao o autor fica procurando qual das dele e'.
#[test]
fn variavel_ausente_nomeia_a_variavel() {
    let dir = workspace("variavel");
    let mut core = core_with_empty_search_path("grafana-variavel");
    abrir(&mut core, &dir);

    let _ = core.handle_request(&JsonRpcRequest::new(
        60_i64,
        "grafana.save",
        Some(json!({ "profile": {
            "url": "http://localhost:3000",
            "tokenSource": "environment",
            "tokenVariable": "KINEIN_TESTE_TOKEN_QUE_NAO_EXISTE"
        }})),
    ));

    let resposta = core.handle_request(&JsonRpcRequest::new(61_i64, "grafana.probe", None));
    let erro = resposta.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::SecretRequired);
    assert!(erro.message.contains("KINEIN_TESTE_TOKEN_QUE_NAO_EXISTE"));
}

/// O DISCO NAO PODE PARECER CREDENCIAL. Complementa o teste do `store.rs`
/// entrando pelo caminho real: o que a UI chama, gravando o que ela manda.
#[test]
fn nada_gravado_pelo_handler_parece_credencial() {
    let dir = workspace("disco");
    let mut core = core_with_empty_search_path("grafana-disco");
    abrir(&mut core, &dir);

    let _ = core.handle_request(&JsonRpcRequest::new(
        70_i64,
        "grafana.save",
        Some(json!({ "profile": { "url": "http://localhost:3000", "tokenSource": "prompt" }})),
    ));

    let gravado = std::fs::read_to_string(dir.join(".kinein").join("grafana.json")).unwrap();
    let baixo = gravado.to_lowercase();
    for proibido in ["\"token\"", "\"apikey\"", "\"password\"", "\"secret\""] {
        assert!(
            !baixo.contains(proibido),
            "o disco ganhou {proibido}: {gravado}"
        );
    }
}
