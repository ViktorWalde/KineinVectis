//! One lease per managed serial channel, held by its actual process owner.
//! Discovery and coordination never open the device itself.
// Like platform, these types are crate-only in a restricted module: `pub`
// conflicts with unreachable_pub, while Clippy asks for it here.
#![allow(clippy::redundant_pub_crate)]
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

use kinein_protocol::{JsonRpcError, JsonRpcErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
mod coordination;
use coordination::Coordination;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Owner {
    operation: String,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workspace: Option<String>,
    core_pid: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    process_id: Option<u32>,
    #[serde(default)]
    automatic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Channel {
    Device(String),
    Automatic,
}

/// In-memory by default; only the real persisted core enables file locks.
#[derive(Debug, Default)]
pub(crate) struct Registry {
    active: Mutex<BTreeMap<Channel, Weak<Reservation>>>,
    directory: Option<PathBuf>,
}

/// Unique lifetime guard: moving it to a job/waiter keeps the channel reserved.
#[derive(Debug)]
pub(crate) struct Lease {
    reservation: Arc<Reservation>,
}

#[derive(Debug)]
struct Reservation {
    owner: Owner,
    files: Option<Coordination>,
}

impl Registry {
    pub(crate) fn use_directory(&mut self, directory: PathBuf) {
        self.directory = Some(directory);
    }

    pub(crate) fn acquire(
        &self,
        device: &str,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        let key = crate::platform::serial_device_key(Path::new(device))
            .map_err(|error| unavailable(Some(device), &error))?;
        self.claim(
            Channel::Device(key),
            Some(device),
            operation,
            title,
            workspace,
        )
    }

    /// The tool chooses a port: exclude all managed serial operations.
    pub(crate) fn acquire_automatic(
        &self,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        self.claim(Channel::Automatic, None, operation, title, workspace)
    }

    fn claim(
        &self,
        channel: Channel,
        device: Option<&str>,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        let mut active = self.active.lock().map_err(|_| {
            unavailable(
                device,
                &io::Error::other("o registro de uso está indisponível"),
            )
        })?;
        active.retain(|_, reservation| reservation.strong_count() > 0);
        let conflict = active.iter().find_map(|(key, reservation)| {
            (channel == Channel::Automatic || *key == Channel::Automatic || *key == channel)
                .then(|| reservation.upgrade())
                .flatten()
        });
        if let Some(reservation) = conflict {
            return Err(occupied(device, Some(&reservation.owner)));
        }
        let owner = Owner {
            operation: operation.to_owned(),
            title: title.to_owned(),
            workspace: workspace.map(|root| root.display().to_string()),
            core_pid: std::process::id(),
            process_id: None,
            automatic: channel == Channel::Automatic,
        };
        let key = match &channel {
            Channel::Device(key) => Some(key.as_str()),
            Channel::Automatic => None,
        };
        let files = self
            .directory
            .as_deref()
            .map(|directory| Coordination::acquire(directory, key, device, &owner))
            .transpose()?;
        let reservation = Arc::new(Reservation { owner, files });
        active.insert(channel, Arc::downgrade(&reservation));
        drop(active);
        Ok(Lease { reservation })
    }
}

fn occupied(device: Option<&str>, owner: Option<&Owner>) -> JsonRpcError {
    let channel = device.map_or_else(
        || "uma porta automaticamente".to_owned(),
        |device| format!("a porta {device}"),
    );
    let message = owner.map_or_else(
        || format!("Não foi possível reservar {channel}: há outra operação serial da IDE em andamento. Aguarde sua conclusão ou escolha uma porta livre no painel Embarcados."),
        |owner| {
            let location = owner.workspace.as_deref().map_or_else(String::new, |root| format!(" no projeto {root}"));
            let window = if owner.core_pid == std::process::id() { String::new() }
                else { " (outra instância da IDE)".to_owned() };
            let automatic = if owner.automatic { " com seleção automática; os canais permanecem reservados até sua conclusão" } else { "" };
            let remedy = if owner.core_pid != std::process::id() {
                "Encerre essa operação na instância que a iniciou ou aguarde a ferramenta encerrar"
            } else if owner.operation == "serial.monitor" {
                "Feche a aba do monitor no Terminal"
            } else if matches!(owner.operation.as_str(), "run.start" | "run.script") {
                "Pare a execução ou feche sua aba no Terminal"
            } else {
                "Cancele essa operação no painel Jobs ou aguarde sua conclusão"
            };
            format!("Não foi possível reservar {channel}: {}{location}{window} está em andamento{automatic}. {remedy} antes de tentar novamente.", owner.title)
        },
    );
    JsonRpcError::new(
        JsonRpcErrorCode::SerialPortBusy,
        message,
        Some(
            json!({ "scope": if device.is_some() { "device" } else { "automatic" }, "device": device, "owner": owner }),
        ),
    )
}

fn unavailable(device: Option<&str>, error: &io::Error) -> JsonRpcError {
    let channel = device.unwrap_or("a seleção automática de porta serial");
    JsonRpcError::new(
        JsonRpcErrorCode::InternalError,
        format!(
            "Não foi possível reservar {channel} com segurança: {error}. A ferramenta não foi iniciada; confira o acesso ao estado da IDE e tente novamente."
        ),
        Some(json!({ "device": device })),
    )
}

impl Lease {
    pub(crate) fn invocation(
        &self,
        program: &Path,
        args: &[String],
    ) -> io::Result<(PathBuf, Vec<String>)> {
        self.reservation.files.as_ref().map_or_else(
            || Ok((program.to_path_buf(), args.to_vec())),
            |files| files.invocation(program, args),
        )
    }
}

/// Private exec mode of the core binary; never starts an RPC loop or executor.
pub fn run_serial_tool(arguments: &[std::ffi::OsString]) -> io::Result<()> {
    coordination::run(arguments)
}
