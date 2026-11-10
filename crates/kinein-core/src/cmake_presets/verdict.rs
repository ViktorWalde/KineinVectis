//! What the `CMake` of the project says of its presets (C1-25).
//!
//! The schema of the presets changes with the `CMake`: the fields of each
//! file version, their types, the required ones and the generators it has.
//! Decided by the author on 2026-11-10 (consolidacao C §18): the IDE asks
//! `cmake --list-presets` of the `CMake` the default kit of the workspace
//! pins, else of the `cmake` on the `PATH`, and the answer wins over the
//! native reading. It holds while neither the executable nor a file the
//! reading opened changes. Measured with 4.4.5: the form without `=all`
//! leaves out a preset whose generator this `CMake` does not have, as
//! `cmake.presets.list` must.

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock, PoisonError},
    time::SystemTime,
};

use kinein_protocol::ToolchainRole;

use super::{
    expansion::inherited,
    files::{Kind, Presets},
    reading::Reading,
};

/// What `cmake --list-presets` answered.
#[derive(Debug, Clone)]
enum Answer {
    /// The lines of the list without the indent: `"name"`, or `"name"`, the
    /// spaces that align the column, and `- displayName`.
    Listed(Vec<String>),
    /// Why it refused the files.
    Refused(String),
}

/// An answer holds for the executable and the files as they were.
#[derive(PartialEq, Eq)]
struct Key {
    program: PathBuf,
    stamp: Option<(u64, SystemTime)>,
    opened: Vec<(PathBuf, Option<u64>)>,
}

/// The last answer for each root.
static ANSWERS: OnceLock<Mutex<HashMap<PathBuf, (Key, Answer)>>> = OnceLock::new();

/// The `CMake` that answers: the one the default kit pins, else `cmake` on
/// the `PATH`, as the configure without a kit.
pub(super) fn program(root: &Path) -> Option<PathBuf> {
    program_on(root, std::env::var_os("PATH").unwrap_or_default())
}

/// Idem, with `path` in place of the `PATH` of the core.
fn program_on(root: &Path, path: OsString) -> Option<PathBuf> {
    crate::kits::pinned_program(root, "", ToolchainRole::Cmake)
        .or_else(|| crate::tools::ToolDetector::with_search_path(path).find_in_path("cmake"))
}

/// The presets as the `CMake` of the project reads them. Its refusal refuses
/// them all; when it lists them, the native reading gives the values and
/// learns what it leaves out. Without presets files, or without a `CMake`
/// that runs, the native reading alone.
pub(super) fn apply(
    root: &Path,
    native: Result<(Presets, Reading), String>,
    opened: &[(PathBuf, Option<u64>)],
    program: Option<&Path>,
) -> Result<(Presets, Reading), String> {
    if opened.iter().all(|(_, text)| text.is_none()) {
        return native;
    }
    let Some(answer) = program.and_then(|program| answer(root, program, opened)) else {
        return native;
    };
    let lines = match answer {
        Answer::Refused(reason) => return Err(format!("o CMake recusa os presets: {reason}")),
        Answer::Listed(lines) => lines,
    };
    let (presets, mut reading) = native
        .map_err(|reason| format!("o CMake aceita os presets, mas a IDE não os lê: {reason}"))?;
    // The list is the one of the `CMake`, also where the native reading
    // took a `condition` for false.
    reading
        .disabled
        .retain(|name| !lines.iter().any(|line| names(line, name)));
    reading.dropped = dropped(&presets, &reading, &lines);
    // What the IDE did not decide, the list did (C1-26).
    reading.unknown.clear();
    reading.order = lines
        .iter()
        .filter_map(|line| {
            presets
                .entries
                .iter()
                .find(|entry| names(line, &entry.name))
                .map(|entry| entry.name.clone())
        })
        .collect();
    Ok((presets, reading))
}

fn answer(root: &Path, program: &Path, opened: &[(PathBuf, Option<u64>)]) -> Option<Answer> {
    let key = Key {
        program: program.to_path_buf(),
        stamp: std::fs::metadata(program)
            .ok()
            .and_then(|metadata| Some((metadata.len(), metadata.modified().ok()?))),
        opened: opened.to_vec(),
    };
    let answers = ANSWERS.get_or_init(Mutex::default);
    if let Some((known, answer)) = answers
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(root)
        && *known == key
    {
        return Some(answer.clone());
    }
    let answer = ask(root, program)?;
    answers
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(root.to_path_buf(), (key, answer.clone()));
    Some(answer)
}

/// `cmake --list-presets` at the root; `None` when it does not run.
fn ask(root: &Path, program: &Path) -> Option<Answer> {
    let mut command = Command::new(program);
    command
        .arg("--list-presets")
        .current_dir(root)
        .stdin(Stdio::null());
    let output = crate::tools::output_when_free(&mut command).ok()?;
    if output.status.success() {
        return Some(Answer::Listed(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| line.strip_prefix("  "))
                .filter(|line| line.starts_with('"'))
                .map(str::to_owned)
                .collect(),
        ));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Some(Answer::Refused(reason(&stderr).unwrap_or_else(|| {
        format!("o cmake --list-presets terminou com {}", output.status)
    })))
}

/// The line after `Could not read presets from <folder>:`, else the first
/// line with text, without its `CMake Error: `.
fn reason(stderr: &str) -> Option<String> {
    let first = stderr
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    let after = stderr
        .lines()
        .skip_while(|line| !line.contains("Could not read presets from"))
        .nth(1)
        .map(str::trim)
        .filter(|line| !line.is_empty());
    Some(
        after
            .unwrap_or(first)
            .trim_start_matches("CMake Error: ")
            .to_owned(),
    )
}

/// The configure presets the `CMake` does not list, with why: a `condition`
/// the IDE did not decide, else the generator. A preset the native reading
/// already left out, hidden too, keeps its own reason (`Reading::unusable`
/// gives it first).
fn dropped(presets: &Presets, reading: &Reading, lines: &[String]) -> Vec<(String, String)> {
    presets
        .entries
        .iter()
        .filter(|entry| !lines.iter().any(|line| names(line, &entry.name)))
        .map(|entry| {
            let name = &entry.name;
            let generator = inherited(presets, Kind::Configure, entry, &["generator"])
                .and_then(|(value, _)| value.as_str());
            let reason = if reading.unknown.contains(name) {
                format!("a condition do preset {name} é falsa para o CMake")
            } else if let Some(generator) = generator {
                format!("o CMake não tem o gerador \"{generator}\" do preset {name}")
            } else {
                format!("o CMake não lista o preset {name}")
            };
            (name.clone(), reason)
        })
        .collect()
}

/// Whether a line of the list is the one of `name`: the name between quotes,
/// alone or before the spaces and `- displayName` (measured: the spaces align
/// the longest name). The `CMake` does not escape quotes.
fn names(line: &str, name: &str) -> bool {
    line.strip_prefix('"')
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('"'))
        .is_some_and(|rest| {
            rest.is_empty() || (rest.starts_with(' ') && rest.trim_start().starts_with("- "))
        })
}

#[cfg(test)]
mod tests;
