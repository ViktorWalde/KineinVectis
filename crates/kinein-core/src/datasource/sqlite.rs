//! `SQLite`: um ARQUIVO, sem servidor, sem porta e sem senha.
//!
//! POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Decisao do autor: a IDE precisa
//! falar com bancos relacionais, temporais e nao-relacionais. O `SQLite` e' o
//! primeiro passo dessa lista e o mais barato de todos — o `rusqlite` JA' e'
//! dependencia deste core desde a rede de seguranca de rascunhos
//! (`docs/seguranca/23`). Zero crate nova, zero auditoria de licenca.
//!
//! # O que muda em relacao ao `PostgreSQL`
//!
//! ```text
//! nao ha' servidor    o "host" e' o proprio arquivo; nao existe porta
//! nao ha' usuario     quem abre o arquivo e' o processo, com a permissao dele
//! nao ha' senha       e por isso a politica de segredo nao se aplica
//! ```
//!
//! Isso e' o oposto do que uma IDE costuma fazer com `SQLite`: tratar o
//! caminho como se fosse um host e deixar campos vazios na tela. O perfil
//! carrega o `engine`, e a validacao cobra exatamente o que aquele motor pede.
//!
//! # O catalogo do `SQLite` e o do `PostgreSQL` respondem a mesma pergunta
//!
//! `sqlite_master` faz aqui o que o `information_schema` faz la': diz o que
//! existe dentro. A arvore devolvida e' a MESMA (`esquema -> tabela ->
//! coluna`), com um unico esquema chamado `main`, que e' como o proprio
//! `SQLite` chama o banco principal. A UI nao ganha um segundo formato.

use kinein_protocol::{DataSourceColumn, DataSourceProfile, DataSourceSchema, DataSourceTable};
use rusqlite::Connection;

use super::connection::ConnectionFailure;

/// Teto de tabelas lidas, pelo mesmo motivo do lado `PostgreSQL`.
const MAX_TABLES: usize = 5_000;

/// Abre o arquivo e devolve a versao do `SQLite` embutido.
///
/// # Errors
/// Arquivo inexistente, sem permissao, ou que nao e' um banco `SQLite`.
pub fn probe_file(profile: &DataSourceProfile) -> Result<String, ConnectionFailure> {
    let conexao = abrir(profile)?;
    // `SELECT sqlite_version()` responde a MESMA pergunta que o
    // `SELECT version()` do Postgres: "com o que eu estou falando?".
    let versao: String = conexao
        .query_row("SELECT sqlite_version()", [], |linha| linha.get(0))
        .map_err(|erro| falha(&erro))?;
    Ok(format!("SQLite {versao}"))
}

/// Le a estrutura do arquivo: tabelas, views e colunas.
///
/// # Errors
/// As mesmas do [`probe_file`].
pub fn read_structure(
    profile: &DataSourceProfile,
) -> Result<Vec<DataSourceSchema>, ConnectionFailure> {
    let conexao = abrir(profile)?;

    // `sqlite_master` e' o catalogo do proprio SQLite. O filtro tira as
    // tabelas internas (`sqlite_sequence`, `sqlite_stat1`): sao do motor, nao
    // do autor — a mesma regra que exclui `pg_catalog` do lado Postgres.
    let mut consulta = conexao
        .prepare(
            "SELECT name, type FROM sqlite_master \
             WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite\\_%' ESCAPE '\\' \
             ORDER BY name",
        )
        .map_err(|erro| falha(&erro))?;
    let encontradas: Vec<(String, String)> = consulta
        .query_map([], |linha| Ok((linha.get(0)?, linha.get(1)?)))
        .map_err(|erro| falha(&erro))?
        .filter_map(Result::ok)
        .take(MAX_TABLES)
        .collect();

    let mut tabelas = Vec::with_capacity(encontradas.len());
    for (nome, tipo) in encontradas {
        tabelas.push(DataSourceTable {
            columns: colunas_de(&conexao, &nome)?,
            kind: if tipo == "view" { "view" } else { "table" }.to_owned(),
            name: nome,
        });
    }

    // UM esquema, chamado `main`, que e' como o proprio SQLite chama o banco
    // principal. Inventar outro nome faria a tela mentir sobre o motor.
    Ok(vec![DataSourceSchema {
        name: "main".to_owned(),
        tables: tabelas,
    }])
}

/// Colunas de uma tabela, na ordem em que foram declaradas.
fn colunas_de(
    conexao: &Connection,
    tabela: &str,
) -> Result<Vec<DataSourceColumn>, ConnectionFailure> {
    // `pragma_table_info` e' a forma consultavel do `PRAGMA table_info`, e
    // aceita PARAMETRO — o `PRAGMA` cru exigiria concatenar o nome da tabela
    // na string, que e' injecao de SQL com nome vindo do banco.
    let mut consulta = conexao
        .prepare("SELECT name, type, \"notnull\" FROM pragma_table_info(?1) ORDER BY cid")
        .map_err(|erro| falha(&erro))?;
    let colunas = consulta
        .query_map([tabela], |linha| {
            let nao_nulo: i64 = linha.get(2)?;
            Ok(DataSourceColumn {
                name: linha.get(0)?,
                data_type: linha.get(1)?,
                nullable: nao_nulo == 0,
            })
        })
        .map_err(|erro| falha(&erro))?
        .filter_map(Result::ok)
        .collect();
    Ok(colunas)
}

/// Abre o arquivo do perfil, recusando cedo o que nao existe.
fn abrir(profile: &DataSourceProfile) -> Result<Connection, ConnectionFailure> {
    let caminho = profile.database.trim();
    if caminho.is_empty() {
        return Err(ConnectionFailure {
            message: "informe o caminho do arquivo .db".to_owned(),
            sql_state: None,
            secret_required: false,
        });
    }
    // ABRIR SEM CRIAR. O `Connection::open` cria o arquivo se ele nao existe, e
    // "testar a conexao" que CRIA um banco vazio seria a IDE inventando dado —
    // o autor pediria um teste e ganharia um arquivo.
    if !std::path::Path::new(caminho).is_file() {
        return Err(ConnectionFailure {
            message: format!("{caminho}: arquivo nao encontrado"),
            sql_state: None,
            secret_required: false,
        });
    }
    Connection::open(caminho).map_err(|erro| falha(&erro))
}

/// Erro do `rusqlite` na forma que a UI ja' entende.
///
/// `secret_required` e' SEMPRE `false`, e a constante diz algo: `SQLite` nao
/// tem autenticacao. Pedir senha aqui seria um dialogo que nao resolve nada.
fn falha(erro: &rusqlite::Error) -> ConnectionFailure {
    ConnectionFailure {
        message: super::connection::describe(erro),
        sql_state: None,
        secret_required: false,
    }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::{DataSourceEngine, SecretSource};

    use super::*;

    fn perfil(caminho: &str) -> DataSourceProfile {
        DataSourceProfile {
            engine: DataSourceEngine::Sqlite,
            name: "arquivo".to_owned(),
            host: String::new(),
            port: 0,
            database: caminho.to_owned(),
            user: String::new(),
            secret_source: SecretSource::Automatic,
            secret_variable: None,
        }
    }

    fn temp_db(nome: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-datasource-sqlite")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("dados.db");
        let conexao = Connection::open(&caminho).unwrap();
        conexao
            .execute_batch(
                "CREATE TABLE pedidos (id INTEGER PRIMARY KEY, cliente TEXT NOT NULL, total REAL);\
                 CREATE VIEW resumo AS SELECT cliente FROM pedidos;",
            )
            .unwrap();
        caminho
    }

    #[test]
    fn le_tabela_view_e_colunas_na_ordem_declarada() {
        let caminho = temp_db("estrutura");
        let esquemas = read_structure(&perfil(caminho.to_str().unwrap())).unwrap();

        assert_eq!(esquemas.len(), 1, "SQLite tem UM esquema, chamado main");
        assert_eq!(esquemas[0].name, "main");

        let tabelas = &esquemas[0].tables;
        let pedidos = tabelas.iter().find(|t| t.name == "pedidos").unwrap();
        assert_eq!(pedidos.kind, "table");
        let nomes: Vec<_> = pedidos.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(nomes, vec!["id", "cliente", "total"], "ordem de declaracao");
        assert!(!pedidos.columns[1].nullable, "cliente e' NOT NULL");
        assert!(pedidos.columns[2].nullable, "total aceita NULL");

        let resumo = tabelas.iter().find(|t| t.name == "resumo").unwrap();
        assert_eq!(resumo.kind, "view", "view e tabela nao sao a mesma coisa");
    }

    /// Tabela interna do motor nao e' do autor — a mesma regra que exclui o
    /// `pg_catalog` do lado `PostgreSQL`.
    #[test]
    fn tabela_interna_do_motor_fica_de_fora() {
        let caminho = temp_db("internas");
        let conexao = Connection::open(&caminho).unwrap();
        conexao
            .execute_batch("CREATE TABLE t (id INTEGER PRIMARY KEY AUTOINCREMENT);")
            .unwrap();
        drop(conexao);

        let esquemas = read_structure(&perfil(caminho.to_str().unwrap())).unwrap();
        assert!(
            !esquemas[0]
                .tables
                .iter()
                .any(|t| t.name.starts_with("sqlite_")),
            "tabela interna do SQLite apareceu na lista do autor"
        );
    }

    /// TESTAR NAO PODE CRIAR. `Connection::open` cria o arquivo se ele nao
    /// existe, e um teste de conexao que inventa um banco vazio e' pior que um
    /// erro: o autor pediria um diagnostico e ganharia um arquivo.
    #[test]
    fn arquivo_inexistente_falha_sem_criar_nada() {
        let dir = std::env::temp_dir()
            .join("kinein-datasource-sqlite")
            .join(format!("{}-ausente", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let caminho = dir.join("nao-existe.db");

        let erro = probe_file(&perfil(caminho.to_str().unwrap())).unwrap_err();
        assert!(erro.message.contains("nao encontrado"), "{}", erro.message);
        assert!(!caminho.exists(), "o teste de conexao CRIOU o arquivo");
        assert!(!erro.secret_required, "SQLite nao tem senha para pedir");
    }

    #[test]
    fn caminho_vazio_diz_o_que_falta() {
        let erro = probe_file(&perfil("   ")).unwrap_err();
        assert!(erro.message.contains(".db"), "{}", erro.message);
    }
}
