//! O unico lugar do Kinein Vectis com `unsafe` (`DocsPublic/roadmaps/60` §3.1,
//! decisao D7 do autor, 2026-10-09).
//!
//! O core e' `forbid(unsafe_code)`. O que so' a API do Windows faz mora aqui,
//! atras de uma interface segura:
//!
//! - [`Job`]: o Job Object, o grupo de processos que morre junto. E' o par do
//!   grupo de processo do Unix, e nenhum descendente escapa dele;
//! - [`rename_noreplace`]: mover sem nunca sobrescrever, atomico no mesmo
//!   volume, o par do `renameat2(RENAME_NOREPLACE)`;
//! - [`resume_process`]: retomar um processo criado suspenso, para que ele
//!   entre no job antes de rodar a primeira instrucao.
//!
//! Cada bloco `unsafe` diz, no `SAFETY`, por que a chamada e' valida (a lint
//! `undocumented_unsafe_blocks` reprova o que nao diz). No Unix este crate e'
//! vazio: o core usa o `rustix` e a biblioteca padrao.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{Job, rename_noreplace, resume_process};
