//! Native `pio run -t upload` for ESP32 (52 §14.44).
//!
//! `pio run` builds, may install missing packages and runs the platform's
//! uploader. The preview reads the configuration and the board through
//! `PlatformIO`'s own API (`query.py`), pins the env and refuses
//! what it cannot show: scripts, custom upload commands, other protocols and
//! boards that move to another port during the upload. The intent binds the
//! configuration, the platform and the board by content, never sources or
//! images: changed code builds and flashes on Run.
use std::{
    ffi::OsString,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{FlashIntent, FlashProposalParams};
use serde_json::{Value, json};

use super::native::{NativeRequest, Prepared};
use super::{FlashCommand, FlashError, FlashPlan, chosen_port, snapshot};

mod query;
use query::{Board, Platform, Query};

/// Project options `PlatformIO` 6.2 reads from `PLATFORMIO_*` (`sysenvvar`).
const PROJECT_VARIABLES: [&str; 26] = [
    "PLATFORMIO_BOARDS_DIR",
    "PLATFORMIO_BUILD_CACHE_DIR",
    "PLATFORMIO_BUILD_DIR",
    "PLATFORMIO_BUILD_FLAGS",
    "PLATFORMIO_BUILD_SRC_FILTER",
    "PLATFORMIO_BUILD_SRC_FLAGS",
    "PLATFORMIO_BUILD_UNFLAGS",
    "PLATFORMIO_CACHE_DIR",
    "PLATFORMIO_CORE_DIR",
    "PLATFORMIO_DATA_DIR",
    "PLATFORMIO_DEFAULT_ENVS",
    "PLATFORMIO_EXTRA_SCRIPTS",
    "PLATFORMIO_GLOBALLIB_DIR",
    "PLATFORMIO_INCLUDE_DIR",
    "PLATFORMIO_LIBDEPS_DIR",
    "PLATFORMIO_LIB_DIR",
    "PLATFORMIO_LIB_EXTRA_DIRS",
    "PLATFORMIO_MONITOR_DIR",
    "PLATFORMIO_PACKAGES_DIR",
    "PLATFORMIO_PLATFORMS_DIR",
    "PLATFORMIO_SHARED_DIR",
    "PLATFORMIO_SRC_DIR",
    "PLATFORMIO_TEST_DIR",
    "PLATFORMIO_UPLOAD_FLAGS",
    "PLATFORMIO_UPLOAD_PORT",
    "PLATFORMIO_WORKSPACE_DIR",
];
/// Where the installation and caches live; they do not change what is built.
const KEPT: [&str; 5] = [
    "PLATFORMIO_CORE_DIR",
    "PLATFORMIO_PLATFORMS_DIR",
    "PLATFORMIO_PACKAGES_DIR",
    "PLATFORMIO_CACHE_DIR",
    "PLATFORMIO_BUILD_CACHE_DIR",
];
/// Options whose effect the preview cannot show.
const REFUSED_OPTIONS: [(&str, &str); 3] = [
    (
        "extra_scripts",
        "extra_scripts roda Python do projeto dentro do build e pode trocar o upload",
    ),
    ("upload_command", "upload_command substitui o uploader"),
    (
        "platform_packages",
        "platform_packages troca pacotes da plataforma e instala pela rede",
    ),
];

/// Inherited names removed from the native `pio` and its queries.
#[must_use]
pub fn inherited_variables() -> Vec<OsString> {
    let mut names = super::esptool::inherited_variables();
    names.extend(
        PROJECT_VARIABLES
            .iter()
            .filter(|name| !KEPT.contains(name) && std::env::var_os(name).is_some())
            .map(OsString::from),
    );
    names.sort();
    names.dedup();
    names
}

/// A native `PlatformIO` intent names a port; target, probe, catalogue
/// firmware and build directory belong to other destinations.
///
/// # Errors
/// `InvalidSelection` for fields of another destination kind.
pub(super) fn check_selection(selection: &FlashProposalParams) -> Result<(), FlashError> {
    if selection.target.is_some()
        || selection.probe.is_some()
        || selection.firmware.is_some()
        || selection.build_directory.is_some()
    {
        return Err(FlashError::InvalidSelection {
            message: "target, probe, firmware e buildDirectory não valem para o PlatformIO nativo, que grava o env do projeto"
                .to_owned(),
        });
    }
    Ok(())
}

/// What the checks accepted: the env and the parts the preview shows.
struct Accepted {
    env: String,
    platform: Platform,
    board: Board,
    uploader_version: String,
}

/// Plan, bind and fingerprint a native `pio run -t upload`.
///
/// # Errors
/// Missing port, an unsupported Core, configuration, platform, protocol or
/// board, an esptool configuration file, cancellation or files changing
/// while they are read.
pub(super) fn prepare(
    root: &Path,
    mut request: NativeRequest,
    cancel: &Arc<AtomicBool>,
) -> Result<Prepared, FlashError> {
    check_selection(&request.selection)?;
    let device = chosen_port(request.selection.device.as_deref())?.to_owned();
    let device_key = crate::platform::serial_device_revision(Path::new(&device))
        .map_err(|error| invalid(&error.to_string()))?;
    let (core, python) = query::system_info(&request.tool, cancel)?;
    let query = query::query(&python, root, cancel)?;
    let accepted = check(&query, &core)?;
    // Under pio run the esptool runs in the project directory.
    let home = crate::platform::home_dir();
    if let Some(path) = super::esptool::config::first_declared(root, home.as_deref())
        .map_err(|reason| invalid(&reason))?
    {
        return Err(invalid(&format!(
            "o esptool leria a configuração {} (seção [esptool]), que pode mudar reset, tentativas e tempos fora desta prévia; mova ou renomeie o arquivo, ou use o modo manual",
            path.display()
        )));
    }
    let mut paths = vec![query.config_path.clone()];
    paths.extend(accepted.platform.files.iter().cloned());
    paths.push(accepted.board.manifest.clone());
    paths.push(request.tool.clone());
    let mut contents = Vec::new();
    let mut files = Vec::new();
    for path in paths {
        let (hash, observation) =
            snapshot::file(&path, None, cancel).map_err(|message| invalid(&message))?;
        contents.push(json!([path, hash]));
        files.push(observation);
    }

    let root_text = root.display().to_string();
    let command = FlashCommand::new(
        request.tool.clone(),
        [
            "run",
            "-d",
            &root_text,
            "-e",
            &accepted.env,
            "-t",
            "upload",
            "--upload-port",
            &device,
        ]
        .map(str::to_owned)
        .to_vec(),
    );
    let mut plan = FlashPlan::new(
        "Gravar (PlatformIO)".to_owned(),
        "platformio".to_owned(),
        source(&query, &accepted, &core, &device),
        WARNINGS.map(str::to_owned).to_vec(),
        command,
    );
    request.selection.native = true;
    request.selection.engine = Some("platformio".to_owned());
    request.selection.device = Some(device);
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
            core,
            accepted.platform.version,
            accepted.uploader_version,
            accepted.board.upload,
            query.options,
            query.platformio,
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

const WARNINGS: [&str; 4] = [
    "compila antes de gravar: as imagens gravadas são as do build no momento da execução, não as desta prévia",
    "grava as imagens do build inteiras (bootloader, partições e app), sem pular o que a placa já tem",
    "pode instalar pacotes da plataforma e lib_deps que faltarem, pela rede, como o pio run faz",
    "o pio run apaga o .pio/build quando a configuração ou a lista de fontes muda (project.checksum)",
];

/// The env, platform, protocol and board this slice can promise.
fn check(query: &Query, core: &str) -> Result<Accepted, FlashError> {
    if let Some(error) = &query.error {
        return Err(invalid(&format!(
            "o PlatformIO recusou a configuração: {error}"
        )));
    }
    if query.core != core {
        return Err(invalid(&format!(
            "a consulta respondeu pelo PlatformIO {} e o pio pelo {core}",
            query.core
        )));
    }
    let env = match (query.candidates.as_slice(), query.env.as_ref()) {
        ([only], Some(env)) if only == env => env.clone(),
        _ => {
            return Err(invalid(&format!(
                "o pio run gravaria {} env(s) ({}); deixe um único env em default_envs ou use o modo manual",
                query.candidates.len(),
                query.candidates.join(", ")
            )));
        }
    };
    if query
        .platformio
        .get("extra_configs")
        .is_some_and(|value| !is_empty(value))
    {
        return Err(invalid(
            "extra_configs junta outros arquivos à configuração; use o modo manual",
        ));
    }
    for (option, reason) in REFUSED_OPTIONS {
        if query
            .options
            .get(option)
            .is_some_and(|value| !is_empty(value))
        {
            return Err(invalid(&format!("{reason}; use o modo manual")));
        }
    }
    let frameworks = query
        .options
        .get("framework")
        .map(to_list)
        .unwrap_or_default();
    if frameworks.iter().any(|framework| framework == "espidf") {
        return Err(invalid(
            "o framework espidf pode ligar secure boot ou criptografia de flash no sdkconfig do env; a gravação nativa dele ainda não foi validada",
        ));
    }
    let Some(platform) = &query.platform else {
        return Err(invalid("a consulta não informou a plataforma do env"));
    };
    if platform.name != "espressif32" {
        return Err(invalid(&format!(
            "a plataforma {} tem uploader e destino próprios, ainda sem gravação nativa validada; use o modo manual",
            platform.name
        )));
    }
    let Some(board) = &query.board else {
        return Err(invalid("o env não declara board"));
    };
    if query.protocol.as_deref() != Some("esptool") {
        return Err(invalid(&format!(
            "o upload_protocol {} não grava pela porta serial com o esptool; use o modo manual",
            query.protocol.as_deref().unwrap_or("(nenhum)")
        )));
    }
    for flag in ["use_1200bps_touch", "wait_for_upload_port"] {
        if board.upload.get(flag).is_some_and(python_true) {
            return Err(invalid(&format!(
                "a placa {} usa upload.{flag}: o upload troca de porta durante a gravação",
                board.id
            )));
        }
    }
    let uploader = query.uploader.as_ref();
    let (Some(_), Some(version)) = (
        uploader.and_then(|uploader| uploader.dir.as_ref()),
        uploader.and_then(|uploader| uploader.version.clone()),
    ) else {
        return Err(invalid(
            "o pacote tool-esptool da plataforma não está instalado; compile o projeto uma vez",
        ));
    };
    Ok(Accepted {
        env,
        platform: platform.clone(),
        board: board.clone(),
        uploader_version: version,
    })
}

fn source(query: &Query, accepted: &Accepted, core: &str, device: &str) -> Vec<String> {
    let upload = &accepted.board.upload;
    let text = |value: Option<&Value>, fallback: &str| {
        value.map_or_else(
            || fallback.to_owned(),
            |value| {
                value
                    .as_str()
                    .map_or_else(|| value.to_string(), str::to_owned)
            },
        )
    };
    let speed = query.options.get("upload_speed").map_or_else(
        || format!("{} (placa)", text(upload.get("speed"), "padrão do esptool")),
        |speed| format!("{} (upload_speed do platformio.ini)", text(Some(speed), "")),
    );
    let why = if query.default_envs.is_empty() {
        "o único env do platformio.ini"
    } else {
        "o único default_envs"
    };
    let mut lines = vec![
        "motor: pio run -t upload (PlatformIO, resolvido para a intenção nativa)".to_owned(),
        format!(
            "PlatformIO Core {core}; plataforma {} {}; esptool do pacote tool-esptool {}",
            accepted.platform.name, accepted.platform.version, accepted.uploader_version
        ),
        format!("env: {} ({why}, fixado com -e)", accepted.env),
        format!(
            "placa: {} ({})",
            accepted.board.id,
            accepted.board.manifest.display()
        ),
        format!(
            "porta: {device} (escolhida em Gravar e depurar; --upload-port vence o upload_port do platformio.ini)"
        ),
        format!(
            "baud: {speed}; reset: {} antes e {} depois (placa)",
            text(upload.get("before_reset"), "default-reset"),
            text(upload.get("after_reset"), "hard-reset")
        ),
    ];
    if let Some(flags) = query
        .options
        .get("upload_flags")
        .filter(|flags| !is_empty(flags))
    {
        lines.push(format!(
            "upload_flags do platformio.ini, antes dos argumentos do esptool: {}",
            to_list(flags).join(" ")
        ));
    }
    let inherited = inherited_variables();
    lines.push(if inherited.is_empty() {
        "ambiente: nenhuma variável ESPTOOL_* ou PLATFORMIO_* de opção do projeto herdada"
            .to_owned()
    } else {
        format!(
            "ambiente: {} não chegam ao pio",
            inherited
                .iter()
                .map(|name| name.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    lines.push("configuração: nenhum esptool.cfg, setup.cfg ou tox.ini com [esptool] na raiz, na pasta de configuração do usuário ou no home".to_owned());
    lines
}

/// Python truthiness, which the platform's `BeforeUpload` applies: a
/// `board_upload.*` key the manifest lacks stays a string, and any non-empty
/// string, even "no", makes the upload touch or wait for another port.
fn python_true(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|number| number != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
    }
}

fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(items) => items.is_empty(),
        _ => false,
    }
}

fn to_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map_or_else(|| item.to_string(), str::to_owned)
            })
            .collect(),
        Value::String(text) => vec![text.clone()],
        other => vec![other.to_string()],
    }
}

fn invalid(message: &str) -> FlashError {
    FlashError::InvalidContext {
        message: format!(
            "Não foi possível validar a gravação nativa: {message}. Nenhuma gravação foi iniciada."
        ),
    }
}
