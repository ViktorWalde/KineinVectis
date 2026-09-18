//! O que existe DENTRO do banco: esquemas, tabelas e colunas.
//!
//! POR QUE ESTE ARQUIVO EXISTE (2026-09-04). O perfil diz onde conectar e o
//! `probe_server` prova que da' para conectar. Nenhum dos dois responde a
//! primeira pergunta de quem abre um cliente de banco: *"o que tem aqui
//! dentro?"*. Sem essa resposta o painel e' um testador de conexao, nao um
//! cliente.
//!
//! COMO SE PERGUNTA. Tres consultas ao `information_schema`, que e' padrao
//! SQL e por isso vale igual no `PostgreSQL` e no `TimescaleDB` — que fala o
//! mesmo protocolo e expoe os mesmos catalogos. Nao ha' nada especifico de
//! fornecedor aqui, e isso e' deliberado: a etapa 27 herda esta leitura sem
//! reescrever nada.
//!
//! O QUE FICA DE FORA, e o motivo:
//!
//! ```text
//! pg_catalog, information_schema   catalogo do proprio servidor; ninguem
//!                                  abre uma IDE para olhar isso
//! pg_toast, pg_temp_*              armazenamento interno do PostgreSQL
//! ```
//!
//! TETO. Um banco de producao tem dezenas de milhares de colunas, e mandar
//! tudo pela pipe travaria a UI antes de desenhar a primeira linha. O teto e'
//! por CONSULTA e o resultado diz quando truncou — "mostrei parte" e' honesto,
//! "mostrei tudo" quando nao mostrou nao e'.

use super::connection::{ConnectionFailure, connect, describe, failure_from};
use super::secret::Secret;
use kinein_protocol::{DataSourceColumn, DataSourceProfile, DataSourceSchema, DataSourceTable};

/// Teto de linhas por consulta de catalogo.
const MAX_ROWS: i64 = 5_000;

/// Esquemas que sao do servidor, nao do autor.
const SQL_SCHEMAS: &str = "\
    SELECT nspname FROM pg_namespace \
    WHERE nspname NOT IN ('pg_catalog', 'information_schema') \
      AND nspname NOT LIKE 'pg\\_%' \
    ORDER BY nspname LIMIT $1";

/// Tabelas e views, com o tipo, para a UI distinguir.
const SQL_TABLES: &str = "\
    SELECT table_schema, table_name, table_type \
    FROM information_schema.tables \
    WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
      AND table_schema NOT LIKE 'pg\\_%' \
    ORDER BY table_schema, table_name LIMIT $1";

/// Colunas na ORDEM em que foram declaradas — a ordem que o autor escreveu.
const SQL_COLUMNS: &str = "\
    SELECT table_schema, table_name, column_name, data_type, is_nullable \
    FROM information_schema.columns \
    WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
      AND table_schema NOT LIKE 'pg\\_%' \
    ORDER BY table_schema, table_name, ordinal_position LIMIT $1";

/// Le a estrutura do banco de um perfil.
///
/// # Errors
/// A mesma [`ConnectionFailure`] do teste de conexao — inclusive
/// `secret_required`, porque introspectar tambem exige autenticar.
pub fn read_structure(
    profile: &DataSourceProfile,
    secret: Option<&Secret>,
) -> Result<Vec<DataSourceSchema>, ConnectionFailure> {
    let mut client = connect(profile, secret)?;

    let mut schemas: Vec<DataSourceSchema> = client
        .query(SQL_SCHEMAS, &[&MAX_ROWS])
        .map_err(|erro| failure_from(&erro))?
        .iter()
        .map(|linha| DataSourceSchema {
            name: linha.get(0),
            tables: Vec::new(),
        })
        .collect();

    for linha in client
        .query(SQL_TABLES, &[&MAX_ROWS])
        .map_err(|erro| failure_from(&erro))?
    {
        let esquema: String = linha.get(0);
        let tipo: String = linha.get(2);
        if let Some(alvo) = schemas.iter_mut().find(|s| s.name == esquema) {
            alvo.tables.push(DataSourceTable {
                name: linha.get(1),
                // "BASE TABLE" e "VIEW" sao os valores do padrao SQL; a UI
                // desenha diferente, entao o valor cru vai junto.
                kind: if tipo == "VIEW" { "view" } else { "table" }.to_owned(),
                columns: Vec::new(),
            });
        }
    }

    for linha in client
        .query(SQL_COLUMNS, &[&MAX_ROWS])
        .map_err(|erro| failure_from(&erro))?
    {
        let esquema: String = linha.get(0);
        let tabela: String = linha.get(1);
        let anulavel: String = linha.get(4);
        if let Some(alvo) = schemas
            .iter_mut()
            .find(|s| s.name == esquema)
            .and_then(|s| s.tables.iter_mut().find(|t| t.name == tabela))
        {
            alvo.columns.push(DataSourceColumn {
                name: linha.get(2),
                data_type: linha.get(3),
                nullable: anulavel == "YES",
            });
        }
    }

    Ok(schemas)
}

/// A cadeia de causas de um erro, para quem so' tem a mensagem.
///
/// Reexportado aqui porque a introspeccao e o teste de conexao contam a mesma
/// historia ao autor, e duas frases diferentes para a mesma falha seriam duas
/// verdades sobre um fato so'.
#[must_use]
pub fn describe_error(error: &dyn std::error::Error) -> String {
    describe(error)
}
