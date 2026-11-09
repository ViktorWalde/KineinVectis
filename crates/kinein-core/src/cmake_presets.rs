//! Os configure presets do projeto: a lista, o padrao, os campos com
//! `inherits` e o `binaryDir` resolvido.
//!
//! O `CMake` nao informa o `binaryDir` de um preset sem configurar: medido com
//! o 4.4.5, `cmake --preset X -N` so' lista o cache. Por isso a IDE le
//! `CMakePresets.json` e `CMakeUserPresets.json` como o cmake-presets(7)
//! descreve, com `include` (C1-18, `files.rs`). O que ela nao sabe resolver
//! fica sem pasta, nunca com uma pasta adivinhada (consolidacao C §16), e o
//! que o `CMake` recusa ela tambem recusa, com o motivo (§17).

mod files;
mod macros;

use std::path::{Path, PathBuf};

use kinein_protocol::CmakePresetInfo;
use serde_json::Value;

use files::{PROJECT, Presets, USER};

/// Lista os configure presets nao ocultos de `CMakePresets.json` e
/// `CMakeUserPresets.json`, com o que eles incluem, na ordem dos arquivos.
pub fn list_presets(root: &Path) -> Result<Vec<CmakePresetInfo>, String> {
    Ok(files::load(root)?
        .entries
        .iter()
        .filter(|entry| entry.value.get("hidden").and_then(Value::as_bool) != Some(true))
        .map(|entry| CmakePresetInfo {
            name: entry.name.clone(),
            display_name: entry
                .value
                .get("displayName")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
        .collect())
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
    /// O `CMake` recusaria todos os presets (C1-18): o motivo, nunca um
    /// configure sem preset que ignore os arquivos.
    Refused(String),
}

/// O preset PADRAO do projeto, quando nem o pedido nem o kit dizem um.
///
/// Os `configurePresets` nao ocultos do primeiro lado que tem algum: o de
/// `CMakeUserPresets.json` (a escolha do usuario para ESTA maquina vence),
/// com o que so' ele inclui, senao o de `CMakePresets.json`, pulando os que
/// uma `condition` exclui no Linux. Ate' 2026-10-16 (P0, 40.7 §7.46) valia o
/// PRIMEIRO da lista; desde a 0.4.0b, com varios, a UI pergunta
/// (`roadmaps/52` §14.2).
#[must_use]
pub fn default_preset(root: &Path) -> PresetDefault {
    let presets = match files::load(root) {
        Ok(presets) => presets,
        Err(reason) => return PresetDefault::Refused(reason),
    };
    for (file, user_side) in [(USER, true), (PROJECT, false)] {
        let mut names: Vec<String> = presets
            .entries
            .iter()
            .filter(|entry| {
                presets.file_of(entry).user_side == user_side && visible_on_linux(&entry.value)
            })
            .map(|entry| entry.name.clone())
            .collect();
        match names.len() {
            0 => {}
            1 => return PresetDefault::One(names.remove(0), file),
            _ => return PresetDefault::Several(file, names),
        }
    }
    PresetDefault::Absent
}

/// Nao oculto, e a `condition` nao o exclui no Linux.
fn visible_on_linux(item: &Value) -> bool {
    item.get("hidden").and_then(Value::as_bool) != Some(true)
        && condition_allows_linux(item.get("condition"))
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

/// Profundidade de `inherits`: o `CMake` recusa ciclos, e um ciclo aqui so'
/// pode deixar o campo ausente.
const MAX_DEPTH: usize = 16;

/// O valor de um campo depois da heranca.
#[derive(Debug, PartialEq, Eq)]
enum Field {
    Text(String),
    /// `null`: o preset desfaz o que herdaria.
    Unset,
    Absent,
}

/// Um campo de configure preset, seguindo `inherits` como o `CMake`: o
/// proprio preset primeiro, depois os pais na ordem declarada. Valor `null`
/// desfaz a heranca. Valor objeto de `cacheVariables` usa seu `value`.
/// Presets que o `CMake` recusaria nao tem campo.
pub(crate) fn preset_value(root: &Path, preset: &str, field: &[&str]) -> Option<String> {
    if preset.is_empty() {
        return None;
    }
    match lookup(&files::load(root).ok()?, preset, field, 0) {
        Field::Text(text) => Some(text),
        Field::Unset | Field::Absent => None,
    }
}

fn lookup(presets: &Presets, name: &str, field: &[&str], depth: usize) -> Field {
    if depth > MAX_DEPTH {
        return Field::Absent;
    }
    let Some(entry) = presets.find(name) else {
        return Field::Absent;
    };
    match field
        .iter()
        .try_fold(&entry.value, |value, key| value.get(*key))
    {
        Some(Value::Null) => return Field::Unset,
        Some(Value::String(text)) => return Field::Text(text.clone()),
        Some(object) => {
            if let Some(text) = object.get("value").and_then(Value::as_str) {
                return Field::Text(text.to_owned());
            }
        }
        None => {}
    }
    match entry.value.get("inherits") {
        Some(Value::String(parent)) => lookup(presets, parent, field, depth + 1),
        Some(Value::Array(parents)) => parents
            .iter()
            .filter_map(Value::as_str)
            .map(|parent| lookup(presets, parent, field, depth + 1))
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
/// resolvida nao aparece; presets recusados nao tem nenhuma.
pub(crate) fn binary_dirs(root: &Path) -> Vec<PresetDirectory> {
    let Ok(presets) = files::load(root) else {
        return Vec::new();
    };
    presets
        .entries
        .iter()
        .filter(|entry| visible_on_linux(&entry.value))
        .filter_map(|entry| {
            resolve_binary_dir(root, &presets, &entry.name)
                .ok()
                .map(|directory| PresetDirectory {
                    name: entry.name.clone(),
                    directory,
                })
        })
        .collect()
}

/// A pasta de build que o preset declara, ou por que ela nao e' conhecida.
pub(crate) fn binary_dir(root: &Path, preset: &str) -> Result<PathBuf, String> {
    resolve_binary_dir(root, &files::load(root)?, preset)
}

fn resolve_binary_dir(root: &Path, presets: &Presets, preset: &str) -> Result<PathBuf, String> {
    let declared = match lookup(presets, preset, &["binaryDir"], 0) {
        Field::Text(text) => text,
        Field::Unset | Field::Absent => return Err("o preset não declara binaryDir".to_owned()),
    };
    let file_dir = presets
        .find(preset)
        .map_or(root, |entry| presets.file_of(entry).dir.as_path());
    let expanded = macros::Macros {
        root,
        presets,
        preset,
        file_dir,
    }
    .expand(&declared, 0)?;
    let path = PathBuf::from(expanded);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

/// Um arquivo de presets nas mensagens: relativo a raiz quando dentro dela.
fn label(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).map_or_else(
        |_| path.display().to_string(),
        crate::platform::portable_relative,
    )
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
              {"name":"empty","hidden":true},
              {"name":"child","inherits":["empty","base"]},
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
              {"name":"empty","hidden":true},
              {"name":"later","inherits":["empty","b"]}]}"#,
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

    fn write(root: &Path, relative: &str, body: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn names(root: &Path) -> Vec<String> {
        super::list_presets(root)
            .unwrap()
            .into_iter()
            .map(|preset| preset.name)
            .collect()
    }

    /// C1-18, medido com o `CMake` 4.4.5 em 2026-11-09 (consolidacao C §17):
    /// `cmake --preset top` configurou em `out/top` e `--preset sub` em
    /// `sub/out/sub`, com o `binaryDir` herdado de `sub/more.json`.
    #[test]
    fn include_is_followed_nested_relative_to_the_including_file_and_read_once() {
        let root = root(
            "include",
            r#"{"version":6,"include":["sub/more.json","shared/x.json"],"configurePresets":[
              {"name":"top","inherits":"base","generator":"Ninja"}]}"#,
        );
        write(
            &root,
            "sub/more.json",
            r#"{"version":6,"include":["../shared/./x.json"],"configurePresets":[
              {"name":"base","hidden":true,"binaryDir":"${fileDir}/out/${presetName}"},
              {"name":"sub","inherits":"base","generator":"Ninja"}]}"#,
        );
        write(
            &root,
            "shared/x.json",
            r#"{"version":6,"configurePresets":[{"name":"shared","binaryDir":"${fileDir}/s"}]}"#,
        );
        assert_eq!(names(&root), ["top", "sub", "shared"]);
        assert_eq!(binary_dir(&root, "top").unwrap(), root.join("out/top"));
        assert_eq!(binary_dir(&root, "sub").unwrap(), root.join("sub/out/sub"));
        assert_eq!(binary_dir(&root, "shared").unwrap(), root.join("shared/s"));
        let found: Vec<PathBuf> = binary_dirs(&root)
            .into_iter()
            .map(|p| p.directory)
            .collect();
        assert_eq!(
            found,
            [
                root.join("out/top"),
                root.join("sub/out/sub"),
                root.join("shared/s")
            ]
        );
        assert_eq!(
            super::default_preset(&root),
            super::PresetDefault::Several(
                "CMakePresets.json",
                vec!["top".to_owned(), "sub".to_owned(), "shared".to_owned()]
            )
        );
        assert_eq!(
            preset_value(&root, "top", &["generator"]).as_deref(),
            Some("Ninja")
        );
    }

    /// Um caso que o `CMake` 4.4.5 recusa: o nome, o `CMakePresets.json`,
    /// os outros arquivos e o comeco do motivo.
    type Refusal = (
        &'static str,
        &'static str,
        &'static [(&'static str, &'static str)],
        &'static str,
    );

    const OK: &str = r#"{"version":6,"configurePresets":[{"name":"m","binaryDir":"o"}]}"#;

    const REFUSALS: [Refusal; 12] = [
        (
            "cycle",
            r#"{"version":6,"include":["sub/b.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[(
                "sub/b.json",
                r#"{"version":6,"include":["../CMakePresets.json"],"configurePresets":[]}"#,
            )],
            "ciclo de include entre arquivos de presets: CMakePresets.json",
        ),
        (
            "missing",
            r#"{"version":6,"include":["nada.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[],
            "arquivo de presets não encontrado: nada.json",
        ),
        (
            "old",
            r#"{"version":3,"include":["sub/m.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[("sub/m.json", OK)],
            "CMakePresets.json: include exige version 4 ou maior",
        ),
        (
            "duplicate",
            r#"{"version":6,"include":["sub/m.json"],"configurePresets":[{"name":"m","binaryDir":"p"}]}"#,
            &[("sub/m.json", OK)],
            "preset de configure repetido: \"m\"",
        ),
        (
            "unreachable",
            r#"{"version":6,"configurePresets":[{"name":"p","inherits":"u","binaryDir":"o"}]}"#,
            &[(
                "CMakeUserPresets.json",
                r#"{"version":6,"configurePresets":[{"name":"u","hidden":true}]}"#,
            )],
            "o preset de configure \"p\" herda de um preset que o arquivo dele não inclui",
        ),
        (
            "siblings",
            r#"{"version":6,"include":["sub/a.json","sub/b.json"],"configurePresets":[]}"#,
            &[
                (
                    "sub/a.json",
                    r#"{"version":6,"configurePresets":[{"name":"a","inherits":"b0","binaryDir":"o"}]}"#,
                ),
                (
                    "sub/b.json",
                    r#"{"version":6,"configurePresets":[{"name":"b0","hidden":true}]}"#,
                ),
            ],
            "o preset de configure \"a\" herda de um preset que o arquivo dele não inclui",
        ),
        (
            "noversion",
            r#"{"version":6,"include":["sub/x.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[("sub/x.json", r#"{"configurePresets":[]}"#)],
            "sub/x.json sem o campo version",
        ),
        (
            "invalid",
            r#"{"version":6,"include":["sub/x.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[("sub/x.json", r#"{"version":6,"configurePresets":["#)],
            "sub/x.json inválido: ",
        ),
        (
            "loop",
            r#"{"version":6,"include":["CMakeUserPresets.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[(
                "CMakeUserPresets.json",
                r#"{"version":6,"configurePresets":[{"name":"u","binaryDir":"o"}]}"#,
            )],
            "ciclo de include entre arquivos de presets: CMakeUserPresets.json",
        ),
        (
            "noparent",
            r#"{"version":6,"configurePresets":[{"name":"p","inherits":"missing","binaryDir":"o"}]}"#,
            &[],
            "o preset de configure \"p\" herda de \"missing\", que não existe",
        ),
        (
            "inheritcycle",
            r#"{"version":6,"configurePresets":[{"name":"p","inherits":"b","binaryDir":"o"},
                  {"name":"b","inherits":"p","hidden":true}]}"#,
            &[],
            "herança cíclica no preset de configure \"p\"",
        ),
        (
            "macro",
            r#"{"version":9,"include":["${presetName}/m.json"],"configurePresets":[{"name":"p","binaryDir":"o"}]}"#,
            &[],
            "CMakePresets.json: include inválido: macro não aceita em include: ${presetName}",
        ),
    ];

    /// O que o `CMake` 4.4.5 recusa ("Could not read presets") recusa todos
    /// os presets, com o motivo: nunca parte da lista nem um padrao.
    #[test]
    fn what_cmake_refuses_refuses_every_preset_with_the_reason() {
        for (name, presets, others, reason) in REFUSALS {
            let root = root(name, presets);
            for (relative, body) in others {
                write(&root, relative, body);
            }
            let refused = super::list_presets(&root).unwrap_err();
            assert!(refused.starts_with(reason), "{name}: {refused}");
            assert_eq!(
                super::default_preset(&root),
                super::PresetDefault::Refused(refused.clone()),
                "{name}"
            );
            assert_eq!(binary_dir(&root, "p").unwrap_err(), refused, "{name}");
            assert!(binary_dirs(&root).is_empty(), "{name}");
            assert_eq!(preset_value(&root, "p", &["binaryDir"]), None, "{name}");
        }
    }

    #[test]
    fn include_macros_follow_the_version_of_the_including_file() {
        use super::macros::include_path;
        let root = Path::new("/r/proj");
        let dir = Path::new("/r/proj/cmake");
        let path = std::env::var("PATH").unwrap_or_default();
        for text in ["${sourceDir}/a.json", "$penv{PATH}", "$env{X}"] {
            assert_eq!(
                include_path(text, 6, root, dir).unwrap(),
                text,
                "literal na 6"
            );
        }
        assert_eq!(
            include_path("$penv{PATH}/a", 7, root, dir).unwrap(),
            format!("{path}/a")
        );
        assert_eq!(
            include_path("${sourceDir}/a", 8, root, dir).unwrap_err(),
            "macro não aceita em include na versão 8"
        );
        assert_eq!(
            include_path(
                "${sourceDir}/${fileDir}/${sourceDirName}${dollar}",
                9,
                root,
                dir
            )
            .unwrap(),
            "/r/proj//r/proj/cmake/proj$"
        );
        assert_eq!(
            include_path("$env{X}", 9, root, dir).unwrap_err(),
            "macro não aceita em include na versão 9"
        );
        assert_eq!(
            include_path("${generator}", 10, root, dir).unwrap_err(),
            "macro não aceita em include: ${generator}"
        );
        assert_eq!(
            include_path("${sourceDir", 9, root, dir).unwrap_err(),
            "macro sem fechamento: ${sourceDir"
        );

        let root = super::tests::root(
            "include-macros",
            r#"{"version":9,"include":["${sourceDir}/cmake/a.json"],"configurePresets":[]}"#,
        );
        write(
            &root,
            "cmake/a.json",
            r#"{"version":9,"include":["${fileDir}/b.json"],"configurePresets":[{"name":"a","binaryDir":"o"}]}"#,
        );
        write(
            &root,
            "cmake/b.json",
            r#"{"version":9,"configurePresets":[{"name":"b","binaryDir":"o"}]}"#,
        );
        assert_eq!(names(&root), ["a", "b"]);
    }

    /// O padrao (52 §14.2) e' do lado do usuario quando so' o
    /// `CMakeUserPresets.json` alcanca o preset; um arquivo que os dois
    /// alcancam e' do projeto.
    #[test]
    fn presets_reached_only_from_the_user_file_are_the_user_side() {
        let root = root(
            "user-side",
            r#"{"version":6,"include":["common.json"],"configurePresets":[
              {"name":"p1","binaryDir":"o"},{"name":"p2","binaryDir":"o"}]}"#,
        );
        write(
            &root,
            "common.json",
            r#"{"version":6,"configurePresets":[{"name":"c","binaryDir":"o"}]}"#,
        );
        write(
            &root,
            "CMakeUserPresets.json",
            r#"{"version":6,"include":["local/mine.json","common.json"],"configurePresets":[]}"#,
        );
        write(
            &root,
            "local/mine.json",
            r#"{"version":6,"configurePresets":[{"name":"mine","binaryDir":"${fileDir}/m"}]}"#,
        );
        assert_eq!(names(&root), ["p1", "p2", "c", "mine"]);
        assert_eq!(
            super::default_preset(&root),
            super::PresetDefault::One("mine".to_owned(), "CMakeUserPresets.json")
        );
        assert_eq!(binary_dir(&root, "mine").unwrap(), root.join("local/m"));
        // O `include` implicito e' do arquivo do usuario, nao do que ele
        // inclui: o `CMake` 4.4.5 recusa ("unreachable from preset's file").
        write(
            &root,
            "local/mine.json",
            r#"{"version":6,"configurePresets":[{"name":"mine","inherits":"p1"}]}"#,
        );
        assert_eq!(
            super::list_presets(&root).unwrap_err(),
            "o preset de configure \"mine\" herda de um preset que o arquivo dele não inclui"
        );
    }
}
