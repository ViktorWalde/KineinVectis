//! O build de um projeto `CMake`: configure quando precisa (sem cache, ou
//! com outro perfil de rigor, `crate::cmake::needs_configure`) e
//! `cmake --build` na pasta da configuracao ativa (C1-21).

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

fn refused(reason: String) -> BuildError {
    BuildError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        reason,
    ))
}

pub(super) fn run_cmake_build(
    root: &Path,
    profile: RigorProfile,
    toolchain: &Toolchain,
    extra: &[String],
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    // Sem configuracao ativa, o build recusa com o motivo do configure, sem
    // voltar a `.kinein/build` em silencio.
    let mut build_dir = crate::cmake::active_dir(root).map_err(refused)?;
    crate::cmake::write_file_api_query(&build_dir).map_err(BuildError::Io)?;

    // Configura sem cache E quando o perfil de rigor mudou: o perfil vive no
    // cache do CMake, e so' um configure novo o troca.
    if crate::cmake::needs_configure(&build_dir, profile) {
        if let Some(before) = crate::cmake::applied_rigor(&build_dir) {
            sink(BuildEvent::Output {
                stream: "stdout",
                line: format!(
                    "perfil de rigor mudou ({before} -> {}): reconfigurando",
                    crate::cmake::rigor_key(profile)
                ),
            });
        }
        let (preset, _) = crate::cmake::choose_preset(root, None)
            .and_then(|choice| toolchain.unavailable_compiler().map_or(Ok(choice), Err))
            .map_err(refused)?;
        // O configure vai para a pasta da configuracao do preset: o
        // `.kinein/build` de antes do C-04 vale ate' o proximo configure.
        build_dir = crate::cmake::configuration_dir(root, preset.as_deref()).map_err(refused)?;
        crate::cmake::write_file_api_query(&build_dir).map_err(BuildError::Io)?;
        let attempt = crate::cmake::begin_configure(
            root,
            &build_dir,
            preset.as_deref(),
            &toolchain.to_result().preset,
        )
        .map_err(BuildError::Io)?;
        for line in crate::cmake::configure_notes(root, &build_dir, preset.as_deref()) {
            sink(BuildEvent::Output {
                stream: "stdout",
                line,
            });
        }
        let configure = || {
            crate::cmake::configure_with_rigor(
                root,
                &build_dir,
                preset.as_deref(),
                toolchain,
                extra,
                profile,
            )
        };

        let mut outcome = stream_command(
            configure(),
            "cmake (configure)",
            DiagnosticFormat::GccLike,
            cancel,
            sink,
        )?;
        // C1-14: o cache recriado sem as definicoes da IDE nao gera a CDB.
        if outcome.success && crate::cmake::attempt::lost_ide_definitions(&build_dir) {
            sink(BuildEvent::Output {
                stream: "stdout",
                line: crate::cmake::attempt::DEFINITIONS_LOST.to_owned(),
            });
            outcome = stream_command(
                configure(),
                "cmake (configure)",
                DiagnosticFormat::GccLike,
                cancel,
                sink,
            )?;
            outcome.success =
                outcome.success && !crate::cmake::attempt::lost_ide_definitions(&build_dir);
        }
        if !outcome.success {
            return Ok(outcome);
        }
        crate::cmake::record_rigor(&build_dir, profile);
        crate::cmake::record_preset(&build_dir, preset.as_deref());
        crate::cmake::complete_configure(attempt).map_err(BuildError::Io)?;
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
