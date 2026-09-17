//! O catalogo de instalacao das ferramentas de EMBARCADOS (A5 do bloco A do
//! `roadmaps/41`, 2026-09-17). Mesma regra do `catalog.rs`, sem excecao:
//! **nenhum comando aqui foi escrito por quem programou a IDE** — cada passo
//! vem da pagina oficial da ferramenta ou, para pacote da distro, da pagina
//! do pacote no indice da distro, e a entrada carrega a URL e a data.
//!
//! # Duas classes de fonte, ditas
//!
//! 1. **A pagina da ferramenta** (esptool, mpremote, espflash, probe-rs,
//!    picotool): o comando e' o dela, verbatim (`pip install esptool`,
//!    `pipx install mpremote`, `cargo install espflash --locked`, o script
//!    do probe-rs, o `BUILDING.md` do picotool).
//! 2. **O indice de pacotes da distro** (openocd, dfu-util, picocom, tio,
//!    gcc-arm-none-eabi, QEMU): a pagina do pacote prova que o pacote EXISTE
//!    com aquele nome; o comando e' a forma padrao do gerenciador da familia
//!    (`sudo apt install`, `sudo dnf install`, `sudo pacman -S`). Onde o
//!    indice nao tem o pacote (tio e picotool no Arch oficial; picotool e
//!    espflash no Fedora), NAO ha' entrada — o painel mostra o site.
//!
//! Conferido em 2026-09-17 com `curl` nos indices (HTTP 200 no pacote) e
//! `WebFetch` nas paginas das ferramentas. O que NAO se conferiu: nenhum
//! destes comandos foi executado nesta maquina.
//!
//! Arquivo proprio porque o `catalog.rs` esta' em 392 linhas e a catraca
//! e' 500: dominio novo (embarcados), arquivo novo — nao se paga debito com
//! debito.

use super::catalog::{Guide, Step, Tool};

pub(super) static TOOLS: &[Tool] = &[
    Tool {
        id: "esptool",
        name: "esptool",
        summary: "Fala com o bootloader de ROM dos ESP32: le o chip e a flash (a lupa do \
                  painel de Embarcados) e grava o firmware (o motor do Gravar).",
        website: "https://docs.espressif.com/projects/esptool/en/latest/esp32/installation.html",
        probe_binary: "esptool",
    },
    Tool {
        id: "mpremote",
        name: "mpremote",
        summary: "A ferramenta oficial do MicroPython: REPL, arquivos na placa e rodar um \
                  .py nela — e' o Executar e o monitor de um projeto MicroPython.",
        website: "https://docs.micropython.org/en/latest/reference/mpremote.html",
        probe_binary: "mpremote",
    },
    Tool {
        id: "espflash",
        name: "espflash",
        summary: "Grava e monitora Espressif a partir do ELF (Rust e C); o monitor dele \
                  decodifica o backtrace com o ELF.",
        website: "https://github.com/esp-rs/espflash",
        probe_binary: "espflash",
    },
    Tool {
        id: "probe-rs",
        name: "probe-rs",
        summary: "Grava, roda e depura ARM/RISC-V por uma sonda (ST-Link, J-Link, CMSIS-DAP, \
                  o USB-JTAG dos ESP32-C3/S3): e' o adaptador DAP do kit embarcado.",
        website: "https://probe.rs/docs/getting-started/installation/",
        probe_binary: "probe-rs",
    },
    Tool {
        id: "picotool",
        name: "picotool",
        summary: "A ferramenta do Raspberry Pi Pico: grava o UF2 pelo USB (BOOTSEL) e le a \
                  placa. O pico-sdk precisa dela INSTALADA (cmake --install), nao so' no PATH.",
        website: "https://github.com/raspberrypi/picotool",
        probe_binary: "picotool",
    },
    Tool {
        id: "dfu-util",
        name: "dfu-util",
        summary: "Grava por DFU (USB) sem sonda: o bootloader de fabrica dos STM32 e de \
                  outras placas com DFU.",
        website: "https://dfu-util.sourceforge.net/",
        probe_binary: "dfu-util",
    },
    Tool {
        id: "tio",
        name: "tio",
        summary: "Monitor serial simples e moderno (tio -b 115200 /dev/ttyUSB0): um dos \
                  candidatos do papel serialMonitor do kit.",
        website: "https://github.com/tio/tio",
        probe_binary: "tio",
    },
    Tool {
        id: "picocom",
        name: "picocom",
        summary: "Monitor serial minimo, presente em toda distro; o kit o usa quando nao \
                  ha' tio nem espflash.",
        website: "https://github.com/npat-efault/picocom",
        probe_binary: "picocom",
    },
    Tool {
        id: "arm-none-eabi",
        name: "GCC ARM (arm-none-eabi)",
        summary: "O compilador cross para Cortex-M (STM32, nRF, Pico): gcc, binutils e \
                  newlib. O GDB multiarch depura pelo `gdb -i dap` com o servidor do kit.",
        website: "https://developer.arm.com/Tools%20and%20Software/GNU%20Toolchain",
        probe_binary: "arm-none-eabi-gcc",
    },
    Tool {
        id: "qemu-embedded",
        name: "QEMU (ARM e RISC-V)",
        summary: "Emula a placa: e' como o ciclo de embarcado da IDE e' provado SEM \
                  hardware (lm3s6965evb com `gdb -i dap`).",
        website: "https://www.qemu.org/download/#linux",
        probe_binary: "qemu-system-arm",
    },
    Tool {
        id: "openocd",
        name: "OpenOCD",
        summary: "Servidor GDB para sondas JTAG/SWD; alternativa ao probe-rs como \
                  `debugServer` do kit (`openocd -f …`). GPL-2.0: processo, nunca embutido.",
        website: "https://openocd.org/pages/getting-openocd.html",
        probe_binary: "openocd",
    },
];

// --------------------------------------------------------------------------
// esptool — docs.espressif.com/projects/esptool/en/latest/esp32/installation.html
// (conferido 2026-09-17): "The latest stable esptool release can be installed
// from PyPI via pip: $ pip install esptool" e a recomendacao de ambiente
// virtual. Numa distro que segue a PEP 668 o `pip install` solto e' recusado;
// o proprio site nao documenta pipx, entao o guia `any` e' o do site (pip
// dentro de um venv) e o pipx entra como o que o Fedora/Debian documentam para
// ferramentas Python (guia do pipx no catalogo principal). Debian trixie e
// Fedora tambem empacotam `esptool` (indices conferidos).
// --------------------------------------------------------------------------

static ESPTOOL_ANY: &[Step] = &[
    Step {
        explanation: "Cria e ativa um ambiente virtual so' para o esptool, como a pagina \
                      recomenda (evita o bloqueio da PEP 668 das distros novas).",
        command: "python3 -m venv ~/.local/share/esptoolenv && source ~/.local/share/esptoolenv/bin/activate",
    },
    Step {
        explanation: "Instala do PyPI, como a pagina escreve.",
        command: "pip install esptool",
    },
];

static ESPTOOL_DEBIAN: &[Step] = &[Step {
    explanation: "O pacote da distro (packages.debian.org/trixie/esptool).",
    command: "sudo apt install esptool",
}];

static ESPTOOL_REDHAT: &[Step] = &[Step {
    explanation: "O pacote da distro (packages.fedoraproject.org/pkgs/esptool).",
    command: "sudo dnf install esptool",
}];

// --------------------------------------------------------------------------
// mpremote — docs.micropython.org/en/latest/reference/mpremote.html
// (conferido 2026-09-17): "$ pip install --user mpremote" e "$ pipx install
// mpremote". O pipx e' o que sobrevive a PEP 668.
// --------------------------------------------------------------------------

static MPREMOTE_ANY: &[Step] = &[Step {
    explanation: "Instala pelo pipx, como a pagina do mpremote escreve (o pipx esta' no \
                  catalogo, guia proprio).",
    command: "pipx install mpremote",
}];

// --------------------------------------------------------------------------
// espflash — github.com/esp-rs/espflash/blob/main/espflash/README.md
// (conferido 2026-09-17): "cargo install espflash --locked" (exige rustc >=
// 1.95) e "cargo binstall espflash". O Arch empacota. O Debian trixie tambem,
// mas o Ubuntu 26.04 NAO (apt-cache sem candidato, medido) — e a familia
// `debian` cobre os dois; um guia que falha em metade da familia nao entra:
// no Debian vale o `any` do cargo tambem.
// --------------------------------------------------------------------------

static ESPFLASH_ANY: &[Step] = &[Step {
    explanation: "Compila e instala pelo cargo, como o README escreve (rustc >= 1.95).",
    command: "cargo install espflash --locked",
}];

static ESPFLASH_ARCH: &[Step] = &[Step {
    explanation: "O pacote da distro (archlinux.org/packages/extra/x86_64/espflash).",
    command: "sudo pacman -S espflash",
}];

// --------------------------------------------------------------------------
// probe-rs — probe.rs/docs/getting-started/installation/ (conferido
// 2026-09-17): o script oficial; alternativas `cargo install probe-rs-tools
// --locked` e `cargo binstall probe-rs-tools`; dependencias no Debian
// "sudo apt install -y pkg-config libudev-dev cmake git". A regra udev e'
// outra pagina (probe-setup) e o painel de Embarcados a da' no canal `probe`.
// --------------------------------------------------------------------------

static PROBE_RS_ANY: &[Step] = &[Step {
    explanation: "O instalador oficial (binario pre-compilado em ~/.cargo/bin), como a \
                  pagina escreve.",
    command: "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh",
}];

static PROBE_RS_DEBIAN: &[Step] = &[
    Step {
        explanation: "As dependencias que a pagina lista para Debian/Ubuntu (so' fazem falta \
                      ao compilar pelo cargo; o instalador nao precisa delas).",
        command: "sudo apt install -y pkg-config libudev-dev cmake git",
    },
    Step {
        explanation: "O instalador oficial, como a pagina escreve.",
        command: "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh",
    },
];

// --------------------------------------------------------------------------
// picotool — github.com/raspberrypi/picotool/blob/master/BUILDING.md
// (conferido 2026-09-17): "sudo apt install build-essential pkg-config
// libusb-1.0-0-dev cmake", "mkdir build / cd build / cmake .. / make", e a
// regra "sudo cp udev/60-picotool.rules /etc/udev/rules.d/". O README manda
// INSTALAR (cmake --install .) para o pico-sdk o achar. Debian trixie tem o
// pacote; o Ubuntu 26.04 nao (apt-cache sem candidato, medido); Fedora e o
// Arch oficial nao tem. So' o guia debian (pelo BUILDING.md) existe.
// --------------------------------------------------------------------------

static PICOTOOL_DEBIAN: &[Step] = &[
    Step {
        explanation: "As dependencias de build, como o BUILDING.md escreve.",
        command: "sudo apt install build-essential pkg-config libusb-1.0-0-dev cmake",
    },
    Step {
        explanation: "Clona o fonte (o pico-sdk tem de estar em PICO_SDK_PATH ou o cmake o \
                      baixa com -DPICOTOOL_FETCH_FROM_GIT_PATH).",
        command: "git clone https://github.com/raspberrypi/picotool.git ~/.local/src/picotool && cd ~/.local/src/picotool",
    },
    Step {
        explanation: "Compila, como o BUILDING.md escreve.",
        command: "mkdir -p build && cd build && cmake .. && make",
    },
    Step {
        explanation: "INSTALA — o README e' explicito: copiar o binario para o PATH nao \
                      basta para o pico-sdk o achar.",
        command: "sudo cmake --install .",
    },
    Step {
        explanation: "A regra udev para usar sem sudo, como o BUILDING.md escreve.",
        command: "sudo cp ../udev/60-picotool.rules /etc/udev/rules.d/",
    },
];

// --------------------------------------------------------------------------
// Pacotes da distro — indices conferidos em 2026-09-17 (HTTP 200 em cada
// pagina de pacote citada). O comando e' a forma padrao do gerenciador.
// --------------------------------------------------------------------------

static DFU_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/dfu-util",
    command: "sudo apt install dfu-util",
}];
static DFU_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/dfu-util",
    command: "sudo dnf install dfu-util",
}];
static DFU_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/dfu-util",
    command: "sudo pacman -S dfu-util",
}];

static TIO_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/tio (o README do tio manda consultar o \
                  gerenciador da distro).",
    command: "sudo apt install tio",
}];
static TIO_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/tio",
    command: "sudo dnf install tio",
}];

static PICOCOM_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/picocom",
    command: "sudo apt install picocom",
}];
static PICOCOM_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/picocom",
    command: "sudo dnf install picocom",
}];
static PICOCOM_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/picocom",
    command: "sudo pacman -S picocom",
}];

static ARM_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/gcc-arm-none-eabi e gdb-multiarch (o GDB que \
                  depura o alvo cross).",
    command: "sudo apt install gcc-arm-none-eabi gdb-multiarch",
}];
static ARM_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/arm-none-eabi-gcc-cs, arm-none-eabi-newlib \
                  e arm-none-eabi-binutils-cs; o gdb do Fedora ja' e' multiarch.",
    command: "sudo dnf install arm-none-eabi-gcc-cs arm-none-eabi-newlib arm-none-eabi-binutils-cs gdb",
}];
static ARM_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/arm-none-eabi-gcc e arm-none-eabi-gdb.",
    command: "sudo pacman -S arm-none-eabi-gcc arm-none-eabi-newlib arm-none-eabi-gdb",
}];

static QEMU_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/qemu-system-arm e qemu-system-misc (o RISC-V \
                  esta' no `misc`).",
    command: "sudo apt install qemu-system-arm qemu-system-misc",
}];
static QEMU_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/qemu/qemu-system-arm e qemu-system-riscv.",
    command: "sudo dnf install qemu-system-arm qemu-system-riscv",
}];
static QEMU_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/qemu-system-arm e qemu-system-riscv.",
    command: "sudo pacman -S qemu-system-arm qemu-system-riscv",
}];

static OPENOCD_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/openocd (traz o 60-openocd.rules).",
    command: "sudo apt install openocd",
}];
static OPENOCD_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/openocd",
    command: "sudo dnf install openocd",
}];
static OPENOCD_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/openocd",
    command: "sudo pacman -S openocd",
}];

const ESPTOOL_URL: &str =
    "https://docs.espressif.com/projects/esptool/en/latest/esp32/installation.html";
const PROBE_RS_URL: &str = "https://probe.rs/docs/getting-started/installation/";
const CONFERIDO: &str = "2026-09-17";

/// Um guia de pacote da distro: a URL e' a pagina do pacote.
const fn pacote(
    tool: &'static str,
    family: &'static str,
    steps: &'static [Step],
    url: &'static str,
) -> Guide {
    Guide {
        tool,
        family,
        steps,
        source_url: url,
        checked_at: CONFERIDO,
    }
}

pub(super) static GUIDES: &[Guide] = &[
    pacote("esptool", "any", ESPTOOL_ANY, ESPTOOL_URL),
    pacote(
        "esptool",
        "debian",
        ESPTOOL_DEBIAN,
        "https://packages.debian.org/trixie/esptool",
    ),
    pacote(
        "esptool",
        "redhat",
        ESPTOOL_REDHAT,
        "https://packages.fedoraproject.org/pkgs/esptool/esptool/",
    ),
    pacote(
        "mpremote",
        "any",
        MPREMOTE_ANY,
        "https://docs.micropython.org/en/latest/reference/mpremote.html",
    ),
    pacote(
        "espflash",
        "any",
        ESPFLASH_ANY,
        "https://github.com/esp-rs/espflash/blob/main/espflash/README.md",
    ),
    pacote(
        "espflash",
        "arch",
        ESPFLASH_ARCH,
        "https://archlinux.org/packages/extra/x86_64/espflash/",
    ),
    pacote("probe-rs", "any", PROBE_RS_ANY, PROBE_RS_URL),
    pacote("probe-rs", "debian", PROBE_RS_DEBIAN, PROBE_RS_URL),
    pacote(
        "picotool",
        "debian",
        PICOTOOL_DEBIAN,
        "https://github.com/raspberrypi/picotool/blob/master/BUILDING.md",
    ),
    pacote(
        "dfu-util",
        "debian",
        DFU_DEBIAN,
        "https://packages.debian.org/trixie/dfu-util",
    ),
    pacote(
        "dfu-util",
        "redhat",
        DFU_REDHAT,
        "https://packages.fedoraproject.org/pkgs/dfu-util/dfu-util/",
    ),
    pacote(
        "dfu-util",
        "arch",
        DFU_ARCH,
        "https://archlinux.org/packages/extra/x86_64/dfu-util/",
    ),
    pacote(
        "tio",
        "debian",
        TIO_DEBIAN,
        "https://packages.debian.org/trixie/tio",
    ),
    pacote(
        "tio",
        "redhat",
        TIO_REDHAT,
        "https://packages.fedoraproject.org/pkgs/tio/tio/",
    ),
    pacote(
        "picocom",
        "debian",
        PICOCOM_DEBIAN,
        "https://packages.debian.org/trixie/picocom",
    ),
    pacote(
        "picocom",
        "redhat",
        PICOCOM_REDHAT,
        "https://packages.fedoraproject.org/pkgs/picocom/picocom/",
    ),
    pacote(
        "picocom",
        "arch",
        PICOCOM_ARCH,
        "https://archlinux.org/packages/extra/x86_64/picocom/",
    ),
    pacote(
        "arm-none-eabi",
        "debian",
        ARM_DEBIAN,
        "https://packages.debian.org/trixie/gcc-arm-none-eabi",
    ),
    pacote(
        "arm-none-eabi",
        "redhat",
        ARM_REDHAT,
        "https://packages.fedoraproject.org/pkgs/arm-none-eabi-gcc-cs/arm-none-eabi-gcc-cs/",
    ),
    pacote(
        "arm-none-eabi",
        "arch",
        ARM_ARCH,
        "https://archlinux.org/packages/extra/x86_64/arm-none-eabi-gcc/",
    ),
    pacote(
        "qemu-embedded",
        "debian",
        QEMU_DEBIAN,
        "https://packages.debian.org/trixie/qemu-system-arm",
    ),
    pacote(
        "qemu-embedded",
        "redhat",
        QEMU_REDHAT,
        "https://packages.fedoraproject.org/pkgs/qemu/qemu-system-arm/",
    ),
    pacote(
        "qemu-embedded",
        "arch",
        QEMU_ARCH,
        "https://archlinux.org/packages/extra/x86_64/qemu-system-arm/",
    ),
    pacote(
        "openocd",
        "debian",
        OPENOCD_DEBIAN,
        "https://packages.debian.org/trixie/openocd",
    ),
    pacote(
        "openocd",
        "redhat",
        OPENOCD_REDHAT,
        "https://packages.fedoraproject.org/pkgs/openocd/openocd/",
    ),
    pacote(
        "openocd",
        "arch",
        OPENOCD_ARCH,
        "https://archlinux.org/packages/extra/x86_64/openocd/",
    ),
];
