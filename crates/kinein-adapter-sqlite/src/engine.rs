//! As operacoes do adaptador: sessoes e o motor `SQLite` que o core ja' tem.

use std::collections::BTreeMap;

use kinein_core::datasource::{classification, impact, measurement, query, sqlite};
use kinein_protocol::driver::operation::{
    Context, OptionValue, Request, StatementParams, TerminalResult,
};
use kinein_protocol::driver::{
    ApiRange, Error, Failure, FailureReason, InitializeParams, Limits, Operation, OperationOutcome,
};
use kinein_protocol::{
    DataSourceCatalogUpdate, DataSourceEngine, DataSourceProfile, JsonRpcId, SecretSource,
};
use serde_json::{Value, json};

use crate::budget::Stream;

/// Uma sessao aberta: o arquivo e a restricao de escrita que o core mandou.
#[derive(Debug)]
struct Session {
    path: String,
    read_only: bool,
}

/// O estado do processo: o que foi negociado e as sessoes abertas.
#[derive(Debug, Default)]
pub struct Adapter {
    limits: Option<Limits>,
    engine_message: bool,
    sessions: BTreeMap<String, Session>,
    next_session: u64,
}

impl Adapter {
    /// Guarda o efetivo do handshake: o menor de cada lado e o minor comum.
    pub fn negotiate(&mut self, params: &InitializeParams, own: ApiRange, own_limits: Limits) {
        let core = params.limits;
        self.limits = Some(Limits {
            message_bytes: core.message_bytes.min(own_limits.message_bytes),
            in_flight: core.in_flight.min(own_limits.in_flight),
            rows: core.rows.min(own_limits.rows),
            columns: core.columns.min(own_limits.columns),
            cell_bytes: core.cell_bytes.min(own_limits.cell_bytes),
            retained_bytes: core.retained_bytes.min(own_limits.retained_bytes),
            catalogue_items: core.catalogue_items.min(own_limits.catalogue_items),
        });
        // A mensagem do banco so' vai quando os dois falam 1.1.
        self.engine_message =
            params.api.max.major == own.max.major && params.api.max.minor.min(own.max.minor) >= 1;
    }

    /// Executa um pedido e devolve as linhas a escrever (chunks e terminal).
    pub fn run(&mut self, id: &JsonRpcId, request: Request) -> Vec<Value> {
        let Some(limits) = self.limits else {
            return vec![self.failure(
                id,
                FailureReason::IncompatibleApi,
                OperationOutcome::NotStarted,
                None,
            )];
        };
        let context = request.context().clone();
        let mut stream = Stream::new(context.clone(), limits);
        if !matches!(request, Request::Open(_) | Request::Shutdown(_))
            && !context
                .session_id
                .as_ref()
                .is_some_and(|id| self.sessions.contains_key(id))
        {
            return vec![self.failure(
                id,
                FailureReason::ContextChanged,
                OperationOutcome::NotStarted,
                None,
            )];
        }
        let result = match request {
            Request::Open(params) => self.open(
                &params.engine,
                &params.options.fields,
                params.restrictions.read_only,
            ),
            Request::Test(_) => self.test(&context),
            Request::Introspect(_) => self.introspect(&context, &mut stream),
            Request::Query(params) => self.query(&context, &params, &mut stream),
            Request::Impact(params) => Ok(self.impact(&context, &params.text)),
            Request::Cancel(params) => Ok(TerminalResult::Cancel {
                // Os pedidos rodam um por vez: quando o cancelamento chega, o
                // alvo ja' terminou. `SQLite` sem interrupcao nesta fatia.
                target_operation_id: params.target_operation_id,
                accepted: false,
            }),
            Request::Close(_) => {
                if let Some(session) = &context.session_id {
                    self.sessions.remove(session);
                }
                Ok(TerminalResult::Close {})
            }
            Request::Shutdown(_) => {
                self.sessions.clear();
                Ok(TerminalResult::Shutdown {})
            }
            Request::Preview(_) | Request::Decide(_) => Err((
                FailureReason::UnsupportedOperation,
                OperationOutcome::NotStarted,
                None,
            )),
        };
        match result {
            Ok(result) => stream.finish(id, result),
            Err((reason, outcome, message)) => vec![self.failure(id, reason, outcome, message)],
        }
    }

    fn session(&self, context: &Context) -> &Session {
        // `run` ja' conferiu que a sessao existe.
        &self.sessions[context.session_id.as_deref().unwrap_or_default()]
    }

    fn open(
        &mut self,
        engine: &str,
        fields: &BTreeMap<String, OptionValue>,
        read_only: bool,
    ) -> Result<TerminalResult, Refusal> {
        let path = match fields.get("path") {
            Some(OptionValue::Text(path)) if engine == "sqlite" && fields.len() == 1 => {
                path.clone()
            }
            _ => {
                return Err((
                    FailureReason::UnsupportedOperation,
                    OperationOutcome::NotStarted,
                    None,
                ));
            }
        };
        // Abrir prova o arquivo agora, sem criar: a falha vem com a causa.
        let version = sqlite::probe_file(&profile(&path, true)).map_err(|failure| {
            (
                FailureReason::ConnectionFailed,
                OperationOutcome::NotStarted,
                Some(failure.message),
            )
        })?;
        self.next_session += 1;
        let session_id = format!("sqlite-{}", self.next_session);
        self.sessions
            .insert(session_id.clone(), Session { path, read_only });
        Ok(TerminalResult::Open {
            session_id,
            server_version: Some(version),
            operations: crate::OPERATIONS
                .iter()
                .copied()
                .filter(|op| *op != Operation::Open)
                .collect(),
        })
    }

    fn test(&self, context: &Context) -> Result<TerminalResult, Refusal> {
        let session = self.session(context);
        sqlite::probe_file(&profile(&session.path, true))
            .map(|version| TerminalResult::Test {
                server_version: Some(version),
            })
            .map_err(|failure| {
                (
                    FailureReason::ConnectionFailed,
                    OperationOutcome::NotStarted,
                    Some(failure.message),
                )
            })
    }

    fn introspect(
        &self,
        context: &Context,
        stream: &mut Stream,
    ) -> Result<TerminalResult, Refusal> {
        let session = self.session(context);
        let mut schemas =
            sqlite::read_structure(&profile(&session.path, true)).map_err(|failure| {
                (
                    FailureReason::ConnectionFailed,
                    OperationOutcome::NotStarted,
                    Some(failure.message),
                )
            })?;
        // O catalogo externo nao leva instrucoes: o core as gera (40 §3).
        for table in schemas
            .iter_mut()
            .flat_map(|schema| schema.tables.iter_mut())
        {
            table.statements = None;
            table.read_sql = None;
        }
        let truncated = stream.catalogue(schemas);
        Ok(TerminalResult::Introspect { truncated })
    }

    fn query(
        &self,
        context: &Context,
        params: &StatementParams,
        stream: &mut Stream,
    ) -> Result<TerminalResult, Refusal> {
        let session = self.session(context);
        let writes = !query::is_read(&params.text);
        // O adaptador tambem impoe a restricao que recebeu, antes de abrir.
        if writes && session.read_only {
            return Err((FailureReason::ReadOnly, OperationOutcome::NotStarted, None));
        }
        let result = query::run_sqlite(
            &profile(&session.path, session.read_only),
            &params.text,
            params.max_rows,
        )
        .map_err(|failure| {
            (
                FailureReason::ExecutionFailed,
                OperationOutcome::Failed,
                Some(failure.message),
            )
        })?;
        let cut = stream.rows(&result.columns, result.rows)?;
        let statements = classification::classify(DataSourceEngine::Sqlite, &params.text);
        Ok(TerminalResult::Query {
            affected: result.affected,
            truncated: result.truncated || cut,
            elapsed_ms: result.elapsed_ms,
            catalog_update: if classification::invalidates_catalog(
                DataSourceEngine::Sqlite,
                &statements,
            ) {
                DataSourceCatalogUpdate::Reload
            } else {
                DataSourceCatalogUpdate::None
            },
        })
    }

    fn impact(&self, context: &Context, text: &str) -> TerminalResult {
        let session = self.session(context);
        let mut statements = measurement::statements(&profile(&session.path, true), None, text);
        // Diagnostico livre nao viaja nesta API (40 §3): o core explica.
        for statement in &mut statements {
            statement.note = None;
        }
        TerminalResult::Impact {
            severity: impact::overall(&statements),
            statements,
        }
    }

    fn failure(
        &self,
        id: &JsonRpcId,
        reason: FailureReason,
        outcome: OperationOutcome,
        message: Option<String>,
    ) -> Value {
        let error = Error {
            code: -32000,
            message: message.clone().unwrap_or_default(),
            data: Some(Failure {
                reason,
                outcome,
                engine_message: message.filter(|_| self.engine_message),
            }),
        };
        json!({ "jsonrpc": "2.0", "id": id, "error": error })
    }
}

/// O motivo, o desfecho e a mensagem do banco de uma recusa.
pub type Refusal = (FailureReason, OperationOutcome, Option<String>);

/// A versao do `SQLite` embutido, sem abrir arquivo.
#[must_use]
pub fn sqlite_version() -> String {
    sqlite::embedded_version()
}

/// O perfil que as funcoes do motor do core esperam.
fn profile(path: &str, read_only: bool) -> DataSourceProfile {
    DataSourceProfile {
        name: "adaptador".to_owned(),
        engine: DataSourceEngine::Sqlite,
        production: false,
        read_only,
        host: String::new(),
        port: 0,
        database: path.to_owned(),
        user: String::new(),
        secret_source: SecretSource::Automatic,
        secret_variable: None,
        sample_size: None,
        tls: None,
        ca_file: None,
        installation: None,
    }
}
