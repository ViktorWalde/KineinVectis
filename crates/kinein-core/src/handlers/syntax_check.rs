//! O AGENDAMENTO da checagem do compilador ao salvar (C1, `roadmaps/52`
//! §5.2 e §14.3).
//!
//! Depois de um `fs.write` bem-sucedido de um `.c/.cc/.cpp/.cxx` com unidade
//! na `compile_commands.json`: um job curto por unidade, que espera 400 ms
//! (salvar varias vezes nao enfileira varias checagens: o salvamento novo
//! CANCELA o anterior da mesma unidade), roda o compilador do alvo com
//! `-fsyntax-only` e prazo de 10 s, e funde o resultado com o do LSP pela
//! chave `compiler` — para cada arquivo citado e para os que a unidade citou
//! da vez anterior, que e' o que apaga o erro corrigido. Falha (compilador
//! ausente, prazo) nao apaga nada: o job falha com a saida bruta.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kinein_protocol::{ContextOrigin, JobRisk};

use crate::Core;
use crate::build::syntax_check::{SyntaxCheck, diagnostics_by_file, syntax_only_command};
use crate::jobs::{JobContext, JobOutcome};
use crate::lsp::{MergedDiagnostics, record_diagnostics};

/// A espera antes de checar: salvar de novo dentro dela cancela esta.
const DEBOUNCE: Duration = Duration::from_millis(400);
/// O prazo do compilador; passou, o job falha com a saida bruta. Nos testes,
/// 1 s: o teste do prazo nao segura a trava dos executaveis por 10 s.
#[cfg(not(test))]
const DEADLINE: Duration = Duration::from_secs(10);
#[cfg(test)]
const DEADLINE: Duration = Duration::from_secs(1);
/// A chave do compilador na fusao dos diagnosticos.
const MERGE_KEY: &str = "compiler";

/// Por unidade: o job em andamento e os arquivos que a ultima checagem citou.
#[derive(Debug, Default, Clone)]
pub(crate) struct SyntaxChecks {
    running: Arc<Mutex<HashMap<String, String>>>,
    reported: Arc<Mutex<HashMap<String, BTreeSet<String>>>>,
}

impl SyntaxChecks {
    /// Troca de workspace: nada do projeto anterior fica.
    pub(crate) fn clear(&self) {
        if let Ok(mut running) = self.running.lock() {
            running.clear();
        }
        if let Ok(mut reported) = self.reported.lock() {
            reported.clear();
        }
    }
}

impl Core {
    /// Agenda a checagem de `path`, salvo agora, quando ela vale.
    pub(crate) fn schedule_syntax_check(&self, root: &Path, path: &Path) {
        let language = crate::index::language_of(path);
        // So' em projeto confiavel (16.4b): o compilador vem da CDB, e a CDB
        // pode ter vindo no repositorio, nomeando qualquer programa.
        let trusted = self.workspace.as_ref().is_some_and(|open| open.trusted);
        if !trusted
            || !matches!(language, "c" | "cpp")
            || !crate::settings::effective_compiler_check_on_save(root)
        {
            return;
        }
        let Some(check) = self
            .cdb_unit_for(root, path, language)
            .and_then(|unit| syntax_only_command(&unit))
        else {
            return;
        };
        let (Some(jobs), Some(merged)) = (
            self.jobs.as_ref(),
            self.lsp
                .as_ref()
                .map(crate::lsp::LspManager::merged_diagnostics),
        ) else {
            return;
        };
        let unit_key = path.display().to_string();
        if let Ok(mut running) = self.syntax_checks.running.lock()
            && let Some(previous) = running.remove(&unit_key)
        {
            jobs.cancel(&previous);
        }
        let reported = Arc::clone(&self.syntax_checks.reported);
        let name = path
            .file_name()
            .map_or_else(|| unit_key.clone(), |n| n.to_string_lossy().into_owned());
        let key = unit_key.clone();
        let job_id = jobs.spawn(
            "build.syntaxCheck",
            format!("Checar {name}"),
            JobRisk::Low,
            true,
            move |ctx| run_check(ctx, &check, &merged, &reported, &key),
        );
        if let Ok(mut running) = self.syntax_checks.running.lock() {
            running.insert(unit_key, job_id);
        }
    }

    /// A unidade da CDB do arquivo: so' ela traz a linha inteira do
    /// compilador (a do file-api traz fragmentos).
    fn cdb_unit_for(
        &self,
        root: &Path,
        path: &Path,
        language: &str,
    ) -> Option<kinein_protocol::CompileUnit> {
        let context = {
            let index = self.index.lock().ok()?;
            index.context.as_ref()?.for_file(root, path, language)
        };
        let from_cdb =
            context.effective.as_ref().and_then(|e| e.flags_origin) == Some(ContextOrigin::Cdb);
        from_cdb.then_some(context.unit).flatten()
    }
}

fn run_check(
    ctx: &JobContext,
    check: &SyntaxCheck,
    merged: &MergedDiagnostics,
    reported: &Mutex<HashMap<String, BTreeSet<String>>>,
    unit_key: &str,
) -> JobOutcome {
    let waited = Instant::now();
    while waited.elapsed() < DEBOUNCE {
        if ctx.is_cancelled() {
            return JobOutcome::Success;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let stop = Arc::new(AtomicBool::new(false));
    let timed_out = Arc::new(AtomicBool::new(false));
    let finished = Arc::new(AtomicBool::new(false));
    let watchdog = {
        let (stop, timed_out, finished, cancel) = (
            Arc::clone(&stop),
            Arc::clone(&timed_out),
            Arc::clone(&finished),
            ctx.cancellation(),
        );
        std::thread::spawn(move || {
            let started = Instant::now();
            while !finished.load(Ordering::SeqCst) {
                if cancel.load(Ordering::SeqCst) {
                    stop.store(true, Ordering::SeqCst);
                    break;
                }
                if started.elapsed() > DEADLINE {
                    timed_out.store(true, Ordering::SeqCst);
                    stop.store(true, Ordering::SeqCst);
                    break;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        })
    };
    let mut command = std::process::Command::new(&check.program);
    command.args(&check.args).current_dir(&check.directory);
    let mut lines = Vec::new();
    let outcome =
        crate::process::stream_command_lines_cancelable(command, &stop, &mut |_stream, line| {
            ctx.emit_output(&line);
            lines.push(line);
        });
    finished.store(true, Ordering::SeqCst);
    drop(watchdog.join());
    if ctx.is_cancelled() {
        return JobOutcome::Success;
    }
    if timed_out.load(Ordering::SeqCst) {
        ctx.emit_output(&format!(
            "a checagem passou de {} s: nada mudou nos problemas",
            DEADLINE.as_secs()
        ));
        return JobOutcome::Failed;
    }
    if let Err(error) = outcome {
        ctx.emit_output(&match &error {
            crate::process::ProcessError::Spawn(source) => {
                format!(
                    "o compilador {} não pôde ser iniciado: {source}",
                    check.program
                )
            }
            crate::process::ProcessError::Wait(source) => {
                format!("falha aguardando o compilador: {source}")
            }
        });
        return JobOutcome::Failed;
    }
    let by_file = diagnostics_by_file(&lines, &check.directory, Some(ctx.id()));
    let Ok(mut reported) = reported.lock() else {
        return JobOutcome::Failed;
    };
    let previous = reported.remove(unit_key).unwrap_or_default();
    let mut paths: BTreeSet<String> = previous;
    paths.extend(by_file.keys().cloned());
    for path in &paths {
        let diagnostics = by_file.get(path).cloned().unwrap_or_default();
        let event = record_diagnostics(merged, MERGE_KEY, path, diagnostics);
        ctx.emit_event(&event.method, event.params.unwrap_or_default());
    }
    reported.insert(unit_key.to_owned(), by_file.keys().cloned().collect());
    if by_file.is_empty() {
        JobOutcome::Success
    } else {
        JobOutcome::Warning
    }
}
