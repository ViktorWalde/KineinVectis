//! O kit exportado como toolchain file (0.4.1a, `roadmaps/52` §14.7) e a
//! instalacao do compilador C fixada no kit (consolidacao C §15). O conteudo
//! e' do dono `toolchain` e chega pelo handler ([`super::Inputs`]); aqui mora
//! so' o plano, com a previa e o consentimento de toda acao.

use std::path::Path;

use super::Inputs;
use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_optional};

/// Grava `.kinein/toolchains/<kit>.cmake` com os valores do configure.
pub(super) fn export_plan(root: &Path, inputs: &Inputs) -> Result<ActionPlan, ConfigActionError> {
    let exported = match &inputs.kit_export {
        Some(Ok(exported)) => exported,
        Some(Err(reason)) => {
            return Err(ConfigActionError::NotApplicable {
                reason: reason.clone(),
            });
        }
        None => {
            return Err(ConfigActionError::NotApplicable {
                reason: "o kit do projeto não foi lido".to_owned(),
            });
        }
    };
    let path = exported.path.clone();
    let before = read_optional(root, &path)?;
    Ok(ActionPlan::edit(
        format!("Exporta o kit para {path}"),
        PlannedFile {
            path: path.clone(),
            before,
            after: exported.contents.clone(),
        },
    )
    .with_note(format!(
        "No terminal ou no CI: cmake --toolchain {path} -S . -B <build>. A IDE não lê o \
         arquivo: o kit continua a fonte, e exportar de novo o atualiza."
    )))
}

/// Grava em `.kinein/toolchain.json` a escolha que o `toolchain` planejou.
pub(super) fn compiler_plan(inputs: &Inputs) -> Result<ActionPlan, ConfigActionError> {
    let choice = match &inputs.compiler_choice {
        Some(Ok(choice)) => choice,
        Some(Err(reason)) => {
            return Err(ConfigActionError::InvalidParam {
                name: "compiler",
                reason: reason.clone(),
            });
        }
        None => {
            return Err(ConfigActionError::NotApplicable {
                reason: "a instalação do compilador C não foi conferida".to_owned(),
            });
        }
    };
    let mut plan = ActionPlan::edit(
        choice.summary.clone(),
        PlannedFile {
            path: choice.path.clone(),
            before: choice.before.clone(),
            after: choice.after.clone(),
        },
    );
    plan.notes.clone_from(&choice.notes);
    Ok(plan)
}
