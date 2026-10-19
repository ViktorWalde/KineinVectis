//! O leitor da `compile_commands.json`: as unidades de compilacao, nas duas
//! formas do padrao do clang, e a CDB envelhecida por `CMakeLists.txt` de
//! subpasta (o `cdb::status` da raiz nao ve).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::{Value, value::RawValue};

/// A unidade de compilacao de um arquivo, ja' interpretada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CdbUnit {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) compiler: String,
    pub(super) directory: String,
    pub(super) standard: Option<String>,
    pub(super) includes: Vec<String>,
    pub(super) defines: Vec<String>,
    pub(super) output: Option<String>,
    pub(super) arguments: Vec<String>,
}

/// A `compile_commands.json`: `file` -> unidade, com `arguments` ou `command`
/// (as duas formas do padrao do clang), caminhos relativos resolvidos contra
/// `directory`, e `-I`/`-isystem`/`-iquote`/`-D`/`-std=`/`-o` separados do
/// resto sem perder o resto.
pub(super) fn parse_cdb(text: &str) -> HashMap<PathBuf, CdbUnit> {
    let mut units = HashMap::new();
    let Ok(entries) = serde_json::from_str::<Vec<&RawValue>>(text) else {
        return units;
    };
    let (mut cursor, mut line, mut column) = (0, 1, 1);
    for raw in entries {
        // RawValue borrows this input: the offset identifies the selected
        // object even with escaped paths, compact JSON and duplicate files.
        let offset = raw.get().as_ptr().addr() - text.as_ptr().addr();
        // Entries are ordered: scan each prefix once, including on a single
        // huge line. Recounting from the line start would be quadratic.
        for character in text[cursor..offset].chars() {
            if character == '\n' {
                line += 1;
                column = 1;
            } else {
                column += character.len_utf16();
            }
        }
        cursor = offset;
        let Ok(entry) = serde_json::from_str::<Value>(raw.get()) else {
            continue;
        };
        if let Some((key, unit)) = parse_entry(&entry, line, column) {
            units.insert(key, unit);
        }
    }
    units
}

/// Interpret one entry; invalid units have no source location to expose.
fn parse_entry(entry: &Value, line: usize, column: usize) -> Option<(PathBuf, CdbUnit)> {
    let file = entry.get("file").and_then(Value::as_str)?;
    let directory = entry.get("directory").and_then(Value::as_str)?;
    let dir = Path::new(directory);
    let arguments: Vec<String> = entry
        .get("arguments")
        .and_then(Value::as_array)
        .map_or_else(
            || {
                entry
                    .get("command")
                    .and_then(Value::as_str)
                    .map(crate::platform::split_command_line)
                    .unwrap_or_default()
            },
            |list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            },
        );
    if arguments.is_empty() {
        return None;
    }
    let mut unit = CdbUnit {
        line,
        column,
        compiler: arguments[0].clone(),
        directory: directory.to_owned(),
        standard: None,
        includes: Vec::new(),
        defines: Vec::new(),
        output: entry
            .get("output")
            .and_then(Value::as_str)
            .map(str::to_owned),
        arguments: arguments.clone(),
    };
    let msvc = msvc_driver(&arguments[0]);
    let mut i = 1;
    while i < arguments.len() {
        let arg = &arguments[i];
        let next_arg = arguments.get(i + 1);
        if msvc && let Some((flag, attached)) = msvc_flag(arg) {
            let value = if attached.is_empty() {
                i += 1;
                next_arg.cloned()
            } else {
                Some(attached.to_owned())
            };
            if let Some(value) = value {
                match flag {
                    MsvcFlag::Include => unit.includes.push(absolute(dir, &value)),
                    MsvcFlag::Define => unit.defines.push(value),
                    MsvcFlag::Standard => unit.standard = Some(value),
                    MsvcFlag::Output => {
                        unit.output.get_or_insert(value);
                    }
                }
            }
            i += 1;
            continue;
        }
        // `-Ifoo` colado, ou `-I foo` em dois argumentos (o segundo
        // consome o proximo).
        let mut take_value = |prefix: &str| -> Option<String> {
            if arg == prefix {
                i += 1;
                return next_arg.cloned();
            }
            arg.strip_prefix(prefix)
                .filter(|r| !r.is_empty())
                .map(str::to_owned)
        };
        if let Some(v) = take_value("-isystem")
            .or_else(|| take_value("-iquote"))
            .or_else(|| take_value("-I"))
        {
            unit.includes.push(absolute(dir, &v));
        } else if let Some(v) = take_value("-D") {
            unit.defines.push(v);
        } else if let Some(v) = arg.strip_prefix("-std=") {
            unit.standard = Some(v.to_owned());
        } else if let Some(v) = take_value("-o") {
            unit.output.get_or_insert(v);
        }
        i += 1;
    }
    // A chave e' o caminho REAL: a CDB do CMake escreve `file` absolute,
    // mas a do Ninja/Meson pode escrever `../src/a.cpp` relativo ao
    // `directory`, e a consulta chega pelo caminho do editor.
    let key = PathBuf::from(absolute(dir, file));
    let key = crate::platform::canonicalize(&key).unwrap_or(key);
    Some((key, unit))
}

/// O `cdb::status` so' compara a CDB com os arquivos de build da RAIZ. Medido
/// em 2026-09-12 neste repositorio: o `ui/CMakeLists.txt` ganhou fontes oito
/// dias depois da CDB e o diagnostico dizia "nao envelheceu" — cinco arquivos
/// sem unidade, em silencio. Aqui, com as unidades em maos, cada diretorio de
/// fonte sobe ate' a raiz procurando um `CMakeLists.txt` mais novo que a CDB.
pub(super) fn cmakelists_mais_novo(
    root: &Path,
    cdb: &Path,
    unidades: &HashMap<PathBuf, CdbUnit>,
) -> Option<String> {
    let cdb_em = modificado_em(cdb)?;
    let mut chaves: Vec<&PathBuf> = unidades.keys().collect();
    chaves.sort();
    let mut vistos: HashSet<&Path> = HashSet::new();
    for arquivo in chaves {
        let mut dir = arquivo.parent();
        while let Some(d) = dir {
            if !d.starts_with(root) || !vistos.insert(d) {
                break;
            }
            let lista = d.join("CMakeLists.txt");
            if modificado_em(&lista).is_some_and(|em| em > cdb_em) {
                return Some(lista.strip_prefix(root).map_or_else(
                    |_| lista.display().to_string(),
                    crate::platform::portable_relative,
                ));
            }
            dir = d.parent();
        }
    }
    None
}

fn modificado_em(caminho: &Path) -> Option<SystemTime> {
    std::fs::metadata(caminho).ok()?.modified().ok()
}

/// O compilador da unidade e' o do MSVC (`cl`) ou o `clang-cl`: as flags dele
/// comecam por `/` ou `-` (60 §3.3, W6b1). Com outro compilador, um `/D...`
/// e' caminho, e nao flag.
fn msvc_driver(compiler: &str) -> bool {
    let name = compiler
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(compiler)
        .to_ascii_lowercase();
    let stem = name.strip_suffix(".exe").unwrap_or(&name);
    stem == "cl" || stem == "clang-cl"
}

/// O que uma flag do MSVC diz da unidade.
#[derive(Debug, Clone, Copy)]
enum MsvcFlag {
    Include,
    Define,
    Standard,
    Output,
}

/// As flags do MSVC, com `/` ou `-`: a inclusao (`I`, `external:I`, `imsvc`),
/// a definicao (`D`), o padrao (`std:`) e a saida (`Fo`). Devolve o tipo e o
/// valor colado, vazio quando ele vem no argumento seguinte.
fn msvc_flag(arg: &str) -> Option<(MsvcFlag, &str)> {
    let rest = arg.strip_prefix('/').or_else(|| arg.strip_prefix('-'))?;
    [
        ("external:I", MsvcFlag::Include),
        ("imsvc", MsvcFlag::Include),
        ("I", MsvcFlag::Include),
        ("D", MsvcFlag::Define),
        ("std:", MsvcFlag::Standard),
        ("Fo", MsvcFlag::Output),
    ]
    .into_iter()
    .find_map(|(name, flag)| rest.strip_prefix(name).map(|value| (flag, value)))
}

fn absolute(dir: &Path, caminho: &str) -> String {
    let p = Path::new(caminho);
    if p.is_absolute() {
        caminho.to_owned()
    } else {
        dir.join(p).display().to_string()
    }
}
