//! O indice do projeto INTEIRO (pilar 0 do `roadmaps/42`, decisao do autor em
//! 2026-09-12: a IDE le todas as pastas, arquivos e declaracoes).
//!
//! O que se prova: que o build ve TODO arquivo (contado) e le SO' os de fonte;
//! que pastas de saida ficam fora; que as declaracoes vem com linha e
//! container (o metodo dentro da classe C++); que a busca ordena exato >
//! prefixo > substring; que o incremento reindexa o que mudou e esquece o que
//! sumiu; que arquivo grande ou ilegivel e' contado e DITO em `skipped`; que o
//! `fn` dentro de `impl` nao aparece em dobro; e, pelo despacho real, que abrir
//! o workspace ja' deixa o indice pronto para `index.symbols`.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use kinein_protocol::{IndexState, JsonRpcRequest};
use serde_json::json;

use super::core_with_empty_search_path;
use crate::index;
use crate::lang::extract::SymbolExtractor;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-index-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Um projeto pequeno com as quatro linguagens, um `build/` que nao conta, e
/// um arquivo qualquer que so' se conta.
fn projeto(name: &str) -> PathBuf {
    let raiz = temp_dir(name);
    std::fs::create_dir_all(raiz.join("src/net")).unwrap();
    std::fs::create_dir_all(raiz.join("build")).unwrap();
    std::fs::create_dir_all(raiz.join("tools")).unwrap();
    std::fs::write(
        raiz.join("src/main.rs"),
        "struct Motor;\nimpl Motor {\n    fn ligar(&self) {}\n}\nfn main() { }\nfn ligar_tudo() {}\nfn ler_config() {}\nfn reler() {}\n",
    )
    .unwrap();
    std::fs::write(
        raiz.join("src/net/sensor.cpp"),
        "class Sensor {\npublic:\n    int ler();\n    void calibrar() {}\n};\nint Sensor::ler() { return 1; }\nstatic int contador = 0;\n",
    )
    .unwrap();
    std::fs::write(
        raiz.join("src/util.c"),
        "int soma(int a, int b) { return a + b; }\n",
    )
    .unwrap();
    std::fs::write(raiz.join("tools/gera.py"), "def gera():\n    pass\n").unwrap();
    // Um stub .pyi: e' Python para a gramatica tanto quanto o .py.
    std::fs::write(raiz.join("tools/gera.pyi"), "def gera() -> None: ...\n").unwrap();
    std::fs::write(raiz.join("README.md"), "# projeto\n").unwrap();
    // O build/ tem fonte com muitas funcoes: se contasse, os numeros mentiriam.
    std::fs::write(
        raiz.join("build/gerado.c"),
        "int a() {}\nint b() {}\nint c() {}\n",
    )
    .unwrap();
    raiz
}

fn build(raiz: &Path) -> index::ProjectIndex {
    let parado = AtomicBool::new(false);
    index::build(raiz, &parado, &|_, _| {})
}

#[test]
fn the_whole_tree_is_seen_source_is_read_and_output_dirs_are_not() {
    let raiz = projeto("inteiro");
    let indice = build(&raiz);
    let stats = indice.stats();
    assert_eq!(stats.state, IndexState::Ready);
    // README conta; build/gerado.c NAO; as 5 fontes sao lidas.
    assert_eq!(
        stats.files,
        6,
        "{:?}",
        indice.files.keys().collect::<Vec<_>>()
    );
    assert_eq!(stats.source_files, 5);
    assert_eq!(stats.folders, 4, "raiz, src, src/net, tools — build/ fora");
    assert!(!indice.files.contains_key("build/gerado.c"));
    let por = |l: &str| {
        stats
            .by_language
            .iter()
            .find(|x| x.language == l)
            .cloned()
            .unwrap()
    };
    assert_eq!((por("rust").files, por("rust").lines), (1, 8));
    assert_eq!(por("cpp").files, 1);
    assert_eq!(por("python").files, 2, ".py e .pyi");
    assert_eq!(
        por("python").symbols,
        2,
        "Python: a gramatica entrou em 2026-09-12 — `gera` e' declaracao nos dois"
    );
    assert_eq!(por("other").files, 1);
    assert!(stats.lines >= 8 + 7 + 1 + 2);
    assert!(stats.skipped.is_empty());
    assert!(stats.elapsed_ms < 5_000);
}

#[test]
fn declarations_come_with_line_kind_and_container_and_no_duplicates() {
    let raiz = projeto("declaracoes");
    let indice = build(&raiz);
    let (todos, _) = indice.query("", None, 100);
    let nomes: Vec<(&str, &str, u64)> = todos
        .iter()
        .map(|s| (s.name.as_str(), s.kind.as_str(), s.line))
        .collect();
    // Rust: a struct (a query tags oficial a chama de `class` — medido em
    // 2026-09-12; o indice nao renomeia o que a gramatica diz), o metodo (UMA
    // vez, como method), main e ligar_tudo.
    assert!(nomes.contains(&("Motor", "class", 1)), "{nomes:?}");
    let ligar: Vec<_> = todos
        .iter()
        .filter(|s| s.name == "ligar" && s.path.ends_with("main.rs"))
        .collect();
    assert_eq!(
        ligar.len(),
        1,
        "o fn dentro de impl nao aparece em dobro: {nomes:?}"
    );
    assert_eq!(ligar[0].kind, "method");
    assert!(
        nomes
            .iter()
            .any(|(n, k, _)| *n == "main" && *k == "function")
    );
    // C++: a classe e os metodos com o container.
    let calibrar = todos.iter().find(|s| s.name == "calibrar").unwrap();
    assert_eq!(calibrar.container.as_deref(), Some("Sensor"));
    assert_eq!(calibrar.language, "cpp");
    assert!(
        todos
            .iter()
            .any(|s| s.name == "Sensor" && s.kind == "class")
    );
    // C: a funcao.
    assert!(
        todos
            .iter()
            .any(|s| s.name == "soma" && s.language == "c" && s.kind == "function")
    );
    // Python: a funcao, pela gramatica oficial.
    assert!(
        todos
            .iter()
            .any(|s| s.name == "gera" && s.language == "python" && s.kind == "function"),
        "{nomes:?}"
    );
    let stats = indice.stats();
    assert!(stats.functions >= 5 && stats.types >= 2, "{stats:?}");
}

#[test]
fn query_ranks_exact_then_prefix_then_substring_and_filters_by_kind() {
    let raiz = projeto("busca");
    let indice = build(&raiz);
    let (achados, total) = indice.query("ligar", None, 10);
    assert_eq!(total, 2);
    assert_eq!(achados[0].name, "ligar", "exato antes de prefixo");
    assert_eq!(achados[1].name, "ligar_tudo");
    // exato ("ler" do C++) > prefixo ("ler_config") > substring ("reler"),
    // sem diferenciar caixa.
    // ("ler" vem duas vezes de proposito: a declaracao na classe e a definicao
    // `Sensor::ler` fora dela sao duas linhas, e o indice mostra as duas.)
    let (achados, total) = indice.query("LER", None, 10);
    let nomes: Vec<&str> = achados.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(total, 4, "{nomes:?}");
    assert_eq!(nomes, vec!["ler", "ler", "ler_config", "reler"]);
    let (achados, total) = indice.query("a", Some("class"), 10);
    assert_eq!(
        (achados.len(), total),
        (0, 0),
        "'Sensor' nao contem 'a'; o filtro por kind vale"
    );
    let (achados, _) = indice.query("sensor", Some("class"), 10);
    assert_eq!(achados[0].kind, "class");
    let (limitados, total) = indice.query("", None, 2);
    assert_eq!(limitados.len(), 2);
    assert!(total > 2, "o total e' antes do limite");
}

#[test]
fn incremental_reindex_follows_edits_and_deletions() {
    let raiz = projeto("incremento");
    let mut indice = build(&raiz);
    let mut extractor = SymbolExtractor::default();
    // Editar: uma funcao nova aparece.
    std::fs::write(
        raiz.join("src/util.c"),
        "int soma(int a, int b) { return a + b; }\nint dobro(int x) { return 2 * x; }\n",
    )
    .unwrap();
    let mudou = indice.reindex_paths(&[raiz.join("src/util.c")], &mut extractor);
    assert_eq!(mudou, 1);
    assert_eq!(indice.query("dobro", None, 5).1, 1);
    // Apagar: some do indice, e o total cai.
    let antes = indice.stats().files;
    std::fs::remove_file(raiz.join("src/util.c")).unwrap();
    indice.reindex_paths(&[raiz.join("src/util.c")], &mut extractor);
    assert_eq!(indice.stats().files, antes - 1);
    assert_eq!(indice.query("soma", None, 5).1, 0);
    // Criar: entra.
    std::fs::write(raiz.join("src/novo.rs"), "pub fn novo() {}\n").unwrap();
    indice.reindex_paths(&[raiz.join("src/novo.rs")], &mut extractor);
    assert_eq!(indice.query("novo", None, 5).1, 1);
    // Fora da raiz: ignorado, sem panico.
    assert_eq!(
        indice.reindex_paths(&[PathBuf::from("/tmp/fora.rs")], &mut extractor),
        0
    );
}

#[test]
fn oversized_and_unreadable_files_are_counted_and_named_in_skipped() {
    let raiz = temp_dir("pulados");
    let grande = vec![b'/'; usize::try_from(index::ARQUIVO_MAXIMO + 1).unwrap()];
    std::fs::write(raiz.join("gigante.c"), grande).unwrap();
    std::fs::write(raiz.join("binario.rs"), [0xff, 0xfe, 0x00, 0x80]).unwrap();
    let indice = build(&raiz);
    let stats = indice.stats();
    assert_eq!(stats.files, 2, "os dois contam");
    assert_eq!(stats.symbols, 0);
    assert_eq!(stats.skipped.len(), 2, "{:?}", stats.skipped);
    assert!(
        stats
            .skipped
            .iter()
            .any(|s| s.starts_with("gigante.c") && s.contains("teto"))
    );
    assert!(
        stats
            .skipped
            .iter()
            .any(|s| s.starts_with("binario.rs") && s.contains("UTF-8"))
    );
}

#[test]
fn cancelling_leaves_the_index_failed_with_the_reason() {
    let raiz = projeto("cancelado");
    let parado = AtomicBool::new(true);
    let indice = index::build(&raiz, &parado, &|_, _| {});
    assert_eq!(indice.state, IndexState::Failed);
    assert_eq!(indice.error.as_deref(), Some("indexacao cancelada"));
}

/// Pelo despacho real: sem jobs (como nos testes) o indice e' construido no
/// proprio `workspace.open`, e `index.symbols` responde `ready`.
#[test]
fn opening_a_workspace_builds_the_index_and_symbols_answer() {
    let raiz = projeto("despacho").canonicalize().unwrap();
    let mut core = core_with_empty_search_path("index");
    let vazio = core.handle_request(&JsonRpcRequest::new(1_i64, "index.status", Some(json!({}))));
    assert_eq!(vazio.response().result.as_ref().unwrap()["state"], "idle");
    let opened = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "workspace.open",
        Some(json!({ "path": raiz.to_str().unwrap() })),
    ));
    assert!(opened.response().error.is_none());
    let resposta =
        core.handle_request(&JsonRpcRequest::new(3_i64, "index.status", Some(json!({}))));
    let totais = resposta.response().result.clone().unwrap();
    assert_eq!(totais["state"], "ready");
    assert_eq!(totais["sourceFiles"], 5);
    let busca = core.handle_request(&JsonRpcRequest::new(
        4_i64,
        "index.symbols",
        Some(json!({ "query": "calibrar" })),
    ));
    let resultado = busca.response().result.clone().unwrap();
    assert_eq!(resultado["total"], 1);
    assert_eq!(resultado["symbols"][0]["container"], "Sensor");
    assert_eq!(resultado["state"], "ready");
}
