//! CDB controlada: flags de linguagem, aliases e ausencia de contexto.

use super::*;
use crate::index::context::{CompileContext, ContextTools};

#[test]
fn c_identity_cdb_overrides_extensions_without_a_standard_allowlist() {
    let p = Project::new("commands");
    let ordered_input = p.root.join("after.c").display().to_string();
    let cases = [
        ("after.c", vec!["gcc", ordered_input.as_str(), "-xc++"], "c"),
        ("legacy.c", vec!["gcc", "-std=iso9899:199409"], "c"),
        ("future.c", vec!["clang", "-std=vendor-C2099"], "c"),
        ("upper.C", vec!["gcc"], "cpp"),
        ("upper.H", vec!["gcc"], "cpp"),
        ("override.cpp", vec!["clang", "-xc"], "c"),
        ("override.c", vec!["gcc", "-x", "c++"], "cpp"),
        ("driver.c", vec!["clang++"], "cpp"),
        ("reset.c", vec!["clang", "-xc++", "-xnone"], "c"),
        ("direct.h", vec!["clang", "-x", "c-header"], "c"),
        ("cpp.h", vec!["clang", "-xc++-header"], "cpp"),
        ("msvc.c", vec!["cl.exe", "/TP"], "cpp"),
        ("msvc.cpp", vec!["cl.exe", "/TC"], "c"),
    ];
    let mut entries = Vec::new();
    for (path, arguments, _) in &cases {
        p.put(path, "int value(void);\n");
        entries.push(json!({"directory":p.root,"file":p.root.join(path),"arguments":arguments}));
    }
    p.put("compile_commands.json", &json!(entries).to_string());
    let context = CompileContext::load_cdb(&p.root);
    for (path, arguments, expected) in &cases {
        let path = p.root.join(path);
        assert_eq!(
            context.document_language(&path),
            *expected,
            "{}",
            path.display()
        );
        let result = context.for_file(&p.root, &path, "c");
        assert_eq!(result.language, *expected);
        assert_eq!(result.unit.unwrap().arguments, *arguments);
    }
}

#[test]
fn c_identity_headers_follow_homogeneous_cdb_and_report_ambiguity() {
    let p = Project::new("headers");
    p.put("main.c", "int main(void) { return 0; }\n");
    p.put("header.h", "int value(void);\n");
    let entries =
        json!([{"directory":p.root,"file":p.root.join("main.c"),"arguments":["gcc","-std=c2099"]}]);
    p.put("compile_commands.json", &entries.to_string());
    let context = CompileContext::load(&p.root, &ContextTools::default());
    assert_eq!(context.document_language(&p.root.join("header.h")), "c");
    let mut duplicated = entries.as_array().unwrap().clone();
    duplicated
        .push(json!({"directory":p.root,"file":p.root.join("main.c"),"arguments":["clang++"]}));
    p.put("compile_commands.json", &json!(duplicated).to_string());
    let context = CompileContext::load_cdb(&p.root);
    let header = context.for_file(&p.root, &p.root.join("header.h"), "cpp");
    assert_eq!(header.language, "cpp");
    assert!(header.hint.unwrap().contains("ambíguo"));
    let empty = CompileContext::default();
    assert_eq!(empty.document_language(&p.root.join("header.h")), "cpp");
    assert_eq!(empty.document_language(&p.root.join("unit.C")), "cpp");
}
