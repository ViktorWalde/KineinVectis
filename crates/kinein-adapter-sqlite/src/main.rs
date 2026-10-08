//! O executavel `kinein-adapter-sqlite`: le uma linha JSON por vez do core e
//! escreve as respostas; a logica mora na biblioteca deste crate.

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut adapter = kinein_adapter_sqlite::engine::Adapter::default();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let (messages, finished) = kinein_adapter_sqlite::handle(&mut adapter, &line);
        for message in messages {
            if writeln!(out, "{message}")
                .and_then(|()| out.flush())
                .is_err()
            {
                return;
            }
        }
        if finished {
            return;
        }
    }
}
