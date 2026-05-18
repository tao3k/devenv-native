use anyhow::Result;
use std::process::Command;

use tempfile::TempDir;

mod directory_style;
mod fragments;
mod json_output;
mod obsidian;
#[cfg(feature = "semantic-sql")]
mod semantic;
mod syntax;
mod targets;
mod text_output;

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

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_lint(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    run_semantic_lint_with_args(temp, scope, &[])
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_lint_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("lint").arg("semantic");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((output.status.code(), stdout))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_refresh_projections(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    run_semantic_refresh_projections_with_args(temp, scope, &[])
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_refresh_projections_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    let (status, stdout, _) =
        run_semantic_refresh_projections_with_args_and_stderr(temp, scope, args)?;
    Ok((status, stdout))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_refresh_projections_with_args_and_stderr(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("semantic").arg("refresh-projections");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    Ok((output.status.code(), stdout, stderr))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_describe_read_model(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("semantic").arg("describe-read-model");
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((output.status.code(), stdout))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_snapshot_read_model(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("semantic").arg("snapshot-read-model");
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok((output.status.code(), stdout))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_check_read_model_snapshot_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("semantic").arg("check-read-model-snapshot");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    Ok((output.status.code(), stdout, stderr))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_plan_read_model_materialization_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command
        .arg("semantic")
        .arg("plan-read-model-materialization");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    Ok((output.status.code(), stdout, stderr))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_preflight_read_model_materialization_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command
        .arg("semantic")
        .arg("preflight-read-model-materialization");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    Ok((output.status.code(), stdout, stderr))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_query_read_model_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    let (status, stdout, _) =
        run_semantic_query_read_model_with_args_and_stderr(temp, scope, args)?;
    Ok((status, stdout))
}

#[cfg(feature = "semantic-sql")]
pub(super) fn run_semantic_query_read_model_with_args_and_stderr(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wendao-client"));
    command.arg("--root").arg(temp.path());
    command.arg("semantic").arg("query-read-model");
    for arg in args {
        command.arg(arg);
    }
    if let Some(scope) = scope {
        command.arg(scope);
    }

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    Ok((output.status.code(), stdout, stderr))
}

pub(super) fn assert_lint_text_snapshot(name: &str, output: &str) {
    insta::with_settings!({
        snapshot_path => "../../snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, output);
    });
}
