//! Executable entrypoint for `kernwerk-core`.

#![forbid(unsafe_code)]

use std::{
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    match kernwerk_core::run_stdio() {
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
