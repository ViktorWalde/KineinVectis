//! E5 do `integracoes/38` §6: a identidade Espressif PELO CANAL.
//!
//! O VID:PID de uma ponte `CP210x` diz "ponte" e nada sobre o chip atras dela
//! (`serial/mod.rs::familia`). Quem sabe o chip e' o proprio bootloader de
//! ROM, e o `esptool` e' quem fala com ele: `esptool --port <porta> flash-id`
//! imprime o chip, as features, o cristal, o MAC e a flash SPI. Esta unidade
//! monta essa linha e LE a resposta; quem roda e' o job (`handlers/serial`).
//!
//! # Por que o parser e' TOLERANTE (a mesma decisao do `probe.rs`)
//!
//! O formato foi levantado do CODIGO do esptool 5.4.0 instalado nesta maquina
//! (`esptool/__init__.py` ~l.558-575 e `cmds.py::print_flash_id`/`flash_id`,
//! 2026-09-17), nao de uma placa: linha que casa vira campo; linha que nao
//! casa e' ignorada; `raw` volta inteiro para a tela mostrar o que o parser
//! nao entendeu. Um parser rigido diria "nenhum chip" na primeira versao que
//! mudasse um rotulo — a pior mentira possivel aqui.
//!
//! # Por que abrir a porta e' gesto explicito
//!
//! O esptool puxa DTR/RTS para entrar no bootloader: a placa RESETA. Um
//! `serial.list` que identificasse ao abrir o painel reiniciaria o firmware
//! do usuario a cada olhada. Por isso `serial.identify` existe separado e a
//! tela so' o pede num clique.

use std::path::Path;

use kinein_protocol::SerialIdentity;

/// Subcomando na v5 (2025+): comandos com hifen, binario `esptool`.
pub const FLASH_ID_V5: &str = "flash-id";
/// Subcomando na v4: `flash_id` (o `esptool.py` classico). O job tenta a v5
/// e cai para a v4 quando a ferramenta nao conhece o comando.
pub const FLASH_ID_V4: &str = "flash_id";

/// A linha de comando: `esptool --port <device> --chip auto <subcomando>`.
///
/// `--chip auto` e' o padrao do esptool, escrito para o comando ecoado dizer
/// o que a IDE pediu; `--before default-reset` idem. O `--after hard-reset`
/// devolve a placa ao firmware dela quando a leitura termina — sem isso ela
/// ficaria parada no bootloader ate' o proximo reset.
#[must_use]
pub fn command_line(esptool: &Path, device: &str, subcommand: &str) -> (String, Vec<String>) {
    (
        esptool.display().to_string(),
        vec![
            "--port".to_owned(),
            device.to_owned(),
            "--chip".to_owned(),
            "auto".to_owned(),
            "--before".to_owned(),
            "default-reset".to_owned(),
            "--after".to_owned(),
            "hard-reset".to_owned(),
            subcommand.to_owned(),
        ],
    )
}

/// Como a tela mostra o comando (argumentos separados por espaco; a porta
/// nunca tem espaco, e' um no' de `/dev`).
#[must_use]
pub fn display(program: &str, args: &[String]) -> String {
    let mut partes = vec![program.to_owned()];
    partes.extend(args.iter().cloned());
    partes.join(" ")
}

/// A ferramenta nao conhece o subcomando.
///
/// E' uma v4 recebendo `flash-id` (Click: "No such command") ou uma v5
/// recebendo `flash_id`. Medido no esptool 5.4.0 (`rich-click`): `Error: No
/// such command 'flash_id'.`; a v4 (argparse) diz `invalid choice: 'flash-id'`.
#[must_use]
pub fn unknown_subcommand(saida: &str) -> bool {
    saida.contains("No such command") || saida.contains("invalid choice")
}

/// Le o que o `flash-id` imprimiu. Nunca falha: o que nao casa fica em `raw`.
#[must_use]
pub fn parse_flash_id(saida: &str) -> SerialIdentity {
    let mut id = SerialIdentity::default();
    for linha in saida.lines() {
        let linha = linha.trim_end_matches('\r');
        if let Some(v) = campo(linha, "Chip type:") {
            id.chip = chip_key(v);
            id.chip_description = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "Features:") {
            id.features = v
                .split(',')
                .map(str::trim)
                .filter(|f| !f.is_empty())
                .map(str::to_owned)
                .collect();
        } else if let Some(v) = campo(linha, "Crystal frequency:") {
            id.crystal = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "USB mode:") {
            id.usb_mode = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "MAC:") {
            // `BASE MAC:`/`MAC_EXT:` sao outros rotulos e nao casam aqui.
            id.mac = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "Manufacturer:") {
            id.flash_manufacturer = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "Device:") {
            id.flash_device = Some(v.to_owned());
        } else if let Some(v) = campo(linha, "Detected flash size:") {
            if !v.eq_ignore_ascii_case("unknown") {
                id.flash_size = Some(v.to_owned());
                id.flash_size_bytes = tamanho_em_bytes(v);
            }
        }
    }
    id
}

/// `Rotulo:   valor` -> `valor`, so' quando a linha COMECA pelo rotulo (o
/// esptool alinha com `{:<20}`; o valor vem depois dos espacos).
fn campo<'a>(linha: &'a str, rotulo: &str) -> Option<&'a str> {
    let resto = linha.trim_start().strip_prefix(rotulo)?;
    // `MAC:` nao pode casar `MAC_EXT:`; o rotulo inteiro termina em ':' e o
    // que vem depois e' espaco ou o valor.
    let valor = resto.trim();
    (resto.is_empty() || resto.starts_with(' ') || resto.starts_with('\t')).then_some(valor)
}

/// As series do IDF (`idf.py --list-targets`, ESP-IDF 5.x): so' elas
/// entram na chave; qualquer outro sufixo e' encapsulamento do classico.
const SERIES: [&str; 8] = ["s2", "s3", "c2", "c3", "c5", "c6", "h2", "p4"];

/// `ESP32-C3 (QFN32) (revision v0.4)` -> `esp32c3`; `ESP32-D0WD-V3
/// (revision v3.1)` -> `esp32`.
///
/// A chave que o kit, o `IDF_TARGET` e as tabelas do `project/mod.rs` usam:
/// `esp32` mais o sufixo da serie (`s2`, `s3`, `c2`…`p4`) quando ha' um.
/// O que vem depois do `ESP32-` no chip classico e' o ENCAPSULAMENTO
/// (`D0WD-V3`, `D0WDQ6`, `PICO-D4`, `U4WDH`, `S0WD`), nao a serie: medido no
/// ESP32 do autor em 2026-09-17 (`ESP32-D0WD-V3`), a chave e' `esp32` — a
/// primeira versao devolvia `esp32d0wdv3`, que nenhum kit conhece. O
/// `ESP8685 (QFN28)` e' um C3 de outro encapsulamento e o esptool o nomeia
/// pelo numero comercial: a tabela abaixo o devolve ao alvo do IDF.
#[must_use]
pub fn chip_key(descricao: &str) -> Option<String> {
    let token = descricao.split_whitespace().next()?;
    let chave = token.to_ascii_lowercase();
    // Variantes de encapsulamento que o esptool nomeia pelo numero comercial
    // (`targets/esp32c3.py`): sao o mesmo alvo do IDF.
    match chave.as_str() {
        "esp8685" | "esp8686" => return Some("esp32c3".to_owned()),
        "esp8684" => return Some("esp32c2".to_owned()),
        "esp8266" | "esp8266ex" => return Some("esp8266".to_owned()),
        _ => {}
    }
    let serie = chave.strip_prefix("esp32")?;
    let serie = serie.trim_start_matches('-');
    let sufixo = SERIES.iter().find(|s| serie == **s).copied().unwrap_or("");
    Some(format!("esp32{sufixo}"))
}

/// `4MB` -> 4194304; `512KB` -> 524288. Outra grafia -> `None`.
fn tamanho_em_bytes(texto: &str) -> Option<u64> {
    let texto = texto.trim().to_ascii_uppercase();
    let (numero, mult) = if let Some(n) = texto.strip_suffix("MB") {
        (n, 1024 * 1024)
    } else if let Some(n) = texto.strip_suffix("KB") {
        (n, 1024)
    } else {
        return None;
    };
    numero.trim().parse::<u64>().ok().map(|n| n * mult)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A saida REAL do `flash-id` (esptool 5.4.0) no ESP32 do autor
    /// (CP2102, `/dev/ttyUSB0`), medida em 2026-09-17 — substituiu a fixture
    /// escrita a partir do codigo. Diferencas em relacao a ela: ha' um
    /// cabecalho (`Serial port`, `Detecting chip type...`, `Connected to`) e
    /// as linhas do stub flasher ANTES do bloco de flash; `Flash voltage set
    /// by a strapping pin` no lugar de `by eFuse`; sem `USB mode:` (o ESP32
    /// classico nao tem USB nativo). O parser tolerante nao precisou mudar;
    /// a chave do chip precisou (`esp32`, nao `esp32d0wdv3`).
    const FIXTURE_V5: &str = "\
esptool v5.4.0
Serial port /dev/ttyUSB0:
Connecting.....
Detecting chip type... ESP32
Connected to ESP32 on /dev/ttyUSB0:
Chip type:          ESP32-D0WD-V3 (revision v3.1)
Features:           Wi-Fi, BT, Dual Core + LP Core, 240MHz, Vref calibration in eFuse, Coding Scheme None
Crystal frequency:  40MHz
MAC:                70:4b:ca:04:54:54

Uploading stub flasher...
Running stub flasher...
Stub flasher running.

Flash Memory Information:
=========================
Manufacturer: 5e
Device: 4016
Detected flash size: 4MB
Flash voltage set by a strapping pin: 3.3V

Hard resetting via RTS pin...
";

    /// A fixture anterior, escrita a partir do CODIGO do esptool com os
    /// valores de um C3 (fica como o segundo formato que o parser cobre:
    /// `USB mode:` e o bloco de eFuse).
    const FIXTURE_C3: &str = "\
esptool v5.4.0
Connected to ESP32-C3 on /dev/ttyUSB0:
Chip type:          ESP32-C3 (QFN32) (revision v0.4)
Features:           Wi-Fi, BLE, Single Core + LP Core, 160MHz
Crystal frequency:  40MHz
USB mode:           USB-Serial/JTAG
MAC:                34:b4:72:0a:1b:2c

Flash Memory Information:
=========================
Manufacturer: c8
Device: 4016
Detected flash size: 4MB
Flash type set in eFuse: quad (4 data lines)
Flash voltage set by eFuse to 3.3V

Hard resetting via RTS pin...
";

    #[test]
    fn reads_chip_flash_and_mac_from_the_real_esp32_output() {
        let id = parse_flash_id(FIXTURE_V5);
        assert_eq!(id.chip.as_deref(), Some("esp32"));
        assert_eq!(
            id.chip_description.as_deref(),
            Some("ESP32-D0WD-V3 (revision v3.1)")
        );
        assert_eq!(id.features.len(), 6);
        assert_eq!(id.features[2], "Dual Core + LP Core");
        assert_eq!(id.crystal.as_deref(), Some("40MHz"));
        assert_eq!(id.usb_mode, None, "o ESP32 classico nao tem USB nativo");
        assert_eq!(id.mac.as_deref(), Some("70:4b:ca:04:54:54"));
        assert_eq!(id.flash_manufacturer.as_deref(), Some("5e"));
        assert_eq!(id.flash_device.as_deref(), Some("4016"));
        assert_eq!(id.flash_size.as_deref(), Some("4MB"));
        assert_eq!(id.flash_size_bytes, Some(4 * 1024 * 1024));
    }

    #[test]
    fn reads_chip_flash_and_mac_from_the_c3_output() {
        let id = parse_flash_id(FIXTURE_C3);
        assert_eq!(id.chip.as_deref(), Some("esp32c3"));
        assert_eq!(
            id.chip_description.as_deref(),
            Some("ESP32-C3 (QFN32) (revision v0.4)")
        );
        assert_eq!(
            id.features,
            ["Wi-Fi", "BLE", "Single Core + LP Core", "160MHz"]
        );
        assert_eq!(id.crystal.as_deref(), Some("40MHz"));
        assert_eq!(id.usb_mode.as_deref(), Some("USB-Serial/JTAG"));
        assert_eq!(id.mac.as_deref(), Some("34:b4:72:0a:1b:2c"));
        assert_eq!(id.flash_manufacturer.as_deref(), Some("c8"));
        assert_eq!(id.flash_device.as_deref(), Some("4016"));
        assert_eq!(id.flash_size.as_deref(), Some("4MB"));
        assert_eq!(id.flash_size_bytes, Some(4 * 1024 * 1024));
    }

    /// Flash desconhecida nao vira "Unknown" na tela; MAC de 64 bits
    /// (`BASE MAC:`/`MAC_EXT:`) nao confunde o `MAC:`; linha estranha e'
    /// ignorada, nao derruba o resto.
    #[test]
    fn unknown_flash_extra_macs_and_odd_lines_are_tolerated() {
        let id = parse_flash_id(
            "Connecting....\nChip type:          ESP32-H2 (revision v0.1)\nMAC:                \
             11:22:33:44:55:66:77:88\nBASE MAC:           11:22:33:44:55:66\nMAC_EXT:  \
             77:88\nrotulo novo: valor\nDetected flash size: Unknown\n",
        );
        assert_eq!(id.chip.as_deref(), Some("esp32h2"));
        assert_eq!(id.mac.as_deref(), Some("11:22:33:44:55:66:77:88"));
        assert_eq!(id.flash_size, None);
        assert_eq!(id.flash_size_bytes, None);
        assert!(id.features.is_empty());
    }

    /// Nada casou: identidade vazia, sem panico — o job decide que e' falha.
    #[test]
    fn nothing_recognised_is_an_empty_identity() {
        let id = parse_flash_id("A fatal error occurred: Failed to connect to Espressif device\n");
        assert_eq!(id, SerialIdentity::default());
    }

    #[test]
    fn chip_keys_follow_the_idf_target_names() {
        assert_eq!(
            chip_key("ESP32-S3 (QFN56) (revision v0.2)").as_deref(),
            Some("esp32s3")
        );
        assert_eq!(
            chip_key("ESP32-D0WD-V3 (revision v3.1)").as_deref(),
            Some("esp32"),
            "encapsulamento do classico nao e' serie"
        );
        assert_eq!(chip_key("ESP32-PICO-D4").as_deref(), Some("esp32"));
        assert_eq!(chip_key("ESP32 (revision v1.0)").as_deref(), Some("esp32"));
        assert_eq!(chip_key("ESP32-C6 (QFN40)").as_deref(), Some("esp32c6"));
        assert_eq!(chip_key("ESP8266EX").as_deref(), Some("esp8266"));
        assert_eq!(chip_key("ESP8685 (QFN28)").as_deref(), Some("esp32c3"));
        assert_eq!(
            chip_key("ESP32-P4 (revision v1.0)").as_deref(),
            Some("esp32p4")
        );
        assert_eq!(chip_key("Unknown chip"), None);
    }

    #[test]
    fn sizes_parse_in_mb_and_kb_only() {
        assert_eq!(tamanho_em_bytes("16MB"), Some(16 * 1024 * 1024));
        assert_eq!(tamanho_em_bytes("512KB"), Some(512 * 1024));
        assert_eq!(tamanho_em_bytes("4 MB"), Some(4 * 1024 * 1024));
        assert_eq!(tamanho_em_bytes("muita"), None);
    }

    #[test]
    fn command_line_names_port_reset_policy_and_subcommand() {
        let (programa, args) = command_line(Path::new("/x/esptool"), "/dev/ttyACM0", FLASH_ID_V5);
        assert_eq!(programa, "/x/esptool");
        assert_eq!(
            display(&programa, &args),
            "/x/esptool --port /dev/ttyACM0 --chip auto --before default-reset --after hard-reset flash-id"
        );
        assert!(unknown_subcommand("Error: No such command 'flash_id'."));
        assert!(unknown_subcommand(
            "esptool.py: error: argument operation: invalid choice: 'flash-id'"
        ));
        assert!(!unknown_subcommand(
            "A fatal error occurred: Could not open /dev/ttyUSB0"
        ));
    }
}
