//! O console do `MongoDB` com ESCRITA (`0.155.0`, roadmaps/59 §5.5): a
//! gramatica, pura.
//!
//! Sem JavaScript, de proposito. O `mongosh` avalia JS; aqui so' existe uma
//! forma estrita, e os argumentos sao JSON (Extended JSON: `{"$oid": …}` vira
//! `ObjectId`):
//!
//! ```text
//! colecao.find({filtro})                 colecao {filtro}   (a forma antiga)
//! colecao.insertOne({doc})               colecao.insertMany([{doc}, …])
//! colecao.updateOne({filtro}, {mudanca}) colecao.updateMany({filtro}, {mudanca})
//! colecao.deleteOne({filtro})            colecao.deleteMany({filtro})
//! colecao.drop()
//! ```
//!
//! Seguranca (59 §7): o nome da colecao e' validado (sem `$`, sem nulo, sem
//! espaco, com teto), escrita em `system.*` e' recusada, e JSON invalido e'
//! recusado ANTES de tocar o banco. A confirmacao e' decidida por `confirm`.

use kinein_protocol::{SqlImpactSeverity, SqlStatementImpact};
use mongodb::bson::{Bson, Document};

/// O que o comando faz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MongoOp {
    /// Le documentos (o unico que nao escreve).
    Find,
    /// Insere um documento.
    InsertOne,
    /// Insere uma lista de documentos.
    InsertMany,
    /// Altera o primeiro documento que casa com o filtro.
    UpdateOne,
    /// Altera todos os que casam.
    UpdateMany,
    /// Apaga o primeiro que casa.
    DeleteOne,
    /// Apaga todos os que casam.
    DeleteMany,
    /// Remove a colecao inteira.
    Drop,
}

impl MongoOp {
    const ALL: [(&'static str, Self); 8] = [
        ("find", Self::Find),
        ("insertOne", Self::InsertOne),
        ("insertMany", Self::InsertMany),
        ("updateOne", Self::UpdateOne),
        ("updateMany", Self::UpdateMany),
        ("deleteOne", Self::DeleteOne),
        ("deleteMany", Self::DeleteMany),
        ("drop", Self::Drop),
    ];

    /// `true` para tudo que muda o banco.
    #[must_use]
    pub const fn writes(self) -> bool {
        !matches!(self, Self::Find)
    }
}

/// Um comando ja' validado.
#[derive(Debug, Clone, PartialEq)]
pub struct MongoCommand {
    /// A colecao, validada.
    pub collection: String,
    /// A operacao.
    pub op: MongoOp,
    /// O filtro (`find`/`update*`/`delete*`); vazio = todos os documentos.
    pub filter: Document,
    /// Os documentos de um `insert*`.
    pub documents: Vec<Document>,
    /// A mudanca de um `update*` (so' operadores `$set`, `$inc`…).
    pub update: Document,
}

/// Teto do nome: o `MongoDB` aceita mais, mas nome de colecao com centenas de
/// bytes no console e' erro de digitacao ou ataque.
const MAX_NAME: usize = 120;

/// Le o texto do console.
///
/// # Errors
/// A frase que diz o que esta' errado; nada foi enviado ao banco.
pub fn parse(text: &str) -> Result<MongoCommand, String> {
    let text = text.trim().trim_end_matches(';').trim_end();
    let (collection, op, args) = match split_call(text) {
        Some(call) => call,
        None => legacy_find(text)?,
    };
    validate_name(collection, op.writes())?;
    let args = parse_args(args)?;
    build(collection, op, args)
}

/// `colecao.op(args)`: identifica a operacao ANTES dos argumentos, para
/// `.find(` dentro de uma string JSON nunca mudar o comando. Nomes com ponto
/// continuam possiveis, como `logs.2026`.
fn split_call(text: &str) -> Option<(&str, MongoOp, &str)> {
    let (head, args) = text.split_once('(')?;
    let (collection, method) = head.rsplit_once('.')?;
    let args = args.strip_suffix(')')?;
    MongoOp::ALL
        .iter()
        .find_map(|(name, op)| (*name == method).then_some((collection, *op, args)))
}

/// A forma antiga de leitura: `colecao {filtro}` (filtro ausente = `{}`).
fn legacy_find(text: &str) -> Result<(&str, MongoOp, &str), String> {
    let (collection, filter) = text
        .split_once(char::is_whitespace)
        .map_or((text, ""), |(c, f)| (c, f.trim()));
    if collection.contains('(') {
        return Err(
            "comando desconhecido — use find, insertOne, insertMany, updateOne, \
                    updateMany, deleteOne, deleteMany ou drop"
                .to_owned(),
        );
    }
    Ok((collection, MongoOp::Find, filter))
}

fn validate_name(name: &str, writes: bool) -> Result<(), String> {
    if name.is_empty() {
        return Err("escreva a colecao — ex.: `sensores.find({\"placa\": \"esp32\"})`".to_owned());
    }
    if name.len() > MAX_NAME
        || name
            .chars()
            .any(|c| c == '$' || c == '\0' || c.is_whitespace() || c.is_control())
        || name.starts_with('.')
        || name.ends_with('.')
    {
        return Err(format!(
            "`{}` nao e' um nome de colecao valido (sem `$`, espaco ou controle; ate' {MAX_NAME} bytes)",
            name.chars().take(40).collect::<String>()
        ));
    }
    if writes && name.starts_with("system.") {
        return Err(
            "as colecoes `system.*` sao do proprio MongoDB; a IDE nao escreve nelas".to_owned(),
        );
    }
    Ok(())
}

/// Os argumentos como JSON: `a, b` vira o array `[a, b]`.
fn parse_args(args: &str) -> Result<Vec<Bson>, String> {
    let args = args.trim();
    if args.is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value = serde_json::from_str(&format!("[{args}]"))
        .map_err(|e| format!("os argumentos precisam ser JSON: {e}"))?;
    let serde_json::Value::Array(items) = value else {
        return Err("os argumentos precisam ser JSON".to_owned());
    };
    items
        .into_iter()
        .map(|item| Bson::try_from(item).map_err(|e| format!("JSON que o MongoDB nao aceita: {e}")))
        .collect()
}

fn document(value: Option<Bson>, what: &str) -> Result<Document, String> {
    match value {
        None => Ok(Document::new()),
        Some(Bson::Document(doc)) => Ok(doc),
        Some(_) => Err(format!("{what} precisa ser um objeto JSON (`{{…}}`)")),
    }
}

fn build(collection: &str, op: MongoOp, args: Vec<Bson>) -> Result<MongoCommand, String> {
    let expected = match op {
        MongoOp::Drop => 0..=0,
        MongoOp::Find | MongoOp::DeleteOne | MongoOp::DeleteMany => 0..=1,
        MongoOp::InsertOne | MongoOp::InsertMany => 1..=1,
        MongoOp::UpdateOne | MongoOp::UpdateMany => 2..=2,
    };
    if !expected.contains(&args.len()) {
        return Err(format!(
            "{} recebe {} argumento(s), veio {}",
            name_of(op),
            if expected.start() == expected.end() {
                expected.start().to_string()
            } else {
                format!("{} ou {}", expected.start(), expected.end())
            },
            args.len()
        ));
    }
    let mut args = args.into_iter();
    let mut command = MongoCommand {
        collection: collection.to_owned(),
        op,
        filter: Document::new(),
        documents: Vec::new(),
        update: Document::new(),
    };
    match op {
        MongoOp::InsertOne => command.documents = vec![document(args.next(), "o documento")?],
        MongoOp::InsertMany => {
            let Some(Bson::Array(items)) = args.next() else {
                return Err("insertMany recebe uma lista: `[{…}, {…}]`".to_owned());
            };
            if items.is_empty() {
                return Err("insertMany recebeu uma lista vazia".to_owned());
            }
            command.documents = items
                .into_iter()
                .map(|item| document(Some(item), "cada documento"))
                .collect::<Result<_, _>>()?;
        }
        MongoOp::UpdateOne | MongoOp::UpdateMany => {
            command.filter = document(args.next(), "o filtro")?;
            command.update = document(args.next(), "a mudanca")?;
            // So' operadores: um documento "cru" no update SUBSTITUIRIA o
            // documento inteiro — o driver recusaria, mas a frase daqui diz o
            // que fazer.
            if command.update.is_empty() || command.update.keys().any(|k| !k.starts_with('$')) {
                return Err(
                    "a mudanca usa operadores — ex.: `{\"$set\": {\"ativo\": false}}`".to_owned(),
                );
            }
        }
        MongoOp::Find | MongoOp::DeleteOne | MongoOp::DeleteMany => {
            command.filter = document(args.next(), "o filtro")?;
        }
        MongoOp::Drop => {}
    }
    Ok(command)
}

fn name_of(op: MongoOp) -> &'static str {
    MongoOp::ALL
        .iter()
        .find(|(_, candidate)| *candidate == op)
        .map_or("?", |(name, _)| name)
}

/// O que o aviso mostra, antes da contagem. Filtro vazio e `drop` sao
/// destrutivos; `deleteOne`/`updateOne` tocam no maximo um documento.
#[must_use]
pub fn impact(command: &MongoCommand, text: &str) -> SqlStatementImpact {
    let filtered = !command.filter.is_empty();
    let (kind, severity) = match command.op {
        MongoOp::Find => ("read", SqlImpactSeverity::Read),
        MongoOp::InsertOne | MongoOp::InsertMany => ("mongoInsert", SqlImpactSeverity::Write),
        MongoOp::UpdateOne => ("mongoUpdate", SqlImpactSeverity::Write),
        MongoOp::DeleteOne => ("mongoDelete", SqlImpactSeverity::Write),
        MongoOp::UpdateMany | MongoOp::DeleteMany => (
            if command.op == MongoOp::UpdateMany {
                "mongoUpdate"
            } else {
                "mongoDelete"
            },
            if filtered {
                SqlImpactSeverity::Write
            } else {
                SqlImpactSeverity::Destructive
            },
        ),
        MongoOp::Drop => ("dropCollection", SqlImpactSeverity::Destructive),
    };
    SqlStatementImpact {
        text: text.trim().to_owned(),
        kind: kind.to_owned(),
        targets: vec![command.collection.clone()],
        filter: if filtered {
            serde_json::to_string(&command.filter).unwrap_or_default()
        } else {
            String::new()
        },
        severity,
        ..SqlStatementImpact::default()
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{doc, oid::ObjectId};

    use super::{MongoOp, impact, parse};
    use kinein_protocol::SqlImpactSeverity;

    #[test]
    fn every_form_parses_and_the_old_read_still_works() {
        let find = parse("sensores {\"placa\": \"esp32\"}").unwrap();
        assert_eq!(
            (find.op, find.filter),
            (MongoOp::Find, doc! {"placa": "esp32"})
        );
        assert_eq!(parse("sensores").unwrap().filter, doc! {});
        let dotted = parse("logs.2026.deleteMany({\"nivel\": \"debug\"});").unwrap();
        assert_eq!(
            (dotted.collection.as_str(), dotted.op),
            ("logs.2026", MongoOp::DeleteMany)
        );
        let many = parse("s.insertMany([{\"a\": 1}, {\"a\": 2}])").unwrap();
        assert_eq!(many.documents.len(), 2);
        let update = parse("s.updateOne({\"a\": 1}, {\"$set\": {\"b\": 2}})").unwrap();
        assert_eq!(update.update, doc! {"$set": {"b": 2}});
        assert_eq!(parse("s.drop()").unwrap().op, MongoOp::Drop);
        // Extended JSON: o `_id` vira ObjectId de verdade.
        let id = ObjectId::new();
        let by_id = parse(&format!("s.deleteOne({{\"_id\": {{\"$oid\": \"{id}\"}}}})")).unwrap();
        assert_eq!(by_id.filter, doc! {"_id": id});
    }

    #[test]
    fn what_could_hurt_is_refused_before_the_bank() {
        for bad in [
            "s.insertOne({a: 1})",                 // nao e' JSON
            "s.updateMany({}, {\"b\": 1})",        // update sem operador = substituir
            "s.insertMany([])",                    // lista vazia
            "s.insertMany({\"a\": 1})",            // nao e' lista
            "s.drop({})",                          // drop nao tem argumento
            "s.eval(\"db.dropDatabase()\")",       // nao ha' JS
            "system.users.deleteMany({})",         // colecao do proprio MongoDB
            "a$b.find({})",                        // `$` no nome
            "a b.drop()",                          // espaco no nome
            ".s.drop()",                           // ponto na ponta
            "s.deleteOne({\"a\": 1}, {\"b\": 2})", // argumento a mais
        ] {
            assert!(parse(bad).is_err(), "deveria recusar: {bad}");
        }
        assert!(parse(&format!("{}.drop()", "x".repeat(121))).is_err());
        // Ler `system.*` continua possivel; escrever nao.
        assert!(parse("system.version.find({})").is_ok());
    }

    #[test]
    fn empty_filters_and_drop_are_destructive_and_inserts_are_writes() {
        let severity = |text: &str| impact(&parse(text).unwrap(), text).severity;
        assert_eq!(severity("s.deleteMany({})"), SqlImpactSeverity::Destructive);
        assert_eq!(
            severity("s.updateMany({}, {\"$set\": {\"x\": 1}})"),
            SqlImpactSeverity::Destructive
        );
        assert_eq!(severity("s.drop()"), SqlImpactSeverity::Destructive);
        assert_eq!(
            severity("s.deleteMany({\"a\": 1})"),
            SqlImpactSeverity::Write
        );
        assert_eq!(severity("s.deleteOne({})"), SqlImpactSeverity::Write);
        assert_eq!(
            severity("s.insertOne({\"a\": 1})"),
            SqlImpactSeverity::Write
        );
        assert_eq!(severity("s.find({})"), SqlImpactSeverity::Read);
        let drop = impact(&parse("s.drop()").unwrap(), "s.drop()");
        assert_eq!(
            (drop.kind.as_str(), drop.targets.as_slice()),
            ("dropCollection", ["s".to_owned()].as_slice())
        );
    }
}

#[cfg(test)]
mod argument_tests {
    use super::{MongoOp, parse};
    #[test]
    fn method_names_inside_json_strings_never_change_the_operation() {
        for value in ["log.find(", "x.drop(", "s.deleteMany(", "s.updateOne("] {
            let text = format!("s.insertOne({{\"text\": \"{value}\"}})");
            let command = parse(&text).unwrap();
            assert_eq!(command.op, MongoOp::InsertOne);
            assert_eq!(command.collection, "s");
            assert_eq!(command.documents[0].get_str("text").unwrap(), value);
        }
        assert!(parse("s.find({}).drop()").is_err());
        assert!(parse("s.find({});s.drop()").is_err());
    }
}
