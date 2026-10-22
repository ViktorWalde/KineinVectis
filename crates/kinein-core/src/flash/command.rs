//! One structured flash command and the shell preview derived from it.
//! This is planning data, never an executor or a parser of saved commands.

use std::path::PathBuf;

use kinein_protocol::FlashProposalResult;

use super::quote;

/// Program and literal arguments selected by the existing engine planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashCommand {
    /// Executable, before any serial-ownership wrapper is applied.
    pub program: PathBuf,
    /// Literal arguments; shell quoting is never stored here.
    pub args: Vec<String>,
}

impl FlashCommand {
    pub(super) fn new(program: impl Into<PathBuf>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    /// POSIX preview, compatible with the current manual run configuration.
    /// Only plain words are unquoted; project values cannot introduce shell syntax.
    #[must_use]
    pub fn preview(&self) -> String {
        std::iter::once(quote(&self.program.display().to_string()))
            .chain(self.args.iter().map(|argument| {
                if !argument.is_empty()
                    && !argument.starts_with('/')
                    && argument
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_+-=.:/,".contains(&byte))
                {
                    argument.clone()
                } else {
                    quote(argument)
                }
            }))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Structured launch and its existing user-facing proposal, produced together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashPlan {
    /// Program and literal arguments, without reconstructing the shell preview.
    pub command: FlashCommand,
    /// Existing protocol payload; no native intent is implied by this preview.
    pub proposal: FlashProposalResult,
}

impl FlashPlan {
    pub(super) fn new(
        name: String,
        engine: String,
        source: Vec<String>,
        warnings: Vec<String>,
        command: FlashCommand,
    ) -> Self {
        let proposal = FlashProposalResult {
            name,
            command: command.preview(),
            engine,
            source,
            warnings,
            flash: None,
        };
        Self { command, proposal }
    }
}

#[cfg(test)]
mod tests {
    use super::FlashCommand;

    #[test]
    fn preview_and_direct_argv_deliver_identical_literal_bytes() {
        let arguments = [
            "plain",
            "",
            "a b",
            "a'b",
            "$(printf expanded)",
            "a;b",
            "a\nb",
            "$HOME",
            "*",
            "#comment",
            "~",
            "\\",
            "unicode-á",
            "/tmp/a b",
        ];
        let directory =
            std::env::temp_dir().join(format!("{}-flash command's literal", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let program = crate::write_executable(
            directory.join("fake-tool"),
            "#!/bin/sh\nprintf '<%s>' \"$@\"\n",
        );
        let command = FlashCommand::new(program, arguments.map(str::to_owned).to_vec());
        let direct = std::process::Command::new(&command.program)
            .args(&command.args)
            .env("MSYS2_ARG_CONV_EXCL", "*")
            .output()
            .unwrap();
        let preview = crate::sh_command(&command.preview())
            .env("MSYS2_ARG_CONV_EXCL", "*")
            .output()
            .unwrap();
        assert!(direct.status.success() && preview.status.success());
        assert_eq!(direct.stdout, preview.stdout);
        assert!(preview.stderr.is_empty());
    }
}
