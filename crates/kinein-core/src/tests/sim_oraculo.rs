//! O ORACULO: o que ele PERGUNTA, o que ele aceita, e o que ele recusa.
//!
//! ## O que estes testes existem para impedir
//!
//! Medido contra o binario real em 2026-09-06 (`roadmaps/31` §19.0): com
//! `-(k/m)*x - 2*(c/m)*v` no oscilador amortecido a IDE acusava erro de
//! **2,5e-2 numa integracao correta ate' 3,2e-7** — setenta e oito mil vezes o
//! que reportava. A coluna `exato` vinha do CONCEITO e nao olhava a formula
//! digitada, e o `sim.checkFormula` a aprovava porque confere ligacao, nao
//! fisica.
//!
//! ## Por que um `python3` FALSO
//!
//! A pergunta que importa nao e' "a resposta chegou?" — e' **o que exatamente
//! foi perguntado**, e se a resposta passa pelo portao. O
//! `scripts/fake_sympy_oracle.py` responde na hora, grava o pedido que recebeu,
//! e sabe fingir cada modo de falha que a medicao encontrou. Mesmo molde do
//! `fake_lsp_server.py`, e pela mesma razao (olhar o wire, nao a resposta).
//!
//! **E o `SymPy` de verdade nao esta nesta maquina** (medido em 2026-09-10:
//! `ModuleNotFoundError`), entao um teste que dependesse dele nao rodaria — ou,
//! pior, seria pulado em silencio.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use kinein_protocol::{SimAccuracySource, SimBinding, SimInitial, SimMethod, SimValue};

use crate::sim::corrida;
use crate::sim::oraculo::{self, Config, NaoSei, Pergunta};

/// O `python3` falso, pelo caminho do repositorio.
fn falso() -> PathBuf {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("scripts/fake_sympy_oracle.py");
    let script = script.canonicalize().unwrap_or(script);
    assert!(
        script.is_file(),
        "scripts/fake_sympy_oracle.py ausente em {}",
        script.display()
    );
    script
}

/// Um "interpretador" que e' o falso com o cenario ja' escolhido.
///
/// **Por que um invocador em vez de variavel de ambiente.** Este repositorio
/// proibe `unsafe`, e escrever variavel de ambiente virou `unsafe` na edicao
/// 2024 — entao o cenario nao pode viajar por ambiente. Ele viaja por argumento,
/// dentro de um invocador de tres linhas que o teste escreve num diretorio
/// proprio. Sai de graca uma propriedade que o ambiente nao dava: dois testes
/// podem rodar em paralelo com cenarios diferentes.
fn invocador(nome: &str, modo: &str, log: Option<&Path>, expressao: Option<&str>) -> PathBuf {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let pasta = std::env::temp_dir().join(format!("kinein-oraculo-{nome}"));
    let _ = std::fs::create_dir_all(&pasta);
    let caminho = pasta.join("python3-falso");
    let mut partes = vec![format!("exec {:?}", falso()), format!("--modo={modo}")];
    if let Some(destino) = log {
        partes.push(format!("--log={destino:?}"));
    }
    if let Some(texto) = expressao {
        partes.push(format!("--expr={texto:?}"));
    }
    partes.push("\"$@\"".to_owned());
    let linha = format!("#!/bin/sh\n{}\n", partes.join(" "));

    let mut arquivo = std::fs::File::create(&caminho).expect("cria o invocador");
    arquivo
        .write_all(linha.as_bytes())
        .expect("escreve o invocador");
    drop(arquivo);
    let mut permissoes = std::fs::metadata(&caminho)
        .expect("le as permissoes")
        .permissions();
    permissoes.set_mode(0o755);
    std::fs::set_permissions(&caminho, permissoes).expect("torna executavel");
    caminho
}

/// A configuracao apontando para um cenario do falso.
fn config(nome: &str, modo: &str) -> Config {
    Config {
        interpretador: invocador(nome, modo, None, None)
            .to_string_lossy()
            .into_owned(),
        teto: Duration::from_secs(5),
    }
}

fn liga(variavel: &str, grandeza: &str) -> SimBinding {
    SimBinding {
        variable: variavel.to_owned(),
        quantity: grandeza.to_owned(),
    }
}

/// O oscilador amortecido como o usuario o monta: `x` e' a posicao, `v` a
/// velocidade, e `k`, `m`, `c` sao parametros.
fn ligacoes_do_oscilador() -> Vec<SimBinding> {
    vec![
        liga("k", "k"),
        liga("m", "m"),
        liga("c", "c"),
        liga("x", "y"),
        liga("v", "dy"),
    ]
}

fn valores_do_oscilador() -> Vec<SimValue> {
    vec![
        SimValue {
            quantity: "k".to_owned(),
            value: 2.0,
        },
        SimValue {
            quantity: "m".to_owned(),
            value: 1.0,
        },
        SimValue {
            quantity: "c".to_owned(),
            value: 0.5,
        },
    ]
}

fn rodar(oraculo: &Config, formula: &str) -> kinein_protocol::SimRunResult {
    let conceito = crate::sim::catalogo::conceito("oscilador-amortecido")
        .expect("o oscilador amortecido esta no catalogo");
    let inicial = SimInitial {
        y: 1.0,
        dy: Some(0.0),
    };
    corrida::executar(
        &conceito,
        formula,
        &ligacoes_do_oscilador(),
        &valores_do_oscilador(),
        inicial.y,
        inicial.dy,
        10.0,
        0.1,
        SimMethod::Rk4,
        50,
        oraculo,
    )
    .expect("a corrida do oscilador amortecido sai")
}

/// O oraculo recebe os PAPEIS nomeados, nunca a posicao na formula.
///
/// E' a mesma regra que governa o vetor de avaliacao (ADR-0006, armadilha 1):
/// o `var_names()` do `exmex` devolve em ordem ALFABETICA, e quem casar por
/// ordem de leitura calcula a fisica errada sem erro nenhum. Aqui o risco e' o
/// mesmo num lugar novo — quem for `y` e quem for `y'` na equacao que o oraculo
/// monta.
#[test]
fn o_oraculo_recebe_os_papeis_nomeados_e_os_parametros_como_decimal() {
    let log = std::env::temp_dir().join("kinein-oraculo-wire.json");
    let _ = std::fs::remove_file(&log);
    let oraculo = Config {
        interpretador: invocador("wire", "ok", Some(&log), None)
            .to_string_lossy()
            .into_owned(),
        teto: Duration::from_secs(5),
    };

    let _ = rodar(&oraculo, "-(k/m)*x - (c/m)*v");

    let bruto = std::fs::read_to_string(&log).expect("o falso gravou o pedido");
    let pedido: serde_json::Value = serde_json::from_str(&bruto).expect("o pedido e JSON");

    assert_eq!(pedido["estado"], "x", "a POSICAO tem de viajar nomeada");
    assert_eq!(
        pedido["derivada"], "v",
        "a VELOCIDADE tem de viajar nomeada"
    );
    assert_eq!(pedido["ordem"], 2);
    assert_eq!(pedido["y0"], "1");
    assert_eq!(pedido["dy0"], "0");

    // Os parametros vao como TEXTO decimal, e nao como float: `Rational("0.5")`
    // do outro lado e' exato, e `dsolve` com float da' RecursionError depois de
    // 4,2 s (`roadmaps/31` §19.3.3).
    let parametros: BTreeMap<String, String> = pedido["parametros"]
        .as_array()
        .expect("parametros e lista")
        .iter()
        .map(|par| {
            (
                par[0].as_str().unwrap_or_default().to_owned(),
                par[1].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    assert_eq!(parametros.get("c").map(String::as_str), Some("0.5"));
    assert_eq!(parametros.get("k").map(String::as_str), Some("2"));
    assert_eq!(parametros.get("m").map(String::as_str), Some("1"));
    let _ = std::fs::remove_file(&log);
}

/// Com o oraculo respondendo, o `exato` passa a ter a procedencia CERTA.
#[test]
fn com_oraculo_a_procedencia_do_exato_e_a_equacao_digitada() {
    let resultado = rodar(&config("procedencia", "ok"), "-(k/m)*x - (c/m)*v");
    let exatidao = resultado.accuracy.expect("ha' comparacao");

    assert_eq!(exatidao.source, SimAccuracySource::Oracle);
    assert_eq!(exatidao.solved_by.as_deref(), Some("SymPy 1.14.0"));
    assert!(
        exatidao.closed_form.is_some(),
        "a resposta verdadeira vai a tela ao lado do numero calculado"
    );
    assert!(
        resultado.oracle_note.is_none(),
        "com o oraculo respondendo nao ha' ressalva a fazer"
    );

    // O valor do oraculo bate com a analitica derivada a mao ate' a 9a casa —
    // e' a mesma conta, por dois caminhos independentes.
    assert!(
        (exatidao.exact - 0.032_128_319_832_031_9).abs() < 1e-9,
        "o oraculo devolveu {}",
        exatidao.exact
    );
    let relativo = exatidao.relative_error.expect("ha' erro relativo");
    assert!(relativo.is_finite() && relativo >= 0.0);
}

/// **O TESTE QUE PROVA O CONSERTO DA §19.0.**
///
/// Com uma formula DIFERENTE da canonica, o valor exato tem de MUDAR — porque
/// agora ele responde pela equacao que foi digitada. Antes deste conserto ele
/// era o mesmo `0,032128` para as quatro formulas medidas.
#[test]
fn o_exato_muda_quando_a_equacao_muda() {
    let primeira = Config {
        interpretador: invocador("muda-a", "livre", None, Some("exp(-t/4)"))
            .to_string_lossy()
            .into_owned(),
        teto: Duration::from_secs(5),
    };
    let segunda = Config {
        interpretador: invocador("muda-b", "livre", None, Some("exp(-t/2)"))
            .to_string_lossy()
            .into_owned(),
        teto: Duration::from_secs(5),
    };
    let a = rodar(&primeira, "-(k/m)*x - (c/m)*v")
        .accuracy
        .expect("ha' comparacao");
    let b = rodar(&segunda, "-(k/m)*x - 2*(c/m)*v")
        .accuracy
        .expect("ha' comparacao");

    assert_eq!(a.source, SimAccuracySource::Oracle);
    assert_eq!(b.source, SimAccuracySource::Oracle);
    assert!(
        (a.exact - b.exact).abs() > 1e-6,
        "o valor exato NAO pode ser o mesmo para equacoes diferentes: {} contra {}",
        a.exact,
        b.exact
    );
}

/// Sem `SymPy`, a coluna CONTINUA — e diz que e' do conceito.
///
/// Esconder o numero seria trocar uma mentira por um buraco: sem a ferramenta,
/// a solucao do conceito continua sendo a melhor resposta disponivel. O que
/// faltava era **dizer que e' ela**.
#[test]
fn sem_sympy_a_coluna_fica_e_a_procedencia_e_dita() {
    let resultado = rodar(&config("sem-sympy", "nosympy"), "-(k/m)*x - (c/m)*v");
    let exatidao = resultado.accuracy.expect("a coluna do conceito continua");

    assert_eq!(exatidao.source, SimAccuracySource::Concept);
    assert!(exatidao.closed_form.is_none());
    let nota = resultado.oracle_note.expect("a ressalva vai junto");
    assert!(
        nota.contains("SymPy"),
        "a frase tem de dizer o que falta: {nota}"
    );
}

/// O `dsolve` que nao sabe resolver responde "nao sei" — e isso e' resposta.
#[test]
fn quando_o_oraculo_nao_sabe_a_ressalva_diz_isso() {
    let resultado = rodar(&config("nao-sabe", "cannotsolve"), "-(k/m)*x - (c/m)*v");

    assert_eq!(
        resultado
            .accuracy
            .expect("a coluna do conceito continua")
            .source,
        SimAccuracySource::Concept
    );
    let nota = resultado.oracle_note.expect("a ressalva vai junto");
    assert!(nota.contains("não sabe"), "frase inesperada: {nota}");
}

/// O oraculo que TRAVA nao trava a IDE.
///
/// O pendulo nao linearizado (`y'' = -sin(y)`) — a primeira equacao de Fisica I
/// que nao e' de brinquedo — nao volta em 20 s. Sem o teto, a IDE congela numa
/// pergunta que o usuario tem direito de fazer.
#[test]
fn o_oraculo_que_trava_perde_o_prazo_em_vez_de_travar_a_ide() {
    let travado = Config {
        interpretador: invocador("trava", "hang", None, None)
            .to_string_lossy()
            .into_owned(),
        teto: Duration::from_millis(300),
    };
    let comeco = std::time::Instant::now();
    let saida = oraculo::resolver(
        &travado,
        &Pergunta {
            formula: "-sin(x)".to_owned(),
            estado: "x".to_owned(),
            derivada: None,
            tempo: None,
            parametros: Vec::new(),
            ordem: 2,
            y0: "1".to_owned(),
            dy0: Some("0".to_owned()),
        },
    );
    assert_eq!(saida, Err(NaoSei::TempoEsgotado));
    assert!(
        comeco.elapsed() < Duration::from_secs(3),
        "o teto nao foi respeitado: {:?}",
        comeco.elapsed()
    );
}

/// **O PORTAO, e a falha SILENCIOSA que ele existe para pegar.**
///
/// Medido em 2026-09-06 (`roadmaps/31` §19.3.4): `sp.N()` NAO substitui `pi`, e
/// o `exmex` o le' como VARIAVEL LIVRE. Preencher o vetor de avaliacao pelo
/// tamanho de `var_names()` — que e' o que um codigo descuidado faz — devolve
/// `0,086232` onde a resposta e' `0,100000`, **sem erro nenhum**.
///
/// MUTACAO QUE PROVA ESTE GATE: tire a conferencia de `var_names()` do
/// `oraculo::portao` e este teste passa a aceitar a expressao com `pi`.
#[test]
fn o_portao_recusa_solucao_com_variavel_alem_do_tempo() {
    let recusa = oraculo::portao("0.1*cos(pi*t)");
    assert!(
        matches!(recusa, Err(NaoSei::RespostaIlegivel { .. })),
        "o `pi` entrou como variavel livre e passou: {recusa:?}"
    );

    // E o caminho feliz continua passando: uma variavel so', e ela e' `t`.
    assert!(oraculo::portao("exp(-t/4)*cos(t)").is_ok());
}

/// O dialeto do `SymPy` vira o dialeto do `exmex`, e o valor sobrevive a troca.
///
/// O `sstr` escreve `t**2`; o `exmex` recusa `**` ALTO. A troca e' segura por
/// construcao — `**` so' significa potencia —, e este teste mede que ela
/// funciona em vez de supor.
#[test]
fn o_dialeto_do_sympy_vira_o_dialeto_do_exmex() {
    assert!(
        exmex::parse::<f64>("t**2 + 1").is_err(),
        "se o exmex passar a ler `**`, esta conversao virou codigo morto"
    );
    let convertida = oraculo::portao("t**2 + 1").expect("depois da troca o exmex le");
    assert_eq!(convertida, "t^2 + 1");

    let solucao = oraculo::Solucao {
        expressao: convertida,
        resolvedor: "SymPy 1.14.0".to_owned(),
    };
    let valor = oraculo::avaliar(&solucao, 3.0).expect("avalia");
    assert!((valor - 10.0).abs() < 1e-12, "valor {valor}");
}

/// A solucao do oraculo e a analitica escrita a mao dao o MESMO numero.
///
/// E' a medicao de 2026-09-06 virando gate: as duas concordam ate' `6,9e-18`, e
/// e' isso que autoriza uma ida por FORMULA em vez de 500 idas por ponto.
#[test]
fn a_solucao_do_oraculo_e_a_analitica_concordam() {
    let solucao = oraculo::Solucao {
        expressao: "(sqrt(31)*sin(sqrt(31)*t/4)/31 + cos(sqrt(31)*t/4))*exp(-t/4)".to_owned(),
        resolvedor: "SymPy 1.14.0".to_owned(),
    };
    let do_oraculo = oraculo::avaliar(&solucao, 10.0).expect("avalia");

    let mut valores = BTreeMap::new();
    valores.insert("k".to_owned(), 2.0);
    valores.insert("m".to_owned(), 1.0);
    valores.insert("c".to_owned(), 0.5);
    let a_mao = crate::sim::exata::oscilador_amortecido(&valores, 1.0, 0.0, 10.0)
        .expect("a analitica responde");

    assert!(
        (do_oraculo - a_mao).abs() < 1e-12,
        "oraculo {do_oraculo} contra analitica {a_mao}"
    );
}
