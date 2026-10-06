//! A job owns the connection, credential, streaming sample and live transaction.

use std::time::{Duration, Instant};

use futures_util::TryStreamExt;
use kinein_protocol::{
    DataSourcePreviewDecision as Decision, DataSourcePreviewOutcome as Outcome,
    DataSourcePreviewedEvent, DataSourceProfile, DataSourceQueriedEvent, DataSourceQueryAccess,
    DataSourceQueryParams,
};
use tokio::time::{sleep, timeout};

use super::connection::{ConnectionFailure, connect_async, failure_from};
use super::preview::{DECISION_SECONDS, Lease};
use super::preview_rows::Collector;
use super::query::{QueryResult, clamp_rows};
use super::secret::Secret;
use crate::jobs::{JobContext, JobOutcome};

const QUERY_TIMEOUT: Duration = Duration::from_secs(15);
const FINALIZE_TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug)]
struct Failure {
    message: String,
    secret_required: bool,
    outcome: Outcome,
}

impl From<ConnectionFailure> for Failure {
    fn from(failure: ConnectionFailure) -> Self {
        Self {
            message: failure.message,
            secret_required: failure.secret_required,
            outcome: Outcome::Failed,
        }
    }
}

impl Failure {
    fn public(message: &str, outcome: Outcome) -> Self {
        Self {
            message: message.to_owned(),
            secret_required: false,
            outcome,
        }
    }
}

struct Driver(tokio::task::JoinHandle<Result<(), postgres::Error>>);
impl Drop for Driver {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Runs a cancellable preview inside the existing job infrastructure.
#[must_use]
pub fn run(
    ctx: &JobContext,
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    request: &DataSourceQueryParams,
    mut lease: Lease,
    executed_sql: &str,
) -> JobOutcome {
    let started = Instant::now();
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Failure::public("Não foi possível iniciar a prévia.", Outcome::Failed))
        .and_then(|runtime| {
            runtime.block_on(run_async(
                ctx,
                profile,
                secret,
                request,
                &mut lease,
                executed_sql,
            ))
        });
    drop(lease);
    let mut event = DataSourceQueriedEvent {
        job_id: ctx.id().to_owned(),
        name: profile.name.clone(),
        client_context: request.client_context.clone(),
        access: DataSourceQueryAccess::Write,
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        ..DataSourceQueriedEvent::default()
    };
    let status = match result {
        Ok((outcome, sample)) => {
            event.success = true;
            event.preview_outcome = Some(outcome);
            event.affected = sample.affected;
            event.row_count = sample.rows.len();
            event.columns = sample.columns;
            event.rows = sample.rows;
            event.truncated = sample.truncated;
            event.message = Some(
                match outcome {
                    Outcome::Committed => "Alterações confirmadas pelo PostgreSQL.",
                    Outcome::RolledBack => "Alterações desfeitas pelo PostgreSQL.",
                    Outcome::Expired => {
                        "O prazo expirou; as alterações foram desfeitas pelo PostgreSQL."
                    }
                    _ => "A prévia foi cancelada; as alterações foram desfeitas pelo PostgreSQL.",
                }
                .to_owned(),
            );
            if matches!(outcome, Outcome::Committed | Outcome::RolledBack) {
                JobOutcome::Success
            } else {
                JobOutcome::Warning
            }
        }
        Err(failure) => {
            event.message = Some(failure.message);
            event.secret_required = failure.secret_required;
            event.preview_outcome = Some(failure.outcome);
            JobOutcome::Failed
        }
    };
    if let Some(message) = &event.message {
        ctx.emit_output(message);
    }
    ctx.emit_event("event.datasource.queried", serde_json::json!(event));
    status
}

async fn interrupted(ctx: &JobContext, lease: &mut Lease) {
    loop {
        if ctx.is_cancelled() || lease.discarded() {
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }
}

async fn run_async(
    ctx: &JobContext,
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    request: &DataSourceQueryParams,
    lease: &mut Lease,
    executed_sql: &str,
) -> Result<(Outcome, QueryResult), Failure> {
    let (mut client, driver) = tokio::select! {
        result = timeout(Duration::from_secs(6), connect_async(profile, secret)) => result
            .map_err(|_| Failure::public("A conexão da prévia excedeu o prazo.", Outcome::Failed))??,
        () = interrupted(ctx, lease) => return Err(Failure::public("A prévia foi cancelada antes da conexão.", Outcome::Cancelled)),
    };
    let _driver = Driver(driver);
    let started = Instant::now();
    let execute = async {
        let tx = client
            .transaction()
            .await
            .map_err(|error| Failure::from(failure_from(&error)))?;
        tx.batch_execute("SET LOCAL statement_timeout = '10s'; SET LOCAL lock_timeout = '1s'; SET LOCAL idle_in_transaction_session_timeout = '65s'")
            .await.map_err(|error| Failure::from(failure_from(&error)))?;
        let sample = collect(tx.client(), executed_sql, clamp_rows(request.max_rows)).await?;
        Ok::<_, Failure>((tx, sample))
    };
    let (tx, mut sample) = tokio::select! {
        result = timeout(QUERY_TIMEOUT, execute) => result
            .map_err(|_| Failure::public("A prévia excedeu o prazo; a conexão será encerrada para desfazer a transação.", Outcome::Failed))??,
        () = interrupted(ctx, lease) => return Err(Failure::public("A prévia foi cancelada; a conexão será encerrada para desfazer a transação.", Outcome::Cancelled)),
    };
    sample.elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    if let Err(message) = lease.ready(ctx.id()) {
        finalize(tx, Outcome::Cancelled).await?;
        return Err(Failure::public(message, Outcome::Cancelled));
    }
    let event = DataSourcePreviewedEvent {
        job_id: ctx.id().to_owned(),
        name: profile.name.clone(),
        client_context: request.client_context.clone().unwrap_or_default(),
        preview_id: lease.id().to_owned(),
        expires_in_seconds: DECISION_SECONDS,
        sql: request.sql.clone(),
        executed_sql: executed_sql.to_owned(),
        columns: sample.columns.clone(),
        rows: sample.rows.clone(),
        affected: sample.affected.unwrap_or(0),
        truncated: sample.truncated,
        elapsed_ms: sample.elapsed_ms,
    };
    ctx.emit_event("event.datasource.previewed", serde_json::json!(event));
    drop(event);
    let outcome = await_decision(ctx, lease, tx.client()).await;
    finalize(tx, outcome).await?;
    Ok((outcome, sample))
}

async fn collect(
    client: &tokio_postgres::Client,
    sql: &str,
    max_rows: u32,
) -> Result<QueryResult, Failure> {
    let stream = client
        .simple_query_raw(sql)
        .await
        .map_err(|error| Failure::from(failure_from(&error)))?;
    futures_util::pin_mut!(stream);
    let mut collector = Collector::new(max_rows);
    while let Some(message) = stream
        .try_next()
        .await
        .map_err(|error| Failure::from(failure_from(&error)))?
    {
        collector
            .accept(message)
            .map_err(|message| Failure::public(message, Outcome::Failed))?;
    }
    collector
        .finish()
        .map_err(|message| Failure::public(message, Outcome::Failed))
}

async fn await_decision(
    ctx: &JobContext,
    lease: &mut Lease,
    client: &tokio_postgres::Client,
) -> Outcome {
    let cancelled = async {
        loop {
            if ctx.is_cancelled() || client.is_closed() {
                return;
            }
            sleep(Duration::from_millis(50)).await;
        }
    };
    tokio::select! {
        // A decision already accepted takes precedence over subsequent cancellation.
        biased;
        decision = lease.decision() => match decision { Ok(Decision::Commit) => Outcome::Committed,
            Ok(Decision::Rollback) => Outcome::RolledBack, Err(_) => Outcome::Cancelled },
        () = cancelled => Outcome::Cancelled,
        () = sleep(Duration::from_secs(u64::from(DECISION_SECONDS))) => Outcome::Expired,
    }
}

async fn finalize(tx: tokio_postgres::Transaction<'_>, outcome: Outcome) -> Result<(), Failure> {
    if outcome == Outcome::Committed {
        match timeout(FINALIZE_TIMEOUT, tx.commit()).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) if commit_refused(&error) => Err(Failure::public(
                "O PostgreSQL recusou COMMIT; a prévia falhou.",
                Outcome::Failed,
            )),
            _ => Err(Failure::public(
                "A confirmação perdeu a resposta do servidor; o resultado é desconhecido. Confira os dados em outra conexão antes de repetir.",
                Outcome::Unknown,
            )),
        }
    } else {
        match timeout(FINALIZE_TIMEOUT, tx.rollback()).await {
            Ok(Ok(())) => Ok(()),
            _ => Err(Failure::public(
                "Não foi possível obter a confirmação de ROLLBACK; a conexão foi encerrada. Confira o destino antes de repetir.",
                Outcome::Failed,
            )),
        }
    }
}

fn commit_refused(error: &postgres::Error) -> bool {
    error.as_db_error().is_some_and(|error| {
        error.parsed_severity() == Some(tokio_postgres::error::Severity::Error)
            && error.code().code() != "40003"
            && !error.code().code().starts_with("08")
    })
}
