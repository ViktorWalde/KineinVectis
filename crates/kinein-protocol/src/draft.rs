//! Draft (autosave) payloads — rede de segurança de dados (docs/23).
//!
//! Rascunhos de buffers não salvos, persistidos em `SQLite` no core. Salvar/
//! limpar reusam `FsWriteParams` (`{ path, content }`) e `FsPathParams`
//! (`{ path }`). A recuperação vem embutida na resposta de `workspace.open`
//! como uma lista de [`DraftInfo`].

use serde::{Deserialize, Serialize};

/// Result of `draft.save`: o timestamp gravado (unix millis).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftSaveResult {
    /// Unix millis do autosave persistido.
    pub saved_at: i64,
}

/// Um rascunho não salvo recuperável (na resposta de `workspace.open`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftInfo {
    /// Caminho absoluto do arquivo com alterações não salvas.
    pub path: String,
    /// Conteúdo do buffer não salvo a restaurar.
    pub content: String,
    /// Unix millis do último autosave.
    pub saved_at: i64,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DraftInfo, DraftSaveResult};

    #[test]
    fn save_result_serializes_camel_case() {
        let value = serde_json::to_value(DraftSaveResult { saved_at: 42 }).unwrap();
        assert_eq!(value["savedAt"], 42);
    }

    #[test]
    fn draft_info_roundtrips() {
        let info = DraftInfo {
            path: "/ws/a.rs".to_owned(),
            content: "fn a() {}".to_owned(),
            saved_at: 7,
        };
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["path"], "/ws/a.rs");
        assert_eq!(value["savedAt"], 7);
        let back: DraftInfo = serde_json::from_value(json!({
            "path": "/ws/a.rs", "content": "fn a() {}", "savedAt": 7
        }))
        .unwrap();
        assert_eq!(back, info);
    }
}
