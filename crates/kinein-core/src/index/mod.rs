//! O indice proprio do projeto INTEIRO: pastas, arquivos, funcoes e tipos.
//!
//! Decisao do autor em 2026-09-12 (`roadmaps/42` P0): a IDE le todo o projeto
//! que abre — todas as pastas, todos os arquivos, todas as funcoes/tipos —
//! para C, C++, Rust e Python, sem esperar language server. O clangd e o
//! rust-analyzer continuam donos da semantica profunda (tipos, referencias,
//! rename); este indice e' o MAPA ESTRUTURAL que existe desde o primeiro
//! segundo, construido com as mesmas gramaticas Tree-sitter do editor.
//!
//! # Forma
//!
//! ```text
//! build()           caminha a arvore (a lista de pastas ignoradas e' a MESMA do
//!                   watcher, para os dois verem o mesmo projeto), classifica
//!                   por linguagem, le e parseia os arquivos-fonte, extrai as
//!                   declaracoes; reporta progresso; respeita cancelamento
//! reindex_paths()   o incremento: os caminhos que o `event.fs.changed` trouxe
//!                   sao relidos (ou removidos) e os totais recalculados
//! query()           nome exato > prefixo > substring, sem diferenciar caixa
//! ```
//!
//! O que NAO entra: conteudo de arquivo em memoria (so' as declaracoes e os
//! numeros), pastas de saida, arquivos acima de [`ARQUIVO_MAXIMO`].
//!
//! O CONTEXTO DE COMPILADOR (com que cada arquivo e' compilado) mora em
//! [`context`] e e' carregado junto do indice — o mapa diz o que ha'; o
//! contexto diz como se compila.

pub mod context;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use kinein_protocol::{IndexState, IndexStats, IndexSymbol, LanguageStats, SyntaxOutlineItem};

use crate::lang::extract::SymbolExtractor;

use self::context::CompileContext;

/// Arquivo maior que isto e' contado, mas nao parseado (o mesmo teto do
/// editor: 4 MiB).
pub const ARQUIVO_MAXIMO: u64 = 4 * 1024 * 1024;

/// Extensoes de Python: contadas e medidas ja'; declaracoes quando a gramatica
/// entrar (bloco B do `roadmaps/41`).
const PYTHON: &[&str] = &["py", "pyi"];

/// Um arquivo indexado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedFile {
    /// Caminho relativo a raiz.
    pub path: String,
    /// `c`, `cpp`, `rust`, `python`, `other`.
    pub language: String,
    /// Linhas (contadas para fonte; 0 para `other`).
    pub lines: u64,
    /// Bytes no disco.
    pub bytes: u64,
    /// Declaracoes achatadas, com `container` para as aninhadas.
    pub symbols: Vec<IndexSymbol>,
}

/// O indice inteiro: os arquivos e os totais.
#[derive(Debug, Clone)]
pub struct ProjectIndex {
    /// Raiz indexada.
    pub root: PathBuf,
    /// Por caminho relativo, em ordem.
    pub files: BTreeMap<String, IndexedFile>,
    /// Pastas vistas.
    pub folders: u64,
    /// Arquivos que nao deu para ler/parsear.
    pub skipped: Vec<String>,
    /// Estado e ultimo tempo de build.
    pub state: IndexState,
    /// Motivo da falha, quando `Failed`.
    pub error: Option<String>,
    /// Duracao do ultimo build completo.
    pub elapsed_ms: u64,
    /// O contexto de compilador, quando ja' carregado.
    pub context: Option<CompileContext>,
}

impl Default for ProjectIndex {
    fn default() -> Self {
        Self::empty(Path::new(""))
    }
}

impl ProjectIndex {
    /// Um indice vazio para `root`, em `Idle`.
    #[must_use]
    pub fn empty(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            files: BTreeMap::new(),
            folders: 0,
            skipped: Vec::new(),
            state: IndexState::Idle,
            error: None,
            elapsed_ms: 0,
            context: None,
        }
    }

    /// Os totais, como o protocolo os carrega.
    #[must_use]
    pub fn stats(&self) -> IndexStats {
        let mut por_linguagem: BTreeMap<&str, LanguageStats> = BTreeMap::new();
        let (mut source_files, mut lines, mut bytes, mut symbols, mut functions, mut types) =
            (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        for arquivo in self.files.values() {
            let entrada = por_linguagem
                .entry(arquivo.language.as_str())
                .or_insert_with(|| LanguageStats {
                    language: arquivo.language.clone(),
                    ..LanguageStats::default()
                });
            entrada.files += 1;
            entrada.lines += arquivo.lines;
            entrada.symbols += arquivo.symbols.len() as u64;
            if arquivo.language != "other" {
                source_files += 1;
                lines += arquivo.lines;
                bytes += arquivo.bytes;
            }
            symbols += arquivo.symbols.len() as u64;
            for s in &arquivo.symbols {
                if e_funcao(&s.kind) {
                    functions += 1;
                } else if e_tipo(&s.kind) {
                    types += 1;
                }
            }
        }
        let mut by_language: Vec<LanguageStats> = por_linguagem.into_values().collect();
        by_language.sort_by(|a, b| {
            b.files
                .cmp(&a.files)
                .then_with(|| a.language.cmp(&b.language))
        });
        IndexStats {
            state: self.state,
            folders: self.folders,
            files: self.files.len() as u64,
            source_files,
            lines,
            bytes,
            symbols,
            functions,
            types,
            by_language,
            skipped: self.skipped.clone(),
            elapsed_ms: self.elapsed_ms,
            error: self.error.clone(),
            context: self.context.as_ref().map(CompileContext::summary),
        }
    }

    /// Busca por nome: exato antes de prefixo antes de substring; empate por
    /// caminho e linha, para a ordem ser estavel.
    #[must_use]
    pub fn query(
        &self,
        needle: &str,
        kind: Option<&str>,
        limit: usize,
    ) -> (Vec<IndexSymbol>, usize) {
        let agulha = needle.trim().to_lowercase();
        let mut achados: Vec<(u8, &IndexSymbol)> = Vec::new();
        for arquivo in self.files.values() {
            for simbolo in &arquivo.symbols {
                if kind.is_some_and(|k| simbolo.kind != k) {
                    continue;
                }
                let nome = simbolo.name.to_lowercase();
                let peso = if agulha.is_empty() || nome == agulha {
                    0
                } else if nome.starts_with(&agulha) {
                    1
                } else if nome.contains(&agulha) {
                    2
                } else {
                    continue;
                };
                achados.push((peso, simbolo));
            }
        }
        achados.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.name.len().cmp(&b.1.name.len()))
                .then_with(|| a.1.path.cmp(&b.1.path))
                .then_with(|| a.1.line.cmp(&b.1.line))
        });
        let total = achados.len();
        (
            achados
                .into_iter()
                .take(limit)
                .map(|(_, s)| s.clone())
                .collect(),
            total,
        )
    }

    /// Reindexa os caminhos que mudaram: relidos se existem e sao fonte,
    /// removidos se sumiram. Devolve quantos entraram/sairam.
    pub(crate) fn reindex_paths(
        &mut self,
        paths: &[PathBuf],
        extractor: &mut SymbolExtractor,
    ) -> usize {
        let mut mudou = 0;
        for caminho in paths {
            let Ok(relativo) = caminho.strip_prefix(&self.root) else {
                continue;
            };
            let chave = relativo.to_string_lossy().into_owned();
            if caminho.is_file() {
                if let Some(arquivo) =
                    indexar_arquivo(&self.root, caminho, extractor, &mut self.skipped)
                {
                    self.files.insert(chave, arquivo);
                    mudou += 1;
                }
            } else if self.files.remove(&chave).is_some() {
                mudou += 1;
            }
        }
        mudou
    }
}

/// Constroi o indice de `root`. `progress` recebe (arquivos, simbolos) de
/// tempos em tempos; `cancel` interrompe entre arquivos e deixa o indice em
/// `Failed` com o motivo.
#[must_use]
pub fn build(root: &Path, cancel: &AtomicBool, progress: &dyn Fn(u64, u64)) -> ProjectIndex {
    let inicio = Instant::now();
    let mut indice = ProjectIndex::empty(root);
    indice.state = IndexState::Building;
    let mut extractor = SymbolExtractor::default();
    let mut pilha = vec![root.to_path_buf()];
    let mut simbolos: u64 = 0;
    while let Some(dir) = pilha.pop() {
        if cancel.load(Ordering::SeqCst) {
            indice.state = IndexState::Failed;
            indice.error = Some("indexacao cancelada".to_owned());
            indice.elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
            return indice;
        }
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            indice
                .skipped
                .push(relativo(root, &dir) + " (pasta ilegivel)");
            continue;
        };
        indice.folders += 1;
        let mut arquivos: Vec<PathBuf> = Vec::new();
        for entrada in entradas.filter_map(Result::ok) {
            let caminho = entrada.path();
            let nome = entrada.file_name().to_string_lossy().into_owned();
            let Ok(tipo) = entrada.file_type() else {
                continue;
            };
            if tipo.is_dir() {
                if !crate::fswatch::SKIP_DIRS.contains(&nome.as_str()) {
                    pilha.push(caminho);
                }
            } else if tipo.is_file() {
                arquivos.push(caminho);
            }
            // Links simbolicos ficam de fora: um link para fora do projeto
            // (ou um ciclo) nao e' o projeto.
        }
        arquivos.sort();
        for caminho in arquivos {
            if let Some(arquivo) =
                indexar_arquivo(root, &caminho, &mut extractor, &mut indice.skipped)
            {
                simbolos += arquivo.symbols.len() as u64;
                indice.files.insert(arquivo.path.clone(), arquivo);
                if indice.files.len() % 200 == 0 {
                    progress(indice.files.len() as u64, simbolos);
                }
            }
        }
    }
    indice.state = IndexState::Ready;
    indice.elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    progress(indice.files.len() as u64, simbolos);
    indice
}

fn relativo(root: &Path, caminho: &Path) -> String {
    caminho
        .strip_prefix(root)
        .unwrap_or(caminho)
        .to_string_lossy()
        .into_owned()
}

/// Classifica, le e parseia UM arquivo. Todo arquivo entra (contado); so' os
/// de fonte sao lidos; so' os com gramatica ganham simbolos.
fn indexar_arquivo(
    root: &Path,
    caminho: &Path,
    extractor: &mut SymbolExtractor,
    skipped: &mut Vec<String>,
) -> Option<IndexedFile> {
    let path = relativo(root, caminho);
    let meta = std::fs::metadata(caminho).ok()?;
    let language = linguagem_de(caminho);
    if language == "other" {
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    }
    if meta.len() > ARQUIVO_MAXIMO {
        skipped.push(format!("{path} ({} bytes: acima do teto)", meta.len()));
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    }
    let Ok(conteudo) = std::fs::read_to_string(caminho) else {
        skipped.push(format!("{path} (nao e' UTF-8 legivel)"));
        return Some(IndexedFile {
            path,
            language: language.to_owned(),
            lines: 0,
            bytes: meta.len(),
            symbols: Vec::new(),
        });
    };
    let lines = conteudo.lines().count() as u64;
    let mut symbols = Vec::new();
    if let Some(outline) = extractor.symbols(caminho, &conteudo) {
        achatar(&outline, None, &path, language, &mut symbols);
        deduplicar(&mut symbols);
    }
    Some(IndexedFile {
        path,
        language: language.to_owned(),
        lines,
        bytes: meta.len(),
        symbols,
    })
}

/// `c`/`cpp`/`rust` pela gramatica do editor; `python` pela extensao; o resto
/// e' `other` — contado, nao lido.
pub(crate) fn linguagem_de(caminho: &Path) -> &'static str {
    if let Some(l) = SymbolExtractor::language_for(caminho) {
        return l;
    }
    let ext = caminho
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if PYTHON.contains(&ext.as_str()) {
        "python"
    } else {
        "other"
    }
}

/// O outline aninhado do editor vira lista plana com `container`.
fn achatar(
    itens: &[SyntaxOutlineItem],
    container: Option<&str>,
    path: &str,
    language: &str,
    saida: &mut Vec<IndexSymbol>,
) {
    for item in itens {
        saida.push(IndexSymbol {
            name: item.name.clone(),
            kind: item.kind.clone(),
            path: path.to_owned(),
            language: language.to_owned(),
            line: item.line,
            end_line: item.end_line,
            container: container.map(str::to_owned),
        });
        achatar(&item.children, Some(&item.name), path, language, saida);
    }
}

/// A query `tags` do Rust captura um `fn` dentro de `impl` DUAS vezes —
/// `definition.function` e `definition.method` (medido em 2026-09-12 no
/// proprio repositorio: `handle_request` aparecia em dobro). Uma declaracao
/// e' uma: fica a mais especifica (method) por (linha, nome).
fn deduplicar(symbols: &mut Vec<IndexSymbol>) {
    let mut vistos: BTreeMap<(u64, String), usize> = BTreeMap::new();
    let mut manter: Vec<bool> = vec![true; symbols.len()];
    for (i, s) in symbols.iter().enumerate() {
        let chave = (s.line, s.name.clone());
        match vistos.get(&chave) {
            None => {
                vistos.insert(chave, i);
            }
            Some(&anterior) => {
                if especificidade(&s.kind) > especificidade(&symbols[anterior].kind) {
                    manter[anterior] = false;
                    vistos.insert(chave, i);
                } else {
                    manter[i] = false;
                }
            }
        }
    }
    let mut i = 0;
    symbols.retain(|_| {
        let fica = manter[i];
        i += 1;
        fica
    });
}

fn especificidade(kind: &str) -> u8 {
    match kind {
        "method" | "constructor" => 2,
        "function" => 1,
        _ => 0,
    }
}

fn e_funcao(kind: &str) -> bool {
    matches!(kind, "function" | "method" | "constructor" | "macro")
}

fn e_tipo(kind: &str) -> bool {
    matches!(
        kind,
        "class" | "struct" | "enum" | "union" | "interface" | "type" | "trait" | "typedef"
    )
}
