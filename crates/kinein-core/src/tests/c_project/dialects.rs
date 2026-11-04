//! Aliases e dialetos abertos: o contexto conserva o que o produtor recebeu.

use crate::index::context::CompileContext;
use serde_json::json;
use std::{path::Path, process::Command};

// Corpus das ferramentas exercitadas; nao limita o reconhecedor da IDE.
const TOOL_MODES: &[&str] = &[
    "c89",
    "c90",
    "iso9899:1990",
    "iso9899:199409",
    "c99",
    "c9x",
    "iso9899:1999",
    "c11",
    "c1x",
    "c17",
    "c18",
    "iso9899:2018",
    "c23",
    "c2x",
    "iso9899:2024",
    "c2y",
    "gnu89",
    "gnu99",
    "gnu11",
    "gnu17",
    "gnu18",
    "gnu23",
    "gnu2y",
    "-ansi",
];

#[test]
fn c_project_dialects_from_real_compilers_are_preserved_without_a_version_allowlist() {
    let root = std::env::temp_dir().join(format!("kinein-c-dialects-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut entries = Vec::new();
    for compiler in ["gcc", "clang"] {
        for &mode in TOOL_MODES {
            let index = entries.len();
            let path = root.join(format!("case_{index}.c"));
            std::fs::write(&path, "int main(void) { return 0; }\n").unwrap();
            let option = if mode.starts_with('-') {
                mode.to_owned()
            } else {
                format!("-std={mode}")
            };
            let arguments = vec![
                compiler.to_owned(),
                option,
                "-pedantic-errors".to_owned(),
                "-fsyntax-only".to_owned(),
                path.to_str().unwrap().to_owned(),
            ];
            let result = Command::new(compiler)
                .args(&arguments[1..])
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{compiler} {mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            entries.push(
                json!({"directory":root, "file":path, "arguments":arguments, "expected":mode}),
            );
        }
    }
    // Estes valores testam conservacao, sem afirmar capacidade de ferramentas futuras.
    for (name, args, expected) in [
        (
            "future",
            vec!["cc", "-std=vendor-C2099"],
            Some("vendor-C2099"),
        ),
        (
            "future_iso",
            vec!["cc", "-std=iso9899:2099"],
            Some("iso9899:2099"),
        ),
        ("default", vec!["cc"], None),
        (
            "last",
            vec!["cc", "-std=c11", "-ansi", "-std=c999"],
            Some("c999"),
        ),
        ("ansi", vec!["gcc", "--ansi"], Some("--ansi")),
        ("msvc", vec!["cl.exe", "/std:clatest"], Some("clatest")),
    ] {
        let path = root.join(format!("{name}.c"));
        std::fs::write(&path, "int main(void) { return 0; }\n").unwrap();
        entries.push(json!({"directory":root,"file":path,"arguments":args,"expected":expected}));
    }
    std::fs::write(
        root.join("compile_commands.json"),
        serde_json::to_string(&entries).unwrap(),
    )
    .unwrap();
    let context = CompileContext::load_cdb(&root);
    for entry in entries {
        let result = context.for_file(&root, Path::new(entry["file"].as_str().unwrap()), "c");
        let unit = result.unit.unwrap();
        assert_eq!(
            serde_json::to_value(unit.arguments).unwrap(),
            entry["arguments"]
        );
        assert_eq!(
            serde_json::to_value(unit.standard).unwrap(),
            entry["expected"]
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
