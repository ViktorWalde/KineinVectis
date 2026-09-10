//! O integrador, verificado como a engenharia manda: por ORDEM DE CONVERGENCIA.
//!
//! O que estes testes provam nao e "o codigo roda" — e' que **cada metodo tem a
//! ordem que ele diz ter**. A norma que separa as duas perguntas e' a ASME
//! V&V 20 (`roadmaps/31` §11.4):
//!
//! ```text
//! VERIFICACAO  as equacoes foram implementadas corretamente?  <- e' isto aqui
//! VALIDACAO    as equacoes descrevem a realidade?  <- exige experimento, e
//!                                                     esta' FORA de uma IDE
//! ```
//!
//! Um integrador com um sinal trocado ainda produz curva bonita e ainda decai
//! quando deveria decair. O que ele NAO faz e' convergir na ordem certa — por
//! isso a verificacao e' a ordem, e nao "o resultado parece razoavel".
//!
//! ## A ARMADILHA, medida antes de escrever este arquivo
//!
//! A ordem so' e' mensuravel numa faixa de `dt`. Medido em `roadmaps/31` §11.3,
//! no oscilador amortecido com solucao fechada:
//!
//! ```text
//! de 1e-1 para 1e-2   ordem 3,85   confirmada
//! de 1e-2 para 1e-3   ordem 3,99   confirmada
//! de 1e-3 para 1e-4   ordem 2,21   CONTAMINADA pelo piso do f64
//! de 1e-4 para 1e-5   ordem 0,34   CONTAMINADA
//! ```
//!
//! **Um teste que medisse a ordem do RK4 entre 1e-4 e 1e-5 REPROVARIA um codigo
//! correto.** A faixa e' parte do teste, e por isso ela esta' escrita em cada
//! assercao abaixo em vez de escondida numa constante.

use kinein_protocol::{SimInitial, SimMethod};

use crate::sim::catalogo;
use crate::sim::integrador::{Corrida, ErroDeCorrida, TETO_DE_PASSOS, rotulo};

/// Oscilador harmonico amortecido: `m y'' + c y' + k y = 0`.
/// Com `m=2, c=0.5, k=10`: `y'' = -5y - 0.25y'`.
const M: f64 = 2.0;
const C: f64 = 0.5;
const K: f64 = 10.0;

/// A solucao fechada — **a do CATALOGO**, e nao uma copia dela.
///
/// Usar a mesma funcao que a IDE mostra na tela e' deliberado: uma copia aqui
/// seria um segundo lugar onde o mesmo bug pode morar, e o teste passaria com a
/// tela errada. A independencia que uma verificacao precisa ja' existe, e vem
/// de fora: esta formula foi derivada a mao E conferida contra o `dsolve` do
/// `SymPy` em 2026-09-05 — os dois deram `x(10) = -0.27588626694065...`, iguais
/// ate' a 14a casa (`roadmaps/31` §13.2).
fn analitica(t: f64, y0: f64, v0: f64) -> f64 {
    let exata = catalogo::exata("oscilador-amortecido").expect("o conceito tem solucao fechada");
    let valores: std::collections::BTreeMap<String, f64> = [
        ("m".to_string(), M),
        ("k".to_string(), K),
        ("c".to_string(), C),
    ]
    .into_iter()
    .collect();
    exata(&valores, y0, v0, t).expect("subamortecido: a formula vale")
}

fn corrida(metodo: SimMethod, passo: f64, duracao: f64) -> Corrida {
    Corrida {
        passo,
        duracao,
        metodo,
        amostras: 16,
        inicial: SimInitial {
            y: 1.0,
            dy: Some(0.0),
        },
        segunda_ordem: true,
    }
}

/// `y'' = (-k*y - c*y') / m`.
///
/// A assinatura devolve `Option` porque e' a do trait `Derivada`, que precisa
/// dela: a expressao do usuario pode falhar ao avaliar. Aqui ela nunca falha,
/// e o `allow` registra isso em vez de mudar o contrato so' para o teste.
#[allow(clippy::unnecessary_wraps, clippy::suboptimal_flops)]
fn oscilador(_t: f64, y: f64, dy: f64) -> Option<f64> {
    Some((-K * y - C * dy) / M)
}

fn erro_final(metodo: SimMethod, passo: f64, duracao: f64) -> f64 {
    let saida = corrida(metodo, passo, duracao)
        .integrar(oscilador)
        .expect("integra");
    (saida.final_y - analitica(duracao, 1.0, 0.0)).abs()
}

/// A ordem medida entre dois passos: `log10(erro_grosso / erro_fino)`.
fn ordem(metodo: SimMethod, grosso: f64, fino: f64, duracao: f64) -> f64 {
    (erro_final(metodo, grosso, duracao) / erro_final(metodo, fino, duracao)).log10()
}

#[test]
fn euler_explicito_converge_em_ordem_um() {
    // Faixa 1e-3 -> 1e-4: bem acima do piso do f64 para um metodo de ordem 1.
    let medida = ordem(SimMethod::Euler, 1e-3, 1e-4, 10.0);
    assert!(
        (medida - 1.0).abs() < 0.15,
        "Euler explicito deveria ser ordem 1; medido {medida:.3}"
    );
}

#[test]
fn euler_simpletico_converge_em_ordem_um() {
    let medida = ordem(SimMethod::EulerSymplectic, 1e-3, 1e-4, 10.0);
    assert!(
        (medida - 1.0).abs() < 0.15,
        "Euler simpletico deveria ser ordem 1; medido {medida:.3}"
    );
}

#[test]
fn rk4_converge_em_ordem_quatro_na_faixa_onde_isso_e_mensuravel() {
    // A FAIXA E' PARTE DO TESTE. Entre 1e-1 e 1e-2 o erro cai de 4,9e-5 para
    // 6,9e-9, longe do piso do f64. Medir entre 1e-4 e 1e-5 daria 0,34 e
    // reprovaria este mesmo codigo (`roadmaps/31` §11.3).
    let medida = ordem(SimMethod::Rk4, 1e-1, 1e-2, 10.0);
    assert!(
        (medida - 4.0).abs() < 0.35,
        "RK4 deveria ser ordem 4 nesta faixa; medido {medida:.3}"
    );
}

#[test]
fn o_metodo_muda_o_numero_e_a_diferenca_e_enorme() {
    // A razao de o metodo ser escolha EXPLICITA do usuario (arquitetura/34
    // §2.1) esta neste teste: com o mesmo passo, o erro varia por cinco ordens
    // de grandeza. Um padrao escolhido pela IDE seria a IDE decidindo a
    // exatidao da conta de alguem.
    let exato = analitica(10.0, 1.0, 0.0);
    let e_euler = erro_final(SimMethod::Euler, 1e-1, 10.0);
    let e_rk4 = erro_final(SimMethod::Rk4, 1e-1, 10.0);
    assert!(
        e_euler > exato.abs(),
        "Euler com dt=0.1 deveria errar MAIS que o proprio valor da resposta \
         ({exato:.6}); errou {e_euler:.6}"
    );
    assert!(
        e_rk4 < 1e-3,
        "RK4 com dt=0.1 deveria acertar; errou {e_rk4:e}"
    );
    assert!(
        e_euler / e_rk4 > 1e4,
        "a diferenca entre os metodos encolheu: {:.1e}",
        e_euler / e_rk4
    );
}

#[test]
fn o_simpletico_ganha_do_explicito_no_oscilador() {
    // Os dois sao ordem 1, e mesmo assim um e' muito melhor aqui: o simpletico
    // usa a velocidade NOVA para andar com a posicao, e por isso nao injeta
    // energia. Uma linha de diferenca no codigo.
    let explicito = erro_final(SimMethod::Euler, 1e-2, 10.0);
    let simpletico = erro_final(SimMethod::EulerSymplectic, 1e-2, 10.0);
    assert!(
        simpletico < explicito / 10.0,
        "simpletico {simpletico:e} deveria ser bem melhor que explicito {explicito:e}"
    );
}

#[test]
fn primeira_ordem_confere_com_exponencial() {
    // `y' = -2y`, `y(0)=1`  ->  `y(t) = exp(-2t)`. Uma segunda equacao com
    // solucao fechada, para a forma de primeira ordem nao ficar sem verificacao.
    let mut c = corrida(SimMethod::Rk4, 1e-3, 2.0);
    c.segunda_ordem = false;
    c.inicial = SimInitial { y: 1.0, dy: None };
    let saida = c
        .integrar(|_t: f64, y: f64, _dy: f64| Some(-2.0 * y))
        .expect("integra");
    let exato = (-2.0f64 * 2.0).exp();
    assert!(
        (saida.final_y - exato).abs() < 1e-10,
        "RK4 em y'=-2y deu {} contra {exato}",
        saida.final_y
    );
}

#[test]
fn a_trilha_e_amostrada_e_diz_de_quanto_em_quanto() {
    // Guardar tudo e' inviavel por ordem de grandeza (roadmaps/31 §14). O que
    // impede a tabela amostrada de PARECER completa e' o `a_cada` viajar junto.
    let mut c = corrida(SimMethod::Rk4, 1e-4, 10.0);
    c.amostras = 100;
    let saida = c.integrar(oscilador).expect("integra");
    assert_eq!(saida.passos, 100_000);
    assert_eq!(saida.a_cada, 1_000);
    // ~100 amostras mais a inicial; o instante final entra sempre.
    assert!(
        saida.trilha.len() >= 100 && saida.trilha.len() <= 102,
        "trilha com {} pontos",
        saida.trilha.len()
    );
    assert!((saida.trilha[0].t - 0.0).abs() < 1e-12);
    let ultimo = saida.trilha.last().expect("nao vazia");
    assert!(
        (ultimo.t - 10.0).abs() < 1e-9,
        "o instante pedido nao entrou: {}",
        ultimo.t
    );
}

#[test]
fn segunda_ordem_sem_derivada_inicial_e_recusada() {
    // `y''` precisa de `y'(0)`. Supor zero seria a IDE preenchendo por voce.
    let mut c = corrida(SimMethod::Rk4, 1e-3, 1.0);
    c.inicial = SimInitial { y: 1.0, dy: None };
    assert_eq!(
        c.integrar(oscilador).expect_err("deveria recusar"),
        ErroDeCorrida::FaltaDerivadaInicial
    );
}

#[test]
fn passo_e_duracao_invalidos_sao_recusados() {
    for passo in [0.0, -1e-3, f64::NAN, f64::INFINITY] {
        let c = corrida(SimMethod::Rk4, passo, 1.0);
        assert_eq!(
            c.integrar(oscilador).expect_err("deveria recusar"),
            ErroDeCorrida::PassoInvalido,
            "passo {passo}"
        );
    }
    for duracao in [0.0, -1.0, f64::NAN] {
        let c = corrida(SimMethod::Rk4, 1e-3, duracao);
        assert_eq!(
            c.integrar(oscilador).expect_err("deveria recusar"),
            ErroDeCorrida::DuracaoInvalida,
            "duracao {duracao}"
        );
    }
}

#[test]
fn corrida_longa_demais_e_recusada_com_o_numero_na_recusa() {
    // Recusar dizendo QUANTOS passos seriam e' o que permite ao autor decidir
    // reduzir o passo ou a duracao. Recusar sem o numero seria uma parede.
    let c = corrida(SimMethod::Rk4, 1e-9, 1.0);
    match c.integrar(oscilador).expect_err("deveria recusar") {
        ErroDeCorrida::CorridaLongaDemais { passos, teto } => {
            assert_eq!(teto, TETO_DE_PASSOS);
            assert!(passos > teto, "{passos} deveria passar de {teto}");
        }
        outro => panic!("recusa errada: {outro:?}"),
    }
}

#[test]
fn divergencia_ate_o_infinito_e_erro_e_nao_trilha() {
    // Passo grande num sistema rigido diverge de verdade, e o `f64` chega a
    // `inf`. Entregar a trilha ate' ali com cara de resultado seria pior que
    // parar — e' a mesma decisao que o `NaoFinito` da forma algebrica.
    let c = corrida(SimMethod::Euler, 5.0, 10_000.0);
    match c.integrar(oscilador).expect_err("deveria divergir") {
        ErroDeCorrida::Divergiu { passo, t } => {
            assert!(passo > 0, "passo da divergencia nao foi registrado");
            assert!(t > 0.0, "instante da divergencia nao foi registrado");
        }
        outro => panic!("erro errado: {outro:?}"),
    }
}

#[test]
fn o_crescimento_absurdo_abaixo_do_infinito_aparece_na_magnitude() {
    // ESTE TESTE NASCEU DE UMA FALHA, em 2026-09-05. A checagem de `is_finite`
    // parecia suficiente ate' um teste mostrar que com `dt=5` e 200 passos o
    // oscilador amortecido chega a 3,5e209 SEM nunca virar `inf` — lixo com
    // cara de resultado, e a mesma forma do defeito de estabilidade da EDP.
    //
    // A IDE **nao adivinha** se o crescimento e' fisico: uma EDO exponencial
    // cresce mesmo, e chutar um teto seria decidir pelo usuario. O que ela faz
    // e' MOSTRAR a magnitude — e, com solucao fechada, o erro.
    let saida = corrida(SimMethod::Euler, 5.0, 1000.0)
        .integrar(oscilador)
        .expect("nao chega a inf, entao integra");
    assert!(
        saida.maior_abs > 1e100,
        "a magnitude absurda nao foi registrada: {:e}",
        saida.maior_abs
    );
    // E o oraculo e' quem denuncia: a resposta certa em t=1000 e' ~0.
    let exato = analitica(1000.0, 1.0, 0.0);
    assert!(
        exato.abs() < 1e-20,
        "o amortecido deveria ter morrido: {exato:e}"
    );
    assert!(
        (saida.final_y - exato).abs() > 1e100,
        "o erro contra a solucao fechada deveria ser gigante"
    );
}

#[test]
fn corrida_saudavel_tem_magnitude_do_tamanho_da_condicao_inicial() {
    // O contraponto do teste acima: com passo sensato, `maior_abs` fica na
    // ordem de `y(0)`. Sem esta assercao a magnitude poderia ser sempre enorme
    // e o teste de cima passaria por acidente.
    let saida = corrida(SimMethod::Rk4, 1e-3, 10.0)
        .integrar(oscilador)
        .expect("integra");
    assert!(
        saida.maior_abs <= 1.0 + 1e-9,
        "oscilador amortecido nao deveria passar de y(0)=1: {}",
        saida.maior_abs
    );
}

#[test]
fn o_rotulo_carrega_metodo_e_passo() {
    // Numero sem procedencia mente: a tela nunca mostra o resultado sem dizer
    // com que metodo e com que passo ele saiu.
    let texto = rotulo(SimMethod::Rk4, 1e-3);
    assert!(texto.contains("Runge-Kutta 4"), "{texto}");
    assert!(texto.contains("1e-3"), "{texto}");
}

#[test]
fn a_contagem_de_passos_e_publica_para_a_ide_mostrar_antes() {
    // A IDE mostra o custo ANTES de rodar (arquitetura/34 §6). Se este numero
    // nao fosse acessivel sem integrar, a estimativa teria de ser uma segunda
    // conta — e duas contas para a mesma verdade e' o anti-padrao da §8.1.
    let c = corrida(SimMethod::Rk4, 1e-4, 10.0);
    assert_eq!(c.passos(), 100_000);
}

// ---------------------------------------------------------------------------
// A CORRIDA DE PONTA A PONTA, pelo IPC: formula do usuario -> trajetoria, com
// o oraculo dizendo o quanto ela errou.
// ---------------------------------------------------------------------------

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::core_with_empty_search_path;
use crate::Core;

/// A montagem do oscilador amortecido como o autor a faria na tela.
fn params_oscilador(metodo: &str, passo: f64, duracao: f64) -> Value {
    json!({
        "concept": "oscilador-amortecido",
        // `y'' = (-k*y - c*dy) / m`, escrita pelo usuario.
        "formula": "(-k*y - c*dy) / m",
        "bindings": [
            { "variable": "k", "quantity": "k" },
            { "variable": "y", "quantity": "y" },
            { "variable": "c", "quantity": "c" },
            { "variable": "dy", "quantity": "dy" },
            { "variable": "m", "quantity": "m" }
        ],
        "values": [
            { "quantity": "m", "value": 2.0 },
            { "quantity": "k", "value": 10.0 },
            { "quantity": "c", "value": 0.5 }
        ],
        "initial": { "y": 1.0, "dy": 0.0 },
        "duration": duracao,
        "step": passo,
        "method": metodo,
        "samples": 50
    })
}

fn rodar(core: &mut Core, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(9_i64, "sim.run", Some(params.clone())));
    let resposta = saida.response();
    assert!(
        resposta.error.is_none(),
        "sim.run falhou: {:?}",
        resposta.error
    );
    resposta.result.clone().expect("resultado")
}

fn recusar_run(core: &mut Core, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(
        10_i64,
        "sim.run",
        Some(params.clone()),
    ));
    let erro = saida.response().error.clone().expect("deveria recusar");
    serde_json::to_value(erro).expect("erro serializa")
}

#[test]
fn sim_run_integra_a_formula_do_usuario_e_diz_o_erro() {
    let mut core = core_with_empty_search_path("sim-run");
    let resultado = rodar(&mut core, &params_oscilador("rk4", 1e-3, 10.0));

    assert_eq!(resultado["stepsTaken"], json!(10_000));
    // A amostragem VAI A TELA: uma tabela amostrada parece completa.
    assert_eq!(resultado["sampleEvery"], json!(200));
    assert!(resultado["trail"].as_array().expect("trilha").len() >= 50);
    // Numero sem procedencia mente: o metodo e o passo viajam junto.
    let rotulo = resultado["methodLabel"].as_str().expect("rotulo");
    assert!(rotulo.contains("Runge-Kutta 4"), "{rotulo}");

    // O ORACULO: a solucao fechada ao lado do numerico, com a diferenca.
    let exatidao = &resultado["accuracy"];
    assert!(
        !exatidao.is_null(),
        "conceito com solucao fechada sem exatidao"
    );
    let exato = exatidao["exact"].as_f64().expect("exato");
    let erro = exatidao["absoluteError"].as_f64().expect("erro");
    assert!(
        (exato - (-0.275_886_266_940_653)).abs() < 1e-9,
        "a solucao fechada mudou: {exato}"
    );
    assert!(
        erro < 1e-9,
        "RK4 com dt=1e-3 deveria acertar; errou {erro:e}"
    );
}

#[test]
fn o_oraculo_denuncia_o_metodo_ruim_pelo_ipc() {
    // Este e' o teste que torna "exato e preciso" um mecanismo em vez de uma
    // promessa. A MESMA formula, a MESMA fisica, o MESMO passo — e o erro sai
    // cinco ordens de grandeza diferente por causa do metodo. Sem a coluna do
    // erro, as duas curvas apareceriam como resultado.
    let mut core = core_with_empty_search_path("sim-run-oraculo");
    let bom = rodar(&mut core, &params_oscilador("rk4", 1e-1, 10.0));
    let ruim = rodar(&mut core, &params_oscilador("euler", 1e-1, 10.0));

    let erro_bom = bom["accuracy"]["absoluteError"].as_f64().expect("erro");
    let erro_ruim = ruim["accuracy"]["absoluteError"].as_f64().expect("erro");
    let exato = bom["accuracy"]["exact"].as_f64().expect("exato");

    assert!(erro_bom < 1e-3, "RK4 dt=0.1 deveria acertar: {erro_bom:e}");
    assert!(
        erro_ruim > exato.abs(),
        "Euler dt=0.1 deveria errar MAIS que o proprio valor da resposta \
         ({exato}); errou {erro_ruim}"
    );
    assert!(
        erro_ruim / erro_bom > 1e4,
        "a denuncia encolheu: {:e}",
        erro_ruim / erro_bom
    );
}

#[test]
fn conceito_algebrico_recusa_sim_run_em_vez_de_devolver_reta() {
    // Pedir trajetoria de algo que nao tem uma e' um engano do usuario, e a
    // recusa diz isso. Devolver uma reta seria a IDE inventando um resultado.
    let mut core = core_with_empty_search_path("sim-run-algebrico");
    let erro = recusar_run(
        &mut core,
        &json!({
            "concept": "energia-cinetica",
            "formula": "0.5*m*v^2",
            "bindings": [
                { "variable": "m", "quantity": "m" },
                { "variable": "v", "quantity": "v" }
            ],
            "values": [{ "quantity": "m", "value": 2.0 }, { "quantity": "v", "value": 3.0 }],
            "initial": { "y": 0.0, "dy": 0.0 },
            "duration": 1.0, "step": 1e-3, "method": "rk4", "samples": 10
        }),
    );
    assert_eq!(erro["details"]["reason"], json!("notIntegrable"));
}

#[test]
fn corrida_longa_demais_recusa_com_o_numero_para_o_autor_decidir() {
    let mut core = core_with_empty_search_path("sim-run-longa");
    let erro = recusar_run(&mut core, &params_oscilador("rk4", 1e-9, 1.0));
    assert_eq!(erro["details"]["reason"], json!("tooManySteps"));
    // Sem o numero, a recusa e' uma parede; com ele, o autor reduz o passo.
    let passos = erro["details"]["steps"].as_u64().expect("passos");
    let teto = erro["details"]["limit"].as_u64().expect("teto");
    assert!(passos > teto, "{passos} vs {teto}");
}

#[test]
fn divergencia_pelo_ipc_diz_onde_parou() {
    let mut core = core_with_empty_search_path("sim-run-diverge");
    let erro = recusar_run(&mut core, &params_oscilador("euler", 5.0, 10_000.0));
    assert_eq!(erro["details"]["reason"], json!("diverged"));
    assert!(erro["details"]["step"].as_u64().expect("passo") > 0);
}

#[test]
fn primeira_ordem_nao_exige_velocidade_inicial() {
    // `decaimento-exponencial` e' Ode1: pedir `y'(0)` seria pedir algo que a
    // equacao nao tem.
    let mut core = core_with_empty_search_path("sim-run-ode1");
    let resultado = rodar(
        &mut core,
        &json!({
            "concept": "decaimento-exponencial",
            "formula": "-k*y",
            "bindings": [
                { "variable": "k", "quantity": "k" },
                { "variable": "y", "quantity": "y" }
            ],
            "values": [{ "quantity": "k", "value": 2.0 }],
            "initial": { "y": 1.0 },
            "duration": 2.0, "step": 1e-4, "method": "rk4", "samples": 20
        }),
    );
    let exatidao = &resultado["accuracy"];
    let exato = exatidao["exact"].as_f64().expect("exato");
    // exp(-2*2) = 0.0183156...
    assert!((exato - (-4.0f64).exp()).abs() < 1e-12, "{exato}");
    assert!(exatidao["absoluteError"].as_f64().expect("erro") < 1e-12);
}

#[test]
fn a_estimativa_mostra_o_custo_antes_de_rodar() {
    // A IDE MOSTRA e o usuario escolhe (arquitetura/34 §2.1). Este teste prova
    // que ela tem o que mostrar: passos, amostragem, o tamanho da trilha
    // completa contra a amostrada, e se compilar valeria a pena.
    let mut core = core_with_empty_search_path("sim-estimate");
    let r = rodar_metodo(
        &mut core,
        "sim.estimate",
        &json!({ "duration": 10.0, "step": 1e-4, "samples": 100 }),
    );
    assert_eq!(r["steps"], json!(100_000));
    assert_eq!(r["sampleEvery"], json!(1_000));
    assert_eq!(r["tooMany"], json!(false));
    // 100.000 passos x 24 bytes = 2,4 MB completos contra 2,4 KB amostrados.
    assert_eq!(r["fullTrailBytes"], json!(2_400_000));
    assert_eq!(r["sampledTrailBytes"], json!(2_400));
    // Nesta escala, interpretar ganha: o ponto de virada e' 21,5 milhoes.
    assert_eq!(r["compilingWouldPay"], json!(false));
}

#[test]
fn a_estimativa_avisa_quando_compilar_passaria_a_valer() {
    let mut core = core_with_empty_search_path("sim-estimate-compilar");
    let r = rodar_metodo(
        &mut core,
        "sim.estimate",
        &json!({ "duration": 100.0, "step": 1e-6, "samples": 1000 }),
    );
    assert_eq!(r["steps"], json!(100_000_000));
    assert_eq!(r["compilingWouldPay"], json!(true));
    // E ela tambem diz que essa corrida seria recusada: 1e8 passos e' o teto.
    assert_eq!(r["tooMany"], json!(false));
    // A trilha completa seria 2,4 GB — o numero que justifica a amostragem.
    assert_eq!(r["fullTrailBytes"], json!(2_400_000_000_u64));
}

fn rodar_metodo(core: &mut Core, metodo: &str, params: &Value) -> Value {
    let saida = core.handle_request(&JsonRpcRequest::new(11_i64, metodo, Some(params.clone())));
    let resposta = saida.response();
    assert!(
        resposta.error.is_none(),
        "{metodo} falhou: {:?}",
        resposta.error
    );
    resposta.result.clone().expect("resultado")
}
