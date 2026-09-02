//! As quatro acoes que editam o `Cargo.toml` da raiz.
//!
//! # Por que a edicao e textual, e por que ela RECUSA em vez de adivinhar
//!
//! O `Cargo.toml` e do usuario: comentarios, ordem das chaves e agrupamento
//! carregam intencao. Reescrever pelo modelo (parse -> serializa) perderia
//! tudo isso, e o projeto nao tem — nem quer, para uma unica feature — um
//! crate de TOML preservador de formato. Entao a edicao acontece na linha:
//! acha a secao, acha o fim dela, insere.
//!
//! O preco e que a IDE so entende as formas que ela sabe editar. Quando o
//! arquivo tem string multilinha (`"""`), ou declara `dependencies` como
//! tabela inline no topo, ou pede uma chave que ja existe, a acao **recusa
//! com motivo** — nunca escreve por aproximacao. Corromper um manifest e um
//! estrago sem desfazer, e um preview correto de uma escrita errada nao
//! ajudaria ninguem.

use std::collections::BTreeMap;

use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_required};
use super::{optional_param, required_param};

/// Arquivo editado por este modulo.
pub(super) const CARGO_TOML: &str = "Cargo.toml";

/// Editions aceitas pelo Cargo 1.96 / rustc 1.98 (a 2024 estabilizou na 1.85).
const EDITIONS: &[&str] = &["2015", "2018", "2021", "2024"];

/// Uma secao `[nome]` do arquivo, com o intervalo de linhas do corpo.
struct Section {
    name: String,
    body: std::ops::Range<usize>,
}

/// Quebra o arquivo em secoes, recusando o que nao da para ler com seguranca.
fn sections(text: &str) -> Result<Vec<Section>, ConfigActionError> {
    if text.contains("\"\"\"") || text.contains("'''") {
        return Err(ConfigActionError::UnsupportedShape {
            path: CARGO_TOML.to_owned(),
            reason: "ha string multilinha (\"\"\" ou ''') e a IDE nao consegue \
                     distinguir cabecalho de secao dentro dela"
                .to_owned(),
        });
    }

    let lines: Vec<&str> = text.lines().collect();
    let mut headers: Vec<(String, usize)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("[[")
            .and_then(|rest| rest.strip_suffix("]]"))
        {
            headers.push((name.trim().to_owned(), index));
        } else if let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            headers.push((name.trim().to_owned(), index));
        }
    }

    let mut result = Vec::with_capacity(headers.len());
    for (position, (name, index)) in headers.iter().enumerate() {
        let end = headers
            .get(position + 1)
            .map_or(lines.len(), |(_, next)| *next);
        result.push(Section {
            name: name.clone(),
            body: (index + 1)..end,
        });
    }
    Ok(result)
}

/// Linha de insercao no fim do corpo da secao, pulando linhas em branco.
fn insertion_point(lines: &[&str], body: &std::ops::Range<usize>) -> usize {
    let mut end = body.end;
    while end > body.start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

/// A chave ja existe nesta secao (como linha ou como subtabela)?
fn declares_key(lines: &[&str], sections: &[Section], section: &str, key: &str) -> bool {
    let subtable = format!("{section}.{key}");
    if sections.iter().any(|found| found.name == subtable) {
        return true;
    }
    sections
        .iter()
        .filter(|found| found.name == section)
        .any(|found| {
            lines[found.body.clone()].iter().any(|line| {
                key_of(line).is_some_and(|found_key| {
                    found_key == key || found_key == format!("{key}.workspace")
                })
            })
        })
}

/// Nome da chave declarada na linha, quando ela declara uma.
fn key_of(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let (raw, _) = trimmed.split_once('=')?;
    Some(raw.trim().trim_matches(['"', '\'']).to_owned())
}

/// Insere `entry` na secao `section`, criando a secao no fim se ela faltar.
fn insert_into_section(before: &str, section: &str, entry: &str) -> String {
    let lines: Vec<&str> = before.lines().collect();
    let found = sections(before)
        .ok()
        .and_then(|found| {
            found
                .into_iter()
                .rfind(|candidate| candidate.name == section)
        })
        .map(|candidate| insertion_point(&lines, &candidate.body));

    let Some(at) = found else {
        let mut after = before.trim_end().to_owned();
        if !after.is_empty() {
            after.push_str("\n\n");
        }
        after.push('[');
        after.push_str(section);
        after.push_str("]\n");
        after.push_str(entry);
        after.push('\n');
        return after;
    };

    let mut after = String::with_capacity(before.len() + entry.len() + 1);
    for line in &lines[..at] {
        after.push_str(line);
        after.push('\n');
    }
    after.push_str(entry);
    after.push('\n');
    for line in &lines[at..] {
        after.push_str(line);
        after.push('\n');
    }
    after
}

/// `[dependencies]` / `[dev-dependencies]`: acrescenta uma crate.
pub(super) fn add_dependency(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
    development: bool,
) -> Result<ActionPlan, ConfigActionError> {
    let section = if development {
        "dev-dependencies"
    } else {
        "dependencies"
    };
    let name = validate_name(required_param(params, "name")?, "name")?;
    let version = validate_version(required_param(params, "version")?)?;
    let features = optional_param(params, "features")
        .map(|raw| validate_list(raw, "features"))
        .transpose()?
        .unwrap_or_default();

    let before = read_required(root, CARGO_TOML)?;
    let found = sections(&before)?;
    reject_inline_table(&before, &found, section)?;
    let lines: Vec<&str> = before.lines().collect();
    if declares_key(&lines, &found, section, &name) {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{name} ja esta em [{section}] do {CARGO_TOML}"),
        });
    }

    let entry = if features.is_empty() {
        format!("{name} = \"{version}\"")
    } else {
        let list = features
            .iter()
            .map(|feature| format!("\"{feature}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{name} = {{ version = \"{version}\", features = [{list}] }}")
    };

    let after = insert_into_section(&before, section, &entry);
    Ok(ActionPlan::edit(
        format!("Acrescenta {name} {version} em [{section}]"),
        PlannedFile {
            path: CARGO_TOML.to_owned(),
            before: Some(before),
            after,
        },
    )
    .with_note(
        "A versao nao e resolvida na rede: a IDE escreve exatamente o que voce \
         informou. Rode 'cargo check' para o Cargo baixar e travar a dependencia."
            .to_owned(),
    ))
}

/// `[features]`: acrescenta uma feature.
pub(super) fn add_feature(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let name = validate_name(required_param(params, "name")?, "name")?;
    let enables = optional_param(params, "enables")
        .map(|raw| validate_list(raw, "enables"))
        .transpose()?
        .unwrap_or_default();

    let before = read_required(root, CARGO_TOML)?;
    let found = sections(&before)?;
    reject_inline_table(&before, &found, "features")?;
    let lines: Vec<&str> = before.lines().collect();
    if declares_key(&lines, &found, "features", &name) {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("a feature {name} ja existe em [features] do {CARGO_TOML}"),
        });
    }

    let list = enables
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let entry = format!("{name} = [{list}]");
    let after = insert_into_section(&before, "features", &entry);
    Ok(ActionPlan::edit(
        format!("Acrescenta a feature {name} em [features]"),
        PlannedFile {
            path: CARGO_TOML.to_owned(),
            before: Some(before),
            after,
        },
    ))
}

/// `[package] edition`: troca (ou declara) a edition do pacote.
pub(super) fn set_edition(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let edition = required_param(params, "edition")?.trim().to_owned();
    if !EDITIONS.contains(&edition.as_str()) {
        return Err(ConfigActionError::InvalidParam {
            name: "edition",
            reason: format!("use uma de {}", EDITIONS.join(", ")),
        });
    }

    let before = read_required(root, CARGO_TOML)?;
    let found = sections(&before)?;
    if !found.iter().any(|section| section.name == "package") {
        return Err(ConfigActionError::NotApplicable {
            reason: format!(
                "{CARGO_TOML} nao tem [package] (workspace virtual nao declara edition)"
            ),
        });
    }
    let lines: Vec<&str> = before.lines().collect();
    let package = found
        .iter()
        .find(|section| section.name == "package")
        .map(|section| section.body.clone())
        .unwrap_or_default();

    let current = lines[package.clone()]
        .iter()
        .enumerate()
        .find_map(|(offset, line)| {
            key_of(line)
                .filter(|key| key == "edition" || key == "edition.workspace")
                .map(|key| (package.start + offset, key))
        });

    let entry = format!("edition = \"{edition}\"");
    let after = match current {
        Some((index, key)) if key == "edition.workspace" => {
            return Err(ConfigActionError::NotApplicable {
                reason: format!(
                    "a edition e herdada do workspace (linha {}: edition.workspace); \
                     mude-a no Cargo.toml do workspace",
                    index + 1
                ),
            });
        }
        Some((index, _)) => {
            if lines[index].trim() == entry {
                return Err(ConfigActionError::NotApplicable {
                    reason: format!("o pacote ja usa a edition {edition}"),
                });
            }
            let mut after = String::with_capacity(before.len());
            for (position, line) in lines.iter().enumerate() {
                after.push_str(if position == index { &entry } else { line });
                after.push('\n');
            }
            after
        }
        None => insert_into_section(&before, "package", &entry),
    };

    Ok(ActionPlan::edit(
        format!("Define edition = \"{edition}\" em [package]"),
        PlannedFile {
            path: CARGO_TOML.to_owned(),
            before: Some(before),
            after,
        },
    )
    .with_note(
        "Trocar de edition pode exigir ajustes no codigo; rode 'cargo check' depois.".to_owned(),
    ))
}

/// Recusa `secao = { ... }` no topo do arquivo (tabela inline).
fn reject_inline_table(
    before: &str,
    found: &[Section],
    section: &str,
) -> Result<(), ConfigActionError> {
    let head_end = found.first().map_or(usize::MAX, |first| first.body.start);
    let inline = before
        .lines()
        .take(head_end)
        .any(|line| key_of(line).is_some_and(|key| key == section));
    if inline {
        return Err(ConfigActionError::UnsupportedShape {
            path: CARGO_TOML.to_owned(),
            reason: format!("{section} esta declarado como tabela inline no topo do arquivo"),
        });
    }
    Ok(())
}

/// Nome de crate/feature: o que o Cargo aceita em identificador de pacote.
fn validate_name(raw: &str, param: &'static str) -> Result<String, ConfigActionError> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(ConfigActionError::MissingParam { name: param });
    }
    let valid = name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && name.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_');
    if !valid {
        return Err(ConfigActionError::InvalidParam {
            name: param,
            reason: "use letras, digitos, '_' ou '-', comecando por letra".to_owned(),
        });
    }
    Ok(name.to_owned())
}

/// Requisito de versao: aceita o vocabulario semver do Cargo, sem aspas.
fn validate_version(raw: &str) -> Result<String, ConfigActionError> {
    let version = raw.trim();
    if version.is_empty() {
        return Err(ConfigActionError::MissingParam { name: "version" });
    }
    let valid = version.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '.' | '^' | '~' | '=' | '<' | '>' | '*' | '-' | '+' | ',' | ' '
            )
    });
    if !valid {
        return Err(ConfigActionError::InvalidParam {
            name: "version",
            reason: "caractere fora do vocabulario de versao do Cargo".to_owned(),
        });
    }
    Ok(version.to_owned())
}

/// Lista separada por virgula/espaco (features, dependencias de feature).
fn validate_list(raw: &str, param: &'static str) -> Result<Vec<String>, ConfigActionError> {
    raw.split([',', ' ', '\t'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| validate_feature_value(value, param))
        .collect()
}

/// Valor de feature: nome simples ou `dep/feature`, como o Cargo escreve.
fn validate_feature_value(raw: &str, param: &'static str) -> Result<String, ConfigActionError> {
    let valid = !raw.is_empty()
        && raw
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/' | '?' | ':'));
    if valid {
        Ok(raw.to_owned())
    } else {
        Err(ConfigActionError::InvalidParam {
            name: param,
            reason: format!("{raw} nao e um valor de feature valido"),
        })
    }
}
