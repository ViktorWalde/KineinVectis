//! The macros of cmake-presets(7): one scanner, two policies.
//!
//! A preset field expands in the context of the preset being read, as
//! `CMake` 4.4.5 does when it reads the files (C1-24, consolidacao C §17.4):
//! the macros its file version knows, `${generator}` empty without one, and
//! `$env{}` from the `environment` after inheritance, with cycles refused.
//! `$vendor{}` stops the preset: `CMake` ignores it. An `include` path expands
//! by the version of the including file (C1-18): none up to version 6, only
//! `$penv{}` in 7 and 8, and from 9 on everything except `$env{}` and the
//! macros of a preset (`presetName`, `generator`).

use std::{collections::BTreeMap, path::Path};

/// The namespace of one macro.
#[derive(Clone, Copy)]
enum Namespace {
    Plain,
    Env,
    ParentEnvironment,
    Vendor,
}

/// What a text expands to.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Expanded {
    Text(String),
    /// A `$vendor{}` macro: `CMake` stops there and ignores the preset.
    Vendor,
}

/// Replaces each macro by what `resolve` says, left to right; a `$` that
/// opens none stays. The first `$vendor{}` stops the text.
fn expand(
    text: &str,
    resolve: &mut dyn FnMut(Namespace, &str) -> Result<Expanded, String>,
) -> Result<Expanded, String> {
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
        match resolve(namespace, name)? {
            Expanded::Text(value) => result.push_str(&value),
            Expanded::Vendor => return Ok(Expanded::Vendor),
        }
    }
    result.push_str(rest);
    Ok(Expanded::Text(result))
}

/// Whether every macro of `text` is `${fileDir}`. In a file of version 12 or
/// more, such a text names the folder of its own file, even when inherited;
/// any other macro leaves it to the preset in use (measured with 4.4.5 in
/// `binaryDir`, `installDir`, `toolchainFile`, `graphviz` and
/// `cacheVariables`, never in `environment` or `condition`).
pub(super) fn only_file_dir(text: &str) -> bool {
    let mut only = true;
    let scanned = expand(text, &mut |namespace, name| {
        only &= matches!(namespace, Namespace::Plain) && name == "fileDir";
        Ok(Expanded::Text(String::new()))
    });
    scanned.is_ok() && only
}

/// The first presets version that knows a macro: an older file that uses it
/// is refused (measured with 4.4.5).
fn since(name: &str) -> u64 {
    match name {
        "hostSystemName" => 3,
        "fileDir" => 4,
        "pathListSep" => 5,
        _ => 1,
    }
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
    let expanded = expand(text, &mut |namespace, name| match namespace {
        Namespace::ParentEnvironment => parent_environment(name).map(Expanded::Text),
        Namespace::Plain if version >= 9 => source_macro(root, file_dir, name)
            .map(Expanded::Text)
            .ok_or_else(|| format!("macro não aceita em include: ${{{name}}}")),
        _ => Err(format!("macro não aceita em include na versão {version}")),
    })?;
    match expanded {
        Expanded::Text(path) => Ok(path),
        Expanded::Vendor => unreachable!("$vendor{{}} recusado acima"),
    }
}

/// The context of the preset being read.
pub(super) struct Context<'a> {
    pub root: &'a Path,
    /// `${presetName}`.
    pub preset: &'a str,
    /// `${generator}`: the generator of the configure preset, also the one a
    /// build or test preset names; empty without one and in a package preset
    /// (measured: no error).
    pub generator: &'a str,
    /// The `environment` after inheritance; `None` where a `null` removed
    /// the variable, and `$env{}` falls back to the core environment.
    pub environment: &'a BTreeMap<String, Option<String>>,
    /// The version of the preset's file: it decides the macros, also in a
    /// field inherited from a newer file.
    pub version: u64,
    /// The folder of the preset's file: `${fileDir}`.
    pub file_dir: &'a Path,
}

impl Context<'_> {
    /// A field of the preset.
    pub(super) fn expand(&self, text: &str) -> Result<Expanded, String> {
        self.expand_in(text, self.file_dir, &mut Vec::new())
    }

    /// A field written in `file_dir`, in a file of `version`: the version 12
    /// rule of `${fileDir}` (see `only_file_dir`).
    pub(super) fn expand_written(
        &self,
        text: &str,
        file_dir: &Path,
        version: u64,
    ) -> Result<Expanded, String> {
        if version >= 12 && only_file_dir(text) {
            return self.expand_in(text, file_dir, &mut Vec::new());
        }
        self.expand(text)
    }

    /// The value of one `environment` variable: a value that comes back to
    /// the variable is a cycle.
    pub(super) fn expand_variable(&self, name: &str) -> Result<Expanded, String> {
        self.environment_value(name, &mut Vec::new())
    }

    fn expand_in(
        &self,
        text: &str,
        file_dir: &Path,
        path: &mut Vec<String>,
    ) -> Result<Expanded, String> {
        expand(text, &mut |namespace, name| match namespace {
            Namespace::Plain => self.macro_value(name, file_dir).map(Expanded::Text),
            Namespace::Env => self.environment_value(name, path),
            Namespace::ParentEnvironment => parent_environment(name).map(Expanded::Text),
            Namespace::Vendor => Ok(Expanded::Vendor),
        })
    }

    fn macro_value(&self, name: &str, file_dir: &Path) -> Result<String, String> {
        if self.version < since(name) {
            return Err(format!(
                "a macro ${{{name}}} exige version {} ou maior",
                since(name)
            ));
        }
        if let Some(value) = source_macro(self.root, file_dir, name) {
            return Ok(value);
        }
        match name {
            "presetName" => Ok(self.preset.to_owned()),
            "generator" => Ok(self.generator.to_owned()),
            other => Err(format!("macro desconhecida: ${{{other}}}")),
        }
    }

    /// `$env{X}`: the `environment` of the preset (with the macros of the
    /// value itself), else the core environment; absent expands empty.
    fn environment_value(&self, name: &str, path: &mut Vec<String>) -> Result<Expanded, String> {
        if name.is_empty() {
            return Err("$env{} sem nome".to_owned());
        }
        let Some(Some(value)) = self.environment.get(name) else {
            return parent_environment(name).map(Expanded::Text);
        };
        if path.iter().any(|seen| seen == name) {
            return Err(format!("ciclo de macros em environment: {name}"));
        }
        path.push(name.to_owned());
        let expanded = self.expand_in(value, self.file_dir, path);
        path.pop();
        expanded
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
