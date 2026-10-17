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
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use kinein_protocol::{Diagnostic, JsonRpcRequest};

use super::parse::diagnostics_notification;

/// Por arquivo, a lista de cada servidor, pela chave.
///
/// Em ordem estavel — o principal `python` antes de `python-ruff`. A checagem
/// do compilador ao salvar (0.4.0c) entra com a chave `compiler`.
pub type MergedDiagnostics = Arc<Mutex<HashMap<String, BTreeMap<&'static str, Vec<Diagnostic>>>>>;

/// Grava o que `key` publicou para `path` e devolve o evento com a uniao.
///
/// A checagem do compilador ao salvar a usa (`handlers/syntax_check.rs`); a
/// thread leitora de um servidor usa [`record_while_running`], que descarta o
/// que chega de um servidor ja' parado.
pub fn record(
    merged: &MergedDiagnostics,
    key: &'static str,
    path: &str,
    diagnostics: Vec<Diagnostic>,
) -> JsonRpcRequest {
    let Ok(mut cache) = merged.lock() else {
        return diagnostics_notification(path, &diagnostics);
    };
    record_locked(&mut cache, key, path, diagnostics)
}

/// Como [`record`], para a thread leitora de um servidor, que ja' ENVIA o
/// evento; o que chega depois de o servidor ser parado e esquecido nao volta
/// a tela. `false` = a thread sai (servidor parado ou canal fechado).
///
/// A conferencia e' DENTRO da trava: o `kill_server` liga o `stopping` antes
/// do `forget`, entao uma gravacao que entra depois dele ve o `stopping` e e'
/// descartada, e a que entrou antes e' apagada pelo proprio `forget`. Fora da
/// trava, um `publishDiagnostics` que estava no buffer quando o servidor
/// morreu era gravado DEPOIS do `forget` (40.7 §7.309). O ENVIO tambem e'
/// dentro dela (0.4.0e): enviado fora, o evento gravado antes do `forget`
/// podia chegar a UI depois do que o apaga, e o diagnostico do servidor
/// parado voltava a tela.
pub(super) fn record_while_running(
    merged: &MergedDiagnostics,
    key: &'static str,
    path: &str,
    diagnostics: Vec<Diagnostic>,
    stopping: &AtomicBool,
    events: &super::EventSender,
) -> bool {
    let Ok(mut cache) = merged.lock() else {
        return events
            .send(diagnostics_notification(path, &diagnostics))
            .is_ok();
    };
    if stopping.load(Ordering::SeqCst) {
        return false;
    }
    events
        .send(record_locked(&mut cache, key, path, diagnostics))
        .is_ok()
}

fn record_locked(
    cache: &mut HashMap<String, BTreeMap<&'static str, Vec<Diagnostic>>>,
    key: &'static str,
    path: &str,
    diagnostics: Vec<Diagnostic>,
) -> JsonRpcRequest {
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

    use std::sync::atomic::{AtomicBool, Ordering};

    use super::{MergedDiagnostics, clear, forget, record, record_while_running};
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

    fn messages(evento: &kinein_protocol::JsonRpcRequest) -> Vec<String> {
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
        assert_eq!(messages(&e1), ["pyright"]);
        let e2 = record(&merged, "python-ruff", "/p/a.py", vec![diag("ruff")]);
        assert_eq!(
            messages(&e2),
            ["pyright", "ruff"],
            "a uniao, principal primeiro"
        );
        let e3 = record(&merged, "python", "/p/a.py", Vec::new());
        assert_eq!(messages(&e3), ["ruff"], "o pyright limpou; o ruff fica");
        let e4 = record(&merged, "python-ruff", "/p/a.py", Vec::new());
        assert!(messages(&e4).is_empty());
        assert!(
            merged.lock().unwrap().is_empty(),
            "arquivo sem nada sai do cache"
        );
        // Outro arquivo nao se mistura.
        record(&merged, "python", "/p/a.py", vec![diag("a")]);
        let eb = record(&merged, "python-ruff", "/p/b.py", vec![diag("b")]);
        assert_eq!(messages(&eb), ["b"]);
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
        assert_eq!(messages(&eventos[0]), ["pyright"]);
        assert_eq!(caminho(&eventos[1]), "/p/b.py");
        assert!(messages(&eventos[1]).is_empty(), "b.py fica limpo na tela");
        assert_eq!(merged.lock().unwrap().len(), 2, "a.py e c.py continuam");
        assert!(
            forget(&merged, "python-ruff").is_empty(),
            "segunda vez: nada"
        );
        clear(&merged);
        assert!(merged.lock().unwrap().is_empty());
    }

    /// O servidor parado e esquecido: o que a thread leitora ainda tinha no
    /// buffer nao volta (40.7 §7.309); antes de parar, grava como sempre.
    #[test]
    fn a_publish_that_arrives_after_the_server_stopped_is_dropped() {
        let merged: MergedDiagnostics = Arc::new(Mutex::new(std::collections::HashMap::default()));
        let stopping = AtomicBool::new(false);
        let (events, received) = std::sync::mpsc::channel();
        record(&merged, "python", "/p/a.py", vec![diag("pyright")]);
        let live = record_while_running(
            &merged,
            "python-ruff",
            "/p/a.py",
            vec![diag("ruff")],
            &stopping,
            &events,
        );
        assert!(live, "rodando, a thread segue");
        assert_eq!(messages(&received.try_recv().unwrap()), ["pyright", "ruff"]);
        stopping.store(true, Ordering::SeqCst);
        let gone = forget(&merged, "python-ruff");
        assert_eq!(messages(&gone[0]), ["pyright"]);
        let late = record_while_running(
            &merged,
            "python-ruff",
            "/p/a.py",
            vec![diag("ruff atrasado")],
            &stopping,
            &events,
        );
        assert!(!late, "parado, a thread sai");
        assert!(received.try_recv().is_err(), "o atrasado nao vira evento");
        let next = record(&merged, "python", "/p/a.py", vec![diag("pyright")]);
        assert_eq!(messages(&next), ["pyright"], "e nao ficou no cache");
    }
}
