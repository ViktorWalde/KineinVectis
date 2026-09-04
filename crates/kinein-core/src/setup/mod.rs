//! Como instalar o que falta — passo a passo OFICIAL, para a distro detectada.
//!
//! POR QUE ESTE DOMINIO EXISTE (2026-09-04). Ideia do autor: *"mostrar os
//! comandos de acordo com as distros mais famosas, auxiliando dentro da propria
//! IDE o usuario iniciante, mostrar o comando oficial e dar o link oficial para
//! ele conferir caso haja desconfianca"*.
//!
//! Ele resolve um buraco concreto que a IDE tinha: quando uma biblioteca nao
//! esta' instalada, o painel escrevia `FetchContent` com uma URL do `GitHub` no
//! `CMakeLists.txt` do autor — e o proprio autor questionou isso. O caminho que
//! DISPENSA a URL e' instalar o pacote no sistema, e ate' aqui a IDE sabia
//! dizer "instale" sem dizer COMO.
//!
//! # A decisao anterior, e por que a premissa mudou
//!
//! O `tools.rs` §`install_command` registrou que sugerir instalacao por distro
//! seria *"palpite disfarcado de instrucao"*. **A decisao continua valendo
//! contra palpite.** O que este dominio faz e' o oposto:
//!
//! ```text
//! nao ADIVINHA a distro   le' /etc/os-release, que a propria distribuicao
//!                         escreve sobre si
//! nao INVENTA o comando   copia da documentacao OFICIAL do projeto, com a
//!                         URL e a DATA em que foi conferida
//! nao EXECUTA nada        devolve texto; quem roda e' o autor
//! ```
//!
//! Sem fonte para uma familia, a entrada nao existe e a IDE diz isso — em vez
//! de traduzir um comando de outra distro, que e' exatamente o palpite proibido.

pub mod catalog;
pub mod distro;

use kinein_protocol::{SetupGuide, SetupStep, SetupToolInfo};

use crate::tools::ToolDetector;

/// O guia de instalacao de cada ferramenta, para ESTA maquina.
///
/// `installed` sai da mesma deteccao que o resto da IDE usa: o `PATH` e o bit
/// de execucao. Ferramenta ja' instalada continua na lista, com o guia junto —
/// quem quer conferir como reinstalar nao deveria ter de desinstalar antes.
#[must_use]
pub fn guides(detector: &ToolDetector) -> Vec<SetupToolInfo> {
    let atual = distro::detect();
    let familia = atual.family.to_string();

    catalog::TOOLS
        .iter()
        .map(|tool| SetupToolInfo {
            id: tool.id.to_owned(),
            name: tool.name.to_owned(),
            summary: tool.summary.to_owned(),
            website: tool.website.to_owned(),
            // A MESMA deteccao que o resto da IDE usa: `PATH` e bit de
            // execucao. Uma segunda forma de "esta instalado?" divergiria da
            // primeira, e o autor veria a IDE discordar de si mesma.
            installed: detector
                .detect(&crate::tools::ToolSpec {
                    id: tool.id,
                    display_name: tool.name,
                    binary: tool.probe_binary,
                    alternative_binary: None,
                    install_command: None,
                })
                .path
                .is_some(),
            guide: catalog::GUIDES
                .iter()
                .find(|guia| guia.tool == tool.id && guia.family == familia)
                .map(|guia| SetupGuide {
                    family: guia.family.to_owned(),
                    source_url: guia.source_url.to_owned(),
                    checked_at: guia.checked_at.to_owned(),
                    steps: guia
                        .steps
                        .iter()
                        .map(|passo| SetupStep {
                            explanation: passo.explanation.to_owned(),
                            // O `format!` de varias linhas do catalogo deixa
                            // espaco duplo na juncao; a UI mostra o comando
                            // para ser COPIADO, e espaco a mais quebra a copia.
                            command: normalizar(passo.command),
                        })
                        .collect(),
                }),
        })
        .collect()
}

/// A distro desta maquina, como a UI a mostra.
#[must_use]
pub fn current_distro() -> (String, String, String) {
    let atual = distro::detect();
    (atual.id, atual.pretty_name, atual.family.to_string())
}

/// Junta as quebras de linha do fonte num comando de uma linha so'.
fn normalizar(comando: &str) -> String {
    comando.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A regra que este dominio existe para cumprir: comando sem fonte nao
    /// entra. Se alguem acrescentar um guia sem URL ou sem data, este teste
    /// reprova antes de a IDE mostrar uma instrucao que ninguem conferiu.
    #[test]
    fn todo_guia_carrega_fonte_e_data() {
        for guia in catalog::GUIDES {
            assert!(
                guia.source_url.starts_with("https://"),
                "{}/{}: fonte ausente ou nao e' URL",
                guia.tool,
                guia.family
            );
            assert!(
                guia.checked_at.len() == 10 && guia.checked_at.starts_with("202"),
                "{}/{}: data de conferencia invalida `{}`",
                guia.tool,
                guia.family,
                guia.checked_at
            );
            assert!(
                !guia.steps.is_empty(),
                "{}/{}: guia sem passo nenhum",
                guia.tool,
                guia.family
            );
            for passo in guia.steps {
                assert!(
                    !passo.explanation.is_empty(),
                    "{}/{}: passo sem explicacao — o comando sozinho nao ensina",
                    guia.tool,
                    guia.family
                );
            }
        }
    }

    /// Guia so' existe para ferramenta que existe.
    #[test]
    fn nenhum_guia_aponta_para_ferramenta_desconhecida() {
        for guia in catalog::GUIDES {
            assert!(
                catalog::TOOLS.iter().any(|tool| tool.id == guia.tool),
                "guia de `{}` sem ferramenta correspondente",
                guia.tool
            );
        }
    }

    /// O comando vai para a tela para ser COPIADO: quebra de linha do fonte
    /// Rust nao pode virar espaco duplo no meio de uma URL.
    #[test]
    fn o_comando_sai_em_uma_linha_so() {
        assert_eq!(
            normalizar("sudo apt   install  -y   grafana"),
            "sudo apt install -y grafana"
        );
        for guia in catalog::GUIDES {
            for passo in guia.steps {
                let comando = normalizar(passo.command);
                assert!(!comando.contains("  "), "espaco duplo em `{comando}`");
                assert!(!comando.contains('\n'), "quebra de linha em `{comando}`");
            }
        }
    }
}
