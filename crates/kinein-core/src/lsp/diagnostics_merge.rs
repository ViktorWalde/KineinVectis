//! A FUSAO dos diagnosticos de varios servidores de uma linguagem, por arquivo.
//!
//! Por que existe (2026-09-13): com o `ruff server` ao lado do basedpyright,
//! dois processos publicam `publishDiagnostics` para o MESMO `.py`. A UI
//! substitui a lista por arquivo a cada `event.lsp.diagnostics` — e' o
//! contrato certo para um servidor, e dois eventos parciais se apagariam um
//! ao outro (o F401 do ruff sumiria no proximo evento do pyright). O core
//! guarda a parte de cada servidor e emite sempre a UNIAO; a UI nao sabe que
//! sao dois.
//!
//! O cache e' compartilhado entre as threads leitoras (uma por servidor) e o
//! manager (que esquece um servidor ao derruba-lo).

use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

use kinein_protocol::{Diagnostic, JsonRpcRequest};

use super::parse::diagnostics_notification;

/// Por arquivo, a lista de cada servidor (pela chave do servidor, em ordem
/// estavel — o principal `python` antes de `python-ruff`).
pub(super) type MergedDiagnostics =
    Arc<Mutex<HashMap<String, BTreeMap<&'static str, Vec<Diagnostic>>>>>;

/// Grava o que `key` publicou para `path` e devolve o evento com a uniao.
pub(super) fn record(
    merged: &MergedDiagnostics,
    key: &'static str,
    path: &str,
    diagnostics: Vec<Diagnostic>,
) -> JsonRpcRequest {
    let Ok(mut cache) = merged.lock() else {
        return diagnostics_notification(path, &diagnostics);
    };
    let por_servidor = cache.entry(path.to_owned()).or_default();
    if diagnostics.is_empty() {
        por_servidor.remove(key);
    } else {
        por_servidor.insert(key, diagnostics);
    }
    let uniao = union(por_servidor);
    if por_servidor.is_empty() {
        cache.remove(path);
    }
    diagnostics_notification(path, &uniao)
}

/// Esquece tudo o que `key` publicou; devolve um evento por arquivo afetado,
/// ja' sem a parte dele — e' o que apaga da tela os diagnosticos de um
/// companheiro que saiu ou vai reiniciar.
pub(super) fn forget(merged: &MergedDiagnostics, key: &str) -> Vec<JsonRpcRequest> {
    let Ok(mut cache) = merged.lock() else {
        return Vec::new();
    };
    let mut eventos = Vec::new();
    cache.retain(|path, por_servidor| {
        if por_servidor.remove(key).is_none() {
            return true;
        }
        eventos.push(diagnostics_notification(path, &union(por_servidor)));
        !por_servidor.is_empty()
    });
    eventos
}

/// Esquece tudo (troca de workspace: nenhum evento — a UI ja' limpa sozinha).
pub(super) fn clear(merged: &MergedDiagnostics) {
    if let Ok(mut cache) = merged.lock() {
        cache.clear();
    }
}

fn union(por_servidor: &BTreeMap<&'static str, Vec<Diagnostic>>) -> Vec<Diagnostic> {
    por_servidor.values().flatten().cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::{MergedDiagnostics, clear, forget, record};
    use kinein_protocol::{Diagnostic, DiagnosticSeverity, DiagnosticSource};

    fn diag(mensagem: &str) -> Diagnostic {
        Diagnostic {
            id: None,
            source: DiagnosticSource::Lsp,
            severity: DiagnosticSeverity::Warning,
            category: Some("lsp".to_owned()),
            message: mensagem.to_owned(),
            file: None,
            line: Some(1),
            column: Some(1),
            end_line: Some(1),
            end_column: Some(2),
            code: None,
            job_id: None,
            command: None,
            target: None,
            log_ref: None,
        }
    }

    fn mensagens(evento: &kinein_protocol::JsonRpcRequest) -> Vec<String> {
        evento.params.as_ref().unwrap()["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["message"].as_str().unwrap().to_owned())
            .collect()
    }

    fn caminho(evento: &kinein_protocol::JsonRpcRequest) -> String {
        evento.params.as_ref().unwrap()["path"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// Dois servidores, um arquivo: cada evento leva a UNIAO, na ordem estavel
    /// das chaves; a lista vazia de um apaga so' a parte dele.
    #[test]
    fn each_event_carries_the_union_and_an_empty_publish_removes_only_that_server() {
        let merged: MergedDiagnostics = Arc::new(Mutex::new(std::collections::HashMap::default()));
        let e1 = record(&merged, "python", "/p/a.py", vec![diag("pyright")]);
        assert_eq!(e1.method, "event.lsp.diagnostics");
        assert_eq!(caminho(&e1), "/p/a.py");
        assert_eq!(mensagens(&e1), ["pyright"]);
        let e2 = record(&merged, "python-ruff", "/p/a.py", vec![diag("ruff")]);
        assert_eq!(
            mensagens(&e2),
            ["pyright", "ruff"],
            "a uniao, principal primeiro"
        );
        let e3 = record(&merged, "python", "/p/a.py", Vec::new());
        assert_eq!(mensagens(&e3), ["ruff"], "o pyright limpou; o ruff fica");
        let e4 = record(&merged, "python-ruff", "/p/a.py", Vec::new());
        assert!(mensagens(&e4).is_empty());
        assert!(
            merged.lock().unwrap().is_empty(),
            "arquivo sem nada sai do cache"
        );
        // Outro arquivo nao se mistura.
        record(&merged, "python", "/p/a.py", vec![diag("a")]);
        let eb = record(&merged, "python-ruff", "/p/b.py", vec![diag("b")]);
        assert_eq!(mensagens(&eb), ["b"]);
    }

    /// Esquecer um servidor devolve um evento por arquivo em que ele falava,
    /// ja' sem a parte dele — e nada para os arquivos em que nao falava.
    #[test]
    fn forgetting_a_server_reemits_only_the_files_it_touched() {
        let merged: MergedDiagnostics = Arc::new(Mutex::new(std::collections::HashMap::default()));
        record(&merged, "python", "/p/a.py", vec![diag("pyright")]);
        record(&merged, "python-ruff", "/p/a.py", vec![diag("ruff")]);
        record(&merged, "python-ruff", "/p/b.py", vec![diag("ruff-b")]);
        record(&merged, "python", "/p/c.py", vec![diag("pyright-c")]);
        let mut eventos = forget(&merged, "python-ruff");
        eventos.sort_by_key(caminho);
        assert_eq!(eventos.len(), 2, "a.py e b.py; c.py nao tinha ruff");
        assert_eq!(caminho(&eventos[0]), "/p/a.py");
        assert_eq!(mensagens(&eventos[0]), ["pyright"]);
        assert_eq!(caminho(&eventos[1]), "/p/b.py");
        assert!(mensagens(&eventos[1]).is_empty(), "b.py fica limpo na tela");
        assert_eq!(merged.lock().unwrap().len(), 2, "a.py e c.py continuam");
        assert!(
            forget(&merged, "python-ruff").is_empty(),
            "segunda vez: nada"
        );
        clear(&merged);
        assert!(merged.lock().unwrap().is_empty());
    }
}
