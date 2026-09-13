//! Buffer formatting dispatch (`format.text`).

use serde_json::json;

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

fn workspace_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-format-{test_name}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").unwrap();
    std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("notas.txt"), "texto\n").unwrap();
    dir
}

fn open_workspace(core: &mut crate::Core, dir: &std::path::Path) -> String {
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir.to_str().unwrap() })),
    ));
    opened.response().result.as_ref().unwrap()["root"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn format_text_requires_open_workspace() {
    let mut core = core_with_empty_search_path("format-no-workspace");
    let outcome = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "format.text",
        Some(json!({ "path": "/tmp/a.rs", "text": "fn main(){}" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidRequest);
    assert_eq!(error.message, "nenhum workspace aberto");
}

#[test]
fn format_text_formats_a_rust_buffer_with_rustfmt() {
    let dir = workspace_dir("rustfmt");
    let mut core = core_with_empty_search_path("format-rustfmt");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        3_i64,
        "format.text",
        Some(json!({
            "path": format!("{root}/src/main.rs"),
            "text": "fn main(   ){ let x=1;println!(\"{x}\") ; }",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap().clone();
    assert_eq!(result["formatter"], "rustfmt");
    assert_eq!(result["changed"], true);
    assert!(
        result["path"].as_str().unwrap().ends_with("src/main.rs"),
        "path ecoado: {}",
        result["path"]
    );
    let text = result["text"].as_str().unwrap();
    assert!(text.contains("fn main() {"), "texto formatado: {text}");
    assert!(text.contains("let x = 1;"), "texto formatado: {text}");
}

#[test]
fn format_text_reports_unchanged_buffers() {
    let dir = workspace_dir("unchanged");
    let mut core = core_with_empty_search_path("format-unchanged");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "format.text",
        Some(json!({
            "path": format!("{root}/src/main.rs"),
            "text": "fn main() {}\n",
        })),
    ));
    let result = outcome.response().result.as_ref().unwrap().clone();
    assert_eq!(result["changed"], false);
    assert_eq!(result["text"], "fn main() {}\n");
}

#[test]
fn format_text_rejects_extensions_without_formatter() {
    let dir = workspace_dir("unsupported");
    let mut core = core_with_empty_search_path("format-unsupported");
    let root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        5_i64,
        "format.text",
        Some(json!({ "path": format!("{root}/notas.txt"), "text": "texto" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
    assert!(error.message.contains("nenhum formatter"));
}

#[test]
fn format_text_rejects_paths_outside_workspace() {
    let dir = workspace_dir("outside");
    let mut core = core_with_empty_search_path("format-outside");
    let _root = open_workspace(&mut core, &dir);

    let outcome = core.handle_request(&JsonRpcRequest::new(
        6_i64,
        "format.text",
        Some(json!({ "path": "/etc/hostname", "text": "x" })),
    ));
    let error = outcome.response().error.as_ref().unwrap();
    assert_eq!(error.code, JsonRpcErrorCode::InvalidParams);
}

#[test]
fn command_list_includes_format_text() {
    let mut core = core_with_empty_search_path("format-command-list");
    let outcome = core.handle_request(&JsonRpcRequest::new(7_i64, "command.list", None));
    let commands = outcome.response().result.as_ref().unwrap()["commands"]
        .as_array()
        .unwrap()
        .clone();
    let format_command = commands
        .iter()
        .find(|command| command["id"] == "format.text")
        .expect("format.text deveria estar em command.list");
    assert_eq!(format_command["defaultShortcut"], "Ctrl+Alt+L");
    assert_eq!(format_command["requiresWorkspace"], true);
}

/// Python (cadeia do roadmaps/41 bloco B, fatia 2): `format.text` num `.py`
/// roda o ruff DETECTADO (pipx/uv em `~/.local/bin`, fora do PATH do processo
/// da IDE) com `format --stdin-filename <arquivo>` no root, e devolve o stdout.
/// O ruff aqui e' um script que grava o que recebeu — o teste nao depende de
/// ter o ruff instalado — e, quando o ruff real existe, o resultado dele
/// tambem e' conferido.
#[test]
fn format_text_formats_a_python_buffer_with_ruff() {
    use std::os::unix::fs::PermissionsExt;

    let dir = workspace_dir("ruff");
    std::fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    std::fs::create_dir_all(dir.join("pacote")).unwrap();
    std::fs::write(dir.join("pacote/app.py"), "x=1\n").unwrap();
    let bin = dir.join("bin-falso");
    std::fs::create_dir_all(&bin).unwrap();
    let registro = dir.join("ruff-recebeu.txt");
    std::fs::write(
        bin.join("ruff"),
        format!(
            "#!/bin/sh\necho \"$@\" > {reg}\npwd >> {reg}\ncat >> {reg}\nprintf 'formatado\\n'\n",
            reg = registro.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(bin.join("ruff"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(&bin));
    let root = open_workspace(&mut core, &dir);
    let outcome = core.handle_request(&JsonRpcRequest::new(
        7_i64,
        "format.text",
        Some(json!({
            "path": format!("{root}/pacote/app.py"),
            "text": "def  f( a,b ):\n  return a+b\n",
        })),
    ));
    let result = outcome.response().result.clone().unwrap();
    assert_eq!(result["formatter"], "ruff");
    assert_eq!(result["changed"], true);
    assert_eq!(result["text"], "formatado\n", "{result}");
    let recebeu = std::fs::read_to_string(&registro).unwrap();
    let mut linhas = recebeu.lines();
    assert_eq!(
        linhas.next().unwrap(),
        format!("format --stdin-filename {root}/pacote/app.py"),
        "o nome do arquivo decide a configuracao (ruff.toml mais proximo)"
    );
    assert_eq!(linhas.next().unwrap(), root, "corre no root do workspace");
    assert_eq!(
        linhas.next().unwrap(),
        "def  f( a,b ):",
        "o buffer vai por stdin"
    );

    // Com o ruff real (quando existe): o texto volta formatado de verdade.
    if let Some(ruff_real) = crate::tools::ToolDetector::from_environment().find_in_path("ruff") {
        let bin_real = ruff_real.parent().unwrap().to_path_buf();
        let mut core =
            crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(bin_real));
        let root = open_workspace(&mut core, &dir);
        let outcome = core.handle_request(&JsonRpcRequest::new(
            8_i64,
            "format.text",
            Some(json!({
                "path": format!("{root}/pacote/app.py"),
                "text": "def  f( a,b ):\n  return a+b\n",
            })),
        ));
        let result = outcome.response().result.clone().unwrap();
        assert_eq!(
            result["text"], "def f(a, b):\n    return a + b\n",
            "{result}"
        );
    }

    // .pyi tambem e' do ruff; .txt continua sem formatter.
    assert_eq!(
        crate::format::formatter_for_path(std::path::Path::new("stubs/x.pyi")),
        Some(crate::format::FormatterKind::Ruff)
    );
}
