//! Bounded before/after compiler evidence for the existing LSP edit consumer.

// Crate-internal consumer shared across domains; public export conflicts with unreachable_pub.
#![allow(
    clippy::redundant_pub_crate,
    reason = "Keep shared verifier details internal to the core"
)]

mod command;
mod dependencies;
mod execute;
mod mapping;
mod prepare;
mod scratch;
use scratch::Scratch;

use crate::cdb::{VerificationContext, verification::ContextGuard};
use kinein_protocol::{EditVerification, EditVerificationStatus};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct VerificationRun {
    pub evidence: EditVerification,
    pub guard: Option<ContextGuard>,
}

/// Immutable document identity for one proposed correction.
#[derive(Clone, Copy)]
pub(crate) struct Comparison<'a> {
    pub root: &'a Path,
    pub path: &'a Path,
    pub before: &'a str,
    pub after: &'a str,
    pub version: Option<i64>,
    pub edits: &'a [(usize, usize, usize)],
}

/// No project write and no compiler substitution. Call only for trusted workspaces.
pub(crate) fn verify(
    input: Comparison<'_>,
    cancel: &Arc<AtomicBool>,
    budget: Duration,
    prepare: impl FnOnce() -> Result<VerificationContext, String>,
) -> VerificationRun {
    let started = Instant::now();
    let mut evidence = EditVerification::not_verified("Evidência incompleta.");
    evidence.document_version = input.version;
    let mut guard = None;
    if let Err(reason) = compare(
        &input,
        cancel,
        started + budget,
        &mut evidence,
        &mut guard,
        prepare,
    ) {
        evidence.status = EditVerificationStatus::NotVerified;
        evidence.reason = reason;
    }
    evidence.obsolete = cancel.load(Ordering::SeqCst)
        || guard
            .as_ref()
            .is_some_and(|g| !g.is_current_before(started + budget));
    if evidence.obsolete {
        evidence.status = EditVerificationStatus::NotVerified;
    }
    evidence.duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    VerificationRun { evidence, guard }
}

fn compare(
    input: &Comparison<'_>,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
    evidence: &mut EditVerification,
    guard: &mut Option<ContextGuard>,
    prepare: impl FnOnce() -> Result<VerificationContext, String>,
) -> Result<(), String> {
    let Comparison {
        path,
        before,
        after,
        ..
    } = *input;
    if before.len().max(after.len()) > 1_048_576 {
        return Err("Unidade excede 1 MiB.".into());
    }
    if cancel.load(Ordering::SeqCst) {
        return Err("Verificação cancelada.".into());
    }
    let mapping = mapping::Mapping::new(path, before, after, input.edits)?;
    let prepare::Prepared {
        check,
        source,
        sandbox,
    } = prepare::prepare(input, prepare()?, deadline, evidence, guard)?;
    let guard = guard.as_mut().ok_or("Contexto não capturado.")?;
    let scratch = Scratch::new()?;
    let execute = |content: &str, guard: &mut ContextGuard| {
        let mappings = scratch.mappings(path, &source, content, guard)?;
        let overlay = scratch.overlay(&mappings)?;
        execute::run(
            &execute::Invocation {
                check: &check,
                overlay: &overlay,
                mappings: &mappings,
                sandbox: sandbox.as_deref(),
                dependency: &scratch.dependency_file(),
                original: path,
            },
            cancel,
            deadline,
            guard,
        )
    };
    let probe_errors = execute("#error kinein_vfs_correction_probe\n", guard)?;
    if !probe_errors.errors.iter().any(|d| {
        d.file.as_deref() == path.to_str() && d.message.contains("kinein_vfs_correction_probe")
    }) {
        return Err("Ferramenta incompatível: leitura do documento isolado não comprovada.".into());
    }
    let (before_errors, after_errors) = loop {
        let generation = guard.frozen_count();
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let before_errors = execute(before, guard)?;
        let after_errors = execute(after, guard)?;
        let same_clock = std::env::var_os("SOURCE_DATE_EPOCH").is_some()
            || clock
                == std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
        if !guard.is_current_before(deadline) {
            return Err("Contexto alterado ou prazo excedido.".into());
        }
        if generation == guard.frozen_count() && same_clock {
            break (before_errors, after_errors);
        }
    };
    evidence.before_errors = Some(before_errors.errors.len() as u64);
    evidence.after_errors = Some(after_errors.errors.len() as u64);
    evidence.new_errors = mapping.additional(&before_errors.errors, &after_errors.errors);
    if evidence.new_errors.is_empty() && !after_errors.complete {
        return Err(
            "O compilador interrompeu a análise: não há evidência completa para comparar.".into(),
        );
    }
    evidence.status = if evidence.new_errors.is_empty() {
        EditVerificationStatus::NoNewError
    } else {
        EditVerificationStatus::NewError
    };
    evidence.reason =
        "Comparação de diagnósticos com as flags da CDB. Não prova equivalência, link ou testes."
            .into();
    evidence.new_errors.truncate(32);
    Ok(())
}

#[cfg(test)]
mod tests;
