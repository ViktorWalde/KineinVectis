//! O tamanho de um ELF: quanto ele ocupa de nao-volatil e de RAM.
//!
//! Roda `<prefix>size -A` (formato `SysV`, que lista secao a secao) e, quando o
//! projeto tem um linker script com bloco `MEMORY`, le as CAPACIDADES das
//! regioes para dizer a FRACAO usada — que e' o numero que importa num
//! embarcado (131 KB num chip de 256 KB e' confortavel; num de 128 KB nao
//! cabe). A fonte do `size` e' o prefixo do cross (`arm-none-eabi-size`); sem
//! cross, o `size` do sistema.
//!
//! POR QUE `SysV` e nao Berkeley. O `size` sem `-A` devolve `text/data/bss` num
//! unico numero cada, e a conta "flash = text+data, ram = data+bss" vira
//! convencao opaca. O `-A` lista as secoes com nome e endereco, e dai o
//! mapeamento para regiao e' EXPLICITO e conferivel.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

/// Uma secao do ELF, como o `size -A` reporta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    /// Nome (`.text`, `.data`, `.bss`, ...).
    pub name: String,
    /// Tamanho em bytes.
    pub size: u64,
    /// Endereco de carga, quando a secao tem um.
    pub addr: u64,
}

/// Uma regiao de memoria do linker script, com o quanto dela foi usado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    /// Nome declarado no `MEMORY` (`FLASH`, `RAM`, `SRAM`, ...).
    pub name: String,
    /// Bytes usados: a soma das secoes cujo endereco cai nesta regiao.
    pub used: u64,
    /// Capacidade declarada (`LENGTH`).
    pub size: u64,
}

/// O resultado de medir um ELF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeReport {
    /// A ferramenta rodou?
    pub tool_available: bool,
    /// Qual binario foi chamado (`arm-none-eabi-size` ou `size`).
    pub tool: String,
    /// O ELF medido.
    pub program: String,
    /// Secoes com tamanho > 0.
    pub sections: Vec<Section>,
    /// Regioes do linker script com a fracao usada — vazio quando nao houve
    /// `MEMORY` para ler, e a UI entao mostra so' os totais.
    pub regions: Vec<Region>,
    /// Saida crua da ferramenta; sempre presente, como no `probe.list`.
    pub raw_output: String,
}

/// Mede `program` com o `size` de `prefix` (`Some("arm-none-eabi-")`) e,
/// quando dado, le as regioes de `linker_script`.
#[must_use]
pub fn measure(program: &Path, prefix: Option<&str>, linker_script: Option<&Path>) -> SizeReport {
    let tool = format!("{}size", prefix.unwrap_or(""));
    let saida = Command::new(&tool).arg("-A").arg(program).output();
    let relatorio = |tool_available, raw_output, sections| SizeReport {
        tool_available,
        tool: tool.clone(),
        program: program.display().to_string(),
        sections,
        regions: Vec::new(),
        raw_output,
    };
    let Ok(saida) = saida else {
        return relatorio(false, String::new(), Vec::new());
    };
    if !saida.status.success() {
        let erro = String::from_utf8_lossy(&saida.stderr).into_owned();
        return relatorio(true, erro, Vec::new());
    }
    let texto = String::from_utf8_lossy(&saida.stdout).into_owned();
    let sections = parse_sysv(&texto);
    let regioes = linker_script
        .and_then(|caminho| std::fs::read_to_string(caminho).ok())
        .map(|conteudo| regions_from(&parse_memory(&conteudo), &sections))
        .unwrap_or_default();
    SizeReport {
        regions: regioes,
        ..relatorio(true, texto, sections)
    }
}

/// Parseia a saida `size -A` (SysV): linhas `nome  tamanho  endereco`.
///
/// Ignora o cabecalho, a linha `Total` e secoes de tamanho zero — uma `.bss`
/// vazia nao e' informacao. So' entram secoes com endereco declarado para o
/// mapeamento de regiao; `.comment`/`.ARM.attributes` tem endereco 0 e nao
/// ocupam memoria do alvo, mas ficam na lista porque o usuario as ve no `size`.
#[must_use]
pub fn parse_sysv(texto: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    for linha in texto.lines() {
        let campos: Vec<&str> = linha.split_whitespace().collect();
        // `nome size addr` — exatamente tres, e os dois ultimos numericos.
        if campos.len() != 3 || campos[0] == "section" || campos[0] == "Total" {
            continue;
        }
        let (Ok(size), Ok(addr)) = (campos[1].parse::<u64>(), campos[2].parse::<u64>()) else {
            continue;
        };
        if size == 0 {
            continue;
        }
        sections.push(Section {
            name: campos[0].to_owned(),
            size,
            addr,
        });
    }
    sections
}

/// `(origin, length)` por regiao do bloco `MEMORY` de um linker script.
///
/// Best-effort e deliberadamente simples: le `NOME (attrs) : ORIGIN = X,
/// LENGTH = Y`, com `X`/`Y` em hex (`0x...`) ou decimal com sufixo `K`/`M`.
/// Um linker script que o core nao entende NAO vira palpite — a regiao
/// simplesmente nao aparece, e a UI mostra os totais sem a barra.
#[must_use]
pub fn parse_memory(texto: &str) -> BTreeMap<String, (u64, u64)> {
    let mut regioes = BTreeMap::new();
    let Some(inicio) = texto.find("MEMORY") else {
        return regioes;
    };
    let resto = &texto[inicio..];
    let Some(abre) = resto.find('{') else {
        return regioes;
    };
    let Some(fecha) = resto[abre..].find('}') else {
        return regioes;
    };
    for linha in resto[abre + 1..abre + fecha].lines() {
        let linha = linha.trim();
        let Some((nome_attrs, corpo)) = linha.split_once(':') else {
            continue;
        };
        // O nome e' o primeiro token antes do `(attrs)`.
        let Some(nome) = nome_attrs.split_whitespace().next() else {
            continue;
        };
        let origin = campo_numerico(corpo, "ORIGIN").or_else(|| campo_numerico(corpo, "org"));
        let length = campo_numerico(corpo, "LENGTH").or_else(|| campo_numerico(corpo, "len"));
        if let (Some(origin), Some(length)) = (origin, length) {
            regioes.insert(nome.to_owned(), (origin, length));
        }
    }
    regioes
}

/// Le `CHAVE = <numero>` de um corpo `ORIGIN = 0x0, LENGTH = 256K`.
fn campo_numerico(corpo: &str, chave: &str) -> Option<u64> {
    let pos = corpo.find(chave)?;
    let depois = corpo[pos + chave.len()..].trim_start();
    let depois = depois.strip_prefix('=')?.trim_start();
    let token: String = depois
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == 'x' || *c == 'X')
        .collect();
    parse_tamanho(&token)
}

/// `0x20000000`, `256K`, `64M`, `1024` — os que um linker script usa.
fn parse_tamanho(token: &str) -> Option<u64> {
    let token = token.trim();
    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).ok();
    }
    let (numero, fator) = match token.chars().last() {
        Some('K' | 'k') => (&token[..token.len() - 1], 1024),
        Some('M' | 'm') => (&token[..token.len() - 1], 1024 * 1024),
        _ => (token, 1),
    };
    numero.parse::<u64>().ok().map(|n| n * fator)
}

/// Secao que NAO ocupa memoria do alvo: metadados que o linker guarda no ELF
/// mas nao carrega no chip. O `size -A` as lista com endereco 0, e num alvo
/// cuja FLASH comeca em 0x0 (o `lm3s6965`, por ex.) elas cairiam na regiao e
/// inflariam o uso — 35 bytes da string de versao do compilador viram "flash
/// usada" que nao existe. A exclusao e' por nome, como os size tools fazem.
fn nao_alocada(nome: &str) -> bool {
    const PREFIXOS: [&str; 6] = [
        ".comment",
        ".ARM.attributes",
        ".debug",
        ".note",
        ".symtab",
        ".strtab",
    ];
    PREFIXOS.iter().any(|p| nome.starts_with(p))
}

/// Cruza secoes com regioes por endereco.
///
/// Cada secao ALOCADA com endereco dentro de `[origin, origin+length)` soma na
/// regiao. Regiao sem nenhuma secao entra com `used=0` — dizer "RAM: 0 de
/// 64 KB" e' informacao, nao ruido.
#[must_use]
pub fn regions_from(memoria: &BTreeMap<String, (u64, u64)>, sections: &[Section]) -> Vec<Region> {
    memoria
        .iter()
        .map(|(nome, (origin, length))| {
            let fim = origin.saturating_add(*length);
            let used = sections
                .iter()
                .filter(|s| !nao_alocada(&s.name) && s.addr >= *origin && s.addr < fim)
                .map(|s| s.size)
                .sum();
            Region {
                name: nome.clone(),
                used,
                size: *length,
            }
        })
        .collect()
}

/// A regiao de FLASH de um projeto ESP-IDF: a particao `app`.
///
/// E' a particao que a receita de gravacao aponta, com o tamanho da IMAGEM
/// (o `.bin` que vai para ela) como usado — o que o `idf.py size` chama de
/// "total image size" e o que a checagem de particao do build compara. O
/// `.ld` do ESP-IDF nao declara a flash; a particao e' a capacidade de verdade.
///
/// `app_offset` e' o offset do `app` na receita; a particao que casa e' a que
/// COMECA nele — nao "a primeira app", que numa tabela OTA seria outra.
#[must_use]
pub fn region_from_partition(
    partitions: &[kinein_protocol::Partition],
    app_offset: u32,
    app_image_bytes: u64,
) -> Option<Region> {
    let particao = partitions
        .iter()
        .find(|p| p.kind == "app" && p.offset == app_offset)?;
    Some(Region {
        name: format!("{} (particao app @0x{:x})", particao.name, particao.offset),
        used: app_image_bytes,
        size: u64::from(particao.size),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_memory, parse_sysv, parse_tamanho, region_from_partition, regions_from};

    const SAIDA: &str = "\
/tmp/fw.elf  :
section           size        addr
.isr_vector         16           0
.text              116          16
.bss                 4   536870912
.comment            35           0
.ARM.attributes     45           0
Total              216";

    #[test]
    fn a_regiao_da_particao_app_casa_pelo_offset_da_receita() {
        let particoes = vec![
            kinein_protocol::Partition {
                name: "factory".into(),
                kind: "app".into(),
                subtype: "factory".into(),
                offset: 0x10000,
                size: 0x10_0000,
                flags: None,
            },
            kinein_protocol::Partition {
                name: "ota_0".into(),
                kind: "app".into(),
                subtype: "ota_0".into(),
                offset: 0x11_0000,
                size: 0x10_0000,
                flags: None,
            },
            kinein_protocol::Partition {
                name: "nvs".into(),
                kind: "data".into(),
                subtype: "nvs".into(),
                offset: 0x9000,
                size: 0x6000,
                flags: None,
            },
        ];
        // A receita aponta a app em 0x110000 (OTA): e' a ota_0, nao a factory.
        let r = region_from_partition(&particoes, 0x11_0000, 200_000).unwrap();
        assert_eq!(r.name, "ota_0 (particao app @0x110000)");
        assert_eq!((r.used, r.size), (200_000, 0x10_0000));
        // Offset que nao e' de particao app: nenhuma regiao, nenhum palpite.
        assert!(region_from_partition(&particoes, 0x9000, 10).is_none());
        assert!(region_from_partition(&particoes, 0x20000, 10).is_none());
    }

    #[test]
    fn sysv_ignora_cabecalho_total_e_secao_vazia() {
        let secoes = parse_sysv(SAIDA);
        let nomes: Vec<&str> = secoes.iter().map(|s| s.name.as_str()).collect();
        // `.isr_vector`, `.text`, `.bss`, `.comment`, `.ARM.attributes` — NAO
        // `section` (cabecalho) nem `Total`.
        assert_eq!(
            nomes,
            [
                ".isr_vector",
                ".text",
                ".bss",
                ".comment",
                ".ARM.attributes"
            ]
        );
        assert_eq!(secoes[1].size, 116);
        assert_eq!(secoes[2].addr, 536_870_912);
    }

    #[test]
    fn memory_le_hex_e_sufixo_k() {
        let ld = "MEMORY\n{\n  FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K\n  \
                  SRAM (rwx) : ORIGIN = 0x20000000, LENGTH = 64K\n}\n";
        let mem = parse_memory(ld);
        assert_eq!(mem.get("FLASH"), Some(&(0, 256 * 1024)));
        assert_eq!(mem.get("SRAM"), Some(&(0x2000_0000, 64 * 1024)));
    }

    #[test]
    fn tamanho_aceita_hex_decimal_k_e_m() {
        assert_eq!(parse_tamanho("0x100"), Some(256));
        assert_eq!(parse_tamanho("1024"), Some(1024));
        assert_eq!(parse_tamanho("256K"), Some(256 * 1024));
        assert_eq!(parse_tamanho("2M"), Some(2 * 1024 * 1024));
        assert_eq!(parse_tamanho("lixo"), None);
    }

    #[test]
    fn regioes_somam_as_secoes_pelo_endereco() {
        let mem = parse_memory(
            "MEMORY\n{\n  FLASH (rx) : ORIGIN = 0x0, LENGTH = 256K\n  \
             SRAM (rwx) : ORIGIN = 0x20000000, LENGTH = 64K\n}\n",
        );
        let regioes = regions_from(&mem, &parse_sysv(SAIDA));
        let flash = regioes.iter().find(|r| r.name == "FLASH").unwrap();
        let sram = regioes.iter().find(|r| r.name == "SRAM").unwrap();
        // FLASH: isr_vector(16) + text(116) = 132. O `.comment`(35) e o
        // `.ARM.attributes`(45) tambem estao em addr 0, mas NAO sao alocadas —
        // metadados do ELF que nao vao para o chip —, entao nao contam.
        assert_eq!(flash.used, 16 + 116);
        assert_eq!(flash.size, 256 * 1024);
        // SRAM: so' a .bss (addr 0x20000000), 4 bytes.
        assert_eq!(sram.used, 4);
        assert_eq!(sram.size, 64 * 1024);
    }

    #[test]
    fn sem_bloco_memory_nao_ha_regiao() {
        assert!(parse_memory("ENTRY(Reset_Handler)\nSECTIONS { }").is_empty());
    }
}
