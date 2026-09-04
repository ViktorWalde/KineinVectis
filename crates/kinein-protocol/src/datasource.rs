//! Types for the `datasource.*` domain: connections to the user's databases.
//!
//! **There is no password field in this file, and that is the point.** The
//! decision is registered in `docs/seguranca/40-cofre-de-credencial.md`
//! (author, 2026-09-04): the IDE owns the PROFILE — which is not a secret —
//! and never writes a password to disk. The password reaches the driver from
//! the session prompt (memory only), from an environment variable, or from
//! `PostgreSQL`'s own `~/.pgpass`.
//!
//! Adding a `password` field here would silently undo that decision, since the
//! profile is what gets persisted.

use serde::{Deserialize, Serialize};

/// Where the password for a profile comes from, when a connection is opened.
///
/// This is a POLICY, not a secret: it is safe to persist because it says
/// *where to look*, never *what was found*.
///
/// # Why there is a "no password at all" default
///
/// The IDE is not the one asking for a password — the user's own server is. A
/// local development `PostgreSQL` reached over a Unix socket with `peer`
/// authentication has the OS user *as* the identity, and `trust` asks for
/// nothing either. Making the author type a password in that case would be an
/// obstacle the IDE invented on its own.
///
/// So the default is [`SecretSource::Automatic`]: send no password and let the
/// server decide. If it does demand one, the UI can still ask — falling back
/// costs one round trip and is invisible when it is not needed.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SecretSource {
    /// Send no password: let the server and libpq settle it.
    ///
    /// This one option covers three real cases at once — Unix socket with
    /// `peer` auth, `trust` on a dev box, and `~/.pgpass` / `PGPASSFILE`
    /// (which libpq reads on its own, requiring mode 0600 and ignoring the
    /// file entirely when it is looser).
    #[default]
    Automatic,
    /// Read from an environment variable, named by the profile.
    Environment,
    /// Ask the author when the connection is opened; keep it in memory for the
    /// session only. Nothing is written to disk.
    Prompt,
}

/// Which engine a profile talks to.
///
/// POR QUE ISTO EXISTE (2026-09-04). Decisao do autor: a IDE precisa falar com
/// bancos **relacionais, temporais e nao-relacionais**, e o perfil ate' aqui
/// assumia `PostgreSQL` — `host`, `port`, `user`. Um arquivo `.db` do `SQLite` nao
/// tem nenhum dos tres, e forcar os campos vazios seria pedir ao autor que
/// preenchesse o que nao existe.
///
/// `TimescaleDB` NAO e' um valor daqui, e a ausencia e' a resposta certa: ele
/// e' uma EXTENSAO do `PostgreSQL`, fala o mesmo protocolo e usa o mesmo driver.
/// Inventar um valor para ele criaria dois caminhos identicos com nomes
/// diferentes. A IDE detecta a extensao DEPOIS de conectar, e mostra.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceEngine {
    /// `PostgreSQL` e tudo que fala o protocolo dele — `TimescaleDB` incluso.
    #[default]
    Postgres,
    /// `SQLite`: um ARQUIVO, sem servidor, sem porta e sem usuario.
    Sqlite,
    /// `MongoDB`: colecao -> documento, sem esquema fixo.
    ///
    /// POR QUE ELE NAO CABE NA MESMA ARVORE (decisao do autor, 2026-09-04).
    /// `PostgreSQL` e `SQLite` respondem `esquema -> tabela -> coluna`, e a
    /// coluna carrega tres garantias: ela existe em toda linha, tem UM tipo, e
    /// nao aninha. Nenhuma das tres vale aqui. Desenhar um documento como
    /// linha faria a tela AFIRMAR as tres — e a tela mentindo e' o defeito que
    /// nenhum gate pega.
    ///
    /// Por isso o resultado da introspeccao tem uma segunda forma
    /// ([`MongoCollection`]), e nao um preenchimento criativo da primeira.
    Mongo,
}

/// A saved connection to a database. Never carries a password.
///
/// `deny_unknown_fields` is deliberate and load-bearing here, not tidiness.
/// Without it, a UI that sent `"password": "..."` would be answered with a
/// cheerful success while the field was silently dropped — the caller would
/// believe the password had been stored. Refusing is the honest answer, and a
/// test in `tests/datasource.rs` pins it.
///
/// The cost is accepted knowingly: a field removed from this struct later can
/// no longer be read back from an older `.kinein/datasources.json`. For a
/// boundary that guards a secret, strict beats forgiving.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceProfile {
    /// Stable identity, unique per workspace. Also what the UI shows.
    pub name: String,
    /// Which engine this profile talks to.
    ///
    /// Defaults to `postgres` so profiles saved before 2026-09-04 keep
    /// working: they were all `PostgreSQL`, and a missing field means exactly
    /// that.
    #[serde(default)]
    pub engine: DataSourceEngine,
    /// Host name, address, or — starting with `/` — a Unix socket DIRECTORY.
    ///
    /// libpq treats a host beginning with a slash as a socket directory (for
    /// example `/var/run/postgresql`), which is the usual way to reach a local
    /// server with `peer` authentication and no password at all.
    pub host: String,
    /// TCP port.
    pub port: u16,
    /// Database name — or, for `SQLite`, the PATH of the `.db` file.
    ///
    /// One field for two meanings is deliberate: it is "what to open" in both
    /// engines, and the UI labels it per engine. A second field would be empty
    /// half the time and would make "which one is filled?" a question the
    /// reader has to ask.
    pub database: String,
    /// Role used to connect.
    pub user: String,
    /// Where the password comes from.
    #[serde(default)]
    pub secret_source: SecretSource,
    /// Environment variable holding the password, for `Environment`.
    ///
    /// Only the NAME of the variable lives here; the value is read at connect
    /// time and never stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_variable: Option<String>,
    /// Quantos documentos a amostra le', para os motores sem esquema fixo.
    ///
    /// So' o `Mongo` usa este campo, e ele mora aqui — e nao num tipo proprio —
    /// porque e' preferencia do PERFIL: dois bancos Mongo do mesmo autor podem
    /// merecer amostras de tamanhos diferentes, e a escolha precisa sobreviver
    /// ao fechamento da IDE.
    ///
    /// Ausente significa o padrao do core. O motivo de o padrao NAO ser os
    /// 1.000 do Compass esta' no `crate::datasource::mongo`: o `$sample` do
    /// `MongoDB` varre a colecao inteira quando N nao e' menor que 5% dela, e
    /// 1.000 dispara essa varredura em toda colecao com menos de 20.000
    /// documentos.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_size: Option<u32>,
}

/// Result payload for `datasource.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceListResult {
    /// Saved profiles, ordered by name.
    pub profiles: Vec<DataSourceProfile>,
}

/// Parameters for `datasource.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceListParams {}

/// Parameters for `datasource.save` — creates or replaces by `name`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceSaveParams {
    /// The profile to store.
    pub profile: DataSourceProfile,
}

/// Parameters for `datasource.remove`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceRemoveParams {
    /// Name of the profile to drop.
    pub name: String,
}

/// Result payload for `datasource.save` and `datasource.remove`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceWriteResult {
    /// The catalogue after the write, ordered by name.
    pub profiles: Vec<DataSourceProfile>,
}

/// Parameters for `datasource.test` — connect once and report back.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceTestParams {
    /// Which saved profile to try.
    pub name: String,
    /// The session password, when the profile's policy is `Prompt`.
    ///
    /// This is the ONE place a password crosses the wire, and it crosses a
    /// pipe between two processes of the same user — never the disk. The Qt
    /// client redacts fields named like this one before writing its request
    /// log, because that log is also written to a file
    /// (`ui/src/core_client_process.cpp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Result payload for `datasource.test` — the job that will report the answer.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceTestAccepted {
    /// Job to follow; the answer arrives as `event.datasource.tested`.
    pub job_id: String,
}

/// One column of a table, in the order it was declared.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceColumn {
    /// Column name.
    pub name: String,
    /// SQL type as `information_schema` reports it (`integer`, `text`, ...).
    pub data_type: String,
    /// Whether the column accepts NULL.
    pub nullable: bool,
}

/// A table or a view, with its columns.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceTable {
    /// Table name.
    pub name: String,
    /// `"table"` or `"view"` — the UI draws them differently.
    pub kind: String,
    /// Columns, in declaration order.
    pub columns: Vec<DataSourceColumn>,
}

/// A schema the author owns; server catalogues are filtered out.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceSchema {
    /// Schema name.
    pub name: String,
    /// Tables and views in it.
    pub tables: Vec<DataSourceTable>,
}

/// Parameters for `datasource.introspect` — same shape as the connection test.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceIntrospectParams {
    /// Which saved profile to read.
    pub name: String,
    /// The session password, when the profile's policy is `Prompt`.
    ///
    /// Redacted from the client log exactly like `datasource.test`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Um campo de documento, achatado pelo CAMINHO ate' ele.
///
/// O caminho e' a chave: `meta.placa` diz o que `meta` -> `placa` diria numa
/// arvore, e permite que a lista seja plana no protocolo e vire arvore na
/// tela sem que nenhum dos dois lados precise de recursao.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoField {
    /// Caminho completo, com pontos: `carga.origem`.
    pub path: String,
    /// Quantos pontos ha' no caminho. A UI indenta por este numero.
    pub depth: u16,
    /// Os tipos BSON vistos neste caminho, ordenados.
    ///
    /// MAIS DE UM NAO E' ERRO, e e' justamente o que uma coluna nao consegue
    /// dizer: o mesmo campo pode ser `string` num documento e `int` noutro, e
    /// e' isso que quebra o codigo de quem assumiu um so'.
    pub types: Vec<String>,
    /// Fracao dos documentos amostrados em que o campo apareceu, de 0 a 1.
    ///
    /// `None` quando o campo veio de um `$jsonSchema` DECLARADO: ali nao houve
    /// amostra, e inventar uma porcentagem seria a tela afirmando uma medicao
    /// que nao aconteceu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<f64>,
    /// Declarado como obrigatorio pelo validador. Sempre `false` no inferido.
    #[serde(default)]
    pub required: bool,
}

/// Uma colecao, com os campos DECLARADOS ou INFERIDOS — nunca os dois.
///
/// A distincao e' a decisao central desta frente (autor, 2026-09-04): quando a
/// colecao tem validador `$jsonSchema`, a IDE mostra o esquema REAL e diz que
/// e' declarado; sem validador, ela amostra e diz quantos documentos leu. As
/// duas coisas nunca se parecem na tela.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MongoCollection {
    /// Nome da colecao.
    pub name: String,
    /// `"collection"`, `"view"` ou `"timeseries"`, como o servidor classifica.
    pub kind: String,
    /// `true` quando os campos vieram de um validador, `false` quando de amostra.
    pub declared: bool,
    /// Campo de tempo declarado, para colecao temporal.
    ///
    /// COLECAO TEMPORAL SE ANUNCIA SOZINHA: isto sai do catalogo do servidor,
    /// sem ler um documento. Nao ha' inferencia envolvida, e por isso ele nao
    /// aparece como campo comum no meio dos outros.
    #[serde(default)]
    pub time_field: String,
    /// Campo de metadados declarado, para colecao temporal.
    #[serde(default)]
    pub meta_field: String,
    /// Granularidade declarada (`seconds`, `minutes`, `hours`).
    #[serde(default)]
    pub granularity: String,
    /// Quantos documentos a colecao tem. `None` para visao, que nao e' contada.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_count: Option<u64>,
    /// Quantos documentos a amostra leu. `0` quando os campos sao declarados.
    #[serde(default)]
    pub sampled: u32,
    /// `true` quando a amostra obrigou o servidor a VARRER a colecao inteira.
    ///
    /// O `$sample` do `MongoDB` so' usa o cursor pseudoaleatorio barato quando
    /// N e' menor que 5% da colecao E ela tem mais de 100 documentos. Fora
    /// disso ele le' tudo e ordena. A IDE calcula isso ANTES e conta na tela,
    /// em vez de disparar trabalho no banco do autor em silencio.
    #[serde(default)]
    pub full_scan: bool,
    /// O que a leitura cortou, em uma frase. Vazio quando nada foi cortado.
    #[serde(default)]
    pub truncated: String,
    /// Campos, ordenados pelo caminho.
    pub fields: Vec<MongoField>,
}
