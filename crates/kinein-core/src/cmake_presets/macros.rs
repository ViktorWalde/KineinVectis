//! The macros of cmake-presets(7): one scanner, two policies.
//!
//! The `binaryDir` of a preset expands every documented macro in the context
//! of the preset in use (consolidacao C §16). An `include` path expands by the
//! version of the including file (C1-18, measured with `CMake` 4.4.5): none up
//! to version 6, only `$penv{}` in 7 and 8, and from 9 on everything except
//! `$env{}` and the macros of a preset (`presetName`, `generator`).

use std::path::Path;

use super::{Field, files::Presets, lookup};

/// Depth of macros inside `environment`: `CMake` refuses cycles, and here one
/// only leaves the preset without a folder.
const MAX_DEPTH: usize = 16;

/// The namespace of one macro.
#[derive(Clone, Copy)]
enum Namespace {
    Plain,
    Env,
    ParentEnvironment,
    Vendor,
}

/// Replaces each macro by what `resolve` says; a `$` that opens none stays.
fn expand(
    text: &str,
    resolve: &mut dyn FnMut(Namespace, &str) -> Result<String, String>,
) -> Result<String, String> {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('$') {
        result.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some((namespace, after)) = [
            ("${", Namespace::Plain),
            ("$env{", Namespace::Env),
            ("$penv{", Namespace::ParentEnvironment),
            ("$vendor{", Namespace::Vendor),
        ]
        .iter()
        .find_map(|(prefix, namespace)| rest.strip_prefix(prefix).map(|after| (*namespace, after))) else {
            result.push('$');
            rest = &rest[1..];
            continue;
        };
        let end = after
            .find('}')
            .ok_or_else(|| format!("macro sem fechamento: {rest}"))?;
        let name = &after[..end];
        rest = &after[end + 1..];
        result.push_str(&resolve(namespace, name)?);
    }
    result.push_str(rest);
    Ok(result)
}

/// The macros a preset does not depend on.
fn source_macro(root: &Path, file_dir: &Path, name: &str) -> Option<String> {
    let text = |path: &Path| path.to_string_lossy().into_owned();
    Some(match name {
        "sourceDir" => text(root),
        "fileDir" => text(file_dir),
        "sourceParentDir" => text(root.parent().unwrap_or(root)),
        "sourceDirName" => root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        "hostSystemName" => host_system_name().to_owned(),
        "dollar" => "$".to_owned(),
        "pathListSep" => if cfg!(windows) { ";" } else { ":" }.to_owned(),
        _ => return None,
    })
}

/// An `include` path of a file of `version` in `file_dir`.
pub(super) fn include_path(
    text: &str,
    version: u64,
    root: &Path,
    file_dir: &Path,
) -> Result<String, String> {
    if version < 7 {
        return Ok(text.to_owned());
    }
    expand(text, &mut |namespace, name| match namespace {
        Namespace::ParentEnvironment => parent_environment(name),
        Namespace::Plain if version >= 9 => source_macro(root, file_dir, name)
            .ok_or_else(|| format!("macro não aceita em include: ${{{name}}}")),
        _ => Err(format!("macro não aceita em include na versão {version}")),
    })
}

/// The expansion of a `binaryDir` in the context of one preset.
pub(super) struct Macros<'a> {
    pub root: &'a Path,
    pub presets: &'a Presets,
    pub preset: &'a str,
    /// The folder of the file that defines the preset in use: also for a
    /// field inherited from another file (measured with versions 6 and 12).
    pub file_dir: &'a Path,
}

impl Macros<'_> {
    pub(super) fn expand(&self, text: &str, depth: usize) -> Result<String, String> {
        if depth > MAX_DEPTH {
            return Err("ciclo de macros em environment".to_owned());
        }
        expand(text, &mut |namespace, name| match namespace {
            Namespace::Plain => self.macro_value(name),
            Namespace::Env => self.environment(name, depth),
            Namespace::ParentEnvironment => parent_environment(name),
            Namespace::Vendor => Err(format!("$vendor{{{name}}}: o CMake ignora este preset")),
        })
    }

    fn macro_value(&self, name: &str) -> Result<String, String> {
        if let Some(value) = source_macro(self.root, self.file_dir, name) {
            return Ok(value);
        }
        match name {
            "presetName" => Ok(self.preset.to_owned()),
            "generator" => match lookup(self.presets, self.preset, &["generator"], 0) {
                Field::Text(generator) => Ok(generator),
                Field::Unset | Field::Absent => {
                    Err("a macro generator exige generator no preset".to_owned())
                }
            },
            other => Err(format!("macro desconhecida: ${{{other}}}")),
        }
    }

    /// `$env{X}`: o `environment` do preset (com heranca, e com as macros do
    /// proprio valor), senao o ambiente do core; ausente expande vazio.
    fn environment(&self, name: &str, depth: usize) -> Result<String, String> {
        if name.is_empty() {
            return Err("$env{} sem nome".to_owned());
        }
        match lookup(self.presets, self.preset, &["environment", name], 0) {
            Field::Text(value) => self.expand(&value, depth + 1),
            Field::Unset => Ok(String::new()),
            Field::Absent => parent_environment(name),
        }
    }
}

fn parent_environment(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("$penv{} sem nome".to_owned());
    }
    Ok(std::env::var(name).unwrap_or_default())
}

/// O `${hostSystemName}`: o nome que o `CMake` da' ao sistema do host.
const fn host_system_name() -> &'static str {
    if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else {
        "Linux"
    }
}
