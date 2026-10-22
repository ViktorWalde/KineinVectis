//! probe-rs 0.32 queries for native flashing: version, chip cores, probe
//! listing and the inherited options the tool would apply. Bounded by the
//! probe query owner; nothing here opens a probe or writes a chip.
use crate::{
    dap::gdb_pick::ElfMachine,
    flash::FlashError,
    probe::query::{self, Failure},
};
use kinein_protocol::ProbeInfo;
use serde_json::Value;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

const DEADLINE: Duration = Duration::from_secs(5);
/// Every option probe-rs 0.32 inherits from the environment uses this prefix
/// (`util/common_options.rs` and `main.rs` of the release source).
const INHERITED_PREFIX: &[u8] = b"PROBE_RS_";
/// `main.rs::load_config` of 0.32.0 merges `.probe-rs.<extension>`.
const CONFIGURATION_EXTENSIONS: [&str; 4] = ["toml", "json", "yaml", "yml"];

/// Names of the inherited `PROBE_RS_*` variables, sorted: they are removed
/// from every probe-rs process the native flow starts, queries included.
#[must_use]
pub fn inherited_variables() -> Vec<OsString> {
    let mut names: Vec<OsString> = std::env::vars_os()
        .map(|(name, _)| name)
        .filter(|name| name.as_encoded_bytes().starts_with(INHERITED_PREFIX))
        .collect();
    names.sort();
    names
}

/// The `.probe-rs.*` files 0.32 would load: the working directory (the root,
/// as in the PTY), the executable's folder (given and resolved) and home.
pub(super) fn configuration_files(root: &Path, tool: &Path) -> Vec<PathBuf> {
    let mut folders = vec![root.to_path_buf()];
    folders.extend(tool.parent().map(Path::to_path_buf));
    folders.extend(
        crate::platform::canonicalize(tool)
            .ok()
            .and_then(|resolved| resolved.parent().map(Path::to_path_buf)),
    );
    folders.extend(crate::platform::home_dir());
    folders.sort();
    folders.dedup();
    folders
        .iter()
        .flat_map(|folder| {
            CONFIGURATION_EXTENSIONS
                .iter()
                .map(move |extension| folder.join(format!(".probe-rs.{extension}")))
        })
        .filter(|path| std::fs::symlink_metadata(path).is_ok())
        .collect()
}

fn run(
    tool: &Path,
    root: &Path,
    args: &[&str],
    cancel: &Arc<AtomicBool>,
) -> Result<String, FlashError> {
    let mut command = Command::new(tool);
    command.args(args).current_dir(root).env("NO_COLOR", "1");
    for name in inherited_variables() {
        command.env_remove(name);
    }
    let answer = query::run(command, DEADLINE, cancel);
    match answer.failure {
        None => Ok(answer.output),
        Some(Failure::Spawn(error)) => Err(unsupported(&format!(
            "{} não pôde iniciar: {error}",
            tool.display()
        ))),
        Some(Failure::Incomplete(reason)) => Err(unsupported(&format!(
            "`probe-rs {}` não confirmou ({reason}): {}",
            args.join(" "),
            last_lines(&answer.output)
        ))),
    }
}

/// `mi meta` must report exactly one 0.32.x version.
pub(super) fn version(
    tool: &Path,
    root: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<String, FlashError> {
    let output = run(tool, root, &["mi", "meta"], cancel)?;
    let versions: Vec<String> = output
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
        .filter_map(|meta| meta.get("version")?.as_str().map(str::to_owned))
        .collect();
    let [version] = versions.as_slice() else {
        return Err(unsupported(
            "`probe-rs mi meta` não respondeu uma única versão em JSON; confira o adaptador de depuração do kit",
        ));
    };
    if !supported(version) {
        return Err(unsupported(&format!(
            "probe-rs {version}: a gravação nativa foi validada só com a 0.32.x; use essa versão ou o modo manual"
        )));
    }
    Ok(version.clone())
}

fn supported(version: &str) -> bool {
    let mut parts = version.split('.');
    parts.next() == Some("0")
        && parts.next() == Some("32")
        && parts
            .next()
            .is_some_and(|patch| !patch.is_empty() && patch.bytes().all(|b| b.is_ascii_digit()))
        && parts.next().is_none()
}

/// Core architectures `chip info` declares for `chip` (no hardware needed).
pub(super) fn cores(
    tool: &Path,
    root: &Path,
    chip: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<String>, FlashError> {
    let output = run(tool, root, &["chip", "info", chip], cancel)?;
    parse_cores(&output).ok_or_else(|| {
        unsupported(&format!(
            "`probe-rs chip info {chip}` respondeu fora do formato da 0.32.0"
        ))
    })
}

/// `Cores (N):` followed by N lines `    - <name> (<CoreType>)`.
fn parse_cores(output: &str) -> Option<Vec<String>> {
    let mut lines = output.lines();
    let count: usize = lines.by_ref().find_map(|line| {
        line.trim()
            .strip_prefix("Cores (")?
            .strip_suffix("):")?
            .parse()
            .ok()
    })?;
    let cores = lines
        .take(count)
        .map(|line| {
            let (_, kind) = line
                .trim()
                .strip_prefix("- ")?
                .strip_suffix(')')?
                .rsplit_once(" (")?;
            (!kind.is_empty()).then(|| kind.to_owned())
        })
        .collect::<Option<Vec<_>>>()?;
    (count > 0 && cores.len() == count).then_some(cores)
}

/// Whether an ELF for `machine` runs on a 0.32 `CoreType`.
pub(super) fn runs_on(machine: ElfMachine, core: &str) -> bool {
    match core {
        "Armv6m" | "Armv7a" | "Armv7r" | "Armv7m" | "Armv7em" | "Armv8m" => {
            machine == ElfMachine::Arm
        }
        "Armv8a" => matches!(machine, ElfMachine::Arm | ElfMachine::AArch64),
        "Riscv" | "Riscv64" => machine == ElfMachine::RiscV,
        "Xtensa" => machine == ElfMachine::Xtensa,
        _ => false,
    }
}

/// Every listed probe, in the strict 0.32.0 format.
pub(super) fn listed(
    tool: &Path,
    root: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<ProbeInfo>, FlashError> {
    let output = run(tool, root, &["list"], cancel)?;
    crate::probe::parse_listing(&output).map_err(|message| unsupported(&message))
}

fn last_lines(output: &str) -> String {
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let tail = lines[lines.len().saturating_sub(3)..].join(" / ");
    tail.chars().take(400).collect()
}

fn unsupported(message: &str) -> FlashError {
    FlashError::UnsupportedTool {
        message: format!(
            "{}. Nenhuma gravação foi iniciada.",
            message.trim_end_matches('.')
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{ElfMachine, parse_cores, runs_on, supported};

    /// `probe-rs chip info STM32F411RETx`, 0.32.0 installed on 2026-10-22.
    const STM32F411: &str = "STM32F411RETx\nCores (1):\n    - main (Armv7em)\nNVM: 0x08000000..0x08080000 (512.0 KiB)\nRAM: 0x20000000..0x20020000 (128.0 KiB)\n";

    #[test]
    fn chip_info_cores_follow_the_measured_format() {
        assert_eq!(parse_cores(STM32F411), Some(vec!["Armv7em".to_owned()]));
        assert_eq!(
            parse_cores("rp235x\nCores (2):\n    - core0 (Armv8m)\n    - core1 (Riscv)\n"),
            Some(vec!["Armv8m".to_owned(), "Riscv".to_owned()])
        );
        for broken in [
            "Cores (2):\n    - core0 (Armv8m)\n",
            "Cores (0):\n",
            "Cores (1):\n    core0 Armv7em\n",
            "Error: The requested chip 'X' was not found",
        ] {
            assert_eq!(parse_cores(broken), None, "{broken}");
        }
    }

    #[test]
    fn elf_architecture_must_match_a_core() {
        assert!(runs_on(ElfMachine::Arm, "Armv7em"));
        assert!(runs_on(ElfMachine::AArch64, "Armv8a"));
        assert!(runs_on(ElfMachine::RiscV, "Riscv"));
        assert!(!runs_on(ElfMachine::Arm, "Riscv"));
        assert!(!runs_on(ElfMachine::Xtensa, "Armv7m"));
        assert!(!runs_on(ElfMachine::X86_64, "Armv7m"));
        assert!(!runs_on(ElfMachine::Arm, "Unknown"));
    }

    #[test]
    fn only_the_measured_minor_is_supported() {
        assert!(supported("0.32.0"));
        assert!(supported("0.32.7"));
        for other in [
            "0.31.0",
            "0.33.0",
            "1.32.0",
            "0.32",
            "0.32.0-rc1",
            "0.32.0.1",
            "",
        ] {
            assert!(!supported(other), "{other}");
        }
    }
}
