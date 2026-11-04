//! Dialetos literais e sonda limitada do driver C selecionado na criação.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

pub(super) fn valid_standard(value: &str) -> bool {
    let numeric = value.len() <= 6
        && value.starts_with(|c: char| c.is_ascii_digit() && c != '0')
        && value.bytes().all(|c| c.is_ascii_digit());
    let literal = value.len() <= 128
        && value.starts_with(|c: char| c.is_ascii_alphabetic())
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b':' | b'.' | b'-'));
    numeric || literal
}

pub(super) fn block(target: &str, standard: &str) -> String {
    if standard.bytes().all(|c| c.is_ascii_digit()) {
        crate::configaction::c_standard_block(target, standard, "OFF")
    } else {
        format!(
            "# Kinein: dialeto C literal do alvo {target}\ntarget_compile_options({target} PRIVATE -std={standard})\n"
        )
    }
}

pub(super) fn probe(executable: &Path, standard: Option<&str>) -> Result<(), String> {
    let scratch = Scratch::new().map_err(|e| format!("sonda C indisponível: {e}"))?;
    let source = scratch.0.join("probe.c");
    fs::write(
        &source,
        "#if defined(__cplusplus) || !defined(__STDC__)\n#error Kinein: selecione um driver C, não C++\n#endif\nint main(void) { return 0; }\n",
    )
    .map_err(|e| format!("sonda C indisponível: {e}"))?;
    let mut command = Command::new(executable);
    command
        .current_dir(&scratch.0)
        .args(["-fsyntax-only", "-Werror"]);
    // Numeric values belong to CMake, whose spelling may use a compiler alias.
    if let Some(dialect) = standard.filter(|value| !value.bytes().all(|c| c.is_ascii_digit())) {
        command.arg(format!("-std={dialect}"));
    }
    command.arg(&source);
    let mut diagnostic = String::new();
    let (status, expired) = crate::process::stream_command_lines_with_timeout(
        command,
        Duration::from_secs(3),
        &mut |_, line| {
            if diagnostic.len() < 2048 {
                diagnostic.extend(line.chars().take(512));
                diagnostic.push('\n');
            }
        },
    )
    .map_err(|e| format!("não foi possível testar o driver C: {e:?}"))?;
    if expired {
        return Err("driver C excedeu o prazo do teste sintático (3 s)".to_owned());
    }
    if !status.success() {
        return Err(format!(
            "driver C {} recusou a fonte C ou o padrão solicitado ({}):\n{}",
            executable.display(),
            standard.unwrap_or("automático"),
            diagnostic.trim()
        ));
    }
    Ok(())
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..16 {
            let path = std::env::temp_dir().join(format!(
                "kinein-c-probe-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match crate::platform::owner_only_dir_builder().create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::other("nenhum diretório livre para sonda C"))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
