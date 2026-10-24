//! The `PlatformIO` queries of the native upload preview (52 §14.44): `pio
//! system info` for the Core and its interpreter, then `query.py` in that
//! interpreter for the configuration, platform and board, in the project
//! directory. Neither runs with the removed variables, and neither loads code
//! the project carries.
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use serde::Deserialize;
use serde_json::{Map, Value};

use super::super::FlashError;

/// The configuration and board query `PlatformIO`'s interpreter runs.
const QUERY: &str = include_str!("query.py");
const QUERY_DEADLINE: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Query {
    pub(super) error: Option<String>,
    #[serde(default)]
    pub(super) core: String,
    #[serde(default)]
    pub(super) config_path: PathBuf,
    #[serde(default)]
    pub(super) default_envs: Vec<String>,
    #[serde(default)]
    pub(super) candidates: Vec<String>,
    #[serde(default)]
    pub(super) platformio: Map<String, Value>,
    pub(super) env: Option<String>,
    #[serde(default)]
    pub(super) options: Map<String, Value>,
    pub(super) platform: Option<Platform>,
    pub(super) board: Option<Board>,
    pub(super) protocol: Option<String>,
    pub(super) uploader: Option<Uploader>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Platform {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Board {
    pub(super) id: String,
    pub(super) manifest: PathBuf,
    pub(super) upload: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Uploader {
    pub(super) dir: Option<PathBuf>,
    pub(super) version: Option<String>,
}

/// `pio system info --json-output`: the Core version (6.x) and `PlatformIO`'s
/// interpreter. It runs beside the `pio` executable: it needs no project.
pub(super) fn system_info(
    pio: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<(String, PathBuf), FlashError> {
    let mut command = Command::new(pio);
    command.args(["system", "info", "--json-output"]);
    let outside = pio
        .parent()
        .map_or_else(std::env::temp_dir, Path::to_path_buf);
    let output = run(command, &outside, cancel, "pio system info")?;
    let info: Value = parse_last_object(&output).ok_or_else(|| {
        refuse(&format!(
            "pio system info não respondeu JSON: {}",
            output.trim()
        ))
    })?;
    let core = info["core_version"]["value"].as_str().unwrap_or_default();
    if !core.starts_with("6.") {
        return Err(refuse(&format!(
            "PlatformIO Core {core} sem efeitos validados; esta integração conhece o Core 6"
        )));
    }
    let python = info["python_exe"]["value"]
        .as_str()
        .filter(|path| !path.is_empty())
        .ok_or_else(|| refuse("pio system info não informou o interpretador"))?;
    Ok((core.to_owned(), PathBuf::from(python)))
}

pub(super) fn query(
    python: &Path,
    root: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Query, FlashError> {
    let mut command = Command::new(python);
    command.args(["-I", "-c", QUERY]);
    let output = run(command, root, cancel, "a consulta do PlatformIO")?;
    let value = parse_last_object(&output).ok_or_else(|| {
        refuse(&format!(
            "a consulta do PlatformIO não respondeu JSON: {}",
            output.trim()
        ))
    })?;
    serde_json::from_value(value).map_err(|error| {
        refuse(&format!(
            "a consulta do PlatformIO veio malformada: {error}"
        ))
    })
}

/// Run a query in `folder` without the removed variables.
fn run(
    mut command: Command,
    folder: &Path,
    cancel: &Arc<AtomicBool>,
    what: &str,
) -> Result<String, FlashError> {
    command.current_dir(folder).env("PLATFORMIO_NO_ANSI", "1");
    for name in super::inherited_variables() {
        command.env_remove(name);
    }
    let mut output = String::new();
    let result = crate::process::stream_command_lines_with_timeout_cancelable(
        command,
        QUERY_DEADLINE,
        cancel.clone(),
        &mut |_, line| {
            if output.len() < 262_144 {
                output.push_str(&line);
                output.push('\n');
            }
        },
    );
    match result {
        Ok((_, true)) => Err(refuse(&format!("o prazo de {what} expirou"))),
        Ok((status, false)) if status.success() => Ok(output),
        Ok((status, false)) => Err(refuse(&format!(
            "{what} terminou com {status}: {}",
            output.trim()
        ))),
        Err(error) => Err(refuse(&format!("falha em {what}: {error:?}"))),
    }
}

fn parse_last_object(output: &str) -> Option<Value> {
    output
        .lines()
        .rev()
        .map(str::trim)
        .filter(|line| line.starts_with('{'))
        .find_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(Value::is_object)
}

fn refuse(reason: &str) -> FlashError {
    FlashError::UnsupportedTool {
        message: format!(
            "Não foi possível preparar a gravação com o PlatformIO: {reason}. Nenhum dispositivo foi acessado."
        ),
    }
}
