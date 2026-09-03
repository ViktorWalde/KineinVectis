//! O catalogo CURADO de bibliotecas C/C++, e mais nada.
//!
//! Mesmo molde do [`crate::configaction::catalog`]: DADO ESTATICO, sem registro
//! dinamico e sem contribuicao de terceiro (`ARCHITECTURE.md` §8.1). Biblioteca
//! nova e uma entrada aqui.
//!
//! **POR QUE CADA CAMPO EXISTE** (decisao do autor em 2026-09-03,
//! `roadmaps/35` §2.1). Quando a IDE oferece uma biblioteca ela AFIRMA "esta e'
//! solida", e a afirmacao precisa de quem a sustente. Por isso nenhuma entrada
//! entra sem licenca VERIFICADA, ultimo release com data e versao PINADA.
//!
//! A verificacao e' na fonte, nao no palpite: em 2026-09-03 o campo `license`
//! da API do GitHub devolveu `NOASSERTION` para spdlog e CLI11 — as duas TEM
//! licenca permissiva, o detector e' que falhou. Quem copiasse o campo teria
//! marcado as duas como desconhecidas; quem abrisse o arquivo `LICENSE` acha
//! MIT e BSD-3-Clause. **Ler a fonte e' o metodo.**
//!
//! **Disponibilidade nao mora aqui.** Quem mede o que existe NESTA MAQUINA e'
//! o [`super::availability`]; este arquivo nao sabe o que e um workspace.

/// Uma entrada auditada do catalogo.
pub(super) struct LibraryDefinition {
    /// Id estavel usado pelos metodos `library.*`.
    pub(super) id: &'static str,
    /// Nome como o projeto se chama.
    pub(super) name: &'static str,
    /// O que ela FAZ, em uma frase, para quem nunca ouviu falar.
    pub(super) summary: &'static str,
    /// Categoria da lista.
    pub(super) category: &'static str,
    /// Identificador SPDX, verificado no arquivo de licenca do projeto.
    pub(super) license: &'static str,
    /// Versao pinada: a tag que o `FetchContent` usa. NUNCA "latest".
    pub(super) pinned_version: &'static str,
    /// Data do release da versao pinada (ISO), para a idade ser visivel.
    pub(super) released_at: &'static str,
    /// Nome do pacote em `find_package`.
    pub(super) package_name: &'static str,
    /// Alvos a linkar (`target_link_libraries`).
    pub(super) targets: &'static [&'static str],
    /// Repositorio, para o `FetchContent`.
    pub(super) repository: &'static str,
    /// Documentacao oficial.
    pub(super) documentation: &'static str,
}

// NAO ha campo "caminho de integracao suportado" aqui, e a ausencia e
// deliberada. A primeira versao tinha um enum com FindPackage/`FetchContent`/
// Either, e as SETE entradas auditadas sairam `Either` — os lints estritos
// reprovaram as duas variantes nunca construidas, e estavam certos. O caminho
// e' decidido pela MEDICAO (`availability`), nao por um campo do catalogo.
// Quando existir uma biblioteca que so' aceita um dos dois, o campo volta com
// um usuario de verdade.

/// O catalogo. **Auditado em 2026-09-03**, lendo a API do GitHub e, quando o
/// detector de licenca falhou, o proprio arquivo `LICENSE`.
///
/// Comeca pequeno de proposito: catalogo e' divida por ENTRADA, nao por
/// feature — cada linha aqui envelhece sozinha e precisa ser remedida.
#[must_use]
pub(super) const fn definitions() -> &'static [LibraryDefinition] {
    &[
        LibraryDefinition {
            id: "fmt",
            name: "fmt",
            summary: "Formatacao de texto segura e rapida; e a base do std::format do C++20.",
            category: "Texto",
            license: "MIT",
            pinned_version: "12.2.0",
            released_at: "2026-06-16",
            package_name: "fmt",
            targets: &["fmt::fmt"],
            repository: "https://github.com/fmtlib/fmt",
            documentation: "https://fmt.dev/latest/index.html",
        },
        LibraryDefinition {
            id: "spdlog",
            name: "spdlog",
            summary: "Log rapido, com niveis e destinos (arquivo, console) configuraveis.",
            category: "Diagnostico",
            license: "MIT",
            pinned_version: "v1.17.0",
            released_at: "2026-01-04",
            package_name: "spdlog",
            targets: &["spdlog::spdlog"],
            repository: "https://github.com/gabime/spdlog",
            documentation: "https://github.com/gabime/spdlog/wiki",
        },
        LibraryDefinition {
            id: "nlohmann_json",
            name: "nlohmann/json",
            summary: "Ler e escrever JSON com sintaxe parecida com a de um container do C++.",
            category: "Dados",
            license: "MIT",
            pinned_version: "v3.12.0",
            released_at: "2025-04-11",
            package_name: "nlohmann_json",
            targets: &["nlohmann_json::nlohmann_json"],
            repository: "https://github.com/nlohmann/json",
            documentation: "https://json.nlohmann.me/",
        },
        LibraryDefinition {
            id: "catch2",
            name: "Catch2",
            summary: "Framework de teste unitario; o teste e uma funcao, sem classe nem macro de registro.",
            category: "Teste",
            license: "BSL-1.0",
            pinned_version: "v3.16.0",
            released_at: "2026-08-25",
            package_name: "Catch2",
            targets: &["Catch2::Catch2WithMain"],
            repository: "https://github.com/catchorg/Catch2",
            documentation: "https://github.com/catchorg/Catch2/blob/devel/docs/Readme.md",
        },
        LibraryDefinition {
            id: "googletest",
            name: "GoogleTest",
            summary: "Framework de teste e de mock; o mais comum em base de codigo grande.",
            category: "Teste",
            license: "BSD-3-Clause",
            pinned_version: "v1.18.0",
            released_at: "2026-08-10",
            package_name: "GTest",
            targets: &["GTest::gtest_main"],
            repository: "https://github.com/google/googletest",
            documentation: "https://google.github.io/googletest/",
        },
        LibraryDefinition {
            id: "cli11",
            name: "CLI11",
            summary: "Parser de linha de comando: opcoes, subcomandos e validacao, em um header.",
            category: "Aplicacao",
            license: "BSD-3-Clause",
            pinned_version: "v2.7.2",
            released_at: "2026-08-02",
            package_name: "CLI11",
            targets: &["CLI11::CLI11"],
            repository: "https://github.com/CLIUtils/CLI11",
            documentation: "https://cliutils.github.io/CLI11/book/",
        },
        LibraryDefinition {
            id: "benchmark",
            name: "Google Benchmark",
            summary: "Micro-benchmark com estatistica: mede funcao, nao o programa inteiro.",
            category: "Desempenho",
            license: "Apache-2.0",
            pinned_version: "v1.9.5",
            released_at: "2026-01-21",
            package_name: "benchmark",
            targets: &["benchmark::benchmark"],
            repository: "https://github.com/google/benchmark",
            documentation: "https://github.com/google/benchmark/blob/main/docs/user_guide.md",
        },
    ]
}

/// Busca uma definicao por id.
#[must_use]
pub(super) fn find(id: &str) -> Option<&'static LibraryDefinition> {
    definitions().iter().find(|entry| entry.id == id)
}
