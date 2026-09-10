//! Simulacoes salvas em `.kinein/simulacoes/`.
//!
//! O que estes testes protegem nao e' "o JSON grava" — sao duas decisoes:
//!
//! 1. **O RESULTADO nao entra no arquivo.** E' a licao do `.ipynb`, que e' texto
//!    e mesmo assim falha por misturar o autoral com a saida da maquina. Se
//!    alguem acrescentar a trilha ao `SimSaved`, o teste `o_arquivo_guarda_o_que_o
//!    _autor_montou_e_nada_do_que_a_maquina_produziu` cai.
//! 2. **O nome do autor nao vira caminho.** Um nome com `../` escreveria fora do
//!    workspace.

use std::path::{Path, PathBuf};

use kinein_protocol::{JsonRpcRequest, SimBinding, SimInitial, SimMethod, SimSaved, SimValue};
use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::Core;
use crate::sim::persistencia;

fn workspace(nome: &str) -> PathBuf {
    let base = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-sim-persist-{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("cria workspace");
    base
}

fn abrir(core: &mut Core, raiz: &Path) {
    let saida = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().expect("utf-8") })),
    ));
    assert!(saida.response().error.is_none(), "workspace.open falhou");
}

fn chamar(core: &mut Core, metodo: &str, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(2_i64, metodo, Some(params.clone())));
    let resposta = saida.response();
    assert!(resposta.error.is_none(), "{metodo}: {:?}", resposta.error);
    resposta.result.clone().expect("resultado")
}

fn montagem(nome: &str) -> SimSaved {
    SimSaved {
        name: nome.to_string(),
        concept: "oscilador-amortecido".to_string(),
        formula: "(-k*y - c*dy) / m".to_string(),
        bindings: vec![
            SimBinding {
                variable: "k".into(),
                quantity: "k".into(),
            },
            SimBinding {
                variable: "y".into(),
                quantity: "y".into(),
            },
            SimBinding {
                variable: "m".into(),
                quantity: "m".into(),
            },
        ],
        values: vec![
            SimValue {
                quantity: "m".into(),
                value: 2.0,
            },
            SimValue {
                quantity: "k".into(),
                value: 10.0,
            },
        ],
        initial: Some(SimInitial {
            y: 1.0,
            dy: Some(0.0),
        }),
        method: Some(SimMethod::Rk4),
        step: Some(1e-3),
        duration: Some(10.0),
        samples: Some(50),
    }
}

#[test]
fn salva_lista_e_esquece_pelo_ipc() {
    let raiz = workspace("ciclo");
    let mut core = core_with_empty_search_path("sim-persist-ciclo");
    abrir(&mut core, &raiz);

    assert_eq!(
        chamar(&mut core, "sim.list", &json!({}))["simulations"]
            .as_array()
            .expect("lista")
            .len(),
        0
    );

    let salvo = chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("Meu oscilador") }),
    );
    let lista = salvo["simulations"].as_array().expect("lista");
    assert_eq!(lista.len(), 1);
    // O nome ORIGINAL sobrevive: o que muda e' so' o caminho no disco.
    assert_eq!(lista[0]["name"], json!("Meu oscilador"));
    assert_eq!(lista[0]["method"], json!("rk4"));

    let depois = chamar(&mut core, "sim.forget", &json!({ "name": "Meu oscilador" }));
    assert_eq!(depois["simulations"].as_array().expect("lista").len(), 0);
}

#[test]
fn uma_simulacao_por_arquivo_e_nao_um_catalogo() {
    // Um arquivo unico poria dois trabalhos sem relacao no mesmo diff, e todo
    // conflito de merge seria entre simulacoes que nao se conhecem.
    let raiz = workspace("por-arquivo");
    let mut core = core_with_empty_search_path("sim-persist-arquivo");
    abrir(&mut core, &raiz);
    chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("primeira") }),
    );
    chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("segunda") }),
    );

    let arquivos: Vec<_> = std::fs::read_dir(persistencia::pasta(&raiz))
        .expect("pasta existe")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(arquivos.len(), 2, "{arquivos:?}");
    assert!(
        arquivos.contains(&"primeira.json".to_string()),
        "{arquivos:?}"
    );
    assert!(
        arquivos.contains(&"segunda.json".to_string()),
        "{arquivos:?}"
    );
}

#[test]
fn o_arquivo_guarda_o_que_o_autor_montou_e_nada_do_que_a_maquina_produziu() {
    // A DECISAO MAIS IMPORTANTE DO FORMATO (arquitetura/34 §9). E' a licao do
    // `.ipynb`: ele e' texto, e' diffavel, e mesmo assim falha porque mistura o
    // que o usuario escreveu com o que a maquina produziu.
    let raiz = workspace("sem-resultado");
    let mut core = core_with_empty_search_path("sim-persist-sem-resultado");
    abrir(&mut core, &raiz);
    chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("oscilador") }),
    );

    let corpo = std::fs::read_to_string(persistencia::pasta(&raiz).join("oscilador.json"))
        .expect("le o arquivo");
    // O que o autor montou ESTA la.
    assert!(corpo.contains("\"formula\""), "{corpo}");
    assert!(corpo.contains("\"bindings\""), "{corpo}");
    assert!(corpo.contains("\"schemaVersion\""), "{corpo}");
    // O que a maquina produziu NAO esta.
    for proibido in [
        "trail",
        "accuracy",
        "stepsTaken",
        "numeric",
        "absoluteError",
    ] {
        assert!(
            !corpo.contains(proibido),
            "o arquivo guardou `{proibido}`, que e saida da maquina:\n{corpo}"
        );
    }
}

#[test]
fn nome_do_autor_nao_vira_caminho() {
    // Sem isto, `../../etc/algo` escreveria fora do workspace e `a/b` criaria
    // diretorio. O nome original fica DENTRO do arquivo, entao nada se perde.
    assert_eq!(persistencia::nome_de_arquivo("../../escapar"), "escapar");
    assert_eq!(
        persistencia::nome_de_arquivo("pasta/arquivo"),
        "pasta-arquivo"
    );
    assert_eq!(
        persistencia::nome_de_arquivo("Meu Oscilador!"),
        "meu-oscilador"
    );
    assert_eq!(persistencia::nome_de_arquivo("   "), "simulacao");
    assert_eq!(persistencia::nome_de_arquivo("..."), "simulacao");

    let raiz = workspace("escape");
    let mut core = core_with_empty_search_path("sim-persist-escape");
    abrir(&mut core, &raiz);
    chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("../../../fora") }),
    );
    // O arquivo caiu DENTRO da pasta, e o nome original sobreviveu no conteudo.
    let dentro = persistencia::pasta(&raiz).join("fora.json");
    assert!(dentro.exists(), "escapou da pasta");
    let salvo = persistencia::listar(&raiz);
    assert_eq!(salvo.len(), 1);
    assert_eq!(salvo[0].name, "../../../fora");
}

#[test]
fn arquivo_invalido_e_pulado_e_nao_esconde_os_outros() {
    // Arquivo corrompido nao pode derrubar a lista: quem abre a IDE com um
    // `.kinein/` estragado perde a simulacao, nao a sessao. Mesma regra do
    // `runconfigs.json`.
    let raiz = workspace("invalido");
    let mut core = core_with_empty_search_path("sim-persist-invalido");
    abrir(&mut core, &raiz);
    chamar(
        &mut core,
        "sim.save",
        &json!({ "simulation": montagem("boa") }),
    );
    std::fs::write(
        persistencia::pasta(&raiz).join("quebrada.json"),
        "{ nao e json",
    )
    .expect("escreve lixo");
    std::fs::write(
        persistencia::pasta(&raiz).join("futura.json"),
        "{\"schemaVersion\": 99, \"name\": \"do futuro\"}",
    )
    .expect("escreve schema desconhecido");

    let lista = persistencia::listar(&raiz);
    assert_eq!(lista.len(), 1, "{lista:?}");
    assert_eq!(lista[0].name, "boa");
}

#[test]
fn nome_vazio_e_recusado() {
    let raiz = workspace("vazio");
    let mut core = core_with_empty_search_path("sim-persist-vazio");
    abrir(&mut core, &raiz);
    let mut m = montagem("");
    m.name = "   ".to_string();
    let saida = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "sim.save",
        Some(json!({ "simulation": m })),
    ));
    assert!(
        saida.response().error.is_some(),
        "deveria recusar nome vazio"
    );
}

#[test]
fn sem_workspace_aberto_a_persistencia_recusa_em_vez_de_escrever_em_qualquer_lugar() {
    let mut core = core_with_empty_search_path("sim-persist-sem-ws");
    for metodo in ["sim.list", "sim.save", "sim.forget"] {
        let saida = core.handle_request(&JsonRpcRequest::new(4_i64, metodo, Some(json!({}))));
        assert!(
            saida.response().error.is_some(),
            "{metodo} deveria exigir workspace"
        );
    }
}
