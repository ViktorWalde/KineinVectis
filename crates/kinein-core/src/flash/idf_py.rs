//! Native `idf.py flash` (52 §14.40).
//!
//! The ESP-IDF wrapper builds, then flashes through its own esptool. The
//! intent binds the build's configuration by content, never its images:
//! changing code and pressing Run builds and flashes the new code, while a
//! changed configuration refuses.
//!
//! What `idf.py -p PORT flash` does in ESP-IDF v6.1 (`serial_ext.py`,
//! `run_serial_tool.cmake`, esptool 5.6.0): it depends on `all`, so it
//! rebuilds and may reconfigure; port and baud come from `-p`/`-b` or
//! `ESPPORT`/`ESPBAUD`; it fast-reflashes by default (`--diff-with` on
//! `build/*_flashed.bin`, or `--skip-flashed`), every region verified by MD5,
//! and copies the flashed images to `build/*_flashed.bin` afterwards.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use kinein_protocol::{FlashIntent, FlashProposalParams};
use serde_json::{Value, json};

use super::native::{NativeRequest, Prepared};
use super::{ESPTOOL_BAUD, FlashCommand, FlashError, FlashPlan, chosen_port, snapshot};
use crate::build::engine::idf_command_args;

/// What `idf.py flash` and its esptool read from the environment beyond
/// `ESPTOOL_*`: port, baud, full/trusted fast reflash, extra arguments and
/// the folders of `idf.py` extensions, which can redefine `flash` (52 §14.44).
const IDF_VARIABLES: [&str; 8] = [
    "ESPPORT",
    "ESPBAUD",
    "IDF_FLASH_FULL",
    "IDF_TRUST_FLASH_CONTENT",
    "SERIAL_TOOL_EXTRA_PRE_CMD_ARGS",
    "SERIAL_TOOL_EXTRA_ARGS",
    "IDF_EXTRA_ACTIONS_PATH",
    "IDF_EXTENSION_ALLOW_UNTRUSTED",
];
/// Activating ESP-IDF and asking `idf.py --version` takes seconds.
const VERSION_DEADLINE: Duration = Duration::from_secs(30);

/// Inherited names removed from the native `idf.py` and its version query.
#[must_use]
pub fn inherited_variables() -> Vec<OsString> {
    let mut names = super::esptool::inherited_variables();
    names.extend(
        IDF_VARIABLES
            .iter()
            .filter(|name| std::env::var_os(name).is_some())
            .map(OsString::from),
    );
    names.sort();
    names.dedup();
    names
}

/// A native `idf.py` intent names a port; it has no `CMake` target, probe or
/// catalogue firmware (the build is the project's).
///
/// # Errors
/// `InvalidSelection` for fields of another destination kind.
pub(super) fn check_selection(selection: &FlashProposalParams) -> Result<(), FlashError> {
    if selection.target.is_some() || selection.probe.is_some() || selection.firmware.is_some() {
        return Err(FlashError::InvalidSelection {
            message: "target, probe e firmware não valem para o idf.py nativo, que grava o build do projeto"
                .to_owned(),
        });
    }
    Ok(())
}

/// Plan, bind and fingerprint a native `idf.py flash`.
///
/// # Errors
/// Missing activation or port, an unproven or eFuse-burning build, an esptool
/// configuration file, an unsupported ESP-IDF version, cancellation or files
/// changing while they are read.
pub(super) fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Prepared, FlashError> {
    check_selection(&request.selection)?;
    let activation = request
        .activation
        .clone()
        .ok_or_else(|| FlashError::MissingTool {
            tool: "ESP-IDF".to_owned(),
            hint: "a ativação do ESP-IDF não foi encontrada; instale pelo EIM ou defina IDF_PATH"
                .to_owned(),
        })?;
    let device = chosen_port(request.selection.device.as_deref())?.to_owned();
    let device_key = crate::platform::serial_device_revision(Path::new(&device))
        .map_err(|error| invalid(&error.to_string()))?;
    let origin = crate::project::flash_origin::validate(
        root,
        request.selection.build_directory.as_deref(),
        &request.model,
    )
    .map_err(|message| invalid(&message))?;
    // Under idf.py the esptool runs in the build directory.
    let home = crate::platform::home_dir();
    for folder in [origin.build.as_path(), root] {
        if let Some(path) = super::esptool::config::first_declared(folder, home.as_deref())
            .map_err(|reason| invalid(&reason))?
        {
            return Err(invalid(&format!(
                "o esptool leria a configuração {} (seção [esptool]), que pode mudar reset, tentativas e tempos fora desta prévia; mova ou renomeie o arquivo, ou use o modo manual",
                path.display()
            )));
        }
    }
    refuse_extensions(root, &origin.build)?;
    let version = version(&request.tool, &activation, cancel)?;
    let (contents, files) = bind(root, &origin, &request.tool, &activation, cancel)?;

    let build = origin.build.display().to_string();
    let command = FlashCommand::new(
        request.tool.clone(),
        idf_command_args(
            &activation,
            &["-B", &build, "-p", &device, "-b", ESPTOOL_BAUD, "flash"],
        ),
    );
    let inherited = inherited_variables();
    let recipe = &origin.recipe;
    let source = vec![
        "motor: idf.py (o wrapper do ESP-IDF, resolvido para a intenção nativa)".to_owned(),
        format!("ambiente: . {} ({version})", activation.display()),
        format!("build: {build} (descrição, receita e configuração vinculadas por conteúdo)"),
        format!("porta: {device} (escolhida no painel de Embarcados), {ESPTOOL_BAUD} baud"),
        format!(
            "reset: {} antes e {} depois; {} (receita do build)",
            recipe.before.as_deref().unwrap_or("default-reset"),
            recipe.after.as_deref().unwrap_or("hard-reset"),
            if recipe.stub { "com stub" } else { "sem stub" }
        ),
        if inherited.is_empty() {
            "ambiente: nenhuma variável ESPPORT, ESPBAUD, IDF_FLASH_*, SERIAL_TOOL_EXTRA_* ou ESPTOOL_* herdada".to_owned()
        } else {
            format!(
                "ambiente: {} não chegam ao idf.py",
                inherited
                    .iter()
                    .map(|name| name.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        "configuração: nenhum esptool.cfg, setup.cfg ou tox.ini com [esptool] no build, na raiz, na pasta de configuração do usuário ou no home".to_owned(),
    ];
    let warnings = vec![
        "compila antes de gravar: as imagens gravadas são as do build no momento da execução, não as desta prévia".to_owned(),
        "grava só os setores que mudaram (fast reflash do ESP-IDF 6), conferidos por MD5, e copia as imagens gravadas para build/*_flashed.bin".to_owned(),
    ];
    let mut plan = FlashPlan::new(
        "Gravar (idf.py)".to_owned(),
        "idf.py".to_owned(),
        source,
        warnings,
        command,
    );
    request.selection.native = true;
    request.selection.engine = Some("idf.py".to_owned());
    request.selection.device = Some(device);
    request.selection.build_directory = Some(build);
    let fingerprint = snapshot::digest(
        &serde_json::to_vec(&json!([
            root,
            request.context,
            request.selection,
            request.model.target,
            request.model.frameworks,
            request.model.sdks,
            plan.command.program,
            plan.command.args,
            plan.proposal.source,
            device_key,
            version,
            contents
        ]))
        .map_err(|error| invalid(&error.to_string()))?,
    );
    let intent = FlashIntent {
        version: 1,
        workspace: root.display().to_string(),
        selection: request.selection,
        program: plan.command.program.display().to_string(),
        args: plan.command.args.clone(),
        device_key,
        fingerprint,
    };
    super::native::validate_intent(root, &intent, Some(&plan.proposal.command))
        .map_err(|message| invalid(&message))?;
    plan.proposal.flash = Some(intent.clone());
    let prepared = Prepared::from_parts(plan, intent, files);
    if !prepared.unchanged() {
        return Err(invalid("arquivos mudaram ao preparar a prévia; gere outra"));
    }
    Ok(prepared)
}

/// `idf.py` imports `idf_ext.py` from the project directory and from the
/// build components it trusts, on every invocation; an extension can
/// redefine `flash` (52 §14.44).
fn refuse_extensions(root: &Path, build: &Path) -> Result<(), FlashError> {
    let description: Value = std::fs::read(build.join("project_description.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| invalid("project_description.json ilegível"))?;
    let components = description["build_component_info"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(_, info)| info["dir"].as_str().map(PathBuf::from));
    for folder in std::iter::once(root.to_path_buf()).chain(components) {
        let extension = folder.join("idf_ext.py");
        if extension.exists() {
            return Err(invalid(&format!(
                "{} estende o idf.py e pode redefinir a gravação fora desta prévia; use o modo manual",
                extension.display()
            )));
        }
    }
    Ok(())
}

/// `idf.py --version` in the activated environment, without the removed
/// variables; only ESP-IDF 6 has the measured flash semantics. It runs in
/// the activation's folder: in the project, `idf.py` would import the
/// project's `idf_ext.py` (52 §14.44).
fn version(
    shell: &Path,
    activation: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<String, FlashError> {
    let outside = activation
        .parent()
        .map_or_else(std::env::temp_dir, Path::to_path_buf);
    let mut command = Command::new(shell);
    command
        .args(idf_command_args(activation, &["--version"]))
        .current_dir(outside)
        .env("NO_COLOR", "1");
    for name in inherited_variables() {
        command.env_remove(name);
    }
    let mut output = String::new();
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        VERSION_DEADLINE,
        cancel.clone(),
        &mut |_, line| {
            if output.len() < 16_384 {
                output.push_str(&line);
                output.push('\n');
            }
        },
    );
    let refuse = |reason: String| FlashError::UnsupportedTool {
        message: format!(
            "Não foi possível preparar a gravação com idf.py: {reason}. Nenhum dispositivo foi acessado."
        ),
    };
    match result {
        Ok((_, true)) => Err(refuse("o prazo da consulta de versão expirou".to_owned())),
        Ok((status, false)) if status.success() => {
            let reported = output
                .lines()
                .map(str::trim)
                .find(|line| line.starts_with("ESP-IDF v"))
                .ok_or_else(|| {
                    refuse(format!(
                        "idf.py --version não informou a versão: {}",
                        output.trim()
                    ))
                })?;
            if reported.trim_start_matches("ESP-IDF v").starts_with("6.") {
                Ok(reported.to_owned())
            } else {
                Err(refuse(format!(
                    "{reported} sem efeitos validados; esta integração conhece o ESP-IDF 6"
                )))
            }
        }
        Ok((status, false)) => Err(refuse(format!(
            "idf.py --version terminou com {status}: {}",
            output.trim()
        ))),
        Err(error) => Err(refuse(format!("falha na consulta de versão: {error:?}"))),
    }
}

/// Content hashes of the build description, recipe and configuration (the
/// root `sdkconfig`, every `sdkconfig.defaults*` and the effective
/// `config/sdkconfig.json`), plus the shell and the activation script.
fn bind(
    root: &Path,
    origin: &crate::project::flash_origin::FlashOrigin,
    shell: &Path,
    activation: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<(Vec<Value>, Vec<snapshot::Observation>), FlashError> {
    let mut paths: Vec<PathBuf> = origin
        .sources
        .iter()
        .map(|(path, _)| path.clone())
        .collect();
    let mut defaults: Vec<PathBuf> = std::fs::read_dir(root)
        .map_err(|error| invalid(&error.to_string()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("sdkconfig.defaults"))
        })
        .collect();
    defaults.sort();
    paths.extend(defaults);
    paths.push(shell.to_path_buf());
    paths.push(activation.to_path_buf());
    let mut contents = Vec::new();
    let mut files = Vec::new();
    for path in paths {
        let (hash, observation) =
            snapshot::file(&path, None, cancel).map_err(|message| invalid(&message))?;
        contents.push(json!([path, hash]));
        files.push(observation);
    }
    Ok((contents, files))
}

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!(
            "Não foi possível validar a gravação nativa: {message}. Nenhuma gravação foi iniciada."
        ),
    }
}
