use crate::{ClientCommand, ClientContext, LintCommand, lint};
use anyhow::Result;

/// Stable process outcome for standalone and embedded command entrypoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandOutcome {
    exit_code: u8,
}

impl CommandOutcome {
    /// Successful command outcome.
    #[must_use]
    pub const fn success() -> Self {
        Self { exit_code: 0 }
    }

    /// Failing command outcome with a stable process exit code.
    #[must_use]
    pub const fn failure(exit_code: u8) -> Self {
        Self { exit_code }
    }

    /// Exit code suitable for `std::process::ExitCode`.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        self.exit_code
    }
}

/// Execute one reusable client command.
pub fn run_command(command: &ClientCommand, context: &ClientContext) -> Result<CommandOutcome> {
    match command {
        ClientCommand::Lint { command } => run_lint_command(command, context),
    }
}

fn run_lint_command(command: &LintCommand, context: &ClientContext) -> Result<CommandOutcome> {
    match command {
        LintCommand::Markdown(args) => lint::run_markdown_lint(args, context),
    }
}
