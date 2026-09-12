//! Terminal/PTY payloads (`terminal.*`).

use serde::{Deserialize, Serialize};

/// Result payload for `terminal.open`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpenResult {
    /// Id da sessão criada (D2.3): todo comando e evento seguinte usa esse id.
    pub id: String,
    /// Shell the session is running (from `$SHELL`).
    pub shell: String,
}

/// Parameters for `terminal.close` (D2.3): qual sessão fechar.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalCloseParams {
    /// Id da sessão.
    pub id: String,
}

/// Parameters for `terminal.input`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalInputParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Raw bytes forwarded to the shell PTY (keys, control chars). The UI
    /// sends each keystroke, not whole lines (D2, DocsPublic/roadmaps/24).
    pub data: String,
}

/// Parameters for `terminal.resize` (D2, DocsPublic/roadmaps/24): new grid size in cells.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalResizeParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Number of columns (cells wide).
    pub cols: u16,
    /// Number of rows (cells tall).
    pub rows: u16,
}

/// Parameters for `terminal.scroll` (D2.2): rows above the live bottom.
///
/// Rolagem EXPLÍCITA do histórico: é o que a barra de rolagem pede. Um gesto de
/// roda não é isto — ele passa por `terminal.mouse`, porque só o core sabe se a
/// aplicação capturou o mouse.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalScrollParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Scrollback offset in rows (0 = live/bottom; clamped to the buffer).
    pub offset: u16,
}

/// Botão de mouse relatável a uma aplicação (R4, `DocsPublic/roadmaps/26`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TerminalMouseButton {
    /// Botão esquerdo (código 0 no relatório).
    Left,
    /// Botão do meio (código 1).
    Middle,
    /// Botão direito (código 2).
    Right,
}

/// Gesto de mouse observado sobre a grade.
///
/// A UI relata o GESTO CRU; **quem decide o que ele significa é o core**, que lê
/// o modo VT mantido pelo emulador e escolhe entre relatar à aplicação, rolar o
/// histórico local ou traduzir em setas. A UI não sabe — e não deve saber — se a
/// aplicação capturou o mouse: isso é estado do terminal, não da tela.
///
/// Só `Wheel` está implementado. As demais variantes fixam o contrato de R5
/// (seleção/clique) para que a superfície não precise mudar de novo; o core
/// as rejeita explicitamente por enquanto, em vez de fingir que funcionam.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum TerminalMouseEvent {
    /// Roda girada `lines` linhas. Positivo = para cima (histórico mais
    /// antigo); negativo = para baixo. Zero é ignorado.
    Wheel {
        /// Linhas do gesto, já convertidas de pixels/ângulo pela UI.
        lines: i16,
    },
    /// Botão pressionado. Contrato fixado; não implementado (R5).
    Press {
        /// Botão apertado.
        button: TerminalMouseButton,
    },
    /// Botão solto. Contrato fixado; não implementado (R5).
    Release {
        /// Botão solto.
        button: TerminalMouseButton,
    },
    /// Ponteiro movido; `button` presente indica arrasto. Contrato fixado; não
    /// implementado (R5).
    Motion {
        /// Botão mantido durante o movimento, quando houver.
        button: Option<TerminalMouseButton>,
    },
}

/// Modificadores válidos no instante do gesto.
///
/// `shift` tem semântica especial e é convenção de terminal (xterm, Alacritty,
/// Zed): força o gesto a valer para o TERMINAL, ignorando a captura de mouse da
/// aplicação. É a válvula de escape para rolar/copiar de dentro de uma TUI.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TerminalMouseModifiers {
    /// Escapa a captura da aplicação e devolve o gesto ao terminal.
    pub shift: bool,
    /// Alt/Meta (bit 8 do relatório).
    pub alt: bool,
    /// Control (bit 16 do relatório).
    pub ctrl: bool,
}

/// Parameters for `terminal.mouse` (R4): um gesto de mouse na grade.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalMouseParams {
    /// Id da sessão (D2.3).
    pub id: String,
    /// Coluna da célula sob o ponteiro, 0-based na grade visível — a mesma
    /// origem que `event.terminal.render` usa para o cursor. O core converte
    /// para 1-based ao montar o relatório, como o xterm especifica.
    pub col: u16,
    /// Linha da célula sob o ponteiro, 0-based na grade visível.
    pub row: u16,
    /// O gesto observado pela UI.
    pub event: TerminalMouseEvent,
    /// Modificadores no instante do gesto.
    #[serde(default)]
    pub modifiers: TerminalMouseModifiers,
}
