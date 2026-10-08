//! Os orcamentos negociados (40 §4).
//!
//! Cada chunk cabe numa mensagem; o total cabe na retencao; o catalogo cabe
//! no teto de itens. O que nao cabe corta o
//! CONJUNTO (linhas ou tabelas) e diz `truncated`; uma celula ou um numero de
//! colunas acima do teto recusa a operacao, porque cortar um valor e publica-lo
//! como inteiro seria mentir.

use kinein_protocol::DataSourceSchema;
use kinein_protocol::JsonRpcId;
use kinein_protocol::driver::operation::{Chunk, ChunkPayload, Context, Terminal, TerminalResult};
use kinein_protocol::driver::{FailureReason, Limits, OperationOutcome};
use serde_json::{Value, json};

use crate::engine::Refusal;

/// Folga para o envelope de cada mensagem.
const ENVELOPE: usize = 1024;
/// O que fica guardado na retencao para o terminal.
const TERMINAL_RESERVE: usize = 16 * 1024;

/// As mensagens de UMA operacao, na ordem, com a sequencia e o que ja' gastou.
#[derive(Debug)]
pub struct Stream {
    context: Context,
    limits: Limits,
    messages: Vec<Value>,
    retained: usize,
}

impl Stream {
    /// Um stream vazio para a operacao de `context`, nos orcamentos dados.
    #[must_use]
    pub const fn new(context: Context, limits: Limits) -> Self {
        Self {
            context,
            limits,
            messages: Vec::new(),
            retained: 0,
        }
    }

    const fn message_room(&self) -> usize {
        (self.limits.message_bytes as usize).saturating_sub(ENVELOPE)
    }

    const fn retained_room(&self) -> usize {
        (self.limits.retained_bytes as usize)
            .saturating_sub(TERMINAL_RESERVE)
            .saturating_sub(self.retained)
    }

    fn push(&mut self, payload: ChunkPayload) {
        let chunk = Chunk {
            context: self.context.clone(),
            sequence: u32::try_from(self.messages.len()).unwrap_or(u32::MAX),
            payload,
        };
        let message = json!({ "jsonrpc": "2.0", "method": "driver.chunk", "params": chunk });
        self.retained += message.to_string().len();
        self.messages.push(message);
    }

    /// As linhas em chunks; `Ok(true)` quando o orcamento cortou o conjunto.
    ///
    /// # Errors
    /// Colunas ou celula acima do teto negociado.
    pub fn rows(
        &mut self,
        columns: &[String],
        rows: Vec<Vec<Option<String>>>,
    ) -> Result<bool, Refusal> {
        let over = || (FailureReason::LimitExceeded, OperationOutcome::Failed, None);
        if columns.len() > self.limits.columns as usize {
            return Err(over());
        }
        let cell = self.limits.cell_bytes as usize;
        if rows
            .iter()
            .flatten()
            .flatten()
            .any(|value| value.len() > cell)
        {
            return Err(over());
        }
        if columns.is_empty() {
            return Ok(false);
        }
        let header = json!({ "kind": "rows", "columns": columns, "rows": [] })
            .to_string()
            .len();
        let mut batch: Vec<Vec<Option<String>>> = Vec::new();
        let mut batch_bytes = header;
        let mut cut = false;
        for row in rows {
            let size = serde_json::to_string(&row).map_or(usize::MAX, |text| text.len() + 1);
            if batch_bytes + size > self.retained_room() {
                cut = true;
                break;
            }
            if !batch.is_empty() && batch_bytes + size > self.message_room() {
                self.push(ChunkPayload::Rows {
                    columns: columns.to_vec(),
                    rows: std::mem::take(&mut batch),
                });
                batch_bytes = header;
                if batch_bytes + size > self.retained_room() {
                    cut = true;
                    break;
                }
            }
            batch_bytes += size;
            batch.push(row);
        }
        if !batch.is_empty() || self.messages.is_empty() {
            self.push(ChunkPayload::Rows {
                columns: columns.to_vec(),
                rows: batch,
            });
        }
        Ok(cut)
    }

    /// O catalogo em chunks por esquema, no teto de itens (tabela e coluna
    /// contam um cada); `true` quando cortou tabelas.
    pub fn catalogue(&mut self, schemas: Vec<DataSourceSchema>) -> bool {
        let mut items = 0usize;
        let mut cut = false;
        for mut schema in schemas {
            let tables = std::mem::take(&mut schema.tables);
            let mut current = DataSourceSchema {
                tables: Vec::new(),
                ..schema.clone()
            };
            let mut current_bytes =
                json!({ "kind": "catalogue", "schemas": [&current], "collections": [] })
                    .to_string()
                    .len();
            for table in tables {
                let cost = 1 + table.columns.len();
                let size = serde_json::to_string(&table).map_or(usize::MAX, |text| text.len() + 1);
                if items + cost > self.limits.catalogue_items as usize
                    || current_bytes + size > self.retained_room()
                {
                    cut = true;
                    break;
                }
                if !current.tables.is_empty() && current_bytes + size > self.message_room() {
                    let full = std::mem::replace(
                        &mut current,
                        DataSourceSchema {
                            tables: Vec::new(),
                            ..schema.clone()
                        },
                    );
                    self.push(ChunkPayload::Catalogue {
                        schemas: vec![full],
                        collections: Vec::new(),
                    });
                    current_bytes = 128 + schema.name.len();
                }
                items += cost;
                current_bytes += size;
                current.tables.push(table);
            }
            self.push(ChunkPayload::Catalogue {
                schemas: vec![current],
                collections: Vec::new(),
            });
            if cut {
                break;
            }
        }
        cut
    }

    /// As mensagens da operacao e o terminal, com a sequencia seguinte.
    #[must_use]
    pub fn finish(self, id: &JsonRpcId, result: TerminalResult) -> Vec<Value> {
        let terminal = Terminal {
            sequence: u32::try_from(self.messages.len()).unwrap_or(u32::MAX),
            context: self.context,
            result,
        };
        let mut messages = self.messages;
        messages.push(json!({ "jsonrpc": "2.0", "id": id, "result": terminal }));
        messages
    }
}
