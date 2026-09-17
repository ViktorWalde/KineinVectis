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
mod catalog_embedded;
pub mod distro;

use kinein_protocol::{SetupGuide, SetupStep, SetupToolInfo};

use crate::tools::ToolDetector;

/// Todas as ferramentas: o catalogo geral (banco, Python, Grafana) e o de
/// embarcados (A5 do roadmaps/41), na ordem em que os arquivos as listam.
fn all_tools() -> impl Iterator<Item = &'static catalog::Tool> {
    catalog::TOOLS.iter().chain(catalog_embedded::TOOLS.iter())
}

/// Todos os guias, dos dois catalogos.
fn all_guides() -> impl Iterator<Item = &'static catalog::Guide> {
    catalog::GUIDES
        .iter()
        .chain(catalog_embedded::GUIDES.iter())
}

/// O guia de instalacao de cada ferramenta, para ESTA maquina.
///
/// `installed` sai da mesma deteccao que o resto da IDE usa: o `PATH` e o bit
/// de execucao. Ferramenta ja' instalada continua na lista, com o guia junto —
/// quem quer conferir como reinstalar nao deveria ter de desinstalar antes.
/// Desde 2026-09-17 a lista une os dois catalogos (geral e embarcados).
#[must_use]
pub fn guides(detector: &ToolDetector) -> Vec<SetupToolInfo> {
    let atual = distro::detect();
    let familia = atual.family.to_string();

    all_tools()
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
            // A familia desta maquina primeiro; o guia `any` (fonte oficial
            // agnostica de distro: pipx, uv tool) quando ela nao tem um proprio.
            guide: all_guides()
                .find(|guia| guia.tool == tool.id && guia.family == familia)
                .or_else(|| all_guides().find(|guia| guia.tool == tool.id && guia.family == "any"))
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
        for guia in all_guides() {
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
        for guia in all_guides() {
            assert!(
                all_tools().any(|tool| tool.id == guia.tool),
                "guia de `{}` sem ferramenta correspondente",
                guia.tool
            );
        }
    }

    /// A familia desta maquina vence o guia `any`; o `any` so' vale quando a
    /// fonte oficial e' agnostica de distro (uv, basedpyright) — e uma
    /// ferramenta sem guia nenhum continua na lista, so' com o site.
    #[test]
    fn a_familia_da_maquina_vence_o_guia_agnostico() {
        let familia = distro::detect().family.to_string();
        let detector = ToolDetector::with_search_path(std::env::temp_dir());
        let lista = guides(&detector);
        let ruff = lista.iter().find(|t| t.id == "ruff").unwrap();
        let guia = ruff.guide.as_ref().expect("ruff tem guia any");
        let esperado = if matches!(familia.as_str(), "arch" | "suse") {
            familia.as_str()
        } else {
            "any"
        };
        assert_eq!(guia.family, esperado);
        let uv = lista.iter().find(|t| t.id == "uv").unwrap();
        assert_eq!(uv.guide.as_ref().unwrap().family, "any");
        assert_eq!(
            uv.guide.as_ref().unwrap().steps[0].command,
            "pipx install uv"
        );
        // Ferramenta com guia so' para algumas familias: fora delas, sem guia
        // e com o site.
        let pipx = lista.iter().find(|t| t.id == "pipx").unwrap();
        assert!(pipx.website.starts_with("https://pipx.pypa.io"));
        if !matches!(familia.as_str(), "redhat" | "debian") {
            assert!(pipx.guide.is_none());
        }
    }

    /// O catalogo de embarcados (A5, 2026-09-17): ids unicos entre os dois
    /// catalogos; toda ferramenta de embarcado tem site; os guias que a
    /// conferencia registrou existem para as familias certas — e NAO existem
    /// onde o indice nao tinha o pacote (tio e picotool no Arch; picotool e
    /// espflash no Fedora).
    #[test]
    fn o_catalogo_de_embarcados_tem_ids_unicos_e_so_as_familias_conferidas() {
        let mut ids: Vec<&str> = all_tools().map(|t| t.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "id duplicado entre os catalogos");
        for esperado in [
            "esptool",
            "mpremote",
            "espflash",
            "probe-rs",
            "picotool",
            "dfu-util",
            "tio",
            "picocom",
            "arm-none-eabi",
            "qemu-embedded",
            "openocd",
        ] {
            let tool = all_tools()
                .find(|t| t.id == esperado)
                .unwrap_or_else(|| panic!("{esperado}"));
            assert!(tool.website.starts_with("https://"));
            assert!(
                all_guides().any(|g| g.tool == esperado),
                "{esperado} sem guia nenhum"
            );
        }
        let tem =
            |tool: &str, family: &str| all_guides().any(|g| g.tool == tool && g.family == family);
        assert!(!tem("tio", "arch") && !tem("picotool", "arch"));
        assert!(!tem("picotool", "redhat") && !tem("espflash", "redhat"));
        assert!(tem("probe-rs", "any") && tem("probe-rs", "debian"));
        assert!(tem("esptool", "any") && tem("mpremote", "any"));
        // Nenhum guia de embarcado traduz comando de outra familia.
        for guia in catalog_embedded::GUIDES {
            for passo in guia.steps {
                let c = passo.command;
                match guia.family {
                    "debian" => assert!(!c.contains("dnf ") && !c.contains("pacman "), "{c}"),
                    "redhat" => assert!(!c.contains("apt ") && !c.contains("pacman "), "{c}"),
                    "arch" => assert!(!c.contains("apt ") && !c.contains("dnf "), "{c}"),
                    _ => assert!(
                        !c.contains("apt ") && !c.contains("dnf ") && !c.contains("pacman "),
                        "{c}"
                    ),
                }
            }
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
        for guia in all_guides() {
            for passo in guia.steps {
                let comando = normalizar(passo.command);
                assert!(!comando.contains("  "), "espaco duplo em `{comando}`");
                assert!(!comando.contains('\n'), "quebra de linha em `{comando}`");
            }
        }
    }
}
