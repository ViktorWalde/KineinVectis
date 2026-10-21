//! Executable entrypoint for `kinein-core`.

#![forbid(unsafe_code)]

use std::{
    io::{self, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let outcome = if arguments
        .first()
        .is_some_and(|argument| argument == "--serial-tool")
    {
        kinein_core::run_serial_tool(&arguments[1..])
    } else {
        kinein_core::run_stdio().map_err(io::Error::other)
    };
    match outcome {
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
