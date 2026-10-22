//! Esptool syntax selected from the binary's observed major version.
//! The planner is pure; `probe` only asks `version`, never a serial operation.

use super::{ESPTOOL_BAUD, FlashCommand, FlashError, ProposalTools, chosen_port, readable_size};
use kinein_protocol::{FlashRecipe, ProjectModel};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

/// The exact executable and version used to prepare a proposal.
#[derive(Debug, Clone)]
pub struct Selected {
    /// The executable returned by the existing detector.
    pub path: PathBuf,
    version: String,
    legacy: bool,
}

impl Selected {
    /// Accept one consistent version from the command's bare or esptool-prefixed lines.
    ///
    /// # Errors
    /// Refuses missing, conflicting or unsupported version evidence.
    pub fn from_output(path: PathBuf, output: &str) -> Result<Self, FlashError> {
        let versions: BTreeSet<&str> = output.lines().filter_map(version_line).collect();
        if versions.len() != 1 {
            return Err(unsupported(
                &path,
                "a consulta não informou uma versão única reconhecível",
            ));
        }
        let version = versions.first().copied().unwrap_or_default();
        let major = version.split('.').next().unwrap_or_default();
        if !matches!(major, "4" | "5") {
            return Err(unsupported(
                &path,
                &format!(
                    "versão {version} sem sintaxe validada; esta integração conhece esptool 4 e 5"
                ),
            ));
        }
        Ok(Self {
            path,
            version: version.to_owned(),
            legacy: major == "4",
        })
    }

    /// Translates only a syntax token, never an image path or user shell command.
    #[must_use]
    pub fn syntax(&self, token: &str) -> String {
        let (prefix, word) = token
            .strip_prefix("--")
            .map_or(("", token), |word| ("--", word));
        let word = if self.legacy {
            word.replace('-', "_")
        } else {
            word.replace('_', "-")
        };
        format!("{prefix}{word}")
    }

    pub(super) fn evidence(&self) -> String {
        format!(
            "ferramenta: {} · esptool {} (versão consultada)",
            self.path.display(),
            self.version
        )
    }
}

fn version_line(line: &str) -> Option<&str> {
    let line = line.trim();
    let value = line
        .strip_prefix("esptool.py ")
        .or_else(|| line.strip_prefix("esptool "))
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or(line);
    let version = value.strip_prefix('v').unwrap_or(value);
    let (major, rest) = version.split_once('.')?;
    if major.is_empty()
        || !major.bytes().all(|byte| byte.is_ascii_digit())
        || !rest.as_bytes().first().is_some_and(u8::is_ascii_digit)
        || !rest
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-+_".contains(&byte))
    {
        return None;
    }
    Some(version)
}

fn unsupported(path: &Path, reason: &str) -> FlashError {
    FlashError::UnsupportedTool {
        message: format!(
            "Não foi possível preparar a gravação com {}: {reason}. Confira a ferramenta selecionada e gere outra prévia; nenhum dispositivo foi acessado.",
            path.display()
        ),
    }
}

/// Version-only query, using the existing process streamer and its process ownership.
///
/// # Errors
/// Returns a proposal refusal on failure, deadline or unsupported/inconsistent output.
pub fn probe(path: PathBuf, root: &Path, timeout: Duration) -> Result<Selected, FlashError> {
    probe_cancelable(path, root, timeout, Arc::new(AtomicBool::new(false)))
}

/// Version-only query that also observes cancellation of a pending native launch.
///
/// # Errors
/// Same version/deadline refusals as [`probe`], without any serial operation.
pub fn probe_cancelable(
    path: PathBuf,
    root: &Path,
    timeout: Duration,
    cancel: Arc<AtomicBool>,
) -> Result<Selected, FlashError> {
    let mut command = Command::new(&path);
    command
        .arg("version")
        .current_dir(root)
        .env("NO_COLOR", "1");
    let mut output = String::new();
    let mut overflow = false;
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        timeout,
        cancel,
        &mut |_, line| {
            if output.len() + line.len() < 16_384 {
                output.push_str(&line);
                output.push('\n');
            } else {
                overflow = true;
            }
        },
    );
    match result {
        Ok((_, true)) => Err(unsupported(&path, "o prazo da consulta de versão expirou")),
        Ok(_) if overflow => Err(unsupported(
            &path,
            "a saída da consulta de versão excedeu o limite",
        )),
        Ok((status, false)) if status.success() => Selected::from_output(path, &output),
        Ok((status, false)) => Err(unsupported(
            &path,
            &format!(
                "consulta de versão terminou com {status}: {}",
                output.trim()
            ),
        )),
        Err(error) => Err(unsupported(
            &path,
            &format!("falha na consulta de versão: {error:?}"),
        )),
    }
}

pub(super) fn selected<'a>(tools: &'a ProposalTools<'_>) -> Result<&'a Selected, FlashError> {
    if let Some(selected) = &tools.esptool {
        return Ok(selected);
    }
    let found = (tools.find)("esptool").or_else(|| (tools.find)("esptool.py"));
    if let Some(path) = found {
        return Err(unsupported(&path, "a versão do binário não foi confirmada"));
    }
    Err(FlashError::MissingTool {
        tool: "esptool".to_owned(),
        hint: "pipx install esptool (o painel de instalação mostra o passo)".to_owned(),
    })
}

pub(super) fn propose(
    model: &ProjectModel,
    device: Option<&str>,
    flash_size_bytes: Option<u64>,
    tools: &ProposalTools<'_>,
    source: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<FlashCommand, FlashError> {
    let selected = selected(tools)?;
    let recipe: &FlashRecipe = model
        .artifacts
        .flash_recipe
        .as_ref()
        .filter(|r| !r.files.is_empty())
        .ok_or_else(|| FlashError::NoArtifact {
            message: "sem receita de gravação: o ESP-IDF escreve build/flasher_args.json no \
                      build — compile o projeto primeiro"
                .to_owned(),
        })?;
    let device = chosen_port(device)?;
    let chip = recipe
        .chip
        .as_deref()
        .or(model.target.chip.as_deref())
        .unwrap_or("auto");
    let mut parts = vec![
        "--chip".to_owned(),
        chip.to_owned(),
        "--port".to_owned(),
        device.to_owned(),
        "--baud".to_owned(),
        ESPTOOL_BAUD.to_owned(),
        "--before".to_owned(),
        selected.syntax(recipe.before.as_deref().unwrap_or("default-reset")),
        "--after".to_owned(),
        selected.syntax(recipe.after.as_deref().unwrap_or("hard-reset")),
    ];
    if !recipe.stub {
        parts.push("--no-stub".to_owned());
    }
    parts.push(selected.syntax("write-flash"));
    for (flag, valor) in [
        ("--flash-mode", &recipe.flash_mode),
        ("--flash-size", &recipe.flash_size),
        ("--flash-freq", &recipe.flash_freq),
    ] {
        if let Some(v) = valor {
            parts.push(selected.syntax(flag));
            parts.push(v.clone());
        }
    }
    let mut encrypted_files = Vec::new();
    for image in &recipe.files {
        parts.push(format!("0x{:x}", image.offset));
        parts.push(image.file.clone());
        if image.encrypted {
            encrypted_files.push(image.name.clone().unwrap_or_else(|| image.file.clone()));
        }
    }
    append_evidence(model, recipe, chip, device, source);
    source.push(selected.evidence());
    append_warnings(recipe, &encrypted_files, flash_size_bytes, warnings);
    Ok(FlashCommand::new(selected.path.clone(), parts))
}

/// As linhas de evidencia do esptool: receita, chip e porta.
fn append_evidence(
    model: &ProjectModel,
    recipe: &FlashRecipe,
    chip: &str,
    device: &str,
    source: &mut Vec<String>,
) {
    let capacity = recipe
        .flash_size
        .as_deref()
        .map(|s| format!(", flash {s}"))
        .unwrap_or_default();
    let mode = recipe
        .flash_mode
        .as_deref()
        .map(|m| format!(" {m}"))
        .unwrap_or_default();
    source.push(format!(
        "receita: {} ({} imagens{capacity}{mode})",
        model
            .artifacts
            .flasher_args
            .as_deref()
            .unwrap_or("flasher_args.json"),
        recipe.files.len(),
    ));
    let origin = if recipe.chip.is_some() {
        "receita"
    } else {
        "kit/modelo"
    };
    source.push(format!("chip: {chip} ({origin})"));
    source.push(format!(
        "porta: {device} (escolhida no painel de Embarcados)"
    ));
}

/// O que o usuario deve saber antes de gravar: imagens cifradas e flash
/// menor que a receita.
fn append_warnings(
    recipe: &FlashRecipe,
    encrypted_files: &[String],
    flash_size_bytes: Option<u64>,
    warnings: &mut Vec<String>,
) {
    if !encrypted_files.is_empty() {
        warnings.push(format!(
            "imagens marcadas como cifradas ({}): a gravação proposta grava em claro; com flash \
             encryption ativa na placa, acrescente `--encrypt` a linha antes de rodar",
            encrypted_files.join(", ")
        ));
    }
    if let (Some(capacity_bytes), Some(recipe_bytes)) = (flash_size_bytes, recipe.flash_size_bytes)
        && u64::from(recipe_bytes) > capacity_bytes
    {
        warnings.push(format!(
            "a receita foi gerada para {} de flash e a placa identificada tem {}: ajuste \
                 CONFIG_ESPTOOLPY_FLASHSIZE e recompile",
            recipe.flash_size.as_deref().unwrap_or("?"),
            readable_size(capacity_bytes)
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinein_protocol::{FlashFile, ProjectArtifacts, TargetModel};

    #[test]
    fn version_is_parsed_from_output_and_not_the_executable_name() {
        for (path, output, legacy) in [
            ("esptool", "esptool.py v4.8.1\n4.8.1\n", true),
            ("esptool.py", "esptool v5.0.0\n5.0.0\n", false),
            ("other-name", "5.4.0.dev3\n", false),
        ] {
            let tool = Selected::from_output(PathBuf::from(path), output).unwrap();
            assert_eq!(tool.legacy, legacy);
            assert_eq!(tool.path, PathBuf::from(path));
        }
        for output in [
            "",
            "no version",
            "Python 4.8.1",
            "4.8.1\n5.4.0",
            "6.0.0",
            "5",
            "5.bad",
        ] {
            assert!(
                Selected::from_output(PathBuf::from("esptool"), output).is_err(),
                "{output}"
            );
        }
    }

    #[test]
    fn syntax_preserves_option_prefixes_and_maps_both_reset_spellings() {
        let old = Selected::from_output(PathBuf::new(), "4.8.1").unwrap();
        let new = Selected::from_output(PathBuf::new(), "5.0.0").unwrap();
        for (input, expected_old, expected_new) in [
            ("write-flash", "write_flash", "write-flash"),
            ("--flash-mode", "--flash_mode", "--flash-mode"),
            ("default-reset", "default_reset", "default-reset"),
            ("hard_reset", "hard_reset", "hard-reset"),
            ("no_reset_no_sync", "no_reset_no_sync", "no-reset-no-sync"),
        ] {
            assert_eq!(old.syntax(input), expected_old);
            assert_eq!(new.syntax(input), expected_new);
        }
    }

    #[test]
    fn only_syntax_changes_while_image_paths_and_values_remain_literal() {
        let path = "/project/write-flash/a-b_c's image.bin";
        let model = ProjectModel {
            root: "/project".to_owned(),
            embedded: true,
            frameworks: Vec::new(),
            sdks: Vec::new(),
            target: TargetModel::default(),
            hints: Vec::new(),
            artifacts: ProjectArtifacts {
                flash_recipe: Some(FlashRecipe {
                    chip: Some("esp32".to_owned()),
                    before: Some("no_reset_no_sync".to_owned()),
                    after: Some("hard_reset".to_owned()),
                    stub: false,
                    flash_mode: Some("dio".to_owned()),
                    flash_size: None,
                    flash_size_bytes: None,
                    flash_freq: None,
                    files: vec![FlashFile {
                        offset: 0x1000,
                        file: path.to_owned(),
                        name: None,
                        encrypted: false,
                    }],
                }),
                ..ProjectArtifacts::default()
            },
        };
        for (version, write, before, mode) in [
            ("4.8.1", "write_flash", "no_reset_no_sync", "--flash_mode"),
            ("5.0.0", "write-flash", "no-reset-no-sync", "--flash-mode"),
        ] {
            let tools = ProposalTools {
                find: &|_| None,
                esptool: Some(
                    Selected::from_output(PathBuf::from("/tools/esptool.py"), version).unwrap(),
                ),
            };
            let mut source = Vec::new();
            let mut warnings = Vec::new();
            let command = propose(
                &model,
                Some("/device/board-one_two"),
                None,
                &tools,
                &mut source,
                &mut warnings,
            )
            .unwrap();
            assert_eq!(command.program, PathBuf::from("/tools/esptool.py"));
            assert_eq!(command.args.last().map(String::as_str), Some(path));
            assert!(
                command
                    .args
                    .iter()
                    .any(|arg| arg == "/device/board-one_two")
            );
            let preview = command.preview();
            assert!(preview.contains(&super::super::quote(path)), "{preview}");
            assert!(preview.contains("'/device/board-one_two'"));
            assert!(preview.contains(&format!("--before {before}")));
            assert!(preview.contains(&format!("--no-stub {write} {mode} dio 0x1000")));
            assert!(source.iter().any(|line| line.contains(version)));
            assert!(warnings.is_empty());
        }
    }
}
