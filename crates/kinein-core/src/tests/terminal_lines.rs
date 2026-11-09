//! INF-05: as linhas lógicas da tela vêm da marca `wrapped` do render
//! (0.208.0), não do comprimento da linha.
//!
//! Até 2026-11-09 o ajudante `render_lines` juntava uma linha cheia que não
//! terminava em espaço. Uma quebra exatamente num espaço partia o texto, e o
//! veredito de `serial::ownership_run` dependia do comprimento do `TMPDIR` e do
//! PID (62 §3.4).

use std::time::Duration;

use kinein_protocol::JsonRpcRequest;
use serde_json::{Value, json};

use super::{core_with_empty_search_path, render_lines, terminal_run_until_closed};

fn frame(cols: u16, lines: &Value, wrapped: &Value) -> JsonRpcRequest {
    JsonRpcRequest::notification(
        "event.terminal.render",
        Some(json!({"cols": cols, "lines": lines, "wrapped": wrapped})),
    )
}

#[test]
fn only_the_wrapped_mark_joins_rows_and_the_dropped_space_comes_back() {
    // O espaço depois do texto colorido é o run final em estilo default, que
    // o core descarta: a linha quebrada ainda ocupa as cinco colunas.
    let lines = json!([
        [{"text": "aaaa", "cells": 4, "fg": 1}],
        [{"text": "bb", "cells": 2}],
        [{"text": "ccccc", "cells": 5}],
        [{"text": "d", "cells": 1}],
        [{"text": "eeeee", "cells": 5}],
        [{"text": "fffff", "cells": 5}],
        [{"text": "g", "cells": 1}],
        [{"text": "hh", "cells": 2}],
        [],
    ]);
    // A última quebra continua numa linha que o programa apagou: o
    // preenchimento não sobra no fim.
    assert_eq!(
        render_lines(&frame(5, &lines, &json!([0, 4, 5, 7]))),
        ["aaaa bb", "ccccc", "d", "eeeeefffffg", "hh"]
    );
    // Sem a marca, nenhuma linha cheia é juntada.
    assert_eq!(
        render_lines(&frame(5, &lines, &json!([]))),
        ["aaaa", "bb", "ccccc", "d", "eeeee", "fffff", "g", "hh"]
    );
}

/// Pela tela de verdade: `printf` imprime uma linha com o espaço exatamente
/// na última coluna e outra que enche a linha e termina com `\n`. O texto
/// antes do espaço é colorido, então o render descarta o espaço (o run final
/// em estilo default) e quem junta as linhas tem de devolvê-lo.
#[test]
fn a_soft_wrap_at_a_space_is_one_line_and_a_full_line_that_ended_is_not_joined() {
    let _serial = crate::serialize_executables();
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut core = core_with_empty_search_path("terminal-lines");
    core.enable_lsp(sender);
    let dir = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-terminal-lines", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    std::fs::create_dir_all(&dir).unwrap();
    let opened = core.handle_request(&JsonRpcRequest::new(
        1_i64,
        "workspace.open",
        Some(json!({ "path": dir })),
    ));
    assert!(opened.response().error.is_none());

    let before = "a".repeat(79);
    let full = "f".repeat(80);
    let command = format!("printf '\\033[31m%s\\033[0m %s\\n%s\\n%s\\n' {before} b {full} c");
    let started = core.handle_request(&JsonRpcRequest::new(
        2_i64,
        "run.start",
        Some(json!({ "command": command })),
    ));
    let result = started.response().result.clone().unwrap();
    let id = result["terminalId"].as_str().unwrap().to_owned();
    let (lines, exit_code) = terminal_run_until_closed(&receiver, &id, Duration::from_secs(10));
    assert_eq!(exit_code, Some(0), "{lines:?}");
    let printed: Vec<&String> = lines
        .iter()
        .skip_while(|line| !line.starts_with('a'))
        .take(3)
        .collect();
    assert_eq!(printed, [&format!("{before} b"), &full, &"c".to_owned()]);
}
