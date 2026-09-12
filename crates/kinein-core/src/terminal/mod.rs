//! Terminal profissional: PTY real + emulador VT (grid), fatia D2 (DocsPublic/roadmaps/24).
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
//!
//! ORGANIZAÇÃO (ARCHITECTURE.md §4, regra 4). Este domínio era um arquivo único
//! de 955 linhas de código: cinco responsabilidades sob um lock só. Agora cada
//! uma tem o seu submódulo, e a fronteira entre elas é o que os nomes dizem:
//!
//! ```text
//! session  a SESSAO viva: PTY, threads, ciclo de vida, roteamento de gesto
//! state    o ESTADO VT: o grid do emulador e as dimensoes do viewport
//! render   o CONTRATO de saida: snapshot do grid -> `event.terminal.render`
//! input    a DECISAO de entrada: o que a roda significa, e como se relata
//! error    o VOCABULARIO de erro, que o `rpc.rs` mapeia para JSON-RPC
//! ```

mod error;
mod input;
mod render;
mod session;
mod state;

pub use error::TerminalError;
pub use session::TerminalManager;

/// Canal de eventos do loop principal.
type EventSender = crate::lsp::EventSender;

/// Teto de terminais abertos ao mesmo tempo (cada um é um shell + 3 threads).
///
/// Compartilhada: o `session` a aplica, o `error` a cita na mensagem do limite.
const MAX_SESSIONS: usize = 12;
