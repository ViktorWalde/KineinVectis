//! Configuration Actions: receitas de configuracao de projeto aplicadas com
//! preview e consentimento.
//!
//! # O ciclo, que e o contrato
//!
//! ```text
//! configAction.list      o que faz sentido NESTE workspace, e por que
//! configAction.preview   o plano: o diff exato, ou o relatorio
//! configAction.apply     escreve o plano, comparando com o snapshot do preview
//! ```
//!
//! Preview e apply calculam **o mesmo** plano ([`plan::ActionPlan`]); o preview
//! so nao grava. E isso que garante que o diff mostrado e o diff aplicado — a
//! spec 9.1 §10 pede "explicar, mostrar preview, aplicar com consentimento e
//! validar", e um segundo caminho de codigo para gravar quebraria a promessa no
//! ponto exato em que ela importa.
//!
//! # Organizacao interna
//!
//! - [`catalog`]: a tabela das 16 acoes do MVP;
//! - [`availability`]: o que cada acao pode fazer NESTE workspace, e por que nao;
//! - [`plan`]: o plano e a escrita com `compare-before-save`;
//! - [`cmakelists`]: as acoes que editam `CMakeLists.txt`;
//! - [`presets`]: as acoes que criam configure presets;
//! - [`builddir`]: inspecionar o cache e reparar o build dir;
//! - [`cargotoml`]: as acoes que editam `Cargo.toml`;
//! - [`error`]: o erro estruturado do dominio.
//!
//! # O que este dominio NAO faz
//!
//! Nao executa ferramenta nem duplica dominio existente (`ARCHITECTURE.md`
//! §8.1): `cargo.check` devolve o job que o `handlers/cargo.rs` ja sabe
//! disparar, e a run config e salva pelo [`crate::runconfig`], dono do
//! `.kinein/runconfigs.json`.

mod availability;
mod builddir;
mod cargotoml;
mod catalog;
mod cmakelists;
mod error;
mod plan;
mod presets;
mod remover;
mod rigor;

use std::{collections::BTreeMap, path::Path};

use kinein_protocol::{
    BuildSystem, ConfigActionApplyParams, ConfigActionEffect, ConfigActionListResult,
    ConfigActionPreviewParams, ConfigActionPreviewResult,
};

pub use error::ConfigActionError;

use availability::WorkspaceFacts;
use plan::ActionPlan;

/// O que [`apply`] conseguiu concluir dentro do dominio.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConfigActionOutcome {
    /// Efeito concluido aqui mesmo.
    Done {
        /// Frase curta para a barra de status.
        message: String,
        /// Caminhos relativos escritos.
        files: Vec<String>,
    },
    /// A acao e um job: quem tem o `JobManager` dispara o metodo devolvido.
    Job {
        /// Metodo IPC ja existente que executa o trabalho (`cargo.check`).
        method: &'static str,
    },
}

/// Lista as acoes do workspace, filtradas pelo build system ativo.
#[must_use]
pub fn list(
    root: &Path,
    build_systems: &[BuildSystem],
    include_hidden_by_scope: bool,
) -> ConfigActionListResult {
    let facts = WorkspaceFacts::measure(root);
    let actions = catalog::definitions()
        .iter()
        .filter_map(|action| {
            let in_scope = build_systems.contains(&action.scope.build_system());
            if !in_scope && !include_hidden_by_scope {
                return None;
            }
            Some(availability::describe(action, &facts, in_scope))
        })
        .collect();
    ConfigActionListResult {
        actions,
        active_build_systems: build_systems.to_vec(),
    }
}

/// Calcula o plano de uma acao sem escrever nada.
pub fn preview(
    root: &Path,
    build_systems: &[BuildSystem],
    request: &ConfigActionPreviewParams,
) -> Result<ConfigActionPreviewResult, ConfigActionError> {
    let action = resolve(&request.id, build_systems)?;
    let plan = build_plan(root, &request.id, &request.params)?;
    Ok(ConfigActionPreviewResult {
        id: request.id.clone(),
        title: action.title.to_owned(),
        summary: plan.summary.clone(),
        files: plan.file_changes(),
        report: plan.report,
        notes: plan.notes,
    })
}

/// Recalcula o plano e o grava, exigindo o snapshot visto no preview.
pub fn apply(
    root: &Path,
    build_systems: &[BuildSystem],
    request: &ConfigActionApplyParams,
) -> Result<ConfigActionOutcome, ConfigActionError> {
    let action = resolve(&request.id, build_systems)?;

    if action.effect == ConfigActionEffect::Job {
        // Um job nao nasce aqui: o dominio nao alcanca o `JobManager`
        // (arquitetura/04 §3). Quem tem o canal de eventos e o handler.
        return Ok(ConfigActionOutcome::Job {
            method: "cargo.check",
        });
    }

    let plan = build_plan(root, &request.id, &request.params)?;

    if action.effect == ConfigActionEffect::Inspect {
        return Ok(ConfigActionOutcome::Done {
            message: plan.summary,
            files: Vec::new(),
        });
    }

    if action.effect == ConfigActionEffect::Delegate {
        return delegate(root, &request.id, &request.params);
    }

    let snapshot = |path: &str| {
        request
            .expected
            .iter()
            .find(|expected| expected.path == path)
            .map(|expected| expected.content.clone())
    };
    let files = plan::apply_files(root, &plan.files, &snapshot)?;
    Ok(ConfigActionOutcome::Done {
        message: plan.summary,
        files,
    })
}

/// Efeitos que nao sao edicao de texto e vivem em outro dono dentro do core.
fn delegate(
    root: &Path,
    id: &str,
    params: &BTreeMap<String, String>,
) -> Result<ConfigActionOutcome, ConfigActionError> {
    match id {
        "cmake.repairBuildDir" => Ok(ConfigActionOutcome::Done {
            message: builddir::repair(root)?,
            files: Vec::new(),
        }),
        "cargo.createRunConfig" => {
            let name = required_param(params, "name")?;
            let command = required_param(params, "command")?;
            crate::runconfig::save(root, None, name, command)
                .map_err(|reason| ConfigActionError::NotApplicable { reason })?;
            Ok(ConfigActionOutcome::Done {
                message: format!("Run config '{name}' criada e ativada"),
                files: vec![".kinein/runconfigs.json".to_owned()],
            })
        }
        _ => Err(ConfigActionError::UnknownAction { id: id.to_owned() }),
    }
}

/// A acao existe e o build system dela esta ativo?
fn resolve(
    id: &str,
    build_systems: &[BuildSystem],
) -> Result<&'static catalog::ActionDefinition, ConfigActionError> {
    let action = catalog::definition(id)
        .ok_or_else(|| ConfigActionError::UnknownAction { id: id.to_owned() })?;
    if !build_systems.contains(&action.scope.build_system()) {
        return Err(ConfigActionError::OutOfScope { id: id.to_owned() });
    }
    Ok(action)
}

/// Despacha para o planejador do arquivo que a acao edita.
fn build_plan(
    root: &Path,
    id: &str,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    match id {
        "cmake.enableCompileCommands" => cmakelists::enable_compile_commands(root),
        "cmake.createDebugPreset" => {
            presets::create_configure_preset(root, "debug", "Debug", "Debug")
        }
        "cmake.createReleasePreset" => {
            presets::create_configure_preset(root, "release", "Release", "Release")
        }
        "cmake.addExecutable" => cmakelists::add_executable(root, params),
        "cmake.addStaticLibrary" => cmakelists::add_static_library(root, params),
        "cmake.addSourceToTarget" => cmakelists::add_source_to_target(root, params),
        "cmake.addIncludeDirectory" => cmakelists::add_include_directory(root, params),
        "cmake.findPackage" => cmakelists::find_package(root, params),
        "cmake.fetchContent" => cmakelists::fetch_content(root, params),
        "cmake.addTargetLinkLibraries" => cmakelists::add_target_link_libraries(root, params),
        "cmake.removeTargetLinkLibraries" => remover::remove_target_link_libraries(root, params),
        "cmake.strictWarnings" => rigor::strict_warnings(root, params),
        "cmake.setCxxStandard" => rigor::cxx_standard(root, params),
        "cmake.enableSanitizers" => rigor::sanitizers(root, params),
        "cmake.enableOpenMP" => rigor::openmp(root, params),
        "cmake.generateHexBin" => rigor::hex_and_bin(root, params),
        "cargo.embeddedTarget" => rigor::cargo_embedded_target(root, params),
        "cmake.inspectCache" => builddir::inspect_cache(root),
        "cmake.repairBuildDir" => builddir::repair_plan(root),
        "cargo.addDependency" => cargotoml::add_dependency(root, params, false),
        "cargo.addDevDependency" => cargotoml::add_dependency(root, params, true),
        "cargo.addFeature" => cargotoml::add_feature(root, params),
        "cargo.setEdition" => cargotoml::set_edition(root, params),
        "cargo.check" => Ok(
            ActionPlan::effect("Roda cargo check --workspace --all-targets").with_note(
                "Nao altera arquivo nenhum. Os erros aparecem na aba Problemas, \
             pelo mesmo caminho do clippy."
                    .to_owned(),
            ),
        ),
        "cargo.createRunConfig" => run_config_plan(root, params),
        _ => Err(ConfigActionError::UnknownAction { id: id.to_owned() }),
    }
}

fn run_config_plan(
    root: &Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let name = required_param(params, "name")?.trim();
    let command = required_param(params, "command")?.trim();
    if name.is_empty() {
        return Err(ConfigActionError::MissingParam { name: "name" });
    }
    if command.is_empty() {
        return Err(ConfigActionError::MissingParam { name: "command" });
    }
    let existing = crate::runconfig::load(root).configs.len();
    Ok(ActionPlan::effect(format!(
        "Salva a run config '{name}' com o comando '{command}'"
    ))
    .with_note(format!(
        "Escrita pelo dominio de run configs em .kinein/runconfigs.json \
         ({existing} config(s) hoje); a nova vira a ATIVA."
    )))
}

/// Valor obrigatorio de um parametro, ja rejeitando vazio.
pub(crate) fn required_param<'params>(
    params: &'params BTreeMap<String, String>,
    name: &'static str,
) -> Result<&'params str, ConfigActionError> {
    params
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigActionError::MissingParam { name })
}

/// Valor opcional de um parametro; vazio conta como ausente.
pub(crate) fn optional_param<'params>(
    params: &'params BTreeMap<String, String>,
    name: &str,
) -> Option<&'params str> {
    params
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}
