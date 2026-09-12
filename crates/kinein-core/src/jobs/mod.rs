//! Job system: asynchronous, cancelable, event-emitting long operations.
//!
//! Every long-running operation the core performs should be a job: it returns an
//! id immediately, streams `event.job.*` notifications and can be cancelled,
//! keeping the UI responsive. See `DocsPublic/arquitetura/ARCHITECTURE.md` (Seção 7) for why this
//! is built before adding more long-running services, and the target model in
//! `DocsPublic/especificacoes/arquitetura-interna-core-ipc-jobs.md`.
//!
//! Organizacao interna:
//! - [`context`]: o handle que o trabalho usa para progresso/output/cancel;
//! - [`manager`]: o registro e o driver de ciclo de vida dos jobs.

mod context;
mod manager;

pub use context::{JobContext, JobOutcome};
pub use manager::JobManager;
