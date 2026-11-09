//! INF-03 (RUSTSEC-2026-0459, CVE-2026-119375): um `-wal` irmão com page size
//! menor que o do banco. Antes do `SQLite` 3.54, `sqlite3WalReadFrame` copiava
//! só o tamanho do quadro e deixava o resto da página com memória não
//! inicializada; o 3.54 zera esse resto.
//!
//! O par é o que um terceiro deixaria num repositório: o banco de rascunhos de
//! verdade e um `-wal` de um quadro com página de 1024 bytes, que leva o começo
//! da página de `drafts`. O ponteiro da célula aponta para além do quadro. O
//! processo filho roda com `MALLOC_PERTURB_`, e a glibc preenche toda memória
//! nova com um byte conhecido: memória não inicializada não vem zerada por
//! acaso. Medido em 2026-11-09: com o 3.46.0, a consulta da fonte de dados lê
//! esses bytes e falha com "database disk image is malformed"; com o 3.54.1,
//! lê zeros, um registro sem valores. O rascunho do projeto some nas duas
//! versões, porque o par forjado esconde a página e o `SQLite` não o detecta.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{DataSourceEngine, DataSourceProfile, SecretSource};

use crate::db::DraftStore;

const CHILD_ROOT: &str = "KINEIN_WAL_PAIR_ROOT";
const DRAFT_PATH: &str = "/ws/principal.c";
const SECRET: &str = "kv-segredo-de-outro-banco";
const WAL_PAGE: usize = 1024;

fn temp_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Conteúdo longo o bastante para a célula ficar no fim da página.
fn padded(text: &str) -> String {
    format!("{text:<240}")
}

/// O banco de rascunhos com um só rascunho, fechado: o `-wal` foi consolidado.
fn draft_database(root: &Path, content: &str) -> PathBuf {
    std::fs::create_dir_all(root).unwrap();
    let store = DraftStore::open(root).unwrap();
    store.save(DRAFT_PATH, content).unwrap();
    drop(store);
    let database = root.join(".kinein/kinein.db");
    assert!(!wal_of(&database).exists(), "o fechamento consolida o -wal");
    database
}

fn wal_of(database: &Path) -> PathBuf {
    database.with_extension("db-wal")
}

fn root_page(database: &Path, table: &str) -> u32 {
    let connection = rusqlite::Connection::open(database).unwrap();
    connection
        .query_row(
            "SELECT rootpage FROM sqlite_schema WHERE name = ?1",
            [table],
            |row| row.get(0),
        )
        .unwrap()
}

/// O checksum do WAL (`wal.c`), em palavras big-endian e acumulado.
fn checksum(data: &[u8], (mut first, mut second): (u32, u32)) -> (u32, u32) {
    for pair in data.chunks_exact(8) {
        let low = u32::from_be_bytes(pair[..4].try_into().unwrap());
        let high = u32::from_be_bytes(pair[4..].try_into().unwrap());
        first = first.wrapping_add(low).wrapping_add(second);
        second = second.wrapping_add(high).wrapping_add(first);
    }
    (first, second)
}

/// Um `-wal` com um quadro de commit de [`WAL_PAGE`] bytes: o começo da
/// página `page` do banco.
fn small_page_wal(database: &[u8], page: u32) -> Vec<u8> {
    let page_size = usize::from(u16::from_be_bytes([database[16], database[17]]));
    assert!(
        page_size > WAL_PAGE,
        "a página do banco é maior que o quadro"
    );
    let salt = [0x4b56_0001_u32, 0x4b56_0002];
    let mut wal = Vec::new();
    let frame_size = u32::try_from(WAL_PAGE).unwrap();
    for word in [0x377f_0683, 3_007_000, frame_size, 0, salt[0], salt[1]] {
        wal.extend_from_slice(&u32::to_be_bytes(word));
    }
    let header_sum = checksum(&wal, (0, 0));
    wal.extend_from_slice(&header_sum.0.to_be_bytes());
    wal.extend_from_slice(&header_sum.1.to_be_bytes());

    let start = (page as usize - 1) * page_size;
    let data = &database[start..start + WAL_PAGE];
    let pages = u32::try_from(database.len() / page_size).unwrap();
    let mut frame = Vec::new();
    frame.extend_from_slice(&page.to_be_bytes());
    frame.extend_from_slice(&pages.to_be_bytes());
    let frame_sum = checksum(data, checksum(&frame, header_sum));
    for word in [salt[0], salt[1], frame_sum.0, frame_sum.1] {
        frame.extend_from_slice(&word.to_be_bytes());
    }
    wal.extend_from_slice(&frame);
    wal.extend_from_slice(data);
    wal
}

/// O par forjado em `root`. Antes, o processo lê e libera um banco igual cujo
/// rascunho é a agulha: a memória antiga, se vazar, é a dela.
fn crafted_pair(root: &Path) -> (PathBuf, String) {
    let own = padded("int visivel = 1; /* rascunho deste projeto */");
    let primer = root.join("outro banco");
    draft_database(&primer, &padded(SECRET));
    let database = draft_database(&root.join("projeto ação"), &own);
    let page = root_page(&database, "drafts");
    assert!(!wal_of(&database).exists());
    let bytes = std::fs::read(&database).unwrap();
    std::fs::write(wal_of(&database), small_page_wal(&bytes, page)).unwrap();

    let store = DraftStore::open(&primer).unwrap();
    assert!(store.list().unwrap()[0].content.starts_with(SECRET));
    drop(store);
    (database, own)
}

fn sqlite_profile(database: &Path) -> DataSourceProfile {
    DataSourceProfile {
        production: false,
        read_only: false,
        name: "terceiro".to_owned(),
        engine: DataSourceEngine::Sqlite,
        host: String::new(),
        port: 0,
        database: database.display().to_string(),
        user: String::new(),
        secret_source: SecretSource::Automatic,
        secret_variable: None,
        sample_size: None,
        tls: None,
        ca_file: None,
        installation: None,
    }
}

#[test]
fn the_linked_sqlite_zeroes_the_rest_of_a_smaller_wal_page() {
    assert!(
        rusqlite::version_number() >= 3_054_000,
        "SQLite {} é anterior ao 3.54.0",
        rusqlite::version()
    );
}

#[test]
fn a_smaller_page_wal_reads_zeros_and_never_process_memory() {
    let root = temp_root("wal-pair");
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "tests::sqlite_wal_pair::smaller_page_wal_child"])
        .env(CHILD_ROOT, &root)
        .env("MALLOC_PERTURB_", "165")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("filho concluiu").exists(), "o teste filho rodou");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn smaller_page_wal_child() {
    let Some(root) = std::env::var_os(CHILD_ROOT) else {
        return;
    };
    let root = PathBuf::from(root);
    let (database, own) = crafted_pair(&root);

    // A fonte de dados abre o par só para ler. A célula fora do quadro é
    // zero: um registro sem valores, nunca o byte da memória nova.
    let read = crate::datasource::query::run_sqlite(
        &sqlite_profile(&database),
        "SELECT content FROM drafts",
        10,
    )
    .map(|result| result.rows)
    .map_err(|failure| failure.message);
    assert_eq!(read, Ok(vec![vec![None]]));

    // A recuperação de rascunhos não restaura conteúdo de outro banco.
    let store = DraftStore::open(database.parent().unwrap().parent().unwrap()).unwrap();
    let drafts = store.list().unwrap();
    assert!(
        drafts.iter().all(|draft| draft.content == own),
        "{drafts:?}"
    );
    std::fs::write(root.join("filho concluiu"), "").unwrap();
}
