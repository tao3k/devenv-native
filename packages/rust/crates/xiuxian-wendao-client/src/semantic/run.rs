//! Runtime dispatch for semantic SSOT commands.

use super::{SemanticCommand, SemanticReadModelQueryArgs, SemanticRefreshProjectionsArgs};
use crate::lint::{
    self, SemanticLintArgs, SemanticLintProjectionValidationArgs, SemanticLintValidationArgs,
    SemanticLintWritebackArgs,
};
use crate::{ClientContext, CommandOutcome, OutputFormat};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use xiuxian_wendao_sql::semantic_read_model::query_semantic_read_model_payload;
use xiuxian_wendao_sql::{SqlBatchPayload, SqlQueryPayload};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelQueryReport {
    root: PathBuf,
    query: String,
    advisory: bool,
    authority: String,
    payload: SqlQueryPayload,
}

pub(crate) fn run_command(
    command: &SemanticCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        SemanticCommand::QueryReadModel(args) => run_query_read_model(args, context),
        SemanticCommand::RefreshProjections(args) => run_refresh_projections_worker(args, context),
    }
}

fn run_query_read_model(
    args: &SemanticReadModelQueryArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_read_model_root(args, context.root());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to create semantic read-model query runtime")?;
    let payload = runtime
        .block_on(query_semantic_read_model_payload(
            root.as_path(),
            args.query_text.as_str(),
        ))
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "failed to query semantic read model under `{}`",
                root.display()
            )
        })?;
    let report = SemanticReadModelQueryReport {
        root: display_semantic_root(root.as_path(), context.root()),
        query: args.query_text.clone(),
        advisory: true,
        authority: "repo_native_semantic_artifacts".to_string(),
        payload,
    };
    emit_read_model_query_report(&report, context.output())?;
    Ok(CommandOutcome::success())
}

fn run_refresh_projections_worker(
    args: &SemanticRefreshProjectionsArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    if args.require_clean_worktree {
        ensure_clean_git_worktree(context.root())?;
    }

    let mut completed_runs = 0_usize;
    loop {
        let outcome = run_refresh_projections_worker_pass(args, context)?;
        completed_runs += 1;
        if outcome.exit_code() != 0 {
            return Ok(outcome);
        }
        if args
            .max_runs
            .is_some_and(|max_runs| completed_runs >= max_runs.get())
        {
            return Ok(outcome);
        }
        if args.interval_secs == 0 && args.max_runs.is_none() {
            return Ok(outcome);
        }
        if args.interval_secs > 0 {
            thread::sleep(Duration::from_secs(args.interval_secs));
        }
    }
}

fn run_refresh_projections_worker_pass(
    args: &SemanticRefreshProjectionsArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let lint_args = SemanticLintArgs {
        validation: SemanticLintValidationArgs {
            read_model_summary: false,
            semantic_sql_guard: false,
            lifecycle_plan: false,
            projection: SemanticLintProjectionValidationArgs {
                projection_refresh_plan: true,
                require_fresh_projections: true,
            },
        },
        writeback: SemanticLintWritebackArgs {
            refresh_projections: true,
            apply_lifecycle_plan: false,
        },
        paths: args.paths.clone(),
    };
    lint::run_semantic_lint(&lint_args, context)
}

fn semantic_read_model_root(args: &SemanticReadModelQueryArgs, context_root: &Path) -> PathBuf {
    args.path.as_ref().map_or_else(
        || context_root.join("semantic"),
        |path| {
            if path.is_absolute() {
                path.clone()
            } else {
                context_root.join(path)
            }
        },
    )
}

fn display_semantic_root(root: &Path, context_root: &Path) -> PathBuf {
    root.strip_prefix(context_root)
        .map_or_else(|_| root.to_path_buf(), std::path::Path::to_path_buf)
}

fn emit_read_model_query_report(
    report: &SemanticReadModelQueryReport,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_read_model_query_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn render_read_model_query_text_report(report: &SemanticReadModelQueryReport) -> String {
    let mut rendered = format!(
        "Semantic read-model query returned {} row(s) across {} batch(es) from {} using {}.\n",
        report.payload.metadata.result_row_count,
        report.payload.metadata.result_batch_count,
        report.root.display(),
        report
            .payload
            .metadata
            .local_relation_engine
            .as_deref()
            .unwrap_or("unknown")
    );
    rendered.push_str("- authority: ");
    rendered.push_str(report.authority.as_str());
    rendered.push_str("\n- registered tables: ");
    rendered.push_str(
        report
            .payload
            .metadata
            .registered_tables
            .join(", ")
            .as_str(),
    );
    rendered.push('\n');
    for batch in &report.payload.batches {
        render_read_model_query_batch(batch, &mut rendered);
    }
    rendered
}

fn render_read_model_query_batch(batch: &SqlBatchPayload, rendered: &mut String) {
    let columns = batch
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect::<Vec<_>>();
    if !columns.is_empty() {
        rendered.push_str("- columns: ");
        rendered.push_str(columns.join(", ").as_str());
        rendered.push('\n');
    }
    for row in &batch.rows {
        rendered.push_str("  - ");
        let cells = columns
            .iter()
            .map(|column| {
                let value = row.get(*column).unwrap_or(&Value::Null);
                format!("{column}={}", render_query_value(value))
            })
            .collect::<Vec<_>>();
        rendered.push_str(cells.join(", ").as_str());
        rendered.push('\n');
    }
}

fn render_query_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn render_json_report<T: serde::Serialize>(report: &T, pretty: bool) -> Result<String> {
    let rendered = if pretty {
        serde_json::to_string_pretty(report)
    } else {
        serde_json::to_string(report)
    }
    .context("failed to serialize semantic read-model query report")?;
    Ok(format!("{rendered}\n"))
}

fn ensure_clean_git_worktree(root: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output()
        .with_context(|| {
            format!(
                "failed to run git clean-worktree check at `{}`",
                root.display()
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "semantic refresh supervisor clean-worktree check requires a git worktree at `{}`: {}",
            root.display(),
            stderr.trim()
        );
    }
    if !output.stdout.is_empty() {
        let status = String::from_utf8_lossy(&output.stdout);
        bail!(
            "semantic refresh supervisor clean-worktree check requires a clean git worktree at `{}`; pending changes:\n{}",
            root.display(),
            status.trim_end()
        );
    }
    Ok(())
}
