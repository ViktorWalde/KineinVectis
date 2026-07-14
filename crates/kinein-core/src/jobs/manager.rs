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
    time::{Duration, Instant},
};

use kinein_protocol::{JobInfo, JobRisk, JobStatus, JsonRpcRequest};
use serde_json::json;

use super::context::{JobContext, JobOutcome};
use crate::lsp::EventSender;

/// Maximum number of jobs kept in memory for `job.list`.
///
/// Active jobs are never removed. When the registry grows beyond this cap, the
/// oldest terminal jobs are pruned first.
const MAX_RETAINED_JOBS: usize = 100;

/// Janela de espera cooperativa no shutdown: os runners observam o cancel a
/// cada ~50ms (`process.rs`) e matam o filho; damos esse tempo antes do core
/// sair para não deixar `cargo`/`cmake`/`lldb` órfãos (fatia M4.3b).
const SHUTDOWN_DRAIN: Duration = Duration::from_millis(500);

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
            prune_finished_jobs(&mut jobs);
        }
        drop(self.events.send(JsonRpcRequest::notification(
            "event.job.created",
            Some(json!(created)),
        )));

        let context = JobContext::new(id.clone(), self.events.clone(), cancel, progress);
        let events = self.events.clone();
        let finished_id = id.clone();
        let jobs = Arc::clone(&self.jobs);
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
            if let Ok(mut guard) = jobs.lock() {
                prune_finished_jobs(&mut guard);
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
    /// running; already-finished, already-cancel-requested or unknown jobs
    /// return `false`.
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
        let Ok(mut status) = record.status.lock() else {
            return false;
        };
        if *status != JobStatus::Running {
            return false;
        }
        *status = JobStatus::CancelRequested;
        record.cancel.store(true, Ordering::SeqCst);

        let mut params = json!({
            "jobId": job_id,
            "status": JobStatus::CancelRequested,
            "message": "cancelamento solicitado",
        });
        if let (Some(map), Some(progress)) = (
            params.as_object_mut(),
            record.progress.lock().ok().and_then(|guard| *guard),
        ) {
            map.insert("progress".to_owned(), json!(progress));
        }
        drop(self.events.send(JsonRpcRequest::notification(
            "event.job.progress",
            Some(params),
        )));

        true
    }

    /// Sinaliza cancelamento a TODOS os jobs canceláveis ainda em execução
    /// (fatia M4.3b). Devolve quantos foram sinalizados; não bloqueia — o
    /// `Drop` faz a espera (drain) no shutdown.
    pub fn cancel_all(&self) -> usize {
        let ids: Vec<String> = match self.jobs.lock() {
            Ok(jobs) => jobs.keys().cloned().collect(),
            Err(_poisoned) => return 0,
        };
        let mut signalled = 0;
        for id in &ids {
            if self.cancel(id) {
                signalled += 1;
            }
        }
        signalled
    }

    /// `true` quando nenhum job está mais `Running`/`CancelRequested`.
    fn all_jobs_settled(&self) -> bool {
        let Ok(jobs) = self.jobs.lock() else {
            return true;
        };
        jobs.values().all(|record| {
            record.status.lock().is_ok_and(|status| {
                !matches!(*status, JobStatus::Running | JobStatus::CancelRequested)
            })
        })
    }
}

impl Drop for JobManager {
    fn drop(&mut self) {
        // Shutdown (core.shutdown, EOF da UI morta, ou unwind): cancela os
        // jobs vivos e dá uma janela curta para os runners matarem o processo
        // filho, senão ele fica órfão. Barato quando não há job (sai na hora).
        self.cancel_all();
        if self.all_jobs_settled() {
            return;
        }
        let deadline = Instant::now() + SHUTDOWN_DRAIN;
        while Instant::now() < deadline {
            if self.all_jobs_settled() {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn prune_finished_jobs(jobs: &mut HashMap<String, JobRecord>) {
    if jobs.len() <= MAX_RETAINED_JOBS {
        return;
    }

    let mut removable = jobs
        .iter()
        .filter_map(|(id, record)| {
            let is_terminal = record
                .status
                .lock()
                .is_ok_and(|status| status.is_terminal());
            is_terminal.then(|| (record.seq, id.clone()))
        })
        .collect::<Vec<_>>();
    removable.sort_by_key(|(seq, _id)| *seq);

    let mut excess = jobs.len().saturating_sub(MAX_RETAINED_JOBS);
    for (_seq, id) in removable {
        if excess == 0 {
            break;
        }
        if jobs.remove(&id).is_some() {
            excess -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc, Mutex,
            mpsc::{self, Receiver},
        },
        time::Duration,
    };

    use kinein_protocol::{JobRisk, JobStatus, JsonRpcRequest};

    use super::super::context::JobOutcome;
    use super::{JobManager, MAX_RETAINED_JOBS};

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
        let (seen_cancel_sender, seen_cancel_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let manager = JobManager::new(sender);

        let id = manager.spawn("run", "sleep loop", JobRisk::Low, true, move |ctx| {
            while !ctx.is_cancelled() {
                std::thread::sleep(Duration::from_millis(10));
            }
            seen_cancel_sender
                .send(())
                .expect("sinaliza que viu cancelamento");
            release_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("teste libera finalizacao");
            JobOutcome::Success
        });

        // The job is running until we cancel it.
        assert!(manager.cancel(&id));
        seen_cancel_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("job viu cancelamento");
        assert_eq!(manager.list()[0].status, JobStatus::CancelRequested);
        release_sender.send(()).expect("libera job");

        let events = collect_until_finished(&receiver);
        let cancel_requested = events
            .iter()
            .find(|event| {
                event.method == "event.job.progress"
                    && event.params.as_ref().unwrap()["status"] == "cancelRequested"
            })
            .expect("evento de cancelamento solicitado");
        assert_eq!(cancel_requested.params.as_ref().unwrap()["jobId"], id);

        let finished = events.last().unwrap().params.as_ref().unwrap();
        assert_eq!(finished["status"], "cancelled");

        // A second cancel on the now-finished job is a no-op.
        assert!(!manager.cancel(&id));
        // Unknown ids never cancel.
        assert!(!manager.cancel("job_999"));
    }

    #[test]
    fn cancel_all_signals_every_running_cancelable_job() {
        let (sender, _receiver) = mpsc::channel();
        let (seen_sender, seen_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel::<()>();
        let manager = JobManager::new(sender);
        let release = Arc::new(Mutex::new(release_receiver));

        // Dois jobs canceláveis em loop até verem o cancel (M4.3b).
        for _ in 0..2 {
            let seen = seen_sender.clone();
            let release = Arc::clone(&release);
            manager.spawn("run", "loop", JobRisk::Low, true, move |ctx| {
                while !ctx.is_cancelled() {
                    std::thread::sleep(Duration::from_millis(5));
                }
                seen.send(()).expect("sinaliza cancel visto");
                let _ = release.lock().unwrap().recv_timeout(Duration::from_secs(5));
                JobOutcome::Success
            });
        }

        // cancel_all sinaliza os DOIS jobs vivos.
        assert_eq!(manager.cancel_all(), 2);
        for _ in 0..2 {
            seen_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("job viu cancelamento");
        }
        // Nada mais a sinalizar numa segunda chamada.
        assert_eq!(manager.cancel_all(), 0);
        release_sender.send(()).expect("libera 1");
        release_sender.send(()).expect("libera 2");
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

    #[test]
    fn list_prunes_old_finished_jobs() {
        let (sender, receiver) = mpsc::channel();
        let manager = JobManager::new(sender);

        for index in 0..(MAX_RETAINED_JOBS + 5) {
            manager.spawn(
                "finished",
                format!("job {index}"),
                JobRisk::Low,
                false,
                |_ctx| JobOutcome::Success,
            );
            loop {
                let event = receiver
                    .recv_timeout(Duration::from_secs(5))
                    .expect("job finaliza dentro do timeout");
                if event.method == "event.job.finished" {
                    break;
                }
            }
        }

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            let jobs = manager.list();
            if jobs.len() <= MAX_RETAINED_JOBS && jobs.iter().all(|job| job.status.is_terminal()) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        let jobs = manager.list();
        assert_eq!(jobs.len(), MAX_RETAINED_JOBS);
        assert_eq!(jobs[0].id, "job_6");
        assert!(jobs.iter().all(|job| job.status.is_terminal()));
    }
}
