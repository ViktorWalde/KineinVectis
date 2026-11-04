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
//!                   sao relidos (ou removidos) e os totais recalculados; uma
//!                   PASTA nova e' caminhada inteira, uma pasta apagada leva
//!                   os arquivos dela junto
//! folder_paths      as pastas caminhadas, para o Core REGISTRA-LAS no watcher
//!                   (uma a uma, nao recursivo — ADR-0001): e' assim que o
//!                   indice segue o disco inteiro, e nao so' as pastas que a
//!                   UI abriu
//! query()           nome exato > prefixo > substring, sem diferenciar caixa
//! ```
//!
//! O que NAO entra: conteudo de arquivo em memoria (so' as declaracoes e os
//! numeros), pastas de saida, arquivos acima de [`MAX_FILE_BYTES`].
//!
//! O CONTEXTO DE COMPILADOR (com que cada arquivo e' compilado) mora em
//! [`context`] e e' carregado junto do indice — o mapa diz o que ha'; o
//! contexto diz como se compila.

pub mod context;
mod file;
use file::index_file;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use kinein_protocol::{IndexState, IndexStats, IndexSymbol, LanguageStats};

use crate::lang::extract::SymbolExtractor;

use self::context::CompileContext;

/// Arquivo maior que isto e' contado, mas nao parseado (o mesmo teto do
/// editor: 4 MiB).
pub const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// Compatibility name for the existing structural-index byte limit.
pub const ARQUIVO_MAXIMO: u64 = MAX_FILE_BYTES;

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
    /// Pastas caminhadas (absolutas, a raiz inclusa), na ordem em que entraram.
    pub folder_paths: Vec<PathBuf>,
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
            folder_paths: Vec::new(),
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
            folders: self.folder_paths.len() as u64,
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

    /// Replaces context and reparses files whose local grammar changed.
    pub(crate) fn set_context(&mut self, context: CompileContext) {
        let paths: Vec<PathBuf> = self
            .files
            .values()
            .filter_map(|file| {
                let path = self.root.join(&file.path);
                (context.document_language(&path) != file.language).then_some(path)
            })
            .collect();
        self.context = Some(context);
        self.reindex_paths(&paths, &mut SymbolExtractor::default());
    }

    /// Reindexa os caminhos que mudaram: arquivo relido se existe e e' fonte;
    /// pasta nova caminhada inteira (ela entra em `folder_paths`, e o Core
    /// registra no watcher tudo que esta' la' a cada `event.index.finished`);
    /// o que sumiu sai — e uma pasta que sumiu leva os arquivos e as subpastas
    /// dela junto. Devolve quantos arquivos/pastas entraram ou sairam.
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
            let chave = crate::platform::portable_relative(relativo);
            if caminho.is_file() {
                if let Some(arquivo) = index_file(
                    &self.root,
                    caminho,
                    extractor,
                    &mut self.skipped,
                    self.context.as_ref(),
                ) {
                    self.files.insert(chave, arquivo);
                    mudou += 1;
                }
            } else if caminho.is_dir() {
                if pasta_ignorada(&self.root, caminho) || self.folder_paths.contains(caminho) {
                    continue;
                }
                let root = self.root.clone();
                let antes = self.folder_paths.len();
                let parado = AtomicBool::new(false);
                mudou += caminhar(&root, caminho, &parado, extractor, self, &|_, _| {});
                mudou += self.folder_paths.len() - antes;
            } else {
                if self.files.remove(&chave).is_some() {
                    mudou += 1;
                }
                let prefixo = format!("{chave}/");
                let embaixo: Vec<String> = self
                    .files
                    .keys()
                    .filter(|k| k.starts_with(&prefixo))
                    .cloned()
                    .collect();
                for k in embaixo {
                    self.files.remove(&k);
                    mudou += 1;
                }
                let pastas_antes = self.folder_paths.len();
                self.folder_paths.retain(|p| !p.starts_with(caminho));
                mudou += pastas_antes - self.folder_paths.len();
            }
        }
        mudou
    }
}

/// Uma pasta que o indice (e o watcher) nao seguem: qualquer componente na
/// lista de pastas ignoradas.
fn pasta_ignorada(root: &Path, caminho: &Path) -> bool {
    caminho
        .strip_prefix(root)
        .unwrap_or(caminho)
        .components()
        .any(|c| {
            c.as_os_str()
                .to_str()
                .is_some_and(|nome| crate::fswatch::SKIP_DIRS.contains(&nome))
        })
}

/// Constroi o indice de `root`. `progress` recebe (arquivos, simbolos) de
/// tempos em tempos; `cancel` interrompe entre arquivos e deixa o indice em
/// `Failed` com o motivo.
#[must_use]
pub fn build(root: &Path, cancel: &AtomicBool, progress: &dyn Fn(u64, u64)) -> ProjectIndex {
    build_with_context(root, cancel, progress, None)
}

pub(crate) fn build_with_context(
    root: &Path,
    cancel: &AtomicBool,
    progress: &dyn Fn(u64, u64),
    context: Option<CompileContext>,
) -> ProjectIndex {
    let inicio = Instant::now();
    let mut index = ProjectIndex::empty(root);
    index.context = context;
    index.state = IndexState::Building;
    let mut extractor = SymbolExtractor::default();
    caminhar(root, root, cancel, &mut extractor, &mut index, progress);
    if cancel.load(Ordering::SeqCst) {
        index.state = IndexState::Failed;
        index.error = Some("indexação cancelada".to_owned());
    } else {
        index.state = IndexState::Ready;
    }
    index.elapsed_ms = u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
    let simbolos: u64 = index.files.values().map(|a| a.symbols.len() as u64).sum();
    progress(index.files.len() as u64, simbolos);
    index
}

/// Caminha a subarvore de `inicio` (dentro de `root`) para dentro de
/// `indice`: cada pasta entra em `folder_paths`, cada arquivo e' lido. E' o
/// mesmo caminho do build inteiro e do incremento por pasta nova. Devolve
/// quantos arquivos entraram.
fn caminhar(
    root: &Path,
    inicio: &Path,
    cancel: &AtomicBool,
    extractor: &mut SymbolExtractor,
    indice: &mut ProjectIndex,
    progress: &dyn Fn(u64, u64),
) -> usize {
    let mut pilha = vec![inicio.to_path_buf()];
    let mut lidos = 0;
    let mut simbolos: u64 = indice.files.values().map(|a| a.symbols.len() as u64).sum();
    while let Some(dir) = pilha.pop() {
        if cancel.load(Ordering::SeqCst) {
            return lidos;
        }
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            indice
                .skipped
                .push(relative_path(root, &dir) + " (pasta ilegível)");
            continue;
        };
        indice.folder_paths.push(dir.clone());
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
            if let Some(arquivo) = index_file(
                root,
                &caminho,
                extractor,
                &mut indice.skipped,
                indice.context.as_ref(),
            ) {
                simbolos += arquivo.symbols.len() as u64;
                indice.files.insert(arquivo.path.clone(), arquivo);
                lidos += 1;
                if indice.files.len().is_multiple_of(200) {
                    progress(indice.files.len() as u64, simbolos);
                }
            }
        }
    }
    lidos
}

fn relative_path(root: &Path, caminho: &Path) -> String {
    caminho.strip_prefix(root).map_or_else(
        |_| caminho.to_string_lossy().into_owned(),
        crate::platform::portable_relative,
    )
}

/// `c`/`cpp`/`rust`/`python` pela gramatica do editor (o Python entrou em
/// 2026-09-12, bloco B do `roadmaps/41`); o resto e' `other` — contado, nao
/// lido.
pub(crate) fn language_of(caminho: &Path) -> &'static str {
    SymbolExtractor::language_for(caminho).unwrap_or("other")
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
