//! One lease per managed serial channel or debug probe, held by its actual
//! process owner. Discovery and coordination never open the device itself.
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
    /// probe-rs selector, canonical; keyed apart from serial nodes.
    Probe(String),
    Usb(String),
    Automatic,
}

/// What a refusal names: a port, a probe or the automatic selection.
#[derive(Debug, Clone, Copy)]
pub(super) struct Resource<'a> {
    kind: Kind,
    name: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Serial,
    Probe,
    Usb,
}

impl Resource<'_> {
    fn label(self) -> String {
        match (self.kind, self.name) {
            (Kind::Probe, Some(name)) => format!("a sonda {name}"),
            (Kind::Usb, Some(name)) => format!("o dispositivo USB {name}"),
            (_, Some(name)) => format!("a porta {name}"),
            (_, None) => "uma porta automaticamente".to_owned(),
        }
    }

    const fn scope(self) -> &'static str {
        match (self.kind, self.name) {
            (Kind::Probe, _) => "probe",
            (Kind::Usb, _) => "usb",
            (Kind::Serial, Some(_)) => "device",
            (Kind::Serial, None) => "automatic",
        }
    }
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
        let resource = Resource {
            kind: Kind::Serial,
            name: Some(device),
        };
        let key = crate::platform::serial_device_key(Path::new(device))
            .map_err(|error| unavailable(resource, &error))?;
        self.claim(Channel::Device(key), resource, operation, title, workspace)
    }

    /// A debug probe by its canonical selector: excludes the same probe and
    /// automatic port selection, never an explicit serial port.
    pub(crate) fn acquire_probe(
        &self,
        selector: &str,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        let resource = Resource {
            kind: Kind::Probe,
            name: Some(selector),
        };
        self.claim(
            Channel::Probe(format!("probe-rs:{selector}")),
            resource,
            operation,
            title,
            workspace,
        )
    }

    /// USB identity shared by every alternative of the same physical selection.
    pub(crate) fn acquire_usb(
        &self,
        identity: &str,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        self.claim(
            Channel::Usb(format!("usb:{identity}")),
            Resource {
                kind: Kind::Usb,
                name: Some(identity),
            },
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
        let resource = Resource {
            kind: Kind::Serial,
            name: None,
        };
        self.claim(Channel::Automatic, resource, operation, title, workspace)
    }

    fn claim(
        &self,
        channel: Channel,
        resource: Resource<'_>,
        operation: &str,
        title: &str,
        workspace: Option<&Path>,
    ) -> Result<Lease, JsonRpcError> {
        let mut active = self.active.lock().map_err(|_| {
            unavailable(
                resource,
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
            return Err(occupied(resource, Some(&reservation.owner)));
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
            Channel::Device(key) | Channel::Probe(key) | Channel::Usb(key) => Some(key.as_str()),
            Channel::Automatic => None,
        };
        let files = self
            .directory
            .as_deref()
            .map(|directory| Coordination::acquire(directory, key, resource, &owner))
            .transpose()?;
        let reservation = Arc::new(Reservation { owner, files });
        active.insert(channel, Arc::downgrade(&reservation));
        drop(active);
        Ok(Lease { reservation })
    }
}

fn occupied(resource: Resource<'_>, owner: Option<&Owner>) -> JsonRpcError {
    let channel = resource.label();
    let message = owner.map_or_else(
        || format!("Não foi possível reservar {channel}: há outra operação serial da IDE em andamento. Aguarde sua conclusão ou escolha uma porta livre em Gravar e depurar."),
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
            } else if owner.operation == "debug.start" {
                "Pare a depuração"
            } else {
                "Cancele essa operação no painel Jobs ou aguarde sua conclusão"
            };
            format!("Não foi possível reservar {channel}: {}{location}{window} está em andamento{automatic}. {remedy} antes de tentar novamente.", owner.title)
        },
    );
    JsonRpcError::new(
        JsonRpcErrorCode::SerialPortBusy,
        message,
        Some(json!({ "scope": resource.scope(), "device": resource.name, "owner": owner })),
    )
}

fn unavailable(resource: Resource<'_>, error: &io::Error) -> JsonRpcError {
    let channel = resource.name.map_or_else(
        || "a seleção automática de porta serial".to_owned(),
        |_| resource.label(),
    );
    JsonRpcError::new(
        JsonRpcErrorCode::InternalError,
        format!(
            "Não foi possível reservar {channel} com segurança: {error}. A ferramenta não foi iniciada; confira o acesso ao estado da IDE e tente novamente."
        ),
        Some(json!({ "device": resource.name })),
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
