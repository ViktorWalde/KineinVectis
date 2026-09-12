//! Job system payloads (`job.list`, `job.cancel`, `event.job.*`).
//!
//! A job is a long-running operation the core executes asynchronously: it is
//! tracked by id, reports progress through events and can be cancelled. See
//! `DocsPublic/especificacoes/arquitetura-interna-core-ipc-jobs.md` for the
//! target model and `DocsPublic/arquitetura/ARCHITECTURE.md` for why this is built early.

use serde::{Deserialize, Serialize};

/// Lifecycle status of a job.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    /// Accepted but not started yet.
    Queued,
    /// Currently running.
    Running,
    /// Cancellation was requested and the running work has not finished yet.
    CancelRequested,
    /// Finished successfully.
    Success,
    /// Finished, but with warnings the caller should surface.
    Warning,
    /// Finished with an error.
    Failed,
    /// Cancelled by the user before finishing.
    Cancelled,
}

impl JobStatus {
    /// Returns `true` when no more work is expected for this status.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Success | Self::Warning | Self::Failed | Self::Cancelled
        )
    }
}

/// Risk level of the operation a job performs.
///
/// `High` requires explicit confirmation; `Dangerous` is blocked or requires a
/// very explicit confirmation. Actions coming from AI are never auto-applied.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JobRisk {
    /// Read-only or trivially reversible.
    Low,
    /// Writes project-local config or runs a build.
    Medium,
    /// Edits user files or clears build directories.
    High,
    /// Destructive or outside the workspace; blocked by default.
    Dangerous,
}

/// Snapshot of one job, used by `job.list` and `event.job.created`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    /// Stable job identifier, such as `job_1`.
    pub id: String,
    /// Machine-readable kind, such as `build` or `cmake.configure`.
    pub kind: String,
    /// Human-readable title shown in the UI.
    pub title: String,
    /// Current lifecycle status.
    pub status: JobStatus,
    /// Last reported progress in `0.0..=1.0`, when the job reports it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    /// Whether the job exposes cancellation.
    pub can_cancel: bool,
    /// Risk level of the job's operation.
    pub risk: JobRisk,
}

/// Result payload for `job.list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobListResult {
    /// Jobs retained by the core, in stable id order.
    pub jobs: Vec<JobInfo>,
}

/// Parameters for `job.cancel`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JobCancelParams {
    /// Identifier of the job to cancel.
    pub job_id: String,
}

/// Result payload for `job.cancel`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCancelResult {
    /// Identifier that was requested.
    pub job_id: String,
    /// `true` when a running, cancelable job was signalled to stop.
    pub cancelled: bool,
}

/// Result of a request that starts a job and returns immediately.
///
/// The actual work runs asynchronously; progress and the final outcome arrive
/// as `event.job.*` (and any domain events like `event.build.*`) tagged with
/// this id.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobAcceptedResult {
    /// Identifier of the job that was started.
    pub job_id: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{JobCancelParams, JobInfo, JobRisk, JobStatus};

    #[test]
    fn job_info_serializes_camel_case_and_omits_absent_progress() {
        let info = JobInfo {
            id: "job_1".to_owned(),
            kind: "build".to_owned(),
            title: "cargo build".to_owned(),
            status: JobStatus::Running,
            progress: None,
            can_cancel: true,
            risk: JobRisk::Medium,
        };
        let value = serde_json::to_value(info).unwrap();

        assert_eq!(value["id"], "job_1");
        assert_eq!(value["status"], "running");
        assert_eq!(value["canCancel"], true);
        assert_eq!(value["risk"], "medium");
        assert!(value.get("progress").is_none());
    }

    #[test]
    fn cancel_requested_status_serializes_camel_case() {
        let value = serde_json::to_value(JobStatus::CancelRequested).unwrap();

        assert_eq!(value, "cancelRequested");
    }

    #[test]
    fn job_status_knows_terminal_states() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(!JobStatus::CancelRequested.is_terminal());
        assert!(JobStatus::Success.is_terminal());
        assert!(JobStatus::Warning.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
    }

    #[test]
    fn job_cancel_params_reject_unknown_fields() {
        let valid = serde_json::from_value::<JobCancelParams>(json!({ "jobId": "job_1" }));
        let invalid =
            serde_json::from_value::<JobCancelParams>(json!({ "jobId": "job_1", "force": true }));

        assert_eq!(valid.unwrap().job_id, "job_1");
        assert!(invalid.is_err());
    }
}
