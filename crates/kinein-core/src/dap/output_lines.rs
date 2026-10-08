//! As linhas de `event.debug.output`, inteiras (achado em 2026-10-08).
//!
//! O adaptador DAP pode entregar UMA linha do programa em varios eventos
//! `output`: sob carga, o debugpy mandou o `print("resultado", 5)` como
//! "resultado" e depois " 5\n". Quebrar cada evento por `lines()` fazia o
//! console mostrar duas linhas, "resultado" e " 5". Aqui o pedaco sem quebra
//! espera o resto, por categoria (stdout e stderr nao se misturam), e o que
//! sobrou sai quando o programa para, termina ou o fluxo acaba: nada se perde.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::reader::send_event;
use crate::lsp::EventSender;

/// O pedaco pendente de cada categoria.
#[derive(Debug, Default)]
pub(super) struct OutputLines {
    partial: BTreeMap<String, String>,
}

impl OutputLines {
    /// Um evento `output` do adaptador: emite as linhas que fecharam e guarda
    /// o resto (telemetry e' ignorada).
    pub(super) fn push(&mut self, events: &EventSender, body: &Value) {
        let category = body
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or("console");
        if category == "telemetry" {
            return;
        }
        let Some(output) = body.get("output").and_then(Value::as_str) else {
            return;
        };
        let mut text = self.partial.remove(category).unwrap_or_default();
        text.push_str(output);
        let mut rest = text.as_str();
        while let Some(end) = rest.find('\n') {
            emit(events, category, &rest[..end]);
            rest = &rest[end + 1..];
        }
        if !rest.is_empty() {
            self.partial.insert(category.to_owned(), rest.to_owned());
        }
    }

    /// O que ficou sem quebra de linha sai agora (o programa parou ou acabou).
    pub(super) fn flush(&mut self, events: &EventSender) {
        for (category, line) in std::mem::take(&mut self.partial) {
            emit(events, &category, &line);
        }
    }
}

fn emit(events: &EventSender, category: &str, line: &str) {
    send_event(
        events,
        "event.debug.output",
        json!({ "category": category, "line": line.strip_suffix('\r').unwrap_or(line) }),
    );
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn lines(receiver: &mpsc::Receiver<kinein_protocol::JsonRpcRequest>) -> Vec<(String, String)> {
        receiver
            .try_iter()
            .map(|event| {
                let params = event.params.unwrap();
                (
                    params["category"].as_str().unwrap().to_owned(),
                    params["line"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    fn out(category: &str, text: &str) -> Value {
        json!({ "category": category, "output": text })
    }

    #[test]
    fn a_line_split_across_events_comes_out_whole() {
        let (sender, receiver) = mpsc::channel();
        let mut output = OutputLines::default();
        output.push(&sender, &out("stdout", "resultado"));
        assert!(lines(&receiver).is_empty(), "o pedaco nao e' linha ainda");
        output.push(&sender, &out("stdout", " 5\r\nsegunda\nterc"));
        assert_eq!(
            lines(&receiver),
            [
                ("stdout".into(), "resultado 5".into()),
                ("stdout".into(), "segunda".into())
            ]
        );
        output.flush(&sender);
        assert_eq!(lines(&receiver), [("stdout".into(), "terc".into())]);
        output.flush(&sender);
        assert!(
            lines(&receiver).is_empty(),
            "esvaziar duas vezes nao repete"
        );
    }

    #[test]
    fn categories_do_not_mix_and_empty_lines_survive() {
        let (sender, receiver) = mpsc::channel();
        let mut output = OutputLines::default();
        output.push(&sender, &out("stdout", "meio"));
        output.push(&sender, &out("stderr", "erro\n"));
        output.push(&sender, &out("stdout", " fim\n\n"));
        output.push(&sender, &out("telemetry", "ignorado\n"));
        assert_eq!(
            lines(&receiver),
            [
                ("stderr".into(), "erro".into()),
                ("stdout".into(), "meio fim".into()),
                ("stdout".into(), String::new())
            ]
        );
    }
}
