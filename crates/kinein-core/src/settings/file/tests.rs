//! O `settings.json` que a IDE não entende (INF-07), as chaves que ela não
//! conhece (INF-11) e o link simbólico (INF-12), nos dois escopos.
//!
//! O global entra pelo `Place::global` com um caminho de teste: o de
//! verdade vem do `XDG_CONFIG_HOME`, que um teste em processo não troca. Os
//! links são de pasta, ou para o que não existe: `ln -s` no Unix e a junção
//! no Windows, que não pede privilégio (60 §3.3).

use std::path::{Path, PathBuf};

use kinein_protocol::SettingsValues;
use serde_json::{Value, json};

use super::Place;
use crate::kinein_dir::link;
use crate::settings::{SetError, set_at, stored};

fn base(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-settings-tests")
        .join(format!("{}-file-{name}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    crate::platform::canonicalize(&dir).unwrap()
}

fn is_link(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn font(size: u32) -> SettingsValues {
    SettingsValues {
        editor_font_size: Some(size),
        ..SettingsValues::default()
    }
}

/// Os dois escopos de `dir`: o global em `config/kinein-vectis` e o do
/// projeto em `projeto`, com o nome que o motivo dá a cada arquivo.
fn places(dir: &Path) -> [(Place, PathBuf, String); 2] {
    let global = dir
        .join("config")
        .join("kinein-vectis")
        .join("settings.json");
    let root = dir.join("projeto");
    let workspace = root.join(".kinein").join("settings.json");
    [
        (
            Place::global(global.clone()),
            global.clone(),
            global.display().to_string(),
        ),
        (
            Place::workspace(&root),
            workspace,
            ".kinein/settings.json".to_owned(),
        ),
    ]
}

/// A leitura devolve o vazio e o motivo; o `settings.set` é recusado com o
/// mesmo motivo e não toca no arquivo.
fn assert_refused(place: &Place, path: &Path, problem: &str) {
    let before = std::fs::symlink_metadata(path).ok().map(|metadata| {
        (
            metadata.file_type().is_symlink(),
            metadata.is_dir(),
            std::fs::read(path).ok(),
        )
    });
    let read = stored(place);
    assert_eq!(read.values, SettingsValues::default(), "{problem}");
    assert_eq!(read.problem.as_deref(), Some(problem));
    assert_eq!(
        set_at(place, &font(15)),
        Err(SetError::Refused(format!(
            "As configurações não foram gravadas: {problem}. A IDE não grava por cima do que não entende; corrija ou remova esse caminho e tente de novo."
        )))
    );
    let after = std::fs::symlink_metadata(path).ok().map(|metadata| {
        (
            metadata.file_type().is_symlink(),
            metadata.is_dir(),
            std::fs::read(path).ok(),
        )
    });
    assert_eq!(after, before, "{problem}");
}

/// INF-07: os cinco casos medidos em 2026-11-12 e os outros motivos da
/// leitura, cada um nos dois escopos, com o `autoSave: false` que deixa de
/// valer mas fica no disco.
#[test]
fn a_file_the_ide_cannot_read_is_kept_and_says_why() {
    let mut big = b"{\"schemaVersion\": 1, \"x\": \"".to_vec();
    big.resize(big.len() + 1024 * 1024, b'a');
    big.extend_from_slice(b"\"}");
    let cases: [(&str, Vec<u8>, &str); 9] = [
        (
            "sem-schema",
            b"{\"autoSave\": false}".to_vec(),
            "não tem schemaVersion",
        ),
        (
            "schema-2",
            b"{\"schemaVersion\": 2, \"autoSave\": false}".to_vec(),
            "tem schemaVersion 2, e esta versão da IDE só conhece a 1",
        ),
        (
            "schema-texto",
            b"{\"schemaVersion\": \"1\", \"autoSave\": false}".to_vec(),
            r#"tem schemaVersion "1", e esta versão da IDE só conhece a 1"#,
        ),
        (
            "cortado",
            b"{\"schemaVersion\": 1, \"autoSave\": fal".to_vec(),
            "não é JSON válido (linha 1, coluna 36)",
        ),
        (
            "nao-utf8",
            b"{\"schemaVersion\": 1, \"autoSave\": false, \"n\": \"\xff\"}".to_vec(),
            "não está em UTF-8",
        ),
        (
            "tipo-errado",
            b"{\"schemaVersion\": 1, \"autoSave\": \"nao\"}".to_vec(),
            r#"tem em autoSave um valor que esta versão da IDE não entende (invalid type: string "nao", expected a boolean)"#,
        ),
        ("lista", b"[1, 2]".to_vec(), "não é um objeto JSON"),
        ("vazio", Vec::new(), "não é JSON válido (linha 1, coluna 0)"),
        ("grande", big, "passa de 1 MiB, o limite de leitura da IDE"),
    ];
    for (name, body, reason) in cases {
        let dir = base(&format!("entende-{name}"));
        for (place, path, shown) in places(&dir) {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &body).unwrap();
            assert_refused(&place, &path, &format!("{shown} {reason}"));
        }
    }
}

/// INF-07 e INF-12: a pasta no lugar do arquivo não é lida nem trocada, nos
/// dois escopos; no projeto, o `.kinein` que é arquivo também não.
#[test]
fn what_is_not_a_regular_file_is_kept() {
    let dir = base("nao-regular");
    for (place, path, shown) in places(&dir) {
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("dentro.txt"), "fica").unwrap();
        assert_refused(&place, &path, &format!("{shown} não é um arquivo regular"));
        assert_eq!(
            std::fs::read_to_string(path.join("dentro.txt")).unwrap(),
            "fica"
        );
    }
    let root = dir.join("kinein-arquivo");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(".kinein"), "não é pasta").unwrap();
    assert_refused(
        &Place::workspace(&root),
        &root.join(".kinein"),
        ".kinein não é uma pasta",
    );
}

/// INF-11: a chave que esta versão não conhece fica no disco com o valor
/// dela, nos dois escopos; a conhecida que vem `null` também, até ser
/// gravada, e nunca em dobro.
#[test]
fn the_keys_this_version_does_not_know_stay_on_disk() {
    let dir = base("chaves");
    for (place, path, _) in places(&dir) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"{"schemaVersion": 1, "autoSave": false, "chaveFutura": {"a": [1, 2]}, "outra": 7, "formatOnSave": null}"#,
        )
        .unwrap();
        let read = stored(&place);
        assert_eq!(read.problem, None);
        assert_eq!(read.values.auto_save, Some(false));
        set_at(&place, &font(15)).unwrap();
        let written: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            written,
            json!({
                "schemaVersion": 1,
                "autoSave": false,
                "editorFontSize": 15,
                "chaveFutura": { "a": [1, 2] },
                "outra": 7,
                "formatOnSave": null,
            })
        );
        set_at(
            &place,
            &SettingsValues {
                format_on_save: Some(true),
                ..SettingsValues::default()
            },
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches("\"formatOnSave\"").count(), 1, "{text}");
        assert_eq!(text.matches("\"schemaVersion\"").count(), 1, "{text}");
        let written: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(written["formatOnSave"], true);
        assert_eq!(written["chaveFutura"], json!({ "a": [1, 2] }));
        assert_eq!(written["outra"], 7);
        assert_eq!(written["editorFontSize"], 15);
    }
}

/// INF-12: no projeto, o link no `.kinein` ou no arquivo é recusado, para
/// uma pasta de fora com um `settings.json`, para uma pasta e para o que não
/// existe; nada se cria nem muda fora do projeto, e o link fica.
#[test]
fn the_project_does_not_follow_a_link_and_writes_nothing_outside() {
    let dir = base("link-projeto");
    let outside = dir.join("fora");
    std::fs::create_dir_all(&outside).unwrap();
    let original = r#"{"schemaVersion": 1, "autoSave": false}"#;
    std::fs::write(outside.join("settings.json"), original).unwrap();
    let folder_link = ".kinein é um link simbólico, que a IDE não segue no projeto";
    let file_link = ".kinein/settings.json é um link simbólico, que a IDE não segue no projeto";
    let cases: [(&str, bool, PathBuf, &str); 4] = [
        ("pasta-existente", true, outside.clone(), folder_link),
        ("pasta-ausente", true, dir.join("ausente-1"), folder_link),
        ("arquivo-ausente", false, dir.join("ausente-2"), file_link),
        ("arquivo-pasta", false, outside.clone(), file_link),
    ];
    for (name, on_folder, target, problem) in cases {
        let root = dir.join(name);
        std::fs::create_dir_all(&root).unwrap();
        let linked = if on_folder {
            root.join(".kinein")
        } else {
            std::fs::create_dir_all(root.join(".kinein")).unwrap();
            root.join(".kinein").join("settings.json")
        };
        link(&target, &linked);
        assert_refused(&Place::workspace(&root), &linked, problem);
        assert!(is_link(&linked), "{name}");
        assert_eq!(
            std::fs::read_to_string(outside.join("settings.json")).unwrap(),
            original,
            "{name}"
        );
        let names: Vec<_> = std::fs::read_dir(&outside)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(names, ["settings.json"], "{name}");
    }
    assert!(!dir.join("ausente-1").exists());
    assert!(!dir.join("ausente-2").exists());
}

/// INF-12: o global segue o link, porque é da pessoa e pode vir de dotfiles:
/// lê e grava o alvo, e o link fica; o alvo que não é arquivo regular é
/// recusado como o arquivo que não é.
#[test]
fn the_global_follows_a_link_and_refuses_what_is_not_a_file() {
    let dir = base("link-global");
    let linked = dir.join("dotfiles").join("kinein-vectis");
    std::fs::create_dir_all(&linked).unwrap();
    std::fs::write(
        linked.join("settings.json"),
        r#"{"schemaVersion": 1, "autoSave": false, "chaveFutura": 7}"#,
    )
    .unwrap();
    let config = dir.join("config");
    std::fs::create_dir_all(&config).unwrap();
    link(&linked, &config.join("kinein-vectis"));
    let place = Place::global(config.join("kinein-vectis").join("settings.json"));
    let read = stored(&place);
    assert_eq!(read.problem, None);
    assert_eq!(read.values.auto_save, Some(false));
    set_at(&place, &font(15)).unwrap();
    assert!(is_link(&config.join("kinein-vectis")));
    let written: Value =
        serde_json::from_slice(&std::fs::read(linked.join("settings.json")).unwrap()).unwrap();
    assert_eq!(
        written,
        json!({ "schemaVersion": 1, "autoSave": false, "editorFontSize": 15, "chaveFutura": 7 })
    );

    let other = dir.join("config-2").join("kinein-vectis");
    std::fs::create_dir_all(&other).unwrap();
    let folder = dir.join("uma-pasta");
    std::fs::create_dir_all(&folder).unwrap();
    let path = other.join("settings.json");
    link(&folder, &path);
    assert_refused(
        &Place::global(path.clone()),
        &path,
        &format!("{} não é um arquivo regular", path.display()),
    );
    assert!(is_link(&path));
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
}
