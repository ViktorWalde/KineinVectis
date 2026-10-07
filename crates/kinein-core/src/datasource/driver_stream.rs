//! Pure operational stream validation; no process, job or database effects.

use std::io::{self, Write};

use kinein_protocol::driver::operation::{
    Chunk, ChunkPayload, Context, Request, Terminal, TerminalResult,
};
use kinein_protocol::driver::{Limits, Operation};
use serde::{Serialize, de::DeserializeOwned};

#[path = "driver_stream_validation.rs"]
mod validation;

/// Fixed validation failures; never contain adapter text or raw JSON.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamFailure {
    /// Context identifiers/session shape are invalid.
    InvalidContext,
    /// Budget is inconsistent or greater than local ceilings.
    InvalidLimits,
    /// Another instance, session, generation or operation sent this data.
    WrongContext,
    /// Sequence is not the next contiguous value.
    WrongSequence,
    /// A terminal has already completed this operation.
    Finished,
    /// Chunk kind does not belong to the originating operation.
    UnexpectedPayload,
    /// A row has the wrong width, or headers exceed the column ceiling.
    InvalidRows,
    /// Headers changed between row chunks.
    ChangedColumns,
    /// A complete value/name/text exceeds its UTF-8 ceiling.
    CellLimit,
    /// Cumulative row count exceeds the negotiated ceiling.
    RowLimit,
    /// Cumulative catalogue/impact items exceed their ceiling.
    CatalogueLimit,
    /// One message exceeds its negotiated byte ceiling.
    MessageLimit,
    /// Cumulative accepted message bytes exceed the retention ceiling.
    RetainedLimit,
    /// Catalogue shape, presence, or executable instructions are invalid.
    InvalidCatalogue,
    /// Preview readiness, order, identity or outcome is invalid.
    InvalidPreview,
    /// Terminal kind or metadata is invalid for this operation.
    InvalidTerminal,
    /// Message is not the expected bounded JSON object.
    InvalidJson,
}

/// Cumulative accepted data, including headers, context and terminal bytes.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Statistics {
    /// Rows accepted across all chunks.
    pub rows: u32,
    /// Catalogue objects/columns/fields or impact statements accepted.
    pub catalogue_items: u32,
    /// Conservative JSON bytes retained across all messages.
    pub retained_bytes: u64,
}

/// Validates before a future bridge retains data; stores no rows or catalogue.
#[derive(Debug)]
pub struct StreamGuard {
    context: Context,
    operation: Operation,
    limits: Limits,
    next_sequence: u32,
    finished: bool,
    columns: Option<Vec<String>>,
    preview_id: Option<String>,
    catalogue_is_mongo: Option<bool>,
    expected_target: Option<(String, Option<String>)>,
    statistics: Statistics,
}

impl StreamGuard {
    /// Establishes an exact operation context and already negotiated budgets.
    ///
    /// # Errors
    /// Invalid context or limits that negotiation would reject/reduce.
    pub fn new(
        context: Context,
        operation: Operation,
        limits: Limits,
    ) -> Result<Self, StreamFailure> {
        if matches!(operation, Operation::Decide | Operation::Cancel) {
            return Err(StreamFailure::InvalidContext);
        }
        Self::new_internal(context, operation, limits)
    }

    /// Uses the exact request target and its effective row ceiling.
    ///
    /// # Errors
    /// Invalid context/limits, absent target, or excessive requested row budget.
    pub fn for_request(request: &Request, limits: Limits) -> Result<Self, StreamFailure> {
        let mut guard = Self::new_internal(request.context().clone(), request.operation(), limits)?;
        match request {
            Request::Query(params) | Request::Preview(params) => {
                if params.max_rows == 0 || params.max_rows > limits.rows {
                    return Err(StreamFailure::InvalidLimits);
                }
                guard.limits.rows = params.max_rows;
            }
            Request::Decide(params) => {
                validation::target(
                    &params.preview_operation_id,
                    Some(&params.preview_id),
                    &guard.context,
                )?;
                guard.expected_target = Some((
                    params.preview_operation_id.clone(),
                    Some(params.preview_id.clone()),
                ));
            }
            Request::Cancel(params) => {
                validation::target(&params.target_operation_id, None, &guard.context)?;
                guard.expected_target = Some((params.target_operation_id.clone(), None));
            }
            _ => {}
        }
        Ok(guard)
    }

    fn new_internal(
        context: Context,
        operation: Operation,
        limits: Limits,
    ) -> Result<Self, StreamFailure> {
        validation::context(&context, operation)?;
        if super::driver_contract::negotiate_limits(limits)
            .map_err(|_| StreamFailure::InvalidLimits)?
            != limits
        {
            return Err(StreamFailure::InvalidLimits);
        }
        Ok(Self {
            context,
            operation,
            limits,
            next_sequence: 0,
            finished: false,
            columns: None,
            preview_id: None,
            catalogue_is_mongo: None,
            expected_target: None,
            statistics: Statistics::default(),
        })
    }

    /// Current accepted budgets; failed messages never change them.
    #[must_use]
    pub const fn statistics(&self) -> Statistics {
        self.statistics
    }

    /// Whether a success or correlated failure already terminated the operation.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// Validates a typed chunk without building another full serialized buffer.
    ///
    /// # Errors
    /// Context, ordering, shape or cumulative budget violation.
    pub fn accept_chunk(&mut self, chunk: &Chunk) -> Result<(), StreamFailure> {
        let bytes = measure(chunk, self.limits.message_bytes)?;
        self.accept_chunk_sized(chunk, bytes)
    }

    /// Bounds bytes before decoding, then returns data only after validation.
    ///
    /// # Errors
    /// Invalid JSON or the same failures as [`Self::accept_chunk`].
    pub fn accept_chunk_bytes(&mut self, bytes: &[u8]) -> Result<Chunk, StreamFailure> {
        let chunk = decode(bytes, self.limits.message_bytes)?;
        self.accept_chunk_sized(&chunk, bytes.len() as u64)?;
        Ok(chunk)
    }

    /// Validates a typed terminal; acceptance is the only terminal transition.
    ///
    /// # Errors
    /// Context, sequence, terminal semantics or cumulative budget violation.
    pub fn accept_terminal(&mut self, terminal: &Terminal) -> Result<(), StreamFailure> {
        let bytes = measure(terminal, self.limits.message_bytes)?;
        self.accept_terminal_sized(terminal, bytes)
    }

    /// Bounds the success payload before decoding and applying terminal checks.
    ///
    /// # Errors
    /// Invalid JSON or the same failures as [`Self::accept_terminal`].
    pub fn accept_terminal_bytes(&mut self, bytes: &[u8]) -> Result<Terminal, StreamFailure> {
        let terminal = decode(bytes, self.limits.message_bytes)?;
        self.accept_terminal_sized(&terminal, bytes.len() as u64)?;
        Ok(terminal)
    }

    /// Completes a numeric error correlated by the runtime with its pending id.
    ///
    /// # Errors
    /// Context changed or the operation already had a terminal response.
    pub fn finish_failure(&mut self, context: &Context) -> Result<(), StreamFailure> {
        self.check_context(context)?;
        self.finished = true;
        Ok(())
    }

    fn check_context(&self, context: &Context) -> Result<(), StreamFailure> {
        if self.finished {
            return Err(StreamFailure::Finished);
        }
        if context != &self.context {
            return Err(StreamFailure::WrongContext);
        }
        Ok(())
    }

    fn check_sequence(&self, context: &Context, sequence: u32) -> Result<(), StreamFailure> {
        self.check_context(context)?;
        if sequence != self.next_sequence || sequence == u32::MAX {
            return Err(StreamFailure::WrongSequence);
        }
        Ok(())
    }

    fn next_statistics(
        &self,
        rows: u32,
        items: u32,
        bytes: u64,
    ) -> Result<Statistics, StreamFailure> {
        let rows = self
            .statistics
            .rows
            .checked_add(rows)
            .ok_or(StreamFailure::RowLimit)?;
        let catalogue_items = self
            .statistics
            .catalogue_items
            .checked_add(items)
            .ok_or(StreamFailure::CatalogueLimit)?;
        let retained_bytes = self
            .statistics
            .retained_bytes
            .checked_add(bytes)
            .ok_or(StreamFailure::RetainedLimit)?;
        if rows > self.limits.rows {
            return Err(StreamFailure::RowLimit);
        }
        if catalogue_items > self.limits.catalogue_items {
            return Err(StreamFailure::CatalogueLimit);
        }
        if retained_bytes > u64::from(self.limits.retained_bytes) {
            return Err(StreamFailure::RetainedLimit);
        }
        Ok(Statistics {
            rows,
            catalogue_items,
            retained_bytes,
        })
    }

    fn accept_chunk_sized(&mut self, chunk: &Chunk, bytes: u64) -> Result<(), StreamFailure> {
        self.check_sequence(&chunk.context, chunk.sequence)?;
        let (rows, items) = validation::chunk(&chunk.payload, self.operation, self.limits)?;
        match &chunk.payload {
            ChunkPayload::Rows { columns, .. } => {
                if self.preview_id.is_some() {
                    return Err(StreamFailure::InvalidPreview);
                }
                if self
                    .columns
                    .as_ref()
                    .is_some_and(|previous| previous != columns)
                {
                    return Err(StreamFailure::ChangedColumns);
                }
            }
            ChunkPayload::PreviewReady { .. } if self.preview_id.is_some() => {
                return Err(StreamFailure::InvalidPreview);
            }
            ChunkPayload::PreviewReady {
                affected,
                truncated,
                ..
            } if u64::from(self.statistics.rows) > *affected
                || (!truncated && u64::from(self.statistics.rows) != *affected) =>
            {
                return Err(StreamFailure::InvalidPreview);
            }
            ChunkPayload::Catalogue {
                schemas,
                collections,
            } => {
                let mongo = !collections.is_empty();
                if (!schemas.is_empty() || mongo)
                    && self
                        .catalogue_is_mongo
                        .is_some_and(|previous| previous != mongo)
                {
                    return Err(StreamFailure::InvalidCatalogue);
                }
            }
            ChunkPayload::PreviewReady { .. } => {}
        }
        let statistics = self.next_statistics(rows, items, bytes)?;
        // All checks precede clones and state/budget changes.
        match &chunk.payload {
            ChunkPayload::Rows { columns, .. } if self.columns.is_none() => {
                self.columns = Some(columns.clone());
            }
            ChunkPayload::PreviewReady { preview_id, .. } => {
                self.preview_id = Some(preview_id.clone());
            }
            ChunkPayload::Catalogue {
                schemas,
                collections,
            } if !schemas.is_empty() || !collections.is_empty() => {
                self.catalogue_is_mongo = Some(!collections.is_empty());
            }
            _ => {}
        }
        self.statistics = statistics;
        self.next_sequence += 1;
        Ok(())
    }

    fn accept_terminal_sized(
        &mut self,
        terminal: &Terminal,
        bytes: u64,
    ) -> Result<(), StreamFailure> {
        self.check_sequence(&terminal.context, terminal.sequence)?;
        let items = validation::terminal(&terminal.result, self.operation, self.limits)?;
        match (&terminal.result, &self.expected_target) {
            (
                TerminalResult::Decide {
                    preview_operation_id,
                    preview_id,
                    ..
                },
                Some((target, Some(preview))),
            ) if preview_operation_id != target || preview_id != preview => {
                return Err(StreamFailure::InvalidTerminal);
            }
            (
                TerminalResult::Cancel {
                    target_operation_id,
                    ..
                },
                Some((target, None)),
            ) if target_operation_id != target => {
                return Err(StreamFailure::InvalidTerminal);
            }
            _ => {}
        }
        if let TerminalResult::Preview {
            preview_id,
            outcome,
            ..
        } = &terminal.result
        {
            validation::preview_completion(
                self.preview_id.as_deref(),
                preview_id.as_deref(),
                *outcome,
            )?;
        }
        self.statistics = self.next_statistics(0, items, bytes)?;
        self.finished = true;
        Ok(())
    }
}

fn decode<T: DeserializeOwned>(bytes: &[u8], max: u32) -> Result<T, StreamFailure> {
    if bytes.len() > max as usize {
        return Err(StreamFailure::MessageLimit);
    }
    if bytes.is_empty() || bytes.iter().any(|byte| matches!(byte, b'\n' | b'\r')) {
        return Err(StreamFailure::InvalidJson);
    }
    serde_json::from_slice(bytes).map_err(|_| StreamFailure::InvalidJson)
}

struct Counter {
    bytes: u64,
    max: u64,
    exceeded: bool,
}
impl Write for Counter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let Some(next) = self.bytes.checked_add(buffer.len() as u64) else {
            self.exceeded = true;
            return Err(io::Error::other("message limit"));
        };
        if next > self.max {
            self.exceeded = true;
            return Err(io::Error::other("message limit"));
        }
        self.bytes = next;
        Ok(buffer.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn measure(value: &impl Serialize, max: u32) -> Result<u64, StreamFailure> {
    let mut counter = Counter {
        bytes: 0,
        max: u64::from(max),
        exceeded: false,
    };
    if serde_json::to_writer(&mut counter, value).is_err() {
        return Err(if counter.exceeded {
            StreamFailure::MessageLimit
        } else {
            StreamFailure::InvalidJson
        });
    }
    Ok(counter.bytes)
}

#[cfg(test)]
mod tests;
