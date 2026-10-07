//! Object instructions from catalogue metadata. No connection or execution.
//! SQL identifiers are quoted once here; ODBC keeps the driver's SQL intact.
use kinein_protocol::{DataSourceEngine, DataSourceSchema, DataSourceStatements, DataSourceTable};

fn quote(name: &str) -> Option<String> {
    (!name.is_empty() && !name.contains('\0')).then(|| format!("\"{}\"", name.replace('"', "\"\"")))
}

/// Attach engine-owned instructions to every relational catalogue object.
pub fn populate(engine: DataSourceEngine, schemas: &mut [DataSourceSchema]) {
    for schema in schemas {
        for table in &mut schema.tables {
            table.statements = relational(engine, &schema.name, table);
        }
    }
}

fn relational(
    engine: DataSourceEngine,
    schema: &str,
    table: &DataSourceTable,
) -> Option<DataSourceStatements> {
    if engine == DataSourceEngine::Odbc {
        return table
            .read_sql
            .as_ref()
            .filter(|text| !text.is_empty())
            .map(|text| DataSourceStatements {
                select: text.clone(),
                ..DataSourceStatements::default()
            });
    }
    if !matches!(
        engine,
        DataSourceEngine::Postgres | DataSourceEngine::Sqlite
    ) || !matches!(table.kind.as_str(), "table" | "view")
    {
        return None;
    }
    let object = if schema.is_empty() {
        quote(&table.name)?
    } else {
        format!("{}.{}", quote(schema)?, quote(&table.name)?)
    };
    let mut result = DataSourceStatements {
        select: format!("SELECT * FROM {object} LIMIT 200;"),
        remove: Some(format!(
            "DROP {} {object};",
            if table.kind == "view" {
                "VIEW"
            } else {
                "TABLE"
            }
        )),
        ..DataSourceStatements::default()
    };
    if table.kind == "view" {
        return Some(result);
    }
    result.clear = Some(if engine == DataSourceEngine::Sqlite {
        format!("DELETE FROM {object};")
    } else {
        format!("TRUNCATE TABLE {object};")
    });
    let columns: Option<Vec<_>> = table
        .columns
        .iter()
        .map(|column| quote(&column.name))
        .collect();
    if let Some(columns) = columns.filter(|columns| !columns.is_empty()) {
        let values = vec!["<valor>"; columns.len()].join(", ");
        result.insert = Some(format!(
            "-- Revise as colunas e preencha os valores.\nINSERT INTO {object} ({})\nVALUES ({values});",
            columns.join(", ")
        ));
        let assignments = columns
            .iter()
            .map(|column| format!("{column} = <valor>"))
            .collect::<Vec<_>>()
            .join(",\n    ");
        result.update = Some(format!(
            "-- Revise as colunas e preencha valores e filtro.\nUPDATE {object}\nSET {assignments}\nWHERE <condição>;"
        ));
    }
    Some(result)
}

/// Only commands whose collection name round-trips through the existing parser.
#[must_use]
pub fn mongo(name: &str, kind: &str) -> Option<DataSourceStatements> {
    let select = format!("{name}.find({{}})");
    if super::mongo_command::parse(&select).ok()?.collection != name {
        return None;
    }
    let mut result = DataSourceStatements {
        select,
        ..DataSourceStatements::default()
    };
    let remove = format!("{name}.drop()");
    if super::mongo_command::parse(&remove).is_ok_and(|command| command.collection == name) {
        result.remove = Some(remove);
        if kind == "collection" {
            result.insert = Some(format!("{name}.insertOne(<documento JSON>)"));
            result.update = Some(format!(
                "{name}.updateMany(<filtro JSON>, {{\"$set\": <alterações JSON>}})"
            ));
            result.clear = Some(format!("{name}.deleteMany({{}})"));
        }
    }
    Some(result)
}

#[cfg(test)]
#[path = "object_statements_tests.rs"]
mod tests;
