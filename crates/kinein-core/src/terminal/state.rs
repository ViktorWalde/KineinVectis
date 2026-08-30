//! Estado VT de uma sessão: o grid do emulador e as dimensões do viewport.
//!
//! O emulador é o `alacritty_terminal` (ADR-0004). Este módulo é a ÚNICA porta
//! para ele: quem quiser grid, cursor ou modo VT passa por aqui, e ninguém mais
//! constrói um `Term` na mão.

use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
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
            processor: Processor::new(),
        }
    }

    pub(super) fn process(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
    }
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::vte::ansi::CursorShape;

    use super::TerminalState;

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
