//! Incremental syntax-tree IPC dispatch.

use kinein_protocol::{JsonRpcRequest, SyntaxTreeSnapshotResult};
use serde_json::json;

use super::core_with_empty_search_path;

#[test]
fn syntax_update_returns_structural_layers_without_lsp() {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-syntax-update", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("main.rs");
    std::fs::write(&path, "fn main() {}\n").unwrap();
    let mut core = core_with_empty_search_path("syntax-update");
    let opened = core.handle_request(&JsonRpcRequest::new(
        801_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));
    assert!(opened.response().error.is_none());

    let response = core.handle_request(&JsonRpcRequest::new(
        802_i64,
        "syntaxTree.update",
        Some(json!({
            "path": path,
            "content": "fn main() {\n    let state = 1;\n}\n",
            "version": 4
        })),
    ));
    let snapshot = serde_json::from_value::<SyntaxTreeSnapshotResult>(
        response.response().result.clone().unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot.language, "rust");
    assert_eq!(snapshot.version, 4);
    assert!(snapshot.outline.iter().any(|item| item.name == "main"));
    assert!(!snapshot.folding_ranges.is_empty());
    assert!(!snapshot.highlights.is_empty());
}

/// Python entrou na fundacao (2026-09-12, bloco B do `roadmaps/41`): a mesma
/// gramatica oficial da' highlights, outline com container, folding e locals
/// — o que ate' entao so' C/C++/Rust tinham, e o que faz o indice enxergar
/// as declaracoes de um `.py`.
#[test]
fn syntax_update_understands_python_like_the_other_three() {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-syntax-python", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("sensor.py");
    let fonte = "import machine\n\nclass Sensor:\n    \"\"\"Le o ADC.\"\"\"\n    def ler(self, pino):\n        valor = pino.read()  # cru\n        return valor\n\ndef media(amostras):\n    return sum(amostras) / len(amostras)\n";
    std::fs::write(&path, fonte).unwrap();
    let mut core = core_with_empty_search_path("syntax-python");
    let opened = core.handle_request(&JsonRpcRequest::new(
        811_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));
    assert!(opened.response().error.is_none());

    let response = core.handle_request(&JsonRpcRequest::new(
        812_i64,
        "syntaxTree.update",
        Some(json!({ "path": path, "content": fonte, "version": 1 })),
    ));
    let snapshot = serde_json::from_value::<SyntaxTreeSnapshotResult>(
        response.response().result.clone().unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot.language, "python");
    assert!(!snapshot.has_errors, "a fonte e' Python valido");
    // O outline: a classe, o metodo DENTRO dela, a funcao de modulo — com
    // linha 1-based e o kind que a tags oficial usa.
    let plano: Vec<(String, String, u64)> = achatar(&snapshot.outline);
    assert!(
        plano.contains(&("Sensor".into(), "class".into(), 3)),
        "{plano:?}"
    );
    assert!(
        plano
            .iter()
            .any(|(n, k, l)| n == "ler" && *l == 5 && (k == "method" || k == "function")),
        "{plano:?}"
    );
    assert!(
        plano.contains(&("media".into(), "function".into(), 9)),
        "{plano:?}"
    );
    let classe = snapshot
        .outline
        .iter()
        .find(|i| i.name == "Sensor")
        .unwrap();
    assert!(
        classe.children.iter().any(|c| c.name == "ler"),
        "o metodo e' filho da classe: {:?}",
        classe.children
    );
    // Highlights com os escopos que o realce do editor entende.
    let escopos: std::collections::BTreeSet<&str> = snapshot
        .highlights
        .iter()
        .map(|h| h.scope.split('.').next().unwrap_or(""))
        .collect();
    for esperado in ["keyword", "function", "comment", "string"] {
        assert!(
            escopos.contains(esperado),
            "falta {esperado} em {escopos:?}"
        );
    }
    assert!(
        !snapshot.folding_ranges.is_empty(),
        "classe e funcoes dobram"
    );
    assert!(
        snapshot
            .locals
            .iter()
            .any(|l| l.kind == kinein_protocol::SyntaxLocalKind::Scope),
        "a funcao e' escopo"
    );
}

fn achatar(itens: &[kinein_protocol::SyntaxOutlineItem]) -> Vec<(String, String, u64)> {
    let mut saida = Vec::new();
    for item in itens {
        saida.push((item.name.clone(), item.kind.clone(), item.line));
        saida.extend(achatar(&item.children));
    }
    saida
}

#[test]
fn syntax_update_rejects_paths_outside_workspace() {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-syntax-confine", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let mut core = core_with_empty_search_path("syntax-confine");
    let _ = core.handle_request(&JsonRpcRequest::new(
        803_i64,
        "workspace.open",
        Some(json!({ "path": root })),
    ));

    let response = core.handle_request(&JsonRpcRequest::new(
        804_i64,
        "syntaxTree.update",
        Some(json!({
            "path": "/etc/hosts",
            "content": "127.0.0.1 localhost\n",
            "version": 1
        })),
    ));

    assert_eq!(
        response.response().error.as_ref().unwrap().code,
        kinein_protocol::JsonRpcErrorCode::InvalidParams
    );
}
