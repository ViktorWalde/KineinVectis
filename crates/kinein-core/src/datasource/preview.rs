//! Live preview registry: public destination and a one-use channel, no secrets.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use kinein_protocol::{
    DataSourceOperationContext, DataSourcePreviewDecideParams, DataSourcePreviewDecision,
};
use tokio::sync::oneshot;

/// Maximum time the author has to decide, from the ready preview event.
pub const DECISION_SECONDS: u32 = 60;
const MAX_ACTIVE: usize = 4;

#[derive(Debug)]
struct Entry {
    context: DataSourceOperationContext,
    token: String,
    sender: Option<oneshot::Sender<DataSourcePreviewDecision>>,
    ready: Option<(String, Instant)>,
    decision: Option<DataSourcePreviewDecision>,
}

#[derive(Debug)]
struct Registry {
    epoch: String,
    serial: u64,
    entries: HashMap<String, Entry>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            epoch: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            serial: 0,
            entries: HashMap::new(),
        }
    }
}

/// Per-core registry; workers hold a lease, workspace transitions clear entries.
#[derive(Debug, Clone, Default)]
pub struct Session {
    registry: Arc<Mutex<Registry>>,
}

/// A reserved preview slot, removed on every worker exit path.
#[derive(Debug)]
pub struct Lease {
    id: String,
    session: Session,
    receiver: oneshot::Receiver<DataSourcePreviewDecision>,
}

impl Session {
    /// Reserves capacity before connecting; one live preview per destination.
    pub fn reserve(
        &self,
        context: DataSourceOperationContext,
        token: String,
    ) -> Result<Lease, &'static str> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|_| "A sessão de prévia está indisponível.")?;
        if registry.entries.len() >= MAX_ACTIVE
            || registry.entries.values().any(|entry| {
                entry.context.workspace == context.workspace
                    && entry.context.profile.name == context.profile.name
            })
        {
            return Err("Já existe uma prévia neste destino ou o limite de prévias foi atingido.");
        }
        registry.serial += 1;
        let id = format!("preview-{}-{}", registry.epoch, registry.serial);
        let (sender, receiver) = oneshot::channel();
        registry.entries.insert(
            id.clone(),
            Entry {
                context,
                token,
                sender: Some(sender),
                ready: None,
                decision: None,
            },
        );
        drop(registry);
        Ok(Lease {
            id,
            session: self.clone(),
            receiver,
        })
    }

    /// Accepts exactly one ready decision matching the original public context.
    pub fn decide(&self, request: &DataSourcePreviewDecideParams) -> Result<String, &'static str> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|_| "A sessão de prévia está indisponível.")?;
        let entry = registry
            .entries
            .get(&request.preview_id)
            .ok_or("A prévia não existe mais; execute novamente.")?;
        if entry.sender.is_none() {
            return Err("A prévia já recebeu uma decisão ou foi encerrada.");
        }
        if entry.context != request.expected_context
            || entry.context.profile.name != request.name
            || entry.token != request.client_context
        {
            return Err("O contexto da prévia mudou; execute novamente.");
        }
        let (job_id, deadline) = entry
            .ready
            .as_ref()
            .ok_or("A prévia ainda não está pronta.")?;
        if Instant::now() >= *deadline {
            if let Some(entry) = registry.entries.get_mut(&request.preview_id) {
                entry.sender.take();
            }
            return Err("O prazo da prévia expirou; as alterações serão desfeitas.");
        }
        let job_id = job_id.clone();
        registry
            .entries
            .get_mut(&request.preview_id)
            .and_then(|entry| entry.sender.take())
            .ok_or("A decisão da prévia já foi aceita.")?
            .send(request.decision)
            .map_err(|_| "A prévia já foi encerrada.")?;
        if let Some(entry) = registry.entries.get_mut(&request.preview_id) {
            entry.decision = Some(request.decision);
        }
        drop(registry);
        Ok(job_id)
    }

    /// Drops pending channels; a worker rolls back unless a decision was accepted.
    pub fn clear(&self) {
        if let Ok(mut registry) = self.registry.lock() {
            for entry in registry.entries.values_mut() {
                entry.sender.take();
            }
        }
    }

    /// Saving/removing a profile invalidates its pending preview.
    pub fn revoke(&self, workspace: &std::path::Path, name: &str) {
        if let Ok(mut registry) = self.registry.lock() {
            for entry in registry.entries.values_mut() {
                if std::path::Path::new(&entry.context.workspace) == workspace
                    && entry.context.profile.name == name
                {
                    entry.sender.take();
                }
            }
        }
    }

    /// Job cancellation consumes an undecided channel before a later COMMIT request.
    pub fn cancel_job(&self, job_id: &str) {
        if let Ok(mut registry) = self.registry.lock() {
            for entry in registry.entries.values_mut() {
                if entry.ready.as_ref().is_some_and(|(id, _)| id == job_id) {
                    entry.sender.take();
                }
            }
        }
    }

    /// An accepted decision is no longer cancellable by the job UI.
    #[must_use]
    pub fn decided_job(&self, job_id: &str) -> bool {
        self.registry.lock().is_ok_and(|registry| {
            registry.entries.values().any(|entry| {
                entry.decision.is_some() && entry.ready.as_ref().is_some_and(|(id, _)| id == job_id)
            })
        })
    }
}

impl Lease {
    /// Public opaque identifier of this transaction.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Starts the decision clock only after execution has finished.
    pub fn ready(&self, job_id: &str) -> Result<(), &'static str> {
        let mut registry = self
            .session
            .registry
            .lock()
            .map_err(|_| "A sessão de prévia está indisponível.")?;
        let entry = registry
            .entries
            .get_mut(&self.id)
            .ok_or("O contexto da prévia foi encerrado.")?;
        if entry.ready.is_some() || entry.sender.is_none() {
            return Err("A prévia já está pronta.");
        }
        entry.ready = Some((
            job_id.to_owned(),
            Instant::now() + Duration::from_secs(u64::from(DECISION_SECONDS)),
        ));
        drop(registry);
        Ok(())
    }

    /// Awaits a decision while the asynchronous `PostgreSQL` driver keeps running.
    pub async fn decision(
        &mut self,
    ) -> Result<DataSourcePreviewDecision, oneshot::error::RecvError> {
        (&mut self.receiver).await
    }

    /// Before readiness, loss of the sender means the destination was discarded.
    #[must_use]
    pub fn discarded(&mut self) -> bool {
        matches!(
            self.receiver.try_recv(),
            Err(oneshot::error::TryRecvError::Closed)
        )
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut registry) = self.session.registry.lock() {
            registry.entries.remove(&self.id);
        }
    }
}

#[cfg(test)]
#[path = "preview_tests.rs"]
mod tests;
