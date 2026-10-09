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

/// Writes and synchronizes an exclusive sibling before replacing the target.
/// Shared by editor saves and workspace catalogues; callers validate paths.
///
/// # Errors
/// Failure to create, write, synchronize or publish the temporary file.
pub(crate) fn atomic_write(target: &std::path::Path, bytes: &[u8]) -> Result<(), FsError> {
    write_atomically(target, bytes, false)
}

/// [`atomic_write`] for private data (`0600` from creation; on Windows, the
/// folder ACL, see `platform::owner_only_file`): the temporary file is never
/// readable by others, not even before the rename.
///
/// # Errors
/// The same as [`atomic_write`].
pub(crate) fn atomic_write_private(target: &std::path::Path, bytes: &[u8]) -> Result<(), FsError> {
    write_atomically(target, bytes, true)
}

fn write_atomically(
    target: &std::path::Path,
    bytes: &[u8],
    owner_only: bool,
) -> Result<(), FsError> {
    use std::{fs, io, io::Write, path::Path};

    let io_err = |path: &Path, source: io::Error| FsError::Io {
        path: path.display().to_string(),
        source,
    };
    let temp = publish::temp_sibling(target);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    if owner_only {
        crate::platform::owner_only_file(&mut options);
    }
    let mut file = options
        .open(&temp)
        .map_err(|source| io_err(&temp, source))?;
    if let Err(source) = file.write_all(bytes) {
        drop(fs::remove_file(&temp));
        return Err(io_err(&temp, source));
    }
    if let Err(source) = file.sync_all() {
        drop(fs::remove_file(&temp));
        return Err(io_err(&temp, source));
    }
    drop(file);
    fs::rename(&temp, target).map_err(|source| {
        drop(fs::remove_file(&temp));
        io_err(target, source)
    })
}
