//! Contrato de saída: traduz o snapshot do grid em `event.terminal.render`.
//!
//! Este é o arquivo que os nomes de campo lidos pelo QML travam. O teste de
//! contrato vive em `tests/terminal.rs`
//! (`render_event_keeps_every_field_the_ui_reads`); mudar uma chave aqui derruba
//! ele de propósito.

use std::sync::{Arc, Mutex};

use alacritty_terminal::grid::{Dimensions, Grid};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::TermMode;
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, CursorShape, NamedColor};
use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::EventSender;
use super::state::TerminalState;

/// Nome da forma no contrato `event.terminal.render`.
const fn cursor_shape_name(shape: CursorShape) -> &'static str {
    match shape {
        CursorShape::Block | CursorShape::HollowBlock => "block",
        CursorShape::Underline => "underline",
        CursorShape::Beam | CursorShape::Hidden => "bar",
    }
}

/// Serializa o grid atual e emite `event.terminal.render` para a sessão `id`
/// (D2.3: a UI roteia o render pra aba certa).
///
/// O emulador já mantém histórico, modos e estilo de cursor; aqui só
/// traduzimos o snapshot para o contrato tipado. `scrollbackMax` vem direto de
/// `history_size()` — o hack de ida e volta que o `vt100` exigia deixou de
/// existir.
pub(super) fn emit_render(events: &EventSender, id: &str, state: &Arc<Mutex<TerminalState>>) {
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
pub(in crate::terminal) mod tests {
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    use kinein_protocol::JsonRpcRequest;

    use super::super::state::TerminalState;
    use super::{build_line, emit_render};

    /// Junta o texto de todos os spans de todas as linhas de um render.
    pub(in crate::terminal) fn render_text(event: &JsonRpcRequest) -> String {
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

    pub(in crate::terminal) fn render_contains(
        receiver: &mpsc::Receiver<JsonRpcRequest>,
        needle: &str,
    ) -> bool {
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
}
