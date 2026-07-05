//! Minimal CLI helper for generating Kernwerk JSON-RPC requests.
//!
//! Each invocation parses argv and emits exactly one JSON-RPC request to
//! stdout, so shell scripts and smoke tests can drive the core without an IPC
//! client. The dispatch and error type live in the `kernwerk_cli` library.

#![forbid(unsafe_code)]

use std::{
    env,
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let stdout = io::stdout();
    let stderr = io::stderr();

    match kernwerk_cli::run(env::args().skip(1), stdout.lock(), stderr.lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut stderr = io::stderr().lock();
            if writeln!(stderr, "{error}").is_err() {
                return ExitCode::FAILURE;
            }
            ExitCode::FAILURE
        }
    }
}
