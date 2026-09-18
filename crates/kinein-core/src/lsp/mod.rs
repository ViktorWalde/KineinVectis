//! Subsistema LSP: clangd, rust-analyzer e futuros servidores.
//!
//! O core sobe cada language server como processo filho, faz o handshake
//! `initialize`/`initialized`, sincroniza documentos (didOpen/didChange/
//! didSave com texto completo) e converte `textDocument/publishDiagnostics`
//! em notificacoes `event.lsp.diagnostics` do protocolo Kinein Vectis.
//!
//! A UI nunca fala LSP diretamente; ela envia `lsp.didChange` pelo IPC e
//! recebe diagnosticos ja traduzidos.
//!
//! Organizacao interna:
//! - [`types`]: DTOs e o erro estruturado que cruzam a fronteira do modulo;
//! - [`manager`]: estado da sessao e as operacoes interativas;
//! - [`registry`]: servidores principais e companheiros por linguagem;
//! - [`server`]: ciclo de vida do processo, handshake e thread leitora;
//! - [`session`]: qual executavel, subir/reiniciar/encerrar e o transporte;
//! - [`sync`]: o que o servidor sabe sobre o TEXTO (didOpen/didChange/didSave);
//! - [`framing`]: framing `Content-Length` do wire LSP;
//! - [`parse`]: conversao das respostas cruas nos tipos do protocolo;
//! - [`edit`]: aplicacao de edits de texto sobre conteudo UTF-8;
//! - [`uri`]: conversao entre caminhos e URIs `file://`.

mod diagnostics_merge;
mod edit;
pub mod framing;
mod handshake;
mod manager;
mod parse;
mod parse_symbols;
mod query;
mod registry;
mod server;
mod session;
mod sync;
mod transaction;
mod types;
mod uri;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc},
};

use kinein_protocol::JsonRpcRequest;
use serde_json::Value;

pub use edit::apply_text_edits;
pub use manager::LspManager;
pub use query::{LspBegun, LspPending, LspQuery};
pub use session::LspReply;
pub use transaction::{
    WorkspaceEditApplied, WorkspaceEditTransactionError, WorkspaceEditTransactions,
};
pub use types::{FileEdits, LspError, LspLocation, TextSpanEdit, WorkspaceEditPlan};

/// Sender usado para empurrar notificacoes assincronas ao loop principal.
pub type EventSender = mpsc::Sender<JsonRpcRequest>;

/// Mapa de requests LSP aguardando resposta, compartilhado com a thread leitora.
pub(crate) type PendingResponses = Arc<Mutex<HashMap<i64, mpsc::Sender<Value>>>>;
