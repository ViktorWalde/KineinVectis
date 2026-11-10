use std::path::{Path, PathBuf};

use kinein_protocol::CmakeConfigurationInfo;

use super::{
    Configuration, Refusal, active_configuration, configuration_dir, configurations, ide_build_dir,
    owned_by_ide, preset_dirs, record_path, select_configuration, selected_configuration,
};
use crate::cmake_presets::PresetDirectory;

fn root(name: &str, presets: Option<&str>) -> PathBuf {
    let root = std::env::temp_dir()
        .join("kinein-cmake-configuration")
        .join(format!("{} {name} ação", std::process::id()));
    drop(std::fs::remove_dir_all(&root));
    std::fs::create_dir_all(&root).unwrap();
    if let Some(presets) = presets {
        std::fs::write(root.join("CMakePresets.json"), presets).unwrap();
    }
    root
}

fn configured(directory: &Path, preset: Option<&str>) {
    std::fs::create_dir_all(directory).unwrap();
    std::fs::write(directory.join("CMakeCache.txt"), "").unwrap();
    if let Some(preset) = preset {
        std::fs::write(directory.join(".kinein-preset"), format!("{preset}\n")).unwrap();
    }
}

fn pending(directory: &Path, preset: Option<&str>) {
    std::fs::create_dir_all(directory).unwrap();
    let body = serde_json::json!({"preset": preset, "kit": ""}).to_string();
    std::fs::write(directory.join(".kinein-configure-pending"), body).unwrap();
}

fn active(root: &Path) -> (Option<String>, PathBuf, &'static str) {
    let Configuration {
        preset,
        directory,
        source,
    } = active_configuration(root).unwrap();
    (preset, directory, source)
}

fn refused(root: &Path) -> String {
    match active_configuration(root).unwrap_err() {
        Refusal::Refused(reason) => reason,
        several @ Refusal::Several(..) => panic!("{several:?}"),
    }
}

const TWO: &str = r#"{"version":3,"configurePresets":[
  {"name":"debug","binaryDir":"${sourceDir}/build/${presetName}"},
  {"name":"release local"},
  {"name":"elsewhere","binaryDir":"${sourceParentDir}/fora/${presetName}"},
  {"name":"a/c","binaryDir":"vazio"},
  {"name":"."},
  {"name":".."},
  {"name":"a/b"},
  {"name":"a\\b"},
  {"name":"vendor","binaryDir":"$vendor{acme}"}]}"#;

#[test]
fn each_preset_configures_in_its_binary_dir_or_a_folder_of_the_ide() {
    let root = root("dir", Some(TWO));
    let dir = |preset: Option<&str>| configuration_dir(&root, preset);
    assert_eq!(dir(None).unwrap(), root.join(".kinein/build"));
    assert_eq!(dir(Some("debug")).unwrap(), root.join("build/debug"));
    assert_eq!(
        dir(Some("release local")).unwrap(),
        root.join(".kinein/presets/release local")
    );
    assert_eq!(
        dir(Some("elsewhere")).unwrap(),
        root.parent().unwrap().join("fora/elsewhere"),
        "uma pasta fora da raiz vale, como no C-05"
    );
    // Um binaryDir declarado vale mesmo com um nome que não serve de pasta.
    // O nome vazio servia aqui até o C1-25: o CMake recusa o arquivo por ele
    // ("Invalid Preset Name"), e o caso foi para `cmake_presets/verdict`.
    assert_eq!(dir(Some("a/c")).unwrap(), root.join("vazio"));
    for name in [".", "..", "a/b", "a\\b"] {
        let reason = dir(Some(name)).unwrap_err();
        assert!(
            reason.contains("não declara binaryDir, e o nome não serve de pasta"),
            "{name}: {reason}"
        );
    }
    // O CMake ignora o preset com `$vendor{}` (C1-24): ele não tem pasta.
    assert_eq!(
        dir(Some("vendor")).unwrap_err(),
        "a pasta do preset vendor não é conhecida: o CMake ignora o preset vendor: ele usa \
         $vendor{}"
    );
    assert_eq!(
        dir(Some("ghost")).unwrap_err(),
        "a pasta do preset ghost não é conhecida: preset de configure inexistente: ghost"
    );
    assert_eq!(
        preset_dirs(&root),
        [
            ("debug", root.join("build/debug")),
            ("elsewhere", root.parent().unwrap().join("fora/elsewhere")),
            ("a/c", root.join("vazio")),
            ("release local", root.join(".kinein/presets/release local")),
        ]
        .map(|(name, directory)| PresetDirectory {
            name: name.to_owned(),
            directory,
        })
    );

    assert!(owned_by_ide(&root, &root.join(".kinein/build")));
    assert!(owned_by_ide(
        &root,
        &root.join(".kinein/presets/release local")
    ));
    for foreign in [
        root.join("build/debug"),
        root.join(".kinein/presets"),
        root.join(".kinein/presets/a/b"),
        root.join(".kinein"),
        root.clone(),
    ] {
        assert!(!owned_by_ide(&root, &foreign), "{}", foreign.display());
    }
}

/// O menu do C1-23: cada preset visível com a pasta dele, ou o motivo de
/// ela não ser conhecida, e se ela já tem cache. Não depende da configuração
/// ativa. O preset que o `CMake` ignora (`$vendor{}` em qualquer campo, C1-24)
/// não entra.
#[test]
fn the_menu_lists_every_visible_preset_with_its_folder_or_why_not() {
    let root = root(
        "menu",
        Some(
            r#"{"version":3,"configurePresets":[
  {"name":"base","hidden":true,"binaryDir":"${sourceDir}/base"},
  {"name":"debug","inherits":"base","displayName":"Depuração local",
   "binaryDir":"${sourceDir}/build/${presetName}"},
  {"name":"release local"},
  {"name":"elsewhere","binaryDir":"${sourceParentDir}/saídas fora/${presetName}"},
  {"name":"a/b"},
  {"name":"vendor","binaryDir":"${sourceDir}/v","cacheVariables":{"V":"$vendor{acme}"}},
  {"name":"shared","binaryDir":"${sourceDir}/build/debug"}]}"#,
        ),
    );
    let outside = root.parent().unwrap().join("saídas fora/elsewhere");
    configured(&root.join("build/debug"), Some("debug"));
    // Cache de outra configuração em `.kinein/build` não conta para nenhuma.
    configured(&root.join(".kinein/build"), Some("release local"));
    // A pasta de `release local` existe, mas sem cache.
    std::fs::create_dir_all(root.join(".kinein/presets/release local")).unwrap();
    let info = |preset: &str, name: Option<&str>, dir: Result<&str, &str>, done: bool| {
        CmakeConfigurationInfo {
            preset: preset.to_owned(),
            display_name: name.map(str::to_owned),
            build_dir: dir.ok().map(str::to_owned),
            build_dir_refused: dir.err().map(str::to_owned),
            configured: done,
            condition_note: None,
        }
    };
    let expected = vec![
        info("debug", Some("Depuração local"), Ok("build/debug"), true),
        info(
            "release local",
            None,
            Ok(".kinein/presets/release local"),
            false,
        ),
        info("elsewhere", None, Ok(outside.to_str().unwrap()), false),
        info(
            "a/b",
            None,
            Err(
                "o preset \"a/b\" não declara binaryDir, e o nome não serve de pasta em \
                 .kinein/presets: declare binaryDir no preset ou mude o nome",
            ),
            false,
        ),
        // Dois presets na mesma pasta: o cache dela vale para os dois.
        info("shared", None, Ok("build/debug"), true),
    ];
    assert_eq!(configurations(&root), expected);
    configured(&outside, None);
    assert!(configurations(&root)[2].configured, "fora da raiz também");
    // Escolher outra não muda a lista.
    select_configuration(&root, Some("release local")).unwrap();
    let after = configurations(&root);
    assert_eq!(after.len(), expected.len());
    assert_eq!(after[0], expected[0]);

    // Sem presets, ou com presets que o CMake recusaria: nada a oferecer.
    std::fs::remove_file(root.join("CMakePresets.json")).unwrap();
    assert_eq!(configurations(&root), []);
    std::fs::write(
        root.join("CMakePresets.json"),
        r#"{"version":3,"configurePresets":[{"name":"debug"},{"name":"debug"}]}"#,
    )
    .unwrap();
    assert_eq!(configurations(&root), []);
    // Uma macro sem fechamento num preset oculto que ninguém usa recusa o
    // arquivo inteiro no CMake 4.4.5 (C1-24): o menu não lista nenhum.
    std::fs::write(
        root.join("CMakePresets.json"),
        r#"{"version":3,"configurePresets":[{"name":"debug"},
  {"name":"open","hidden":true,"binaryDir":"${presetName"}]}"#,
    )
    .unwrap();
    assert_eq!(configurations(&root), []);
    drop(std::fs::remove_dir_all(outside.parent().unwrap()));
}

#[test]
fn the_record_comes_first_and_is_never_replaced() {
    let root = root("record", Some(TWO));
    // Evidência da IDE em .kinein/build perde para o registro.
    configured(&ide_build_dir(&root), Some("debug"));
    pending(&ide_build_dir(&root), Some("debug"));
    select_configuration(&root, Some("release local")).unwrap();
    assert_eq!(
        active(&root),
        (
            Some("release local".to_owned()),
            root.join(".kinein/presets/release local"),
            "configuration"
        )
    );
    select_configuration(&root, Some("ghost")).unwrap();
    assert_eq!(
        refused(&root),
        "configuração escolhida removida dos presets: ghost; escolha outra"
    );
    select_configuration(&root, Some("a/b")).unwrap();
    assert!(refused(&root).contains("o nome não serve de pasta"));
    std::fs::write(record_path(&root), "{").unwrap();
    assert!(
        refused(&root).starts_with("registro da configuração escolhida inválido"),
        "{}",
        refused(&root)
    );
    select_configuration(&root, Some("debug")).unwrap();
    std::fs::write(root.join("CMakePresets.json"), "{").unwrap();
    assert!(
        refused(&root).starts_with("o CMake recusa os presets: CMakePresets.json:1: "),
        "{}",
        refused(&root)
    );
    let _native = crate::cmake_presets::native_reading();
    assert!(refused(&root).starts_with("CMakePresets.json inválido"));
}

#[test]
fn without_a_record_the_ide_build_dir_then_the_default_then_no_preset() {
    let one = r#"{"version":3,"configurePresets":[
      {"name":"only","binaryDir":"out/${presetName}"},
      {"name":"old","hidden":true}]}"#;
    let root = root("automatic", Some(one));
    assert_eq!(
        active(&root),
        (
            Some("only".to_owned()),
            root.join("out/only"),
            "CMakePresets.json"
        )
    );
    // Preset anotado sem cache não conta; com cache, .kinein/build vale,
    // como antes do C-04.
    std::fs::create_dir_all(ide_build_dir(&root)).unwrap();
    std::fs::write(ide_build_dir(&root).join(".kinein-preset"), "only\n").unwrap();
    assert_eq!(active(&root).2, "CMakePresets.json");
    configured(&ide_build_dir(&root), Some("only"));
    assert_eq!(
        active(&root),
        (Some("only".to_owned()), ide_build_dir(&root), "buildDir")
    );
    // Anotado com um preset que saiu (ou oculto): volta ao padrão.
    configured(&ide_build_dir(&root), Some("old"));
    assert_eq!(active(&root).2, "CMakePresets.json");
    // A tentativa pendente vem antes do anotado, também sem preset.
    pending(&ide_build_dir(&root), None);
    assert_eq!(active(&root), (None, ide_build_dir(&root), "buildDir"));
    pending(&ide_build_dir(&root), Some("gone"));
    assert_eq!(
        refused(&root),
        "preset do configure incompleto removido: gone; escolha outro"
    );
    std::fs::write(ide_build_dir(&root).join(".kinein-configure-pending"), "{").unwrap();
    assert!(refused(&root).starts_with("registro do configure incompleto inválido"));
    std::fs::remove_dir_all(ide_build_dir(&root)).unwrap();

    std::fs::write(
        root.join("CMakePresets.json"),
        r#"{"version":3,"configurePresets":[{"name":"a","binaryDir":"a"},{"name":"b","binaryDir":"b"}]}"#,
    )
    .unwrap();
    assert_eq!(
        active_configuration(&root),
        Err(Refusal::Several(
            "CMakePresets.json",
            vec!["a".to_owned(), "b".to_owned()]
        ))
    );
    assert_eq!(
        active_configuration(&root).unwrap_err().reason(),
        "o projeto tem 2 presets de configure em CMakePresets.json (a, b): escolha um"
    );
    std::fs::write(
        root.join("CMakePresets.json"),
        r#"{"version":3,"configurePresets":[{"name":"only"}],"include":["x.json"]}"#,
    )
    .unwrap();
    assert!(refused(&root).contains("File version must be 4 or higher for include support"));
    let native = crate::cmake_presets::native_reading();
    assert!(refused(&root).contains("include exige version 4"));
    drop(native);

    std::fs::remove_file(root.join("CMakePresets.json")).unwrap();
    assert_eq!(active(&root), (None, ide_build_dir(&root), "noPresets"));
}

#[test]
fn the_record_round_trips_and_an_invalid_one_is_reported() {
    let root = root("round trip", None);
    assert_eq!(selected_configuration(&root), Ok(None));
    select_configuration(&root, None).unwrap();
    select_configuration(&root, Some("debug")).unwrap();
    assert_eq!(selected_configuration(&root), Ok(Some("debug".to_owned())));
    std::fs::write(record_path(&root), r#"{"preset":"debug","extra":1}"#).unwrap();
    assert!(
        selected_configuration(&root)
            .unwrap_err()
            .starts_with("registro da configuração escolhida inválido")
    );
    select_configuration(&root, None).unwrap();
    assert_eq!(selected_configuration(&root), Ok(None));
}
