//! O firmware `MicroPython` por placa (C5 do `roadmaps/41` bloco C).
//!
//! Releases PINADAS do micropython.org, baixadas pelo mesmo provedor das toolchains e
//! gravadas pelo motor do E4 (`flash.rs`).
//!
//! # A fonte, lida em 2026-09-17
//!
//! `micropython.org/download/<PLACA>/` publica os arquivos com data e versao
//! no nome (`ESP32_GENERIC-20260824-v1.29.0.bin`) e a instrucao de gravar:
//! ESP32 classico "starting at address 0x1000" (`esptool.py --baud 460800
//! write_flash 0x1000 …`), C3 e S3 "starting at address 0" (`write_flash 0`),
//! Pico/Pico W "copied to the USB mass storage device" (BOOTSEL) — o
//! `picotool load -f -x` do E4 faz a mesma coisa pelo USB. A pagina tambem
//! manda apagar a flash na primeira vez (`esptool.py erase_flash`), e isso
//! vai em `warnings` na proposta.
//!
//! # O checksum NAO vem da fonte
//!
//! O micropython.org nao publica SHA-256 dos firmwares (conferido nas cinco
//! paginas em 2026-09-17). O que esta' aqui foi MEDIDO no download desta
//! sessao (`sha256sum`, tamanho pelo `Content-Length`): protege contra um
//! download corrompido ou trocado depois da medicao — nao contra a fonte
//! ter sido trocada ANTES dela. O `source` diz isso, e a tela mostra.
//! Licenca: MIT (o firmware e' o repositorio micropython/micropython).

use super::catalog::{Entrada, Firmware, Kind};

const LICENCA: &str = "MIT (micropython/micropython LICENSE)";
const FONTE: &str = "micropython.org/download/<placa>; a fonte NAO publica checksum — \
                     SHA-256 medido no download de 2026-09-17";

/// Os firmwares, na ordem em que a tela lista.
pub const FIRMWARE: &[Entrada] = &[
    Entrada {
        kind: Kind::Firmware,
        firmware: Some(Firmware {
            board: "ESP32_GENERIC",
            engine: "esptool",
            offset: Some("0x1000"),
            chip: Some("esp32"),
        }),
        id: "micropython-esp32-generic",
        label: "MicroPython — ESP32_GENERIC (ESP32 classico, .bin)",
        version: "v1.29.0",
        family: "espressif",
        url: "https://micropython.org/resources/firmware/ESP32_GENERIC-20260824-v1.29.0.bin",
        size_bytes: 1_790_544,
        sha256: "e67ad6015a0a504c1fec9aa9bbf589d0432ed28e62546f4f8dd8a147f8bd95f6",
        license: LICENCA,
        source: FONTE,
    },
    Entrada {
        kind: Kind::Firmware,
        firmware: Some(Firmware {
            board: "ESP32_GENERIC_C3",
            engine: "esptool",
            offset: Some("0x0"),
            chip: Some("esp32c3"),
        }),
        id: "micropython-esp32-generic-c3",
        label: "MicroPython — ESP32_GENERIC_C3 (ESP32-C3, .bin)",
        version: "v1.29.0",
        family: "espressif",
        url: "https://micropython.org/resources/firmware/ESP32_GENERIC_C3-20260824-v1.29.0.bin",
        size_bytes: 1_754_736,
        sha256: "bf72ed9eb88ad3a8f49d02c3d371f9ea34c90a8a303aeb9e324d8efd4a2a655a",
        license: LICENCA,
        source: FONTE,
    },
    Entrada {
        kind: Kind::Firmware,
        firmware: Some(Firmware {
            board: "ESP32_GENERIC_S3",
            engine: "esptool",
            offset: Some("0x0"),
            chip: Some("esp32s3"),
        }),
        id: "micropython-esp32-generic-s3",
        label: "MicroPython — ESP32_GENERIC_S3 (ESP32-S3, .bin)",
        version: "v1.29.0",
        family: "espressif",
        url: "https://micropython.org/resources/firmware/ESP32_GENERIC_S3-20260824-v1.29.0.bin",
        size_bytes: 1_783_296,
        sha256: "1dd9dcd27ba2d6a1cb7f7b49a063f46d3d807d68b0151161233920228285e300",
        license: LICENCA,
        source: FONTE,
    },
    Entrada {
        kind: Kind::Firmware,
        firmware: Some(Firmware {
            board: "RPI_PICO",
            engine: "picotool",
            offset: None,
            chip: None,
        }),
        id: "micropython-rpi-pico",
        label: "MicroPython — RPI_PICO (Raspberry Pi Pico, .uf2)",
        version: "v1.29.0",
        family: "rp2040",
        url: "https://micropython.org/resources/firmware/RPI_PICO-20260824-v1.29.0.uf2",
        size_bytes: 680_960,
        sha256: "e1160e602e277d85920adb5fe741435edccba9ba6ecadc85fdab75252eeff4e9",
        license: LICENCA,
        source: FONTE,
    },
    Entrada {
        kind: Kind::Firmware,
        firmware: Some(Firmware {
            board: "RPI_PICO_W",
            engine: "picotool",
            offset: None,
            chip: None,
        }),
        id: "micropython-rpi-pico-w",
        label: "MicroPython — RPI_PICO_W (Raspberry Pi Pico W, .uf2)",
        version: "v1.29.0",
        family: "rp2040",
        url: "https://micropython.org/resources/firmware/RPI_PICO_W-20260824-v1.29.0.uf2",
        size_bytes: 1_811_456,
        sha256: "f918c0a082c6daf53e27998b7f990ba5f065db2e298b21a34024532a20786794",
        license: LICENCA,
        source: FONTE,
    },
];

/// O nome do arquivo na URL — e' o nome com que ele fica na pasta.
#[must_use]
pub fn file_name(e: &Entrada) -> &'static str {
    e.url.rsplit('/').next().unwrap_or("firmware.bin")
}

#[cfg(test)]
mod tests {
    use super::{FIRMWARE, file_name};
    use crate::toolchain::install::catalog::{Kind, entrada};

    /// Cada firmware e' pinado como uma toolchain (URL com versao e data,
    /// SHA-256 de 64 digitos), diz como se grava (motor; offset quando e' o
    /// esptool) e o `source` confessa que o checksum foi medido, nao lido.
    #[test]
    fn every_firmware_is_pinned_and_says_how_it_is_flashed() {
        let mut ids = Vec::new();
        for e in FIRMWARE {
            assert_eq!(e.kind, Kind::Firmware);
            assert!(!ids.contains(&e.id), "id repetido: {}", e.id);
            ids.push(e.id);
            let fw = e.firmware.expect("firmware diz como se grava");
            assert!(
                e.url
                    .starts_with("https://micropython.org/resources/firmware/")
            );
            assert!(
                e.url.contains(e.version) && e.url.contains("2026"),
                "{}",
                e.id
            );
            assert_eq!(e.sha256.len(), 64);
            assert!(
                e.sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            );
            assert!(
                e.size_bytes > 500_000 && e.size_bytes < 5_000_000,
                "{}",
                e.id
            );
            assert!(e.source.contains("medido") && e.source.contains("2026-09-17"));
            match fw.engine {
                "esptool" => {
                    assert!(
                        std::path::Path::new(e.url)
                            .extension()
                            .is_some_and(|x| x == "bin")
                    );
                    assert!(fw.offset.is_some() && fw.chip.is_some(), "{}", e.id);
                }
                "picotool" => {
                    assert!(
                        std::path::Path::new(e.url)
                            .extension()
                            .is_some_and(|x| x == "uf2")
                    );
                    assert!(fw.offset.is_none() && fw.chip.is_none(), "{}", e.id);
                }
                outro => panic!("motor desconhecido: {outro}"),
            }
            assert!(file_name(e).starts_with(fw.board), "{}", e.id);
            assert_eq!(entrada(e.id).map(|x| x.id), Some(e.id));
        }
        assert_eq!(
            entrada("micropython-esp32-generic")
                .and_then(|e| e.firmware)
                .and_then(|f| f.offset),
            Some("0x1000"),
            "a pagina do ESP32 classico manda 0x1000"
        );
    }
}
