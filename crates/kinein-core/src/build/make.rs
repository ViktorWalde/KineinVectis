//! Makefile puro (P0 do 40 §4.1, 2026-09-17): `make` com o `bear` na frente.
//!
//! Submodulo proprio pela regra da `ARCHITECTURE.md` §4: um build system e'
//! uma responsabilidade; o `mod.rs` roteia por `ProjectKind`. O que mora
//! aqui e' so' a linha do make e a decisao "com bear ou a seco, dito".

use std::{
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use super::{BuildError, BuildEvent, BuildOutcome, DiagnosticFormat, stream_command};
use std::path::Path;

/// O que um Makefile puro precisa (P0 do 40 §4.1, 2026-09-17).
///
/// O `make` e, quando existe, o `bear` — que grava a `compile_commands.json`
/// que o clangd le. Resolvidos pelo handler (o job nao alcanca o detector).
#[derive(Debug, Clone, Default)]
pub struct MakeTools {
    /// O `make` detectado; `None` = deixar o `PATH` resolver `make`.
    pub make: Option<std::path::PathBuf>,
    /// O `bear` detectado; `None` = build sem CDB, e o evento diz o passo.
    pub bear: Option<std::path::PathBuf>,
}

/// Makefile puro: `bear -- make` (Bear 3, README: "bear -- <your build
/// command>", conferido em 2026-09-17) quando o `bear` existe — a CDB sai na
/// raiz, onde o clangd a acha sozinho —, senao `make` a seco, com a linha
/// dizendo que sem `bear` nao ha' analise (o passo esta' no painel de
/// instalacao). Nunca um `make` "diferente" para fingir CDB.
pub(super) fn run_make_build(
    root: &Path,
    tools: &MakeTools,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let make = tools
        .make
        .clone()
        .unwrap_or_else(|| std::path::PathBuf::from("make"));
    let sem_bear = "sem `bear` nesta maquina: o make roda, mas nao ha' compile_commands.json \
                    para o clangd (instale o bear pelo painel Instalar ferramentas e compile \
                    de novo)";
    let (mut command, display) = tools.bear.as_ref().map_or_else(
        || {
            sink(BuildEvent::Output {
                stream: "stderr",
                line: sem_bear.to_owned(),
            });
            (Command::new(&make), "make")
        },
        |bear| {
            let mut command = Command::new(bear);
            command.arg("--").arg(&make);
            (command, "bear -- make")
        },
    );
    command.current_dir(root);
    stream_command(command, display, DiagnosticFormat::GccLike, cancel, sink)
}
