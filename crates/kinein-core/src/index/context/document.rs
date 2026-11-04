//! Identidade local C/C++ a partir do contexto existente, sem inventar flags.

use std::path::Path;

use super::{CompileContext, cdb::CdbUnit};
use crate::lang::extract::SymbolExtractor;

impl CompileContext {
    pub(super) fn load_document_languages(&mut self) {
        self.document_languages = self
            .units
            .iter()
            .filter_map(|(path, unit)| {
                (!unit.ambiguous)
                    .then(|| command_language(path, unit))
                    .flatten()
                    .map(|language| (path.clone(), language))
            })
            .collect();
        self.document_family = unique(
            self.units
                .keys()
                .map(|path| self.document_languages.get(path).copied()),
        );
    }

    /// Escolha de gramatica/identidade; nao e uma unidade inferida de header.
    pub(crate) fn document_language(&self, path: &Path) -> &'static str {
        self.contextual_language(path)
            .or_else(|| SymbolExtractor::language_for(path))
            .unwrap_or("other")
    }

    fn contextual_language(&self, path: &Path) -> Option<&'static str> {
        let path = crate::platform::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if let Some(language) = self.document_languages.get(&path) {
            return Some(language);
        }
        // Uma entrada duplicada nao determina uma unica identidade.
        if self.units.contains_key(&path) || !ambiguous_header(&path) {
            return None;
        }
        if let Some(model) = &self.cmake {
            let owners = model.targets_for(&path);
            if !owners.is_empty() {
                return unique(
                    owners
                        .iter()
                        .flat_map(|target| target.compile_groups.iter())
                        .map(|group| match group.language.as_str() {
                            "C" => Some("c"),
                            "CXX" => Some("cpp"),
                            _ => None,
                        }),
                );
            }
        }
        self.document_family
    }

    pub(super) fn document_hint(&self, path: &Path) -> &'static str {
        match self.contextual_language(path) {
            Some("c") => {
                "gramática local C orientada pelo contexto; flags de inclusão continuam no clangd"
            }
            Some("cpp") => {
                "gramática local C++ orientada pelo contexto; flags de inclusão continuam no clangd"
            }
            _ => {
                "gramática local C++ de compatibilidade: contexto C/C++ ausente ou ambíguo; flags de inclusão continuam no clangd"
            }
        }
    }
}

pub(super) fn ambiguous_header(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "h")
}

fn unique(languages: impl Iterator<Item = Option<&'static str>>) -> Option<&'static str> {
    let mut result = None;
    for language in languages {
        let language = language?;
        if result.is_some_and(|previous| previous != language) {
            return None;
        }
        result = Some(language);
    }
    result
}

fn command_language(path: &Path, unit: &CdbUnit) -> Option<&'static str> {
    let driver = Path::new(&unit.compiler).file_stem()?.to_str()?;
    let cpp_driver = driver == "c++"
        || driver
            .split('-')
            .any(|part| matches!(part, "clang++" | "g++"));
    let fallback = if cpp_driver {
        Some("cpp")
    } else if ambiguous_header(path) {
        Some("c")
    } else {
        SymbolExtractor::language_for(path)
    };
    let mut language = fallback.filter(|language| matches!(*language, "c" | "cpp"));
    let mut input_seen = false;
    let mut arguments = unit.arguments.iter().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            break;
        }
        if !argument.starts_with('-') {
            let input = Path::new(&unit.directory).join(argument);
            input_seen |= crate::platform::canonicalize(&input).is_ok_and(|input| input == path);
        }
        let mode = if argument == "-x" {
            arguments.next().map(String::as_str)
        } else {
            argument
                .strip_prefix("-x")
                .filter(|value| !value.is_empty())
        };
        if let Some(mode) = mode {
            // GCC/Clang -x applies only to following inputs, unlike global /TC.
            if input_seen {
                continue;
            }
            language = match mode {
                "c" | "c-header" | "cpp-output" => Some("c"),
                "c++" | "c++-header" | "c++-cpp-output" => Some("cpp"),
                "none" => fallback,
                _ => None,
            };
        } else if matches!(argument.as_str(), "/TC" | "/TP") {
            language = Some(if argument == "/TC" { "c" } else { "cpp" });
        }
    }
    language
}
