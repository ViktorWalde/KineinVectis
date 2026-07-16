//! Terminal profissional: PTY real + emulador VT (grid), fatia D2 (docs/roadmaps/24).
//!
//! Antes o core usava `script` como PTY falso com `TERM=dumb` e removia todo
//! o ANSI (texto puro, sem cor/cursor/TUI). Agora:
//! - PTY REAL via `portable-pty` (do wezterm), `$SHELL` interativo em
//!   `TERM=xterm-256color`, cwd na raiz do workspace;
//! - EMULADOR VT via `alacritty_terminal` (ADR-0004): a saída crua alimenta um
//!   `Term` que mantém o GRID (células com cor/atributos), cursor, modos, tela
//!   alternada e scrollback. Como a Kinein anuncia `xterm-256color` ao
//!   processo, o emulador precisa ser de verdade — é o mesmo motor do Alacritty
//!   e do Zed. Nada do `tty`/`event_loop` da crate é usado: o PTY continua
//!   sendo o `portable-pty`;
//! - a UI recebe o grid PRONTO em `event.terminal.render` (linhas de spans
//!   estilizados) e só desenha — sem interpretar ANSI. Suporta prompts com
//!   `\r`, barra de progresso do cargo e TUIs.
//!
//! MÚLTIPLAS sessões (D2.3): cada `terminal.open` cria uma sessão nova com um
//! `id`, e todo comando (`input`/`resize`/`scroll`/`close`) e todo evento
//! (`render`/`closed`) carrega esse `id`. Input é caractere-a-caractere
//! (`terminal.input` com bytes crus), e `terminal.resize` reflui o grid.

use std::{
    collections::HashMap,
    error::Error,
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

use kinein_protocol::{TerminalMouseEvent, TerminalMouseModifiers};

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::{Dimensions, Grid, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, CursorShape, CursorStyle, NamedColor, Processor};
use kinein_protocol::JsonRpcRequest;
use portable_pty::{Child, ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde_json::{Value, json};

/// Linhas de histórico (scrollback) mantidas pelo emulador.
const SCROLLBACK: usize = 5000;
/// Tamanho inicial do grid até a UI mandar o primeiro `resize`.
const DEFAULT_ROWS: u16 = 24;
const DEFAULT_COLS: u16 = 80;
/// Tamanho de cada leitura do PTY, em bytes.
const READ_CHUNK_BYTES: usize = 8192;
/// Cadência máxima de render (~30fps): coalesce rajadas de saída.
const FRAME: Duration = Duration::from_millis(33);
/// Teto de terminais abertos ao mesmo tempo (cada um é um shell + 3 threads).
const MAX_SESSIONS: usize = 12;

/// Dimensões do viewport entregues ao emulador.
///
/// O histórico não entra aqui: quem define o scrollback é
/// `Config::scrolling_history`, então `total_lines` do viewport é igual a
/// `screen_lines`.
#[derive(Debug, Clone, Copy)]
struct GridSize {
    columns: usize,
    screen_lines: usize,
}

impl GridSize {
    const fn new(rows: u16, cols: u16) -> Self {
        Self {
            columns: cols as usize,
            screen_lines: rows as usize,
        }
    }
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.screen_lines
    }

    fn screen_lines(&self) -> usize {
        self.screen_lines
    }

    fn columns(&self) -> usize {
        self.columns
    }
}

/// Preferência de cursor do terminal Kinein quando a aplicação não pede uma.
///
/// O emulador resolve DECSCUSR sozinho e volta a este default no reset, então o
/// frontend recebe sempre uma forma concreta.
const DEFAULT_CURSOR_STYLE: CursorStyle = CursorStyle {
    shape: CursorShape::Beam,
    blinking: true,
};

/// Nome da forma no contrato `event.terminal.render`.
const fn cursor_shape_name(shape: CursorShape) -> &'static str {
    match shape {
        CursorShape::Block | CursorShape::HollowBlock => "block",
        CursorShape::Underline => "underline",
        CursorShape::Beam | CursorShape::Hidden => "bar",
    }
}

/// Snapshot terminal mantido sob um único lock para grid e cursor não
/// divergirem entre a leitura do PTY e a emissão de um frame.
///
/// O emulador é o `alacritty_terminal` (ADR-0004): a Kinein anuncia
/// `xterm-256color` ao processo, então precisa entregar um VT de verdade —
/// tela alternada, mouse, reflow, wide chars e DECSCUSR inclusos. O PTY continua
/// sendo o `portable-pty`; nada do `tty`/`event_loop` da crate é usado.
struct TerminalState {
    term: Term<VoidListener>,
    processor: Processor,
}

impl TerminalState {
    fn new(rows: u16, cols: u16, scrollback: usize) -> Self {
        let config = Config {
            scrolling_history: scrollback,
            default_cursor_style: DEFAULT_CURSOR_STYLE,
            ..Config::default()
        };
        Self {
            term: Term::new(config, &GridSize::new(rows, cols), VoidListener),
            processor: Processor::new(),
        }
    }

    fn process(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
    }
}

/// Error produced by the terminal session manager.
#[derive(Debug)]
pub enum TerminalError {
    /// Too many sessions open at once.
    TooMany,
    /// No session with the given id (or it already died).
    NotOpen,
    /// Gesto de mouse cujo contrato existe mas ainda não tem comportamento
    /// (clique/arrasto/movimento, fatia R5 de `docs/roadmaps/26`).
    MouseUnimplemented,
    /// The session process could not be spawned or reached.
    Process {
        /// Underlying failure description.
        message: String,
    },
}

impl fmt::Display for TerminalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooMany => write!(
                formatter,
                "limite de {MAX_SESSIONS} terminais abertos atingido"
            ),
            Self::NotOpen => write!(formatter, "sessao de terminal inexistente ou encerrada"),
            Self::MouseUnimplemented => write!(
                formatter,
                "gesto de mouse ainda nao implementado: so a roda esta ativa"
            ),
            Self::Process { message } => write!(formatter, "{message}"),
        }
    }
}

impl Error for TerminalError {}

/// Canal de eventos do loop principal.
type EventSender = crate::lsp::EventSender;

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
            state.term.resize(GridSize::new(rows, cols));
        }
        emit_render(&self.events, id, &session.state);
        Ok(())
    }

    /// Rola o histórico (scrollback): `offset` linhas acima do fundo (0 = ao
    /// vivo). O contrato da UI é ABSOLUTO; o emulador trabalha por delta, então
    /// convertemos e deixamos ele clampar ao histórico real, mantendo o eco da
    /// verdade no render (D2.2, docs/roadmaps/24).
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

/// Serializa o grid atual e emite `event.terminal.render` para a sessão `id`
/// (D2.3: a UI roteia o render pra aba certa).
///
/// O emulador já mantém histórico, modos e estilo de cursor; aqui só
/// traduzimos o snapshot para o contrato tipado. `scrollbackMax` vem direto de
/// `history_size()` — o hack de ida e volta que o `vt100` exigia deixou de
/// existir.
fn emit_render(events: &EventSender, id: &str, state: &Arc<Mutex<TerminalState>>) {
    let Ok(state) = state.lock() else {
        return;
    };
    let cursor_style = state.term.cursor_style();
    let mode = *state.term.mode();
    let grid = state.term.grid();
    let rows = grid.screen_lines();
    let cols = grid.columns();
    let scrollback = grid.display_offset();
    let scrollback_max = grid.history_size();

    // A linha do cursor é relativa ao viewport ao vivo; rolar o histórico
    // desloca a visão, então a linha visível é a do cursor mais o offset. Fora
    // da janela o cursor simplesmente não é desenhado.
    let cursor_row = grid.cursor.point.line.0 + i32::try_from(scrollback).unwrap_or(i32::MAX);
    let cursor_col = grid.cursor.point.column.0;
    let cursor_visible = usize::try_from(cursor_row).is_ok_and(|row| row < rows)
        && mode.contains(TermMode::SHOW_CURSOR)
        && cursor_style.shape != CursorShape::Hidden;
    let lines: Vec<Value> = (0..rows)
        .map(|row| build_line(grid, row, cols, scrollback))
        .collect();

    drop(events.send(JsonRpcRequest::notification(
        "event.terminal.render",
        Some(json!({
            "id": id,
            "cols": cols,
            "rows": rows,
            "cursor": {
                "row": cursor_row.max(0),
                "col": cursor_col,
                "visible": cursor_visible,
                "shape": cursor_shape_name(cursor_style.shape),
                "blinking": cursor_style.blinking,
            },
            // Modos que mudam como um terminal real deve traduzir input.
            // A UI continua burra em relacao ao TUI: apenas respeita o estado
            // VT mantido pelo emulador ao enviar teclas e paste.
            "alternateScreen": mode.contains(TermMode::ALT_SCREEN),
            "applicationCursor": mode.contains(TermMode::APP_CURSOR),
            "bracketedPaste": mode.contains(TermMode::BRACKETED_PASTE),
            // D2.3/B2: a UI precisa dos dois pra desenhar a barra de rolagem.
            // `scrollback` é a verdade sobre onde a view está (o emulador clampa
            // o pedido da UI); `scrollbackMax` é quanto histórico existe.
            "scrollback": scrollback,
            "scrollbackMax": scrollback_max,
            "lines": lines,
        })),
    )));
}

// ---------------------------------------------------------------------------
// Codificação de mouse (R4). Os números abaixo não são escolha nossa: são o
// formato de fio que as aplicações TUI esperam. Confirmados verbatim em três
// implementações profissionais independentes, que concordam entre si:
//
//   Zed        `crates/terminal/src/mappings/mouse.rs`  → `|= 64`, MouseFormat
//   xterm.js   `src/common/services/MouseStateService.ts` → `code |= 64`,
//              SHIFT=4/ALT=8/CTRL=16, SGR sem offset, DEFAULT com `+32`
//   JediTerm   `core/.../model/JediTerminal.java#mouseReport` → `\033[<%d;%d;%dM`
//              com `x + 1, y + 1`, e `cb -= 4; cb |= 64`
//
// Armadilha registrada: as constantes do JediTerm (`SCROLLDOWN = 4`,
// `SCROLLUP = 5`) estão MAL NOMEADAS. A aritmética delas converte o botão X11 4
// — que é a roda para CIMA — no código 64. Zed, xterm.js e o ctlseqs do xterm
// concordam que 64 é cima. Seguimos os três; o nome do JediTerm foi descartado.
// ---------------------------------------------------------------------------

/// Destino de um gesto de roda.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum WheelAction {
    /// A aplicação capturou o mouse: relatar o gesto para ela.
    Report,
    /// A aplicação pediu alternate scroll: traduzir em cursor keys.
    AltScroll,
    /// Ninguém capturou: rolar o histórico do próprio terminal.
    ScrollHistory,
}

/// **A regra que estava na UI.** Decide o destino da roda a partir do modo VT.
///
/// Pura de propósito: é o único ponto onde "o que a roda significa" é decidido,
/// e dá para exercê-la sem PTY. A ordem dos ramos vem do `scroll_wheel` do Zed.
///
/// `shift` vence tudo — convenção do xterm para escapar de uma TUI que capturou
/// o mouse e falar com o terminal em vez da aplicação.
fn wheel_action(mode: TermMode, shift: bool) -> WheelAction {
    if shift {
        return WheelAction::ScrollHistory;
    }
    if mode.intersects(TermMode::MOUSE_MODE) {
        return WheelAction::Report;
    }
    if mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) {
        return WheelAction::AltScroll;
    }
    WheelAction::ScrollHistory
}

/// Roda para cima: botão 1 (código 0) somado ao bit de scroll.
const MOUSE_WHEEL_UP: u8 = 64;
/// Roda para baixo: botão 2 (código 1) somado ao bit de scroll.
const MOUSE_WHEEL_DOWN: u8 = 65;
/// Bit de Shift no código do botão.
const MOUSE_MOD_SHIFT: u8 = 4;
/// Bit de Alt/Meta no código do botão.
const MOUSE_MOD_ALT: u8 = 8;
/// Bit de Control no código do botão.
const MOUSE_MOD_CTRL: u8 = 16;
/// Offset de cada campo no formato legado (`ESC [ M`).
const MOUSE_LEGACY_OFFSET: u16 = 32;
/// Maior valor representável num campo do formato legado: ele gasta um byte por
/// campo, então `32 + valor` precisa caber em `u8`.
const MOUSE_LEGACY_MAX: u16 = 255 - MOUSE_LEGACY_OFFSET;

/// Monta o relatório de mouse no formato que a aplicação pediu.
///
/// `col`/`row` chegam 0-based (mesma origem do render) e vão 1-based no fio,
/// como o xterm especifica e o `JediTerm` faz explicitamente (`x + 1, y + 1`).
///
/// Devolve `None` quando o formato legado não consegue representar a
/// coordenada: o xterm.js suprime o evento nesse caso, e é o certo — um campo
/// truncado vira um clique numa célula errada.
fn mouse_report(
    button: u8,
    col: u16,
    row: u16,
    modifiers: TerminalMouseModifiers,
    mode: TermMode,
) -> Option<Vec<u8>> {
    let mut code = button;
    if modifiers.shift {
        code |= MOUSE_MOD_SHIFT;
    }
    if modifiers.alt {
        code |= MOUSE_MOD_ALT;
    }
    if modifiers.ctrl {
        code |= MOUSE_MOD_CTRL;
    }
    let col = col.saturating_add(1);
    let row = row.saturating_add(1);

    if mode.contains(TermMode::SGR_MOUSE) {
        // `ESC [ < Cb ; Cx ; Cy M` — decimal, sem teto de coordenada. `M` é
        // press/motion; `m` seria release, que a roda não emite.
        return Some(format!("\x1b[<{code};{col};{row}M").into_bytes());
    }

    // `ESC [ M (32+Cb) (32+Cx) (32+Cy)` — um byte por campo.
    if col > MOUSE_LEGACY_MAX || row > MOUSE_LEGACY_MAX {
        return None;
    }
    let field = |value: u16| u8::try_from(value + MOUSE_LEGACY_OFFSET).unwrap_or(u8::MAX);
    Some(vec![
        0x1b,
        b'[',
        b'M',
        code.saturating_add(32),
        field(col),
        field(row),
    ])
}

/// Traduz a roda em cursor keys, para TUIs que pedem `?1007` (alternate scroll)
/// mas não capturam o mouse.
///
/// Formato e repetição vindos do `alt_scroll` do Zed: `ESC O A` por linha para
/// cima, `ESC O B` para baixo.
fn alt_scroll(lines: i16) -> Vec<u8> {
    let key = if lines > 0 { b'A' } else { b'B' };
    let mut bytes = Vec::with_capacity(lines.unsigned_abs() as usize * 3);
    for _ in 0..lines.unsigned_abs() {
        bytes.extend_from_slice(&[0x1b, b'O', key]);
    }
    bytes
}

/// Estilo de uma célula, comparável para agrupar runs de mesmo estilo.
type CellStyle = (Color, Color, bool, bool, bool, bool);

const fn cell_style(cell: &Cell) -> CellStyle {
    (
        cell.fg,
        cell.bg,
        cell.flags.contains(Flags::BOLD),
        cell.flags.contains(Flags::ITALIC),
        cell.flags.contains(Flags::UNDERLINE),
        cell.flags.contains(Flags::INVERSE),
    )
}

/// Célula vazia do emulador: cores do tema e nenhum atributo.
const DEFAULT_STYLE: CellStyle = (
    Color::Named(NamedColor::Foreground),
    Color::Named(NamedColor::Background),
    false,
    false,
    false,
    false,
);

/// Monta uma linha VISÍVEL do grid como array de spans (runs de mesmo estilo).
///
/// `display_offset` desloca a janela para o histórico: a linha visível `row`
/// corresponde a `Line(row - display_offset)` no emulador.
fn build_line(grid: &Grid<Cell>, row: usize, cols: usize, display_offset: usize) -> Value {
    let mut spans: Vec<Value> = Vec::new();
    let mut run_text = String::new();
    let mut run_cells = 0_u16;
    let mut run_style: Option<CellStyle> = None;
    let mut run_isolated = false;

    let line = Line(i32::try_from(row).unwrap_or(0) - i32::try_from(display_offset).unwrap_or(0));
    for col in 0..cols {
        let cell = &grid[line][Column(col)];
        // A segunda célula de um glifo largo ocupa espaço na grade, mas não
        // deve virar outro glifo desenhado. O número de células segue separado
        // do texto para a UI manter o cursor exatamente na coluna VT
        // autoritativa.
        let wide_continuation = cell.flags.contains(Flags::WIDE_CHAR_SPACER);
        let glyph = if wide_continuation {
            String::new()
        } else {
            let mut glyph = String::from(cell.c);
            if let Some(zerowidth) = cell.zerowidth() {
                glyph.extend(zerowidth.iter().copied());
            }
            glyph
        };
        // Runs excepcionais ficam isolados para a UI conseguir converter uma
        // seleção em colunas de volta para texto sem reimplementar Unicode
        // width. A continuação larga se junta somente à sua célula inicial.
        let isolated = cell.flags.contains(Flags::WIDE_CHAR)
            || wide_continuation
            || glyph.chars().count() != 1;
        let style = if wide_continuation {
            run_style.unwrap_or_else(|| cell_style(cell))
        } else {
            cell_style(cell)
        };

        let extends_run = run_style == Some(style)
            && ((!run_isolated && !isolated) || (run_isolated && wide_continuation));
        if extends_run {
            run_text.push_str(&glyph);
            run_cells = run_cells.saturating_add(1);
        } else {
            flush_span(&mut spans, &run_text, run_cells, run_style);
            run_text = glyph;
            run_cells = 1;
            run_style = Some(style);
            run_isolated = isolated;
        }
    }
    // Descarta o run final se for só espaços no estilo default (economia).
    if !(run_style == Some(DEFAULT_STYLE) && run_text.trim().is_empty()) {
        flush_span(&mut spans, &run_text, run_cells, run_style);
    }
    Value::Array(spans)
}

fn flush_span(spans: &mut Vec<Value>, text: &str, cells: u16, style: Option<CellStyle>) {
    let Some(style) = style else {
        return;
    };
    if cells == 0 {
        return;
    }
    let (fg, bg, bold, italic, underline, inverse) = style;
    let mut span = serde_json::Map::new();
    span.insert("text".to_owned(), json!(text));
    span.insert("cells".to_owned(), json!(cells));
    if let Some(color) = color_value(fg) {
        span.insert("fg".to_owned(), color);
    }
    if let Some(color) = color_value(bg) {
        span.insert("bg".to_owned(), color);
    }
    for (set, key) in [
        (bold, "bold"),
        (italic, "italic"),
        (underline, "underline"),
        (inverse, "inverse"),
    ] {
        if set {
            span.insert(key.to_owned(), json!(true));
        }
    }
    spans.push(Value::Object(span));
}

/// `None` para a cor default (a UI usa a do tema); índice 0–255 vira número;
/// truecolor vira `#rrggbb`.
fn color_value(color: Color) -> Option<Value> {
    match color {
        Color::Named(named) => named_color_index(named).map(|index| json!(index)),
        Color::Indexed(index) => Some(json!(index)),
        Color::Spec(rgb) => Some(json!(format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b))),
    }
}

/// Índice ANSI de uma cor nomeada, ou `None` quando ela significa "use o tema".
///
/// O emulador separa cores nomeadas (0–15), a paleta 256 e truecolor. Os slots
/// de tema (foreground/background/cursor) não têm índice: viram `None` para a
/// UI aplicar a própria paleta, exatamente como o contrato antigo fazia com a
/// cor default.
const fn named_color_index(named: NamedColor) -> Option<u8> {
    match named {
        // Dim* não tem índice ANSI próprio: cai na cor base e a intensidade
        // fica a cargo do tema.
        NamedColor::Black | NamedColor::DimBlack => Some(0),
        NamedColor::Red | NamedColor::DimRed => Some(1),
        NamedColor::Green | NamedColor::DimGreen => Some(2),
        NamedColor::Yellow | NamedColor::DimYellow => Some(3),
        NamedColor::Blue | NamedColor::DimBlue => Some(4),
        NamedColor::Magenta | NamedColor::DimMagenta => Some(5),
        NamedColor::Cyan | NamedColor::DimCyan => Some(6),
        NamedColor::White | NamedColor::DimWhite => Some(7),
        NamedColor::BrightBlack => Some(8),
        NamedColor::BrightRed => Some(9),
        NamedColor::BrightGreen => Some(10),
        NamedColor::BrightYellow => Some(11),
        NamedColor::BrightBlue => Some(12),
        NamedColor::BrightMagenta => Some(13),
        NamedColor::BrightCyan => Some(14),
        NamedColor::BrightWhite => Some(15),
        NamedColor::Foreground
        | NamedColor::Background
        | NamedColor::Cursor
        | NamedColor::BrightForeground
        | NamedColor::DimForeground => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex, mpsc},
        time::Duration,
    };

    use kinein_protocol::{JsonRpcRequest, TerminalMouseEvent, TerminalMouseModifiers};

    use super::{
        CursorShape, MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP, TermMode, TerminalError, TerminalManager,
        TerminalState, WheelAction, alt_scroll, build_line, emit_render, mouse_report,
        wheel_action,
    };

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-terminal-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    /// Junta o texto de todos os spans de todas as linhas de um render.
    fn render_text(event: &JsonRpcRequest) -> String {
        let mut text = String::new();
        if let Some(lines) = event.params.as_ref().and_then(|p| p["lines"].as_array()) {
            for line in lines {
                if let Some(spans) = line.as_array() {
                    for span in spans {
                        if let Some(chunk) = span["text"].as_str() {
                            text.push_str(chunk);
                        }
                    }
                }
                text.push('\n');
            }
        }
        text
    }

    fn render_contains(receiver: &mpsc::Receiver<JsonRpcRequest>, needle: &str) -> bool {
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(10)) {
            if event.method == "event.terminal.render" && render_text(&event).contains(needle) {
                return true;
            }
            if event.method == "event.terminal.closed" {
                break;
            }
        }
        false
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

    #[test]
    fn render_exposes_input_modes_and_scrollbar_capacity() {
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(TerminalState::new(3, 20, 100)));
        if let Ok(mut state) = state.lock() {
            state.process(b"\x1b[?1h\x1b[?2004h1\r\n2\r\n3\r\n4\r\n5");
        }

        emit_render(&sender, "t1", &state);
        let event = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        let params = event.params.unwrap();

        assert_eq!(params["applicationCursor"], true);
        assert_eq!(params["bracketedPaste"], true);
        assert_eq!(params["alternateScreen"], false);
        assert_eq!(params["cursor"]["shape"], "bar");
        assert_eq!(params["cursor"]["blinking"], true);
        assert!(params["scrollbackMax"].as_u64().is_some_and(|max| max > 0));
    }

    /// DECSCUSR agora é do emulador (ADR-0004); o que a Kinein garante é que a
    /// forma pedida pela TUI chega ao render e que o reset cai na preferência
    /// da casa (barra pulsante), inclusive com a sequência partida entre
    /// leituras do PTY.
    #[test]
    fn decscusr_cursor_style_survives_chunk_boundaries() {
        let mut state = TerminalState::new(2, 10, 0);

        for (parameter, shape, blinking) in [
            (1, CursorShape::Block, true),
            (2, CursorShape::Block, false),
            (3, CursorShape::Underline, true),
            (4, CursorShape::Underline, false),
            (5, CursorShape::Beam, true),
            (6, CursorShape::Beam, false),
        ] {
            let sequence = format!("\x1b[{parameter} q");
            state.process(sequence.as_bytes());
            assert_eq!(state.term.cursor_style().shape, shape);
            assert_eq!(state.term.cursor_style().blinking, blinking);
        }

        state.process(b"\x1b[0 q");
        assert_eq!(state.term.cursor_style().shape, CursorShape::Beam);
        assert!(state.term.cursor_style().blinking);

        state.process(b"ordinary text\x1b[6");
        assert_eq!(state.term.cursor_style().shape, CursorShape::Beam);
        assert!(state.term.cursor_style().blinking);

        state.process(b" q");
        assert_eq!(state.term.cursor_style().shape, CursorShape::Beam);
        assert!(!state.term.cursor_style().blinking);
    }

    #[test]
    fn render_exposes_cursor_style_requested_by_tui() {
        let (sender, receiver) = mpsc::channel();
        let state = Arc::new(Mutex::new(TerminalState::new(2, 10, 0)));
        if let Ok(mut state) = state.lock() {
            // Crossterm `SteadyBar`, usado pelo compositor do Codex.
            state.process(b"\x1b[6 q");
        }

        emit_render(&sender, "t1", &state);
        let event = receiver.recv_timeout(Duration::from_secs(1)).unwrap();
        let cursor = &event.params.as_ref().unwrap()["cursor"];

        assert_eq!(cursor["shape"], "bar");
        assert_eq!(cursor["blinking"], false);
    }

    // ---------------------------------------------------------------------
    // R4 — roda do mouse. As sequências abaixo NÃO são inventadas: foram
    // medidas nas CLIs reais em 2026-07-16, abrindo cada agente num PTY dentro
    // de um workspace confiado e capturando os modos DEC privados que ele liga.
    // Elas são a razão do desenho, então são o que os testes exercem.
    // ---------------------------------------------------------------------

    /// O que o `claude` liga ao abrir: tela alternada + captura total de mouse
    /// com codificação SGR. Note a AUSÊNCIA de `?1007` (alternate scroll) — é
    /// por isso que traduzir a roda em setas não o consertaria.
    const CLAUDE_MODES: &[u8] = b"\x1b[?1049h\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h";
    /// O que o `codex` liga: nem tela alternada, nem mouse. Desenha inline, tem
    /// histórico de verdade na grade — e por isso já rolava.
    const CODEX_MODES: &[u8] = b"\x1b[?2004h\x1b[?1004h";

    fn mode_after(sequence: &[u8]) -> TermMode {
        let mut state = TerminalState::new(24, 80, 100);
        state.process(sequence);
        *state.term.mode()
    }

    /// A regressão que originou a fatia: o Claude captura o mouse, então a roda
    /// tem que ser RELATADA a ele, não virar rolagem local (que não faria nada,
    /// porque tela alternada não tem histórico).
    #[test]
    fn claude_mode_set_routes_wheel_to_the_application() {
        let mode = mode_after(CLAUDE_MODES);
        assert!(mode.contains(TermMode::ALT_SCREEN));
        assert!(mode.intersects(TermMode::MOUSE_MODE));
        assert_eq!(wheel_action(mode, false), WheelAction::Report);
    }

    /// A ORDEM dos ramos é carga estrutural, e este teste existe para fixá-la.
    ///
    /// `ALTERNATE_SCROLL` nasce LIGADO (`TermMode::default()` do
    /// `alacritty_terminal`, igual ao xterm, onde o modo 1007 já vem ativo).
    /// Então o Claude satisfaz as condições dos dois ramos ao mesmo tempo:
    /// tela alternada + alternate scroll E captura de mouse. Se `AltScroll`
    /// fosse avaliado primeiro, ele receberia setas em vez de relatórios e
    /// continuaria sem rolar. Quem captura o mouse tem precedência.
    #[test]
    fn mouse_capture_takes_precedence_over_default_alternate_scroll() {
        let mode = mode_after(CLAUDE_MODES);
        assert!(
            mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL),
            "alternate scroll deveria estar ligado por padrao"
        );
        assert_eq!(
            wheel_action(mode, false),
            WheelAction::Report,
            "captura de mouse tem que vencer o alternate scroll padrao"
        );
    }

    /// O contraste que provou o diagnóstico: o codex não captura nada, então o
    /// ramo local é o certo — e é o que já funcionava.
    #[test]
    fn codex_mode_set_keeps_wheel_on_local_history() {
        let mode = mode_after(CODEX_MODES);
        assert!(!mode.contains(TermMode::ALT_SCREEN));
        assert!(!mode.intersects(TermMode::MOUSE_MODE));
        assert_eq!(wheel_action(mode, false), WheelAction::ScrollHistory);
    }

    /// Shift é a válvula de escape do xterm: fala com o terminal, não com a TUI.
    #[test]
    fn shift_escapes_application_mouse_capture() {
        let mode = mode_after(CLAUDE_MODES);
        assert_eq!(wheel_action(mode, true), WheelAction::ScrollHistory);
    }

    /// TUIs que pedem `?1007` sem capturar mouse (vim, less) esperam cursor keys.
    #[test]
    fn alternate_scroll_tui_receives_cursor_keys() {
        let mode = mode_after(b"\x1b[?1049h\x1b[?1007h");
        assert_eq!(wheel_action(mode, false), WheelAction::AltScroll);
        // Uma tecla por linha, e a direção não pode inverter.
        assert_eq!(alt_scroll(2), b"\x1bOA\x1bOA");
        assert_eq!(alt_scroll(-1), b"\x1bOB");
    }

    /// Formato de fio SGR, com as coordenadas 1-based que o xterm especifica.
    /// 64 = cima e 65 = baixo: Zed, xterm.js e o ctlseqs concordam (as
    /// constantes do `JediTerm` têm o nome trocado e foram descartadas).
    #[test]
    fn wheel_report_uses_sgr_with_one_based_coordinates() {
        let mode = mode_after(CLAUDE_MODES);
        let modifiers = TerminalMouseModifiers::default();

        let up = mouse_report(MOUSE_WHEEL_UP, 9, 4, modifiers, mode).unwrap();
        assert_eq!(up, b"\x1b[<64;10;5M");

        let down = mouse_report(MOUSE_WHEEL_DOWN, 0, 0, modifiers, mode).unwrap();
        assert_eq!(down, b"\x1b[<65;1;1M");
    }

    #[test]
    fn wheel_report_encodes_modifier_bits() {
        let mode = mode_after(CLAUDE_MODES);
        let modifiers = TerminalMouseModifiers {
            shift: false,
            alt: true,
            ctrl: true,
        };
        // 64 | 8 | 16 = 88.
        let report = mouse_report(MOUSE_WHEEL_UP, 0, 0, modifiers, mode).unwrap();
        assert_eq!(report, b"\x1b[<88;1;1M");
    }

    /// Sem `?1006` o formato é o legado de um byte por campo, com `+32`.
    #[test]
    fn wheel_report_falls_back_to_legacy_format_without_sgr() {
        let mode = mode_after(b"\x1b[?1000h");
        let report = mouse_report(
            MOUSE_WHEEL_UP,
            9,
            4,
            TerminalMouseModifiers::default(),
            mode,
        )
        .unwrap();
        assert_eq!(report, &[0x1b, b'[', b'M', 64 + 32, 10 + 32, 5 + 32]);
    }

    /// O formato legado não representa coluna além de 223. O xterm.js suprime o
    /// evento; truncar viraria um clique em outra célula.
    #[test]
    fn legacy_report_is_suppressed_beyond_representable_range() {
        let mode = mode_after(b"\x1b[?1000h");
        let modifiers = TerminalMouseModifiers::default();
        assert!(mouse_report(MOUSE_WHEEL_UP, 500, 0, modifiers, mode).is_none());
        // Com SGR a mesma coordenada passa: é decimal, sem teto.
        let sgr = mode_after(b"\x1b[?1000h\x1b[?1006h");
        assert!(mouse_report(MOUSE_WHEEL_UP, 500, 0, modifiers, sgr).is_some());
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

    #[test]
    fn render_span_preserves_authoritative_vt_cell_width() {
        let mut state = TerminalState::new(1, 12, 0);
        state.process("\x1b[32;1mA界B\x1b[0m".as_bytes());

        let line = build_line(state.term.grid(), 0, 12, 0);
        let spans = line.as_array().expect("a linha deve conter spans");
        let texts = spans
            .iter()
            .map(|span| span["text"].as_str().unwrap_or_default())
            .collect::<String>();
        let cells = spans
            .iter()
            .map(|span| span["cells"].as_u64().unwrap_or_default())
            .collect::<Vec<_>>();

        assert_eq!(texts, "A界B");
        assert_eq!(cells, vec![1, 2, 1]);
        assert!(spans.iter().all(|span| span["fg"] == 2));
        assert!(spans.iter().all(|span| span["bold"] == true));
        assert_eq!(state.term.grid().cursor.point.column.0, 4);
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
