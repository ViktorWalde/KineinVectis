//! Portas seriais USB: o que esta em `/dev` desta maquina, e quem mais a segura.
//!
//! Nasceu em 2026-09-11 (E1 do `DocsPublic/integracoes/38` §6), no dia em que um
//! ESP32 chegou a mesa. A porta serial e' o canal que TODA familia bare metal
//! compartilha — bootloader de ROM, console e as linhas DTR/RTS de reset — e
//! por isso e' a fundacao do monitor UART e do "Gravar".
//!
//! # O que este modulo NAO faz, e e' decisao
//!
//! Ele nunca ABRE a porta. Abrir um tty aciona DTR/RTS na maioria das pontes
//! (`CP210x`, CH340, FTDI) e isso RESETA a placa — um `serial.list` que resetasse
//! o firmware a cada abertura do painel seria um defeito, nao uma feature. A
//! permissao e' medida com `access(2)`, que honra ACL (e' assim que
//! `TAG+="uaccess"` da' acesso ao usuario da sessao sem grupo nenhum).
//!
//! # De onde vem cada coisa
//!
//! ```text
//! quais portas      /sys/class/tty/ttyUSB* e ttyACM* (ABI documentada do kernel)
//! VID:PID, nomes    subindo de /sys/class/tty/<n>/device ate' o diretorio USB
//!                   que tem `idVendor` (Documentation/ABI/testing/sysfs-bus-usb)
//! driver            /sys/class/tty/<n>/device/driver -> cp210x, ch341, cdc_acm
//! by-id             /dev/serial/by-id/* cujo alvo e' a porta
//! ModemManager      `udevadm info -q property -n /dev/<n>`: ID_MM_CANDIDATE e
//!                   ID_MM_DEVICE_IGNORE — medido em 2026-09-11 que o MM examinou
//!                   o ESP32 4 s depois do plug
//! familia           tabela VID:PID -> o que o ELO e' (nunca o chip atras de uma
//!                   ponte); fontes no `integracoes/38` §5
//! ```

pub mod access;
pub mod files;
pub mod identify;
pub mod job;
pub mod monitor;

use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;

use kinein_protocol::{
    ModemManagerState, SerialAccess, SerialListResult, SerialPortInfo, SerialPortKind,
};

/// De onde ler. Parametrizado para o teste montar um `/sys` e um `/dev` falsos.
pub struct Ambiente<'a> {
    /// `/sys/class/tty`.
    pub sys_class_tty: &'a Path,
    /// `/dev`.
    pub dev: &'a Path,
    /// `/proc` — para saber se ha' um `ModemManager` rodando.
    pub proc_root: &'a Path,
    /// `/etc/group` — para dar nome ao gid da porta.
    pub etc_group: &'a Path,
    /// As propriedades udev da porta (`udevadm info -q property -n <dev>`), ou
    /// `None` quando nao ha' como perguntar.
    pub udev_props: &'a dyn Fn(&Path) -> Option<String>,
}

impl std::fmt::Debug for Ambiente<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ambiente")
            .field("sys_class_tty", &self.sys_class_tty)
            .field("dev", &self.dev)
            .finish_non_exhaustive()
    }
}

/// Lista as portas seriais USB desta maquina.
#[must_use]
pub fn list() -> SerialListResult {
    list_in(&Ambiente {
        sys_class_tty: Path::new("/sys/class/tty"),
        dev: Path::new("/dev"),
        proc_root: Path::new("/proc"),
        etc_group: Path::new("/etc/group"),
        udev_props: &udevadm_properties,
    })
}

/// Lista as portas lendo do ambiente dado.
#[must_use]
pub fn list_in(ambiente: &Ambiente<'_>) -> SerialListResult {
    let mm_rodando = modem_manager_rodando(ambiente.proc_root);
    let ports: Vec<SerialPortInfo> = enumerar(ambiente.sys_class_tty)
        .into_iter()
        .filter_map(|nome| descrever(ambiente, &nome, mm_rodando))
        .collect();
    let hint = ports.is_empty().then(|| {
        "nenhuma porta serial USB (ttyUSB*/ttyACM*) apareceu. Placa desligada, cabo so' de \
         carga, ou a ponte precisa de driver — `lsusb` mostra o que o USB ve."
            .to_owned()
    });
    SerialListResult { ports, hint }
}

/// Os nomes `ttyUSB*`/`ttyACM*` em `/sys/class/tty`, em ordem natural
/// (`ttyUSB2` antes de `ttyUSB10`).
fn enumerar(sys_class_tty: &Path) -> Vec<String> {
    let Ok(entradas) = std::fs::read_dir(sys_class_tty) else {
        return Vec::new();
    };
    let mut nomes: Vec<String> = entradas
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| kind_for(n).is_some())
        .collect();
    nomes.sort_by_key(|n| {
        (
            n.trim_end_matches(|c: char| c.is_ascii_digit()).to_owned(),
            numero(n),
        )
    });
    nomes
}

fn numero(nome: &str) -> u32 {
    nome.trim_start_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .unwrap_or(0)
}

/// `ttyUSB` e' ponte, `ttyACM` e' CDC; o resto (`ttyS*`, `tty0`) nao e' USB.
fn kind_for(nome: &str) -> Option<SerialPortKind> {
    if nome.starts_with("ttyUSB") {
        Some(SerialPortKind::UsbUartBridge)
    } else if nome.starts_with("ttyACM") {
        Some(SerialPortKind::UsbCdc)
    } else {
        None
    }
}

/// O que o sysfs sabe do dispositivo USB por tras de um tty.
#[derive(Debug, Default)]
struct Identidade {
    vid: String,
    pid: String,
    manufacturer: Option<String>,
    product: Option<String>,
    serial: Option<String>,
    interface: Option<String>,
    driver: Option<String>,
}

fn descrever(ambiente: &Ambiente<'_>, nome: &str, mm_rodando: bool) -> Option<SerialPortInfo> {
    let kind = kind_for(nome)?;
    let identidade = identidade(&ambiente.sys_class_tty.join(nome).join("device"))?;
    let no = ambiente.dev.join(nome);
    let props = (ambiente.udev_props)(&no).map(|texto| parse_udev_properties(&texto));
    Some(SerialPortInfo {
        device: no.to_string_lossy().into_owned(),
        by_id: by_id(ambiente.dev, nome),
        kind,
        family: familia(&identidade.vid, &identidade.pid).map(str::to_owned),
        vid: identidade.vid,
        pid: identidade.pid,
        manufacturer: identidade.manufacturer,
        product: identidade.product,
        serial: identidade.serial,
        interface: identidade.interface,
        driver: identidade.driver,
        access: acesso(&no, ambiente.etc_group),
        modem_manager: props.map(|p| modem_manager(&p, mm_rodando)),
    })
}

/// Sobe de `<tty>/device` ate' achar o diretorio com `idVendor`. No caminho,
/// guarda o `bInterfaceNumber` da interface que atravessou. Sem `idVendor`
/// acima, nao e' USB — e nao entra na lista.
fn identidade(device: &Path) -> Option<Identidade> {
    let mut dir = std::fs::canonicalize(device).ok()?;
    let mut id = Identidade {
        driver: std::fs::read_link(device.join("driver"))
            .ok()
            .and_then(|alvo| alvo.file_name().map(|n| n.to_string_lossy().into_owned())),
        ..Identidade::default()
    };
    loop {
        if dir.join("idVendor").is_file() {
            id.vid = atributo(&dir, "idVendor")?;
            id.pid = atributo(&dir, "idProduct")?;
            id.manufacturer = atributo(&dir, "manufacturer");
            id.product = atributo(&dir, "product");
            id.serial = atributo(&dir, "serial");
            return Some(id);
        }
        if id.interface.is_none() {
            id.interface = atributo(&dir, "bInterfaceNumber");
        }
        // Saiu de `/sys/devices` sem achar USB: e' platform (ttyS*), nao entra.
        if dir.file_name().is_some_and(|n| n == "devices") {
            return None;
        }
        dir = dir.parent()?.to_path_buf();
    }
}

fn atributo(dir: &Path, nome: &str) -> Option<String> {
    let texto = std::fs::read_to_string(dir.join(nome)).ok()?;
    let texto = texto.trim();
    (!texto.is_empty()).then(|| texto.to_owned())
}

/// O link estavel em `/dev/serial/by-id` que aponta para esta porta.
fn by_id(dev: &Path, nome: &str) -> Option<String> {
    let pasta = dev.join("serial").join("by-id");
    let entradas = std::fs::read_dir(&pasta).ok()?;
    entradas.filter_map(Result::ok).find_map(|e| {
        let alvo = std::fs::read_link(e.path()).ok()?;
        (alvo.file_name()? == nome).then(|| e.path().to_string_lossy().into_owned())
    })
}

/// A permissao MEDIDA de um no' desta maquina (`/etc/group` real).
///
/// E' o que o `serial.identify` confere antes de entregar a porta ao esptool:
/// abrir o que vai falhar por permissao resetaria a placa por nada.
#[must_use]
pub fn acesso_de(no: &Path) -> SerialAccess {
    acesso(no, Path::new("/etc/group"))
}

/// `access(2)` para o veredito; `stat` para explicar o veredito.
fn acesso(no: &Path, etc_group: &Path) -> SerialAccess {
    let readable_writable = rustix::fs::access(
        no,
        rustix::fs::Access::READ_OK | rustix::fs::Access::WRITE_OK,
    )
    .is_ok();
    let meta = std::fs::metadata(no).ok();
    let mode = meta
        .as_ref()
        .map_or_else(|| "?".to_owned(), |m| modo_simbolico(m.mode()));
    let group = meta
        .as_ref()
        .and_then(|m| nome_do_grupo(etc_group, m.gid()));
    let hint = (!readable_writable).then(|| {
        group.as_ref().map_or_else(
            || {
                format!(
                    "{} e' {mode} e voce nao consegue ler nem escrever. Falta regra udev ou \
                     grupo; a IDE nao roda como root.",
                    no.display()
                )
            },
            |g| {
                format!(
                    "{} e' {mode} do grupo `{g}` e voce nao esta' nele (nem uma ACL te cobre). \
                     Passo oficial: `sudo usermod -aG {g} $USER` e sair/entrar da sessao. A IDE \
                     nao roda isso.",
                    no.display()
                )
            },
        )
    });
    SerialAccess {
        readable_writable,
        mode,
        group,
        hint,
    }
}

/// `crw-rw----`, como o `ls -l` imprime.
pub(crate) fn modo_simbolico(mode: u32) -> String {
    let tipo = match mode & 0o170_000 {
        0o020_000 => 'c',
        0o060_000 => 'b',
        0o120_000 => 'l',
        0o040_000 => 'd',
        _ => '-',
    };
    let mut s = String::with_capacity(10);
    s.push(tipo);
    for shift in [6u32, 3, 0] {
        let bits = (mode >> shift) & 0o7;
        s.push(if bits & 4 != 0 { 'r' } else { '-' });
        s.push(if bits & 2 != 0 { 'w' } else { '-' });
        s.push(if bits & 1 != 0 { 'x' } else { '-' });
    }
    s
}

/// `name:x:gid:members` — sem `getgrgid`, que e' FFI.
fn nome_do_grupo(etc_group: &Path, gid: u32) -> Option<String> {
    let texto = std::fs::read_to_string(etc_group).ok()?;
    texto.lines().find_map(|linha| {
        let mut campos = linha.split(':');
        let nome = campos.next()?;
        let _senha = campos.next()?;
        (campos.next()?.parse::<u32>().ok()? == gid).then(|| nome.to_owned())
    })
}

/// `udevadm info -q property -n <dev>`; `None` quando o binario falta.
pub(super) fn udevadm_properties(no: &Path) -> Option<String> {
    let saida = Command::new("udevadm")
        .args(["info", "-q", "property", "-n"])
        .arg(no)
        .output()
        .ok()?;
    saida
        .status
        .success()
        .then(|| String::from_utf8_lossy(&saida.stdout).into_owned())
}

/// `CHAVE=valor` por linha; linha sem `=` e' ignorada.
#[must_use]
pub fn parse_udev_properties(texto: &str) -> BTreeMap<String, String> {
    texto
        .lines()
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect()
}

fn modem_manager(props: &BTreeMap<String, String>, running: bool) -> ModemManagerState {
    let ligado = |chave: &str| props.get(chave).is_some_and(|v| v == "1");
    ModemManagerState {
        candidate: ligado("ID_MM_CANDIDATE"),
        ignored: ligado("ID_MM_DEVICE_IGNORE"),
        running,
    }
}

/// Ha' um processo chamado `ModemManager`? Lido de `/proc/*/comm`, sem spawn.
fn modem_manager_rodando(proc_root: &Path) -> bool {
    let Ok(entradas) = std::fs::read_dir(proc_root) else {
        return false;
    };
    entradas
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_digit())
        })
        .any(|e| {
            std::fs::read_to_string(e.path().join("comm"))
                .is_ok_and(|comm| comm.trim() == "ModemManager")
        })
}

/// O que o VID:PID diz sobre o ELO — e nunca sobre o chip atras de uma ponte.
///
/// Fontes (lidas em 2026-09-11, `integracoes/38` §5): `69-probe-rs.rules`
/// oficial e `contrib/60-openocd.rules` do `OpenOCD` (Espressif, ST-Link, FTDI),
/// `picoboot_connection.h` do picotool (Raspberry Pi), `usb.ids` (pontes).
#[must_use]
pub fn familia(vid: &str, pid: &str) -> Option<&'static str> {
    Some(
        match (
            vid.to_ascii_lowercase().as_str(),
            pid.to_ascii_lowercase().as_str(),
        ) {
            ("303a", "1001") => {
                "Espressif USB Serial/JTAG — o proprio chip (S3/C3/C6/H2/C5/P4): serial E JTAG no mesmo cabo"
            }
            ("303a", "1002") => "Espressif USB Bridge — serial + JTAG externo",
            ("10c4", "ea60") => {
                "ponte USB-UART CP210x — o chip do outro lado nao se le pelo USB; a identidade vem pelo canal"
            }
            ("1a86", "7523") => {
                "ponte USB-UART CH340 — o chip do outro lado nao se le pelo USB; a identidade vem pelo canal"
            }
            ("1a86", "55d4") => {
                "ponte USB-UART CH9102 — o chip do outro lado nao se le pelo USB; a identidade vem pelo canal"
            }
            ("0403", "6001" | "6014" | "6015") => {
                "ponte USB-UART FTDI — o chip do outro lado nao se le pelo USB; a identidade vem pelo canal"
            }
            ("0403", "6010") => {
                "FTDI FT2232 (JTAG + UART; e' o do ESP-Prog) — a UART costuma ser a interface 01"
            }
            ("0483", "374b" | "374e" | "374f" | "3752" | "3753" | "3754" | "3757") => {
                "ST-Link com porta serial virtual — a depuracao vai pelo probe-rs, nao por aqui"
            }
            ("2e8a", "000c") => {
                "Raspberry Pi Debug Probe — a UART da placa-alvo (o SWD vai pelo probe-rs)"
            }
            ("2e8a", "0009" | "000a") => "Pico SDK stdio USB — o firmware do usuario falando CDC",
            ("2e8a", "0005") => "MicroPython no RP2040 — o REPL",
            ("1d50", "6018") => "Black Magic Probe — servidor GDB pela propria serial",
            _ => return None,
        },
    )
}
