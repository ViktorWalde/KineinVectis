//! Terminal profissional: PTY real + emulador VT (grid), fatia D2 (docs/24).
//!
//! Antes o core usava `script` como PTY falso com `TERM=dumb` e removia todo
//! o ANSI (texto puro, sem cor/cursor/TUI). Agora:
//! - PTY REAL via `portable-pty` (do wezterm), `$SHELL` interativo em
//!   `TERM=xterm-256color`, cwd na raiz do workspace;
//! - EMULADOR VT via `vt100`: a saída crua alimenta um `Parser` que mantém o
//!   GRID (células com cor/atributos), cursor e scrollback;
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

use kinein_protocol::JsonRpcRequest;
use portable_pty::{Child, ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde_json::{Value, json};
use vt100::Parser;
use vte::{Params, Perform};

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
/// CSI usado por aplicações para apagar também o histórico do terminal.
const ERASE_SCROLLBACK_SEQUENCE: &[u8] = b"\x1b[3J";

/// Forma de cursor solicitada pela aplicação via DECSCUSR.
///
/// O reset/default da aplicação é resolvido pelo core para a preferência do
/// terminal Kinein (`Bar`); o frontend recebe sempre uma forma concreta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorShape {
    Block,
    Underline,
    Bar,
}

impl CursorShape {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Underline => "underline",
            Self::Bar => "bar",
        }
    }
}

/// Estado visual do cursor que o emulador deve respeitar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CursorStyle {
    shape: CursorShape,
    blinking: bool,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            shape: CursorShape::Bar,
            // A preferência default do terminal Kinein é uma barra pulsante.
            blinking: true,
        }
    }
}

/// Handler estreito do parser VT: só observa `CSI Ps SP q` (DECSCUSR).
struct CursorStyleHandler<'a> {
    style: &'a mut CursorStyle,
}

impl Perform for CursorStyleHandler<'_> {
    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: char) {
        if ignore {
            return;
        }
        if intermediates == b"!" && action == 'p' {
            *self.style = CursorStyle::default();
            return;
        }
        if intermediates != b" " || action != 'q' {
            return;
        }

        let parameter = params
            .iter()
            .next()
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(0);
        let next = match parameter {
            0 => CursorStyle::default(),
            1 => CursorStyle {
                shape: CursorShape::Block,
                blinking: true,
            },
            2 => CursorStyle {
                shape: CursorShape::Block,
                blinking: false,
            },
            3 => CursorStyle {
                shape: CursorShape::Underline,
                blinking: true,
            },
            4 => CursorStyle {
                shape: CursorShape::Underline,
                blinking: false,
            },
            5 => CursorStyle {
                shape: CursorShape::Bar,
                blinking: true,
            },
            6 => CursorStyle {
                shape: CursorShape::Bar,
                blinking: false,
            },
            _ => return,
        };
        *self.style = next;
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        if !ignore && intermediates.is_empty() && byte == b'c' {
            *self.style = CursorStyle::default();
        }
    }
}

/// Estado complementar que `vt100` 0.16 ainda não expõe publicamente.
#[derive(Default)]
struct CursorStyleTracker {
    parser: vte::Parser,
    style: CursorStyle,
}

impl CursorStyleTracker {
    fn process(&mut self, bytes: &[u8]) {
        let mut handler = CursorStyleHandler {
            style: &mut self.style,
        };
        self.parser.advance(&mut handler, bytes);
    }
}

/// Snapshot terminal mantido sob um único lock para grid e cursor não
/// divergirem entre a leitura do PTY e a emissão de um frame.
struct TerminalState {
    parser: Parser,
    cursor_style: CursorStyleTracker,
}

impl TerminalState {
    fn new(rows: u16, cols: u16, scrollback: usize) -> Self {
        Self {
            parser: Parser::new(rows, cols, scrollback),
            cursor_style: CursorStyleTracker::default(),
        }
    }

    fn process(&mut self, bytes: &[u8]) {
        self.cursor_style.process(bytes);
        self.parser.process(bytes);
    }
}

/// Filtro estreito para sessões de AI CLI que prometem transcript navegável.
///
/// Algumas versões do Codex emitem `CSI 3 J` mesmo com `--no-alt-screen`.
/// Um emulador VT correto obedece e apaga o scrollback; dentro do KV Context
/// isso destrói justamente o histórico que o modo inline deveria preservar.
/// O filtro remove somente essa sequência, inclusive quando dividida entre
/// leituras do PTY. O Terminal comum continua honrando `clear` integralmente.
#[derive(Debug, Default)]
struct ScrollbackPreserver {
    pending: Vec<u8>,
}

impl ScrollbackPreserver {
    fn push(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut input = std::mem::take(&mut self.pending);
        input.extend_from_slice(bytes);
        let mut output = Vec::with_capacity(input.len());
        let mut index = 0;

        while index < input.len() {
            let remaining = &input[index..];
            if remaining.starts_with(ERASE_SCROLLBACK_SEQUENCE) {
                index += ERASE_SCROLLBACK_SEQUENCE.len();
            } else if ERASE_SCROLLBACK_SEQUENCE.starts_with(remaining) {
                self.pending.extend_from_slice(remaining);
                break;
            } else {
                output.push(input[index]);
                index += 1;
            }
        }
        output
    }

    fn finish(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pending)
    }
}

/// Error produced by the terminal session manager.
#[derive(Debug)]
pub enum TerminalError {
    /// Too many sessions open at once.
    TooMany,
    /// No session with the given id (or it already died).
    NotOpen,
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
    /// The caller must resolve/allowlist the program. This primitive is shared
    /// by the normal shell and opt-in terminal-backed tools such as AI CLIs.
    pub fn open_command(
        &mut self,
        root: &Path,
        program: &str,
        args: &[String],
    ) -> Result<String, TerminalError> {
        self.open_command_with_policy(root, program, args, false)
    }

    /// Opens an explicit command while preserving the host scrollback from
    /// application-issued erase-history sequences. Used only by AI CLI
    /// surfaces whose transcript must remain navigable.
    pub fn open_command_preserving_scrollback(
        &mut self,
        root: &Path,
        program: &str,
        args: &[String],
    ) -> Result<String, TerminalError> {
        self.open_command_with_policy(root, program, args, true)
    }

    fn open_command_with_policy(
        &mut self,
        root: &Path,
        program: &str,
        args: &[String],
        preserve_scrollback: bool,
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

        self.spawn_reader(&id, reader, &state, &dirty, preserve_scrollback);
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
        preserve_scrollback: bool,
    ) {
        let state = Arc::clone(state);
        let dirty = Arc::clone(dirty);
        let events = self.events.clone();
        let id = id.to_owned();
        thread::spawn(move || {
            let mut buffer = [0_u8; READ_CHUNK_BYTES];
            let mut preserver = preserve_scrollback.then(ScrollbackPreserver::default);
            while let Ok(bytes_read) = reader.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }
                let filtered = preserver
                    .as_mut()
                    .map(|filter| filter.push(&buffer[..bytes_read]));
                let bytes = filtered.as_deref().unwrap_or(&buffer[..bytes_read]);
                if bytes.is_empty() {
                    continue;
                }
                if let Ok(mut state) = state.lock() {
                    state.process(bytes);
                }
                dirty.store(true, Ordering::SeqCst);
            }
            if let Some(filter) = preserver.as_mut() {
                let remaining = filter.finish();
                if !remaining.is_empty() {
                    if let Ok(mut state) = state.lock() {
                        state.process(&remaining);
                    }
                    dirty.store(true, Ordering::SeqCst);
                }
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
        let session = self.sessions.get_mut(id).ok_or(TerminalError::NotOpen)?;
        if !session.running.load(Ordering::SeqCst) {
            return Err(TerminalError::NotOpen);
        }
        session
            .writer
            .write_all(data.as_bytes())
            .and_then(|()| session.writer.flush())
            .map_err(|source| TerminalError::Process {
                message: format!("falha ao escrever no terminal: {source}"),
            })
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
            state.parser.screen_mut().set_size(rows, cols);
        }
        emit_render(&self.events, id, &session.state);
        Ok(())
    }

    /// Rola o histórico (scrollback): `offset` linhas acima do fundo (0 = ao
    /// vivo). O `vt100` já limita ao tamanho real do buffer (D2.2, docs/24).
    pub fn scroll(&mut self, id: &str, offset: u16) -> Result<(), TerminalError> {
        let session = self.live(id)?;
        if let Ok(mut state) = session.state.lock() {
            state.parser.screen_mut().set_scrollback(offset as usize);
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

/// Quantas linhas de histórico existem ACIMA do fundo neste momento.
///
/// O `vt100` expõe o offset ATUAL (`screen().scrollback()`) mas não o total
/// disponível. Como `set_scrollback` clampa ao que realmente existe, pedir o
/// máximo e ler de volta devolve o total; o offset original é restaurado em
/// seguida, então a função não tem efeito observável. É o que permite a UI
/// desenhar uma barra de rolagem proporcional e honesta (0 = não há histórico).
fn scrollback_capacity(parser: &mut Parser) -> usize {
    let current = parser.screen().scrollback();
    parser.screen_mut().set_scrollback(usize::MAX);
    let total = parser.screen().scrollback();
    parser.screen_mut().set_scrollback(current);
    total
}

/// Serializa o grid atual do `parser` e emite `event.terminal.render` para a
/// sessão `id` (D2.3: a UI roteia o render pra aba certa).
fn emit_render(events: &EventSender, id: &str, state: &Arc<Mutex<TerminalState>>) {
    let Ok(mut state) = state.lock() else {
        return;
    };
    let cursor_style = state.cursor_style.style;
    let scrollback = state.parser.screen().scrollback();
    let scrollback_max = scrollback_capacity(&mut state.parser);
    let screen = state.parser.screen();
    let (rows, cols) = screen.size();
    let (cursor_row, cursor_col) = screen.cursor_position();
    let lines: Vec<Value> = (0..rows).map(|row| build_line(screen, row, cols)).collect();

    drop(events.send(JsonRpcRequest::notification(
        "event.terminal.render",
        Some(json!({
            "id": id,
            "cols": cols,
            "rows": rows,
            "cursor": {
                "row": cursor_row,
                "col": cursor_col,
                "visible": !screen.hide_cursor(),
                "shape": cursor_style.shape.as_str(),
                "blinking": cursor_style.blinking,
            },
            // Modos que mudam como um terminal real deve traduzir input.
            // A UI continua burra em relacao ao TUI: apenas respeita o estado
            // VT mantido pelo parser ao enviar teclas e paste.
            "alternateScreen": screen.alternate_screen(),
            "applicationCursor": screen.application_cursor(),
            "bracketedPaste": screen.bracketed_paste(),
            // D2.3/B2: a UI precisa dos dois pra desenhar a barra de rolagem.
            // `scrollback` é a verdade sobre onde a view está (o core clampa o
            // pedido da UI); `scrollbackMax` é quanto histórico existe.
            "scrollback": scrollback,
            "scrollbackMax": scrollback_max,
            "lines": lines,
        })),
    )));
}

/// Estilo de uma célula, comparável para agrupar runs de mesmo estilo.
type CellStyle = (vt100::Color, vt100::Color, bool, bool, bool, bool);

fn cell_style(cell: &vt100::Cell) -> CellStyle {
    (
        cell.fgcolor(),
        cell.bgcolor(),
        cell.bold(),
        cell.italic(),
        cell.underline(),
        cell.inverse(),
    )
}

const DEFAULT_STYLE: CellStyle = (
    vt100::Color::Default,
    vt100::Color::Default,
    false,
    false,
    false,
    false,
);

/// Monta uma linha do grid como array de spans (runs de mesmo estilo).
fn build_line(screen: &vt100::Screen, row: u16, cols: u16) -> Value {
    let mut spans: Vec<Value> = Vec::new();
    let mut run_text = String::new();
    let mut run_cells = 0_u16;
    let mut run_style: Option<CellStyle> = None;
    let mut run_isolated = false;

    for col in 0..cols {
        let (glyph, style, isolated, wide_continuation) = screen.cell(row, col).map_or_else(
            || (" ".to_owned(), DEFAULT_STYLE, false, false),
            |cell| {
                // A segunda célula de um glifo largo ocupa espaço na grade,
                // mas não deve virar outro espaço desenhado. O número de
                // células segue separado do texto para a UI manter o cursor
                // exatamente na coluna VT autoritativa.
                let wide_continuation = cell.is_wide_continuation();
                let glyph = if wide_continuation {
                    String::new()
                } else if cell.has_contents() {
                    cell.contents().to_string()
                } else {
                    " ".to_owned()
                };
                // Runs excepcionais ficam isolados para a UI conseguir
                // converter uma seleção em colunas de volta para texto sem
                // reimplementar Unicode width. A continuação larga se junta
                // somente à sua célula inicial.
                let isolated = cell.is_wide()
                    || wide_continuation
                    || (cell.has_contents() && cell.contents().chars().count() != 1);
                let style = if wide_continuation {
                    run_style.unwrap_or_else(|| cell_style(cell))
                } else {
                    cell_style(cell)
                };
                (glyph, style, isolated, wide_continuation)
            },
        );

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
fn color_value(color: vt100::Color) -> Option<Value> {
    match color {
        vt100::Color::Default => None,
        vt100::Color::Idx(index) => Some(json!(index)),
        vt100::Color::Rgb(red, green, blue) => {
            Some(json!(format!("#{red:02x}{green:02x}{blue:02x}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex, mpsc},
        time::Duration,
    };

    use kinein_protocol::JsonRpcRequest;

    use super::{
        CursorShape, CursorStyleTracker, ScrollbackPreserver, TerminalError, TerminalManager,
        TerminalState, build_line, emit_render,
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

    #[test]
    fn cursor_style_tracker_handles_decscusr_across_chunks() {
        let mut tracker = CursorStyleTracker::default();

        for (parameter, shape, blinking) in [
            (1, CursorShape::Block, true),
            (2, CursorShape::Block, false),
            (3, CursorShape::Underline, true),
            (4, CursorShape::Underline, false),
            (5, CursorShape::Bar, true),
            (6, CursorShape::Bar, false),
        ] {
            let sequence = format!("\x1b[{parameter} q");
            tracker.process(sequence.as_bytes());
            assert_eq!(tracker.style.shape, shape);
            assert_eq!(tracker.style.blinking, blinking);
        }

        tracker.process(b"\x1b[0 q");
        assert_eq!(tracker.style.shape, CursorShape::Bar);
        assert!(tracker.style.blinking);

        tracker.process(b"ordinary text\x1b[6");
        assert_eq!(tracker.style.shape, CursorShape::Bar);
        assert!(tracker.style.blinking);

        tracker.process(b" q");
        assert_eq!(tracker.style.shape, CursorShape::Bar);
        assert!(!tracker.style.blinking);

        tracker.process(b"\x1b[!p");
        assert_eq!(tracker.style.shape, CursorShape::Bar);

        tracker.process(b"\x1b[5 q\x1bc");
        assert_eq!(tracker.style.shape, CursorShape::Bar);
        assert!(tracker.style.blinking);
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

    #[test]
    fn render_span_preserves_authoritative_vt_cell_width() {
        let mut parser = vt100::Parser::new(1, 12, 0);
        parser.process("\x1b[32;1mA界B\x1b[0m".as_bytes());

        let line = build_line(parser.screen(), 0, 12);
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
        assert_eq!(parser.screen().cursor_position(), (0, 4));
    }

    #[test]
    fn ai_scrollback_filter_survives_chunk_boundaries() {
        let mut filter = ScrollbackPreserver::default();
        let mut output = filter.push(b"before\x1b[");
        output.extend(filter.push(b"3Jafter\x1b[2J"));
        output.extend(filter.finish());

        assert_eq!(output, b"beforeafter\x1b[2J");
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
