//! Os configure presets do projeto: a lista, o padrao, os campos com
//! `inherits` e o `binaryDir` resolvido.
//!
//! O `CMake` nao informa o `binaryDir` de um preset sem configurar: medido com
//! o 4.4.5, `cmake --preset X -N` so' lista o cache. Por isso a IDE le
//! `CMakePresets.json` e `CMakeUserPresets.json` como o cmake-presets(7)
//! descreve. O que ela nao sabe resolver fica sem pasta, nunca com uma pasta
//! adivinhada (consolidacao C §16). `include` (schema 4+) nao e' seguido
//! ainda: C1-18, retomado no C-04.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::CmakePresetInfo;
use serde_json::Value;

/// Os arquivos de presets da raiz, na ordem do `cmake.presets.list`.
const FILES: [&str; 2] = ["CMakePresets.json", "CMakeUserPresets.json"];

/// Lista os configure presets nao ocultos de `CMakePresets.json` e
/// `CMakeUserPresets.json`, na ordem dos arquivos.
pub fn list_presets(root: &Path) -> Result<Vec<CmakePresetInfo>, String> {
    let mut presets = Vec::new();
    for file in ["CMakePresets.json", "CMakeUserPresets.json"] {
        let path = root.join(file);
        if !path.is_file() {
            continue;
        }
        let body =
            fs::read_to_string(&path).map_err(|error| format!("falha lendo {file}: {error}"))?;
        let value: Value =
            serde_json::from_str(&body).map_err(|error| format!("{file} inválido: {error}"))?;
        let Some(items) = value.get("configurePresets").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            if item.get("hidden").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                continue;
            };
            presets.push(CmakePresetInfo {
                name: name.to_owned(),
                display_name: item
                    .get("displayName")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
    }
    Ok(presets)
}

/// O que os presets do projeto dizem ao configure sem preset pedido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetDefault {
    /// O projeto nao tem presets que valham: configure sem preset.
    Absent,
    /// Um so': ele, com o arquivo de onde veio (a evidencia).
    One(String, &'static str),
    /// Varios no mesmo arquivo: ninguem escolhe pela pessoa (52 §14.2).
    Several(&'static str, Vec<String>),
}

/// O preset PADRAO do projeto, quando nem o pedido nem o kit dizem um.
///
/// Os `configurePresets` nao ocultos do primeiro arquivo que tem algum:
/// `CMakeUserPresets.json` (a escolha do usuario para ESTA maquina vence),
/// senao `CMakePresets.json`, pulando os que uma `condition` exclui no
/// Linux. Ate' 2026-10-16 (P0, 40.7 §7.46) valia o PRIMEIRO da lista; desde a
/// 0.4.0b, com varios, a UI pergunta (`roadmaps/52` §14.2).
#[must_use]
pub fn default_preset(root: &Path) -> PresetDefault {
    for file in ["CMakeUserPresets.json", "CMakePresets.json"] {
        let Ok(body) = fs::read_to_string(root.join(file)) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&body) else {
            continue;
        };
        let Some(items) = value.get("configurePresets").and_then(Value::as_array) else {
            continue;
        };
        let mut names: Vec<String> = items
            .iter()
            .filter(|item| {
                item.get("hidden").and_then(Value::as_bool) != Some(true)
                    && condition_allows_linux(item.get("condition"))
            })
            .filter_map(|item| item.get("name").and_then(Value::as_str).map(str::to_owned))
            .collect();
        match names.len() {
            0 => {}
            1 => return PresetDefault::One(names.remove(0), file),
            _ => return PresetDefault::Several(file, names),
        }
    }
    PresetDefault::Absent
}

/// A `condition` de um preset (cmake-presets(7)): so' as formas que dizem
/// respeito ao sistema — `const`, e `equals`/`notEquals` sobre
/// `${hostSystemName}` — sao avaliadas; qualquer outra e' considerada
/// verdadeira (o `CMake` decide de fato na hora do configure).
fn condition_allows_linux(condition: Option<&Value>) -> bool {
    let Some(condition) = condition else {
        return true;
    };
    match condition.get("type").and_then(Value::as_str) {
        Some("const") => condition.get("value").and_then(Value::as_bool) != Some(false),
        Some(kind @ ("equals" | "notEquals")) => {
            let lhs = condition.get("lhs").and_then(Value::as_str).unwrap_or("");
            let rhs = condition.get("rhs").and_then(Value::as_str).unwrap_or("");
            let host = |s: &str| s == "${hostSystemName}";
            let other = if host(lhs) {
                rhs
            } else if host(rhs) {
                lhs
            } else {
                return true;
            };
            let equal = other == "Linux";
            if kind == "equals" { equal } else { !equal }
        }
        _ => true,
    }
}

/// Profundidade de `inherits` e de macros dentro de `environment`: o `CMake`
/// recusa ciclos, e um ciclo aqui so' pode deixar o preset sem pasta.
const MAX_DEPTH: usize = 16;

/// O valor de um campo depois da heranca.
#[derive(Debug, PartialEq, Eq)]
enum Field {
    Text(String),
    /// `null`: o preset desfaz o que herdaria.
    Unset,
    Absent,
}

/// Todos os configure presets dos dois arquivos; arquivo ausente ou invalido
/// nao contribui.
fn entries(root: &Path) -> Vec<Value> {
    FILES
        .iter()
        .filter_map(|file| std::fs::read_to_string(root.join(file)).ok())
        .filter_map(|body| serde_json::from_str::<Value>(&body).ok())
        .filter_map(|value| {
            value
                .get("configurePresets")
                .and_then(Value::as_array)
                .cloned()
        })
        .flatten()
        .collect()
}

/// Um campo de configure preset, seguindo `inherits` como o `CMake`: o
/// proprio preset primeiro, depois os pais na ordem declarada. Valor `null`
/// desfaz a heranca. Valor objeto de `cacheVariables` usa seu `value`.
pub(crate) fn preset_value(root: &Path, preset: &str, field: &[&str]) -> Option<String> {
    if preset.is_empty() {
        return None;
    }
    match lookup(&entries(root), preset, field, 0) {
        Field::Text(text) => Some(text),
        Field::Unset | Field::Absent => None,
    }
}

fn lookup(entries: &[Value], name: &str, field: &[&str], depth: usize) -> Field {
    if depth > MAX_DEPTH {
        return Field::Absent;
    }
    let Some(entry) = entries
        .iter()
        .find(|entry| entry.get("name").and_then(Value::as_str) == Some(name))
    else {
        return Field::Absent;
    };
    match field.iter().try_fold(entry, |value, key| value.get(*key)) {
        Some(Value::Null) => return Field::Unset,
        Some(Value::String(text)) => return Field::Text(text.clone()),
        Some(object) => {
            if let Some(text) = object.get("value").and_then(Value::as_str) {
                return Field::Text(text.to_owned());
            }
        }
        None => {}
    }
    match entry.get("inherits") {
        Some(Value::String(parent)) => lookup(entries, parent, field, depth + 1),
        Some(Value::Array(parents)) => parents
            .iter()
            .filter_map(Value::as_str)
            .map(|parent| lookup(entries, parent, field, depth + 1))
            .find(|found| *found != Field::Absent)
            .unwrap_or(Field::Absent),
        _ => Field::Absent,
    }
}

/// Um configure preset visivel neste host e a pasta que ele declara.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PresetDirectory {
    pub name: String,
    pub directory: PathBuf,
}

/// O `binaryDir` resolvido dos configure presets nao ocultos que a
/// `condition` nao exclui, na ordem dos arquivos. Um preset sem pasta
/// resolvida nao aparece.
pub(crate) fn binary_dirs(root: &Path) -> Vec<PresetDirectory> {
    let entries = entries(root);
    entries
        .iter()
        .filter(|item| {
            item.get("hidden").and_then(Value::as_bool) != Some(true)
                && condition_allows_linux(item.get("condition"))
        })
        .filter_map(|item| item.get("name").and_then(Value::as_str))
        .filter_map(|name| {
            resolve_binary_dir(root, &entries, name)
                .ok()
                .map(|directory| PresetDirectory {
                    name: name.to_owned(),
                    directory,
                })
        })
        .collect()
}

/// A pasta de build que o preset declara, ou por que ela nao e' conhecida.
pub(crate) fn binary_dir(root: &Path, preset: &str) -> Result<PathBuf, String> {
    resolve_binary_dir(root, &entries(root), preset)
}

fn resolve_binary_dir(root: &Path, entries: &[Value], preset: &str) -> Result<PathBuf, String> {
    let declared = match lookup(entries, preset, &["binaryDir"], 0) {
        Field::Text(text) => text,
        Field::Unset | Field::Absent => return Err("o preset não declara binaryDir".to_owned()),
    };
    let expanded = Macros {
        root,
        entries,
        preset,
    }
    .expand(&declared, 0)?;
    let path = PathBuf::from(expanded);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

/// A expansao das macros do cmake-presets(7) no contexto de um preset.
struct Macros<'a> {
    root: &'a Path,
    entries: &'a [Value],
    preset: &'a str,
}

impl Macros<'_> {
    fn expand(&self, text: &str, depth: usize) -> Result<String, String> {
        if depth > MAX_DEPTH {
            return Err("ciclo de macros em environment".to_owned());
        }
        let mut result = String::new();
        let mut rest = text;
        while let Some(start) = rest.find('$') {
            result.push_str(&rest[..start]);
            rest = &rest[start..];
            let Some((namespace, after)) = ["${", "$env{", "$penv{", "$vendor{"]
                .iter()
                .find_map(|prefix| rest.strip_prefix(prefix).map(|after| (*prefix, after)))
            else {
                result.push('$');
                rest = &rest[1..];
                continue;
            };
            let end = after
                .find('}')
                .ok_or_else(|| format!("macro sem fechamento: {rest}"))?;
            let name = &after[..end];
            rest = &after[end + 1..];
            result.push_str(&match namespace {
                "${" => self.macro_value(name)?,
                "$env{" => self.environment(name, depth)?,
                "$penv{" => parent_environment(name)?,
                _ => return Err(format!("$vendor{{{name}}}: o CMake ignora este preset")),
            });
        }
        result.push_str(rest);
        Ok(result)
    }

    fn macro_value(&self, name: &str) -> Result<String, String> {
        let text = |path: &Path| path.to_string_lossy().into_owned();
        Ok(match name {
            "sourceDir" | "fileDir" => text(self.root),
            "sourceParentDir" => text(self.root.parent().unwrap_or(self.root)),
            "sourceDirName" => self
                .root
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            "presetName" => self.preset.to_owned(),
            "generator" => match lookup(self.entries, self.preset, &["generator"], 0) {
                Field::Text(generator) => generator,
                Field::Unset | Field::Absent => {
                    return Err("a macro generator exige generator no preset".to_owned());
                }
            },
            "hostSystemName" => host_system_name().to_owned(),
            "dollar" => "$".to_owned(),
            "pathListSep" => if cfg!(windows) { ";" } else { ":" }.to_owned(),
            other => return Err(format!("macro desconhecida: ${{{other}}}")),
        })
    }

    /// `$env{X}`: o `environment` do preset (com heranca, e com as macros do
    /// proprio valor), senao o ambiente do core; ausente expande vazio.
    fn environment(&self, name: &str, depth: usize) -> Result<String, String> {
        if name.is_empty() {
            return Err("$env{} sem nome".to_owned());
        }
        match lookup(self.entries, self.preset, &["environment", name], 0) {
            Field::Text(value) => self.expand(&value, depth + 1),
            Field::Unset => Ok(String::new()),
            Field::Absent => parent_environment(name),
        }
    }
}

fn parent_environment(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("$penv{} sem nome".to_owned());
    }
    Ok(std::env::var(name).unwrap_or_default())
}

/// O `${hostSystemName}`: o nome que o `CMake` da' ao sistema do host.
const fn host_system_name() -> &'static str {
    if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else {
        "Linux"
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{PresetDirectory, binary_dir, binary_dirs, preset_value};

    fn root(name: &str, presets: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cmake-presets")
            .join(format!("{}-{name} ação", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CMakePresets.json"), presets).unwrap();
        dir
    }

    #[test]
    fn presets_follow_inherits_and_object_values() {
        let root = root(
            "inherits",
            r#"{"version":6,"configurePresets":[
              {"name":"base","toolchainFile":"base.cmake",
               "cacheVariables":{"CMAKE_C_COMPILER":{"type":"FILEPATH","value":"/opt/cc"}}},
              {"name":"child","inherits":["missing","base"]},
              {"name":"unset","inherits":"base","cacheVariables":{"CMAKE_C_COMPILER":null}}]}"#,
        );
        let compiler = ["cacheVariables", "CMAKE_C_COMPILER"];
        assert_eq!(
            preset_value(&root, "child", &["toolchainFile"]).as_deref(),
            Some("base.cmake")
        );
        assert_eq!(
            preset_value(&root, "child", &compiler).as_deref(),
            Some("/opt/cc")
        );
        assert_eq!(preset_value(&root, "unset", &compiler), None);
        assert_eq!(preset_value(&root, "", &compiler), None);
    }

    /// Um pai que desfaz (`null`) vence o pai seguinte. Medido com o `CMake`
    /// 4.4.5 em 2026-11-09: `inherits: ["a", "b"]`, `a` com `KV: null` e `b`
    /// com `KV: "from-b"` configuram `KV` vazio. Ate' a mudanca para este
    /// modulo, a leitura pulava para `b`.
    #[test]
    fn the_first_parent_that_unsets_a_field_wins_over_later_parents() {
        let root = root(
            "unset-parent",
            r#"{"version":6,"configurePresets":[
              {"name":"a","hidden":true,"cacheVariables":{"KV":null}},
              {"name":"b","hidden":true,"cacheVariables":{"KV":"from-b"}},
              {"name":"child","inherits":["a","b"]},
              {"name":"later","inherits":["missing","b"]}]}"#,
        );
        assert_eq!(
            preset_value(&root, "child", &["cacheVariables", "KV"]),
            None
        );
        assert_eq!(
            preset_value(&root, "later", &["cacheVariables", "KV"]).as_deref(),
            Some("from-b")
        );
    }

    #[test]
    fn binary_dir_expands_the_documented_macros() {
        let root = root(
            "macros",
            r#"{"version":6,"configurePresets":[
              {"name":"base","hidden":true,"generator":"Ninja",
               "binaryDir":"${sourceParentDir}/${sourceDirName}-${presetName}/${generator}/${hostSystemName}"},
              {"name":"dbg","inherits":"base"},
              {"name":"dollar","binaryDir":"b/${dollar}x${pathListSep}y"},
              {"name":"relative","binaryDir":"rel/${presetName}"}]}"#,
        );
        let name = root.file_name().unwrap().to_str().unwrap().to_owned();
        assert_eq!(
            binary_dir(&root, "dbg").unwrap(),
            root.parent()
                .unwrap()
                .join(format!("{name}-dbg"))
                .join("Ninja")
                .join("Linux")
        );
        let separator = if cfg!(windows) { ";" } else { ":" };
        assert_eq!(
            binary_dir(&root, "dollar").unwrap(),
            root.join(format!("b/$x{separator}y"))
        );
        assert_eq!(
            binary_dir(&root, "relative").unwrap(),
            root.join("rel/relative")
        );
        // O oculto nao e' configuracao: so' o filho aparece.
        let listed: Vec<String> = binary_dirs(&root).into_iter().map(|p| p.name).collect();
        assert_eq!(listed, ["dbg", "dollar", "relative"]);
    }

    #[test]
    fn environment_macros_use_the_preset_then_the_core_environment() {
        let root = root(
            "env",
            r#"{"version":6,"configurePresets":[
              {"name":"base","hidden":true,"environment":{"KV_OUT":"$penv{HOME}/from-base"}},
              {"name":"inherited","inherits":"base","binaryDir":"$env{KV_OUT}/x"},
              {"name":"override","inherits":"base","environment":{"KV_OUT":"${sourceDir}/mine"},
               "binaryDir":"$env{KV_OUT}"},
              {"name":"removed","inherits":"base","environment":{"KV_OUT":null},
               "binaryDir":"${sourceDir}/z$env{KV_OUT}"},
              {"name":"loop","environment":{"A":"$env{B}","B":"$env{A}"},"binaryDir":"$env{A}"},
              {"name":"unknownvar","binaryDir":"${sourceDir}/u$env{KV_PRESETS_NOT_SET_ANYWHERE}"}]}"#,
        );
        // `$penv` le o ambiente do core, sem o `environment` do preset.
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(
            binary_dir(&root, "inherited").unwrap(),
            PathBuf::from(format!("{home}/from-base/x"))
        );
        assert_eq!(binary_dir(&root, "override").unwrap(), root.join("mine"));
        assert_eq!(binary_dir(&root, "removed").unwrap(), root.join("z"));
        assert_eq!(
            binary_dir(&root, "loop").unwrap_err(),
            "ciclo de macros em environment"
        );
        assert_eq!(binary_dir(&root, "unknownvar").unwrap(), root.join("u"));
    }

    /// O que o `CMake` recusa ou ignora nao vira pasta.
    #[test]
    fn presets_the_ide_cannot_resolve_have_no_directory() {
        let root = root(
            "unresolvable",
            r#"{"version":6,"configurePresets":[
              {"name":"unknown","binaryDir":"${sourceDir}/${nope}"},
              {"name":"vendor","binaryDir":"${sourceDir}/$vendor{acme}"},
              {"name":"open","binaryDir":"${sourceDir}/${presetName"},
              {"name":"nogenerator","binaryDir":"${sourceDir}/${generator}"},
              {"name":"nodir"},
              {"name":"windows","binaryDir":"w","condition":{"type":"equals","lhs":"${hostSystemName}","rhs":"Windows"}},
              {"name":"ok","binaryDir":"ok"}]}"#,
        );
        for (preset, reason) in [
            ("unknown", "macro desconhecida: ${nope}"),
            ("vendor", "$vendor{acme}: o CMake ignora este preset"),
            ("open", "macro sem fechamento: ${presetName"),
            ("nogenerator", "a macro generator exige generator no preset"),
            ("nodir", "o preset não declara binaryDir"),
            ("absent", "o preset não declara binaryDir"),
        ] {
            assert_eq!(binary_dir(&root, preset).unwrap_err(), reason, "{preset}");
        }
        assert_eq!(
            binary_dirs(&root),
            [PresetDirectory {
                name: "ok".to_owned(),
                directory: root.join("ok"),
            }]
        );
    }

    #[test]
    fn an_absolute_binary_dir_is_kept_and_user_presets_follow_project_presets() {
        let root = root(
            "absolute",
            r#"{"version":6,"configurePresets":[{"name":"abs","binaryDir":"/opt/kv build"}]}"#,
        );
        std::fs::write(
            root.join("CMakeUserPresets.json"),
            r#"{"version":6,"configurePresets":[{"name":"mine","inherits":"abs","binaryDir":"${sourceDir}/m"}]}"#,
        )
        .unwrap();
        assert_eq!(
            binary_dir(&root, "abs").unwrap(),
            Path::new("/opt/kv build")
        );
        let listed: Vec<String> = binary_dirs(&root).into_iter().map(|p| p.name).collect();
        assert_eq!(listed, ["abs", "mine"]);
        assert_eq!(binary_dir(&root, "mine").unwrap(), root.join("m"));
    }
}
