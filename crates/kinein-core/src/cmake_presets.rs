//! Os configure presets do projeto: a lista, o padrao, os campos com
//! `inherits` e o `binaryDir` resolvido.
//!
//! O `CMake` nao informa o `binaryDir` de um preset sem configurar: medido com
//! o 4.4.5, `cmake --preset X -N` so' lista o cache. Por isso a IDE le
//! `CMakePresets.json` e `CMakeUserPresets.json` como o cmake-presets(7)
//! descreve, com `include` (C1-18, `files.rs`) e com as macros de todos os
//! presets expandidas ao ler, como o `CMake` faz (C1-24, `expansion.rs`). O
//! que ela nao sabe resolver fica sem pasta, nunca com uma pasta adivinhada
//! (consolidacao C §16), e o que o `CMake` recusa ela tambem recusa, com o
//! motivo (§17).

mod expansion;
mod files;
mod macros;

use std::path::{Path, PathBuf};

use kinein_protocol::CmakePresetInfo;
use serde_json::Value;

use expansion::{Preset, Reading};
use files::{Entry, Kind, PROJECT, Presets, USER};

/// Os presets lidos como o `CMake` le: os arquivos, e o que a expansao das
/// macros e as `condition` deixaram fora da lista.
fn load(root: &Path) -> Result<(Presets, Reading), String> {
    let presets = files::load(root)?;
    let reading = expansion::read(root, &presets)?;
    Ok((presets, reading))
}

/// Os configure presets que o `CMake` lista: nao ocultos, sem `$vendor{}` e
/// com a `condition` verdadeira (ou que a IDE nao decide, C1-26).
fn listed<'a>(presets: &'a Presets, reading: &'a Reading) -> impl Iterator<Item = &'a Entry> {
    presets.entries.iter().filter(|entry| {
        entry.value.get("hidden").and_then(Value::as_bool) != Some(true)
            && reading.listed(&entry.name)
    })
}

/// Lista os configure presets de `CMakePresets.json` e
/// `CMakeUserPresets.json`, com o que eles incluem, na ordem dos arquivos.
pub fn list_presets(root: &Path) -> Result<Vec<CmakePresetInfo>, String> {
    let (presets, reading) = load(root)?;
    Ok(listed(&presets, &reading)
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
/// Os configure presets listados do primeiro lado que tem algum: o de
/// `CMakeUserPresets.json` (a escolha do usuario para ESTA maquina vence),
/// com o que so' ele inclui, senao o de `CMakePresets.json`. Ate' 2026-10-16
/// (P0, 40.7 §7.46) valia o PRIMEIRO da lista; desde a 0.4.0b, com varios, a
/// UI pergunta (`roadmaps/52` §14.2).
#[must_use]
pub fn default_preset(root: &Path) -> PresetDefault {
    let (presets, reading) = match load(root) {
        Ok(loaded) => loaded,
        Err(reason) => return PresetDefault::Refused(reason),
    };
    for (file, user_side) in [(USER, true), (PROJECT, false)] {
        let mut names: Vec<String> = listed(&presets, &reading)
            .filter(|entry| presets.file_of(entry).user_side == user_side)
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

/// Um campo de configure preset depois da heranca, com as macros expandidas
/// como o `CMake` (C1-24): o proprio preset primeiro, depois os pais na
/// ordem declarada. Valor `null` desfaz a heranca. Valor objeto de
/// `cacheVariables` usa seu `value`. Presets que o `CMake` recusaria ou
/// ignora nao tem campo.
pub(crate) fn preset_value(root: &Path, preset: &str, field: &[&str]) -> Option<String> {
    let (presets, reading) = load(root).ok()?;
    let entry = presets.find(preset)?;
    if reading.unusable(preset).is_some() {
        return None;
    }
    Preset::new(root, &presets, Kind::Configure, entry)
        .field(field)
        .ok()
        .flatten()
}

/// Um configure preset visivel neste host e a pasta que ele declara.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PresetDirectory {
    pub name: String,
    pub directory: PathBuf,
}

/// O `binaryDir` resolvido dos configure presets que o `CMake` lista, na
/// ordem dos arquivos. Um preset sem `binaryDir` ou com pasta nao resolvida
/// nao aparece; presets recusados nao tem nenhuma.
pub(crate) fn binary_dirs(root: &Path) -> Vec<PresetDirectory> {
    let Ok((presets, reading)) = load(root) else {
        return Vec::new();
    };
    listed(&presets, &reading)
        .filter_map(|entry| {
            resolve_binary_dir(root, &presets, entry)
                .ok()
                .flatten()
                .map(|directory| PresetDirectory {
                    name: entry.name.clone(),
                    directory,
                })
        })
        .collect()
}

/// A pasta de build que o preset declara; `None` quando ele nao declara
/// `binaryDir` (C1-19: a IDE configura em `.kinein/presets/<preset>`). Erro
/// quando os presets sao recusados, o preset nao existe, o `CMake` nao o usa
/// (`$vendor{}` ou `condition` falsa) ou a pasta nao se resolve.
pub(crate) fn binary_dir(root: &Path, preset: &str) -> Result<Option<PathBuf>, String> {
    let (presets, reading) = load(root)?;
    let Some(entry) = presets.find(preset) else {
        return Err(format!("preset de configure inexistente: {preset}"));
    };
    if let Some(reason) = reading.unusable(preset) {
        return Err(reason);
    }
    resolve_binary_dir(root, &presets, entry)
}

fn resolve_binary_dir(
    root: &Path,
    presets: &Presets,
    entry: &Entry,
) -> Result<Option<PathBuf>, String> {
    let Some(expanded) =
        Preset::new(root, presets, Kind::Configure, entry).field(&["binaryDir"])?
    else {
        return Ok(None);
    };
    let path = PathBuf::from(expanded);
    Ok(Some(if path.is_absolute() {
        path
    } else {
        root.join(path)
    }))
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
            binary_dir(&root, "dbg").unwrap().unwrap(),
            root.parent()
                .unwrap()
                .join(format!("{name}-dbg"))
                .join("Ninja")
                .join("Linux")
        );
        let separator = if cfg!(windows) { ";" } else { ":" };
        assert_eq!(
            binary_dir(&root, "dollar").unwrap().unwrap(),
            root.join(format!("b/$x{separator}y"))
        );
        assert_eq!(
            binary_dir(&root, "relative").unwrap().unwrap(),
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
              {"name":"unknownvar","binaryDir":"${sourceDir}/u$env{KV_PRESETS_NOT_SET_ANYWHERE}"}]}"#,
        );
        // `$penv` le o ambiente do core, sem o `environment` do preset.
        let home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(
            binary_dir(&root, "inherited").unwrap().unwrap(),
            PathBuf::from(format!("{home}/from-base/x"))
        );
        assert_eq!(
            binary_dir(&root, "override").unwrap().unwrap(),
            root.join("mine")
        );
        assert_eq!(
            binary_dir(&root, "removed").unwrap().unwrap(),
            root.join("z")
        );
        assert_eq!(
            binary_dir(&root, "unknownvar").unwrap().unwrap(),
            root.join("u")
        );
    }

    /// O que o `CMake` 4.4.5 ignora ou nao usa nao vira pasta (C1-24): o
    /// `$vendor{}` em qualquer campo e a `condition` falsa. `${generator}`
    /// sem `generator` expande vazio, como no `CMake`; ate' o C1-24 a IDE
    /// recusava a pasta. Macro desconhecida ou sem fechamento recusa o arquivo
    /// (`expansion/tests.rs`).
    #[test]
    fn presets_cmake_ignores_or_disables_have_no_directory() {
        let root = root(
            "unresolvable",
            r#"{"version":6,"configurePresets":[
              {"name":"vendor","binaryDir":"${sourceDir}/$vendor{acme}"},
              {"name":"vendorcache","binaryDir":"${sourceDir}/vc","cacheVariables":{"X":"$vendor{acme}"}},
              {"name":"nogenerator","binaryDir":"${sourceDir}/g[${generator}]"},
              {"name":"nodir"},
              {"name":"windows","binaryDir":"w","condition":{"type":"equals","lhs":"${hostSystemName}","rhs":"Windows"}},
              {"name":"ok","binaryDir":"ok"}]}"#,
        );
        for (preset, reason) in [
            (
                "vendor",
                "o CMake ignora o preset vendor: ele usa $vendor{}",
            ),
            (
                "vendorcache",
                "o CMake ignora o preset vendorcache: ele usa $vendor{}",
            ),
            (
                "windows",
                "a condition do preset windows é falsa: o CMake não o usa",
            ),
            ("absent", "preset de configure inexistente: absent"),
        ] {
            assert_eq!(binary_dir(&root, preset).unwrap_err(), reason, "{preset}");
        }
        assert_eq!(
            binary_dir(&root, "nogenerator").unwrap().unwrap(),
            root.join("g[]")
        );
        assert_eq!(binary_dir(&root, "nodir"), Ok(None));
        assert_eq!(names(&root), ["nogenerator", "nodir", "ok"]);
        assert_eq!(
            binary_dirs(&root),
            [
                PresetDirectory {
                    name: "nogenerator".to_owned(),
                    directory: root.join("g[]"),
                },
                PresetDirectory {
                    name: "ok".to_owned(),
                    directory: root.join("ok"),
                }
            ]
        );
        assert_eq!(preset_value(&root, "vendorcache", &["binaryDir"]), None);
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
            binary_dir(&root, "abs").unwrap().unwrap(),
            Path::new("/opt/kv build")
        );
        let listed: Vec<String> = binary_dirs(&root).into_iter().map(|p| p.name).collect();
        assert_eq!(listed, ["abs", "mine"]);
        assert_eq!(binary_dir(&root, "mine").unwrap().unwrap(), root.join("m"));
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
        assert_eq!(
            binary_dir(&root, "top").unwrap().unwrap(),
            root.join("out/top")
        );
        assert_eq!(
            binary_dir(&root, "sub").unwrap().unwrap(),
            root.join("sub/out/sub")
        );
        assert_eq!(
            binary_dir(&root, "shared").unwrap().unwrap(),
            root.join("shared/s")
        );
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
        assert_eq!(
            binary_dir(&root, "mine").unwrap().unwrap(),
            root.join("local/m")
        );
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
