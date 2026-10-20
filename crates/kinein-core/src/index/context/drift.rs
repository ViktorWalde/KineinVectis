//! Explicit kit/unit differences. Unknown values are never evidence, and
//! comparing options does not establish target ABI compatibility.

use std::path::{Component, Path};

use kinein_protocol::{
    ContextDrift, ContextDriftAction, ContextDriftKind, ContextOrigin, FileContext, SourcedValue,
};

use super::KitFacts;

/// Compare the translation unit before kit fallbacks fill missing values.
#[must_use]
pub fn compile_drift(context: &FileContext, kit: &KitFacts) -> Vec<ContextDrift> {
    let (Some(unit), Some(effective)) = (&context.unit, &context.effective) else {
        return Vec::new();
    };
    let mut drift = Vec::new();
    let mut compare = |kind, configured: Option<&str>, actual: Option<&SourcedValue>| {
        let (Some(configured), Some(actual)) = (configured, actual) else {
            return;
        };
        if configured.is_empty()
            || actual.value.is_empty()
            || !matches!(
                actual.origin,
                ContextOrigin::Cdb | ContextOrigin::CmakeFileApi
            )
        {
            return;
        }
        let differs = match kind {
            ContextDriftKind::Target => Some(configured != actual.value),
            ContextDriftKind::Sysroot => {
                different_roots(configured, &actual.value, &unit.directory)
            }
            ContextDriftKind::Compiler => {
                different_compilers(configured, &actual.value, &unit.directory)
            }
        };
        if differs != Some(true) {
            return;
        }
        let evidence = match kind {
            ContextDriftKind::Compiler => {
                format!("Compilador declarado pela unidade em {}.", unit.directory)
            }
            ContextDriftKind::Target => {
                "Opção --target/-target explícita da unidade; não é uma verificação de ABI."
                    .to_owned()
            }
            ContextDriftKind::Sysroot => format!(
                "Sysroot explícito da unidade; caminhos relativos usam {}.",
                unit.directory
            ),
        };
        drift.push(ContextDrift {
            kind,
            configured: configured.to_owned(),
            effective: actual.clone(),
            evidence,
            action: Some(ContextDriftAction::InspectKit),
        });
    };
    compare(
        ContextDriftKind::Compiler,
        kit.pinned_compiler.as_deref(),
        effective.compiler.as_ref(),
    );
    compare(
        ContextDriftKind::Target,
        kit.target_triple.as_deref(),
        effective.target.as_ref(),
    );
    compare(
        ContextDriftKind::Sysroot,
        kit.sysroot.as_deref(),
        effective.sysroot.as_ref(),
    );
    drift
}

fn different_compilers(configured: &str, actual: &str, directory: &str) -> Option<bool> {
    let actual = Path::new(actual);
    // A bare executable is selected by the job's environment, not its cwd.
    if !actual.is_absolute() && actual.components().count() < 2 {
        return None;
    }
    let configured = Path::new(configured);
    let actual = Path::new(directory).join(actual);
    if !configured.is_absolute() || !actual.is_absolute() {
        return None;
    }
    crate::platform::same_file(configured, &actual).map(|same| !same)
}

fn different_roots(configured: &str, actual: &str, directory: &str) -> Option<bool> {
    let configured = Path::new(configured);
    let actual = Path::new(directory).join(actual);
    if !configured.is_absolute() || !actual.is_absolute() {
        return None;
    }
    if let Some(same) = crate::platform::same_file(configured, &actual) {
        return Some(!same);
    }
    // A missing path with parents could traverse a symlink; do not guess.
    if configured
        .components()
        .chain(actual.components())
        .any(|part| part == Component::ParentDir)
    {
        return None;
    }
    Some(configured != actual)
}
