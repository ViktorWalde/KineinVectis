//! O consumidor do adaptador EXTERNO (passo 9a.2, `arquitetura/39` §6.2.1).
//!
//! Um perfil com `installation: { kind: "ide" }` roda teste, catalogo,
//! consulta e impacto pelo processo `kinein-adapter-<motor>` instalado ao lado
//! do `kinein-core` (decisao do autor no 39 §5.2). Este modulo e' o dono da
//! instancia: uma por projeto + perfil inteiro + instalacao, aberta no primeiro
//! uso e encerrada ao desconectar, ao salvar ou remover o perfil e ao trocar de
//! projeto. Toda politica (contexto, somente leitura, confirmacao) ja' rodou no
//! core antes de chegar aqui; as instrucoes do catalogo tambem sao do core.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use kinein_protocol::driver::operation::{
    ChunkPayload, Context, ImpactParams, OpenParams, PublicOptions, Request, Restrictions,
    SessionParams, StatementParams, TerminalResult,
};
use kinein_protocol::driver::{Error, OperationOutcome};
use kinein_protocol::{DataSourceProfile, DataSourceSchema, SqlStatementImpact};
use serde_json::json;

use super::connection::ConnectionFailure;
use super::driver_process::{DriverProcess, Event, Kind};
use super::driver_stream::StreamGuard;
use super::query::QueryResult;

/// Teto da mensagem do banco mostrada na tela (39 §4.4).
const ENGINE_MESSAGE_BYTES: usize = 2048;
/// Quanto se espera pelo fim de uma operacao do adaptador.
const OPERATION_TIMEOUT: Duration = Duration::from_secs(300);

/// As instancias vivas e a pasta onde os adaptadores da IDE moram.
#[derive(Debug, Clone)]
pub struct Adapters {
    dir: Option<PathBuf>,
    live: Arc<Mutex<BTreeMap<Key, Arc<Instance>>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    root: PathBuf,
    name: String,
    profile: String,
}

/// Um adaptador aberto para um perfil: o processo e a sessao.
#[derive(Debug)]
struct Instance {
    driver: DriverProcess,
    id: String,
    session: String,
    serial: AtomicU64,
}

impl Default for Adapters {
    fn default() -> Self {
        Self::beside_core()
    }
}

impl Adapters {
    /// Os adaptadores na pasta dada (os testes apontam para a do build).
    #[must_use]
    pub fn in_dir(dir: PathBuf) -> Self {
        Self {
            dir: Some(dir),
            live: Arc::default(),
        }
    }

    /// Os adaptadores ao lado do executavel do `kinein-core`.
    #[must_use]
    pub fn beside_core() -> Self {
        Self {
            dir: std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf)),
            live: Arc::default(),
        }
    }

    /// Quantas instancias estao abertas (o teste confere o encerramento).
    #[must_use]
    pub fn live(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<Key, Arc<Instance>>> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Encerra as instancias de `name` no projeto (desconectar, salvar,
    /// remover). O encerramento espera o adaptador; roda fora do laco.
    pub fn close(&self, root: &Path, name: &str) {
        let mut closing = Vec::new();
        self.lock().retain(|key, instance| {
            let ours = key.root == root && key.name == name;
            if ours {
                closing.push(Arc::clone(instance));
            }
            !ours
        });
        shut_down(closing);
    }

    /// Encerra todas (troca de projeto).
    pub fn close_all(&self) {
        let closing: Vec<_> = std::mem::take(&mut *self.lock()).into_values().collect();
        shut_down(closing);
    }

    /// A instancia do perfil, aberta agora se ainda nao existe. O `open` (que
    /// espera o handshake) roda SEM a trava; se outro job abriu a mesma
    /// instancia nesse meio tempo, a dele fica e esta e' encerrada.
    fn instance(&self, root: &Path, profile: &DataSourceProfile) -> Result<Arc<Instance>, String> {
        let key = Key {
            root: root.to_path_buf(),
            name: profile.name.clone(),
            profile: serde_json::to_string(profile).unwrap_or_default(),
        };
        if let Some(instance) = self.lock().get(&key) {
            return Ok(Arc::clone(instance));
        }
        // O mesmo perfil com outra configuracao nao reaproveita o processo.
        let mut stale = Vec::new();
        self.lock().retain(|old, instance| {
            let replaced = old.root == key.root && old.name == key.name && *old != key;
            if replaced {
                stale.push(Arc::clone(instance));
            }
            !replaced
        });
        shut_down(stale);
        let opened = Arc::new(self.open(profile)?);
        let kept = Arc::clone(
            self.lock()
                .entry(key)
                .or_insert_with(|| Arc::clone(&opened)),
        );
        if !Arc::ptr_eq(&kept, &opened) {
            shut_down(vec![opened]);
        }
        Ok(kept)
    }

    fn open(&self, profile: &DataSourceProfile) -> Result<Instance, String> {
        let engine = super::profile_format::engine_id(profile.engine);
        let program = self
            .dir
            .as_ref()
            .map(|dir| dir.join(format!("kinein-adapter-{engine}")))
            .filter(|path| path.is_file())
            .ok_or_else(|| {
                format!(
                    "o adaptador da IDE para {engine} nao esta' instalado ao lado do core ({})",
                    self.dir.as_ref().map_or_else(
                        || "pasta do core desconhecida".to_owned(),
                        |dir| dir
                            .join(format!("kinein-adapter-{engine}"))
                            .display()
                            .to_string()
                    )
                )
            })?;
        let driver = DriverProcess::start(
            Command::new(&program),
            &[],
            &format!("kinein.{engine}"),
            &engine,
        )
        .map_err(|failure| format!("o adaptador {} nao iniciou: {failure:?}", program.display()))?;
        let id = format!("{}:{}", profile.name, std::process::id());
        let limits = driver.negotiated().limits;
        let mut instance = Instance {
            driver,
            id,
            session: String::new(),
            serial: AtomicU64::new(0),
        };
        let open = Request::Open(OpenParams {
            context: instance.context(None),
            engine,
            options: PublicOptions {
                schema_version: 1,
                fields: super::profile_format::options_for(profile),
            },
            restrictions: Restrictions {
                read_only: profile.read_only,
                limits,
            },
            credential: None,
        });
        match instance.call(&open, Kind::Read) {
            Ok((_, TerminalResult::Open { session_id, .. })) => {
                instance.session = session_id;
                Ok(instance)
            }
            Ok(_) => Err("o adaptador respondeu o open com outra operacao".to_owned()),
            Err(failure) => Err(failure.message),
        }
    }
}

/// Encerra numa thread: o `shutdown` espera o adaptador sair e ser colhido.
fn shut_down(instances: Vec<Arc<Instance>>) {
    if instances.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        for instance in instances {
            let Ok(instance) = Arc::try_unwrap(instance) else {
                // Ainda em uso por um job: o ultimo dono encerra pelo `Drop`
                // do `OwnedChild` (mata e colhe).
                continue;
            };
            let context = json!(instance.context(None));
            let (_confirmed, _closed) = instance.driver.shutdown(&context);
        }
    });
}

/// Por que uma operacao externa nao terminou com sucesso.
#[derive(Debug)]
struct Failure {
    message: String,
}

impl Instance {
    fn context(&self, session: Option<&str>) -> Context {
        Context {
            instance_id: self.id.clone(),
            session_id: session.map(str::to_owned),
            generation: 1,
            operation_id: format!("op-{}", self.serial.fetch_add(1, Ordering::Relaxed)),
        }
    }

    fn session_context(&self) -> Context {
        self.context(Some(&self.session))
    }

    /// Envia, conferindo cada mensagem pelo guardiao, e devolve os chunks e o
    /// terminal; erro do adaptador vira o texto publico.
    fn call(
        &self,
        request: &Request,
        kind: Kind,
    ) -> Result<(Vec<ChunkPayload>, TerminalResult), Failure> {
        let incompatible = |what: &str| Failure {
            message: format!("o adaptador enviou {what} fora do contrato"),
        };
        let wire = serde_json::to_value(request).map_err(|_| incompatible("um pedido"))?;
        let method = wire["method"].as_str().unwrap_or_default().to_owned();
        let mut guard = StreamGuard::for_request(request, self.driver.negotiated().limits)
            .map_err(|_| incompatible("limites"))?;
        let pending = self
            .driver
            .send(
                &method,
                wire["params"].clone(),
                &request.context().operation_id,
                kind,
            )
            .map_err(|refusal| Failure {
                message: format!("o adaptador nao aceitou o pedido ({refusal:?})"),
            })?;
        let mut chunks = Vec::new();
        loop {
            let event = pending
                .events
                .recv_timeout(OPERATION_TIMEOUT)
                .map_err(|_| Failure {
                    message: "o adaptador nao respondeu no prazo".to_owned(),
                })?;
            match event {
                Event::Chunk(raw) => {
                    let chunk = guard
                        .accept_chunk_bytes(raw.get().as_bytes())
                        .map_err(|_| incompatible("um chunk"))?;
                    chunks.push(chunk.payload);
                }
                Event::Terminal(raw) => {
                    let terminal = guard
                        .accept_terminal_bytes(raw.get().as_bytes())
                        .map_err(|_| incompatible("o terminal"))?;
                    return Ok((chunks, terminal.result));
                }
                Event::Failed(error) => {
                    return Err(Failure {
                        message: public_message(&error),
                    });
                }
                Event::Lost(outcome) => {
                    return Err(Failure {
                        message: match outcome {
                            OperationOutcome::NotStarted => {
                                "o adaptador parou antes da operacao; nada rodou".to_owned()
                            }
                            OperationOutcome::Failed | OperationOutcome::Unknown => {
                                "o adaptador parou no meio da operacao; o resultado e' incerto"
                                    .to_owned()
                            }
                        },
                    });
                }
            }
        }
    }
}

/// A mensagem do BANCO (1.1), limitada e sem controle; sem ela, o texto
/// publico da classificacao.
fn public_message(error: &Error) -> String {
    if let Some(text) = error
        .data
        .as_ref()
        .and_then(|data| data.engine_message.as_deref())
    {
        return bounded(text);
    }
    super::driver_contract::public_error(error).message
}

/// Ate' `ENGINE_MESSAGE_BYTES`, cortado em fronteira de caractere, com
/// controle (menos quebra de linha e tabulacao) trocado por espaco.
#[must_use]
pub fn bounded(text: &str) -> String {
    let clean: String = text
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    if clean.len() <= ENGINE_MESSAGE_BYTES {
        return clean;
    }
    let mut end = ENGINE_MESSAGE_BYTES - '…'.len_utf8();
    while !clean.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &clean[..end])
}

const fn failure(message: String) -> ConnectionFailure {
    ConnectionFailure {
        message,
        sql_state: None,
        secret_required: false,
    }
}

impl Adapters {
    /// `datasource.test` pelo adaptador: a versao do servidor.
    ///
    /// # Errors
    /// O adaptador ausente, que nao abriu, ou a falha do banco.
    pub fn test(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
    ) -> Result<String, ConnectionFailure> {
        let instance = self.instance(root, profile).map_err(failure)?;
        let request = Request::Test(SessionParams {
            context: instance.session_context(),
        });
        match instance.call(&request, Kind::Read) {
            Ok((_, TerminalResult::Test { server_version })) => {
                Ok(server_version.unwrap_or_default())
            }
            Ok(_) => Err(failure("o adaptador respondeu outra operacao".to_owned())),
            Err(error) => Err(failure(error.message)),
        }
    }

    /// `datasource.introspect` pelo adaptador: os esquemas juntados por nome,
    /// com as instrucoes geradas AQUI (o catalogo externo nao as traz).
    ///
    /// # Errors
    /// Os mesmos de [`Adapters::test`].
    pub fn introspect(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
    ) -> Result<Vec<DataSourceSchema>, ConnectionFailure> {
        let instance = self.instance(root, profile).map_err(failure)?;
        let request = Request::Introspect(SessionParams {
            context: instance.session_context(),
        });
        let (chunks, _) = instance
            .call(&request, Kind::Read)
            .map_err(|error| failure(error.message))?;
        let mut schemas: Vec<DataSourceSchema> = Vec::new();
        for chunk in chunks {
            let ChunkPayload::Catalogue { schemas: part, .. } = chunk else {
                continue;
            };
            for schema in part {
                match schemas.iter_mut().find(|known| known.name == schema.name) {
                    Some(known) => known.tables.extend(schema.tables),
                    None => schemas.push(schema),
                }
            }
        }
        super::object_statements::populate(profile.engine, &mut schemas);
        Ok(schemas)
    }

    /// `datasource.query` pelo adaptador, no formato da execucao interna.
    ///
    /// # Errors
    /// Os mesmos de [`Adapters::test`].
    pub fn query(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
        sql: &str,
        max_rows: u32,
    ) -> Result<QueryResult, ConnectionFailure> {
        let instance = self.instance(root, profile).map_err(failure)?;
        let request = Request::Query(StatementParams {
            context: instance.session_context(),
            text: sql.to_owned(),
            max_rows,
        });
        let (chunks, terminal) = instance
            .call(&request, Kind::Write)
            .map_err(|error| failure(error.message))?;
        let TerminalResult::Query {
            affected,
            truncated,
            elapsed_ms,
            ..
        } = terminal
        else {
            return Err(failure("o adaptador respondeu outra operacao".to_owned()));
        };
        let mut result = QueryResult {
            affected,
            truncated,
            elapsed_ms,
            ..QueryResult::default()
        };
        for chunk in chunks {
            if let ChunkPayload::Rows { columns, rows } = chunk {
                result.columns = columns;
                result.rows.extend(rows);
            }
        }
        Ok(result)
    }

    /// O impacto medido pelo adaptador; sem medida, as instrucoes
    /// classificadas sem contagem (o aviso continua pedindo confirmacao).
    #[must_use]
    pub fn impact(
        &self,
        root: &Path,
        profile: &DataSourceProfile,
        sql: &str,
    ) -> Vec<SqlStatementImpact> {
        let measured = self.instance(root, profile).ok().and_then(|instance| {
            let request = Request::Impact(ImpactParams {
                context: instance.session_context(),
                text: sql.to_owned(),
            });
            match instance.call(&request, Kind::Read) {
                Ok((_, TerminalResult::Impact { statements, .. })) => Some(statements),
                _ => None,
            }
        });
        measured.unwrap_or_else(|| super::classification::classify(profile.engine, sql))
    }
}

#[cfg(test)]
mod tests {
    use super::bounded;

    #[test]
    fn the_engine_message_is_bounded_and_plain() {
        assert_eq!(bounded("no such table: nada"), "no such table: nada");
        assert_eq!(
            bounded("linha 1\nlinha\t2\u{1b}[31m"),
            "linha 1\nlinha\t2 [31m"
        );
        let long = "é".repeat(2000);
        let cut = bounded(&long);
        assert!(
            cut.len() <= super::ENGINE_MESSAGE_BYTES && cut.ends_with('…'),
            "{}",
            cut.len()
        );
    }
}
