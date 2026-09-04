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
    /// O sinal FORTE, quando existe: a biblioteca virou padrao ISO, ou passou
    /// pela revisao formal do Boost. `None` = so' tem adocao e manutencao.
    ///
    /// **NAO existe "certificacao oficial" de biblioteca C++.** O WG21
    /// padroniza a linguagem e a biblioteca padrao; a Standard C++ Foundation
    /// apoia a comunidade e declara explicitamente que seu objetivo e reduzir
    /// barreiras para ADOTAR bibliotecas no proprio Standard — nao certificar
    /// as de terceiros. Este campo registra o sinal que EXISTE de verdade, em
    /// vez de inventar um selo que ninguem emite.
    pub(super) standard_lineage: Option<&'static str>,
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
/// O tamanho do array e' trava de compilacao sobre a contagem, igual ao
/// `configaction::catalog`: acrescentar uma entrada sem atualizar o numero
/// reprova no build. E' `static` e nao corpo de funcao pelo mesmo motivo de la:
/// tabela de DADO nao deve disparar o lint de funcao longa, porque encurta-la
/// significaria esconder entrada.
pub(super) static DEFINITIONS: [LibraryDefinition; 15] = [
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
        // O README do proprio fmt se descreve como "Implementation of
        // C++20 std::format and C++23 std::print" — o comite ISO pegou a
        // biblioteca e a tornou padrao. E' o sinal mais forte que existe.
        standard_lineage: Some("virou std::format no C++20"),
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
        standard_lineage: None,
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
        standard_lineage: None,
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
        standard_lineage: None,
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
        standard_lineage: None,
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
        standard_lineage: None,
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
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "boost",
        name: "Boost",
        summary: "Colecao de bibliotecas revisadas por pares; varias viraram std:: depois.",
        category: "Colecao",
        license: "BSL-1.0",
        pinned_version: "boost-1.92.0",
        released_at: "2026-08-12",
        package_name: "Boost",
        // Boost e' colecao: o `find_package` pede COMPONENTS. `boost_headers`
        // e' o alvo das partes header-only, que cobre a maioria dos usos —
        // quem precisa de uma parte compilada (filesystem, thread) acrescenta
        // o componente. Chutar a lista de componentes por conta da IDE seria
        // linkar o que o usuario nao pediu.
        targets: &["Boost::boost"],
        repository: "https://github.com/boostorg/boost",
        documentation: "https://www.boost.org/doc/",
        // O segundo sinal mais forte que existe em C++, depois da adocao no
        // padrao: a revisao formal por pares do Boost. E varias bibliotecas
        // dele FORAM adotadas — filesystem, optional, variant, any,
        // shared_ptr, thread e chrono estao no std:: por causa disso.
        standard_lineage: Some(
            "revisao formal por pares; filesystem/optional/variant/shared_ptr viraram std::",
        ),
    },
    LibraryDefinition {
        id: "asio",
        name: "Asio",
        summary: "Rede e I/O assincrono: sockets, timers e o modelo de execucao por trás deles.",
        category: "Rede",
        license: "BSL-1.0",
        pinned_version: "asio-1-38-2",
        released_at: "2026-07-19",
        package_name: "asio",
        targets: &["asio::asio"],
        repository: "https://github.com/chriskohlhoff/asio",
        documentation: "https://think-async.com/Asio/Documentation.html",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "abseil",
        name: "Abseil",
        summary: "Utilitarios do Google que preenchem lacunas da biblioteca padrao.",
        category: "Utilitarios",
        license: "Apache-2.0",
        pinned_version: "20260817.0",
        released_at: "2026-08-18",
        package_name: "absl",
        targets: &["absl::base", "absl::strings"],
        repository: "https://github.com/abseil/abseil-cpp",
        documentation: "https://abseil.io/docs/cpp/",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "zlib",
        name: "zlib",
        summary: "Compressao DEFLATE; a base de gzip, PNG e de metade da internet.",
        category: "Dados",
        license: "Zlib",
        pinned_version: "v1.3.2",
        released_at: "2026-02-17",
        package_name: "ZLIB",
        targets: &["ZLIB::ZLIB"],
        repository: "https://github.com/madler/zlib",
        documentation: "https://zlib.net/manual.html",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "sqlite3",
        name: "SQLite",
        summary: "Banco SQL em um arquivo, sem servidor; o mais implantado do mundo.",
        category: "Dados",
        // Dominio publico VERIFICADO em sqlite.org/copyright.html: "All of
        // the code and documentation in SQLite has been dedicated to the
        // public domain by the authors." Nao e' uma licenca permissiva —
        // e' ausencia de licenca, que e' ainda menos restritivo.
        license: "Dominio publico",
        pinned_version: "version-3.53.4",
        released_at: "2026-07-24",
        package_name: "SQLite3",
        targets: &["SQLite::SQLite3"],
        repository: "https://github.com/sqlite/sqlite",
        documentation: "https://www.sqlite.org/docs.html",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "eigen",
        name: "Eigen",
        summary: "Algebra linear: matrizes, vetores e solvers numericos, so' em cabecalho.",
        category: "Matematica",
        // ATENCAO, e a ressalva e' o motivo de este campo existir: MPL-2.0
        // e' copyleft FRACO, classe diferente das permissivas do resto do
        // catalogo. Verificado no COPYING.README: "Eigen is primarily
        // licensed under the Mozilla Public License 2.0", e ha dependencias
        // externas OPCIONAIS sob outras licencas, algumas GPL. Usar o Eigen
        // basico nao contamina; ligar uma dessas opcionais pode.
        license: "MPL-2.0 (copyleft fraco)",
        pinned_version: "5.0.1",
        released_at: "2025-11-08",
        package_name: "Eigen3",
        targets: &["Eigen3::Eigen"],
        repository: "https://gitlab.com/libeigen/eigen",
        documentation: "https://eigen.tuxfamily.org/dox/",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "glm",
        name: "GLM",
        summary: "Matematica de grafico: vetor, matriz e quaternion com a mesma \
                  sintaxe do GLSL.",
        category: "Matematica",
        // ATENCAO, e a ressalva importa: o `copying.txt` oferece DUAS licencas
        // — "The Happy Bunny License (Modified MIT License)" e "The MIT
        // License" — e quem usa escolhe. A MIT e' a saida limpa; a Happy Bunny
        // acrescenta uma clausula de "nao seja mau" que nao e' OSI. Verificado
        // no proprio copying.txt em 2026-09-04.
        license: "MIT (ou Happy Bunny, a escolha e sua)",
        pinned_version: "1.0.3",
        released_at: "2025-12-31",
        package_name: "glm",
        targets: &["glm::glm"],
        repository: "https://github.com/g-truc/glm",
        documentation: "https://github.com/g-truc/glm/blob/master/manual.md",
        standard_lineage: None,
    },
    LibraryDefinition {
        id: "libpqxx",
        name: "libpqxx",
        summary: "Cliente C++ oficial do PostgreSQL: consulta, transacao e tipos \
                  com RAII.",
        category: "Banco de dados",
        // O COPYING traz o texto da BSD 3-Clause SEM escrever o nome dela no
        // arquivo — conferido em 2026-09-04. O nome aqui e' a identificacao do
        // texto, nao uma citacao do arquivo, e esta ressalva existe para
        // ninguem achar que o projeto se declara assim por escrito.
        license: "BSD-3-Clause (texto, sem o nome no arquivo)",
        pinned_version: "8.0.2",
        released_at: "2026-07-18",
        package_name: "libpqxx",
        // `libpqxx::pqxx`, e nao `libpqxx::libpqxx`: o alias vem do
        // `src/CMakeLists.txt`, `add_library(libpqxx::pqxx ALIAS pqxx)`.
        // Verificado na fonte porque errar o alvo produz um plano que FALHA no
        // link, e o autor so' descobriria compilando.
        targets: &["libpqxx::pqxx"],
        repository: "https://github.com/jtv/libpqxx",
        documentation: "https://libpqxx.readthedocs.io/",
        standard_lineage: None,
    },
];

/// O catalogo.
#[must_use]
pub(super) fn definitions() -> &'static [LibraryDefinition] {
    &DEFINITIONS
}

/// Busca uma definicao por id.
#[must_use]
pub(super) fn find(id: &str) -> Option<&'static LibraryDefinition> {
    definitions().iter().find(|entry| entry.id == id)
}
