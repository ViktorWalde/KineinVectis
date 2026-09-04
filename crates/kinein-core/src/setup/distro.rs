//! Qual distribuicao esta rodando — lido, nao adivinhado.
//!
//! POR QUE ISTO EXISTE (2026-09-04). O `tools.rs` §`install_command` tem uma
//! decisao registrada contraria a sugerir instalacao: *"sugerir `pacman` numa
//! Fedora, ou traduzir nome de pacote por distro, seria palpite disfarcado de
//! instrucao"*. A decisao esta CERTA para o que ela julgava — adivinhar.
//!
//! O que muda a premissa, a pedido do autor em 2026-09-04, sao duas coisas:
//!
//! ```text
//! 1. nao se ADIVINHA a distro: le-se /etc/os-release, que e' o arquivo que a
//!    propria distribuicao escreve sobre si (padrao systemd/freedesktop)
//! 2. nao se INVENTA o comando: ele e' copiado da documentacao OFICIAL do
//!    projeto, com a URL junto para o autor conferir
//! ```
//!
//! Ler + citar nao e' palpite. E o que a IDE mostra continua sendo TEXTO: quem
//! decide rodar e' o autor, e ele ve' cada comando antes.

use std::fmt;

/// Familia de gerenciador de pacotes.
///
/// A familia, e nao a distro exata, porque e' ela que determina o comando:
/// Ubuntu, Debian, Linux Mint e `Pop`!_OS usam o mesmo `apt` com os mesmos
/// pacotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// Debian, Ubuntu e derivadas — `apt`.
    Debian,
    /// Fedora, `RHEL`, Rocky, `AlmaLinux`, `CentOS` — `dnf`.
    RedHat,
    /// Arch, Manjaro, `EndeavourOS` — `pacman`.
    Arch,
    /// `openSUSE` e `SLES` — `zypper`.
    Suse,
    /// Nao reconhecida. A IDE mostra o link oficial e NAO inventa comando.
    Unknown,
}

impl fmt::Display for Family {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Debian => "debian",
            Self::RedHat => "redhat",
            Self::Arch => "arch",
            Self::Suse => "suse",
            Self::Unknown => "unknown",
        })
    }
}

/// O que se sabe da distribuicao desta maquina.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distro {
    /// `ID` do `os-release` (`fedora`, `ubuntu`, `debian`, ...).
    pub id: String,
    /// `PRETTY_NAME`, para a UI mostrar como a distro se chama.
    pub pretty_name: String,
    /// `VERSION_ID`, quando declarado.
    pub version_id: Option<String>,
    /// Familia de gerenciador de pacotes.
    pub family: Family,
}

/// Le a distribuicao de `/etc/os-release`.
///
/// Arquivo ausente ou ilegivel devolve `Unknown` com nome vazio — e essa e' a
/// resposta honesta. Chutar "provavelmente Ubuntu" seria voltar ao palpite que
/// a decisao do `tools.rs` proibiu.
#[must_use]
pub fn detect() -> Distro {
    // `/etc/os-release` e' o caminho do padrao; `/usr/lib` e' o fallback que a
    // propria especificacao define para sistemas com `/etc` vazio.
    let conteudo = std::fs::read_to_string("/etc/os-release")
        .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    parse(&conteudo)
}

/// Interpreta o conteudo de um `os-release`.
#[must_use]
pub fn parse(conteudo: &str) -> Distro {
    let mut id = String::new();
    let mut id_like = String::new();
    let mut pretty_name = String::new();
    let mut version_id = None;

    for linha in conteudo.lines() {
        let Some((chave, valor)) = linha.split_once('=') else {
            continue;
        };
        // O padrao permite aspas; `NAME="Fedora Linux"` e `ID=fedora` convivem.
        let valor = valor.trim().trim_matches('"').trim_matches('\'').to_owned();
        match chave.trim() {
            "ID" => id = valor,
            "ID_LIKE" => id_like = valor,
            "PRETTY_NAME" => pretty_name = valor,
            "VERSION_ID" => version_id = Some(valor),
            _ => {}
        }
    }

    Distro {
        family: family_of(&id, &id_like),
        id,
        pretty_name,
        version_id,
    }
}

/// Familia a partir do `ID` e do `ID_LIKE`.
///
/// O `ID_LIKE` existe exatamente para isto e e' quem cobre as derivadas: o
/// `Pop`!_OS diz `ID_LIKE="ubuntu debian"` sem que este arquivo precise conhecer
/// o `Pop`!_OS. Consultar a lista fixa PRIMEIRO e o `ID_LIKE` depois mantem a
/// resposta certa para as distros que se declaram derivadas de duas coisas.
fn family_of(id: &str, id_like: &str) -> Family {
    const DEBIAN: [&str; 6] = [
        "debian",
        "ubuntu",
        "linuxmint",
        "pop",
        "elementary",
        "raspbian",
    ];
    const REDHAT: [&str; 6] = ["fedora", "rhel", "centos", "rocky", "almalinux", "ol"];
    const ARCH: [&str; 4] = ["arch", "manjaro", "endeavouros", "garuda"];
    const SUSE: [&str; 3] = ["opensuse", "opensuse-tumbleweed", "sles"];

    let candidatos: Vec<&str> = std::iter::once(id)
        .chain(id_like.split_whitespace())
        .collect();
    for candidato in candidatos {
        let normalizado = candidato.to_ascii_lowercase();
        if DEBIAN.contains(&normalizado.as_str()) {
            return Family::Debian;
        }
        if REDHAT.contains(&normalizado.as_str()) || normalizado.starts_with("rhel") {
            return Family::RedHat;
        }
        if ARCH.contains(&normalizado.as_str()) {
            return Family::Arch;
        }
        if SUSE.contains(&normalizado.as_str()) || normalizado.starts_with("opensuse") {
            return Family::Suse;
        }
    }
    Family::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O `os-release` REAL desta maquina, copiado em 2026-09-04.
    const FEDORA: &str = "NAME=\"Fedora Linux\"\n\
                          VERSION=\"44 (Workstation Edition)\"\n\
                          ID=fedora\n\
                          VERSION_ID=44\n\
                          PRETTY_NAME=\"Fedora Linux 44 (Workstation Edition)\"\n";

    #[test]
    fn le_a_fedora_desta_maquina() {
        let distro = parse(FEDORA);
        assert_eq!(distro.id, "fedora");
        assert_eq!(distro.family, Family::RedHat);
        assert_eq!(distro.version_id.as_deref(), Some("44"));
        assert!(distro.pretty_name.contains("Fedora Linux 44"));
    }

    /// O `ID_LIKE` e' quem cobre derivada sem este arquivo conhece-la.
    #[test]
    fn derivada_cai_na_familia_pelo_id_like() {
        let pop = parse("ID=pop\nID_LIKE=\"ubuntu debian\"\nPRETTY_NAME=\"Pop!_OS 22.04\"\n");
        assert_eq!(pop.family, Family::Debian);

        let desconhecida_derivada = parse("ID=minhadistro\nID_LIKE=arch\n");
        assert_eq!(desconhecida_derivada.family, Family::Arch);
    }

    #[test]
    fn debian_11_e_ubuntu_sao_a_mesma_familia() {
        assert_eq!(
            parse("ID=debian\nVERSION_ID=\"11\"\n").family,
            Family::Debian
        );
        assert_eq!(
            parse("ID=ubuntu\nVERSION_ID=\"24.04\"\n").family,
            Family::Debian
        );
    }

    /// Chutar "provavelmente Ubuntu" seria voltar ao palpite que a decisao do
    /// `tools.rs` proibiu.
    #[test]
    fn distro_desconhecida_nao_vira_chute() {
        assert_eq!(parse("ID=algo-que-nao-existe\n").family, Family::Unknown);
        assert_eq!(parse("").family, Family::Unknown);
        assert_eq!(parse("").id, "");
    }

    #[test]
    fn aspas_do_padrao_saem_do_valor() {
        let distro = parse("ID=\"ubuntu\"\nVERSION_ID='24.04'\n");
        assert_eq!(distro.id, "ubuntu");
        assert_eq!(distro.version_id.as_deref(), Some("24.04"));
    }
}
