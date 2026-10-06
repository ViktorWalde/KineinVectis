//! Bounded RETURNING sample, while draining the command to its server count.

use tokio_postgres::SimpleQueryMessage;

use super::query::QueryResult;

const MAX_COLUMNS: usize = 128;
const MAX_CELL_BYTES: usize = 16 * 1024;
const MAX_RETAINED_BYTES: usize = 8 * 1024 * 1024;

/// Retains a sample; never stops draining merely because the row cap was reached.
#[derive(Debug)]
pub(super) struct Collector {
    result: QueryResult,
    max_rows: usize,
    bytes: usize,
}

impl Collector {
    pub(super) fn new(max_rows: u32) -> Self {
        Self {
            result: QueryResult::default(),
            max_rows: max_rows as usize,
            bytes: 0,
        }
    }

    pub(super) fn accept(&mut self, message: SimpleQueryMessage) -> Result<(), &'static str> {
        match message {
            SimpleQueryMessage::RowDescription(columns) => {
                if columns.is_empty()
                    || columns.len() > MAX_COLUMNS
                    || !self.result.columns.is_empty()
                {
                    return Err(
                        "A prévia excedeu o limite de colunas ou devolveu mais de um resultado.",
                    );
                }
                for column in columns.iter() {
                    self.retain(column.name().len())?;
                }
                self.result.columns = columns
                    .iter()
                    .map(|column| column.name().to_owned())
                    .collect();
            }
            SimpleQueryMessage::Row(row) => {
                if row.len() != self.result.columns.len() || row.len() > MAX_COLUMNS {
                    return Err("A prévia devolveu uma linha com estrutura inesperada.");
                }
                for index in 0..row.len() {
                    if row
                        .get(index)
                        .is_some_and(|cell| cell.len() > MAX_CELL_BYTES)
                    {
                        return Err(
                            "Uma célula da prévia excedeu 16 KiB; a transação será desfeita.",
                        );
                    }
                }
                if self.result.rows.len() == self.max_rows {
                    self.result.truncated = true;
                } else {
                    for index in 0..row.len() {
                        self.retain(row.get(index).map_or(0, str::len))?;
                    }
                    self.result.rows.push(
                        (0..row.len())
                            .map(|index| row.get(index).map(str::to_owned))
                            .collect(),
                    );
                }
            }
            SimpleQueryMessage::CommandComplete(count) => {
                if self.result.affected.is_some() {
                    return Err("A prévia devolveu mais de um comando; a transação será desfeita.");
                }
                self.result.affected = Some(count);
            }
            _ => return Err("O servidor devolveu uma mensagem desconhecida na prévia."),
        }
        Ok(())
    }

    const fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        if bytes > MAX_CELL_BYTES || self.bytes.saturating_add(bytes) > MAX_RETAINED_BYTES {
            return Err(
                "A amostra da prévia excedeu o limite de memória; a transação será desfeita.",
            );
        }
        self.bytes += bytes;
        Ok(())
    }

    pub(super) fn finish(self) -> Result<QueryResult, &'static str> {
        if self.result.affected.is_none() || self.result.columns.is_empty() {
            return Err("O servidor não informou o resultado completo da prévia.");
        }
        Ok(self.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_limit_and_server_completion_are_required() {
        let mut collector = Collector::new(3);
        assert!(collector.retain(MAX_CELL_BYTES + 1).is_err());
        for _ in 0..MAX_RETAINED_BYTES / MAX_CELL_BYTES {
            collector.retain(MAX_CELL_BYTES).unwrap();
        }
        assert!(collector.retain(1).is_err());
        assert!(collector.finish().is_err());
        let mut collector = Collector::new(3);
        collector
            .accept(SimpleQueryMessage::CommandComplete(5))
            .unwrap();
        assert!(
            collector
                .accept(SimpleQueryMessage::CommandComplete(5))
                .is_err()
        );
    }
}
