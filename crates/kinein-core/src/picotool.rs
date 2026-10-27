//! Bounded RP2040 BOOTSEL queries; no firmware transfer, force or reboot.
mod parser;
use kinein_protocol::{PicoDevice, PicoListResult};
use std::{
    path::Path,
    process::Command,
    sync::{
        Arc, Mutex, MutexGuard, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

struct Query<'a> {
    program: &'a Path,
    deadline: Instant,
    cancel: &'a Arc<AtomicBool>,
    result: PicoListResult,
    lease: Option<&'a crate::serial::ownership::Lease>,
}

impl Query<'_> {
    fn command(&mut self, args: &[&str]) -> Result<String, String> {
        if self.cancel.load(Ordering::SeqCst) {
            return Err("consulta cancelada".into());
        }
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or("prazo total de 5 s expirou")?;
        let original: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
        let (program, arguments) = if let Some(lease) = self.lease {
            lease
                .invocation(self.program, &original)
                .map_err(|e| format!("não foi possível proteger a consulta: {e}"))?
        } else {
            (self.program.to_path_buf(), original)
        };
        let mut command = Command::new(program);
        command.args(arguments).env("LC_ALL", "C");
        let answer = crate::probe::query::run(command, remaining, self.cancel);
        let prefix = format!("$ picotool {}\n", args.join(" "));
        let output = format!("{prefix}{}", answer.output);
        let remaining = 65_536 - self.result.raw_output.len();
        let mut end = remaining.min(output.len());
        while !output.is_char_boundary(end) {
            end -= 1;
        }
        self.result.raw_output.push_str(&output[..end]);
        if end < output.len() {
            return Err("saída conjunta excedeu 64 KiB".into());
        }
        if let Some(failure) = answer.failure {
            return Err(match failure {
                crate::probe::query::Failure::Spawn(reason) => {
                    self.result.tool_available = false;
                    format!("não pôde iniciar: {reason}")
                }
                crate::probe::query::Failure::Incomplete(reason) => reason,
            });
        }
        Ok(answer.output)
    }

    fn discover(&mut self, selected: Option<&PicoDevice>) -> Result<(), String> {
        let version = parser::version(&self.command(&["version"])?)?;
        self.result.version = Some(version);
        let args = ["info", "--device", "--vid", "0x2e8a", "--pid", "0x0003"];
        let devices = if let Some(selected) = selected {
            vec![selected.clone()]
        } else {
            parser::devices(&self.command(&args)?)?
        };
        for device in &devices {
            let mut selected = args.to_vec();
            selected.extend(["--ser", &device.serial_selector]);
            let confirmed = parser::devices(&self.command(&selected)?)?;
            if confirmed.as_slice() != std::slice::from_ref(device) {
                return Err(
                    "seletor --ser não confirmou um único RP2040 com o mesmo ID e tamanho".into(),
                );
            }
        }
        self.result.devices = devices;
        self.result.complete = true;
        Ok(())
    }
}

/// Query within a caller-owned deadline (including any wait for the old query).
#[must_use]
pub(crate) fn list(
    program: Option<&Path>,
    deadline: Instant,
    cancel: &Arc<AtomicBool>,
    lease: Option<&crate::serial::ownership::Lease>,
) -> PicoListResult {
    query(program, deadline, cancel, lease, None)
}

/// Recheck one explicit native destination under the caller's family lease.
pub(crate) fn confirm(
    program: &Path,
    selected: &PicoDevice,
    deadline: Instant,
    cancel: &Arc<AtomicBool>,
    lease: &crate::serial::ownership::Lease,
) -> PicoListResult {
    query(Some(program), deadline, cancel, Some(lease), Some(selected))
}

fn query(
    program: Option<&Path>,
    deadline: Instant,
    cancel: &Arc<AtomicBool>,
    lease: Option<&crate::serial::ownership::Lease>,
    selected: Option<&PicoDevice>,
) -> PicoListResult {
    let Some(program) = program else {
        return failed(
            None,
            "picotool não encontrado no PATH; instale pelo catálogo antes de consultar",
        );
    };
    let mut query = Query {
        program,
        deadline,
        cancel,
        lease,
        result: PicoListResult {
            devices: Vec::new(),
            complete: false,
            tool_available: true,
            tool: Some(program.display().to_string()),
            version: None,
            raw_output: String::new(),
            hint: None,
        },
    };
    if let Err(reason) = query.discover(selected) {
        query.result.devices.clear();
        query.result.complete = false;
        query.result.hint = Some(format!(
            "Consulta RP2040 não verificada: {reason}. Confira a saída; entre em BOOTSEL e consulte novamente."
        ));
    }
    query.result
}

pub(crate) fn failed(program: Option<&Path>, reason: &str) -> PicoListResult {
    PicoListResult {
        devices: Vec::new(),
        complete: false,
        tool_available: program.is_some(),
        tool: program.map(|p| p.display().to_string()),
        version: None,
        raw_output: String::new(),
        hint: Some(reason.into()),
    }
}

/// One automatic reservation across successive queries; not a parallel registry.
#[derive(Debug)]
pub(crate) struct Pending {
    pub cancel: Arc<AtomicBool>,
    pub gate: Arc<Mutex<()>>,
    pub lease: Arc<crate::serial::ownership::Lease>,
}

pub(crate) fn exclusive<'a>(
    gate: &'a Mutex<()>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Option<MutexGuard<'a, ()>> {
    while !cancel.load(Ordering::SeqCst) && Instant::now() < deadline {
        match gate.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::Poisoned(error)) => return Some(error.into_inner()),
            Err(TryLockError::WouldBlock) => {}
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    None
}

#[cfg(test)]
mod tests;
