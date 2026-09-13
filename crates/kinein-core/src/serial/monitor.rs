//! O monitor serial como PROCESSO PRONTO numa aba de terminal.
//!
//! E' a E3 do `integracoes/38` §6, decisao do autor em 2026-09-11: nunca
//! codigo serial nosso — a forma da extensao oficial da Espressif para o VS
//! Code, que abre `idf.py monitor` num terminal.
//!
//! Este modulo decide QUAL ferramenta e COM QUE ARGUMENTOS; quem sobe o
//! processo e' o dominio `terminal` (`open_command`), como no `container.open`.
//!
//! # Linhas de comando, lidas na fonte em 2026-09-11/12
//!
//! ```text
//! tio       tio -b <baud> <dev>                       (tio 3.x: -b, --baudrate)
//! picocom   picocom -b <baud> <dev>
//! minicom   minicom -D <dev> -b <baud>
//! espflash  espflash monitor --port <dev> --monitor-baud <baud> [--elf <ELF>]
//!           ATENCAO: o `-B/--baud` do espflash e' o baud de GRAVACAO; o do
//!           monitor e' `-r/--monitor-baud` (espflash/src/cli/mod.rs, 4.6.0)
//! mpremote  mpremote connect <dev> repl       (1.29.0, `mpremote connect --help`:
//!           `connect device next_command`; o REPL nao tem baud — e' o raw REPL
//!           do `MicroPython` a 115200). Fatia 5 da cadeia Python, 2026-09-13
//! ```

use std::path::Path;

use kinein_protocol::ToolchainRole;

use crate::toolchain::Toolchain;

/// O baud que ESP32, Pico stdio e a maioria dos firmwares usam por padrao.
pub const DEFAULT_BAUD: u32 = 115_200;

/// A ferramenta escolhida: id do catalogo e caminho do executavel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Escolha {
    /// `tio`, `picocom`, `minicom`, `espflash`.
    pub id: String,
    /// O executavel resolvido.
    pub program: String,
}

/// Qual monitor abrir para este kit.
///
/// Fixado pelo autor no papel `serialMonitor` -> esse. Senao, num projeto
/// `MicroPython` com `mpremote` detectado -> o REPL da placa (`mpremote connect
/// <dev> repl`): num firmware `MicroPython` o "monitor" E' o REPL, e um tio a
/// 115200 mostraria o mesmo texto sem o raw-paste nem o Ctrl-] de sair.
/// Senao, se o chip do kit e' Espressif E o `espflash` esta' detectado ->
/// `espflash monitor`, que decodifica o backtrace. Senao, o efetivo do papel
/// (o primeiro detectado na ordem do catalogo: tio, picocom, minicom,
/// espflash — o mpremote e' o ultimo e nunca vence sozinho).
#[must_use]
pub fn escolher(toolchain: &Toolchain, micropython: bool) -> Option<Escolha> {
    let (efetivo_id, efetivo_path) = toolchain.effective_program(ToolchainRole::SerialMonitor)?;
    let fixado = toolchain.chosen(ToolchainRole::SerialMonitor).is_some();
    if !fixado && micropython {
        if let Some(mpremote) = toolchain.candidate_path(ToolchainRole::SerialMonitor, "mpremote") {
            return Some(Escolha {
                id: "mpremote".to_owned(),
                program: mpremote.to_owned(),
            });
        }
    }
    if !fixado && chip_e_espressif(toolchain.chip()) {
        if let Some(espflash) = toolchain.candidate_path(ToolchainRole::SerialMonitor, "espflash") {
            return Some(Escolha {
                id: "espflash".to_owned(),
                program: espflash.to_owned(),
            });
        }
    }
    Some(Escolha {
        id: efetivo_id.to_owned(),
        program: efetivo_path.to_owned(),
    })
}

/// `esp32`, `ESP32-C3`, `esp32s3`... — o que o `probe-rs chip list` e o
/// `esptool chip-id` chamam de chip Espressif.
fn chip_e_espressif(chip: Option<&str>) -> bool {
    chip.is_some_and(|c| c.trim().to_ascii_lowercase().starts_with("esp32"))
}

/// Programa e argumentos para a aba de terminal. Funcao pura: e' o que se
/// testa sem porta e sem ferramenta.
#[must_use]
pub fn command_line(
    tool_id: &str,
    program: &str,
    device: &str,
    baud: u32,
    elf: Option<&Path>,
) -> (String, Vec<String>) {
    let baud = baud.to_string();
    let args = match tool_id {
        "espflash" => {
            let mut args = vec![
                "monitor".to_owned(),
                "--port".to_owned(),
                device.to_owned(),
                "--monitor-baud".to_owned(),
                baud,
            ];
            if let Some(elf) = elf {
                args.push("--elf".to_owned());
                args.push(elf.display().to_string());
            }
            args
        }
        "minicom" => vec!["-D".to_owned(), device.to_owned(), "-b".to_owned(), baud],
        // O REPL nao tem baud: o mpremote fala o raw REPL do MicroPython.
        "mpremote" => vec!["connect".to_owned(), device.to_owned(), "repl".to_owned()],
        // tio e picocom: `-b <baud> <dev>`, e qualquer outro monitor que
        // entre no catalogo com essa forma classica.
        _ => vec!["-b".to_owned(), baud, device.to_owned()],
    };
    (program.to_owned(), args)
}
