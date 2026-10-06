//! Catalogo ODBC padrao: tabelas/colunas, sem consulta de dialeto adivinhado.

use super::{connection::ConnectionFailure, odbc, secret::Secret};
use kinein_protocol::{DataSourceColumn, DataSourceProfile, DataSourceSchema, DataSourceTable};
use std::collections::BTreeMap;

const MAX_TABLES: u32 = 500;
const MAX_COLUMNS: u32 = 16_000;

/// `SQLTables`/`SQLColumns` retornam a estrutura com teto; nenhum SQL de escrita.
pub fn read_structure(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<Vec<DataSourceSchema>, ConnectionFailure> {
    let connection = odbc::connect(profile, secret)?;
    let quote = connection
        .identifier_quote_char()
        .map_err(|e| odbc::failure(&e, "ler o delimitador de identificador"))?;
    let mut statement = connection
        .preallocate()
        .map_err(|e| odbc::failure(&e, "preparar o catálogo"))?;
    statement
        .set_query_timeout_sec(5)
        .map_err(|e| odbc::failure(&e, "limitar o tempo do catálogo"))?;
    let cursor = statement
        .tables_cursor("", "", "", "TABLE,VIEW")
        .map_err(|e| odbc::failure(&e, "listar tabelas"))?;
    let result = super::odbc_rows::read(cursor, MAX_TABLES)?;
    if result.truncated || result.columns.len() < 5 {
        return Err(catalog_failure("catálogo incompleto ou excede 500 tabelas"));
    }
    let mut tables: BTreeMap<(String, String, String), DataSourceTable> = BTreeMap::new();
    for row in result.rows {
        let catalog = value(&row, 0);
        let schema = value(&row, 1);
        let name = value(&row, 2);
        let kind = value(&row, 3);
        if name.is_empty() || !matches!(kind.as_str(), "TABLE" | "VIEW") {
            continue;
        }
        let read_sql = select_table(quote, &catalog, &schema, &name)?;
        tables.insert(
            (catalog, schema, name.clone()),
            DataSourceTable {
                name,
                kind: kind.to_lowercase(),
                columns: Vec::new(),
                read_sql: Some(read_sql),
            },
        );
    }
    // Cursor bruto: o iterador pronto nao confere truncamento de identificador.
    // Uma passagem evita N+1 e curingas `_`/`%` nas consultas por tabela.
    let cursor = statement
        .columns_cursor("", "", "", "")
        .map_err(|e| odbc::failure(&e, "listar colunas"))?;
    let result = super::odbc_rows::read(cursor, MAX_COLUMNS)?;
    if result.truncated || result.columns.len() < 18 {
        return Err(catalog_failure(
            "catálogo incompleto ou excede 16000 colunas",
        ));
    }
    for row in result.rows {
        let key = (value(&row, 0), value(&row, 1), value(&row, 2));
        if let Some(table) = tables.get_mut(&key) {
            table.columns.push(DataSourceColumn {
                name: value(&row, 3),
                data_type: value(&row, 5),
                nullable: value(&row, 10) != "0",
            });
        }
    }
    let mut schemas: BTreeMap<(String, String), DataSourceSchema> = BTreeMap::new();
    for ((catalog, schema, _), table) in tables {
        let label = if catalog.is_empty() {
            schema.clone()
        } else if schema.is_empty() {
            catalog.clone()
        } else {
            format!("{catalog} / {schema}")
        };
        schemas
            .entry((catalog, schema))
            .or_insert_with(|| DataSourceSchema {
                name: label,
                tables: Vec::new(),
            })
            .tables
            .push(table);
    }
    Ok(schemas.into_values().collect())
}

fn value(row: &[Option<String>], index: usize) -> String {
    row.get(index)
        .and_then(Option::as_ref)
        .cloned()
        .unwrap_or_default()
}

fn catalog_failure(message: &str) -> ConnectionFailure {
    ConnectionFailure {
        message: format!("ODBC: {message}"),
        sql_state: None,
        secret_required: false,
    }
}

fn select_table(
    quote: Option<char>,
    catalog: &str,
    schema: &str,
    table: &str,
) -> Result<String, ConnectionFailure> {
    let mut names = Vec::new();
    for name in [catalog, schema, table] {
        if name.is_empty() {
            continue;
        }
        let quoted = match quote {
            Some('"' | '`' | '[') => {
                let open = quote.unwrap_or('"');
                let close = if open == '[' { ']' } else { open };
                format!(
                    "{open}{}{close}",
                    name.replace(close, &format!("{close}{close}"))
                )
            }
            None | Some(' ')
                if name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                    && !name.starts_with(|c: char| c.is_ascii_digit()) =>
            {
                name.to_owned()
            }
            _ => {
                return Err(catalog_failure(
                    "o driver não informou um delimitador de identificador suportado",
                ));
            }
        };
        names.push(quoted);
    }
    Ok(format!("SELECT * FROM {}", names.join(".")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_names_cannot_escape_the_driver_identifier_delimiter() {
        assert_eq!(
            select_table(Some('"'), "", "schema", "x\"; DROP TABLE t;--").unwrap(),
            "SELECT * FROM \"schema\".\"x\"\"; DROP TABLE t;--\""
        );
        assert_eq!(
            select_table(Some('['), "db", "dbo", "x] y").unwrap(),
            "SELECT * FROM [db].[dbo].[x]] y]"
        );
        assert!(select_table(None, "", "", "x; DROP TABLE t").is_err());
        assert!(select_table(Some('\''), "", "", "normal").is_err());
    }
}
