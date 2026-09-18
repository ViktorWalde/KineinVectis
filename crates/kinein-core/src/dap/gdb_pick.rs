//! O GDB certo para o ELF (pente-fino de 2026-09-18).
//!
//! O que se mediu: o `gdb` nativo desta maquina (Ubuntu, 17.1) nao fala ARM;
//! com o kit escolhendo "gdb" e um ELF Cortex-M do QEMU, o `attach` fica
//! MUDO e a IDE espera para sempre (o gate `verificar-embarcado.sh` parava
//! em "event.debug.stopped nao chegou em 30 s"). O `gdb-multiarch` ao lado
//! resolve. A causa e' decidivel ANTES de subir o adaptador: o `e_machine` do
//! ELF diz a arquitetura, e o catalogo diz quais GDBs de alvo existem.
//!
//! ```text
//! e_machine     ARM 0x28 · AArch64 0xB7 · RISC-V 0xF3 · Xtensa 0x5E · x86-64 0x3E
//! preferencia   ARM      arm-none-eabi-gdb, gdb-multiarch
//!               Xtensa   xtensa-esp-elf-gdb, gdb-multiarch
//!               RISC-V   riscv32-esp-elf-gdb, gdb-multiarch
//!               AArch64  gdb-multiarch
//! ```
//!
//! So' troca quando o escolhido e' o `gdb` nu (fixado ou automatico) e o ELF
//! e' de outra maquina; um GDB de alvo escolhido a dedo fica como esta'.
//! Sem nenhum GDB que sirva, a resposta e' um erro que diz o pacote — nunca
//! o silencio.

use std::path::Path;

use kinein_protocol::{ToolInfo, ToolStatus};

/// A arquitetura de um ELF, pelo `e_machine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfMachine {
    /// `EM_ARM` (Cortex-M/A de 32 bits).
    Arm,
    /// `EM_AARCH64`.
    AArch64,
    /// `EM_RISCV`.
    RiscV,
    /// `EM_XTENSA` (ESP32 classico/S2/S3).
    Xtensa,
    /// `EM_X86_64`.
    X86_64,
    /// Outra, com o codigo.
    Other(u16),
}

impl ElfMachine {
    /// O nome como o autor le.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Arm => "ARM".to_owned(),
            Self::AArch64 => "AArch64".to_owned(),
            Self::RiscV => "RISC-V".to_owned(),
            Self::Xtensa => "Xtensa".to_owned(),
            Self::X86_64 => "x86-64".to_owned(),
            Self::Other(code) => format!("e_machine {code:#x}"),
        }
    }

    /// Os GDBs que falam esta arquitetura, em ordem de preferencia.
    #[must_use]
    pub const fn gdb_candidates(self) -> &'static [&'static str] {
        match self {
            Self::Arm => &["arm-none-eabi-gdb", "gdb-multiarch"],
            Self::Xtensa => &["xtensa-esp-elf-gdb", "gdb-multiarch"],
            Self::RiscV => &["riscv32-esp-elf-gdb", "gdb-multiarch"],
            Self::AArch64 | Self::Other(_) => &["gdb-multiarch"],
            Self::X86_64 => &["gdb"],
        }
    }

    /// O pacote que traz um GDB para esta arquitetura, para a mensagem.
    #[must_use]
    pub const fn install_hint(self) -> &'static str {
        match self {
            Self::Xtensa | Self::RiscV => {
                "instale o GDB da Espressif (xtensa-esp-elf-gdb / riscv32-esp-elf-gdb, pelo ESP-IDF) ou o gdb-multiarch (apt install gdb-multiarch)"
            }
            _ => {
                "instale o gdb-multiarch (apt install gdb-multiarch) ou o arm-none-eabi-gdb da Arm"
            }
        }
    }
}

/// A arquitetura do host que este core roda.
#[must_use]
pub const fn host_machine() -> ElfMachine {
    if cfg!(target_arch = "x86_64") {
        ElfMachine::X86_64
    } else if cfg!(target_arch = "aarch64") {
        ElfMachine::AArch64
    } else {
        ElfMachine::Other(0)
    }
}

/// Le o `e_machine` do ELF; `None` se o arquivo nao e' ELF (um script, um
/// binario de outro formato).
#[must_use]
pub fn elf_machine(path: &Path) -> Option<ElfMachine> {
    let bytes = std::fs::read(path).ok()?;
    parse_elf_machine(&bytes)
}

/// [`elf_machine`] sobre bytes (puro, para o teste).
#[must_use]
pub fn parse_elf_machine(bytes: &[u8]) -> Option<ElfMachine> {
    if bytes.len() < 20 || &bytes[..4] != b"\x7fELF" {
        return None;
    }
    let little = bytes[5] == 1;
    let raw = [bytes[18], bytes[19]];
    let code = if little {
        u16::from_le_bytes(raw)
    } else {
        u16::from_be_bytes(raw)
    };
    Some(match code {
        0x28 => ElfMachine::Arm,
        0xB7 => ElfMachine::AArch64,
        0xF3 => ElfMachine::RiscV,
        0x5E => ElfMachine::Xtensa,
        0x3E => ElfMachine::X86_64,
        other => ElfMachine::Other(other),
    })
}

/// O que `debug.start` faz com a escolha de adaptador diante do ELF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GdbPick {
    /// A escolha serve como esta'.
    Keep,
    /// Trocar o `gdb` nu por este (id, caminho), e dizer por que.
    Replace {
        /// Id do GDB de alvo.
        id: String,
        /// Caminho detectado.
        path: String,
        /// A frase para o console de debug.
        reason: String,
    },
    /// Nenhum GDB desta maquina fala a arquitetura do ELF.
    Unavailable {
        /// A frase, com o pacote.
        message: String,
    },
}

/// Decide pelo `e_machine` do ELF e pelas ferramentas detectadas.
#[must_use]
pub fn pick_gdb(chosen_id: &str, program: &Path, tools: &[ToolInfo]) -> GdbPick {
    if chosen_id != "gdb" {
        return GdbPick::Keep;
    }
    let Some(machine) = elf_machine(program) else {
        return GdbPick::Keep;
    };
    if machine == host_machine() {
        return GdbPick::Keep;
    }
    for candidate in machine.gdb_candidates() {
        if let Some(tool) = tools
            .iter()
            .find(|t| t.id == *candidate && t.status == ToolStatus::Detected)
            && let Some(path) = &tool.path
        {
            return GdbPick::Replace {
                id: (*candidate).to_owned(),
                path: path.clone(),
                reason: format!(
                    "o ELF e' {} e o gdb desta maquina so' fala {}: usando {candidate}",
                    machine.label(),
                    host_machine().label()
                ),
            };
        }
    }
    GdbPick::Unavailable {
        message: format!(
            "o ELF e' {} e o gdb desta maquina so' fala {}; {}",
            machine.label(),
            host_machine().label(),
            machine.install_hint()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn elf(machine: u16) -> Vec<u8> {
        let mut b = vec![0u8; 64];
        b[..4].copy_from_slice(b"\x7fELF");
        b[4] = 1;
        b[5] = 1;
        b[18..20].copy_from_slice(&machine.to_le_bytes());
        b
    }

    fn tool(id: &str, detected: bool) -> ToolInfo {
        ToolInfo {
            id: id.to_owned(),
            display_name: id.to_owned(),
            status: if detected {
                ToolStatus::Detected
            } else {
                ToolStatus::Missing
            },
            path: detected.then(|| format!("/usr/bin/{id}")),
            version: None,
            message: None,
            suggested_install: None,
        }
    }

    #[test]
    fn the_machine_is_read_from_e_machine_in_both_endiannesses() {
        assert_eq!(parse_elf_machine(&elf(0x28)), Some(ElfMachine::Arm));
        assert_eq!(parse_elf_machine(&elf(0x3E)), Some(ElfMachine::X86_64));
        assert_eq!(parse_elf_machine(&elf(0x5E)), Some(ElfMachine::Xtensa));
        let mut big = elf(0);
        big[5] = 2;
        big[18..20].copy_from_slice(&0xF3u16.to_be_bytes());
        assert_eq!(parse_elf_machine(&big), Some(ElfMachine::RiscV));
        assert_eq!(parse_elf_machine(b"#!/usr/bin/env python3\n"), None);
        assert_eq!(
            parse_elf_machine(&elf(0x1234)),
            Some(ElfMachine::Other(0x1234))
        );
    }

    #[test]
    fn a_foreign_elf_with_the_bare_gdb_gets_a_target_gdb_or_an_actionable_error() {
        let dir = std::env::temp_dir().join(format!("kinein-gdb-pick-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let arm = dir.join("arm.elf");
        std::fs::write(&arm, elf(0x28)).unwrap();
        let host = dir.join("host.elf");
        std::fs::write(
            &host,
            elf(if cfg!(target_arch = "x86_64") {
                0x3E
            } else {
                0xB7
            }),
        )
        .unwrap();

        // ELF da maquina, ou GDB de alvo escolhido a dedo: fica.
        assert_eq!(
            pick_gdb("gdb", &host, &[tool("gdb-multiarch", true)]),
            GdbPick::Keep
        );
        assert_eq!(pick_gdb("arm-none-eabi-gdb", &arm, &[]), GdbPick::Keep);
        assert_eq!(pick_gdb("probe-rs", &arm, &[]), GdbPick::Keep);
        // Um script nao e' ELF: fica.
        let py = dir.join("a.py");
        std::fs::write(&py, "print(1)\n").unwrap();
        assert_eq!(pick_gdb("gdb", &py, &[]), GdbPick::Keep);

        // ARM com o gdb nu: o da Arm antes do multiarch; so' os DETECTADOS.
        let escolha = pick_gdb(
            "gdb",
            &arm,
            &[
                tool("gdb-multiarch", true),
                tool("arm-none-eabi-gdb", false),
            ],
        );
        match escolha {
            GdbPick::Replace { id, path, reason } => {
                assert_eq!(id, "gdb-multiarch");
                assert_eq!(path, "/usr/bin/gdb-multiarch");
                assert!(
                    reason.contains("ARM") && reason.contains("gdb-multiarch"),
                    "{reason}"
                );
            }
            outro => panic!("{outro:?}"),
        }
        match pick_gdb(
            "gdb",
            &arm,
            &[tool("gdb-multiarch", true), tool("arm-none-eabi-gdb", true)],
        ) {
            GdbPick::Replace { id, .. } => assert_eq!(id, "arm-none-eabi-gdb"),
            outro => panic!("{outro:?}"),
        }
        // Sem nenhum: o erro diz o pacote.
        match pick_gdb("gdb", &arm, &[tool("gdb", true)]) {
            GdbPick::Unavailable { message } => {
                assert!(message.contains("gdb-multiarch"), "{message}");
            }
            outro => panic!("{outro:?}"),
        }
    }
}
