//! O build de um projeto `CMake`: configure quando precisa (sem cache, ou
//! com outro perfil de rigor, `crate::cmake::needs_configure`) e
//! `cmake --build` no diretorio canonico.

use std::{
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{RigorProfile, ToolchainRole};

use super::{
    BuildError, BuildEvent, BuildOutcome, DiagnosticFormat, program_for_role, stream_command,
};
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
    crate::cmake::write_file_api_query(root).map_err(BuildError::Io)?;

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
        let (preset, _) = crate::cmake::choose_preset(root, None).map_err(|reason| {
            BuildError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                reason,
            ))
        })?;
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
        crate::cmake::record_preset(root, preset.as_deref());
    }

    let mut build = Command::new(program_for_role(toolchain, ToolchainRole::Cmake, "cmake"));
    build.arg("--build").arg(&build_dir);
    crate::msvc::apply_environment(&mut build);

    stream_command(
        build,
        "cmake --build",
        DiagnosticFormat::GccLike,
        cancel,
        sink,
    )
}
