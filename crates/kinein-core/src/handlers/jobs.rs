//! Handlers for `job.*` requests (`impl Core`).
//!
//! `job.list` and `job.cancel` query the [`JobManager`](crate::jobs::JobManager)
//! when it is enabled; without it (tests, `run_json_lines`) they degrade to an
//! empty list and a no-op cancel.

use kinein_protocol::{JobCancelParams, JobCancelResult, JobListResult, JsonRpcResponse};
use serde_json::{Value, json};

use crate::Core;
use crate::jobs::JobManager;
use crate::rpc::parse_params;

impl Core {
    /// Routes the `job.*` methods; `None` when the method is not a job method.
    pub(crate) fn jobs_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "job.list" => Some(self.job_list_response(request_id)),
            "job.cancel" => Some(self.job_cancel_response(request_id, params)),
            _ => None,
        }
    }

    fn job_list_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let jobs = self.jobs.as_ref().map_or_else(Vec::new, JobManager::list);
        JsonRpcResponse::success(request_id, json!(JobListResult { jobs }))
    }

    fn job_cancel_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<JobCancelParams>(
            request_id.as_ref(),
            params,
            "job.cancel requer o campo jobId",
        ) {
            Ok(value) => value,
            Err(response) => return *response,
        };
        let cancelled = self
            .jobs
            .as_ref()
            .is_some_and(|manager| manager.cancel(&parsed.job_id));
        JsonRpcResponse::success(
            request_id,
            json!(JobCancelResult {
                job_id: parsed.job_id,
                cancelled,
            }),
        )
    }
}
