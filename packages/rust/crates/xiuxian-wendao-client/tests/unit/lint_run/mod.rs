use anyhow::Result;
use std::process::Command;

use tempfile::TempDir;

mod directory_style;
mod obsidian;
mod syntax;

pub(super) fn run_markdown_lint(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao"));
    command
        .arg("--root")
        .arg(temp.path())
        .arg("lint")
        .arg("markdown");
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((output.status.code(), stdout))
}
