//! A tabela das ferramentas que o [`super::ToolDetector`] conhece: quem
//! DETECTA e' o detector; esta tabela so' diz o que interessa procurar, com o
//! binario de cada distribuidor. Compilador, depurador ou meta-ferramenta nova
//! e' uma entrada aqui (e, se cumpre um papel de kit, uma no
//! `toolchain/catalog.rs`).

use super::ToolSpec;

/// Every tool the detector looks for, in the order the UI lists them.
pub const KNOWN_TOOLS: &[ToolSpec] = &[
    ToolSpec {
        id: "cargo",
        display_name: "Cargo",
        binary: "cargo",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rustc",
        display_name: "Rust Compiler",
        binary: "rustc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rustup",
        display_name: "rustup",
        binary: "rustup",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "rust-analyzer",
        display_name: "rust-analyzer",
        binary: "rust-analyzer",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "cmake",
        display_name: "CMake",
        binary: "cmake",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "ninja",
        display_name: "Ninja",
        binary: "ninja",
        alternative_binary: None,
        install_command: None,
    },
    // `make` entrou em 2026-09-02 com a toolchain como entidade: o gerador
    // "Unix Makefiles" so pode ser OFERECIDO se ele existir na maquina.
    // Oferecer um gerador ausente e oferecer um configure que vai falhar.
    ToolSpec {
        id: "make",
        display_name: "GNU Make",
        binary: "make",
        alternative_binary: Some("gmake"),
        install_command: None,
    },
    ToolSpec {
        id: "git",
        display_name: "Git",
        binary: "git",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clangd",
        display_name: "clangd",
        binary: "clangd",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clang",
        display_name: "clang",
        binary: "clang",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "clangxx",
        display_name: "clang++",
        binary: "clang++",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gcc",
        display_name: "GCC",
        binary: "gcc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gxx",
        display_name: "g++",
        binary: "g++",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "gdb",
        display_name: "GDB",
        binary: "gdb",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "lldb",
        display_name: "LLDB",
        binary: "lldb",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "lldb-dap",
        display_name: "lldb-dap",
        binary: "lldb-dap",
        alternative_binary: None,
        install_command: None,
    },
    // Cross-compiladores de embarcado. Sao ToolSpec como qualquer outro: quem
    // DETECTA continua sendo o ToolDetector, e o catalogo da toolchain so' diz
    // que eles interessam ao papel de compilador.
    ToolSpec {
        id: "arm-none-eabi-gcc",
        display_name: "GCC (ARM bare-metal)",
        binary: "arm-none-eabi-gcc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "arm-none-eabi-gxx",
        display_name: "G++ (ARM bare-metal)",
        binary: "arm-none-eabi-g++",
        alternative_binary: None,
        install_command: None,
    },
    // As toolchains por alvo do integracoes/39 (2026-09-12): os triples que a
    // industria usa, na grafia dos binarios que cada distribuicao entrega. O
    // `alternative_binary` e' a grafia do OUTRO distribuidor do mesmo alvo
    // (Fedora `arm-linux-gnu-gcc` x Debian/Arm `arm-linux-gnueabihf-gcc`;
    // Arm `aarch64-none-linux-gnu-gcc` x distro `aarch64-linux-gnu-gcc`).
    ToolSpec {
        id: "riscv-none-elf-gcc",
        display_name: "GCC (RISC-V bare-metal)",
        binary: "riscv-none-elf-gcc",
        alternative_binary: Some("riscv32-unknown-elf-gcc"),
        install_command: None,
    },
    ToolSpec {
        id: "riscv-none-elf-gxx",
        display_name: "G++ (RISC-V bare-metal)",
        binary: "riscv-none-elf-g++",
        alternative_binary: Some("riscv32-unknown-elf-g++"),
        install_command: None,
    },
    ToolSpec {
        id: "riscv32-esp-elf-gcc",
        display_name: "GCC (Espressif RISC-V: ESP32-C/H/P)",
        binary: "riscv32-esp-elf-gcc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "riscv32-esp-elf-gxx",
        display_name: "G++ (Espressif RISC-V: ESP32-C/H/P)",
        binary: "riscv32-esp-elf-g++",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "xtensa-esp-elf-gcc",
        display_name: "GCC (Espressif Xtensa: ESP32/S2/S3)",
        binary: "xtensa-esp-elf-gcc",
        alternative_binary: Some("xtensa-esp32-elf-gcc"),
        install_command: None,
    },
    ToolSpec {
        id: "xtensa-esp-elf-gxx",
        display_name: "G++ (Espressif Xtensa: ESP32/S2/S3)",
        binary: "xtensa-esp-elf-g++",
        alternative_binary: Some("xtensa-esp32-elf-g++"),
        install_command: None,
    },
    ToolSpec {
        id: "aarch64-linux-gnu-gcc",
        display_name: "GCC (AArch64 Linux: Pi 4/5, i.MX8, RK3588)",
        binary: "aarch64-linux-gnu-gcc",
        alternative_binary: Some("aarch64-none-linux-gnu-gcc"),
        install_command: None,
    },
    ToolSpec {
        id: "aarch64-linux-gnu-gxx",
        display_name: "G++ (AArch64 Linux: Pi 4/5, i.MX8, RK3588)",
        binary: "aarch64-linux-gnu-g++",
        alternative_binary: Some("aarch64-none-linux-gnu-g++"),
        install_command: None,
    },
    ToolSpec {
        id: "arm-linux-gnueabihf-gcc",
        display_name: "GCC (ARM 32-bit Linux hard-float: Pi 32, BeagleBone)",
        binary: "arm-linux-gnueabihf-gcc",
        alternative_binary: Some("arm-linux-gnu-gcc"),
        install_command: None,
    },
    ToolSpec {
        id: "arm-linux-gnueabihf-gxx",
        display_name: "G++ (ARM 32-bit Linux hard-float: Pi 32, BeagleBone)",
        binary: "arm-linux-gnueabihf-g++",
        alternative_binary: Some("arm-linux-gnu-g++"),
        install_command: None,
    },
    ToolSpec {
        id: "riscv64-linux-gnu-gcc",
        display_name: "GCC (RISC-V 64 Linux: VisionFive 2, Milk-V)",
        binary: "riscv64-linux-gnu-gcc",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "riscv64-linux-gnu-gxx",
        display_name: "G++ (RISC-V 64 Linux: VisionFive 2, Milk-V)",
        binary: "riscv64-linux-gnu-g++",
        alternative_binary: None,
        install_command: None,
    },
    // Os GDBs de alvo: o do sistema ja' e' multi-arquitetura no Fedora (medido
    // em 2026-09-11: falta so' xtensa); Debian/Ubuntu chamam `gdb-multiarch`;
    // os tarballs da Arm e da Espressif trazem o proprio.
    ToolSpec {
        id: "gdb-multiarch",
        display_name: "GDB (multi-arquitetura, Debian/Ubuntu)",
        binary: "gdb-multiarch",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "arm-none-eabi-gdb",
        display_name: "GDB (ARM bare-metal, do tarball da Arm)",
        binary: "arm-none-eabi-gdb",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "xtensa-esp-elf-gdb",
        display_name: "GDB (Espressif Xtensa)",
        binary: "xtensa-esp-elf-gdb",
        alternative_binary: Some("xtensa-esp32-elf-gdb"),
        install_command: None,
    },
    ToolSpec {
        id: "riscv32-esp-elf-gdb",
        display_name: "GDB (Espressif RISC-V)",
        binary: "riscv32-esp-elf-gdb",
        alternative_binary: None,
        install_command: None,
    },
    // Meta-ferramentas de framework (processo; Apache-2.0 / BSD / GPL lidas
    // no integracoes/41): west (Zephyr), pio (PlatformIO), picotool (Pico),
    // dfu-util (STM32 DFU), openocd (servidor GDB), espup (Rust em Xtensa).
    ToolSpec {
        id: "west",
        display_name: "west (Zephyr)",
        binary: "west",
        alternative_binary: None,
        install_command: Some("pipx install west"),
    },
    ToolSpec {
        id: "pio",
        display_name: "PlatformIO Core",
        binary: "pio",
        alternative_binary: Some("platformio"),
        install_command: Some("pipx install platformio"),
    },
    ToolSpec {
        id: "picotool",
        display_name: "picotool (Raspberry Pi Pico)",
        binary: "picotool",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "dfu-util",
        display_name: "dfu-util",
        binary: "dfu-util",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "openocd",
        display_name: "OpenOCD",
        binary: "openocd",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "espup",
        display_name: "espup (Rust para Xtensa)",
        binary: "espup",
        alternative_binary: None,
        install_command: Some("cargo install espup --locked"),
    },
    // Python (bloco B do roadmaps/41, 2026-09-12): o interpretador do sistema
    // (ultimo recurso do resolvedor), o uv que cria ambientes e instala
    // ferramentas, o pipx, e a cadeia que as fatias seguintes ligam (ruff,
    // basedpyright, pytest, mypy, poetry). O mpremote e' o Python no MCU.
    ToolSpec {
        id: "python3",
        display_name: "Python 3",
        binary: "python3",
        alternative_binary: Some("python"),
        install_command: None,
    },
    ToolSpec {
        id: "uv",
        display_name: "uv (ambientes e pacotes Python)",
        binary: "uv",
        alternative_binary: None,
        install_command: Some("pipx install uv"),
    },
    ToolSpec {
        id: "pipx",
        display_name: "pipx",
        binary: "pipx",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "ruff",
        display_name: "ruff (lint e formato Python)",
        binary: "ruff",
        alternative_binary: None,
        install_command: Some("pipx install ruff"),
    },
    ToolSpec {
        id: "basedpyright",
        display_name: "basedpyright (language server Python)",
        binary: "basedpyright-langserver",
        alternative_binary: Some("basedpyright"),
        install_command: Some("uv tool install basedpyright"),
    },
    ToolSpec {
        id: "pytest",
        display_name: "pytest",
        binary: "pytest",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "mypy",
        display_name: "mypy",
        binary: "mypy",
        alternative_binary: None,
        install_command: Some("pipx install mypy"),
    },
    ToolSpec {
        id: "poetry",
        display_name: "Poetry",
        binary: "poetry",
        alternative_binary: None,
        install_command: Some("pipx install poetry"),
    },
    ToolSpec {
        id: "mpremote",
        display_name: "mpremote (MicroPython)",
        binary: "mpremote",
        alternative_binary: None,
        install_command: Some("pipx install mpremote"),
    },
    ToolSpec {
        id: "probe-rs",
        display_name: "probe-rs",
        binary: "probe-rs",
        alternative_binary: None,
        install_command: None,
    },
    // Containers (roadmaps/28 §0: dominio NATIVO). Os dois motores falam a
    // mesma CLI; no Fedora `docker` costuma ser o shim `podman-docker`, e e' o
    // dominio `container` que descobre qual dos dois responde de verdade.
    ToolSpec {
        id: "docker",
        display_name: "Docker",
        binary: "docker",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "podman",
        display_name: "Podman",
        binary: "podman",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "podman-compose",
        display_name: "podman-compose",
        binary: "podman-compose",
        alternative_binary: None,
        install_command: None,
    },
    // Monitores seriais (E3 do integracoes/38 §6): processos prontos que a IDE
    // abre numa aba de terminal. Nenhum e' linkado; nenhum codigo serial nosso.
    ToolSpec {
        id: "tio",
        display_name: "tio",
        binary: "tio",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "picocom",
        display_name: "picocom",
        binary: "picocom",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "minicom",
        display_name: "minicom",
        binary: "minicom",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "espflash",
        display_name: "espflash",
        binary: "espflash",
        alternative_binary: None,
        install_command: Some("cargo install espflash"),
    },
    ToolSpec {
        id: "ripgrep",
        display_name: "ripgrep",
        binary: "rg",
        alternative_binary: None,
        install_command: None,
    },
    ToolSpec {
        id: "fd",
        display_name: "fd",
        binary: "fd",
        alternative_binary: Some("fdfind"),
        install_command: None,
    },
    // CLIs de IA. Detectar `claude` e o MESMO que detectar `cargo`: e
    // CAPACIDADE ("existe no PATH?"), e capacidade pode viver no core. O que
    // NAO pode voltar e POLITICA — como o programa e executado. O core nao tem
    // ramo por programa: nao injeta flag, nao filtra saida e nao sabe que estas
    // entradas sao "de IA". Quem escolhe e roda e a UI.
    //
    // Aqui o `install_command` existe porque npm e canonico e independente de
    // distro — nao e palpite. (Claude Code tambem tem instalador nativo; a
    // sugestao aponta um caminho que funciona em qualquer distro, e o core
    // nunca a executa.)
    ToolSpec {
        id: "claude",
        display_name: "Claude Code",
        binary: "claude",
        alternative_binary: None,
        install_command: Some("npm install -g @anthropic-ai/claude-code"),
    },
    ToolSpec {
        id: "codex",
        display_name: "Codex",
        binary: "codex",
        alternative_binary: None,
        install_command: Some("npm install -g @openai/codex"),
    },
];
