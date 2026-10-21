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
}

/// In-memory by default; only the real persisted core enables file locks.
#[derive(Debug, Default)]
pub(crate) struct Registry {
    active: Mutex<BTreeMap<String, Weak<Reservation>>>,
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
            .map_err(|error| unavailable(device, &error))?;
        let mut active = self.active.lock().map_err(|_| {
            unavailable(
                device,
                &io::Error::other("o registro de uso está indisponível"),
            )
        })?;
        active.retain(|_, reservation| reservation.strong_count() > 0);
        if let Some(reservation) = active.get(&key).and_then(Weak::upgrade) {
            return Err(occupied(device, Some(&reservation.owner)));
        }
        let owner = Owner {
            operation: operation.to_owned(),
            title: title.to_owned(),
            workspace: workspace.map(|root| root.display().to_string()),
            core_pid: std::process::id(),
            process_id: None,
        };
        let files = self
            .directory
            .as_deref()
            .map(|directory| Coordination::acquire(directory, &key, device, &owner))
            .transpose()?;
        let reservation = Arc::new(Reservation { owner, files });
        active.insert(key, Arc::downgrade(&reservation));
        drop(active);
        Ok(Lease { reservation })
    }
}

fn occupied(device: &str, owner: Option<&Owner>) -> JsonRpcError {
    let message = owner.map_or_else(
        || format!("A porta {device} está reservada por outra operação da IDE. Aguarde a conclusão e tente novamente."),
        |owner| {
            let location = owner.workspace.as_deref().map_or_else(String::new, |root| format!(" no projeto {root}"));
            let window = if owner.core_pid == std::process::id() { String::new() }
                else { " (outra instância da IDE)".to_owned() };
            let remedy = if owner.core_pid != std::process::id() {
                "Encerre essa operação na instância que a iniciou ou aguarde a ferramenta encerrar"
            } else if owner.operation == "serial.monitor" {
                "Feche a aba do monitor no Terminal"
            } else {
                "Cancele essa operação no painel Jobs ou aguarde sua conclusão"
            };

            format!("A porta {device} está em uso por {}{location}{window}. {remedy} antes de tentar novamente.", owner.title)
        },
    );
    JsonRpcError::new(
        JsonRpcErrorCode::SerialPortBusy,
        message,
        Some(json!({ "device": device, "owner": owner })),
    )
}

fn unavailable(device: &str, error: &io::Error) -> JsonRpcError {
    JsonRpcError::new(
        JsonRpcErrorCode::InternalError,
        format!(
            "Não foi possível reservar {device} com segurança: {error}. A ferramenta não foi iniciada; confira o acesso ao estado da IDE e tente novamente."
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
