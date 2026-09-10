//! O ORACULO DE EXATIDAO: resolve a equacao que o usuario DIGITOU.
//!
//! ## Por que ele existe, e o que ele conserta
//!
//! Ate' 2026-09-10 a coluna `exato` da tela vinha de
//! [`super::catalogo::exata`], que responde pela equacao CANONICA do conceito e
//! **nao olha a formula digitada**. O `sim.checkFormula` aprova a formula
//! porque confere ligacao, nao fisica — entao as duas divergem sem nada
//! reclamar. Medido contra o binario real em 2026-09-06 (`roadmaps/31` §19.0),
//! no oscilador amortecido com `k=2, m=1, c=0,5`:
//!
//! ```text
//! formula digitada            "erro" que a IDE mostrava    erro REAL
//! -(k/m)*x - (c/m)*v                 3,732e-06             3,732e-06   (igual)
//! (k/m)*x - (c/m)*v                  8,318e+04             1,474e+00
//! -(k/m)*x - 2*(c/m)*v               2,525e-02             3,222e-07   (78.000x)
//! ```
//!
//! **O conserto nao e' esconder o numero: e' dizer de onde ele vem**
//! (`SimAccuracySource`). Com o oraculo presente, a procedencia sobe de
//! "solucao do conceito" para "solucao da SUA equacao".
//!
//! ## A fronteira, e ela e' a do GDB
//!
//! O `SymPy` e' **processo externo orquestrado**, nunca linkado
//! (`arquitetura/34` §7). Ausente, ele nao derruba nada: a simulacao roda, a
//! coluna troca de procedencia, e a tela **diz que trocou e por que**.
//!
//! ## Cinco armadilhas medidas, e a defesa de cada uma
//!
//! ```text
//! 1. o `dsolve` TRAVA          o pendulo NAO linearizado (`y'' = -sin(y)`) —
//!                              a primeira equacao de Fisica I que nao e' de
//!                              brinquedo — nao volta em 20 s. Defesa: TETO DE
//!                              TEMPO, e "nao sei" em vez de congelar
//! 2. o float QUEBRA            `u'' + 4.905*u = 0` da' RecursionError depois de
//!    e quebra DEVAGAR          4,2 s; com `Rational(981/200)` resolve. Defesa:
//!                              racionalizar ANTES de perguntar. Custa 8,9 ms
//! 3. `**` nao e' `^`           o `sstr` do `SymPy` escreve `t**2` e o `exmex`
//!                              RECUSA. Defesa: troca textual — `**` so'
//!                              significa potencia
//! 4. o `pi` e' SILENCIOSO      `sp.N()` nao substitui `pi`, e o `exmex` o le'
//!                              como VARIAVEL LIVRE. Medido: preencher o vetor
//!                              pelo tamanho de `var_names()` devolve 0,086 no
//!                              lugar de 0,100 SEM ERRO NENHUM. E' a armadilha 2
//!                              do ADR-0006 reaparecendo onde nao ha' usuario
//!                              para ligar nada
//! 5. vocabulario alheio        `Abs`, `LambertW`, `erf`, `Piecewise`, `re`,
//!                              `log(t,2)` sao recusados ALTO pelo exmex; `I`
//!                              (imaginario) entra como variavel livre
//! ```
//!
//! **A defesa 4 cobre a 5, e e' por isso que ela e' a unica trava explicita:**
//! o oraculo recusa toda expressao cujo `var_names()` nao seja **exatamente**
//! `["t"]`. O que o exmex nao le' morre no parse (alto); o que ele le' errado
//! vira variavel livre, e a trava pega. Uma trava, dois modos de falha.
//!
//! ## Uma ida por FORMULA, nunca por ponto
//!
//! O `SymPy` resolve, a IDE guarda a EXPRESSAO, e o Rust avalia a trilha inteira
//! com o `exmex`. Medido em 2026-09-06: os dois concordam ate' `6,9e-18`. Sem
//! isso seriam 500 idas de 1,4 ms por corrida.
//!
//! ## O que ele NAO faz
//!
//! Nao narra a resolucao. O passo a passo algebrico de uma EDO nao existe em
//! ferramenta auditavel (`roadmaps/31` §13), e inventar narrativa plausivel e'
//! o anti-padrao da `ARCHITECTURE.md` §8.1.

use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use exmex::prelude::*;
use serde::{Deserialize, Serialize};

/// Quanto o oraculo pode pensar antes de a IDE desistir dele.
///
/// **Nao e' teto de gosto.** Medido em 2026-09-06: as EDOs do catalogo custam
/// de 22 a 229 ms, o arrasto quadratico custa 1,8 s, e o pendulo nao
/// linearizado **nao volta em 20 s**. Cinco segundos cobrem tudo que foi medido
/// e matam o que trava. Passar disso, a IDE diz "nao sei" — que e' resposta, e
/// congelar nao e'.
const TETO: Duration = Duration::from_secs(5);

/// De quanto em quanto se pergunta se o processo terminou.
const RESPIRO: Duration = Duration::from_millis(20);

/// Variavel de ambiente que troca o interpretador.
///
/// E' para a MAQUINA do usuario, nao para o teste: quem tem o `SymPy` numa venv
/// ou num `python3.13` aponta para ele sem recompilar nada. O teste injeta pelo
/// [`Config`], porque `unsafe_code = "forbid"` neste projeto e escrever
/// variavel de ambiente virou `unsafe` na edicao 2024 — e a trava esta certa:
/// ambiente e' estado global, e teste que o escreve contamina o vizinho.
pub const VAR_INTERPRETADOR: &str = "KINEIN_SIM_PYTHON";

/// O interpretador padrao, quando nada o troca.
const INTERPRETADOR_PADRAO: &str = "python3";

/// Com quem falar, e por quanto tempo esperar.
///
/// Existe para a dependencia externa ser INJETADA em vez de descoberta: o
/// produto monta o padrao (o `python3` do `PATH`, ou o que a
/// [`VAR_INTERPRETADOR`] disser), e o teste aponta para um `Python` falso que
/// grava o que recebeu. Sem isso, "o oraculo pergunta certo" seria afirmacao em
/// vez de medicao — e o `SymPy` nao esta nesta maquina para desmentir.
#[derive(Debug, Clone)]
pub struct Config {
    /// O executavel a chamar.
    pub interpretador: String,
    /// Quanto ele pode pensar antes de a IDE desistir.
    pub teto: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interpretador: std::env::var(VAR_INTERPRETADOR)
                .unwrap_or_else(|_| INTERPRETADOR_PADRAO.to_owned()),
            teto: TETO,
        }
    }
}

/// A unica variavel que a solucao pode ter.
const VARIAVEL_DO_TEMPO: &str = "t";

/// O que o oraculo respondeu, quando respondeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solucao {
    /// A expressao ja' no dialeto do `exmex`, com `var_names() == ["t"]`.
    pub expressao: String,
    /// Quem resolveu, por extenso: `SymPy 1.14.0`.
    pub resolvedor: String,
}

/// Por que o oraculo nao respondeu.
///
/// Cada variante vira **frase** na tela, e nao codigo: quem le e' quem digitou
/// a equacao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NaoSei {
    /// Nao ha' interpretador `Python` nesta maquina.
    SemPython,
    /// Ha' `Python`, mas o `SymPy` nao esta instalado.
    SemSympy,
    /// O `dsolve` nao voltou dentro do [`TETO`].
    TempoEsgotado,
    /// O `dsolve` respondeu que nao sabe resolver esta equacao.
    NaoResolve,
    /// Ele resolveu, e a resposta nao passou no portao do `exmex`.
    RespostaIlegivel {
        /// O que exatamente foi recusado, para o registro nao sumir.
        motivo: String,
    },
}

impl NaoSei {
    /// A frase que vai a tela. Diz o que falta E o que fazer a respeito.
    #[must_use]
    pub fn frase(&self) -> String {
        match self {
            Self::SemPython => "O valor exato vem da solução do conceito, não da equação que \
                 você escreveu: para a IDE resolver a SUA equação ela precisa do Python com o \
                 SymPy, e não encontrou Python nesta máquina."
                .to_owned(),
            // NAO se cita nome de pacote aqui, e a omissao e' deliberada:
            // `python3-sympy` no Fedora e' `python-sympy` no Arch, e traduzir
            // nome de pacote por distro e' o "palpite disfarcado de instrucao"
            // que o `tools.rs` recusou em 2026-08 e que o dominio `setup`
            // resolve do jeito certo — citando a FONTE OFICIAL, com data.
            Self::SemSympy => "O valor exato vem da solução do conceito, não da equação que \
                 você escreveu: para a IDE resolver a SUA equação ela precisa do SymPy no \
                 Python desta máquina, e ele não está instalado."
                .to_owned(),
            Self::TempoEsgotado => format!(
                "O valor exato vem da solução do conceito: a IDE tentou resolver a SUA \
                 equação e desistiu depois de {} segundos. Equações não lineares costumam \
                 não ter solução fechada.",
                TETO.as_secs()
            ),
            Self::NaoResolve => "O valor exato vem da solução do conceito: a IDE tentou \
                 resolver a SUA equação e o resolvedor respondeu que não sabe. Isso é comum \
                 e não quer dizer que a sua equação esteja errada."
                .to_owned(),
            Self::RespostaIlegivel { motivo } => format!(
                "O valor exato vem da solução do conceito: a IDE resolveu a SUA equação e \
                 não conseguiu ler a resposta ({motivo})."
            ),
        }
    }
}

/// A pergunta que se faz ao oraculo.
///
/// Os nomes vem da LIGACAO que o usuario fez, nunca da posicao na formula — e
/// e' por isso que eles viajam nomeados ate' aqui.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pergunta {
    /// A formula como ele digitou.
    pub formula: String,
    /// Qual variavel dela e' a posicao.
    pub estado: String,
    /// Qual variavel e' a velocidade. `None` na primeira ordem.
    pub derivada: Option<String>,
    /// Qual variavel e' o tempo, quando ele a usou.
    pub tempo: Option<String>,
    /// Os parametros, como TEXTO decimal — a racionalizacao e' do outro lado,
    /// onde `Rational("4.905")` e' exato e `nsimplify(4.905)` e' adivinhacao.
    pub parametros: Vec<(String, String)>,
    /// 1 ou 2.
    pub ordem: u8,
    /// `y(0)`, como texto decimal.
    pub y0: String,
    /// `y'(0)`, como texto decimal. `None` na primeira ordem.
    pub dy0: Option<String>,
}

/// A resposta crua do processo, antes do portao.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RespostaCrua {
    ok: bool,
    #[serde(default)]
    expression: String,
    #[serde(default)]
    sympy: String,
    #[serde(default)]
    reason: String,
}

/// O programa que roda do outro lado.
///
/// Ele e' pequeno de proposito: tudo que pode ser decidido em Rust e' decidido
/// em Rust. O que so' o `SymPy` sabe fazer e' `sympify`, `nsimplify`, `dsolve` e
/// `sstr` — e e' exatamente isso que esta aqui.
const PROGRAMA: &str = r#"
import json, sys

def responde(objeto):
    sys.stdout.write(json.dumps(objeto))
    sys.stdout.flush()
    raise SystemExit(0)

try:
    import sympy as sp
except Exception as erro:
    responde({"ok": False, "reason": "noSympy"})

try:
    pedido = json.load(sys.stdin)
except Exception as erro:
    responde({"ok": False, "reason": "badRequest"})

try:
    t = sp.Symbol("t")
    y = sp.Function("y")
    # `nsimplify(..., rational=True)` troca todo literal float por Rational.
    # Sem isso o `dsolve` cai em RecursionError depois de 4 segundos.
    corpo = sp.nsimplify(sp.sympify(pedido["formula"]), rational=True)
    troca = {}
    if pedido.get("estado"):
        troca[sp.Symbol(pedido["estado"])] = y(t)
    if pedido.get("derivada"):
        troca[sp.Symbol(pedido["derivada"])] = y(t).diff(t)
    if pedido.get("tempo"):
        troca[sp.Symbol(pedido["tempo"])] = t
    for nome, valor in pedido.get("parametros", []):
        troca[sp.Symbol(nome)] = sp.Rational(valor)
    # `simultaneous=True` impede que a troca de `x` por `y(t)` seja reprocessada
    # pela troca seguinte — substituicao em cascata daria outra equacao.
    lado = corpo.subs(troca, simultaneous=True)
    ordem = int(pedido["ordem"])
    equacao = sp.Eq(y(t).diff(t, ordem), lado)
    inicio = {y(0): sp.Rational(pedido["y0"])}
    if ordem == 2:
        inicio[y(t).diff(t).subs(t, 0)] = sp.Rational(pedido["dy0"])
    solucao = sp.dsolve(equacao, y(t), ics=inicio)
except NotImplementedError:
    responde({"ok": False, "reason": "cannotSolve"})
except RecursionError:
    responde({"ok": False, "reason": "cannotSolve"})
except SystemExit:
    raise
except Exception as erro:
    responde({"ok": False, "reason": "failed"})

if isinstance(solucao, (list, tuple)):
    responde({"ok": False, "reason": "cannotSolve"})

responde({"ok": True, "expression": sp.sstr(solucao.rhs), "sympy": sp.__version__})
"#;

/// Pergunta ao oraculo. Uma ida, uma resposta, ou um motivo.
///
/// # Errors
///
/// Devolve [`NaoSei`] quando a ferramenta falta, quando ela passa do
/// [`TETO`], quando ela responde que nao sabe, ou quando a resposta nao passa
/// no portao do `exmex`.
pub fn resolver(config: &Config, pergunta: &Pergunta) -> Result<Solucao, NaoSei> {
    let corpo = serde_json::to_string(pergunta).map_err(|_| NaoSei::NaoResolve)?;
    let bruta = executar(config, &corpo)?;
    let resposta: RespostaCrua =
        serde_json::from_str(bruta.trim()).map_err(|_| NaoSei::NaoResolve)?;
    if !resposta.ok {
        return Err(match resposta.reason.as_str() {
            "noSympy" => NaoSei::SemSympy,
            _ => NaoSei::NaoResolve,
        });
    }
    let expressao = portao(&resposta.expression)?;
    Ok(Solucao {
        expressao,
        resolvedor: format!("SymPy {}", resposta.sympy),
    })
}

/// Roda o processo com o [`TETO`] e devolve o que ele escreveu.
fn executar(config: &Config, corpo: &str) -> Result<String, NaoSei> {
    let mut filho = Command::new(&config.interpretador)
        .arg("-c")
        .arg(PROGRAMA)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| NaoSei::SemPython)?;

    if let Some(mut entrada) = filho.stdin.take() {
        // Erro de escrita nao e' motivo proprio: o processo morreu, e o que
        // interessa e' o que ele respondeu (ou nao).
        let _ = entrada.write_all(corpo.as_bytes());
    }

    let Some(saida) = filho.stdout.take() else {
        let _ = filho.kill();
        let _ = filho.wait();
        return Err(NaoSei::NaoResolve);
    };
    // Thread leitora: a resposta e' pequena, mas um traceback nao e', e um pipe
    // cheio com ninguem lendo trava o filho ate' o teto — que seria um
    // "TempoEsgotado" mentindo sobre a causa.
    let leitor = std::thread::spawn(move || {
        let mut texto = String::new();
        let _ = BufReader::new(saida).read_to_string(&mut texto);
        texto
    });

    let limite = Instant::now() + config.teto;
    loop {
        // Terminou, ou nao da' mais para perguntar: nos dois casos o que
        // interessa esta no pipe, e quem decide se e' resposta e' o parse.
        if !matches!(filho.try_wait(), Ok(None)) {
            break;
        }
        if Instant::now() >= limite {
            let _ = filho.kill();
            let _ = filho.wait();
            let _ = leitor.join();
            return Err(NaoSei::TempoEsgotado);
        }
        std::thread::sleep(RESPIRO);
    }
    Ok(leitor.join().unwrap_or_default())
}

/// O PORTAO: so' passa expressao que o `exmex` le' e cuja unica variavel e' `t`.
///
/// A segunda metade e' a que defende contra a falha SILENCIOSA. Medido em
/// 2026-09-06: `0.1*cos(pi*t)` tem `var_names() == ["pi", "t"]`, e preencher o
/// vetor pelo tamanho — que e' o que um codigo descuidado faz — devolve
/// `0,086232` onde a resposta e' `0,100000`, **sem erro nenhum**.
///
/// # Errors
///
/// [`NaoSei::RespostaIlegivel`] quando o `exmex` recusa a expressao ou quando
/// ela tem variavel que nao seja `t`.
pub fn portao(expressao: &str) -> Result<String, NaoSei> {
    // `**` so' significa potencia; a troca e' segura por construcao.
    let convertida = expressao.replace("**", "^");
    let analisada = exmex::parse::<f64>(&convertida).map_err(|erro| NaoSei::RespostaIlegivel {
        motivo: format!("o avaliador recusou a expressão: {erro}"),
    })?;
    let variaveis = analisada.var_names();
    let so_o_tempo = variaveis.len() == 1 && variaveis[0] == VARIAVEL_DO_TEMPO;
    if !so_o_tempo {
        return Err(NaoSei::RespostaIlegivel {
            motivo: format!(
                "a solução depende de {} além do tempo",
                variaveis
                    .iter()
                    .filter(|nome| *nome != VARIAVEL_DO_TEMPO)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
    Ok(convertida)
}

/// Avalia a solucao do oraculo num instante.
///
/// # Errors
///
/// [`NaoSei::RespostaIlegivel`] quando a avaliacao falha — o portao ja' provou
/// que a expressao tem uma variavel so', entao aqui so' sobra erro numerico.
pub fn avaliar(solucao: &Solucao, t: f64) -> Result<f64, NaoSei> {
    let analisada =
        exmex::parse::<f64>(&solucao.expressao).map_err(|erro| NaoSei::RespostaIlegivel {
            motivo: format!("o avaliador recusou a expressão: {erro}"),
        })?;
    analisada
        .eval(&[t])
        .map_err(|erro| NaoSei::RespostaIlegivel {
            motivo: format!("a solução não pôde ser avaliada: {erro}"),
        })
}
