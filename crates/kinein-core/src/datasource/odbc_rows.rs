//! Buffer limitado para console e catalogo; truncamento de celula e' erro.

use super::{connection::ConnectionFailure, odbc, query::QueryResult};
use odbc_api::{Cursor, buffers::TextRowSet};
use std::mem::size_of;

const MAX_COLUMNS: usize = 128;
const MAX_CELL: usize = 16 * 1024;
const MAX_RESULT: usize = 8 * 1024 * 1024;

/// Retem no maximo o teto solicitado e sinaliza se havia outra linha.
pub fn read(mut cursor: impl Cursor, max_rows: u32) -> Result<QueryResult, ConnectionFailure> {
    let count = usize::try_from(
        cursor
            .num_result_cols()
            .map_err(|e| odbc::failure(&e, "ler as colunas"))?,
    )
    .unwrap_or(usize::MAX);
    if count == 0 || count > MAX_COLUMNS {
        return Err(failure("colunas (teto: 128)"));
    }
    let columns = cursor
        .column_names()
        .map_err(|e| odbc::failure(&e, "ler os nomes das colunas"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| odbc::failure(&e, "ler os nomes das colunas"))?;
    let mut buffer = TextRowSet::from_max_str_lens(1, std::iter::repeat_n(MAX_CELL, count))
        .map_err(|e| odbc::failure(&e, "reservar o buffer de leitura"))?;
    let mut cursor = cursor
        .bind_buffer(&mut buffer)
        .map_err(|e| odbc::failure(&e, "preparar a leitura"))?;
    let mut rows = Vec::new();
    let mut bytes = columns
        .iter()
        .map(|s| s.len() + size_of::<String>())
        .sum::<usize>();
    let mut truncated = false;
    while let Some(batch) = cursor
        .fetch_with_truncation_check(true)
        .map_err(|e| odbc::failure(&e, "ler uma linha (teto por celula: 16 KiB)"))?
    {
        if rows.len() >= max_rows as usize {
            truncated = true;
            break;
        }
        let row = (0..count)
            .map(|col| {
                batch
                    .at_as_str(col, 0)
                    .map(|s| s.map(str::to_owned))
                    .map_err(|_| failure("texto UTF-8 invalido"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Reserva de Vec inclui capacidade que pode dobrar; conta tambem
        // Option<String> vazia. Milhares de NULL nao driblam o teto de memoria.
        bytes += row.iter().flatten().map(String::len).sum::<usize>()
            + count * size_of::<Option<String>>()
            + 2 * size_of::<Vec<Option<String>>>();
        if bytes > MAX_RESULT {
            truncated = true;
            break;
        }
        rows.push(row);
    }
    Ok(QueryResult {
        columns,
        rows,
        truncated,
        ..QueryResult::default()
    })
}

fn failure(detail: &str) -> ConnectionFailure {
    ConnectionFailure {
        message: format!("ODBC: resultado recusado — {detail}"),
        sql_state: None,
        secret_required: false,
    }
}
