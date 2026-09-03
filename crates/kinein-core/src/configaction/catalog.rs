//! O catalogo: a TABELA das acoes de configuracao, e mais nada.
//!
//! Eram as 16 do MVP; desde 2026-09-03 sao 18 — `cmake.findPackage` e
//! `cmake.fetchContent` entraram com o dominio `library` (etapa 20 do
//! `roadmaps/35`), que produz planos nomeando as duas.
//!
//! A lista fechada vem da §12 da spec de MVP (10 de `CMake`, 6 de Cargo). Ela e
//! DADO ESTATICO de proposito: nao ha registro dinamico nem contribuicao de
//! terceiro neste projeto (`ARCHITECTURE.md` §8.1), entao acao nova e uma
//! entrada aqui mais um planejador no modulo do arquivo que ela edita.
//!
//! **Disponibilidade nao mora aqui.** Quem mede o workspace e decide se uma
//! acao esta `available`, `recommended` ou `unavailable` e o modulo
//! [`super::availability`]; este arquivo nao sabe o que e um workspace.

use kinein_protocol::{ConfigActionEffect, ConfigActionRisk, ConfigActionScope};

/// Definicao estatica de uma acao, antes de olhar o workspace.
pub(super) struct ActionDefinition {
    /// Id estavel usado por `preview`/`apply`.
    pub(super) id: &'static str,
    /// Titulo curto.
    pub(super) title: &'static str,
    /// Descricao de uma linha.
    pub(super) description: &'static str,
    /// Build system dono da acao.
    pub(super) scope: ConfigActionScope,
    /// Categoria da lista.
    pub(super) category: &'static str,
    /// Risco declarado.
    pub(super) risk: ConfigActionRisk,
    /// Arquivos afetados, relativos a raiz.
    pub(super) affects: &'static [&'static str],
    /// O que `apply` faz.
    pub(super) effect: ConfigActionEffect,
    /// Parametros: `(nome, rotulo, obrigatorio, exemplo)`.
    pub(super) params: &'static [(&'static str, &'static str, bool, &'static str)],
    /// Documentacao associada: `(kind, titulo, url_key)`.
    pub(super) docs: &'static [(&'static str, &'static str, &'static str)],
}

/// As acoes, na ordem em que a UI as mostra. O tamanho do array e' trava de
/// compilacao sobre a contagem: acrescentar sem atualizar reprova no build.
#[must_use]
pub(super) fn definitions() -> &'static [ActionDefinition] {
    &DEFINITIONS
}

/// A definicao de um id, quando ele existe.
#[must_use]
pub(super) fn definition(id: &str) -> Option<&'static ActionDefinition> {
    DEFINITIONS.iter().find(|action| action.id == id)
}

const CMAKELISTS_ONLY: &[&str] = &["CMakeLists.txt"];
const PRESETS_ONLY: &[&str] = &["CMakePresets.json"];
const CARGO_ONLY: &[&str] = &["Cargo.toml"];
const NO_FILE: &[&str] = &[];

const VISIBILITY_PARAM: (&str, &str, bool, &str) = ("visibility", "Visibilidade", false, "PRIVATE");

static DEFINITIONS: [ActionDefinition; 18] = [
    ActionDefinition {
        id: "cmake.enableCompileCommands",
        title: "Habilitar compile_commands.json",
        description: "Exporta a compilation database que o clangd usa para resolver includes.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Basic",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[],
        docs: &[(
            "officialDoc",
            "CMake: CMAKE_EXPORT_COMPILE_COMMANDS",
            "cmake.CMAKE_EXPORT_COMPILE_COMMANDS",
        )],
    },
    ActionDefinition {
        id: "cmake.createDebugPreset",
        title: "Criar preset Debug",
        description: "Acrescenta um configure preset 'debug' com CMAKE_BUILD_TYPE=Debug.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Basic",
        risk: ConfigActionRisk::Medium,
        affects: PRESETS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[],
        docs: &[("officialDoc", "CMake: cmake-presets(7)", "cmake.presets")],
    },
    ActionDefinition {
        id: "cmake.createReleasePreset",
        title: "Criar preset Release",
        description: "Acrescenta um configure preset 'release' com CMAKE_BUILD_TYPE=Release.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Basic",
        risk: ConfigActionRisk::Medium,
        affects: PRESETS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[],
        docs: &[("officialDoc", "CMake: cmake-presets(7)", "cmake.presets")],
    },
    ActionDefinition {
        id: "cmake.addExecutable",
        title: "Adicionar executavel",
        description: "Declara um target de executavel com as fontes informadas.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Targets",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Nome do target", true, "app"),
            ("sources", "Fontes", true, "src/main.cpp"),
        ],
        docs: &[(
            "officialDoc",
            "CMake: add_executable",
            "cmake.add_executable",
        )],
    },
    ActionDefinition {
        id: "cmake.addStaticLibrary",
        title: "Adicionar biblioteca estatica",
        description: "Declara um target STATIC com as fontes informadas.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Targets",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Nome do target", true, "core"),
            ("sources", "Fontes", true, "src/core.cpp"),
        ],
        docs: &[("officialDoc", "CMake: add_library", "cmake.add_library")],
    },
    ActionDefinition {
        id: "cmake.addSourceToTarget",
        title: "Adicionar fonte a um target",
        description: "Acrescenta arquivos a um target ja declarado.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Targets",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("target", "Target", true, "app"),
            ("sources", "Fontes", true, "src/extra.cpp"),
            VISIBILITY_PARAM,
        ],
        docs: &[(
            "officialDoc",
            "CMake: target_sources",
            "cmake.target_sources",
        )],
    },
    ActionDefinition {
        id: "cmake.addIncludeDirectory",
        title: "Adicionar include dir",
        description: "Acrescenta um diretorio de cabecalhos ao target.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Intermediate",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("target", "Target", true, "app"),
            ("directories", "Diretorios", true, "include"),
            VISIBILITY_PARAM,
        ],
        docs: &[(
            "officialDoc",
            "CMake: target_include_directories",
            "cmake.target_include_directories",
        )],
    },
    ActionDefinition {
        id: "cmake.findPackage",
        title: "Encontrar um pacote instalado",
        description: "Declara find_package para uma biblioteca ja instalada no sistema.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Intermediate",
        risk: ConfigActionRisk::Low,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("package", "Pacote", true, "fmt"),
            ("version", "Versao minima", false, "12.2.0"),
        ],
        docs: &[("officialDoc", "CMake: find_package", "cmake.find_package")],
    },
    ActionDefinition {
        id: "cmake.fetchContent",
        title: "Baixar uma dependencia na versao fixada",
        description: "Declara FetchContent com a tag PINADA quando o pacote nao esta no sistema.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Intermediate",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Nome", true, "fmt"),
            (
                "repository",
                "Repositorio",
                true,
                "https://github.com/fmtlib/fmt",
            ),
            ("tag", "Tag PINADA", true, "12.2.0"),
        ],
        docs: &[("officialDoc", "CMake: FetchContent", "cmake.fetchcontent")],
    },
    ActionDefinition {
        id: "cmake.addTargetLinkLibraries",
        title: "Linkar bibliotecas ao target",
        description: "Vincula bibliotecas existentes a um target CMake.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Intermediate",
        risk: ConfigActionRisk::Medium,
        affects: CMAKELISTS_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("target", "Target", true, "app"),
            ("libraries", "Bibliotecas", true, "core Threads::Threads"),
            VISIBILITY_PARAM,
        ],
        docs: &[(
            "officialDoc",
            "CMake: target_link_libraries",
            "cmake.target_link_libraries",
        )],
    },
    ActionDefinition {
        id: "cmake.inspectCache",
        title: "Inspecionar o cache do CMake",
        description: "Mostra as variaveis do CMakeCache.txt do build dir da IDE.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Advanced",
        risk: ConfigActionRisk::Low,
        affects: NO_FILE,
        effect: ConfigActionEffect::Inspect,
        params: &[],
        docs: &[(
            "officialDoc",
            "CMake: cmake-variables(7)",
            "cmake.variables",
        )],
    },
    ActionDefinition {
        id: "cmake.repairBuildDir",
        title: "Reparar build dir obsoleto",
        description: "Remove .kinein/build para o proximo configure nascer limpo.",
        scope: ConfigActionScope::Cmake,
        category: "CMake Advanced",
        risk: ConfigActionRisk::High,
        affects: NO_FILE,
        effect: ConfigActionEffect::Delegate,
        params: &[],
        docs: &[(
            "kineinGuide",
            "Kinein: diretorio de build unico",
            "kinein.build_dir",
        )],
    },
    ActionDefinition {
        id: "cargo.addDependency",
        title: "Adicionar dependencia",
        description: "Acrescenta uma crate em [dependencies] do Cargo.toml.",
        scope: ConfigActionScope::Cargo,
        category: "Cargo Dependencies",
        risk: ConfigActionRisk::Medium,
        affects: CARGO_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Crate", true, "serde"),
            ("version", "Versao", true, "1.0"),
            ("features", "Features", false, "derive"),
        ],
        docs: &[(
            "officialDoc",
            "Cargo: specifying dependencies",
            "cargo.dependencies",
        )],
    },
    ActionDefinition {
        id: "cargo.addDevDependency",
        title: "Adicionar dev-dependency",
        description: "Acrescenta uma crate em [dev-dependencies] do Cargo.toml.",
        scope: ConfigActionScope::Cargo,
        category: "Cargo Dependencies",
        risk: ConfigActionRisk::Medium,
        affects: CARGO_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Crate", true, "proptest"),
            ("version", "Versao", true, "1.0"),
            ("features", "Features", false, ""),
        ],
        docs: &[(
            "officialDoc",
            "Cargo: development dependencies",
            "cargo.dev_dependencies",
        )],
    },
    ActionDefinition {
        id: "cargo.addFeature",
        title: "Adicionar feature",
        description: "Declara uma feature em [features] do Cargo.toml.",
        scope: ConfigActionScope::Cargo,
        category: "Cargo Features",
        risk: ConfigActionRisk::Medium,
        affects: CARGO_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[
            ("name", "Feature", true, "extra"),
            ("enables", "Habilita", false, "serde/derive"),
        ],
        docs: &[("officialDoc", "Cargo: features", "cargo.features")],
    },
    ActionDefinition {
        id: "cargo.setEdition",
        title: "Definir a edition",
        description: "Troca a edition do pacote em [package] do Cargo.toml.",
        scope: ConfigActionScope::Cargo,
        category: "Cargo Basic",
        risk: ConfigActionRisk::Medium,
        affects: CARGO_ONLY,
        effect: ConfigActionEffect::Edit,
        params: &[("edition", "Edition", true, "2024")],
        docs: &[("officialDoc", "Cargo: the edition field", "cargo.edition")],
    },
    ActionDefinition {
        id: "cargo.check",
        title: "Rodar cargo check",
        description: "Compila sem gerar binario e joga os erros na aba Problemas.",
        scope: ConfigActionScope::Cargo,
        category: "Cargo Basic",
        risk: ConfigActionRisk::Low,
        affects: NO_FILE,
        effect: ConfigActionEffect::Job,
        params: &[],
        docs: &[("officialDoc", "Cargo: cargo check", "cargo.check")],
    },
    ActionDefinition {
        id: "cargo.createRunConfig",
        title: "Criar run config",
        description: "Salva um comando nomeado em .kinein/runconfigs.json e o ativa.",
        scope: ConfigActionScope::Cargo,
        category: "Run/Debug",
        risk: ConfigActionRisk::Low,
        affects: NO_FILE,
        effect: ConfigActionEffect::Delegate,
        params: &[
            ("name", "Nome", true, "Servidor"),
            ("command", "Comando", true, "cargo run --bin server"),
        ],
        docs: &[(
            "kineinGuide",
            "Kinein: run configurations",
            "kinein.runconfig",
        )],
    },
];
