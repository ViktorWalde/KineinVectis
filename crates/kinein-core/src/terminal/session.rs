//! Sessão de shell viva: PTY real, threads de ciclo de vida e o teto de abertas.
//!
//! Responsabilidade única: manter processos de shell vivos e alcançáveis por
//! `id`. O que a saída VIRA (render) e o que um gesto SIGNIFICA (input) são de
//! outros módulos — aqui só se decide para quem o byte vai.

use std::{
    collections::HashMap,
    fmt,
    io::{Read, Write},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use alacritty_terminal::grid::Scroll;
use kinein_protocol::{JsonRpcRequest, TerminalMouseEvent, TerminalMouseModifiers};
use portable_pty::{Child, ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde_json::json;

use super::error::TerminalError;
use super::input::{
    MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP, WheelAction, alt_scroll, mouse_report, wheel_action,
};
use super::render::emit_render;
use super::state::{GridSize, TerminalState};
use super::{EventSender, MAX_SESSIONS};

mod selection;

/// Linhas de histórico (scrollback) mantidas pelo emulador.
const SCROLLBACK: usize = 5000;
/// Tamanho inicial do grid até a UI mandar o primeiro `resize`.
const DEFAULT_ROWS: u16 = 24;
const DEFAULT_COLS: u16 = 80;
/// Tamanho de cada leitura do PTY, em bytes.
const READ_CHUNK_BYTES: usize = 8192;
/// Cadência máxima de render (~30fps): coalesce rajadas de saída.
const FRAME: Duration = Duration::from_millis(33);

/// Sessão de shell viva: PTY, emulador e handles de escrita/controle.
struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    state: Arc<Mutex<TerminalState>>,
    // `wait()` bloqueia numa thread dedicada. O killer clonado pelo
    // `portable-pty` permite encerrar a sessão sem disputar um mutex com essa
    // thread (o desenho anterior deadlockava ao fechar uma aba).
    killer: Box<dyn ChildKiller + Send + Sync>,
    running: Arc<AtomicBool>,
}

/// Owns the shell sessions a workspace keeps open (multi-terminal, D2.3).
pub struct TerminalManager {
    events: EventSender,
    sessions: HashMap<String, Session>,
    /// Contador monotônico dos ids ("t1", "t2", …). Nunca reusa id de sessão
    /// morta: um render atrasado de uma sessão fechada não pode ser confundido
    /// com uma nova.
    next_id: u64,
}

impl fmt::Debug for TerminalManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TerminalManager")
            .field("sessions", &self.sessions.len())
            .finish_non_exhaustive()
    }
}

impl TerminalManager {
    /// Creates a manager that pushes `event.terminal.*` through `events`.
    #[must_use]
    pub fn new(events: EventSender) -> Self {
        Self {
            events,
            sessions: HashMap::new(),
            next_id: 0,
        }
    }

    /// `true` enquanto a sessão `id` está viva (o waiter zera ao shell sair).
    #[must_use]
    pub fn is_open(&self, id: &str) -> bool {
        self.sessions
            .get(id)
            .is_some_and(|session| session.running.load(Ordering::SeqCst))
    }

    /// `true` se existe qualquer sessão viva.
    #[must_use]
    pub fn any_open(&self) -> bool {
        self.sessions
            .values()
            .any(|session| session.running.load(Ordering::SeqCst))
    }

    /// Sessão viva pelo id, ou `NotOpen`.
    fn live(&self, id: &str) -> Result<&Session, TerminalError> {
        let session = self.sessions.get(id).ok_or(TerminalError::NotOpen)?;
        if session.running.load(Ordering::SeqCst) {
            Ok(session)
        } else {
            Err(TerminalError::NotOpen)
        }
    }

    /// Opens the user's shell (from `$SHELL`) inside a real PTY at `root`.
    /// Returns `(id, shell)` — o `id` identifica a sessão nos comandos e
    /// eventos seguintes (D2.3).
    pub fn open(&mut self, root: &Path) -> Result<(String, String), TerminalError> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_absent| "/bin/bash".to_owned());
        let id = self.open_with_shell(root, &shell)?;
        Ok((id, shell))
    }

    /// Opens `shell` inside a real PTY at `root`. Split out for tests.
    /// Devolve o `id` da sessão criada.
    pub fn open_with_shell(&mut self, root: &Path, shell: &str) -> Result<String, TerminalError> {
        self.open_command(root, shell, &[])
    }

    /// Opens an explicit executable and arguments inside a real PTY at `root`.
    ///
    /// O chamador resolve/allowlista o programa. Não existe política por
    /// programa: um agente de CLI aberto aqui recebe exatamente o mesmo
    /// tratamento do shell — é o que faz rodar `claude` na IDE ser igual a
    /// rodar fora dela.
    pub fn open_command(
        &mut self,
        root: &Path,
        program: &str,
        args: &[String],
    ) -> Result<String, TerminalError> {
        // Sessões mortas (shell saiu) não contam pro teto nem seguram memória.
        self.sessions
            .retain(|_id, session| session.running.load(Ordering::SeqCst));
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(TerminalError::TooMany);
        }
        self.next_id += 1;
        let id = format!("t{}", self.next_id);

        let process = |message: String| TerminalError::Process { message };
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: DEFAULT_ROWS,
                cols: DEFAULT_COLS,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| process(format!("falha ao abrir o PTY: {error}")))?;

        let mut command = CommandBuilder::new(program);
        command.args(args);
        command.cwd(root);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("TERM_PROGRAM", "KineinVectis");
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| process(format!("falha ao abrir o processo ({program}): {error}")))?;
        // O slave é do processo filho; soltamos a nossa ponta.
        drop(pair.slave);

        let writer = pair
            .master
            .take_writer()
            .map_err(|error| process(format!("stdin do terminal indisponivel: {error}")))?;
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| process(format!("stdout do terminal indisponivel: {error}")))?;

        let state = Arc::new(Mutex::new(TerminalState::new(
            DEFAULT_ROWS,
            DEFAULT_COLS,
            SCROLLBACK,
        )));
        let running = Arc::new(AtomicBool::new(true));
        let killer = child.clone_killer();
        let dirty = Arc::new(AtomicBool::new(false));

        self.spawn_reader(&id, reader, &state, &dirty);
        self.spawn_emitter(&id, &state, &dirty, &running);
        self.spawn_waiter(&id, child, &running);

        self.sessions.insert(
            id.clone(),
            Session {
                master: pair.master,
                writer,
                state,
                killer,
                running,
            },
        );
        Ok(id)
    }

    /// Thread leitora: bytes crus do PTY → `parser.process` → marca sujo.
    fn spawn_reader(
        &self,
        id: &str,
        mut reader: Box<dyn Read + Send>,
        state: &Arc<Mutex<TerminalState>>,
        dirty: &Arc<AtomicBool>,
    ) {
        let state = Arc::clone(state);
        let dirty = Arc::clone(dirty);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            let mut buffer = [0_u8; READ_CHUNK_BYTES];
            while let Ok(bytes_read) = reader.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                let bytes = &buffer[..bytes_read];
                if let Ok(mut state) = state.lock() {
                    state.process(bytes);
                }
                dirty.store(true, Ordering::SeqCst);
            }
            // EOF: garante o render do estado final antes do `closed`.
            emit_render(&events, &id, &state);
        });
    }

    /// Thread emissora: a cada FRAME, se sujo, manda o grid pra UI (throttle).
    fn spawn_emitter(
        &self,
        id: &str,
        state: &Arc<Mutex<TerminalState>>,
        dirty: &Arc<AtomicBool>,
        running: &Arc<AtomicBool>,
    ) {
        let state = Arc::clone(state);
        let dirty = Arc::clone(dirty);
        let running = Arc::clone(running);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                thread::sleep(FRAME);
                if dirty.swap(false, Ordering::SeqCst) {
                    emit_render(&events, &id, &state);
                }
            }
        });
    }

    /// Thread waiter: espera o shell sair, zera `running` e emite `closed`.
    fn spawn_waiter(
        &self,
        id: &str,
        mut child: Box<dyn Child + Send + Sync>,
        running: &Arc<AtomicBool>,
    ) {
        let running = Arc::clone(running);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            let code = child
                .wait()
                .ok()
                .and_then(|status| i32::try_from(status.exit_code()).ok());
            running.store(false, Ordering::SeqCst);
            drop(events.send(JsonRpcRequest::notification(
                "event.terminal.closed",
                Some(json!({ "id": id, "exitCode": code })),
            )));
        });
    }

    /// Forwards raw `data` (bytes/teclas) to the shell PTY da sessão `id`.
    pub fn write(&mut self, id: &str, data: &str) -> Result<(), TerminalError> {
        self.write_bytes(id, data.as_bytes())
    }

    /// Escreve bytes crus no PTY da sessão `id`.
    ///
    /// Existe separado de `write` porque o relatório de mouse legado não é
    /// UTF-8: ele codifica cada campo como um único byte `32 + valor`, que pode
    /// cair na faixa 128–255 e não formar um `char` válido. O `JediTerm` resolve
    /// o mesmo problema escolhendo ISO-8859-1 para esse formato; aqui a
    /// codificação simplesmente não se aplica, porque o contrato do PTY é byte.
    fn write_bytes(&mut self, id: &str, bytes: &[u8]) -> Result<(), TerminalError> {
        let session = self.sessions.get_mut(id).ok_or(TerminalError::NotOpen)?;
        if !session.running.load(Ordering::SeqCst) {
            return Err(TerminalError::NotOpen);
        }
        session
            .writer
            .write_all(bytes)
            .and_then(|()| session.writer.flush())
            .map_err(|source| TerminalError::Process {
                message: format!("falha ao escrever no terminal: {source}"),
            })
    }

    /// Aplica um gesto de mouse na sessão `id`.
    ///
    /// **Esta função é a regra de negócio que faltava.** Até aqui a UI decidia
    /// sozinha que roda = rolar histórico, sempre — e por isso o Claude não
    /// rolava: ele desenha em tela alternada, que não tem histórico por
    /// definição VT, então o pedido caía no vazio. Quem sabe o que um gesto
    /// significa é o terminal, porque só ele conhece o modo que a aplicação
    /// ligou.
    pub fn mouse(
        &mut self,
        id: &str,
        col: u16,
        row: u16,
        event: TerminalMouseEvent,
        modifiers: TerminalMouseModifiers,
    ) -> Result<(), TerminalError> {
        match event {
            TerminalMouseEvent::Wheel { lines } => self.wheel(id, col, row, lines, modifiers),
            // Contrato fixado, comportamento não implementado (R5). Um erro
            // explícito é honesto; engolir o gesto em silêncio faria a UI
            // parecer quebrada sem deixar rastro.
            TerminalMouseEvent::Press { .. }
            | TerminalMouseEvent::Release { .. }
            | TerminalMouseEvent::Motion { .. } => Err(TerminalError::MouseUnimplemented),
        }
    }

    /// Roda do mouse: escolhe o destino do gesto pelo modo VT da sessão.
    ///
    /// Os três ramos e a ordem entre eles vêm do `scroll_wheel` do Zed
    /// (`crates/terminal/src/terminal.rs`), referência MODE-D — estudada, não
    /// copiada. `shift` desvia para o histórico local em qualquer caso: é a
    /// válvula de escape padrão do xterm para sair de uma TUI que capturou o
    /// mouse.
    ///
    /// Medido nas CLIs reais em 2026-07-16 (sonda de modos DEC privados, com o
    /// agente aberto num workspace confiado):
    ///
    /// ```text
    /// claude: ?1049h ?1000h ?1002h ?1003h ?1006h  → ramo 1 (relatório SGR)
    /// codex : nenhum destes                       → ramo 3 (histórico local)
    /// ```
    ///
    /// Isso decidiu o desenho: o Claude precisa de RELATÓRIO. Um conserto que
    /// só traduzisse a roda em setas seria inerte para ele.
    ///
    /// A ordem entre os ramos importa: `ALTERNATE_SCROLL` nasce ligado
    /// (`TermMode::default()`), então o Claude satisfaz o ramo 2 também.
    /// Capturar o mouse tem precedência — ver
    /// `mouse_capture_takes_precedence_over_default_alternate_scroll`.
    fn wheel(
        &mut self,
        id: &str,
        col: u16,
        row: u16,
        lines: i16,
        modifiers: TerminalMouseModifiers,
    ) -> Result<(), TerminalError> {
        if lines == 0 {
            return Ok(());
        }
        let session = self.live(id)?;
        let Ok(mode) = session.state.lock().map(|state| *state.term.mode()) else {
            return Err(TerminalError::NotOpen);
        };

        match wheel_action(mode, modifiers.shift) {
            WheelAction::Report => {
                let button = if lines > 0 {
                    MOUSE_WHEEL_UP
                } else {
                    MOUSE_WHEEL_DOWN
                };
                let Some(report) = mouse_report(button, col, row, modifiers, mode) else {
                    // Coordenada fora do alcance do formato legado. O xterm.js
                    // suprime o evento nesse caso; um campo truncado viraria um
                    // clique numa célula errada.
                    return Ok(());
                };
                // Um relatório por linha do gesto, como o `scroll_report` do Zed.
                let mut bytes = Vec::with_capacity(report.len() * lines.unsigned_abs() as usize);
                for _ in 0..lines.unsigned_abs() {
                    bytes.extend_from_slice(&report);
                }
                self.write_bytes(id, &bytes)
            }
            WheelAction::AltScroll => self.write_bytes(id, &alt_scroll(lines)),
            WheelAction::ScrollHistory => {
                if let Ok(mut state) = session.state.lock() {
                    state.term.scroll_display(Scroll::Delta(i32::from(lines)));
                }
                emit_render(&self.events, id, &session.state);
                Ok(())
            }
        }
    }

    /// Reflui o grid e o PTY da sessão `id` para um novo tamanho (cols × rows).
    pub fn resize(&mut self, id: &str, cols: u16, rows: u16) -> Result<(), TerminalError> {
        let session = self.live(id)?;
        if cols == 0 || rows == 0 {
            return Err(TerminalError::NotOpen);
        }
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TerminalError::Process {
                message: format!("falha ao redimensionar o terminal: {error}"),
            })?;
        if let Ok(mut state) = session.state.lock() {
            state.clear_selection();
            state.term.resize(GridSize::new(rows, cols));
        }
        emit_render(&self.events, id, &session.state);
        Ok(())
    }

    /// Rola o histórico (scrollback): `offset` linhas acima do fundo (0 = ao
    /// vivo). O contrato da UI é ABSOLUTO; o emulador trabalha por delta, então
    /// convertemos e deixamos ele clampar ao histórico real, mantendo o eco da
    /// verdade no render (D2.2, DocsPublic/roadmaps/24).
    pub fn scroll(&mut self, id: &str, offset: u16) -> Result<(), TerminalError> {
        let session = self.live(id)?;
        if let Ok(mut state) = session.state.lock() {
            let current = i32::try_from(state.term.grid().display_offset()).unwrap_or(i32::MAX);
            let delta = i32::from(offset) - current;
            if delta != 0 {
                state.term.scroll_display(Scroll::Delta(delta));
            }
        }
        emit_render(&self.events, id, &session.state);
        Ok(())
    }

    /// Descarta o histórico real da sessão sem escrever nada no PTY.
    pub fn clear_scrollback(&mut self, id: &str) -> Result<(), TerminalError> {
        let session = self.live(id)?;
        {
            let mut state = session.state.lock().map_err(|_| TerminalError::Process {
                message: "estado do terminal indisponivel para limpar historico".to_owned(),
            })?;
            state.clear_selection();
            state.term.grid_mut().clear_history();
        }
        emit_render(&self.events, id, &session.state);
        Ok(())
    }

    /// Kills the shell session `id`. The waiter thread reports `closed`.
    pub fn close(&mut self, id: &str) -> Result<(), TerminalError> {
        let mut session = self.sessions.remove(id).ok_or(TerminalError::NotOpen)?;
        session.running.store(false, Ordering::SeqCst);
        session
            .killer
            .kill()
            .map_err(|source| TerminalError::Process {
                message: format!("falha ao fechar o terminal: {source}"),
            })
    }

    /// Mata TODAS as sessões (fechar workspace, shutdown do core).
    pub fn close_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            drop(self.close(&id));
        }
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        // Não deixa shell órfão no shutdown do core.
        self.close_all();
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    use kinein_protocol::{TerminalMouseEvent, TerminalMouseModifiers};

    // O decodificador do evento de render e' do modulo que o EMITE: um teste de
    // sessao afirma que a saida apareceu, sem conhecer a forma do contrato.
    use super::super::render::tests::render_contains;
    use super::{TerminalError, TerminalManager};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-terminal-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn pty_session_renders_command_output_in_grid() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        let root = temp_root("render");

        let Ok(id) = manager.open_with_shell(&root, "sh") else {
            // Sem PTY neste ambiente (ex.: sandbox de CI).
            return;
        };
        assert!(manager.is_open(&id));

        manager.write(&id, "echo render-ok $((3+4))\n").unwrap();
        assert!(
            render_contains(&receiver, "render-ok 7"),
            "a saida do shell nao apareceu no grid"
        );

        manager.write(&id, "exit\n").unwrap();
        let mut closed = false;
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
            if event.method == "event.terminal.closed" {
                closed = true;
                break;
            }
        }
        assert!(closed);
        // O waiter zera `running` ao sair: is_open vira false.
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while manager.is_open(&id) && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!manager.is_open(&id));
        assert!(matches!(
            manager.write(&id, "x\n"),
            Err(TerminalError::NotOpen)
        ));
    }

    #[test]
    fn explicit_command_uses_same_pty_renderer() {
        let (sender, receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        let root = temp_root("explicit-command");
        let args = vec!["-c".to_owned(), "printf ai-bridge-ok; sleep 0.2".to_owned()];

        let Ok(_id) = manager.open_command(&root, "sh", &args) else {
            return;
        };
        assert!(
            render_contains(&receiver, "ai-bridge-ok"),
            "o comando explicito nao apareceu no renderer compartilhado"
        );
    }

    /// Contrato fixado para R5, comportamento ainda não: recusa explícita.
    #[test]
    fn unimplemented_mouse_gestures_are_rejected_explicitly() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        let root = temp_root("mouse-unimplemented");
        let Ok(id) = manager.open_with_shell(&root, "sh") else {
            return;
        };

        let error = manager.mouse(
            &id,
            0,
            0,
            TerminalMouseEvent::Press {
                button: kinein_protocol::TerminalMouseButton::Left,
            },
            TerminalMouseModifiers::default(),
        );
        assert!(matches!(error, Err(TerminalError::MouseUnimplemented)));

        // Roda com zero linha é no-op, não erro.
        assert!(
            manager
                .mouse(
                    &id,
                    0,
                    0,
                    TerminalMouseEvent::Wheel { lines: 0 },
                    TerminalMouseModifiers::default(),
                )
                .is_ok()
        );
        manager.close(&id).ok();
    }

    /// D2.3: o ponto da fatia — duas sessões vivas ao mesmo tempo, com ids
    /// distintos, e fechar uma NÃO derruba a outra.
    #[test]
    fn two_sessions_coexist_and_close_independently() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        let root = temp_root("multi");

        let Ok(first) = manager.open_with_shell(&root, "sh") else {
            return; // sem PTY no ambiente
        };
        let second = manager
            .open_with_shell(&root, "sh")
            .expect("a segunda sessao deve abrir (multi-terminal)");

        assert_ne!(first, second, "cada sessao precisa de um id proprio");
        assert!(manager.is_open(&first));
        assert!(manager.is_open(&second));

        manager.close(&first).unwrap();
        assert!(!manager.is_open(&first));
        assert!(
            manager.is_open(&second),
            "fechar um terminal nao pode matar o outro"
        );
        assert!(manager.any_open());

        manager.close(&second).unwrap();
        assert!(!manager.any_open());
    }

    #[test]
    fn commands_on_unknown_session_error() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = TerminalManager::new(sender);
        assert!(matches!(
            manager.resize("t404", 120, 40),
            Err(TerminalError::NotOpen)
        ));
        assert!(matches!(manager.close("t404"), Err(TerminalError::NotOpen)));
    }
}
