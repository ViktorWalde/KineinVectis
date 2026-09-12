//! Quem pode cumprir cada papel, e como esse papel vira argumento de comando.
//!
//! Tabela estatica, como o catalogo das Configuration Actions e pelo mesmo
//! motivo: nao ha registro dinamico neste projeto. Compilador novo e uma
//! entrada aqui — e uma entrada em [`crate::tools::KNOWN_TOOLS`], porque quem
//! DETECTA continua sendo o `ToolDetector`. Este modulo nao procura binario;
//! ele diz **quais** binarios interessam a cada papel e em que ordem.

use kinein_protocol::ToolchainRole;

/// Um candidato possivel para um papel.
pub(super) struct RoleCandidate {
    /// Id estavel; para compilador/ferramenta e o mesmo do `tools.detect`.
    pub(super) id: &'static str,
    /// Rotulo curto para a UI.
    pub(super) label: &'static str,
    /// Id em [`crate::tools::KNOWN_TOOLS`] que prova que ele existe aqui.
    ///
    /// O gerador tambem tem um: `Ninja` so aparece se o `ninja` existir, e
    /// `Unix Makefiles` so se o `make` existir. Oferecer um gerador que a
    /// maquina nao tem e' oferecer um configure que vai falhar.
    pub(super) tool_id: &'static str,
}

/// Candidatos de um papel, na ordem de preferencia.
///
/// A ordem importa: e' ela que a UI mostra, e o primeiro DETECTADO e o que a
/// dica de "automatico" descreve.
#[must_use]
pub(super) fn candidates_for(role: ToolchainRole) -> &'static [RoleCandidate] {
    match role {
        ToolchainRole::CCompiler => C_COMPILERS,
        ToolchainRole::CxxCompiler => CXX_COMPILERS,
        ToolchainRole::Generator => GENERATORS,
        ToolchainRole::Cmake => CMAKE,
        ToolchainRole::Cargo => CARGO,
        ToolchainRole::DebugAdapter => DEBUG_ADAPTERS,
        ToolchainRole::SerialMonitor => SERIAL_MONITORS,
    }
}

/// O candidato existe no catalogo do papel?
#[must_use]
pub(super) fn is_known(role: ToolchainRole, id: &str) -> bool {
    candidates_for(role).iter().any(|entry| entry.id == id)
}

// Cross-compilador NAO e' um papel novo, e a medicao de 2026-09-03 corrigiu o
// esboco do `roadmaps/35` §5.3 que dizia que era. `arm-none-eabi-gcc` escreve a
// MESMA variavel que o `gcc`: `CMAKE_C_COMPILER`. Dois papeis apontando para a
// mesma variavel seria a duplicacao que este projeto passa o dia removendo — o
// cross e' um CANDIDATO do papel que ja existe.
static C_COMPILERS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "clang",
        label: "Clang",
        tool_id: "clang",
    },
    RoleCandidate {
        id: "gcc",
        label: "GCC",
        tool_id: "gcc",
    },
    RoleCandidate {
        id: "arm-none-eabi-gcc",
        label: "GCC (ARM bare-metal)",
        tool_id: "arm-none-eabi-gcc",
    },
    // Os triples do integracoes/39 (2026-09-12), bare metal e Linux embarcado.
    RoleCandidate {
        id: "riscv-none-elf-gcc",
        label: "GCC (RISC-V bare-metal)",
        tool_id: "riscv-none-elf-gcc",
    },
    RoleCandidate {
        id: "riscv32-esp-elf-gcc",
        label: "GCC (Espressif RISC-V)",
        tool_id: "riscv32-esp-elf-gcc",
    },
    RoleCandidate {
        id: "xtensa-esp-elf-gcc",
        label: "GCC (Espressif Xtensa)",
        tool_id: "xtensa-esp-elf-gcc",
    },
    RoleCandidate {
        id: "aarch64-linux-gnu-gcc",
        label: "GCC (AArch64 Linux)",
        tool_id: "aarch64-linux-gnu-gcc",
    },
    RoleCandidate {
        id: "arm-linux-gnueabihf-gcc",
        label: "GCC (ARM 32 Linux hard-float)",
        tool_id: "arm-linux-gnueabihf-gcc",
    },
    RoleCandidate {
        id: "riscv64-linux-gnu-gcc",
        label: "GCC (RISC-V 64 Linux)",
        tool_id: "riscv64-linux-gnu-gcc",
    },
];

static CXX_COMPILERS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "clangxx",
        label: "Clang++",
        tool_id: "clangxx",
    },
    RoleCandidate {
        id: "gxx",
        label: "G++",
        tool_id: "gxx",
    },
    RoleCandidate {
        id: "arm-none-eabi-gxx",
        label: "G++ (ARM bare-metal)",
        tool_id: "arm-none-eabi-gxx",
    },
    RoleCandidate {
        id: "riscv-none-elf-gxx",
        label: "G++ (RISC-V bare-metal)",
        tool_id: "riscv-none-elf-gxx",
    },
    RoleCandidate {
        id: "riscv32-esp-elf-gxx",
        label: "G++ (Espressif RISC-V)",
        tool_id: "riscv32-esp-elf-gxx",
    },
    RoleCandidate {
        id: "xtensa-esp-elf-gxx",
        label: "G++ (Espressif Xtensa)",
        tool_id: "xtensa-esp-elf-gxx",
    },
    RoleCandidate {
        id: "aarch64-linux-gnu-gxx",
        label: "G++ (AArch64 Linux)",
        tool_id: "aarch64-linux-gnu-gxx",
    },
    RoleCandidate {
        id: "arm-linux-gnueabihf-gxx",
        label: "G++ (ARM 32 Linux hard-float)",
        tool_id: "arm-linux-gnueabihf-gxx",
    },
    RoleCandidate {
        id: "riscv64-linux-gnu-gxx",
        label: "G++ (RISC-V 64 Linux)",
        tool_id: "riscv64-linux-gnu-gxx",
    },
];

static GENERATORS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "Ninja",
        label: "Ninja",
        tool_id: "ninja",
    },
    RoleCandidate {
        id: "Unix Makefiles",
        label: "Unix Makefiles",
        tool_id: "make",
    },
];

static CMAKE: &[RoleCandidate] = &[RoleCandidate {
    id: "cmake",
    label: "CMake",
    tool_id: "cmake",
}];

static CARGO: &[RoleCandidate] = &[RoleCandidate {
    id: "cargo",
    label: "Cargo",
    tool_id: "cargo",
}];

/// Adaptadores de debug (DAP).
///
/// A ordem e' a preferencia: `lldb-dap` primeiro porque e' o alvo de desktop, e
/// era a CONSTANTE que este papel substituiu. `probe-rs` vem em seguida porque
/// fala DAP nativamente por stdin/stdout — a mesma forma que o dominio `dap/`
/// ja usa (`integracoes/36` §3).
///
/// **Este arquivo nao sabe COMO invocar cada um**, e a fronteira e' deliberada:
/// o cabecalho deste modulo diz que ele responde "quais binarios interessam a
/// cada papel", nao "com quais argumentos". `probe-rs` precisa do subcomando
/// `dap-server` e `lldb-dap` nao precisa de nenhum — esse conhecimento e' do
/// dominio `dap`, que e quem sobe o processo.
static DEBUG_ADAPTERS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "lldb-dap",
        label: "lldb-dap",
        tool_id: "lldb-dap",
    },
    RoleCandidate {
        id: "probe-rs",
        label: "probe-rs (embarcado)",
        tool_id: "probe-rs",
    },
    // O GDB fala DAP desde a v14 (`/usr/share/doc/gdb/NEWS`). E' a ponte para
    // tudo que fala GDB remote — QEMU, OpenOCD, pyOCD, `probe-rs gdb` — e por
    // isso o `integracoes/36` §3 foi corrigido em 2026-09-11: nao ha' protocolo
    // novo a escrever. O kit diz a que porta conectar (`remoteTarget`).
    RoleCandidate {
        id: "gdb",
        label: "GDB (alvo remoto: QEMU, OpenOCD)",
        tool_id: "gdb",
    },
    // Os GDBs de alvo (integracoes/39): falam DAP como o `gdb` do sistema —
    // e' o `dap/adapter.rs` que reconhece qualquer id de GDB pelo nome.
    RoleCandidate {
        id: "gdb-multiarch",
        label: "GDB multi-arquitetura (Debian/Ubuntu)",
        tool_id: "gdb-multiarch",
    },
    RoleCandidate {
        id: "arm-none-eabi-gdb",
        label: "GDB ARM bare-metal (tarball da Arm)",
        tool_id: "arm-none-eabi-gdb",
    },
    RoleCandidate {
        id: "xtensa-esp-elf-gdb",
        label: "GDB Espressif Xtensa",
        tool_id: "xtensa-esp-elf-gdb",
    },
    RoleCandidate {
        id: "riscv32-esp-elf-gdb",
        label: "GDB Espressif RISC-V",
        tool_id: "riscv32-esp-elf-gdb",
    },
];

/// Monitor serial (E3 do `integracoes/38` §6, 2026-09-12): o processo que a
/// IDE abre numa aba de terminal sobre a porta. A ordem e' a preferencia
/// AUTOMATICA para uma porta qualquer: `tio` e' o padrao de mercado moderno,
/// `picocom` e `minicom` sao os classicos que quase toda distro tem. O
/// `espflash` vem por ultimo NO CATALOGO e por primeiro NA PRATICA quando o
/// kit e' Espressif — quem decide isso e' o `serial.monitor`, que conhece o
/// chip; este arquivo, como sempre, so' diz quais binarios interessam.
static SERIAL_MONITORS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "tio",
        label: "tio",
        tool_id: "tio",
    },
    RoleCandidate {
        id: "picocom",
        label: "picocom",
        tool_id: "picocom",
    },
    RoleCandidate {
        id: "minicom",
        label: "minicom",
        tool_id: "minicom",
    },
    RoleCandidate {
        id: "espflash",
        label: "espflash monitor (Espressif: decodifica backtrace)",
        tool_id: "espflash",
    },
];
