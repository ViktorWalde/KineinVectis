//! Extracao de declaracoes de um ARQUIVO inteiro, para o indice do projeto.
//!
//! O `SyntaxTreeService` serve o EDITOR: incremental, com cache LRU, por
//! buffer aberto. O indice (`index/`) precisa do contrario — parsear centenas
//! de arquivos uma vez, sem guardar arvore nenhuma — e por isso este extrator
//! nao tem cache: compila as queries uma vez (o registro) e devolve so' a
//! lista aninhada de declaracoes que a query `tags` oficial da gramatica
//! produz. E' a mesma `outline()` do editor; o que muda e' quem chama e quanto
//! fica na memoria.

use std::path::Path;

use kinein_protocol::SyntaxOutlineItem;
use tree_sitter::Parser;

use super::outline::outline;
use super::registry::{LanguageId, LanguageRegistry};

/// Compila as gramaticas sob demanda e extrai declaracoes de arquivos.
#[derive(Debug, Default)]
pub struct SymbolExtractor {
    registry: LanguageRegistry,
}

impl SymbolExtractor {
    /// A linguagem que o indice reconhece para `path`, pelo nome do arquivo.
    #[must_use]
    pub fn language_for(path: &Path) -> Option<&'static str> {
        LanguageId::for_path(path).map(LanguageId::as_str)
    }

    /// As declaracoes de `content`, ou `None` se a linguagem nao tem gramatica
    /// aqui (Python ainda nao tem: conta-se o arquivo, nao os simbolos) ou se a
    /// gramatica falhou. Um arquivo com erro de sintaxe AINDA devolve o que a
    /// arvore parcial permitiu — e' assim que o Tree-sitter foi desenhado.
    pub fn symbols(&mut self, path: &Path, content: &str) -> Option<Vec<SyntaxOutlineItem>> {
        let id = LanguageId::for_path(path)?;
        let runtime = self.registry.runtime(id).ok()?;
        let mut parser = Parser::new();
        parser.set_language(runtime.language()).ok()?;
        let tree = parser.parse(content, None)?;
        Some(outline(&tree, runtime.tags(), content))
    }
}
