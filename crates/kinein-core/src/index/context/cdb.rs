//! O leitor da `compile_commands.json`: as unidades de compilacao, nas duas
//! formas do padrao do clang, e a CDB envelhecida por `CMakeLists.txt` de
//! subpasta (o `cdb::status` da raiz nao ve).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::Value;

/// A unidade de compilacao de um arquivo, ja' interpretada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Unidade {
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
pub(super) fn parse_cdb(texto: &str) -> HashMap<PathBuf, Unidade> {
    let mut unidades = HashMap::new();
    let Ok(Value::Array(entradas)) = serde_json::from_str::<Value>(texto) else {
        return unidades;
    };
    for entrada in &entradas {
        let (Some(file), Some(directory)) = (
            entrada.get("file").and_then(Value::as_str),
            entrada.get("directory").and_then(Value::as_str),
        ) else {
            continue;
        };
        let dir = Path::new(directory);
        let arguments: Vec<String> = entrada
            .get("arguments")
            .and_then(Value::as_array)
            .map_or_else(
                || {
                    entrada
                        .get("command")
                        .and_then(Value::as_str)
                        .map(dividir_comando)
                        .unwrap_or_default()
                },
                |lista| {
                    lista
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                },
            );
        if arguments.is_empty() {
            continue;
        }
        let mut unidade = Unidade {
            compiler: arguments[0].clone(),
            directory: directory.to_owned(),
            standard: None,
            includes: Vec::new(),
            defines: Vec::new(),
            output: entrada
                .get("output")
                .and_then(Value::as_str)
                .map(str::to_owned),
            arguments: arguments.clone(),
        };
        let mut i = 1;
        while i < arguments.len() {
            let arg = &arguments[i];
            let proximo = arguments.get(i + 1);
            // `-Ifoo` colado, ou `-I foo` em dois argumentos (o segundo
            // consome o proximo).
            let mut pega = |prefixo: &str| -> Option<String> {
                if arg == prefixo {
                    i += 1;
                    return proximo.cloned();
                }
                arg.strip_prefix(prefixo)
                    .filter(|r| !r.is_empty())
                    .map(str::to_owned)
            };
            if let Some(v) = pega("-isystem")
                .or_else(|| pega("-iquote"))
                .or_else(|| pega("-I"))
            {
                unidade.includes.push(absoluto(dir, &v));
            } else if let Some(v) = pega("-D") {
                unidade.defines.push(v);
            } else if let Some(v) = arg.strip_prefix("-std=") {
                unidade.standard = Some(v.to_owned());
            } else if let Some(v) = pega("-o") {
                unidade.output.get_or_insert(v);
            }
            i += 1;
        }
        // A chave e' o caminho REAL: a CDB do CMake escreve `file` absoluto,
        // mas a do Ninja/Meson pode escrever `../src/a.cpp` relativo ao
        // `directory`, e a consulta chega pelo caminho do editor.
        let chave = PathBuf::from(absoluto(dir, file));
        let chave = std::fs::canonicalize(&chave).unwrap_or(chave);
        unidades.insert(chave, unidade);
    }
    unidades
}

/// O `cdb::status` so' compara a CDB com os arquivos de build da RAIZ. Medido
/// em 2026-09-12 neste repositorio: o `ui/CMakeLists.txt` ganhou fontes oito
/// dias depois da CDB e o diagnostico dizia "nao envelheceu" — cinco arquivos
/// sem unidade, em silencio. Aqui, com as unidades em maos, cada diretorio de
/// fonte sobe ate' a raiz procurando um `CMakeLists.txt` mais novo que a CDB.
pub(super) fn cmakelists_mais_novo(
    root: &Path,
    cdb: &Path,
    unidades: &HashMap<PathBuf, Unidade>,
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
                let relativo = lista.strip_prefix(root).unwrap_or(&lista);
                return Some(relativo.display().to_string());
            }
            dir = d.parent();
        }
    }
    None
}

fn modificado_em(caminho: &Path) -> Option<SystemTime> {
    std::fs::metadata(caminho).ok()?.modified().ok()
}

fn absoluto(dir: &Path, caminho: &str) -> String {
    let p = Path::new(caminho);
    if p.is_absolute() {
        caminho.to_owned()
    } else {
        dir.join(p).display().to_string()
    }
}

/// Divide uma `command` como um shell dividiria: espaco separa, aspas
/// simples/duplas agrupam, `\` escapa fora de aspas simples.
fn dividir_comando(comando: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut atual = String::new();
    let mut aspas: Option<char> = None;
    let mut escapado = false;
    let mut tem = false;
    for c in comando.chars() {
        if escapado {
            atual.push(c);
            escapado = false;
            continue;
        }
        match (c, aspas) {
            ('\\', Some('\'')) => atual.push(c),
            ('\\', _) => escapado = true,
            (q, None) if q == '\'' || q == '"' => {
                aspas = Some(q);
                tem = true;
            }
            (q, Some(a)) if q == a => aspas = None,
            (' ' | '\t', None) => {
                if tem || !atual.is_empty() {
                    args.push(std::mem::take(&mut atual));
                    tem = false;
                }
            }
            _ => {
                atual.push(c);
                tem = true;
            }
        }
    }
    if tem || !atual.is_empty() {
        args.push(atual);
    }
    args
}
