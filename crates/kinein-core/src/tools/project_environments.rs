//! Instalações paralelas para criação: usa os mesmos diretórios do detector.

use super::{ToolDetector, is_executable};
use kinein_protocol::{ProjectEnvironment, ProjectEnvironmentsResult};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

impl ToolDetector {
    pub(crate) fn project_environments(&self) -> ProjectEnvironmentsResult {
        let mut directories = self.search_dirs();
        if self.install_root.is_some()
            && let Some(home) = crate::platform::home_dir()
        {
            let rustup =
                std::env::var_os("RUSTUP_HOME").map_or_else(|| home.join(".rustup"), PathBuf::from);
            let pyenv = std::env::var_os("PYENV_ROOT")
                .map_or_else(|| home.join(".pyenv"), PathBuf::from)
                .join("versions");
            let uv = std::env::var_os("UV_PYTHON_INSTALL_DIR")
                .map_or_else(|| home.join(".local/share/uv/python"), PathBuf::from);
            for root in [rustup.join("toolchains"), pyenv, uv] {
                if let Ok(entries) = std::fs::read_dir(root) {
                    directories.extend(
                        entries
                            .flatten()
                            .take(128)
                            .map(|entry| entry.path().join("bin")),
                    );
                }
            }
        }
        let mut paths = Vec::new();
        for directory in directories {
            if let Ok(entries) = std::fs::read_dir(directory) {
                paths.extend(entries.flatten().take(32768).filter_map(|entry| {
                    let path = entry.path();
                    if cfg!(windows) && path.extension().is_none_or(|extension| extension != "exe")
                    {
                        return None;
                    }
                    let language = language_for_binary(path.file_name()?.to_str()?)?;
                    is_executable(&path).then_some((language, path))
                }));
            }
        }
        paths.sort();
        let mut seen = HashSet::new();
        let environments = paths
            .into_iter()
            .filter_map(|(language, path)| {
                let real = crate::platform::canonicalize(&path).ok()?;
                if language == "rust" && real.file_stem().is_some_and(|name| name == "rustup") {
                    return None;
                }
                if !seen.insert((language, real)) {
                    return None;
                }
                let version = environment_version(&path).ok()?;
                Some(ProjectEnvironment {
                    language: language.to_owned(),
                    executable: path.display().to_string(),
                    version,
                })
            })
            .collect();
        ProjectEnvironmentsResult { environments }
    }
}

fn language_for_binary(name: &str) -> Option<&'static str> {
    let name = name.strip_suffix(".exe").unwrap_or(name);
    for (binary, language) in [
        ("clang++", "cpp"),
        ("g++", "cpp"),
        ("c++", "cpp"),
        ("clang", "c"),
        ("gcc", "c"),
        ("cc", "c"),
        ("python", "python"),
        ("cargo", "rust"),
    ] {
        if let Some(suffix) = name.strip_prefix(binary) {
            let suffix = suffix.strip_prefix('-').unwrap_or(suffix);
            if suffix.is_empty()
                || suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || byte == b'.')
            {
                return Some(language);
            }
        }
    }
    None
}

/// Version probe with the existing process owner, deadline and bounded retained text.
pub(super) fn environment_version(path: &Path) -> Result<String, String> {
    let mut command = Command::new(path);
    command.env("RUSTUP_AUTO_INSTALL", "0");
    command.arg("--version").current_dir(std::env::temp_dir());
    let mut first = None;
    let (status, expired) = crate::process::stream_command_lines_with_timeout(
        command,
        Duration::from_secs(2),
        &mut |_, line| {
            if first.is_none() && !line.trim().is_empty() {
                first = Some(line.chars().take(512).collect::<String>());
            }
        },
    )
    .map_err(|error| format!("{error:?}"))?;
    if expired || !status.success() {
        return Err("ferramenta não respondeu à consulta de versão".to_owned());
    }
    first.ok_or_else(|| "ferramenta não informou versão".to_owned())
}
