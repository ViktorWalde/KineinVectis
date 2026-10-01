//! File system operations confined to the open workspace root.
//!
//! Workspace operations confine paths to the open root. Explicit import reads
//! selected local sources through descriptor-relative traversal.
//! The UI never touches the file system directly; it goes through `fs.list`,
//! `fs.read`, `fs.createFile`, `fs.createDirectory`, `fs.write`, `fs.rename`,
//! `fs.copy`, `fs.trash`, `fs.delete`, `fs.search` and `fs.findFiles`.
//!
//! Organizacao interna:
//! - [`error`]: o erro estruturado devolvido por toda operacao;
//! - [`confine`]: canonicalizacao e confinamento de caminhos ao root;
//! - [`ops`]: list, read, create, write e rename;
//! - [`remove_ops`]: lixeira e exclusão permanente sob o mesmo confinamento;
//! - [`copy_ops`]: cópia com progresso e cancelamento;
//! - [`publish`]: staging e publicacao sem sobrescrita;
//! - [`walk`]: caminhada deterministica compartilhada por search e replace;
//! - [`search`]: busca literal por conteudo;
//! - [`replace`]: substituicao literal multi-arquivo, transacional;
//! - [`transaction`]: escrita com snapshot e rollback inverso;
//! - [`transfer_batch`]: preflight e resultados por item para cópia/movimento/importação em lote;
//! - [`find`]: busca de arquivos por nome via `fd`.

mod confine;
mod copy_ops;
mod error;
mod find;
mod ops;
mod publish;
mod remove_ops;
mod replace;
mod search;
pub mod transaction;
mod transfer_batch;
mod walk;

pub use confine::{confine_directory, confine_file};
pub use copy_ops::{copy, copy_with_progress};
pub use error::FsError;
pub use find::find_files;
pub use ops::{
    create_directory, create_file, list_dir, read_external_file, read_file, rename, write_file,
    write_file_if_unchanged,
};
pub use remove_ops::{delete, move_to_trash};
pub use replace::replace;
pub use search::search;
pub(crate) use transaction::{TextFileUpdate, write_text_transaction, write_text_transaction_with};
pub use transfer_batch::transfer_batch_with_progress;

/// Maximum file size accepted by `fs.read`, in bytes.
pub const MAX_READ_BYTES: u64 = 1_048_576;

/// Maximum number of matches returned by `fs.search`.
pub const MAX_SEARCH_MATCHES: usize = 500;

/// Maximum number of file matches returned by `fs.findFiles`.
pub const MAX_FILE_MATCHES: usize = 100;

/// Directory names skipped by `fs.search`/`fs.findFiles` (VCS, caches, build output).
const SEARCH_SKIP_DIRS: &[&str] = &[
    ".git",
    ".kinein",
    ".idea",
    ".cache",
    "target",
    "build",
    "node_modules",
];
