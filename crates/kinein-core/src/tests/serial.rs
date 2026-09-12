//! O dominio `serial` (E1 do `DocsPublic/integracoes/38` §6): as portas seriais USB
//! lidas de um `/sys`, `/dev` e `/proc` FALSOS, montados como o kernel os
//! monta — e a fixture e' o ESP32 classico medido em 2026-09-11.
//!
//! O que se prova aqui e' a TRADUCAO: subir do tty ao dispositivo USB, separar
//! ponte de CDC, ignorar `ttyS*`, medir a permissao em vez de supor, e dizer
//! "desconhecido" quando o udevadm nao respondeu. Nada aqui abre porta nenhuma.

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use kinein_protocol::SerialPortKind;

use crate::serial::monitor::{DEFAULT_BAUD, command_line, escolher};
use crate::serial::{Ambiente, familia, list_in, modo_simbolico, parse_udev_properties};
use crate::toolchain::{KitUpdate, Toolchain};
use kinein_protocol::{ToolInfo, ToolStatus, ToolchainRole};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-serial-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn escreve(dir: &Path, nome: &str, valor: &str) {
    std::fs::write(dir.join(nome), format!("{valor}\n")).unwrap();
}

/// Um `/sys` com o ESP32 clássico medido em 2026-09-11 (CP2102 em
/// ttyUSB0) e um ttyACM0 de USB-JTAG da Espressif, mais os `ttyS*` que
/// nao sao USB.
fn sysfs_medido(raiz: &Path) -> PathBuf {
    let usb = raiz.join("sys/devices/pci0000:00/usb8/8-1");
    let iface = usb.join("8-1:1.0");
    let porta = iface.join("ttyUSB0");
    std::fs::create_dir_all(&porta).unwrap();
    escreve(&usb, "idVendor", "10c4");
    escreve(&usb, "idProduct", "ea60");
    escreve(&usb, "manufacturer", "Silicon Labs");
    escreve(&usb, "product", "CP2102 USB to UART Bridge Controller");
    escreve(&usb, "serial", "0001");
    escreve(&iface, "bInterfaceNumber", "00");
    let drivers = raiz.join("sys/bus/usb-serial/drivers/cp210x");
    std::fs::create_dir_all(&drivers).unwrap();
    std::os::unix::fs::symlink(&drivers, porta.join("driver")).unwrap();

    // ttyACM0: o `device` e' a propria interface, sem diretorio de porta.
    let usb2 = raiz.join("sys/devices/pci0000:00/usb1/1-2");
    let iface2 = usb2.join("1-2:1.0");
    std::fs::create_dir_all(&iface2).unwrap();
    escreve(&usb2, "idVendor", "303a");
    escreve(&usb2, "idProduct", "1001");
    escreve(&usb2, "product", "USB JTAG/serial debug unit");
    escreve(&iface2, "bInterfaceNumber", "00");
    let cdc = raiz.join("sys/bus/usb/drivers/cdc_acm");
    std::fs::create_dir_all(&cdc).unwrap();
    std::os::unix::fs::symlink(&cdc, iface2.join("driver")).unwrap();

    let class = raiz.join("sys/class/tty");
    for nome in ["ttyUSB0", "ttyACM0", "ttyS0", "ttyS1"] {
        std::fs::create_dir_all(class.join(nome)).unwrap();
    }
    std::os::unix::fs::symlink(&porta, class.join("ttyUSB0/device")).unwrap();
    std::os::unix::fs::symlink(&iface2, class.join("ttyACM0/device")).unwrap();
    // ttyS0 tem `device` sem idVendor acima: platform, nao USB.
    let plat = raiz.join("sys/devices/platform/serial8250");
    std::fs::create_dir_all(&plat).unwrap();
    std::os::unix::fs::symlink(&plat, class.join("ttyS0/device")).unwrap();

    let dev = raiz.join("dev");
    std::fs::create_dir_all(dev.join("serial/by-id")).unwrap();
    std::fs::write(dev.join("ttyUSB0"), "").unwrap();
    std::fs::write(dev.join("ttyACM0"), "").unwrap();
    std::os::unix::fs::symlink(
        "../../ttyUSB0",
        dev.join(
            "serial/by-id/usb-Silicon_Labs_CP2102_USB_to_UART_Bridge_Controller_0001-if00-port0",
        ),
    )
    .unwrap();
    // O gid do no' falso e' o do usuario que roda o teste; o /etc/group falso
    // o batiza de `dialout`, que e' o que a dica tem de nomear.
    let gid = std::fs::metadata(dev.join("ttyUSB0")).unwrap().gid();
    std::fs::write(
        raiz.join("group"),
        format!("root:x:0:\ndialout:x:{gid}:hugh\n"),
    )
    .unwrap();
    // /proc com um ModemManager vivo.
    std::fs::create_dir_all(raiz.join("proc/1030")).unwrap();
    std::fs::write(raiz.join("proc/1030/comm"), "ModemManager\n").unwrap();
    raiz.to_path_buf()
}

/// Os quatro caminhos de um `/` falso, donos das strings que o `Ambiente`
/// empresta.
struct Raizes {
    class: PathBuf,
    dev: PathBuf,
    proc_root: PathBuf,
    group: PathBuf,
}

impl Raizes {
    fn em(raiz: &Path) -> Self {
        Self {
            class: raiz.join("sys/class/tty"),
            dev: raiz.join("dev"),
            proc_root: raiz.join("proc"),
            group: raiz.join("group"),
        }
    }

    fn ambiente<'a>(&'a self, udev: &'a dyn Fn(&Path) -> Option<String>) -> Ambiente<'a> {
        Ambiente {
            sys_class_tty: &self.class,
            dev: &self.dev,
            proc_root: &self.proc_root,
            etc_group: &self.group,
            udev_props: udev,
        }
    }
}

#[test]
fn walks_up_sysfs_to_the_usb_device_and_skips_platform_ttys() {
    let raiz = sysfs_medido(&temp_dir("sysfs"));
    let udev = |no: &Path| {
        no.ends_with("ttyUSB0")
            .then(|| "ID_MM_CANDIDATE=1\nID_USB_DRIVER=cp210x\n".to_owned())
    };
    let raizes = Raizes::em(&raiz);
    let resultado = list_in(&raizes.ambiente(&udev));
    assert_eq!(resultado.ports.len(), 2, "ttyS* nao entra: {resultado:?}");
    assert!(resultado.hint.is_none());

    let acm = &resultado.ports[0];
    assert!(acm.device.ends_with("/dev/ttyACM0"));
    assert_eq!(acm.kind, SerialPortKind::UsbCdc);
    assert_eq!((acm.vid.as_str(), acm.pid.as_str()), ("303a", "1001"));
    assert_eq!(acm.driver.as_deref(), Some("cdc_acm"));
    assert!(
        acm.family
            .as_deref()
            .unwrap()
            .starts_with("Espressif USB Serial/JTAG")
    );
    assert!(acm.by_id.is_none());

    let usb = &resultado.ports[1];
    assert_eq!(usb.kind, SerialPortKind::UsbUartBridge);
    assert_eq!((usb.vid.as_str(), usb.pid.as_str()), ("10c4", "ea60"));
    assert_eq!(usb.manufacturer.as_deref(), Some("Silicon Labs"));
    assert_eq!(usb.serial.as_deref(), Some("0001"));
    assert_eq!(usb.interface.as_deref(), Some("00"));
    assert_eq!(usb.driver.as_deref(), Some("cp210x"));
    assert!(usb.by_id.as_deref().unwrap().ends_with("-if00-port0"));
    let mm = usb.modem_manager.as_ref().unwrap();
    assert!(mm.candidate && !mm.ignored && mm.running);
    // A porta ACM nao teve udevadm: o estado do MM e' DESCONHECIDO, nao falso.
    assert!(acm.modem_manager.is_none());
}

#[test]
fn access_is_measured_not_guessed_and_the_hint_names_the_group() {
    use std::os::unix::fs::PermissionsExt;
    let raiz = sysfs_medido(&temp_dir("acesso"));
    let no = raiz.join("dev/ttyUSB0");
    std::fs::set_permissions(&no, std::fs::Permissions::from_mode(0o000)).unwrap();
    // root (ou CAP_DAC_OVERRIDE) ignora modo 000; o teste so' vale para
    // usuario comum, e e' o proprio `open` que diz qual dos dois somos.
    if std::fs::File::open(&no).is_ok() {
        return;
    }
    let udev = |_: &Path| None;
    let raizes = Raizes::em(&raiz);
    let ports = list_in(&raizes.ambiente(&udev)).ports;
    let usb = ports
        .iter()
        .find(|p| p.device.ends_with("ttyUSB0"))
        .unwrap();
    assert!(!usb.access.readable_writable);
    assert_eq!(usb.access.mode, "----------");
    assert!(usb.access.hint.as_deref().unwrap().contains("usermod -aG"));
    assert!(!usb.access.hint.as_deref().unwrap().contains("sudo chmod"));

    // Modo com rw so' no GRUPO, num no' que e' NOSSO: o kernel aplica os bits
    // do dono (nenhum) e nega. A heuristica "root:dialout 0660 => olha os bits
    // do grupo" diria sim — e' exatamente a mutacao que este caso mata.
    std::fs::set_permissions(&no, std::fs::Permissions::from_mode(0o060)).unwrap();
    let ports = list_in(&raizes.ambiente(&udev)).ports;
    let usb = ports
        .iter()
        .find(|p| p.device.ends_with("ttyUSB0"))
        .unwrap();
    assert!(
        !usb.access.readable_writable,
        "bits do grupo nao valem para o dono"
    );
    assert_eq!(usb.access.mode, "----rw----");

    // O que este teste NAO cobre, e e' limite de usuario comum: o caso real do
    // `TAG+="uaccess"` — no' de root com ACL nomeada para o usuario. Numa
    // ACL, o dono e' julgado pela entrada do dono (medido: mode 000 + ACL
    // u:eu:rw ainda nega para o proprio dono), entao sem root nao ha' como
    // montar o cenario. Quem o prova e' a exercitacao contra /dev/ttyUSB0.
    std::fs::set_permissions(&no, std::fs::Permissions::from_mode(0o600)).unwrap();
    let ports = list_in(&raizes.ambiente(&udev)).ports;
    let usb = ports
        .iter()
        .find(|p| p.device.ends_with("ttyUSB0"))
        .unwrap();
    assert!(usb.access.readable_writable);
    assert!(usb.access.hint.is_none());
}

#[test]
fn empty_sys_gives_a_hint_and_no_ports() {
    let raiz = temp_dir("vazio");
    let udev = |_: &Path| None;
    let raizes = Raizes::em(&raiz);
    let resultado = list_in(&raizes.ambiente(&udev));
    assert!(resultado.ports.is_empty());
    assert!(resultado.hint.as_deref().unwrap().contains("lsusb"));
}

#[test]
fn family_says_what_the_link_is_never_the_chip_behind_a_bridge() {
    assert!(
        familia("10c4", "EA60")
            .unwrap()
            .contains("nao se le pelo USB")
    );
    assert!(familia("303a", "1001").unwrap().contains("proprio chip"));
    assert!(familia("0483", "374b").unwrap().contains("ST-Link"));
    assert!(familia("2e8a", "000c").unwrap().contains("Debug Probe"));
    assert_eq!(familia("dead", "beef"), None);
}

#[test]
fn udev_properties_and_symbolic_mode() {
    let props = parse_udev_properties("DEVNAME=/dev/ttyUSB0\nID_MM_CANDIDATE=1\nlixo\n");
    assert_eq!(props.get("ID_MM_CANDIDATE").map(String::as_str), Some("1"));
    assert_eq!(props.len(), 2);
    assert_eq!(modo_simbolico(0o020_660), "crw-rw----");
    assert_eq!(modo_simbolico(0o100_755), "-rwxr-xr-x");
}

/// Uma ferramenta DETECTADA num caminho, como o `tools.detect` a descreveria.
fn detectada(id: &str, path: &str) -> ToolInfo {
    ToolInfo {
        id: id.to_owned(),
        display_name: id.to_owned(),
        status: ToolStatus::Detected,
        path: Some(path.to_owned()),
        version: Some("x".to_owned()),
        suggested_install: None,
        message: None,
    }
}

#[test]
fn monitor_command_lines_follow_each_tool_and_only_espflash_takes_the_elf() {
    let elf = Path::new("/tmp/fw.elf");
    let (p, a) = command_line(
        "tio",
        "/usr/bin/tio",
        "/dev/ttyUSB0",
        DEFAULT_BAUD,
        Some(elf),
    );
    assert_eq!(
        (p.as_str(), a),
        (
            "/usr/bin/tio",
            vec!["-b".into(), "115200".into(), "/dev/ttyUSB0".into()]
        )
    );
    let (_, a) = command_line("picocom", "/usr/bin/picocom", "/dev/ttyACM0", 9600, None);
    assert_eq!(a, vec!["-b", "9600", "/dev/ttyACM0"]);
    let (_, a) = command_line("minicom", "/usr/bin/minicom", "/dev/ttyUSB1", 57600, None);
    assert_eq!(a, vec!["-D", "/dev/ttyUSB1", "-b", "57600"]);
    // espflash: e' `--monitor-baud`, NAO `--baud` (que e' o baud de gravacao).
    let (_, a) = command_line(
        "espflash",
        "/x/espflash",
        "/dev/ttyUSB0",
        DEFAULT_BAUD,
        Some(elf),
    );
    assert_eq!(
        a,
        vec![
            "monitor",
            "--port",
            "/dev/ttyUSB0",
            "--monitor-baud",
            "115200",
            "--elf",
            "/tmp/fw.elf"
        ]
    );
    assert!(!a.contains(&"--baud".to_owned()));
    let (_, a) = command_line(
        "espflash",
        "/x/espflash",
        "/dev/ttyUSB0",
        DEFAULT_BAUD,
        None,
    );
    assert_eq!(a.len(), 5, "sem ELF, sem --elf");
}

#[test]
fn the_monitor_choice_prefers_espflash_only_for_espressif_kits_and_respects_a_fixed_choice() {
    let raiz = temp_dir("monitor-escolha");
    let tools = vec![
        detectada("picocom", "/usr/bin/picocom"),
        detectada("minicom", "/usr/bin/minicom"),
        detectada("espflash", "/home/x/.cargo/bin/espflash"),
    ];
    // Sem chip: o primeiro detectado na ordem do catalogo (tio nao esta').
    let tc = Toolchain::resolve(&raiz, &tools);
    let escolha = escolher(&tc).unwrap();
    assert_eq!(
        (escolha.id.as_str(), escolha.program.as_str()),
        ("picocom", "/usr/bin/picocom")
    );

    // Chip Espressif no kit: espflash passa a frente, sem ninguem fixar nada.
    let tc = crate::toolchain::set_kit(
        &raiz,
        &tools,
        "",
        KitUpdate {
            chip: Some("ESP32-C3"),
            ..KitUpdate::default()
        },
    )
    .unwrap();
    assert_eq!(escolher(&tc).unwrap().id, "espflash");

    // Chip que NAO e' Espressif (um STM32) com espflash presente: o espflash
    // NAO passa a frente — ele so' fala com chips Espressif.
    let tc = crate::toolchain::set_kit(
        &raiz,
        &tools,
        "",
        KitUpdate {
            chip: Some("STM32F401CC"),
            ..KitUpdate::default()
        },
    )
    .unwrap();
    assert_eq!(escolher(&tc).unwrap().id, "picocom");
    let tc = crate::toolchain::set_kit(
        &raiz,
        &tools,
        "",
        KitUpdate {
            chip: Some("esp32s3"),
            ..KitUpdate::default()
        },
    )
    .unwrap();
    assert_eq!(escolher(&tc).unwrap().id, "espflash");

    // Chip Espressif SEM espflash detectado: volta ao efetivo, nao falha.
    let sem_espflash: Vec<ToolInfo> = tools
        .iter()
        .filter(|t| t.id != "espflash")
        .cloned()
        .collect();
    let tc = Toolchain::resolve(&raiz, &sem_espflash);
    assert_eq!(escolher(&tc).unwrap().id, "picocom");

    // O autor FIXOU minicom: vale mesmo com chip Espressif e espflash presente.
    let tc = crate::toolchain::set(
        &raiz,
        &tools,
        ToolchainRole::SerialMonitor,
        Some("minicom"),
        "",
    )
    .unwrap();
    assert_eq!(escolher(&tc).unwrap().id, "minicom");

    // Nenhum monitor detectado: None, e o handler diz o que instalar.
    let tc = Toolchain::resolve(&raiz, &[]);
    assert!(escolher(&tc).is_none());
}
