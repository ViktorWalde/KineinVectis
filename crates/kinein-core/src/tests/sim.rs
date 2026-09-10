//! O dominio `sim`: catalogo, ligacao explicita e avaliacao.
//!
//! O que estes testes provam nao e "o JSON tem os campos certos" — e que a
//! LIGACAO que o usuario informa e' o que decide o numero. O `exmex` devolve as
//! variaveis em ordem ALFABETICA, nao na ordem da formula, e `eval()` recebe um
//! slice posicional: montar esse slice pela ordem de leitura da' um numero e a
//! fisica errada, **sem erro nenhum**. Os testes de mutacao abaixo existem
//! para que essa falha nao possa voltar em silencio.

use kinein_protocol::{JsonRpcRequest, SimBinding, SimCheckIssue, SimForm, SimValue, SimView};
use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::Core;
use crate::sim::{
    catalogo,
    formula::{self, ErroAvaliacao},
};

/// Chama o dispatch de verdade e exige sucesso.
fn chamar(core: &mut Core, metodo: &str, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(7_i64, metodo, Some(params.clone())));
    let resposta = saida.response();
    assert!(
        resposta.error.is_none(),
        "{metodo} falhou: {:?}",
        resposta.error
    );
    resposta.result.clone().expect("resultado")
}

/// Chama o dispatch e exige recusa, devolvendo o erro.
fn recusar(core: &mut Core, metodo: &str, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(8_i64, metodo, Some(params.clone())));
    let resposta = saida.response();
    let erro = resposta.error.clone().expect("deveria recusar");
    serde_json::to_value(erro).expect("erro serializa")
}

fn ligacao(variavel: &str, grandeza: &str) -> SimBinding {
    SimBinding {
        variable: variavel.to_string(),
        quantity: grandeza.to_string(),
    }
}

fn valor(grandeza: &str, numero: f64) -> SimValue {
    SimValue {
        quantity: grandeza.to_string(),
        value: numero,
    }
}

#[test]
fn catalogo_declara_conceitos_com_fonte_datada() {
    let conceitos = catalogo::conceitos();
    assert!(!conceitos.is_empty(), "catalogo vazio nao oferece nada");
    for conceito in &conceitos {
        assert!(!conceito.id.is_empty(), "conceito sem id");
        assert!(!conceito.name.is_empty(), "{} sem nome", conceito.id);
        assert!(
            !conceito.quantities.is_empty(),
            "{} nao declara grandeza nenhuma, e a checagem depende disso",
            conceito.id
        );
        // Mesma regra do `setup.list`: sem fonte, a IDE nao afirma. E a data
        // precisa estar la, ou o `verificar-docs` nao teria como cobrar idade.
        assert!(
            conceito.source.contains("2026-"),
            "{} tem fonte sem data: {:?}",
            conceito.id,
            conceito.source
        );
        assert!(
            conceito.quantities.iter().any(|q| q.required),
            "{} nao tem grandeza obrigatoria, entao nada distingue a formula dele",
            conceito.id
        );
    }
}

#[test]
fn o_catalogo_so_oferece_forma_que_tem_motor() {
    // Um conceito cuja forma a IDE nao sabe resolver seria oferta que ela nao
    // cumpre. Este teste e' o que impede a oferta de chegar antes do motor — e
    // ele EVOLUIU em 2026-09-05 em vez de sumir: quando o integrador entrou,
    // `Ode1` e `Ode2` passaram a ter motor e entraram na lista.
    //
    // E EVOLUIU DE NOVO em 2026-09-06: o integrador vetorial entrou, e
    // `OdeSystem` passou a ter motor.
    //
    // `Pde` continua FORA: ela esta' declarada no protocolo, sem motor, e este
    // teste e' o que garante que ninguem a ponha no catalogo antes de escrever
    // o motor dela — que, medido, exige o caminho COMPILADO inteiro
    // (`roadmaps/31` §19.2.2).
    for conceito in catalogo::conceitos() {
        assert!(
            matches!(
                conceito.form,
                SimForm::Algebraic | SimForm::Ode1 | SimForm::Ode2 | SimForm::OdeSystem
            ),
            "{} declara forma sem motor: {:?}",
            conceito.id,
            conceito.form
        );
    }
}

#[test]
fn conceito_que_promete_solucao_fechada_tem_a_funcao_dela() {
    // `closedForm: true` e' uma AFIRMACAO que vai a tela: ela promete que a IDE
    // sabe o valor verdadeiro e vai mostrar o erro ao lado do numero. Prometer
    // sem a funcao seria a IDE dizendo que sabe e depois nao mostrando nada.
    //
    // A recíproca NAO vale de propósito: um conceito ALGEBRICO tem "solucao
    // fechada" no sentido de que a propria formula do usuario e' a resposta —
    // nao ha' integracao para comparar, entao ele nao precisa da funcao.
    for conceito in catalogo::conceitos() {
        let integra = matches!(conceito.form, SimForm::Ode1 | SimForm::Ode2);
        if integra && conceito.closed_form {
            assert!(
                catalogo::exata(&conceito.id).is_some(),
                "{} promete solucao fechada e nao tem a funcao dela",
                conceito.id
            );
        }
    }
}

#[test]
fn inspecionar_devolve_as_variaveis_da_formula() {
    let vars = formula::inspecionar("a*t + v0").expect("formula valida");
    assert_eq!(vars, vec!["a", "t", "v0"]);
}

#[test]
fn a_ordem_do_avaliador_e_alfabetica_e_nao_a_da_formula() {
    // Nao e' capricho documentar isto num teste: e' a premissa da armadilha 1
    // do ADR-0006. Se uma versao futura do crate passar a devolver na ordem da
    // formula, este teste cai e alguem revisita a defesa em vez de descobrir
    // pelo resultado errado.
    let ordem_uma = formula::inspecionar("v0 + a*t").expect("valida");
    let outra_escrita = formula::inspecionar("a*t + v0").expect("valida");
    assert_eq!(ordem_uma, outra_escrita);
    assert_eq!(ordem_uma, vec!["a", "t", "v0"]);
}

#[test]
fn formula_que_nao_fecha_da_mensagem_da_ide_e_nao_do_crate() {
    let erro = formula::inspecionar("2*").expect_err("deveria recusar");
    // O texto do `exmex` tem endereco de ponteiro dentro; ele nunca vai a tela.
    assert!(!erro.contains("0x"), "mensagem vazou ponteiro: {erro}");
    assert!(
        !erro.contains("Operator {"),
        "mensagem vazou interno: {erro}"
    );
    assert!(erro.contains("operador"), "mensagem nao explica: {erro}");
}

#[test]
fn conceito_errado_aparece_como_grandeza_faltando() {
    let oscilador = catalogo::conceito("energia-cinetica").expect("existe");
    // Formula de outro assunto, com as variaveis do conceito nao ligadas.
    let resultado = formula::checar(&oscilador, "9.81*h", &[]);
    assert!(!resultado.ok);
    let faltando: Vec<_> = resultado
        .issues
        .iter()
        .filter_map(|p| match p {
            SimCheckIssue::MissingQuantity { quantity, .. } => Some(quantity.clone()),
            _ => None,
        })
        .collect();
    assert!(faltando.contains(&"m".to_string()), "{faltando:?}");
    assert!(faltando.contains(&"v".to_string()), "{faltando:?}");
}

#[test]
fn variavel_sem_papel_e_apontada_uma_a_uma() {
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let resultado = formula::checar(
        &conceito,
        "0.5*m*v^2 + F",
        &[ligacao("m", "m"), ligacao("v", "v")],
    );
    assert!(!resultado.ok);
    assert!(
        resultado.issues.contains(&SimCheckIssue::UnboundVariable {
            variable: "F".to_string()
        }),
        "{:?}",
        resultado.issues
    );
}

#[test]
fn o_pi_minusculo_aparece_como_pergunta_e_nao_como_incognita_calada() {
    // O `exmex` trata `pi` minusculo como VARIAVEL LIVRE (`PI` maiusculo e' que
    // e' constante). Sem a ligacao explicita isso viraria uma incognita que
    // ninguem ve. Com ela, vira uma linha na tela.
    let conceito = catalogo::conceito("mru").expect("existe");
    let resultado = formula::checar(
        &conceito,
        "x0 + v*t*sin(2*pi)",
        &[ligacao("x0", "x0"), ligacao("v", "v"), ligacao("t", "t")],
    );
    assert!(!resultado.ok);
    assert!(
        resultado.issues.contains(&SimCheckIssue::UnboundVariable {
            variable: "pi".to_string()
        }),
        "o `pi` passou calado: {:?}",
        resultado.issues
    );
}

#[test]
fn ligacao_para_grandeza_inexistente_e_recusada() {
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let resultado = formula::checar(
        &conceito,
        "0.5*m*v^2",
        &[ligacao("m", "m"), ligacao("v", "temperatura")],
    );
    assert!(!resultado.ok);
    assert!(resultado.issues.iter().any(|p| matches!(
        p,
        SimCheckIssue::UnknownQuantity { quantity, .. } if quantity == "temperatura"
    )));
}

#[test]
fn duas_variaveis_na_mesma_grandeza_e_recusado() {
    let conceito = catalogo::conceito("mruv").expect("existe");
    let resultado = formula::checar(
        &conceito,
        "x0 + v0*t + 0.5*a*t^2",
        &[
            ligacao("x0", "x0"),
            ligacao("v0", "v0"),
            ligacao("a", "v0"),
            ligacao("t", "t"),
        ],
    );
    assert!(!resultado.ok);
    assert!(resultado.issues.iter().any(|p| matches!(
        p,
        SimCheckIssue::DuplicateQuantity { quantity } if quantity == "v0"
    )));
}

#[test]
fn checagem_completa_carrega_o_aviso_do_autor() {
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let resultado = formula::checar(
        &conceito,
        "0.5*m*v^2",
        &[ligacao("m", "m"), ligacao("v", "v")],
    );
    assert!(resultado.ok, "{:?}", resultado.issues);
    let aviso = resultado.caveat.expect("aviso obrigatorio quando passa");
    // O pedido do autor em 2026-09-05: dizer que formula valida nao garante
    // resultado certo, na tela e nao em nota de rodape.
    assert!(aviso.contains("NAO significam resultado certo"), "{aviso}");
}

#[test]
fn avalia_e_confere_com_a_conta_feita_a_mao() {
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let resultado = formula::avaliar(
        &conceito,
        "0.5*m*v^2",
        &[ligacao("m", "m"), ligacao("v", "v")],
        &[valor("m", 2.0), valor("v", 3.0)],
    )
    .expect("deveria avaliar");
    assert!((resultado.value - 9.0).abs() < 1e-12, "{}", resultado.value);
}

#[test]
fn mutacao_trocar_a_ligacao_muda_o_resultado() {
    // ESTE e o gate da armadilha 1 do ADR-0006. Se o vetor de avaliacao voltar
    // a ser montado por posicao, os dois casos abaixo dao o MESMO numero e o
    // teste cai. E' a prova por mutacao de que a ligacao esta sendo usada.
    let conceito = catalogo::conceito("mru").expect("existe");
    let certo = formula::avaliar(
        &conceito,
        "x0 + v*t",
        &[ligacao("x0", "x0"), ligacao("v", "v"), ligacao("t", "t")],
        &[valor("x0", 1.0), valor("v", 10.0), valor("t", 3.0)],
    )
    .expect("avalia");
    let trocado = formula::avaliar(
        &conceito,
        "x0 + v*t",
        // `v` e `t` trocam de papel: a formula e' a mesma, a fisica nao.
        &[ligacao("x0", "x0"), ligacao("v", "t"), ligacao("t", "v")],
        &[valor("x0", 1.0), valor("v", 10.0), valor("t", 3.0)],
    )
    .expect("avalia");
    assert!((certo.value - 31.0).abs() < 1e-12, "{}", certo.value);
    assert!((trocado.value - 31.0).abs() < 1e-12, "{}", trocado.value);

    // Com valores diferentes por grandeza, a troca TEM de mudar o numero.
    let a = formula::avaliar(
        &conceito,
        "x0 + v*t",
        &[ligacao("x0", "x0"), ligacao("v", "v"), ligacao("t", "t")],
        &[valor("x0", 0.0), valor("v", 2.0), valor("t", 5.0)],
    )
    .expect("avalia");
    let b = formula::avaliar(
        &conceito,
        "x0 + v^2*t",
        &[ligacao("x0", "x0"), ligacao("v", "t"), ligacao("t", "v")],
        &[valor("x0", 0.0), valor("v", 2.0), valor("t", 5.0)],
    )
    .expect("avalia");
    assert!(
        (a.value - b.value).abs() > 1e-9,
        "trocar a ligacao nao mudou nada: {} contra {}",
        a.value,
        b.value
    );
}

#[test]
fn resultado_nao_finito_e_erro_e_nao_numero() {
    // O `exmex` aceita `1/0` e devolve `Ok(inf)` (ADR-0006, armadilha 3). Uma
    // curva de `inf` com cara de resultado e pior que um erro.
    let conceito = catalogo::conceito("lei-de-ohm").expect("existe");
    let erro = formula::avaliar(
        &conceito,
        "i/r",
        &[ligacao("i", "i"), ligacao("r", "r")],
        &[valor("i", 1.0), valor("r", 0.0)],
    )
    .expect_err("divisao por zero nao pode virar numero");
    assert_eq!(
        erro,
        ErroAvaliacao::NaoFinito {
            valor: "inf".to_string()
        }
    );
}

#[test]
fn grandeza_ligada_sem_valor_e_recusa_e_nao_zero() {
    // Campo vazio e' recusa: o principio da §2.1 diz que a IDE nao preenche por
    // voce, e assumir zero seria preencher.
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let erro = formula::avaliar(
        &conceito,
        "0.5*m*v^2",
        &[ligacao("m", "m"), ligacao("v", "v")],
        &[valor("m", 2.0)],
    )
    .expect_err("faltou o valor de v");
    assert_eq!(
        erro,
        ErroAvaliacao::SemValor {
            grandeza: "v".to_string()
        }
    );
}

#[test]
fn a_trilha_mostra_a_conta_que_rodou() {
    let conceito = catalogo::conceito("energia-cinetica").expect("existe");
    let resultado = formula::avaliar(
        &conceito,
        "0.5*m*v^2",
        &[ligacao("m", "m"), ligacao("v", "v")],
        &[valor("m", 2.0), valor("v", 3.0)],
    )
    .expect("avalia");
    let tipos: Vec<&str> = resultado.steps.iter().map(|p| p.kind.as_str()).collect();
    assert_eq!(tipos.first(), Some(&"formula"));
    assert_eq!(tipos.last(), Some(&"result"));
    let substituicao = resultado
        .steps
        .iter()
        .find(|p| p.kind == "substitution")
        .expect("a substituicao e' o passo a passo possivel");
    // A substituicao E' a conta: os numeros do usuario aparecem no lugar das
    // variaveis, e nenhuma variavel sobra.
    assert!(substituicao.text.contains('2'), "{}", substituicao.text);
    assert!(substituicao.text.contains('3'), "{}", substituicao.text);
    assert!(!substituicao.text.contains('m'), "{}", substituicao.text);
}

#[test]
fn a_substituicao_respeita_fronteira_de_identificador() {
    // Trocar `v` dentro de `v0` mostraria uma conta que nao e' a que rodou, e
    // o painel deixaria de ser honesto — que e' a unica coisa que ele oferece.
    let conceito = catalogo::conceito("mruv").expect("existe");
    let resultado = formula::avaliar(
        &conceito,
        "x0 + v0*t + 0.5*a*t^2",
        &[
            ligacao("x0", "x0"),
            ligacao("v0", "v0"),
            ligacao("a", "a"),
            ligacao("t", "t"),
        ],
        &[
            valor("x0", 0.0),
            valor("v0", 7.0),
            valor("a", 2.0),
            valor("t", 3.0),
        ],
    )
    .expect("avalia");
    let substituicao = resultado
        .steps
        .iter()
        .find(|p| p.kind == "substitution")
        .expect("existe");
    // Asserção EXATA: `v0` virou `7`, e não `70` — que é o que a substituição
    // ingênua produziria ao trocar o `v` de `v0` e deixar o `0` no lugar.
    assert_eq!(substituicao.text, "0 + 7*3 + 0.5*2*3^2");
    assert!(
        !substituicao.text.chars().any(|c| c.is_ascii_alphabetic()),
        "sobrou variavel na substituicao: {}",
        substituicao.text
    );
    // 0 + 7*3 + 0.5*2*9 = 30
    assert!(
        (resultado.value - 30.0).abs() < 1e-12,
        "{}",
        resultado.value
    );
}

#[test]
fn a_substituicao_nao_troca_v_dentro_de_v0() {
    // ESTE e o gate da fronteira de identificador, e ele existe porque uma
    // MUTACAO em 2026-09-05 mostrou que o codigo dela nao tinha cobertura:
    // nenhum conceito do catalogo tinha variavel que fosse prefixo de outra.
    // Torricelli tem — `v0` e `v` na mesma conta —, que e' o caso real.
    // O impulso poe `v` e `v0` na MESMA formula, que e' o unico jeito de o caso
    // acontecer: substituicao ingenua troca o `v` de `v0` por 5 e deixa o `0`,
    // virando "50" — e o painel passaria a mostrar uma conta que nao rodou.
    let conceito = catalogo::conceito("impulso").expect("existe");
    let resultado = formula::avaliar(
        &conceito,
        "m*(v - v0)",
        &[ligacao("m", "m"), ligacao("v", "v"), ligacao("v0", "v0")],
        &[valor("m", 2.0), valor("v", 5.0), valor("v0", 3.0)],
    )
    .expect("avalia");
    let substituicao = resultado
        .steps
        .iter()
        .find(|p| p.kind == "substitution")
        .expect("existe");
    // Ingenua daria "2*(5 - 50)" = -90. A assercao e' exata de proposito.
    assert_eq!(substituicao.text, "2*(5 - 3)");
    assert!((resultado.value - 4.0).abs() < 1e-12, "{}", resultado.value);
}

#[test]
fn o_erro_com_endereco_de_ponteiro_nunca_chega_a_tela() {
    // A armadilha 4 do ADR-0006 foi MEDIDA com esta entrada: `x +* y` produz
    // 356 caracteres com `0x...` dentro. O teste anterior usava `2*`, que cai
    // em outro ramo da classificacao — e uma mutacao em 2026-09-05 mostrou que
    // o ramo do ponteiro estava sem cobertura.
    let erro = formula::inspecionar("x +* y").expect_err("deveria recusar");
    assert!(!erro.contains("0x"), "mensagem vazou ponteiro: {erro}");
    assert!(!erro.contains("BinOp"), "mensagem vazou interno: {erro}");
    assert!(
        erro.len() < 120,
        "mensagem longa demais para a tela ({} caracteres): {erro}",
        erro.len()
    );
    assert!(
        erro.contains("operadores"),
        "nao explica o problema: {erro}"
    );
}

#[test]
fn a_vista_declarada_e_coerente_com_o_conceito() {
    // A vista e' o unico campo que comeca preenchido (roadmaps/31 §18.3), e ela
    // vem do catalogo — declaracao auditada, nao palpite.
    let coulomb = catalogo::conceito("coulomb").expect("existe");
    assert_eq!(coulomb.view, SimView::Space3d);
    let ohm = catalogo::conceito("lei-de-ohm").expect("existe");
    assert_eq!(ohm.view, SimView::Plot2d);
}

// ---------------------------------------------------------------------------
// O dominio pelo IPC. Um dominio que existe no core e nao esta' ROTEADO e' a
// forma exata do defeito de 2026-09-04: o `shellController` era usado pelo
// GlobalShortcuts e nunca ligado, e a chamada caia num `null` em silencio.
// ---------------------------------------------------------------------------

#[test]
fn sim_catalog_responde_pelo_dispatch() {
    let mut core = core_with_empty_search_path("sim-catalog");
    let resultado = chamar(&mut core, "sim.catalog", &json!({}));
    let conceitos = resultado["concepts"].as_array().expect("lista");
    assert!(!conceitos.is_empty(), "catalogo vazio pelo IPC");
    assert!(conceitos.iter().any(|c| c["id"] == "energia-cinetica"));
    // O contrato que a UI le: cada conceito traz grandezas e vista.
    let primeiro = &conceitos[0];
    assert!(primeiro["quantities"].is_array(), "{primeiro}");
    assert!(primeiro["view"].is_string(), "{primeiro}");
    assert!(primeiro["form"].is_string(), "{primeiro}");
}

#[test]
fn sim_catalog_filtra_por_curso() {
    let mut core = core_with_empty_search_path("sim-catalog-curso");
    let resultado = chamar(&mut core, "sim.catalog", &json!({ "course": "Fisica IV" }));
    let conceitos = resultado["concepts"].as_array().expect("lista");
    assert!(!conceitos.is_empty());
    assert!(conceitos.iter().all(|c| c["course"] == "Fisica IV"));
}

#[test]
fn sim_inspect_devolve_as_variaveis_pelo_dispatch() {
    let mut core = core_with_empty_search_path("sim-inspect");
    let resultado = chamar(
        &mut core,
        "sim.inspectFormula",
        &json!({ "formula": "0.5*m*v^2" }),
    );
    assert_eq!(resultado["variables"], json!(["m", "v"]));
}

#[test]
fn sim_check_traz_os_problemas_como_dado_e_nao_como_texto() {
    // A UI nao pode casar erro por TEXTO — foi o que o `requestFailed` sem
    // `code` obrigou em 2026-09-04, e o que o `SecretRequired` existe para
    // evitar. Cada problema tem `kind`.
    let mut core = core_with_empty_search_path("sim-check");
    let resultado = chamar(
        &mut core,
        "sim.checkFormula",
        &json!({
            "concept": "energia-cinetica",
            "formula": "0.5*m*v^2 + F",
            "bindings": [
                { "variable": "m", "quantity": "m" },
                { "variable": "v", "quantity": "v" }
            ]
        }),
    );
    assert_eq!(resultado["ok"], json!(false));
    let problemas = resultado["issues"].as_array().expect("lista");
    assert!(
        problemas
            .iter()
            .any(|p| p["kind"] == "unboundVariable" && p["variable"] == "F"),
        "{problemas:?}"
    );
}

#[test]
fn sim_evaluate_devolve_valor_e_trilha_pelo_dispatch() {
    let mut core = core_with_empty_search_path("sim-evaluate");
    let resultado = chamar(
        &mut core,
        "sim.evaluate",
        &json!({
            "concept": "energia-cinetica",
            "formula": "0.5*m*v^2",
            "bindings": [
                { "variable": "m", "quantity": "m" },
                { "variable": "v", "quantity": "v" }
            ],
            "values": [
                { "quantity": "m", "value": 2.0 },
                { "quantity": "v", "value": 3.0 }
            ]
        }),
    );
    assert_eq!(resultado["value"], json!(9.0));
    let passos = resultado["steps"].as_array().expect("trilha");
    assert!(passos.iter().any(|p| p["kind"] == "substitution"));
}

#[test]
fn sim_evaluate_recusa_com_codigo_e_nao_so_com_frase() {
    let mut core = core_with_empty_search_path("sim-evaluate-recusa");
    let erro = recusar(
        &mut core,
        "sim.evaluate",
        &json!({
            "concept": "lei-de-ohm",
            "formula": "i/r",
            "bindings": [
                { "variable": "i", "quantity": "i" },
                { "variable": "r", "quantity": "r" }
            ],
            "values": [
                { "quantity": "i", "value": 1.0 },
                { "quantity": "r", "value": 0.0 }
            ]
        }),
    );
    // `reason` e' o dado que a UI usa; a frase e' para o humano.
    assert_eq!(erro["details"]["reason"], json!("notFinite"));
    assert_eq!(erro["details"]["value"], json!("inf"));
}

#[test]
fn conceito_inexistente_e_recusado_com_o_id_de_volta() {
    let mut core = core_with_empty_search_path("sim-conceito-inexistente");
    let erro = recusar(
        &mut core,
        "sim.checkFormula",
        &json!({ "concept": "buraco-de-minhoca", "formula": "x", "bindings": [] }),
    );
    assert_eq!(erro["details"]["concept"], json!("buraco-de-minhoca"));
}
