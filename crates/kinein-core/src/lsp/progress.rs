//! O PROGRESSO dos servidores (L6, 0.4.0d, `roadmaps/52` §14.4, protocolo
//! `0.176.0`): o `$/progress` vira `event.lsp.progress`.
//!
//! Medido em 2026-10-16: o clangd 21 manda uma frente ("indexing") com dois
//! relatos; o rust-analyzer 1.96 manda 13 frentes e uns 7 relatos por
//! segundo. O comeco e o fim de cada frente sempre passam; os relatos, no
//! maximo um a cada [`REPORT_INTERVAL`] por frente — a barra nao precisa de
//! mais, e o IPC nao enche. Puro: quem le' o servidor e emite e' a thread
//! leitora (`server.rs`).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::{Map, Value};

/// O intervalo minimo entre dois relatos de uma mesma frente (4 por segundo).
const REPORT_INTERVAL: Duration = Duration::from_millis(250);

/// As frentes abertas de UM servidor, pelo token.
#[derive(Debug, Default)]
pub(super) struct ProgressTracker {
    fronts: HashMap<String, Front>,
}

#[derive(Debug)]
struct Front {
    title: String,
    last_report: Instant,
}

impl ProgressTracker {
    /// O corpo do `event.lsp.progress` (sem a linguagem) para os `params` de
    /// um `$/progress`, ou `None` quando nao ha' o que dizer (relato cedo
    /// demais, frente desconhecida, forma estranha).
    pub(super) fn event(&mut self, params: &Value, now: Instant) -> Option<Value> {
        let token = match params.get("token")? {
            Value::String(token) => token.clone(),
            Value::Number(token) => token.to_string(),
            _ => return None,
        };
        let value = params.get("value")?;
        match value.get("kind")?.as_str()? {
            "begin" => {
                let title = value
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let body = payload(&title, value, false);
                self.fronts.insert(
                    token,
                    Front {
                        title,
                        last_report: now,
                    },
                );
                Some(body)
            }
            "report" => {
                let front = self.fronts.get_mut(&token)?;
                if now.duration_since(front.last_report) < REPORT_INTERVAL {
                    return None;
                }
                front.last_report = now;
                Some(payload(&front.title, value, false))
            }
            "end" => {
                let front = self.fronts.remove(&token)?;
                Some(payload(&front.title, value, true))
            }
            _ => None,
        }
    }
}

fn payload(title: &str, value: &Value, done: bool) -> Value {
    let mut body = Map::new();
    body.insert("title".to_owned(), Value::String(title.to_owned()));
    if let Some(message) = value.get("message").and_then(Value::as_str) {
        body.insert("message".to_owned(), Value::String(message.to_owned()));
    }
    if let Some(percentage) = value.get("percentage").and_then(Value::as_u64) {
        body.insert("percentage".to_owned(), Value::from(percentage.min(100)));
    }
    body.insert("done".to_owned(), Value::Bool(done));
    Value::Object(body)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn progress(token: &str, value: &Value) -> Value {
        json!({ "token": token, "value": value })
    }

    /// Comeco, relatos limitados no tempo, fim: cada um com o titulo da
    /// frente; o relato depois do fim nao vira nada.
    #[test]
    fn a_front_begins_reports_at_most_four_times_a_second_and_ends() {
        let mut tracker = ProgressTracker::default();
        let t0 = Instant::now();
        let begin = tracker
            .event(
                &progress(
                    "rustAnalyzer/Indexing",
                    &json!({ "kind": "begin", "title": "Indexing", "percentage": 0 }),
                ),
                t0,
            )
            .unwrap();
        assert_eq!(
            begin,
            json!({ "title": "Indexing", "percentage": 0, "done": false })
        );
        let early = json!({ "kind": "report", "message": "2/40 (core)", "percentage": 5 });
        assert!(
            tracker
                .event(
                    &progress("rustAnalyzer/Indexing", &early),
                    t0 + Duration::from_millis(100)
                )
                .is_none(),
            "cedo demais"
        );
        let later = tracker
            .event(
                &progress("rustAnalyzer/Indexing", &early),
                t0 + Duration::from_millis(300),
            )
            .unwrap();
        assert_eq!(
            later,
            json!({ "title": "Indexing", "message": "2/40 (core)", "percentage": 5, "done": false })
        );
        let end = tracker
            .event(
                &progress("rustAnalyzer/Indexing", &json!({ "kind": "end" })),
                t0 + Duration::from_millis(310),
            )
            .unwrap();
        assert_eq!(
            end,
            json!({ "title": "Indexing", "done": true }),
            "o fim passa sempre"
        );
        assert!(
            tracker
                .event(
                    &progress("rustAnalyzer/Indexing", &early),
                    t0 + Duration::from_secs(5)
                )
                .is_none()
        );
    }

    /// Token numerico (o clangd usa string, outros usam numero), frente
    /// desconhecida e forma estranha.
    #[test]
    fn numeric_tokens_work_and_unknown_or_odd_messages_say_nothing() {
        let mut tracker = ProgressTracker::default();
        let now = Instant::now();
        let begin = json!({ "token": 7, "value": { "kind": "begin", "title": "indexing", "percentage": 250 } });
        assert_eq!(tracker.event(&begin, now).unwrap()["percentage"], 100);
        assert!(
            tracker
                .event(&json!({ "token": 8, "value": { "kind": "end" } }), now)
                .is_none()
        );
        assert!(tracker.event(&json!({ "token": 7 }), now).is_none());
        assert!(
            tracker
                .event(&json!({ "token": true, "value": { "kind": "begin" } }), now)
                .is_none()
        );
    }
}
