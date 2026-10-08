//! O adaptador `SQLite` REAL (o binario deste crate) pela ponte do passo 8
//! (`DriverProcess`) e com cada stream conferido pelo guardiao do core
//! (`StreamGuard`): quem diz se o adaptador cumpre o contrato e' o consumidor,
//! nao o proprio adaptador (39 §6.2, 9a.1).

#[cfg(test)]
mod adapter {
    use std::{collections::BTreeMap, path::PathBuf, process::Command, time::Duration};

    use kinein_core::datasource::driver_process::{DriverProcess, Event, Kind};
    use kinein_core::datasource::driver_stream::StreamGuard;
    use kinein_protocol::driver::operation::{
        ChunkPayload, Context, ImpactParams, OpenParams, OptionValue, PublicOptions, Request,
        Restrictions, SessionParams, StatementParams, TerminalResult,
    };
    use kinein_protocol::driver::{Error, FailureReason, OperationOutcome, Version};

    const ADAPTER_ID: &str = "kinein.sqlite";

    struct Harness {
        driver: DriverProcess,
        session: Option<String>,
        serial: u32,
    }

    /// O fim de um pedido: os chunks aceitos e o terminal, ou o erro do adaptador.
    struct Outcome {
        chunks: Vec<ChunkPayload>,
        terminal: Result<TerminalResult, Error>,
    }

    fn database(name: &str, rows: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-adapter-sqlite-tests")
            .join(format!("{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("estacao.db");
        let connection = rusqlite_open(&path);
        connection
            .execute_batch(
                "CREATE TABLE leituras (id INTEGER PRIMARY KEY, sensor TEXT, valor REAL);",
            )
            .unwrap();
        for row in rows {
            connection.execute_batch(row).unwrap();
        }
        path
    }

    fn rusqlite_open(path: &std::path::Path) -> rusqlite::Connection {
        rusqlite::Connection::open(path).unwrap()
    }

    impl Harness {
        fn start() -> Self {
            let command = Command::new(env!("CARGO_BIN_EXE_kinein-adapter-sqlite"));
            let driver =
                DriverProcess::start(command, &[], ADAPTER_ID, "sqlite").expect("handshake");
            Self {
                driver,
                session: None,
                serial: 0,
            }
        }

        fn context(&mut self) -> Context {
            self.serial += 1;
            Context {
                instance_id: "instancia".to_owned(),
                session_id: self.session.clone(),
                generation: 1,
                operation_id: format!("op-{}", self.serial),
            }
        }

        fn restrictions(&self, read_only: bool) -> Restrictions {
            Restrictions {
                read_only,
                limits: self.driver.negotiated().limits,
            }
        }

        fn open(&mut self, path: &std::path::Path, read_only: bool) -> Outcome {
            let mut fields = BTreeMap::new();
            fields.insert(
                "path".to_owned(),
                OptionValue::Text(path.display().to_string()),
            );
            let request = Request::Open(OpenParams {
                context: self.context(),
                engine: "sqlite".to_owned(),
                options: PublicOptions {
                    schema_version: 1,
                    fields,
                },
                restrictions: self.restrictions(read_only),
                credential: None,
            });
            let outcome = self.send(&request, Kind::Read);
            if let Ok(TerminalResult::Open { session_id, .. }) = &outcome.terminal {
                self.session = Some(session_id.clone());
            }
            outcome
        }

        fn statement(&mut self, text: &str, max_rows: u32) -> Outcome {
            let request = Request::Query(StatementParams {
                context: self.context(),
                text: text.to_owned(),
                max_rows,
            });
            self.send(&request, Kind::Write)
        }

        /// Envia, recebe ate' o fim e passa cada mensagem pelo guardiao.
        fn send(&self, request: &Request, kind: Kind) -> Outcome {
            let wire = serde_json::to_value(request).unwrap();
            let method = wire["method"].as_str().unwrap().to_owned();
            let mut guard =
                StreamGuard::for_request(request, self.driver.negotiated().limits).unwrap();
            let pending = self
                .driver
                .send(
                    &method,
                    wire["params"].clone(),
                    &request.context().operation_id,
                    kind,
                )
                .expect("enviado");
            let mut chunks = Vec::new();
            loop {
                match pending
                    .events
                    .recv_timeout(Duration::from_secs(20))
                    .expect("evento no prazo")
                {
                    Event::Chunk(raw) => {
                        let chunk = guard
                            .accept_chunk_bytes(raw.get().as_bytes())
                            .expect("chunk aceito pelo guardiao");
                        chunks.push(chunk.payload);
                    }
                    Event::Terminal(raw) => {
                        let terminal = guard
                            .accept_terminal_bytes(raw.get().as_bytes())
                            .expect("terminal aceito pelo guardiao");
                        return Outcome {
                            chunks,
                            terminal: Ok(terminal.result),
                        };
                    }
                    Event::Failed(error) => {
                        guard
                            .finish_failure(request.context())
                            .expect("falha correlacionada");
                        return Outcome {
                            chunks,
                            terminal: Err(error),
                        };
                    }
                    Event::Lost(outcome) => panic!("transporte perdido: {outcome:?}"),
                }
            }
        }

        fn shutdown(mut self) {
            self.session = None;
            let context = serde_json::to_value(self.context()).unwrap();
            let (confirmed, closed) = self.driver.shutdown(&context);
            assert!(confirmed && closed.collected(), "{closed:?}");
        }
    }

    fn rows_of(chunks: &[ChunkPayload]) -> Vec<Vec<Option<String>>> {
        chunks
            .iter()
            .flat_map(|chunk| match chunk {
                ChunkPayload::Rows { rows, .. } => rows.clone(),
                other => panic!("chunk inesperado: {other:?}"),
            })
            .collect()
    }

    fn reason(error: &Error) -> (FailureReason, OperationOutcome, Option<&str>) {
        let data = error.data.as_ref().expect("erro tipado");
        (data.reason, data.outcome, data.engine_message.as_deref())
    }

    #[test]
    fn handshake_open_test_and_catalogue_follow_the_contract() {
        let path = database(
            "catalogo",
            &["INSERT INTO leituras (sensor, valor) VALUES ('bme280', 21.5);"],
        );
        let mut h = Harness::start();
        assert_eq!(h.driver.negotiated().api, Version { major: 1, minor: 1 });
        let opened = h.open(&path, false);
        assert!(
            matches!(&opened.terminal, Ok(TerminalResult::Open { server_version: Some(v), .. }) if v.starts_with("SQLite"))
        );
        let session = h.session.clone().unwrap();
        let test = Request::Test(SessionParams {
            context: h.context(),
        });
        assert!(matches!(
            h.send(&test, Kind::Read).terminal,
            Ok(TerminalResult::Test { .. })
        ));

        let introspect = Request::Introspect(SessionParams {
            context: h.context(),
        });
        let read = h.send(&introspect, Kind::Read);
        assert!(matches!(
            read.terminal,
            Ok(TerminalResult::Introspect { truncated: false })
        ));
        let ChunkPayload::Catalogue {
            schemas,
            collections,
        } = &read.chunks[0]
        else {
            panic!("catalogo esperado");
        };
        assert!(collections.is_empty());
        let table = &schemas[0].tables[0];
        assert_eq!(table.name, "leituras");
        assert_eq!(table.columns.len(), 3);
        // O catalogo externo nao leva instrucoes: o core as gera.
        assert!(table.statements.is_none() && table.read_sql.is_none());

        // Sessao desconhecida: contexto mudou, nada roda.
        h.session = Some("outra".to_owned());
        let stale = Request::Test(SessionParams {
            context: h.context(),
        });
        let refused = h.send(&stale, Kind::Read).terminal.unwrap_err();
        assert_eq!(reason(&refused).0, FailureReason::ContextChanged);
        h.session = Some(session);
        h.shutdown();
    }

    #[test]
    fn reads_keep_null_and_the_ceiling_and_writes_respect_the_restriction() {
        let path = database(
            "leitura",
            &["INSERT INTO leituras (sensor, valor) VALUES ('a', 1), ('b', NULL), ('', 3);"],
        );
        let mut h = Harness::start();
        h.open(&path, false);
        let read = h.statement("SELECT sensor, valor FROM leituras ORDER BY id", 2);
        let Ok(TerminalResult::Query {
            truncated,
            affected,
            ..
        }) = read.terminal
        else {
            panic!("consulta: {:?}", read.terminal.err().map(|e| reason(&e).0));
        };
        assert!(truncated && affected.is_none());
        assert_eq!(
            rows_of(&read.chunks),
            vec![
                vec![Some("a".to_owned()), Some("1".to_owned())],
                vec![Some("b".to_owned()), None]
            ]
        );

        let write = h.statement("UPDATE leituras SET valor = 0 WHERE valor IS NULL", 10);
        assert!(matches!(
            write.terminal,
            Ok(TerminalResult::Query {
                affected: Some(1),
                ..
            })
        ));

        // A mensagem do BANCO chega (1.1), como texto do motor.
        let broken = h.statement("SELECT * FROM nada", 10).terminal.unwrap_err();
        let (why, outcome, message) = reason(&broken);
        assert_eq!(
            (why, outcome),
            (FailureReason::ExecutionFailed, OperationOutcome::Failed)
        );
        assert!(
            message.is_some_and(|text| text.contains("no such table")),
            "{message:?}"
        );
        h.shutdown();

        // Somente leitura: o adaptador recusa a escrita antes de abrir.
        let mut h = Harness::start();
        h.open(&path, true);
        let refused = h
            .statement("DELETE FROM leituras", 10)
            .terminal
            .unwrap_err();
        assert_eq!(reason(&refused).0, FailureReason::ReadOnly);
        assert_eq!(reason(&refused).1, OperationOutcome::NotStarted);
        let count = h.statement("SELECT count(*) FROM leituras", 10);
        assert_eq!(rows_of(&count.chunks), vec![vec![Some("3".to_owned())]]);
        h.shutdown();
    }

    #[test]
    fn large_results_split_into_chunks_and_an_oversized_cell_is_refused_whole() {
        let wide = "x".repeat(900);
        let inserts: Vec<String> = (0..2_500)
            .map(|index| {
                format!("INSERT INTO leituras (sensor, valor) VALUES ('{wide}', {index});")
            })
            .collect();
        let refs: Vec<&str> = inserts.iter().map(String::as_str).collect();
        let path = database("grande", &refs);
        let mut h = Harness::start();
        h.open(&path, false);
        let read = h.statement("SELECT sensor, valor FROM leituras", 2_500);
        assert!(read.chunks.len() >= 2, "{} chunk(s)", read.chunks.len());
        assert_eq!(rows_of(&read.chunks).len(), 2_500);
        assert!(matches!(
            read.terminal,
            Ok(TerminalResult::Query {
                truncated: false,
                ..
            })
        ));

        let huge = h
            .statement(&format!("SELECT '{}'", "y".repeat(20_000)), 10)
            .terminal
            .unwrap_err();
        assert_eq!(reason(&huge).0, FailureReason::LimitExceeded);
        h.shutdown();
    }

    #[test]
    fn impact_is_measured_and_preview_is_not_offered() {
        let path = database(
            "impacto",
            &["INSERT INTO leituras (sensor) VALUES ('a'), ('b'), ('c');"],
        );
        let mut h = Harness::start();
        h.open(&path, false);
        let impact = Request::Impact(ImpactParams {
            context: h.context(),
            text: "DELETE FROM leituras WHERE sensor = 'a'".to_owned(),
        });
        let Ok(TerminalResult::Impact { statements, .. }) = h.send(&impact, Kind::Read).terminal
        else {
            panic!("impacto");
        };
        assert_eq!(statements[0].rows, Some(1));
        assert!(statements[0].note.is_none());
        let preview = Request::Preview(StatementParams {
            context: h.context(),
            text: "DELETE FROM leituras".to_owned(),
            max_rows: 10,
        });
        let refused = h.send(&preview, Kind::Write).terminal.unwrap_err();
        assert_eq!(reason(&refused).0, FailureReason::UnsupportedOperation);
        // Nada foi apagado.
        let count = h.statement("SELECT count(*) FROM leituras", 10);
        assert_eq!(rows_of(&count.chunks), vec![vec![Some("3".to_owned())]]);
        h.shutdown();
    }
}
