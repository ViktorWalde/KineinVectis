//! E2 do `integracoes/38` §6: permissao POR CANAL.
//!
//! O que falta, medido, e o passo OFICIAL para cada canal. A IDE nunca roda
//! `sudo` automaticamente: cada comando depende de clique explícito no terminal, como o painel de
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
    AccessChannel, AccessChannelKind, AccessFix, SerialAccessResult, SerialAccessState,
    SerialPortInfo, SetupStep,
};

use super::{Environment, list_in};

const CHECKED_ON: &str = "2026-09-17";
const GROUP_CHECKED_ON: &str = "2026-10-20";
const GROUP_SOURCE: &str = "https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/establish-serial-connection.html";
const PROBE_SOURCE: &str = "https://probe.rs/docs/getting-started/probe-setup/";
const MODEM_MANAGER_SOURCE: &str =
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
        &Environment {
            sys_class_tty: Path::new("/sys/class/tty"),
            dev: Path::new("/dev"),
            proc_root: Path::new("/proc"),
            etc_group: Path::new("/etc/group"),
            accounts_file: Path::new("/etc/passwd"),
            udev_props: &super::udevadm_properties,
        },
        &dirs,
        device,
    )
}

/// O diagnostico lendo do environment dado (o teste monta `/sys`, `/dev`,
/// `/proc`, `/etc/group` e as pastas de regras falsos).
#[must_use]
pub fn diagnose_in(
    environment: &Environment<'_>,
    rules_dirs: &[PathBuf],
    device: Option<&str>,
) -> SerialAccessResult {
    let mut channels = Vec::new();
    for port in list_in(environment)
        .ports
        .into_iter()
        .filter(|p| device.is_none_or(|d| d == p.device))
    {
        channels.push(canal_serial(&port));
        if let Some(mm) = &port.modem_manager {
            channels.push(canal_modem_manager(
                &port,
                mm.running,
                mm.candidate,
                mm.ignored,
            ));
        }
    }
    if let Some(device) = device.filter(|_| channels.is_empty()) {
        channels.push(AccessChannel {
            kind: AccessChannelKind::Serial,
            device: Some(device.to_owned()),
            ok: false,
            detail: "Porta ausente da lista atual de dispositivos USB".to_owned(),
            problem: Some("A porta selecionada não foi encontrada. Reconecte a placa e clique em Verificar novamente.".to_owned()),
            fix: Some(AccessFix {
                steps: vec![SetupStep {
                    explanation: "Reconecte a placa e clique em Verificar novamente. A porta pode aparecer com outro nome; confira a seleção antes de operar.".to_owned(),
                    command: String::new(),
                }],
                source_url: GROUP_SOURCE.to_owned(),
                checked_on: GROUP_CHECKED_ON.to_owned(),
            }),
            distro_did_it: None,
        });
    }
    channels.push(canal_sonda(rules_dirs));
    SerialAccessResult { channels }
}

fn canal_serial(port: &SerialPortInfo) -> AccessChannel {
    let access = &port.access;
    let mut channel = AccessChannel {
        kind: AccessChannelKind::Serial,
        device: Some(port.device.clone()),
        ok: access.readable_writable,
        detail: format!(
            "{} · {} · grupo `{}`",
            port.device,
            access.mode,
            access.group.as_deref().unwrap_or("desconhecido")
        ),
        problem: access.hint.clone(),
        fix: None,
        distro_did_it: None,
    };
    if access.readable_writable {
        return channel;
    }
    let steps = match access.state {
        SerialAccessState::GroupRequired => vec![
            SetupStep {
                explanation: "Cadastre sua conta no grupo da porta. O terminal pedirá a senha de administrador; confira o resultado antes de continuar.".to_owned(),
                command: format!("sudo usermod -a -G {} -- \"$(id -un)\"", access.group.as_deref().unwrap_or_default()),
            },
            SetupStep {
                explanation: "Depois de o cadastro terminar sem erro, salve seu trabalho, saia e entre na sessão do sistema. Reabra a IDE e clique em Verificar novamente.".to_owned(),
                command: String::new(),
            },
        ],
        SerialAccessState::SessionRefreshRequired => vec![SetupStep {
            explanation: "O cadastro já está feito. Salve seu trabalho e saia da sessão do sistema; entre novamente e reabra a IDE. Reiniciar apenas a IDE nesta mesma sessão pode manter os grupos antigos. Depois clique em Verificar novamente.".to_owned(),
            command: String::new(),
        }],
        SerialAccessState::Unavailable | SerialAccessState::Unsupported => vec![SetupStep {
            explanation: access.hint.clone().unwrap_or_default(),
            command: String::new(),
        }],
        _ => vec![SetupStep {
            explanation: "Confira a identidade e as permissões da porta no terminal. A causa pode incluir modo, ACL ou política do sistema; o cadastro em grupo não foi identificado como solução. Se necessário, peça ao administrador para revisar o acesso.".to_owned(),
            command: format!("id && ls -l -- '{}'", port.device.replace('\'', "'\\''")),
        }],
    };
    channel.fix = Some(AccessFix {
        steps,
        source_url: GROUP_SOURCE.to_owned(),
        checked_on: GROUP_CHECKED_ON.to_owned(),
    });
    channel
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
                "está rodando"
            } else {
                "não está rodando"
            },
            if candidate { "é" } else { "não é" },
            if ignored { "tem" } else { "não tem" }
        ),
        problem: None,
        fix: None,
        distro_did_it: None,
    };
    if !ameaca {
        if ignored {
            canal.distro_did_it =
                Some("uma regra udev já marca esta ponte com ID_MM_DEVICE_IGNORE".to_owned());
        }
        return canal;
    }
    canal.problem = Some(
        "o ModemManager pode abrir esta porta por alguns segundos depois de plugar, e a \
         gravação/monitor falham enquanto ele a segura"
            .to_owned(),
    );
    canal.fix = Some(AccessFix {
        steps: vec![
            SetupStep {
                explanation: format!(
                    "Uma regra udev diz ao ModemManager para ignorar esta ponte ({}:{}). O \
                     numero 77 a poe ANTES do 80-mm-candidate.rules, que marca toda tty como \
                     candidata — é a forma das regras que o próprio pacote instala.",
                    port.vid, port.pid
                ),
                command: format!(
                    "printf 'ATTRS{{idVendor}}==\"{}\", ATTRS{{idProduct}}==\"{}\", ENV{{ID_MM_DEVICE_IGNORE}}=\"1\"\\n' | sudo tee /etc/udev/rules.d/77-mm-kinein-{}-{}.rules",
                    port.vid, port.pid, port.vid, port.pid
                ),
            },
            SetupStep {
                explanation: "Recarrega as regras e as aplica ao que já está plugado; a \
                              propriedade nova aparece no `udevadm info`."
                    .to_owned(),
                command: "sudo udevadm control --reload && sudo udevadm trigger".to_owned(),
            },
        ],
        source_url: MODEM_MANAGER_SOURCE.to_owned(),
        checked_on: CHECKED_ON.to_owned(),
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
                "a distro já instalou: {}",
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
        "sem regra udev, uma sonda (ST-Link, J-Link, CMSIS-DAP, o USB-JTAG do ESP32-C3/S3) só \
         abre como root"
            .to_owned(),
    );
    canal.fix = Some(AccessFix {
        steps: vec![
            SetupStep {
                explanation: "Baixa o 69-probe-rs.rules do probe.rs para /etc/udev/rules.d (a \
                              página manda baixar o arquivo e po-lo lá; o comando é a \
                              transcrição)."
                    .to_owned(),
                command: "curl -fsSL https://probe.rs/files/69-probe-rs.rules | sudo tee \
                          /etc/udev/rules.d/69-probe-rs.rules >/dev/null"
                    .to_owned(),
            },
            SetupStep {
                explanation: "\"to ensure new rules are used\" — como a página escreve.".to_owned(),
                command: "sudo udevadm control --reload".to_owned(),
            },
            SetupStep {
                explanation: "\"to apply rules to already-connected devices\" — idem.".to_owned(),
                command: "sudo udevadm trigger".to_owned(),
            },
        ],
        source_url: PROBE_SOURCE.to_owned(),
        checked_on: CHECKED_ON.to_owned(),
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
        let real = crate::platform::canonicalize(dir).unwrap_or_else(|_| dir.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recovery_does_not_repeat_registration_for_a_stale_session() {
        for (state, expected_commands) in [
            (SerialAccessState::SessionRefreshRequired, 0),
            (SerialAccessState::GroupRequired, 1),
            (SerialAccessState::PermissionDenied, 1),
        ] {
            let port: SerialPortInfo = serde_json::from_value(json!({
                "device": "/dev/ttyUSB0", "kind": "usbUartBridge", "vid": "10c4", "pid": "ea60",
                "access": { "state": state, "readableWritable": false, "mode": "crw-rw----", "group": "dialout", "hint": "core diagnosis" }
            })).unwrap();
            let channel = canal_serial(&port);
            assert!(!channel.ok);
            assert_eq!(channel.problem.as_deref(), Some("core diagnosis"));
            let steps = channel.fix.unwrap().steps;
            assert_eq!(
                steps.iter().filter(|step| !step.command.is_empty()).count(),
                expected_commands
            );
            assert_eq!(
                steps.iter().any(|step| step.command.contains("usermod")),
                state == SerialAccessState::GroupRequired
            );
            if state == SerialAccessState::SessionRefreshRequired {
                assert!(steps[0].explanation.contains("Salve seu trabalho"));
                assert!(steps[0].explanation.contains("sessão do sistema"));
            }
        }
    }
}
