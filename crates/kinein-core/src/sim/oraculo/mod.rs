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

use std::time::Duration;

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
pub(super) const RESPIRO: Duration = Duration::from_millis(20);

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
    /// Ele nao respondeu "nao sei": ele QUEBROU.
    ///
    /// A distincao importa para quem le: "nao sei resolver" e' comum e nao diz
    /// nada sobre a equacao; "quebrou" e' outra coisa, e a frase nao pode
    /// prometer que esta tudo bem com o que voce escreveu.
    Falhou,
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
            Self::Falhou => "O valor exato vem da solução do conceito: a IDE tentou resolver \
                 a SUA equação e o resolvedor falhou — não é o mesmo que \"não sei resolver\"."
                .to_owned(),
            Self::RespostaIlegivel { motivo } => format!(
                "O valor exato vem da solução do conceito: a IDE resolveu a SUA equação e \
                 não conseguiu ler a resposta ({motivo})."
            ),
        }
    }
}

/// A EDO a resolver.
///
/// Os nomes vem da LIGACAO que o usuario fez, nunca da posicao na formula — e
/// e' por isso que eles viajam nomeados ate' aqui.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edo {
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

/// Uma equacao a conferir por UNIDADE.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PedidoDimensao {
    /// Como a tela chama esta equacao. Vazio na forma escalar, que so' tem uma;
    /// o id do componente na vetorial, que tem `n`.
    pub label: String,
    /// A formula como ele digitou.
    pub formula: String,
    /// A unidade DECLARADA de cada variavel, pela ligacao que ele fez.
    pub unidades: Vec<(String, String)>,
    /// A unidade do lado ESQUERDO: a do estado dividida pelo tempo elevado a
    /// ordem. E' o que transforma "os termos combinam entre si" em "a equacao
    /// e' daquela grandeza".
    pub esquerda: String,
}

/// Tudo que se pergunta numa IDA.
///
/// Uma ida so', e as duas perguntas juntas: o processo custa ~200 ms de
/// `import` antes de qualquer conta, e pagar isso duas vezes por corrida seria
/// desenho, nao acidente.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Consulta {
    /// A EDO, quando ha' uma para resolver.
    pub edo: Option<Edo>,
    /// Uma entrada por equacao a conferir.
    pub dimensoes: Vec<PedidoDimensao>,
}

/// O veredito de unidade de UMA equacao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Veredito {
    /// De que equacao e' este veredito.
    pub label: String,
    /// O que foi achado.
    pub achado: Achado,
    /// O detalhe, quando ha' um.
    pub detalhe: String,
}

/// O que a checagem de unidade achou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Achado {
    /// Os termos combinam entre si E com o lado esquerdo.
    Coerente,
    /// Dois termos da soma tem dimensoes diferentes.
    Incoerente,
    /// Os termos combinam, mas a equacao nao e' da grandeza do lado esquerdo.
    LadoErrado,
    /// `sin(x)` com `x` em metros. Nao e' fisica: e' erro que produz numero.
    ArgumentoComDimensao,
    /// O conceito declarou uma unidade que o vocabulario nao conhece.
    ///
    /// **Recusar e' obrigatorio.** Simbolo que o `SymPy` nao reconhece e'
    /// ADIMENSIONAL para ele, e o veredito sairia "coerente" sem nada
    /// reclamar — a falha silenciosa que este dominio persegue.
    UnidadeDesconhecida,
    /// A formula nao pode ser lida como expressao.
    Ilegivel,
}

/// A resposta de uma ida.
#[derive(Debug, Clone)]
pub struct Resposta {
    /// A solucao da EDO, quando ela foi pedida e saiu.
    pub solucao: Option<Solucao>,
    /// Por que a EDO nao saiu, quando foi pedida e nao saiu.
    pub sem_solucao: Option<NaoSei>,
    /// Um veredito por equacao pedida.
    pub dimensoes: Vec<Veredito>,
}

/// Uma LINHA da resposta, antes do portao.
///
/// O processo escreve duas: a checagem de unidade (barata) e a EDO (que pode
/// nao voltar). Ler linha a linha e' o que faz a primeira sobreviver ao teto
/// que mata a segunda.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinhaCrua {
    kind: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    sympy: String,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    expression: String,
    #[serde(default)]
    itens: Vec<DimensaoCrua>,
}

/// O que o outro lado devolve sobre uma equacao conferida.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DimensaoCrua {
    #[serde(default)]
    label: String,
    verdict: String,
    #[serde(default)]
    detail: String,
}

/// Pergunta ao oraculo. UMA ida, e as duas respostas.
///
/// # Errors
///
/// Devolve [`NaoSei`] quando a ferramenta falta ou quando ela passa do
/// [`TETO`] — falhas de TRANSPORTE, que atingem as duas perguntas. O que falha
/// por pergunta vem dentro da [`Resposta`].
pub fn perguntar(config: &Config, consulta: &Consulta) -> Result<Resposta, NaoSei> {
    let corpo = serde_json::to_string(consulta).map_err(|_| NaoSei::NaoResolve)?;
    let (bruta, esgotou) = processo::executar(config, &corpo)?;

    let mut sympy = String::new();
    let mut dimensoes = Vec::new();
    let mut solucao = None;
    let mut sem_solucao = None;
    let mut viu_edo = false;

    for texto in bruta.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(linha) = serde_json::from_str::<LinhaCrua>(texto) else {
            continue;
        };
        if !linha.sympy.is_empty() {
            sympy.clone_from(&linha.sympy);
        }
        match linha.kind.as_str() {
            "fatal" => {
                return Err(if linha.reason == "noSympy" {
                    NaoSei::SemSympy
                } else {
                    NaoSei::Falhou
                });
            }
            "dimensoes" => {
                dimensoes = linha
                    .itens
                    .into_iter()
                    .map(|d| Veredito {
                        label: d.label,
                        achado: match d.verdict.as_str() {
                            "coherent" => Achado::Coerente,
                            "incoherent" => Achado::Incoerente,
                            "wrongSide" => Achado::LadoErrado,
                            "dimensionalArgument" => Achado::ArgumentoComDimensao,
                            "unknownUnit" => Achado::UnidadeDesconhecida,
                            _ => Achado::Ilegivel,
                        },
                        detalhe: d.detail,
                    })
                    .collect();
            }
            "edo" => {
                viu_edo = true;
                if linha.ok {
                    match portao(&linha.expression) {
                        Ok(expressao) => {
                            solucao = Some(Solucao {
                                expressao,
                                resolvedor: format!("SymPy {sympy}"),
                            });
                        }
                        Err(motivo) => sem_solucao = Some(motivo),
                    }
                } else {
                    sem_solucao = Some(if linha.reason == "cannotSolve" {
                        NaoSei::NaoResolve
                    } else {
                        NaoSei::Falhou
                    });
                }
            }
            _ => {}
        }
    }

    // A EDO foi pedida e a linha dela nao veio: ou o teto matou o processo no
    // meio do `dsolve`, ou ele morreu. O veredito de unidade, que ja' chegou,
    // continua valendo — e' para isso que sao duas linhas.
    if consulta.edo.is_some() && !viu_edo && sem_solucao.is_none() {
        sem_solucao = Some(if esgotou {
            NaoSei::TempoEsgotado
        } else {
            NaoSei::Falhou
        });
    }
    if dimensoes.is_empty() && solucao.is_none() && sem_solucao.is_none() {
        return Err(NaoSei::Falhou);
    }

    Ok(Resposta {
        solucao,
        sem_solucao,
        dimensoes,
    })
}

mod portao;
mod processo;
mod programa;

pub use portao::{avaliar, portao};
