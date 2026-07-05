//! `JobContext`: the handle a running job's work uses to report progress and
//! output, and to check for cancellation.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::json;

use crate::lsp::EventSender;

/// Final status a job's work reports; `Cancelled` is derived by the manager
/// from the cancel flag, so the work only returns terminal outcomes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JobOutcome {
    /// Finished successfully.
    Success,
    /// Finished with warnings worth surfacing.
    Warning,
    /// Finished with an error.
    Failed,
}

/// Handle given to a job's work closure.
///
/// Progress and output are pushed as `event.job.*` notifications tagged with the
/// job id. Long or looping work must poll [`JobContext::is_cancelled`] so
/// cancellation is cooperative.
#[derive(Debug)]
pub struct JobContext {
    id: String,
    events: EventSender,
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<f64>>>,
}

impl JobContext {
    pub(super) const fn new(
        id: String,
        events: EventSender,
        cancel: Arc<AtomicBool>,
        progress: Arc<Mutex<Option<f64>>>,
    ) -> Self {
        Self {
            id,
            events,
            cancel,
            progress,
        }
    }

    /// The id of the job this context belongs to.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns `true` once `job.cancel` has signalled this job to stop.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Reports progress in `0.0..=1.0` and emits `event.job.progress`.
    pub fn report_progress(&self, fraction: f64, message: Option<&str>) {
        let clamped = fraction.clamp(0.0, 1.0);
        if let Ok(mut slot) = self.progress.lock() {
            *slot = Some(clamped);
        }
        let mut params = json!({
            "jobId": self.id,
            "status": "running",
            "progress": clamped,
        });
        if let (Some(map), Some(text)) = (params.as_object_mut(), message) {
            map.insert("message".to_owned(), json!(text));
        }
        self.emit("event.job.progress", params);
    }

    /// Emits one `event.job.output` log line for this job.
    pub fn emit_output(&self, line: &str) {
        self.emit(
            "event.job.output",
            json!({ "jobId": self.id, "line": line }),
        );
    }

    /// Emits an arbitrary domain event (e.g. `event.build.*`) for this job.
    ///
    /// The caller is responsible for tagging `params` with `jobId` when the UI
    /// needs to correlate the event with the job.
    pub fn emit_event(&self, method: &str, params: serde_json::Value) {
        self.emit(method, params);
    }

    /// A clone of this job's cancellation flag.
    ///
    /// Process-based work shares this with a watcher so it can kill the child on
    /// cancel instead of only polling [`Self::is_cancelled`].
    #[must_use]
    pub fn cancellation(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    fn emit(&self, method: &str, params: serde_json::Value) {
        drop(
            self.events
                .send(JsonRpcRequest::notification(method, Some(params))),
        );
    }
}
