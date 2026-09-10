//! A formula do usuario: inspecionar, checar contra o conceito, e avaliar.
//!
//! Este arquivo carrega a decisao que governa o dominio inteiro
//! (`docs/arquitetura/34-simulacao-por-conceito.md` §2.1, autor 2026-09-05):
//! **nada e adivinhado**. Em particular, a ligacao entre uma variavel da formula
//! e a grandeza que ela representa e' DADO QUE O USUARIO INFORMA, nunca
//! resultado de casar nomes iguais.
//!
//! ## Por que casar por nome seria errado, e nao apenas impreciso
//!
//! O `exmex` devolve as variaveis em ordem ALFABETICA, e nao na ordem em que
//! aparecem na formula: `"a*t + v0"` e `"v0 + a*t"` devolvem os dois
//! `["a","t","v0"]`. O `eval()` recebe um slice POSICIONAL. Montar esse slice
//! pela ordem de leitura da formula da' um numero — o comprimento bate, os
//! tipos batem — e a fisica sai errada, **sem erro nenhum**. E' a falha
//! silenciosa que a §8 do `ARCHITECTURE.md` proibe, e que este repositorio ja'
//! pagou tres vezes.
//!
//! A defesa nao e' disciplina: e' [`vetor_por_ligacao`], que monta o slice
//! percorrendo `var_names()` e buscando a ligacao de cada nome. Trocar duas
//! variaveis de papel TEM de mudar o resultado, e o teste que prova isso e'
//! mutacao, nao comentario.

use std::collections::{BTreeMap, BTreeSet};

use exmex::prelude::*;
use kinein_protocol::{
    SimBinding, SimCheckIssue, SimCheckResult, SimConcept, SimEvaluateResult, SimStep, SimValue,
};

/// O aviso que o autor pediu na tela, e nao em nota de rodape
/// (`arquitetura/34` §5.1).
pub const AVISO: &str = "Conceito certo e formula valida NAO significam resultado certo: uma \
                         formula errada dentro do conceito certo usa as variaveis certas e \
                         produz um numero — e o numero esta' errado. A IDE nao tem como saber.";

/// As variaveis que a formula usa, na ordem que o avaliador reporta.
///
/// A ordem e' ALFABETICA e nada a jusante pode depender dela; ela sai daqui
/// para a UI montar a tabela de ligacao, e so'.
pub fn inspecionar(formula: &str) -> Result<Vec<String>, String> {
    match exmex::parse::<f64>(formula) {
        Ok(expressao) => Ok(expressao.var_names().to_vec()),
        Err(erro) => Err(mensagem_de_parse(&erro.to_string())),
    }
}

/// Traduz o erro do crate para uma frase que pode ir a' tela.
///
/// O texto do `exmex` tem **endereco de ponteiro** dentro — 356 caracteres com
/// `0x...` no caso medido (ADR-0006, armadilha 4). Repassa-lo seria mostrar
/// memoria ao usuario, entao a IDE classifica e escreve a propria mensagem,
/// como ja' faz com `secretRequired` e com o `code` do `requestFailed`.
fn mensagem_de_parse(bruto: &str) -> String {
    let baixa = bruto.to_lowercase();
    if baixa.contains("empty string") {
        "A formula esta' vazia.".to_string()
    } else if baixa.contains("last element cannot be an operator") {
        "A formula termina num operador: falta o que vem depois dele.".to_string()
    } else if baixa.contains("binary operator") {
        "Ha' dois operadores seguidos, e falta um valor entre eles.".to_string()
    } else if baixa.contains("parenthes") {
        "Os parenteses nao fecham.".to_string()
    } else {
        "A formula nao pode ser lida. Confira operadores, parenteses e nomes de \
         funcao (por exemplo `sin`, e nao `sen`)."
            .to_string()
    }
}

/// Confronta a formula, a ligacao do usuario e o conceito escolhido.
///
/// Devolve TODOS os problemas, nao o primeiro: digitar e' um laco ao vivo, e uma
/// lista que encolhe enquanto se digita ensina mais que um erro por vez.
#[must_use]
pub fn checar(conceito: &SimConcept, formula: &str, ligacoes: &[SimBinding]) -> SimCheckResult {
    let variaveis = match inspecionar(formula) {
        Ok(v) => v,
        Err(message) => {
            return SimCheckResult {
                ok: false,
                issues: vec![SimCheckIssue::ParseFailed { message }],
                variables: Vec::new(),
                caveat: None,
            };
        }
    };

    let mut problemas = Vec::new();
    let conhecidas: BTreeSet<&str> = conceito.quantities.iter().map(|q| q.id.as_str()).collect();
    let na_formula: BTreeSet<&str> = variaveis.iter().map(String::as_str).collect();

    // A ligacao aponta para grandeza que o conceito declara?
    // E para variavel que a formula tem?
    let mut por_variavel: BTreeMap<&str, &str> = BTreeMap::new();
    let mut usos: BTreeMap<&str, usize> = BTreeMap::new();
    for ligacao in ligacoes {
        let (variavel, grandeza) = (ligacao.variable.as_str(), ligacao.quantity.as_str());
        if !conhecidas.contains(grandeza) {
            problemas.push(SimCheckIssue::UnknownQuantity {
                variable: variavel.to_string(),
                quantity: grandeza.to_string(),
            });
            continue;
        }
        if !na_formula.contains(variavel) {
            problemas.push(SimCheckIssue::VariableNotInFormula {
                variable: variavel.to_string(),
            });
            continue;
        }
        por_variavel.insert(variavel, grandeza);
        *usos.entry(grandeza).or_default() += 1;
    }

    // Duas variaveis dizendo ser a mesma grandeza.
    for (grandeza, quantas) in &usos {
        if *quantas > 1 {
            problemas.push(SimCheckIssue::DuplicateQuantity {
                quantity: (*grandeza).to_string(),
            });
        }
    }

    // Variavel da formula sem papel. E' tambem por aqui que o `pi` minusculo
    // aparece: o `exmex` o trata como variavel livre, entao ele chega aqui como
    // pergunta na tela em vez de incognita silenciosa (ADR-0006, armadilha 2).
    for variavel in &variaveis {
        if !por_variavel.contains_key(variavel.as_str()) {
            problemas.push(SimCheckIssue::UnboundVariable {
                variable: variavel.clone(),
            });
        }
    }

    // Grandeza obrigatoria do conceito sem nenhuma variavel ligada a ela.
    let ligadas: BTreeSet<&str> = por_variavel.values().copied().collect();
    for grandeza in &conceito.quantities {
        if grandeza.required && !ligadas.contains(grandeza.id.as_str()) {
            problemas.push(SimCheckIssue::MissingQuantity {
                quantity: grandeza.id.clone(),
                label: grandeza.label.clone(),
            });
        }
    }

    let ok = problemas.is_empty();
    SimCheckResult {
        ok,
        issues: problemas,
        variables: variaveis,
        caveat: ok.then(|| AVISO.to_string()),
    }
}

/// Erro de avaliacao, separado do erro de checagem porque sao momentos
/// diferentes: aqui a formula ja passou.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErroAvaliacao {
    /// A checagem nao passou; a avaliacao nem tenta.
    NaoChecada,
    /// Uma grandeza ligada ficou sem valor preenchido.
    ///
    /// Campo vazio e' recusa, nunca zero implicito: o principio da §2.1 diz que
    /// a IDE nao preenche por voce, e um zero suposto e' preenchimento.
    SemValor {
        /// A grandeza que ficou em branco.
        grandeza: String,
    },
    /// O resultado nao e' um numero utilizavel.
    ///
    /// O `exmex` aceita `1/0` e devolve `Ok(inf)`, `x/x` com `x=0` devolve
    /// `Ok(NaN)`, `log(0)` devolve `-inf` e `sqrt(-1)` devolve `NaN` — todos
    /// sem erro (ADR-0006, armadilha 3). Uma simulacao que segue com `NaN`
    /// entrega uma curva de nada com cara de resultado, entao o dominio corta
    /// aqui.
    NaoFinito {
        /// Como o valor se apresentou: `inf`, `-inf` ou `NaN`.
        valor: String,
    },
}

/// Monta o vetor de avaliacao PELA LIGACAO do usuario.
///
/// Esta funcao e' a defesa contra a armadilha 1 do ADR-0006, e a razao de ela
/// existir separada e' poder ser mutada num teste: trocar a ligacao de duas
/// variaveis tem de mudar o numero que sai.
fn vetor_por_ligacao(
    nomes: &[String],
    ligacoes: &[SimBinding],
    valores: &[SimValue],
) -> Result<Vec<f64>, ErroAvaliacao> {
    let grandeza_de: BTreeMap<&str, &str> = ligacoes
        .iter()
        .map(|l| (l.variable.as_str(), l.quantity.as_str()))
        .collect();
    let valor_de: BTreeMap<&str, f64> = valores
        .iter()
        .map(|v| (v.quantity.as_str(), v.value))
        .collect();

    nomes
        .iter()
        .map(|nome| {
            let grandeza = grandeza_de
                .get(nome.as_str())
                .ok_or(ErroAvaliacao::NaoChecada)?;
            valor_de
                .get(grandeza)
                .copied()
                .ok_or_else(|| ErroAvaliacao::SemValor {
                    grandeza: (*grandeza).to_string(),
                })
        })
        .collect()
}

/// Avalia a forma ALGEBRICA uma vez, e devolve a trilha da conta junto.
///
/// A trilha nao e' enfeite: e' o "passo a passo" possivel. Nenhuma ferramenta
/// auditavel narra a resolucao algebrica de uma EDO (`roadmaps/31` §13), entao
/// o que a IDE mostra e a **substituicao numerica** — literalmente a conta que
/// rodou, impossivel de divergir do resultado porque E' o resultado.
pub fn avaliar(
    conceito: &SimConcept,
    formula: &str,
    ligacoes: &[SimBinding],
    valores: &[SimValue],
) -> Result<SimEvaluateResult, ErroAvaliacao> {
    let checagem = checar(conceito, formula, ligacoes);
    if !checagem.ok {
        return Err(ErroAvaliacao::NaoChecada);
    }
    let expressao = exmex::parse::<f64>(formula).map_err(|_| ErroAvaliacao::NaoChecada)?;
    let nomes = expressao.var_names().to_vec();
    let vetor = vetor_por_ligacao(&nomes, ligacoes, valores)?;

    let valor = expressao
        .eval(&vetor)
        .map_err(|_| ErroAvaliacao::NaoChecada)?;
    if !valor.is_finite() {
        return Err(ErroAvaliacao::NaoFinito {
            valor: if valor.is_nan() {
                "NaN".to_string()
            } else if valor > 0.0 {
                "inf".to_string()
            } else {
                "-inf".to_string()
            },
        });
    }

    Ok(SimEvaluateResult {
        value: valor,
        steps: trilha(conceito, formula, &nomes, ligacoes, &vetor, valor),
        unit: None,
    })
}

/// A trilha da substituicao, na ordem em que o painel a mostra.
fn trilha(
    conceito: &SimConcept,
    formula: &str,
    nomes: &[String],
    ligacoes: &[SimBinding],
    vetor: &[f64],
    valor: f64,
) -> Vec<SimStep> {
    let mut passos = vec![SimStep {
        kind: "formula".to_string(),
        text: format!("{} = {formula}", conceito.name),
    }];

    let rotulo_de: BTreeMap<&str, (&str, &str)> = conceito
        .quantities
        .iter()
        .map(|q| (q.id.as_str(), (q.label.as_str(), q.unit.as_str())))
        .collect();
    let grandeza_de: BTreeMap<&str, &str> = ligacoes
        .iter()
        .map(|l| (l.variable.as_str(), l.quantity.as_str()))
        .collect();

    for (nome, numero) in nomes.iter().zip(vetor) {
        let grandeza = grandeza_de.get(nome.as_str()).copied().unwrap_or("");
        let (rotulo, unidade) = rotulo_de.get(grandeza).copied().unwrap_or(("", ""));
        passos.push(SimStep {
            kind: "binding".to_string(),
            text: if unidade.is_empty() {
                format!("{nome} = {numero}   ({rotulo})")
            } else {
                format!("{nome} = {numero} {unidade}   ({rotulo})")
            },
        });
    }

    let mut substituida = formula.to_string();
    for (nome, numero) in nomes.iter().zip(vetor) {
        substituida = substitui_variavel(&substituida, nome, *numero);
    }
    passos.push(SimStep {
        kind: "substitution".to_string(),
        text: substituida,
    });
    passos.push(SimStep {
        kind: "result".to_string(),
        text: format!("= {valor}"),
    });
    passos
}

/// Troca uma variavel pelo numero dela, respeitando fronteira de identificador.
///
/// Substituicao ingenua por texto trocaria o `v` de `v0` e mostraria uma conta
/// que nao e' a que rodou — o painel deixaria de ser honesto, que e' a unica
/// coisa que ele tem a oferecer.
fn substitui_variavel(texto: &str, nome: &str, numero: f64) -> String {
    let bytes = texto.as_bytes();
    let mut saida = String::with_capacity(texto.len());
    let mut i = 0;
    while i < texto.len() {
        let bate = texto[i..].starts_with(nome)
            && !i
                .checked_sub(1)
                .and_then(|antes| bytes.get(antes))
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
            && !bytes
                .get(i + nome.len())
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
        if bate {
            saida.push_str(&numero.to_string());
            i += nome.len();
        } else {
            let ch = texto[i..].chars().next().unwrap_or('\0');
            saida.push(ch);
            i += ch.len_utf8();
        }
    }
    saida
}
