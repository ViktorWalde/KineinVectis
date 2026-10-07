//! Content checks for the operational stream; no accumulation or effects.

use kinein_protocol::driver::operation::{ChunkPayload, Context, TerminalResult};
use kinein_protocol::driver::{Limits, Operation};
use kinein_protocol::{DataSourcePreviewOutcome, DataSourceSchema, MongoCollection};

use super::StreamFailure;

pub(super) fn context(context: &Context, operation: Operation) -> Result<(), StreamFailure> {
    if !identity(&context.instance_id)
        || !identity(&context.operation_id)
        || context
            .session_id
            .as_deref()
            .is_some_and(|id| !identity(id))
        || (context.session_id.is_none()
            != matches!(operation, Operation::Open | Operation::Shutdown))
    {
        return Err(StreamFailure::InvalidContext);
    }
    Ok(())
}

fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.chars().all(|character| !character.is_control())
}

pub(super) fn target(
    operation_id: &str,
    preview_id: Option<&str>,
    context: &Context,
) -> Result<(), StreamFailure> {
    if !identity(operation_id)
        || operation_id == context.operation_id
        || preview_id.is_some_and(|id| !identity(id))
    {
        return Err(StreamFailure::InvalidContext);
    }
    Ok(())
}

const fn text(value: &str, limits: Limits) -> Result<(), StreamFailure> {
    if value.len() > limits.cell_bytes as usize {
        return Err(StreamFailure::CellLimit);
    }
    Ok(())
}

pub(super) fn chunk(
    payload: &ChunkPayload,
    operation: Operation,
    limits: Limits,
) -> Result<(u32, u32), StreamFailure> {
    match payload {
        ChunkPayload::Catalogue {
            schemas,
            collections,
        } if operation == Operation::Introspect => {
            if !schemas.is_empty() && !collections.is_empty() {
                return Err(StreamFailure::InvalidCatalogue);
            }
            Ok((0, catalogue(schemas, collections, limits)?))
        }
        ChunkPayload::Rows { columns, rows }
            if matches!(operation, Operation::Query | Operation::Preview) =>
        {
            if columns.len() > usize::from(limits.columns)
                || (columns.is_empty() && !rows.is_empty())
            {
                return Err(StreamFailure::InvalidRows);
            }
            for column in columns {
                text(column, limits)?;
            }
            for row in rows {
                if row.len() != columns.len() {
                    return Err(StreamFailure::InvalidRows);
                }
                for cell in row.iter().flatten() {
                    text(cell, limits)?;
                }
            }
            Ok((
                u32::try_from(rows.len()).map_err(|_| StreamFailure::RowLimit)?,
                0,
            ))
        }
        ChunkPayload::PreviewReady {
            preview_id,
            executed_sql,
            expires_in_seconds,
            ..
        } if operation == Operation::Preview => {
            if !identity(preview_id) || !(1..=60).contains(expires_in_seconds) {
                return Err(StreamFailure::InvalidPreview);
            }
            if executed_sql.trim().is_empty() {
                return Err(StreamFailure::InvalidPreview);
            }
            text(executed_sql, limits)?;
            Ok((0, 0))
        }
        _ => Err(StreamFailure::UnexpectedPayload),
    }
}

fn catalogue(
    schemas: &[DataSourceSchema],
    collections: &[MongoCollection],
    limits: Limits,
) -> Result<u32, StreamFailure> {
    let mut items = 0_u32;
    for schema in schemas {
        add_item(&mut items, limits)?;
        text(&schema.name, limits)?;
        for table in &schema.tables {
            add_item(&mut items, limits)?;
            text(&table.name, limits)?;
            text(&table.kind, limits)?;
            if !matches!(table.kind.as_str(), "table" | "view")
                || table.statements.is_some()
                || table.read_sql.is_some()
                || table.columns.len() > usize::from(limits.columns)
            {
                return Err(StreamFailure::InvalidCatalogue);
            }
            for column in &table.columns {
                add_item(&mut items, limits)?;
                text(&column.name, limits)?;
                text(&column.data_type, limits)?;
            }
        }
    }
    for collection in collections {
        add_item(&mut items, limits)?;
        for value in [
            &collection.name,
            &collection.kind,
            &collection.time_field,
            &collection.meta_field,
            &collection.granularity,
        ] {
            text(value, limits)?;
        }
        // An adapter cannot inject templates or free diagnostic strings into the UI.
        if !matches!(
            collection.kind.as_str(),
            "collection" | "view" | "timeseries"
        ) || collection.statements.is_some()
            || !collection.truncated.is_empty()
        {
            return Err(StreamFailure::InvalidCatalogue);
        }
        for field in &collection.fields {
            add_item(&mut items, limits)?;
            text(&field.path, limits)?;
            for kind in &field.types {
                text(kind, limits)?;
            }
            if field
                .presence
                .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
                || (collection.declared && field.presence.is_some())
                || (!collection.declared && field.required)
            {
                return Err(StreamFailure::InvalidCatalogue);
            }
        }
    }
    Ok(items)
}

fn add_item(items: &mut u32, limits: Limits) -> Result<(), StreamFailure> {
    *items = items.checked_add(1).ok_or(StreamFailure::CatalogueLimit)?;
    if *items > limits.catalogue_items {
        return Err(StreamFailure::CatalogueLimit);
    }
    Ok(())
}

pub(super) fn terminal(
    result: &TerminalResult,
    operation: Operation,
    limits: Limits,
) -> Result<u32, StreamFailure> {
    match (result, operation) {
        (
            TerminalResult::Open {
                session_id,
                server_version,
                operations,
            },
            Operation::Open,
        ) => {
            if !identity(session_id)
                || operations.is_empty()
                || operations
                    .iter()
                    .enumerate()
                    .any(|(index, item)| operations[..index].contains(item))
            {
                return Err(StreamFailure::InvalidTerminal);
            }
            if let Some(version) = server_version {
                text(version, limits)?;
            }
        }
        (TerminalResult::Test { server_version }, Operation::Test) => {
            if let Some(version) = server_version {
                text(version, limits)?;
            }
        }
        (TerminalResult::Introspect { .. }, Operation::Introspect)
        | (TerminalResult::Query { .. }, Operation::Query)
        | (TerminalResult::Preview { .. }, Operation::Preview)
        | (TerminalResult::Close {}, Operation::Close)
        | (TerminalResult::Shutdown {}, Operation::Shutdown) => {}
        (
            TerminalResult::Impact {
                statements,
                severity,
            },
            Operation::Impact,
        ) => {
            let mut items = 0;
            for statement in statements {
                add_item(&mut items, limits)?;
                for value in [
                    &statement.text,
                    &statement.kind,
                    &statement.column,
                    &statement.filter,
                ] {
                    text(value, limits)?;
                }
                for target in &statement.targets {
                    text(target, limits)?;
                }
                if statement.note.is_some() || statement.severity > *severity {
                    return Err(StreamFailure::InvalidTerminal);
                }
            }
            return Ok(items);
        }
        (
            TerminalResult::Decide {
                preview_operation_id,
                preview_id,
                ..
            },
            Operation::Decide,
        ) => {
            if !identity(preview_operation_id) || !identity(preview_id) {
                return Err(StreamFailure::InvalidTerminal);
            }
        }
        (
            TerminalResult::Cancel {
                target_operation_id,
                ..
            },
            Operation::Cancel,
        ) => {
            if !identity(target_operation_id) {
                return Err(StreamFailure::InvalidTerminal);
            }
        }
        _ => return Err(StreamFailure::InvalidTerminal),
    }
    Ok(0)
}

pub(super) fn preview_completion(
    ready: Option<&str>,
    terminal: Option<&str>,
    outcome: DataSourcePreviewOutcome,
) -> Result<(), StreamFailure> {
    if ready != terminal
        || (ready.is_none()
            && matches!(
                outcome,
                DataSourcePreviewOutcome::Committed
                    | DataSourcePreviewOutcome::RolledBack
                    | DataSourcePreviewOutcome::Expired
            ))
    {
        return Err(StreamFailure::InvalidPreview);
    }
    Ok(())
}
