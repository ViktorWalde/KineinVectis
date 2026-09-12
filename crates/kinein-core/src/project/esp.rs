//! O que o ESP-IDF escreve e o modelo LE: a receita de gravacao
//! (`flasher_args.json`) e a tabela de particoes (`partitions.csv`).
//!
//! Pilar 0 do `roadmaps/42`, segunda fatia (2026-09-12). A primeira so'
//! LOCALIZAVA os dois arquivos; agora eles viram dados — e' o que o ciclo MCU
//! (gravar) e o tamanho de flash do ESP32 consomem: a "flash" de um ESP32 nao
//! e' o `.ld`, e' a particao `app`.
//!
//! # Formatos, lidos na fonte em 2026-09-12
//!
//! `flasher_args.json` vem do template `components/esptool_py/flasher_args.json.in`
//! do ESP-IDF, preenchido pelo `project_include.cmake`:
//!
//! ```text
//! { "write_flash_args": [...],
//!   "flash_settings": { "flash_mode", "flash_size", "flash_freq" },
//!   "flash_files": { "<offset>": "<arquivo relativo ao build>", ... },
//!   "<nome>": { "offset", "file", "encrypted" },      // bootloader, app,
//!                                                     // partition-table...
//!   "extra_esptool_args": { "after", "before", "stub", "chip" } }
//! ```
//!
//! A tabela em CSV segue o guia *Partition Tables*: `Name, Type, SubType,
//! Offset, Size, Flags`; `#` comenta; offset em branco segue a particao
//! anterior (ou a tabela + 0x1000 na primeira); `app` alinha a 0x10000; tudo
//! alinha a 0x1000; tamanhos aceitam decimal, `0x`, `K` e `M`.

use std::path::Path;

use kinein_protocol::{FlashFile, FlashRecipe, Partition, PartitionTable};
use serde_json::Value;

/// `CONFIG_PARTITION_TABLE_OFFSET` padrao.
pub const TABLE_OFFSET_PADRAO: u32 = 0x8000;
/// Um setor de flash: o alinhamento de toda particao.
const SETOR: u32 = 0x1000;
/// O alinhamento das particoes `app`.
const ALINHAMENTO_APP: u32 = 0x10000;

/// Le a receita de gravacao. `build_dir` resolve os caminhos relativos.
#[must_use]
pub fn flash_recipe(json: &str, build_dir: &Path) -> Option<FlashRecipe> {
    let raiz: Value = serde_json::from_str(json).ok()?;
    let objeto = raiz.as_object()?;
    let texto = |v: Option<&Value>| v.and_then(Value::as_str).map(str::to_owned);
    let settings = objeto.get("flash_settings");
    let extra = objeto.get("extra_esptool_args");

    // As entradas NOMEADAS dizem qual arquivo e' o app, qual e' o bootloader
    // e qual esta' cifrado; `flash_files` e' a lista plana offset -> arquivo.
    let mut files: Vec<FlashFile> = Vec::new();
    for (nome, valor) in objeto {
        let Some(entrada) = valor.as_object() else {
            continue;
        };
        let (Some(offset), Some(file)) = (
            entrada
                .get("offset")
                .and_then(Value::as_str)
                .and_then(numero),
            entrada.get("file").and_then(Value::as_str),
        ) else {
            continue;
        };
        files.push(FlashFile {
            offset,
            file: build_dir.join(file).display().to_string(),
            name: Some(nome.clone()),
            encrypted: entrada
                .get("encrypted")
                .and_then(Value::as_str)
                .is_some_and(|e| e == "true"),
        });
    }
    if let Some(planos) = objeto.get("flash_files").and_then(Value::as_object) {
        for (offset, file) in planos {
            let (Some(offset), Some(file)) = (numero(offset), file.as_str()) else {
                continue;
            };
            if !files.iter().any(|f| f.offset == offset) {
                files.push(FlashFile {
                    offset,
                    file: build_dir.join(file).display().to_string(),
                    name: None,
                    encrypted: false,
                });
            }
        }
    }
    files.sort_by_key(|f| f.offset);

    let flash_size = texto(settings.and_then(|s| s.get("flash_size")));
    Some(FlashRecipe {
        chip: texto(extra.and_then(|e| e.get("chip"))),
        flash_mode: texto(settings.and_then(|s| s.get("flash_mode"))),
        flash_size_bytes: flash_size.as_deref().and_then(tamanho),
        flash_size,
        flash_freq: texto(settings.and_then(|s| s.get("flash_freq"))),
        before: texto(extra.and_then(|e| e.get("before"))),
        after: texto(extra.and_then(|e| e.get("after"))),
        stub: extra
            .and_then(|e| e.get("stub"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        files,
    })
}

/// Le a tabela de particoes em CSV.
///
/// Offsets em branco sao resolvidos como o `gen_esp32part.py` resolve: a
/// primeira particao depois da tabela (+0x1000), as seguintes depois da
/// anterior, `app` alinhada a 0x10000, tudo a 0x1000.
#[must_use]
pub fn partition_table(csv: &str, table_offset: u32) -> PartitionTable {
    let mut entries = Vec::new();
    let mut unreadable = Vec::new();
    let mut proximo = table_offset + SETOR;
    for linha in csv.lines() {
        let limpa = linha.split('#').next().unwrap_or("").trim();
        if limpa.is_empty() {
            continue;
        }
        let campos: Vec<&str> = limpa.split(',').map(str::trim).collect();
        if campos.len() < 5 {
            unreadable.push(linha.to_owned());
            continue;
        }
        let kind = campos[1].to_owned();
        let offset = if campos[3].is_empty() {
            let alinhamento = if kind == "app" {
                ALINHAMENTO_APP
            } else {
                SETOR
            };
            proximo.div_ceil(alinhamento) * alinhamento
        } else if let Some(o) = numero(campos[3]) {
            o
        } else {
            unreadable.push(linha.to_owned());
            continue;
        };
        let Some(size) = tamanho(campos[4]) else {
            unreadable.push(linha.to_owned());
            continue;
        };
        proximo = offset + size;
        entries.push(Partition {
            name: campos[0].to_owned(),
            kind,
            subtype: campos[2].to_owned(),
            offset,
            size,
            flags: campos
                .get(5)
                .filter(|f| !f.is_empty())
                .map(|f| (*f).to_owned()),
        });
    }
    let end = entries
        .iter()
        .map(|p| p.offset + p.size)
        .max()
        .unwrap_or(table_offset + SETOR);
    PartitionTable {
        table_offset,
        entries,
        end,
        unreadable,
    }
}

/// `0x10000`, `65536`.
fn numero(texto: &str) -> Option<u32> {
    let texto = texto.trim();
    texto
        .strip_prefix("0x")
        .or_else(|| texto.strip_prefix("0X"))
        .map_or_else(
            || texto.parse::<u32>().ok(),
            |hex| u32::from_str_radix(hex, 16).ok(),
        )
}

/// `1M`, `24K`, `0x6000`, `4MB`, `2mb`, `1024`.
fn tamanho(texto: &str) -> Option<u32> {
    let texto = texto.trim().to_ascii_uppercase();
    let sufixos: [(&[&str], u32); 2] = [(&["MB", "M"], 1024 * 1024), (&["KB", "K"], 1024)];
    let (base, mult) = sufixos
        .iter()
        .find_map(|(sufs, mult)| {
            sufs.iter()
                .find_map(|suf| texto.strip_suffix(suf))
                .map(|b| (b, *mult))
        })
        .unwrap_or((texto.as_str(), 1));
    numero(base).and_then(|n| n.checked_mul(mult))
}
