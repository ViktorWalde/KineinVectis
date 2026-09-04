//! Perfis de conexao a banco (`datasource.*`) pelo despacho JSON-RPC.
//!
//! O que estes testes provam nao e' "o CRUD funciona" — e' que a decisao
//! registrada em `docs/seguranca/40-cofre-de-credencial.md` sobrevive ao
//! caminho REAL, o que a UI de fato chama. Um perfil que nasce limpo no
//! dominio e vaza senha pelo handler seria a mesma classe de defeito que a
//! regra existe para impedir.

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

/// Workspace vazio com um `Cargo.toml`, o bastante para `workspace.open`.
fn workspace(nome: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-datasource-{nome}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    dir
}

fn perfil() -> serde_json::Value {
    json!({
        "name": "local",
        "host": "localhost",
        "port": 5432,
        "database": "app",
        "user": "postgres",
        "secretSource": "environment",
        "secretVariable": "PGPASSWORD"
    })
}

#[test]
fn datasource_crud_flows_through_dispatch() {
    let dir = workspace("crud");
    let mut core = core_with_empty_search_path("datasource-crud");

    // Sem workspace aberto nao ha' onde guardar perfil.
    let sem_workspace = core.handle_request(&JsonRpcRequest::new(200_i64, "datasource.list", None));
    assert_eq!(
        sem_workspace.response().error.as_ref().unwrap().code,
        JsonRpcErrorCode::InvalidRequest
    );

    let aberto = core.handle_request(&JsonRpcRequest::new(
        201_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    assert!(aberto.response().error.is_none());

    let vazio = core.handle_request(&JsonRpcRequest::new(202_i64, "datasource.list", None));
    assert_eq!(
        vazio.response().result.as_ref().unwrap()["profiles"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let salvo = core.handle_request(&JsonRpcRequest::new(
        203_i64,
        "datasource.save",
        Some(json!({ "profile": perfil() })),
    ));
    let resultado = salvo.response().result.as_ref().unwrap().clone();
    assert_eq!(resultado["profiles"][0]["name"], "local");
    assert_eq!(resultado["profiles"][0]["port"], 5432);

    // Salvar de novo com o MESMO nome edita; nao duplica.
    let mut corrigido = perfil();
    corrigido["port"] = json!(6543);
    let editado = core.handle_request(&JsonRpcRequest::new(
        204_i64,
        "datasource.save",
        Some(json!({ "profile": corrigido })),
    ));
    let resultado = editado.response().result.as_ref().unwrap().clone();
    assert_eq!(
        resultado["profiles"].as_array().unwrap().len(),
        1,
        "corrigir a porta duplicou o perfil"
    );
    assert_eq!(resultado["profiles"][0]["port"], 6543);

    let removido = core.handle_request(&JsonRpcRequest::new(
        205_i64,
        "datasource.remove",
        Some(json!({ "name": "local" })),
    ));
    assert_eq!(
        removido.response().result.as_ref().unwrap()["profiles"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    // Remover o que ja sumiu termina no mesmo estado, sem erro.
    let de_novo = core.handle_request(&JsonRpcRequest::new(
        206_i64,
        "datasource.remove",
        Some(json!({ "name": "local" })),
    ));
    assert!(de_novo.response().error.is_none());
}

#[test]
fn perfil_invalido_reprova_com_mensagem_que_diz_o_que_fazer() {
    let dir = workspace("invalido");
    let mut core = core_with_empty_search_path("datasource-invalido");
    let _ = core.handle_request(&JsonRpcRequest::new(
        300_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));

    let mut sem_host = perfil();
    sem_host["host"] = json!("");
    let resposta = core.handle_request(&JsonRpcRequest::new(
        301_i64,
        "datasource.save",
        Some(json!({ "profile": sem_host })),
    ));
    let erro = resposta.response().error.as_ref().unwrap();
    assert_eq!(erro.code, JsonRpcErrorCode::InvalidParams);
    assert!(
        erro.message.contains("localhost"),
        "a mensagem tinha que dizer o que fazer, e disse: {}",
        erro.message
    );
}

/// A garantia da decisao de 2026-09-04, verificada no caminho REAL.
///
/// Um campo `password` enviado pela UI tem que ser RECUSADO pelo protocolo
/// (`deny_unknown_fields`), nao ignorado em silencio — ignorar faria a UI achar
/// que guardou a senha, que e' pior que recusar.
#[test]
fn senha_enviada_pela_ui_e_recusada_e_nunca_chega_ao_disco() {
    let dir = workspace("segredo");
    let mut core = core_with_empty_search_path("datasource-segredo");
    let _ = core.handle_request(&JsonRpcRequest::new(
        400_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));

    let mut com_senha = perfil();
    com_senha["password"] = json!("hunter2");
    let resposta = core.handle_request(&JsonRpcRequest::new(
        401_i64,
        "datasource.save",
        Some(json!({ "profile": com_senha })),
    ));
    assert!(
        resposta.response().error.is_some(),
        "um campo `password` no perfil tinha que ser RECUSADO"
    );

    let arquivo = dir.join(".kinein").join("datasources.json");
    if let Ok(conteudo) = std::fs::read_to_string(&arquivo) {
        assert!(
            !conteudo.contains("hunter2"),
            "a senha chegou ao disco — ver docs/seguranca/40"
        );
    }
}
