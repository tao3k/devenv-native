use anyhow::Result;
use std::process::Command;

use tempfile::TempDir;

mod directory_style;
mod json_output;
mod obsidian;
mod syntax;

pub(super) fn run_markdown_lint(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    run_markdown_lint_with_output(temp, scope, None)
}

pub(super) fn run_markdown_lint_with_output(
    temp: &TempDir,
    scope: Option<&str>,
    output: Option<&str>,
) -> Result<(Option<i32>, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    if let Some(output) = output {
        command.arg("--output").arg(output);
    }
    command.arg("lint").arg("markdown");
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((output.status.code(), stdout))
}
