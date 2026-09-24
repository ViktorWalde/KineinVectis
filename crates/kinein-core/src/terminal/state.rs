//! Estado VT de uma sessão: o grid do emulador e as dimensões do viewport.
//!
//! O emulador é o `alacritty_terminal` (ADR-0004). Este módulo é a ÚNICA porta
//! para ele: quem quiser grid, cursor ou modo VT passa por aqui, e ninguém mais
//! constrói um `Term` na mão.

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::cell::LineLength;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::{CursorShape, CursorStyle, Processor};

/// Dimensões do viewport entregues ao emulador.
///
/// O histórico não entra aqui: quem define o scrollback é
/// `Config::scrolling_history`, então `total_lines` do viewport é igual a
/// `screen_lines`.
#[derive(Debug, Clone, Copy)]
pub(super) struct GridSize {
    columns: usize,
    screen_lines: usize,
}

impl GridSize {
    pub(super) const fn new(rows: u16, cols: u16) -> Self {
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

/// Snapshot terminal mantido sob um único lock para grid e cursor não
/// divergirem entre a leitura do PTY e a emissão de um frame.
///
/// O emulador é o `alacritty_terminal` (ADR-0004): a Kinein anuncia
/// `xterm-256color` ao processo, então precisa entregar um VT de verdade —
/// tela alternada, mouse, reflow, wide chars e DECSCUSR inclusos. O PTY continua
/// sendo o `portable-pty`; nada do `tty`/`event_loop` da crate é usado.
pub(super) struct TerminalState {
    pub(super) term: Term<VoidListener>,
    pub(super) selection_id: String,
    processor: Processor,
}

impl TerminalState {
    pub(super) fn new(rows: u16, cols: u16, scrollback: usize) -> Self {
        let config = Config {
            scrolling_history: scrollback,
            default_cursor_style: DEFAULT_CURSOR_STYLE,
            ..Config::default()
        };
        Self {
            term: Term::new(config, &GridSize::new(rows, cols), VoidListener),
            selection_id: String::new(),
            processor: Processor::new(),
        }
    }

    pub(super) fn process(&mut self, bytes: &[u8]) {
        // Conservador: qualquer nova saida invalida o gesto, antes de alterar
        // coordenadas/buffer. Scroll da view nao passa por aqui e o preserva.
        self.clear_selection();
        self.processor.advance(&mut self.term, bytes);
    }

    pub(super) fn clear_selection(&mut self) {
        self.selection_id.clear();
        self.term.selection = None;
    }

    pub(super) fn select_all(&mut self, selection_id: &str) {
        let grid = self.term.grid();
        // Nao copiar as linhas ainda nao utilizadas abaixo do prompt. Conteudo
        // desenhado abaixo do cursor (TUI) tambem faz parte do buffer ativo.
        let last_text = (0..=grid.bottommost_line().0)
            .rev()
            .find(|&line| grid[Line(line)].line_length().0 > 0)
            .unwrap_or(0);
        let last_line = Line(last_text.max(grid.cursor.point.line.0));
        let mut selection = Selection::new(
            SelectionType::Simple,
            Point::new(grid.topmost_line(), Column(0)),
            Side::Left,
        );
        selection.update(Point::new(last_line, grid.last_column()), Side::Right);
        self.term.selection = Some(selection);
        selection_id.clone_into(&mut self.selection_id);
    }

    pub(super) fn copy_selection(&self, selection_id: &str) -> Option<String> {
        if selection_id.is_empty() || selection_id != self.selection_id {
            return None;
        }
        self.term.selection_to_string()
    }
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::grid::Scroll;
    use alacritty_terminal::vte::ansi::CursorShape;

    use super::TerminalState;

    #[test]
    fn select_all_includes_retained_history_independent_of_viewport() {
        let mut state = TerminalState::new(2, 20, 50);
        state.process(b"primeiro\r\nsegundo\r\nterceiro\r\nultimo");
        state.select_all("gesture-1");
        let expected = "primeiro\nsegundo\nterceiro\nultimo";
        assert_eq!(state.copy_selection("gesture-1").as_deref(), Some(expected));
        state.term.scroll_display(Scroll::Top);
        assert_eq!(state.copy_selection("gesture-1").as_deref(), Some(expected));
    }

    #[test]
    fn select_all_uses_native_wrap_and_unicode_extraction() {
        let mut state = TerminalState::new(4, 5, 50);
        state.process("ab界e\u{301}xyz".as_bytes());
        state.select_all("unicode");
        assert_eq!(
            state.copy_selection("unicode").as_deref(),
            Some("ab界e\u{301}xyz")
        );
    }

    #[test]
    fn select_all_new_session_copies_only_prompt_without_unused_screen_rows() {
        let mut state = TerminalState::new(24, 80, 50);
        state.select_all("empty");
        assert_eq!(state.copy_selection("empty").as_deref(), Some(""));
        state.process(b"usuario$");
        state.select_all("prompt");
        assert_eq!(state.copy_selection("prompt").as_deref(), Some("usuario$"));
    }

    #[test]
    fn select_all_does_not_copy_inactive_screen_or_old_selection() {
        let mut state = TerminalState::new(3, 20, 50);
        state.process(b"normal");
        state.select_all("old");
        state.process(b"\x1b[?1049h\x1b[HTUI");
        assert_eq!(state.copy_selection("old"), None);
        state.select_all("alt");
        assert_eq!(state.copy_selection("alt").as_deref(), Some("TUI"));
        state.process(b"\x1b[?1049l");
        assert_eq!(state.copy_selection("alt"), None);
        state.select_all("normal");
        assert_eq!(state.copy_selection("normal").as_deref(), Some("normal"));
    }

    #[test]
    fn new_output_and_new_gesture_invalidate_old_copy() {
        let mut state = TerminalState::new(3, 20, 50);
        state.process(b"antes");
        state.select_all("first");
        state.select_all("second");
        assert_eq!(state.copy_selection("first"), None);
        state.process(b"depois");
        assert_eq!(state.copy_selection("second"), None);
        assert!(state.selection_id.is_empty());
        assert!(state.term.selection.is_none());
    }

    #[test]
    fn select_all_includes_text_below_cursor_and_only_retained_history() {
        let mut state = TerminalState::new(2, 20, 1);
        state.process(b"descartado\r\num\r\ndois\r\ntres\x1b[H");
        state.select_all("bounded");
        assert_eq!(
            state.copy_selection("bounded").as_deref(),
            Some("um\ndois\ntres")
        );
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
}
