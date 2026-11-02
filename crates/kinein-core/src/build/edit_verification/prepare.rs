//! Resolve the exact compiler and capture its context before any compilation.
use super::{Comparison, command};
use crate::{
    build::syntax_check::SyntaxCheck,
    cdb::{
        VerificationContext,
        verification::{self as context, ContextGuard},
    },
};
use kinein_protocol::EditVerification;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
pub(super) struct Prepared {
    pub check: SyntaxCheck,
    pub source: PathBuf,
    pub sandbox: Option<PathBuf>,
}
pub(super) fn prepare(
    input: &Comparison<'_>,
    context: VerificationContext,
    deadline: Instant,
    evidence: &mut EditVerification,
    guard: &mut Option<ContextGuard>,
) -> Result<Prepared, String> {
    let VerificationContext { unit, cdb, bytes } = context;
    let path = input.path;
    let command::Check {
        command: mut check,
        backend,
        source,
    } = command::command(&unit, path)?;
    evidence.compiler = Some(check.program.clone());
    evidence.arguments.clone_from(&check.args);
    evidence.directory = Some(check.directory.display().to_string());
    let mut identity = bytes.clone();
    identity.extend(
        serde_json::to_vec(&(&check.program, &check.args, &unit.directory))
            .map_err(|e| e.to_string())?,
    );
    evidence.context_id = Some(context::digest(&identity));
    let guard = guard.insert(ContextGuard::new(input.root, cdb, &bytes, deadline));
    guard.observe(path)?;
    if let Some(parent) = source.parent() {
        guard.observe_directory(parent)?;
    }
    for directory in &unit.includes {
        guard.observe_directory(Path::new(directory))?;
    }
    let mut parent = path.parent();
    while let Some(directory) = parent.filter(|p| p.starts_with(input.root)) {
        for name in [
            "CMakeLists.txt",
            "CMakePresets.json",
            "CMakeUserPresets.json",
            "meson.build",
            "Makefile",
        ] {
            let config = directory.join(name);
            if config.is_file() {
                guard.observe(&config)?;
            }
        }
        parent = directory.parent();
    }
    let compiler = if Path::new(&check.program).is_absolute() || check.program.contains('/') {
        let candidate = check.directory.join(&check.program);
        candidate.is_file().then_some(candidate)
    } else {
        crate::tools::ToolDetector::with_search_path(std::env::var_os("PATH").unwrap_or_default())
            .find_in_path(&check.program)
    };
    let compiler = compiler.ok_or("Compilador ausente ou indisponível.")?;
    guard.observe(&compiler)?;
    check.program = compiler.display().to_string();
    let sandbox = if backend == command::Backend::Gcc {
        Some(
            crate::tools::ToolDetector::with_search_path(
                std::env::var_os("PATH").unwrap_or_default(),
            )
            .find_in_path("bwrap")
            .ok_or("O isolamento do GCC requer bubblewrap disponível no PATH.")?,
        )
    } else {
        None
    };
    Ok(Prepared {
        check,
        source,
        sandbox,
    })
}
