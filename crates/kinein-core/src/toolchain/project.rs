//! O que o projeto aberto de fato usa: o `CMakeCache.txt` do build dir.
//!
//! Decisao do autor (2026-10-16, `roadmaps/59` §7.1.7.1): o automatico da
//! toolchain le o projeto, sem suposicao. Para os papeis que o `CMake`
//! decide (os compiladores e o gerador), o primeiro candidato detectado NAO
//! e' o que o build usa: sem nada fixado, o `cmake_arguments` nao passa
//! nada, e o `CMake` escolhe pelas regras dele (`CC`/`CXX`, `cc`/`c++` no
//! `PATH`, `CMAKE_GENERATOR` ou `Unix Makefiles`) e guarda a escolha no
//! cache. O chip dizia "Clang++ · Ninja" num projeto que compila com GCC e
//! Make (59 §7.1.7, F7). Os papeis que a IDE RODA direto (depurador, monitor
//! serial, `cargo`) continuam com a mescla de 2026-09-04: la' o primeiro
//! detectado e' mesmo o que roda.

use std::{fs, path::Path};

use kinein_protocol::{ToolchainCandidate, ToolchainRole};

/// O que o cache do projeto diz, para os papeis do `CMake`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct ProjectCache {
    c_compiler: Option<String>,
    cxx_compiler: Option<String>,
    generator: Option<String>,
    make_program: Option<String>,
}

impl ProjectCache {
    /// Le o cache do build dir unico da IDE; sem cache (projeto nunca
    /// configurado, ou so' Cargo), tudo ausente.
    ///
    /// O caminho e' o de `cmake::build_dir`, escrito aqui como o `run.rs` o
    /// escreve: o `cmake` ja' depende deste modulo, e a volta seria ciclo.
    pub(super) fn read(root: &Path) -> Self {
        fs::read_to_string(root.join(".kinein").join("build").join("CMakeCache.txt"))
            .map(|body| Self::parse(&body))
            .unwrap_or_default()
    }

    fn parse(body: &str) -> Self {
        let mut cache = Self::default();
        // Comentario (`# ...`, `// ...`) nao casa com nenhum dos quatro nomes.
        for line in body.lines().map(str::trim) {
            let Some((declaration, value)) = line.split_once('=') else {
                continue;
            };
            let name = declaration
                .split_once(':')
                .map_or(declaration, |(name, _)| name);
            let slot = match name {
                "CMAKE_C_COMPILER" => &mut cache.c_compiler,
                "CMAKE_CXX_COMPILER" => &mut cache.cxx_compiler,
                "CMAKE_GENERATOR" => &mut cache.generator,
                "CMAKE_MAKE_PROGRAM" => &mut cache.make_program,
                _ => continue,
            };
            let value = value.trim();
            *slot = (!value.is_empty()).then(|| value.to_owned());
        }
        cache
    }

    /// O papel e' decidido pelo `CMake` (e entao o automatico vem daqui)?
    pub(super) const fn decides(role: ToolchainRole) -> bool {
        matches!(
            role,
            ToolchainRole::CCompiler | ToolchainRole::CxxCompiler | ToolchainRole::Generator
        )
    }

    /// `(resolvedPath, effectiveId)` do automatico de um papel do `CMake`.
    ///
    /// O caminho e' o do cache (o compilador; o programa do gerador). O id e'
    /// o do candidato detectado que e' o MESMO arquivo (`/usr/bin/c++` e o
    /// `g++` detectado resolvem para o mesmo binario), ou, no gerador, o nome
    /// do cache quando ele e' um candidato detectado. Sem casamento, so' o
    /// caminho.
    pub(super) fn automatic(
        &self,
        role: ToolchainRole,
        detected: &[ToolchainCandidate],
    ) -> (Option<String>, Option<String>) {
        match role {
            ToolchainRole::Generator => (
                self.make_program.clone(),
                self.generator
                    .clone()
                    .filter(|name| detected.iter().any(|candidate| candidate.id == *name)),
            ),
            ToolchainRole::CCompiler | ToolchainRole::CxxCompiler => {
                let path = if role == ToolchainRole::CCompiler {
                    self.c_compiler.clone()
                } else {
                    self.cxx_compiler.clone()
                };
                let id = path.as_deref().and_then(|path| {
                    detected
                        .iter()
                        .find(|candidate| {
                            candidate
                                .path
                                .as_deref()
                                .is_some_and(|other| same_file(path, other))
                        })
                        .map(|candidate| candidate.id.clone())
                });
                (path, id)
            }
            _ => (None, None),
        }
    }
}

/// Dois caminhos para o mesmo binario (links resolvidos); sem como resolver,
/// so' o texto igual.
fn same_file(left: &str, right: &str) -> bool {
    crate::platform::same_file(Path::new(left), Path::new(right)).unwrap_or(left == right)
}
