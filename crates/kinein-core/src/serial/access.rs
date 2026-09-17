//! E2 do `integracoes/38` §6: permissao POR CANAL.
//!
//! O que falta, medido, e o passo OFICIAL para cada canal. A IDE nunca roda
//! `sudo`: cada passo e' escrito no terminal dela, como o painel de
//! instalacao faz.
//!
//! Tres canais, tres formas diferentes de "sem acesso" (38 §4.3):
//!
//! ```text
//! serial        o no' tty: grupo dono (dialout na maioria, uucp no Arch) OU
//!               a ACL que o udev poe pela tag `uaccess` (e' assim que a
//!               sessao grafica tem acesso sem grupo nenhum). Medido por
//!               access(2) — a ACL conta, o `stat` sozinho mentiria
//! probe         a regra udev das sondas (69-probe-rs.rules; a distro pode
//!               ja' ter posto a do OpenOCD/ST-Link em /usr/lib/udev/rules.d)
//! modemManager  rodando E a porta candidata E sem regra de ignorar: pode
//!               segurar a porta por segundos depois do plug (medido em
//!               2026-09-11: 4 s no ESP32)
//! ```
//!
//! # As fontes, conferidas em 2026-09-17
//!
//! - grupo: ESP-IDF "Establish Serial Connection" — `sudo usermod -a -G
//!   dialout $USER` (Arch: `uucp`) e "re-login". A pagina do Arch Wiki nao
//!   respondeu ao fetch (Anubis); a do ESP-IDF sim.
//! - sonda: probe.rs "Probe Setup" — baixar `69-probe-rs.rules` de
//!   `https://probe.rs/files/69-probe-rs.rules` para `/etc/udev/rules.d`,
//!   depois `udevadm control --reload` e `udevadm trigger` (citados).
//! - `ModemManager`: a forma da regra de ignorar vem dos arquivos que o
//!   proprio pacote instala (`/usr/lib/udev/rules.d/77-mm-*.rules`,
//!   `ATTRS{idVendor}=="…", ATTRS{idProduct}=="…", ENV{ID_MM_DEVICE_IGNORE}="1"`)
//!   e do `80-mm-candidate.rules` que marca toda tty como candidata — logo
//!   a regra do usuario tem de rodar ANTES do 80 (numero menor). A pagina
//!   de docs do freedesktop devolveu 403 ao fetch; a fonte e' o pacote.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    AccessChannel, AccessChannelKind, AccessFix, SerialAccessResult, SerialPortInfo, SetupStep,
};

use super::{Ambiente, list_in, parse_udev_properties};

const CONFERIDO_EM: &str = "2026-09-17";
const FONTE_GRUPO: &str = "https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/establish-serial-connection.html";
const FONTE_SONDA: &str = "https://probe.rs/docs/getting-started/probe-setup/";
const FONTE_MM: &str =
    "/usr/lib/udev/rules.d/80-mm-candidate.rules e 77-mm-*.rules (pacote ModemManager)";

/// Onde as regras udev moram, na ordem em que o udev as le. As do usuario
/// em `/etc`; as da distro em `/usr/lib` (e `/lib` onde ainda nao ha' link).
pub const RULES_DIRS: [&str; 3] = [
    "/etc/udev/rules.d",
    "/usr/lib/udev/rules.d",
    "/lib/udev/rules.d",
];

/// O diagnostico desta maquina.
#[must_use]
pub fn diagnose(device: Option<&str>) -> SerialAccessResult {
    let dirs: Vec<PathBuf> = RULES_DIRS.iter().map(PathBuf::from).collect();
    diagnose_in(
        &Ambiente {
            sys_class_tty: Path::new("/sys/class/tty"),
            dev: Path::new("/dev"),
            proc_root: Path::new("/proc"),
            etc_group: Path::new("/etc/group"),
            udev_props: &super::udevadm_properties,
        },
        &dirs,
        device,
    )
}

/// O diagnostico lendo do ambiente dado (o teste monta `/sys`, `/dev`,
/// `/proc`, `/etc/group` e as pastas de regras falsos).
#[must_use]
pub fn diagnose_in(
    ambiente: &Ambiente<'_>,
    rules_dirs: &[PathBuf],
    device: Option<&str>,
) -> SerialAccessResult {
    let grupos = grupos_do_usuario(ambiente.proc_root, ambiente.etc_group);
    let mut channels = Vec::new();
    for port in list_in(ambiente)
        .ports
        .into_iter()
        .filter(|p| device.is_none_or(|d| d == p.device))
    {
        let props = (ambiente.udev_props)(Path::new(&port.device))
            .map(|texto| parse_udev_properties(&texto));
        let tags = props
            .as_ref()
            .and_then(|p| p.get("TAGS").cloned())
            .unwrap_or_default();
        channels.push(canal_serial(&port, &grupos, &tags));
        if let Some(mm) = &port.modem_manager {
            channels.push(canal_modem_manager(
                &port,
                mm.running,
                mm.candidate,
                mm.ignored,
            ));
        }
    }
    channels.push(canal_sonda(rules_dirs));
    SerialAccessResult { channels }
}

fn canal_serial(port: &SerialPortInfo, grupos: &[String], tags: &str) -> AccessChannel {
    let acesso = &port.access;
    let grupo = acesso.group.as_deref();
    let no_grupo = grupo.is_some_and(|g| grupos.iter().any(|meu| meu == g));
    let por_acl = tags.split(':').any(|t| t == "uaccess");
    let mut canal = AccessChannel {
        kind: AccessChannelKind::Serial,
        device: Some(port.device.clone()),
        ok: acesso.readable_writable,
        detail: format!(
            "{} e' {}{}; voce {} do grupo",
            port.device,
            acesso.mode,
            grupo
                .map(|g| format!(" do grupo `{g}`"))
                .unwrap_or_default(),
            if no_grupo {
                "faz parte"
            } else {
                "nao faz parte"
            }
        ),
        problem: None,
        fix: None,
        distro_did_it: None,
    };
    if acesso.readable_writable {
        if !no_grupo && por_acl {
            canal.distro_did_it = Some(
                "o udev deu acesso ao usuario da sessao pela tag `uaccess` (ACL no no'), sem \
                 grupo nenhum"
                    .to_owned(),
            );
        }
        return canal;
    }
    canal.problem = Some(
        acesso
            .hint
            .clone()
            .unwrap_or_else(|| "sem leitura/escrita no no'".to_owned()),
    );
    canal.fix = Some(match grupo {
        Some(g @ ("dialout" | "uucp" | "plugdev" | "tty")) => AccessFix {
            steps: vec![
                SetupStep {
                    explanation: format!(
                        "Entra no grupo `{g}`, dono do no'. Vale para toda porta serial desta \
                         maquina."
                    ),
                    command: format!("sudo usermod -a -G {g} $USER"),
                },
                SetupStep {
                    explanation: "Saia e entre na sessao (re-login): o grupo novo so' vale para \
                                  processos iniciados depois."
                        .to_owned(),
                    command: "id -nG".to_owned(),
                },
            ],
            source_url: FONTE_GRUPO.to_owned(),
            checked_on: CONFERIDO_EM.to_owned(),
        },
        _ => AccessFix {
            steps: vec![
                SetupStep {
                    explanation: format!(
                        "O no' nao pertence a um grupo de usuarios; uma regra udev com `uaccess` \
                         da' acesso ao usuario da sessao grafica a esta ponte ({}:{}), como o \
                         70-uaccess.rules do systemd faz para outros dispositivos.",
                        port.vid, port.pid
                    ),
                    command: format!(
                        "printf 'SUBSYSTEM==\"tty\", ATTRS{{idVendor}}==\"{}\", ATTRS{{idProduct}}==\"{}\", TAG+=\"uaccess\"\\n' | sudo tee /etc/udev/rules.d/70-kinein-serial-{}-{}.rules",
                        port.vid, port.pid, port.vid, port.pid
                    ),
                },
                SetupStep {
                    explanation: "Recarrega as regras e as aplica ao que ja' esta' plugado."
                        .to_owned(),
                    command: "sudo udevadm control --reload && sudo udevadm trigger".to_owned(),
                },
            ],
            source_url: "/usr/lib/udev/rules.d/70-uaccess.rules (systemd)".to_owned(),
            checked_on: CONFERIDO_EM.to_owned(),
        },
    });
    canal
}

fn canal_modem_manager(
    port: &SerialPortInfo,
    running: bool,
    candidate: bool,
    ignored: bool,
) -> AccessChannel {
    let ameaca = running && candidate && !ignored;
    let mut canal = AccessChannel {
        kind: AccessChannelKind::ModemManager,
        device: Some(port.device.clone()),
        ok: !ameaca,
        detail: format!(
            "ModemManager {}; a porta {} candidata (ID_MM_CANDIDATE) e {} regra de ignorar",
            if running {
                "esta' rodando"
            } else {
                "nao esta' rodando"
            },
            if candidate { "e'" } else { "nao e'" },
            if ignored { "tem" } else { "nao tem" }
        ),
        problem: None,
        fix: None,
        distro_did_it: None,
    };
    if !ameaca {
        if ignored {
            canal.distro_did_it =
                Some("uma regra udev ja' marca esta ponte com ID_MM_DEVICE_IGNORE".to_owned());
        }
        return canal;
    }
    canal.problem = Some(
        "o ModemManager pode abrir esta porta por alguns segundos depois de plugar, e a \
         gravacao/monitor falham enquanto ele a segura"
            .to_owned(),
    );
    canal.fix = Some(AccessFix {
        steps: vec![
            SetupStep {
                explanation: format!(
                    "Uma regra udev diz ao ModemManager para ignorar esta ponte ({}:{}). O \
                     numero 77 a poe ANTES do 80-mm-candidate.rules, que marca toda tty como \
                     candidata — e' a forma das regras que o proprio pacote instala.",
                    port.vid, port.pid
                ),
                command: format!(
                    "printf 'ATTRS{{idVendor}}==\"{}\", ATTRS{{idProduct}}==\"{}\", ENV{{ID_MM_DEVICE_IGNORE}}=\"1\"\\n' | sudo tee /etc/udev/rules.d/77-mm-kinein-{}-{}.rules",
                    port.vid, port.pid, port.vid, port.pid
                ),
            },
            SetupStep {
                explanation: "Recarrega as regras e as aplica ao que ja' esta' plugado; a \
                              propriedade nova aparece no `udevadm info`."
                    .to_owned(),
                command: "sudo udevadm control --reload && sudo udevadm trigger".to_owned(),
            },
        ],
        source_url: FONTE_MM.to_owned(),
        checked_on: CONFERIDO_EM.to_owned(),
    });
    canal
}

/// A regra das sondas: nao depende de porta — e' da maquina.
fn canal_sonda(rules_dirs: &[PathBuf]) -> AccessChannel {
    let achadas = regras_de_sonda(rules_dirs);
    let mut canal = AccessChannel {
        kind: AccessChannelKind::Probe,
        device: None,
        ok: !achadas.is_empty(),
        detail: if achadas.is_empty() {
            "nenhuma regra udev de sonda (probe-rs, OpenOCD, ST-Link) nas pastas de regras"
                .to_owned()
        } else {
            format!("regras udev de sonda: {}", achadas.join(", "))
        },
        problem: None,
        fix: None,
        distro_did_it: None,
    };
    if canal.ok {
        // A primeira pasta e' a do usuario (/etc); o resto e' da distro.
        let do_usuario = rules_dirs
            .first()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let da_distro: Vec<&String> = achadas
            .iter()
            .filter(|r| do_usuario.is_empty() || !r.starts_with(&do_usuario))
            .collect();
        if !da_distro.is_empty() {
            canal.distro_did_it = Some(format!(
                "a distro ja' instalou: {}",
                da_distro
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        return canal;
    }
    canal.problem = Some(
        "sem regra udev, uma sonda (ST-Link, J-Link, CMSIS-DAP, o USB-JTAG do ESP32-C3/S3) so' \
         abre como root"
            .to_owned(),
    );
    canal.fix = Some(AccessFix {
        steps: vec![
            SetupStep {
                explanation: "Baixa o 69-probe-rs.rules do probe.rs para /etc/udev/rules.d (a \
                              pagina manda baixar o arquivo e po-lo la'; o comando e' a \
                              transcricao)."
                    .to_owned(),
                command: "curl -fsSL https://probe.rs/files/69-probe-rs.rules | sudo tee \
                          /etc/udev/rules.d/69-probe-rs.rules >/dev/null"
                    .to_owned(),
            },
            SetupStep {
                explanation: "\"to ensure new rules are used\" — como a pagina escreve.".to_owned(),
                command: "sudo udevadm control --reload".to_owned(),
            },
            SetupStep {
                explanation: "\"to apply rules to already-connected devices\" — idem.".to_owned(),
                command: "sudo udevadm trigger".to_owned(),
            },
        ],
        source_url: FONTE_SONDA.to_owned(),
        checked_on: CONFERIDO_EM.to_owned(),
    });
    canal
}

/// Os arquivos de regra que falam de sondas, na ordem das pastas.
fn regras_de_sonda(rules_dirs: &[PathBuf]) -> Vec<String> {
    let mut achadas = Vec::new();
    // `/lib` costuma ser link para `/usr/lib` (usrmerge): a mesma pasta nao
    // conta duas vezes.
    let mut vistas: Vec<PathBuf> = Vec::new();
    for dir in rules_dirs {
        let real = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.clone());
        if vistas.contains(&real) {
            continue;
        }
        vistas.push(real);
        let Ok(entradas) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut nomes: Vec<String> = entradas
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| {
                let baixo = n.to_ascii_lowercase();
                std::path::Path::new(&baixo)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("rules"))
                    && (baixo.contains("probe-rs")
                        || baixo.contains("probe_rs")
                        || baixo.contains("openocd")
                        || baixo.contains("stlink")
                        || baixo.contains("jlink")
                        || baixo.contains("cmsis"))
            })
            .collect();
        nomes.sort();
        achadas.extend(nomes.into_iter().map(|n| dir.join(n).display().to_string()));
    }
    achadas
}

/// Os grupos do processo (`<proc>/self/status`, linha `Groups:`), por nome.
fn grupos_do_usuario(proc_root: &Path, etc_group: &Path) -> Vec<String> {
    let Ok(status) = std::fs::read_to_string(proc_root.join("self/status")) else {
        return Vec::new();
    };
    let Ok(grupos) = std::fs::read_to_string(etc_group) else {
        return Vec::new();
    };
    let gids: Vec<u32> = status
        .lines()
        .find_map(|l| l.strip_prefix("Groups:"))
        .map(|l| {
            l.split_whitespace()
                .filter_map(|g| g.parse().ok())
                .collect()
        })
        .unwrap_or_default();
    grupos
        .lines()
        .filter_map(|linha| {
            let mut campos = linha.split(':');
            let nome = campos.next()?;
            let _senha = campos.next()?;
            let gid: u32 = campos.next()?.parse().ok()?;
            gids.contains(&gid).then(|| nome.to_owned())
        })
        .collect()
}
