//! `JobManager`: spawns async jobs, tracks them and drives their lifecycle.
//!
//! Each `spawn` runs the work on its own thread, emits `event.job.created`
//! before starting and `event.job.finished` when it ends, and keeps a record so
//! `job.list` and `job.cancel` can see and stop it. Cancellation is cooperative:
//! the work must poll [`JobContext::is_cancelled`].

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

use kinein_protocol::{JobInfo, JobRisk, JobStatus, JsonRpcRequest};
use serde_json::json;

use super::context::{JobContext, JobOutcome};
use crate::lsp::EventSender;

/// Live record of one job kept in the manager's registry.
#[derive(Debug)]
struct JobRecord {
    seq: u64,
    kind: String,
    title: String,
    risk: JobRisk,
    can_cancel: bool,
    status: Arc<Mutex<JobStatus>>,
    progress: Arc<Mutex<Option<f64>>>,
    cancel: Arc<AtomicBool>,
}

impl JobRecord {
    fn info(&self, id: &str) -> JobInfo {
        JobInfo {
            id: id.to_owned(),
            kind: self.kind.clone(),
            title: self.title.clone(),
            status: self
                .status
                .lock()
                .map_or(JobStatus::Running, |guard| *guard),
            progress: self.progress.lock().ok().and_then(|guard| *guard),
            can_cancel: self.can_cancel,
            risk: self.risk,
        }
    }
}

/// Registry and lifecycle driver for asynchronous jobs.
#[derive(Debug)]
pub struct JobManager {
    events: EventSender,
    next_id: AtomicU64,
    jobs: Arc<Mutex<HashMap<String, JobRecord>>>,
}

impl JobManager {
    /// Creates a manager that pushes `event.job.*` through `events`.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            next_id: AtomicU64::new(1),
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns `work` as a job and returns its id immediately.
    ///
    /// `work` runs on its own thread; its returned [`JobOutcome`] becomes the
    /// final status unless the job was cancelled first, in which case the status
    /// is `Cancelled`.
    pub fn spawn<F>(
        &self,
        kind: impl Into<String>,
        title: impl Into<String>,
        risk: JobRisk,
        can_cancel: bool,
        work: F,
    ) -> String
    where
        F: FnOnce(&JobContext) -> JobOutcome + Send + 'static,
    {
        let seq = self.next_id.fetch_add(1, Ordering::SeqCst);
        let id = format!("job_{seq}");
        let cancel = Arc::new(AtomicBool::new(false));
        let status = Arc::new(Mutex::new(JobStatus::Running));
        let progress = Arc::new(Mutex::new(None));

        let record = JobRecord {
            seq,
            kind: kind.into(),
            title: title.into(),
            risk,
            can_cancel,
            status: Arc::clone(&status),
            progress: Arc::clone(&progress),
            cancel: Arc::clone(&cancel),
        };
        let created = record.info(&id);
        if let Ok(mut jobs) = self.jobs.lock() {
            jobs.insert(id.clone(), record);
        }
        drop(self.events.send(JsonRpcRequest::notification(
            "event.job.created",
            Some(json!(created)),
        )));

        let context = JobContext::new(id.clone(), self.events.clone(), cancel, progress);
        let events = self.events.clone();
        let finished_id = id.clone();
        thread::spawn(move || {
            let outcome = work(&context);
            let final_status = if context.is_cancelled() {
                JobStatus::Cancelled
            } else {
                match outcome {
                    JobOutcome::Success => JobStatus::Success,
                    JobOutcome::Warning => JobStatus::Warning,
                    JobOutcome::Failed => JobStatus::Failed,
                }
            };
            if let Ok(mut guard) = status.lock() {
                *guard = final_status;
            }
            drop(events.send(JsonRpcRequest::notification(
                "event.job.finished",
                Some(json!({ "jobId": finished_id, "status": final_status })),
            )));
        });

        id
    }

    /// Every known job in stable creation order.
    #[must_use]
    pub fn list(&self) -> Vec<JobInfo> {
        let Ok(jobs) = self.jobs.lock() else {
            return Vec::new();
        };
        let mut entries: Vec<(u64, JobInfo)> = jobs
            .iter()
            .map(|(id, record)| (record.seq, record.info(id)))
            .collect();
        entries.sort_by_key(|(seq, _info)| *seq);
        entries.into_iter().map(|(_seq, info)| info).collect()
    }

    /// Signals a running, cancelable job to stop.
    ///
    /// Returns `true` only when the job exists, exposes cancellation and is still
    /// running; already-finished or unknown jobs return `false`.
    pub fn cancel(&self, job_id: &str) -> bool {
        let Ok(jobs) = self.jobs.lock() else {
            return false;
        };
        let Some(record) = jobs.get(job_id) else {
            return false;
        };
        if !record.can_cancel {
            return false;
        }
        let running = record
            .status
            .lock()
            .is_ok_and(|guard| *guard == JobStatus::Running);
        if running {
            record.cancel.store(true, Ordering::SeqCst);
        }
        running
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc::{self, Receiver},
        time::Duration,
    };

    use kinein_protocol::{JobRisk, JobStatus, JsonRpcRequest};

    use super::super::context::JobOutcome;
    use super::JobManager;

    fn collect_until_finished(receiver: &Receiver<JsonRpcRequest>) -> Vec<JsonRpcRequest> {
        let mut events = Vec::new();
        loop {
            let event = receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("evento de job dentro do timeout");
            let done = event.method == "event.job.finished";
            events.push(event);
            if done {
                return events;
            }
        }
    }

    #[test]
    fn spawn_emits_lifecycle_and_reports_progress() {
        let (sender, receiver) = mpsc::channel();
        let manager = JobManager::new(sender);

        let id = manager.spawn("build", "cargo build", JobRisk::Medium, true, |ctx| {
            ctx.report_progress(0.5, Some("compilando"));
            JobOutcome::Success
        });

        let events = collect_until_finished(&receiver);
        assert_eq!(events[0].method, "event.job.created");
        assert_eq!(events[0].params.as_ref().unwrap()["id"], id);

        let progress = events
            .iter()
            .find(|event| event.method == "event.job.progress")
            .expect("evento de progresso");
        assert_eq!(progress.params.as_ref().unwrap()["progress"], 0.5);
        assert_eq!(progress.params.as_ref().unwrap()["message"], "compilando");

        let finished = events.last().unwrap().params.as_ref().unwrap();
        assert_eq!(finished["jobId"], id);
        assert_eq!(finished["status"], "success");
    }

    #[test]
    fn cancel_signals_running_job_and_finishes_cancelled() {
        let (sender, receiver) = mpsc::channel();
        let manager = JobManager::new(sender);

        let id = manager.spawn("run", "sleep loop", JobRisk::Low, true, |ctx| {
            while !ctx.is_cancelled() {
                std::thread::sleep(Duration::from_millis(10));
            }
            JobOutcome::Success
        });

        // The job is running until we cancel it.
        assert!(manager.cancel(&id));

        let events = collect_until_finished(&receiver);
        let finished = events.last().unwrap().params.as_ref().unwrap();
        assert_eq!(finished["status"], "cancelled");

        // A second cancel on the now-finished job is a no-op.
        assert!(!manager.cancel(&id));
        // Unknown ids never cancel.
        assert!(!manager.cancel("job_999"));
    }

    #[test]
    fn list_reflects_jobs_in_creation_order() {
        let (sender, _receiver) = mpsc::channel();
        let manager = JobManager::new(sender);
        assert!(manager.list().is_empty());

        let first = manager.spawn("a", "first", JobRisk::Low, false, |_ctx| {
            JobOutcome::Success
        });
        let second = manager.spawn("b", "second", JobRisk::Low, false, |_ctx| {
            JobOutcome::Failed
        });

        // Wait for both to finish so statuses are terminal.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while manager
            .list()
            .iter()
            .any(|job| job.status == JobStatus::Running)
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }

        let jobs = manager.list();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id, first);
        assert_eq!(jobs[0].kind, "a");
        assert_eq!(jobs[1].id, second);
        assert!(!jobs[1].can_cancel);
    }
}
