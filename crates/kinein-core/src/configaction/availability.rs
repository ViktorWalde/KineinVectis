//! Disponibilidade: o que a acao pode fazer NESTE workspace, e por que nao.
//!
//! Separado do [`super::catalog`] porque sao duas responsabilidades diferentes:
//! la mora a tabela (dado estatico, muda quando o produto ganha uma acao); aqui
//! mora a medicao (le o disco, muda quando o projeto do usuario muda). Depois
//! do corte, o catalogo nao tem uma palavra para "disponivel" — que e o teste
//! de vocabulario da `ARCHITECTURE.md` §4 regra 9.
//!
//! O ESCOPO e o que a spec 9.2 §1 manda: a lista e filtrada pelo build system
//! ativo do workspace, nao pelo compilador. Projeto Cargo nao ve acao `CMake`.

use std::path::Path;

use kinein_protocol::{
    ConfigActionDocLink, ConfigActionInfo, ConfigActionParamInfo, ConfigActionScope,
    ConfigActionState,
};

use super::catalog::ActionDefinition;
use super::{builddir, cargotoml, cmakelists, presets};

/// O que o workspace diz sobre si, lido UMA vez por `configAction.list`.
pub(super) struct WorkspaceFacts {
    /// Conteudo do `CMakeLists.txt`, quando ele existe.
    pub(super) cmakelists: Option<String>,
    /// Conteudo do `CMakePresets.json`, quando ele existe.
    pub(super) presets: Option<String>,
    /// Conteudo do `Cargo.toml`, quando ele existe.
    pub(super) cargo_toml: Option<String>,
    /// O configure da IDE ja rodou.
    pub(super) configured: bool,
    /// Ha uma compilation database alcancavel.
    pub(super) has_cdb: bool,
    /// Motivo pelo qual o build dir merece reparo, quando ha um.
    pub(super) stale: Option<String>,
    /// Targets declarados literalmente no `CMakeLists.txt`.
    pub(super) targets: Vec<String>,
    /// `.cargo/config.toml` existe — a acao de embarcado do Cargo ja rodou.
    pub(super) cargo_config: bool,
}

impl WorkspaceFacts {
    /// Le do disco tudo que a avaliacao de estado precisa.
    #[must_use]
    pub(super) fn measure(root: &Path) -> Self {
        let cmakelists = std::fs::read_to_string(root.join(cmakelists::CMAKELISTS)).ok();
        let targets = cmakelists
            .as_deref()
            .map(cmakelists::declared_targets)
            .unwrap_or_default();
        Self {
            presets: std::fs::read_to_string(root.join(presets::CMAKE_PRESETS)).ok(),
            cargo_toml: std::fs::read_to_string(root.join(cargotoml::CARGO_TOML)).ok(),
            configured: builddir::is_configured(root),
            has_cdb: crate::cdb::status(root).directory.is_some(),
            stale: builddir::stale_reason(root),
            // `.cargo/config.toml` e' o que a acao de embarcado do Cargo cria;
            // sem isso ela nao teria como dizer "ja' esta feito".
            cargo_config: root.join(".cargo/config.toml").is_file(),
            targets,
            cmakelists,
        }
    }
}

/// Monta o payload de protocolo de uma acao, ja com o estado medido.
#[must_use]
pub(super) fn describe(
    root: &Path,
    action: &ActionDefinition,
    facts: &WorkspaceFacts,
    in_scope: bool,
) -> ConfigActionInfo {
    let (state, reason) = if in_scope {
        evaluate(action, facts)
    } else {
        (
            ConfigActionState::HiddenByScope,
            Some(format!(
                "{} nao esta ativo neste workspace",
                match action.scope {
                    ConfigActionScope::Cmake => "CMake",
                    ConfigActionScope::Cargo => "Cargo",
                }
            )),
        )
    };
    ConfigActionInfo {
        id: action.id.to_owned(),
        title: action.title.to_owned(),
        description: action.description.to_owned(),
        scope: action.scope,
        category: action.category.to_owned(),
        risk: action.risk,
        risk_label: action.risk.label().to_owned(),
        risk_explanation: action.risk.explanation().to_owned(),
        affects: action
            .affects
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
        effect: action.effect,
        state,
        reason,
        params: action
            .params
            .iter()
            .map(
                |(name, label, required, placeholder)| ConfigActionParamInfo {
                    name: (*name).to_owned(),
                    label: (*label).to_owned(),
                    required: *required,
                    placeholder: (*placeholder).to_owned(),
                    // O significado e os valores vem do NOME do parametro, e
                    // nao da acao: `target` quer dizer a mesma coisa em todas
                    // elas. Ver `parametros.rs`.
                    description: super::parametros::describe(name).to_owned(),
                    suggestions: super::parametros::suggestions(root, name),
                },
            )
            .collect(),
        docs: action
            .docs
            .iter()
            .map(|(kind, title, url_key)| ConfigActionDocLink {
                kind: (*kind).to_owned(),
                title: (*title).to_owned(),
                url_key: (*url_key).to_owned(),
            })
            .collect(),
    }
}

/// Estado de uma acao neste workspace, com o porque quando nao e `available`.
fn evaluate(
    action: &ActionDefinition,
    facts: &WorkspaceFacts,
) -> (ConfigActionState, Option<String>) {
    match action.scope {
        ConfigActionScope::Cmake if facts.cmakelists.is_none() => {
            return partial("CMakeLists.txt nao existe na raiz do workspace");
        }
        ConfigActionScope::Cargo if facts.cargo_toml.is_none() => {
            return partial("Cargo.toml nao existe na raiz do workspace");
        }
        _ => {}
    }

    match action.id {
        "cmake.enableCompileCommands" => {
            let text = facts.cmakelists.as_deref().unwrap_or_default();
            if cmakelists::exports_compile_commands(text) {
                already_applied(format!(
                    "o CMakeLists.txt ja define {}",
                    cmakelists::EXPORT_COMPILE_COMMANDS
                ))
            } else if facts.has_cdb {
                available()
            } else {
                recommended("sem compilation database, o clangd nao resolve includes")
            }
        }
        // As acoes de REGIME de compilacao (2026-09-04): cada uma sabe dizer
        // se o efeito dela JA' esta no arquivo. Sem isso a lista unificada
        // mostraria como "disponivel" o que o autor ja' ligou — que e' a
        // confusao que o estado `AlreadyApplied` existe para acabar.
        "cmake.setCxxStandard" => marca_no_cmakelists(
            facts,
            "CMAKE_CXX_STANDARD",
            "o CMakeLists.txt ja fixa o padrao C++",
        ),
        "cmake.strictWarnings" => {
            marca_no_cmakelists(facts, "-Wpedantic", "este projeto ja liga avisos rigorosos")
        }
        "cmake.enableSanitizers" => {
            marca_no_cmakelists(facts, "-fsanitize", "este projeto ja liga sanitizers")
        }
        "cmake.enableOpenMP" => marca_no_cmakelists(
            facts,
            "OpenMP::OpenMP_CXX",
            "este projeto ja linka o OpenMP",
        ),
        "cmake.generateHexBin" => marca_no_cmakelists(
            facts,
            "CMAKE_OBJCOPY",
            "este projeto ja gera .hex/.bin depois do build",
        ),
        "cargo.embeddedTarget" if facts.cargo_config => {
            already_applied(".cargo/config.toml ja existe neste projeto".to_owned())
        }
        "cmake.createDebugPreset" => preset_state(facts, "debug"),
        "cmake.createReleasePreset" => preset_state(facts, "release"),
        "cmake.addSourceToTarget"
        | "cmake.addIncludeDirectory"
        | "cmake.addTargetLinkLibraries"
            if facts.targets.is_empty() =>
        {
            partial("nenhum target declarado no CMakeLists.txt")
        }
        "cmake.inspectCache" if !facts.configured => {
            partial("o configure ainda nao rodou nesta IDE")
        }
        "cmake.repairBuildDir" => {
            if facts.configured {
                facts.stale.clone().map_or_else(available, recommended)
            } else {
                unavailable("nao ha diretorio de build da IDE para remover".to_owned())
            }
        }
        _ => available(),
    }
}

fn preset_state(facts: &WorkspaceFacts, name: &str) -> (ConfigActionState, Option<String>) {
    let exists = facts
        .presets
        .as_deref()
        .is_some_and(|text| presets::has_preset(text, name));
    if exists {
        already_applied(format!("o preset {name} ja existe em CMakePresets.json"))
    } else {
        available()
    }
}

const fn available() -> (ConfigActionState, Option<String>) {
    (ConfigActionState::Available, None)
}

/// `AlreadyApplied` quando a marca esta no `CMakeLists.txt`.
///
/// A marca e' um trecho do que a PROPRIA acao escreve — `-Wpedantic`,
/// `-fsanitize`, `OpenMP::OpenMP_CXX`. Busca textual, e o limite e' honesto:
/// um projeto que escreveu a flag a mao, com outra grafia, aparece como
/// disponivel. O pior caso e' oferecer de novo, e a previa mostra o diff antes
/// — nunca o contrario, que seria esconder uma acao que o autor ainda precisa.
fn marca_no_cmakelists(
    facts: &WorkspaceFacts,
    marca: &str,
    razao: &str,
) -> (ConfigActionState, Option<String>) {
    if facts
        .cmakelists
        .as_deref()
        .is_some_and(|texto| texto.contains(marca))
    {
        already_applied(razao.to_owned())
    } else {
        available()
    }
}

/// O efeito JA' esta no projeto — verde na tela, nao cinza.
fn already_applied(reason: impl Into<String>) -> (ConfigActionState, Option<String>) {
    (ConfigActionState::AlreadyApplied, Some(reason.into()))
}

fn recommended(reason: impl Into<String>) -> (ConfigActionState, Option<String>) {
    (ConfigActionState::Recommended, Some(reason.into()))
}

fn partial(reason: &str) -> (ConfigActionState, Option<String>) {
    (
        ConfigActionState::PartiallyAvailable,
        Some(reason.to_owned()),
    )
}

const fn unavailable(reason: String) -> (ConfigActionState, Option<String>) {
    (ConfigActionState::Unavailable, Some(reason))
}
