//! Persistência local em `SQLite` — rede de segurança de dados (docs/seguranca/23).
//!
//! Um DB por-workspace em `.kinein/kinein.db` (WAL + escrita atômica por
//! transação). Hoje guarda RASCUNHOS: o autosave de buffers não salvos, que
//! sobrevive a um crash da UI / power loss e é recuperado no próximo
//! `workspace.open`. A store é reutilizável para Local History e migração da
//! sessão no futuro (P4 do docs/seguranca/23). DB corrompido/incompatível → recria (o
//! mesmo espírito inválido→default do storage JSON); nunca derruba o core.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, params};

/// Versão do schema local (migrações via `PRAGMA user_version`).
const SCHEMA_VERSION: i64 = 2;

/// Um rascunho não salvo, recuperável após um crash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// Caminho absoluto do arquivo.
    pub path: String,
    /// Conteúdo do buffer não salvo.
    pub content: String,
    /// Unix millis do autosave.
    pub saved_at: i64,
}

/// Store de rascunhos em `SQLite`, aberta por-workspace.
#[derive(Debug)]
pub struct DraftStore {
    conn: Connection,
}

impl DraftStore {
    /// Abre (ou cria) `<root>/.kinein/kinein.db`, aplica WAL e as migrações.
    /// Retorna `None` (persistência desabilitada, sem quebrar) se o diretório
    /// não puder ser criado; DB corrompido é recriado do zero.
    #[must_use]
    pub fn open(root: &Path) -> Option<Self> {
        Self::open_in(&root.join(".kinein"))
    }

    /// Abre (ou cria) `<dir>/kinein.db` num diretório arbitrário. É o mesmo
    /// mecanismo do por-workspace, usado também para o escopo GLOBAL da
    /// configuração de integrações (diretório XDG de estado) — uma store só,
    /// sem inventar formato novo (fatia 2.2, 2026-07-19).
    #[must_use]
    pub fn open_in(dir: &Path) -> Option<Self> {
        if std::fs::create_dir_all(dir).is_err() {
            return None;
        }
        let path = dir.join("kinein.db");
        if let Ok(store) = Self::open_at(&path) {
            return Some(store);
        }
        // Corrompido/incompatível: recria (não perde a IDE por causa do cache).
        drop(std::fs::remove_file(&path));
        Self::open_at(&path).ok()
    }

    fn open_at(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        // WAL + synchronous NORMAL: escrita atômica por transação, durável
        // contra crash de app (perde no máximo a última transação em power
        // loss real — aceitável para autosave).
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Salva/atualiza o rascunho de um arquivo (autosave de buffer sujo).
    /// Devolve o timestamp gravado.
    pub fn save(&self, path: &str, content: &str) -> rusqlite::Result<i64> {
        let saved_at = now_millis();
        self.conn.execute(
            "INSERT INTO drafts(path, content, saved_at) VALUES(?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET content = ?2, saved_at = ?3",
            params![path, content, saved_at],
        )?;
        Ok(saved_at)
    }

    /// Remove o rascunho de um arquivo (save/close limpo).
    pub fn clear(&self, path: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM drafts WHERE path = ?1", params![path])?;
        Ok(())
    }

    /// Grava (ou sobrescreve) um valor de configuração de integração.
    /// Devolve `true` quando o valor efetivamente MUDOU (chave nova ou valor
    /// diferente) — é o que decide se `event.integration.changed` é emitido.
    pub fn config_set(&self, integration: &str, key: &str, value: &str) -> rusqlite::Result<bool> {
        let atual: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM integration_config
                 WHERE integration_id = ?1 AND key = ?2",
                params![integration, key],
                |row| row.get(0),
            )
            .ok();
        if atual.as_deref() == Some(value) {
            return Ok(false);
        }
        self.conn.execute(
            "INSERT INTO integration_config(integration_id, key, value)
             VALUES(?1, ?2, ?3)
             ON CONFLICT(integration_id, key) DO UPDATE SET value = ?3",
            params![integration, key, value],
        )?;
        Ok(true)
    }

    /// Remove a sobreposição de uma chave (reset ao default). Devolve `true`
    /// quando havia algo para remover; `false` é um no-op sem evento.
    pub fn config_reset(&self, integration: &str, key: &str) -> rusqlite::Result<bool> {
        let removidas = self.conn.execute(
            "DELETE FROM integration_config WHERE integration_id = ?1 AND key = ?2",
            params![integration, key],
        )?;
        Ok(removidas > 0)
    }

    /// Lista os pares chave→valor armazenados para uma integração.
    pub fn config_list(&self, integration: &str) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value FROM integration_config
             WHERE integration_id = ?1 ORDER BY key",
        )?;
        let rows = stmt.query_map(params![integration], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    /// Lista todos os rascunhos guardados.
    pub fn list(&self) -> rusqlite::Result<Vec<Draft>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, content, saved_at FROM drafts ORDER BY saved_at")?;
        let rows = stmt.query_map([], |row| {
            Ok(Draft {
                path: row.get(0)?,
                content: row.get(1)?,
                saved_at: row.get(2)?,
            })
        })?;
        rows.collect()
    }
}

/// Aplica as migrações até `SCHEMA_VERSION`.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS drafts(
                path TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                saved_at INTEGER NOT NULL
            );",
        )?;
    }
    if version < 2 {
        // Fatia 2.2 (2026-07-19): configuração genérica de integrações,
        // chave→valor por integração. O ESCOPO é o arquivo: o DB do workspace
        // guarda o escopo workspace, o DB global (XDG) guarda o global.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS integration_config(
                integration_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY(integration_id, key)
            );",
        )?;
    }
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

/// Unix millis atual (saturando, nunca entra em pânico).
fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::DraftStore;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-db-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn save_list_clear_roundtrip() {
        let root = temp_root("roundtrip");
        let store = DraftStore::open(&root).expect("abre store");

        store.save("/ws/a.rs", "fn a() {}").unwrap();
        store.save("/ws/b.rs", "fn b() {}").unwrap();
        let drafts = store.list().unwrap();
        assert_eq!(drafts.len(), 2);
        assert!(
            drafts
                .iter()
                .any(|d| d.path == "/ws/a.rs" && d.content == "fn a() {}")
        );

        // Upsert: salvar de novo atualiza, não duplica.
        store.save("/ws/a.rs", "fn a() { changed }").unwrap();
        let drafts = store.list().unwrap();
        assert_eq!(drafts.len(), 2);
        assert_eq!(
            drafts
                .iter()
                .find(|d| d.path == "/ws/a.rs")
                .unwrap()
                .content,
            "fn a() { changed }"
        );

        // Clear remove só o alvo.
        store.clear("/ws/a.rs").unwrap();
        let drafts = store.list().unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].path, "/ws/b.rs");
    }

    #[test]
    fn drafts_survive_reopen() {
        let root = temp_root("reopen");
        {
            let store = DraftStore::open(&root).expect("abre");
            store.save("/ws/c.rs", "persistido").unwrap();
        }
        // Reabrir a store (simula reinício do processo).
        let store = DraftStore::open(&root).expect("reabre");
        let drafts = store.list().unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].content, "persistido");
    }
}
