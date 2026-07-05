//! Error type for the CLI helper.

use std::{error::Error, fmt, io};

/// Error produced while building or writing a CLI request.
#[derive(Debug)]
pub enum CliError {
    /// The request could not be serialized to JSON.
    Serialize(serde_json::Error),
    /// Writing to stdout or stderr failed.
    Write(io::Error),
    /// The command or subcommand is not recognized.
    UnknownCommand(String),
    /// A required positional argument was missing; holds the usage hint.
    MissingArgument(&'static str),
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "failed to serialize request: {error}"),
            Self::Write(error) => write!(formatter, "failed to write output: {error}"),
            Self::UnknownCommand(command) => write!(formatter, "unknown command: {command}"),
            Self::MissingArgument(usage) => write!(formatter, "missing argument: {usage}"),
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Serialize(error) => Some(error),
            Self::Write(error) => Some(error),
            Self::UnknownCommand(_) | Self::MissingArgument(_) => None,
        }
    }
}
