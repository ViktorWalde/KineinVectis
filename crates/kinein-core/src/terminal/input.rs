//! Entrada de mouse: o que um gesto SIGNIFICA e como ele vai no fio.
//!
//! Puro de propósito — nada aqui toca PTY, sessão ou grid. É o que permite
//! exercer a regra de roteamento da roda sem abrir um terminal, e é onde mora a
//! decisão que antes estava espalhada na UI.

use alacritty_terminal::term::TermMode;
use kinein_protocol::TerminalMouseModifiers;

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
pub(super) enum WheelAction {
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
pub(super) fn wheel_action(mode: TermMode, shift: bool) -> WheelAction {
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
pub(super) const MOUSE_WHEEL_UP: u8 = 64;
/// Roda para baixo: botão 2 (código 1) somado ao bit de scroll.
pub(super) const MOUSE_WHEEL_DOWN: u8 = 65;
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
pub(super) fn mouse_report(
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
pub(super) fn alt_scroll(lines: i16) -> Vec<u8> {
    let key = if lines > 0 { b'A' } else { b'B' };
    let mut bytes = Vec::with_capacity(lines.unsigned_abs() as usize * 3);
    for _ in 0..lines.unsigned_abs() {
        bytes.extend_from_slice(&[0x1b, b'O', key]);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::term::TermMode;
    use kinein_protocol::TerminalMouseModifiers;

    use super::super::state::TerminalState;
    use super::{
        MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP, WheelAction, alt_scroll, mouse_report, wheel_action,
    };

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
}
