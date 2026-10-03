//! O build de um projeto `CMake`: configure quando precisa (sem cache, ou
//! com outro perfil de rigor, `crate::cmake::needs_configure`) e
//! `cmake --build` no diretorio canonico.

use std::{
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{RigorProfile, ToolchainRole};

use super::{BuildError, BuildEvent, BuildOutcome, DiagnosticFormat, programa, stream_command};
use crate::toolchain::Toolchain;

pub(super) fn run_cmake_build(
    root: &Path,
    profile: RigorProfile,
    toolchain: &Toolchain,
    extra: &[String],
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let build_dir = crate::cmake::build_dir(root);

    // Configura sem cache E quando o perfil de rigor mudou: o perfil vive no
    // cache do CMake, e so' um configure novo o troca.
    if crate::cmake::needs_configure(root, profile) {
        if let Some(before) = crate::cmake::applied_rigor(root) {
            sink(BuildEvent::Output {
                stream: "stdout",
                line: format!(
                    "perfil de rigor mudou ({before} -> {}): reconfigurando",
                    crate::cmake::rigor_key(profile)
                ),
            });
        }
        let preset = crate::cmake::status(root).preset;
        let configure =
            crate::cmake::configure_with_rigor(root, preset.as_deref(), toolchain, extra, profile);

        let outcome = stream_command(
            configure,
            "cmake (configure)",
            DiagnosticFormat::GccLike,
            cancel,
            sink,
        )?;
        if !outcome.success {
            return Ok(outcome);
        }
        crate::cmake::record_rigor(root, profile);
    }

    let mut build = Command::new(programa(toolchain, ToolchainRole::Cmake, "cmake"));
    build.arg("--build").arg(&build_dir);

    stream_command(
        build,
        "cmake --build",
        DiagnosticFormat::GccLike,
        cancel,
        sink,
    )
}
