//! A escrita e a CONTAGEM do aviso no `MongoDB` (`0.155.0`): o lado que fala
//! com o servidor do que o `mongo_command` validou.
//!
//! A contagem do aviso e' leitura (`countDocuments` com o MESMO filtro,
//! total exato), e a escrita so' chega aqui depois da verificacao do
//! `datasource.query`: escrita comum direta, remocao confirmada.

use std::time::Instant;

use kinein_protocol::{DataSourceProfile, SqlImpactSeverity, SqlStatementImpact};
use mongodb::bson::Document;
use mongodb::sync::{Client, Collection};

use super::mongo::{self, MongoFailure};
use super::mongo_command::{MongoCommand, MongoOp};
use super::query::QueryResult;
use super::secret::Secret;

fn collection(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    name: &str,
) -> Result<Collection<Document>, MongoFailure> {
    let client = Client::with_options(mongo::options_for(profile, secret))
        .map_err(|e| mongo::describe(&e, &profile.host))?;
    Ok(client
        .database(mongo::database_for(profile))
        .collection(name))
}

/// Executa uma escrita ja' confirmada; `affected` e' o que o servidor contou
/// (documentos inseridos, alterados, apagados, ou os que a colecao tinha).
///
/// # Errors
/// A falha do driver, na frase do `mongo::describe`.
pub fn execute(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    command: MongoCommand,
) -> Result<QueryResult, MongoFailure> {
    let target = collection(profile, secret, &command.collection)?;
    let fail = |e: mongodb::error::Error| mongo::describe(&e, &profile.host);
    let start = Instant::now();
    let affected = match command.op {
        MongoOp::Find => 0,
        MongoOp::InsertOne => {
            target
                .insert_one(command.documents.into_iter().next().unwrap_or_default())
                .run()
                .map_err(fail)?;
            1
        }
        MongoOp::InsertMany => u64::try_from(
            target
                .insert_many(command.documents)
                .run()
                .map_err(fail)?
                .inserted_ids
                .len(),
        )
        .unwrap_or(u64::MAX),
        MongoOp::UpdateOne => {
            target
                .update_one(command.filter, command.update)
                .run()
                .map_err(fail)?
                .modified_count
        }
        MongoOp::UpdateMany => {
            target
                .update_many(command.filter, command.update)
                .run()
                .map_err(fail)?
                .modified_count
        }
        MongoOp::DeleteOne => {
            target
                .delete_one(command.filter)
                .run()
                .map_err(fail)?
                .deleted_count
        }
        MongoOp::DeleteMany => {
            target
                .delete_many(command.filter)
                .run()
                .map_err(fail)?
                .deleted_count
        }
        MongoOp::Drop => {
            let had = target
                .count_documents(Document::new())
                .run()
                .map_err(fail)?;
            target.drop().run().map_err(fail)?;
            had
        }
    };
    Ok(QueryResult {
        columns: Vec::new(),
        rows: Vec::new(),
        affected: Some(affected),
        truncated: false,
        elapsed_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

/// O aviso com os numeros: quantos documentos o comando pega e quantos a
/// colecao tem. Um filtro que pega todos de uma colecao com documentos e' tao
/// destrutivo quanto filtro nenhum.
#[must_use]
pub fn measure(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
    command: &MongoCommand,
    mut statement: SqlStatementImpact,
) -> SqlStatementImpact {
    if matches!(command.op, MongoOp::InsertOne | MongoOp::InsertMany) {
        statement.rows = u64::try_from(command.documents.len()).ok();
        return statement;
    }
    let counted = collection(profile, secret, &command.collection).and_then(|target| {
        let fail = |e: mongodb::error::Error| mongo::describe(&e, &profile.host);
        // Total EXATO: com o estimado, um filtro que pega a colecao inteira
        // podia escapar da promocao a destrutivo por diferenca de contagem.
        let total = target
            .count_documents(Document::new())
            .run()
            .map_err(fail)?;
        let hit = match command.op {
            MongoOp::Drop => total,
            MongoOp::UpdateOne | MongoOp::DeleteOne => target
                .count_documents(command.filter.clone())
                .limit(1)
                .run()
                .map_err(fail)?,
            _ => target
                .count_documents(command.filter.clone())
                .run()
                .map_err(fail)?,
        };
        Ok((hit, total))
    });
    match counted {
        Ok((hit, total)) => {
            statement.rows = Some(hit);
            statement.total_rows = Some(total);
            if matches!(command.op, MongoOp::UpdateMany | MongoOp::DeleteMany)
                && total > 0
                && hit == total
            {
                statement.severity = SqlImpactSeverity::Destructive;
            }
        }
        Err(failure) => statement.note = Some(failure.message),
    }
    statement
}

/// Contra um `MongoDB` real, so' quando `KINEIN_TEST_MONGO=host:porta` (o gate
/// nao tem servidor; o pente fino roda com o `mongo:7` em conteiner).
#[cfg(test)]
mod tests {
    use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource, SqlImpactSeverity};

    use super::{execute, measure};
    use crate::datasource::mongo_command::{impact, parse};
    use crate::datasource::query::run_mongo;

    fn profile() -> Option<DataSourceProfile> {
        let address = std::env::var("KINEIN_TEST_MONGO").ok()?;
        let (host, port) = address.split_once(':')?;
        Some(DataSourceProfile {
            production: false,
            read_only: false,
            name: "teste".to_owned(),
            engine: DataSourceEngine::Mongo,
            host: host.to_owned(),
            port: port.parse().ok()?,
            database: format!("kinein_teste_{}", std::process::id()),
            user: String::new(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
            sample_size: None,
            tls: None,
            ca_file: None,
        })
    }

    #[test]
    fn writes_counts_and_the_warning_against_a_real_server() {
        let Some(profile) = profile() else {
            eprintln!("KINEIN_TEST_MONGO ausente: teste contra servidor real pulado");
            return;
        };
        let run = |text: &str| {
            execute(&profile, None, parse(text).unwrap())
                .unwrap()
                .affected
        };
        assert_eq!(
            run(
                r#"s.insertMany([{"placa": "esp32", "v": 1}, {"placa": "pico", "v": 2}, {"placa": "esp32", "v": 3}])"#
            ),
            Some(3)
        );
        assert_eq!(run(r#"s.insertOne({"placa": "pi"})"#), Some(1));

        let warn = |text: &str| {
            let command = parse(text).unwrap();
            measure(&profile, None, &command, impact(&command, text))
        };
        let filtered = warn(r#"s.deleteMany({"placa": "esp32"})"#);
        assert_eq!(
            (filtered.rows, filtered.total_rows, filtered.severity),
            (Some(2), Some(4), SqlImpactSeverity::Write)
        );
        // O filtro que pega TODOS e' tao destrutivo quanto filtro nenhum.
        let everything = warn(r#"s.updateMany({"placa": {"$exists": true}}, {"$set": {"ok": 1}})"#);
        assert_eq!(
            (everything.rows, everything.severity),
            (Some(4), SqlImpactSeverity::Destructive)
        );
        assert_eq!(warn(r#"s.deleteOne({"placa": "esp32"})"#).rows, Some(1));
        assert_eq!(warn("s.drop()").rows, Some(4));

        assert_eq!(
            run(r#"s.updateMany({"placa": "esp32"}, {"$inc": {"v": 10}})"#),
            Some(2)
        );
        assert_eq!(run(r#"s.deleteMany({"placa": "esp32"})"#), Some(2));
        let read = run_mongo(&profile, None, r#"s.find({"v": {"$gt": 1}})"#, 10).unwrap();
        assert_eq!(read.rows.len(), 1, "sobrou o pico (v=2)");
        assert_eq!(run("s.drop()"), Some(2));
        assert!(run_mongo(&profile, None, "s", 10).unwrap().rows.is_empty());
    }
}
