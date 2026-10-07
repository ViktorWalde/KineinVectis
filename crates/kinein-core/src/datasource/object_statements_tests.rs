use super::*;
use kinein_protocol::{DataSourceColumn, SqlImpactSeverity};

fn table(name: &str, kind: &str) -> DataSourceTable {
    DataSourceTable {
        name: name.into(),
        kind: kind.into(),
        columns: vec![DataSourceColumn {
            name: "valor\"😀".into(),
            ..DataSourceColumn::default()
        }],
        ..DataSourceTable::default()
    }
}

#[test]
fn hostile_identifiers_stay_one_object_and_sqlite_models_cannot_execute_unfilled() {
    let name = "produto\" ; DROP TABLE intacta;--";
    let object = table(name, "table");
    let statements = relational(DataSourceEngine::Sqlite, "main", &object).unwrap();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection
        .execute_batch(&format!(
            "CREATE TABLE {} ({} TEXT); CREATE TABLE intacta(id);",
            quote(name).unwrap(),
            quote(&object.columns[0].name).unwrap()
        ))
        .unwrap();
    let insert = format!(
        "INSERT INTO {} VALUES ('valor original');",
        quote(name).unwrap()
    );
    connection.execute_batch(&insert).unwrap();
    let value: String = connection
        .query_row(&statements.select, [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "valor original");
    assert!(
        connection
            .execute_batch(statements.insert.as_ref().unwrap())
            .is_err()
    );
    assert!(
        connection
            .execute_batch(statements.update.as_ref().unwrap())
            .is_err()
    );
    assert_eq!(
        connection
            .query_row::<u32, _, _>("SELECT count(*) FROM intacta", [], |row| row.get(0))
            .unwrap(),
        0
    );
    connection
        .execute_batch(statements.clear.as_ref().unwrap())
        .unwrap();
    assert!(
        connection
            .prepare(&statements.select)
            .unwrap()
            .query([])
            .unwrap()
            .next()
            .unwrap()
            .is_none()
    );
    connection
        .execute_batch(statements.remove.as_ref().unwrap())
        .unwrap();
    assert!(connection.prepare(&statements.select).is_err());
    assert!(connection.prepare("SELECT * FROM intacta").is_ok());
}

#[test]
fn postgres_and_sqlite_clear_are_classified_for_confirmation() {
    for engine in [DataSourceEngine::Postgres, DataSourceEngine::Sqlite] {
        let statements = relational(engine, "audit\"dados", &table("linhas", "table")).unwrap();
        assert!(statements.select.contains("\"audit\"\"dados\".\"linhas\""));
        for instruction in [statements.clear.unwrap(), statements.remove.unwrap()] {
            let impact = super::super::classification::classify(engine, &instruction);
            assert_eq!(impact.len(), 1);
            assert_eq!(impact[0].severity, SqlImpactSeverity::Destructive);
        }
    }
}

#[test]
fn views_and_incomplete_metadata_do_not_advertise_write_models() {
    let statements =
        relational(DataSourceEngine::Sqlite, "main", &table("resumo", "view")).unwrap();
    assert!(
        statements.insert.is_none() && statements.update.is_none() && statements.clear.is_none()
    );
    assert_eq!(
        statements.remove.as_deref(),
        Some("DROP VIEW \"main\".\"resumo\";")
    );
    assert!(relational(DataSourceEngine::Sqlite, "main", &table("", "table")).is_none());
    assert!(relational(DataSourceEngine::Postgres, "\0", &table("x", "table")).is_none());
}

#[test]
fn odbc_keeps_driver_sql_and_never_invents_write_dialect() {
    let mut object = table("x] y", "table");
    assert!(relational(DataSourceEngine::Odbc, "db / dbo", &object).is_none());
    object.read_sql = Some("SELECT * FROM [db].[dbo].[x]] y]".into());
    let statements = relational(DataSourceEngine::Odbc, "db / dbo", &object).unwrap();
    assert_eq!(Some(statements.select), object.read_sql);
    assert!(
        statements.insert.is_none()
            && statements.update.is_none()
            && statements.clear.is_none()
            && statements.remove.is_none()
    );
}

#[test]
fn mongo_commands_round_trip_and_templates_require_real_json() {
    let statements = mongo("telemetria.sensores", "collection").unwrap();
    for instruction in [
        &statements.select,
        statements.clear.as_ref().unwrap(),
        statements.remove.as_ref().unwrap(),
    ] {
        assert_eq!(
            super::super::mongo_command::parse(instruction)
                .unwrap()
                .collection,
            "telemetria.sensores"
        );
    }
    assert!(super::super::mongo_command::parse(statements.insert.as_ref().unwrap()).is_err());
    assert!(super::super::mongo_command::parse(statements.update.as_ref().unwrap()).is_err());
    for name in ["bad name", "bad\0name", "x.find({}).evil", "x);drop();("] {
        assert!(mongo(name, "collection").is_none(), "{name}");
    }
    let view = mongo("resumo", "view").unwrap();
    assert!(view.clear.is_none() && view.update.is_none() && view.insert.is_none());
    assert!(
        mongo("system.users", "collection")
            .unwrap()
            .remove
            .is_none()
    );
}

#[test]
fn old_catalogue_payloads_still_decode_without_statements() {
    let old: DataSourceTable =
        serde_json::from_str(r#"{"name":"old","kind":"table","columns":[]}"#).unwrap();
    assert!(old.statements.is_none());
    let mut schemas = vec![DataSourceSchema {
        name: "main".into(),
        tables: vec![table("dados", "table")],
    }];
    populate(DataSourceEngine::Sqlite, &mut schemas);
    let json = serde_json::to_value(&schemas).unwrap();
    assert_eq!(
        json[0]["tables"][0]["statements"]["clear"],
        "DELETE FROM \"main\".\"dados\";"
    );
}
