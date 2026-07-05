//! File system operations confined to the open workspace root.
//!
//! Every path is canonicalized and rejected when it escapes the workspace.
//! The UI never touches the file system directly; it goes through `fs.list`,
//! `fs.read`, `fs.createFile`, `fs.createDirectory`, `fs.write`, `fs.rename`,
//! `fs.delete`, `fs.search` and `fs.findFiles`.
//!
//! Organizacao interna:
//! - [`error`]: o erro estruturado devolvido por toda operacao;
//! - [`confine`]: canonicalizacao e confinamento de caminhos ao root;
//! - [`ops`]: list, read, create, write, rename e delete;
//! - [`search`]: busca literal por conteudo, com walk deterministico;
//! - [`find`]: busca de arquivos por nome via `fd`.

mod confine;
mod error;
mod find;
mod ops;
mod search;

pub use confine::confine_file;
pub use error::FsError;
pub use find::find_files;
pub use ops::{create_directory, create_file, delete, list_dir, read_file, rename, write_file};
pub use search::search;

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
