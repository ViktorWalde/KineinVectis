//! Registry of the initial C, C++ and Rust grammars and queries.

use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fmt,
    path::Path,
};

use tree_sitter::{Language, Query};

const C_LOCALS_QUERY: &str = r"
(compound_statement) @local.scope
(parameter_declaration declarator: (identifier) @local.definition)
(init_declarator declarator: (identifier) @local.definition)
(identifier) @local.reference
";

const CPP_LOCALS_QUERY: &str = r"
(compound_statement) @local.scope
(parameter_declaration declarator: (identifier) @local.definition)
(init_declarator declarator: (identifier) @local.definition)
(identifier) @local.reference
(field_identifier) @local.reference
";

const RUST_LOCALS_QUERY: &str = r"
(block) @local.scope
(let_declaration pattern: (identifier) @local.definition)
(identifier) @local.reference
";

/// Languages supported by the first syntax-tree foundation.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub(super) enum LanguageId {
    C,
    Cpp,
    Rust,
}

impl LanguageId {
    /// Detects the language from a workspace file name.
    pub(super) fn for_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "c" => Some(Self::C),
            "h" | "hh" | "hpp" | "hxx" | "ipp" | "cc" | "cpp" | "cxx" => Some(Self::Cpp),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }

    /// Stable protocol identifier.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Rust => "rust",
        }
    }
}

/// Failure while compiling an upstream grammar query.
#[derive(Debug)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct RegistryError {
    language: LanguageId,
    query: &'static str,
    source: tree_sitter::QueryError,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "query {} de {} invalida: {}",
            self.query,
            self.language.as_str(),
            self.source
        )
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Compiled runtime data reused by every document of one language.
pub(super) struct LanguageRuntime {
    language: Language,
    highlights: Query,
    tags: Query,
    locals: Query,
}

impl fmt::Debug for LanguageRuntime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LanguageRuntime")
            .finish_non_exhaustive()
    }
}

impl LanguageRuntime {
    pub(super) const fn language(&self) -> &Language {
        &self.language
    }

    pub(super) const fn highlights(&self) -> &Query {
        &self.highlights
    }

    pub(super) const fn tags(&self) -> &Query {
        &self.tags
    }

    pub(super) const fn locals(&self) -> &Query {
        &self.locals
    }
}

/// Lazy registry so startup does not compile queries before an editor opens.
#[derive(Debug, Default)]
pub(super) struct LanguageRegistry {
    runtimes: HashMap<LanguageId, LanguageRuntime>,
}

impl LanguageRegistry {
    pub(super) fn runtime(&mut self, id: LanguageId) -> Result<&LanguageRuntime, RegistryError> {
        match self.runtimes.entry(id) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => Ok(entry.insert(compile_runtime(id)?)),
        }
    }
}

fn compile_runtime(id: LanguageId) -> Result<LanguageRuntime, RegistryError> {
    let (language, highlights, tags, locals): (Language, &str, &str, &str) = match id {
        LanguageId::C => (
            tree_sitter_c::LANGUAGE.into(),
            tree_sitter_c::HIGHLIGHT_QUERY,
            tree_sitter_c::TAGS_QUERY,
            C_LOCALS_QUERY,
        ),
        LanguageId::Cpp => (
            tree_sitter_cpp::LANGUAGE.into(),
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            tree_sitter_cpp::TAGS_QUERY,
            CPP_LOCALS_QUERY,
        ),
        LanguageId::Rust => (
            tree_sitter_rust::LANGUAGE.into(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::TAGS_QUERY,
            RUST_LOCALS_QUERY,
        ),
    };
    Ok(LanguageRuntime {
        highlights: compile_query(id, "highlights", &language, highlights)?,
        tags: compile_query(id, "tags", &language, tags)?,
        locals: compile_query(id, "locals", &language, locals)?,
        language,
    })
}

fn compile_query(
    language_id: LanguageId,
    name: &'static str,
    language: &Language,
    source: &str,
) -> Result<Query, RegistryError> {
    Query::new(language, source).map_err(|source| RegistryError {
        language: language_id,
        query: name,
        source,
    })
}
