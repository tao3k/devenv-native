//! Runtime dispatch for semantic SSOT commands.

use super::{
    SemanticCheckReadModelSnapshotArgs, SemanticCommand, SemanticDescribeReadModelArgs,
    SemanticPlanReadModelMaterializationArgs, SemanticReadModelQueryArgs,
    SemanticRefreshProjectionsArgs, SemanticSnapshotReadModelArgs,
};
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
use xiuxian_wendao_sql::semantic_read_model::{
    SemanticReadModelCatalog, SemanticReadModelMaterializationPlan,
    SemanticReadModelMaterializationStatus, SemanticReadModelSnapshot,
    SemanticReadModelSnapshotCheck, query_semantic_read_model_payload,
    semantic_read_model_catalog_from_root, semantic_read_model_materialization_plan_from_root,
    semantic_read_model_snapshot_check_from_root, semantic_read_model_snapshot_from_root,
};
use xiuxian_wendao_sql::{SqlBatchPayload, SqlQueryPayload};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelCatalogReport {
    root: PathBuf,
    catalog: SemanticReadModelCatalog,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelQueryReport {
    root: PathBuf,
    query: String,
    advisory: bool,
    authority: String,
    payload: SqlQueryPayload,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelSnapshotReport {
    root: PathBuf,
    snapshot: SemanticReadModelSnapshot,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelSnapshotCheckReport {
    root: PathBuf,
    check: SemanticReadModelSnapshotCheck,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticReadModelMaterializationPlanReport {
    root: PathBuf,
    plan: SemanticReadModelMaterializationPlan,
}

pub(crate) fn run_command(
    command: &SemanticCommand,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    match command {
        SemanticCommand::CheckReadModelSnapshot(args) => {
            run_check_read_model_snapshot(args, context)
        }
        SemanticCommand::DescribeReadModel(args) => run_describe_read_model(args, context),
        SemanticCommand::PlanReadModelMaterialization(args) => {
            run_plan_read_model_materialization(args, context)
        }
        SemanticCommand::QueryReadModel(args) => run_query_read_model(args, context),
        SemanticCommand::RefreshProjections(args) => run_refresh_projections_worker(args, context),
        SemanticCommand::SnapshotReadModel(args) => run_snapshot_read_model(args, context),
    }
}

fn run_plan_read_model_materialization(
    args: &SemanticPlanReadModelMaterializationArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_root(args.path.as_ref(), context.root());
    let plan = semantic_read_model_materialization_plan_from_root(
        root.as_path(),
        args.expected_snapshot_revision.as_deref(),
    )
    .map_err(anyhow::Error::msg)
    .with_context(|| {
        format!(
            "failed to plan semantic read-model materialization under `{}`",
            root.display()
        )
    })?;
    let is_blocked = plan.status == SemanticReadModelMaterializationStatus::Blocked;
    let report = SemanticReadModelMaterializationPlanReport {
        root: display_semantic_root(root.as_path(), context.root()),
        plan,
    };
    emit_read_model_materialization_plan_report(&report, context.output())?;
    Ok(if is_blocked {
        CommandOutcome::failure(1)
    } else {
        CommandOutcome::success()
    })
}

fn run_check_read_model_snapshot(
    args: &SemanticCheckReadModelSnapshotArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_root(args.path.as_ref(), context.root());
    let check = semantic_read_model_snapshot_check_from_root(
        root.as_path(),
        args.expected_snapshot_revision.as_str(),
    )
    .map_err(anyhow::Error::msg)
    .with_context(|| {
        format!(
            "failed to check semantic read-model snapshot under `{}`",
            root.display()
        )
    })?;
    let matches = check.matches;
    let report = SemanticReadModelSnapshotCheckReport {
        root: display_semantic_root(root.as_path(), context.root()),
        check,
    };
    emit_read_model_snapshot_check_report(&report, context.output())?;
    Ok(if matches {
        CommandOutcome::success()
    } else {
        CommandOutcome::failure(1)
    })
}

fn run_describe_read_model(
    args: &SemanticDescribeReadModelArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_root(args.path.as_ref(), context.root());
    let catalog = semantic_read_model_catalog_from_root(root.as_path())
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "failed to describe semantic read model under `{}`",
                root.display()
            )
        })?;
    let report = SemanticReadModelCatalogReport {
        root: display_semantic_root(root.as_path(), context.root()),
        catalog,
    };
    emit_read_model_catalog_report(&report, context.output())?;
    Ok(CommandOutcome::success())
}

fn run_query_read_model(
    args: &SemanticReadModelQueryArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_root(args.path.as_ref(), context.root());
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

fn run_snapshot_read_model(
    args: &SemanticSnapshotReadModelArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let root = semantic_root(args.path.as_ref(), context.root());
    let snapshot = semantic_read_model_snapshot_from_root(root.as_path())
        .map_err(anyhow::Error::msg)
        .with_context(|| {
            format!(
                "failed to snapshot semantic read model under `{}`",
                root.display()
            )
        })?;
    let report = SemanticReadModelSnapshotReport {
        root: display_semantic_root(root.as_path(), context.root()),
        snapshot,
    };
    emit_read_model_snapshot_report(&report, context.output())?;
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

fn semantic_root(path: Option<&PathBuf>, context_root: &Path) -> PathBuf {
    path.map_or_else(
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

fn emit_read_model_catalog_report(
    report: &SemanticReadModelCatalogReport,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_read_model_catalog_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
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

fn emit_read_model_snapshot_report(
    report: &SemanticReadModelSnapshotReport,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_read_model_snapshot_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn emit_read_model_snapshot_check_report(
    report: &SemanticReadModelSnapshotCheckReport,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_read_model_snapshot_check_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn emit_read_model_materialization_plan_report(
    report: &SemanticReadModelMaterializationPlanReport,
    output: OutputFormat,
) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => render_read_model_materialization_plan_text_report(report),
        OutputFormat::Json => render_json_report(report, false)?,
        OutputFormat::Pretty => render_json_report(report, true)?,
    };
    print!("{rendered}");
    Ok(())
}

fn render_read_model_catalog_text_report(report: &SemanticReadModelCatalogReport) -> String {
    let mut rendered = format!(
        "Semantic read-model catalog: {} table(s), {} row(s) from {}.\n",
        report.catalog.table_count,
        report.catalog.total_row_count,
        report.root.display()
    );
    rendered.push_str("- authority: ");
    rendered.push_str(report.catalog.authority.as_str());
    rendered.push('\n');
    for table in &report.catalog.tables {
        rendered.push_str("- ");
        rendered.push_str(table.name.as_str());
        rendered.push_str(": ");
        rendered.push_str(table.row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(table.column_count.to_string().as_str());
        rendered.push_str(" column(s)\n");
        for column in &table.columns {
            rendered.push_str("  - ");
            rendered.push_str(column.name.as_str());
            rendered.push_str(": ");
            rendered.push_str(column.data_type.as_str());
            rendered.push(' ');
            rendered.push_str(if column.nullable {
                "nullable"
            } else {
                "not null"
            });
            rendered.push('\n');
        }
    }
    rendered
}

fn render_read_model_materialization_plan_text_report(
    report: &SemanticReadModelMaterializationPlanReport,
) -> String {
    let mut rendered = format!(
        "Semantic read-model materialization plan {}: {} {} from {}.\n",
        report.plan.status.as_str(),
        report.plan.target_engine,
        report.plan.refresh_discipline,
        report.root.display()
    );
    rendered.push_str("- snapshot: ");
    rendered.push_str(report.plan.snapshot_revision.as_str());
    if let Some(expected) = report.plan.expected_snapshot_revision.as_deref() {
        rendered.push_str("\n- expected: ");
        rendered.push_str(expected);
        rendered.push_str(if report.plan.snapshot_matches_expected == Some(true) {
            " (matched)"
        } else {
            " (mismatch)"
        });
    }
    rendered.push_str("\n- authority: ");
    rendered.push_str(report.plan.authority.as_str());
    rendered.push_str("\n- writeback: ");
    rendered.push_str(report.plan.writeback_policy.as_str());
    rendered.push_str("\n- tables: ");
    rendered.push_str(report.plan.tables.len().to_string().as_str());
    rendered.push_str(" planned table(s)\n");
    for table in &report.plan.tables {
        rendered.push_str("- ");
        rendered.push_str(table.name.as_str());
        rendered.push_str(": ");
        rendered.push_str(table.row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(table.column_count.to_string().as_str());
        rendered.push_str(" column(s), ");
        rendered.push_str(table.planned_materialization_state.as_str());
        rendered.push_str(" via ");
        rendered.push_str(table.planned_registration_strategy.as_str());
        rendered.push_str(", revision ");
        rendered.push_str(table.row_revision.as_str());
        rendered.push('\n');
    }
    rendered.push_str("- steps: ");
    rendered.push_str(report.plan.required_steps.join(", ").as_str());
    rendered.push('\n');
    rendered
}

fn render_read_model_snapshot_check_text_report(
    report: &SemanticReadModelSnapshotCheckReport,
) -> String {
    let status = if report.check.matches {
        "passed"
    } else {
        "failed"
    };
    let mut rendered = format!(
        "Semantic read-model snapshot check {status}: {} from {}.\n",
        report.check.current_snapshot_revision,
        report.root.display()
    );
    rendered.push_str("- expected: ");
    rendered.push_str(report.check.expected_snapshot_revision.as_str());
    rendered.push_str("\n- current: ");
    rendered.push_str(report.check.current_snapshot_revision.as_str());
    rendered.push_str("\n- authority: ");
    rendered.push_str(report.check.current_snapshot.authority.as_str());
    rendered.push_str("\n- tables: ");
    rendered.push_str(
        report
            .check
            .current_snapshot
            .catalog
            .table_count
            .to_string()
            .as_str(),
    );
    rendered.push_str(" table(s), ");
    rendered.push_str(
        report
            .check
            .current_snapshot
            .catalog
            .total_row_count
            .to_string()
            .as_str(),
    );
    rendered.push_str(" row(s)\n");
    for table in &report.check.current_snapshot.tables {
        rendered.push_str("- ");
        rendered.push_str(table.name.as_str());
        rendered.push_str(": ");
        rendered.push_str(table.row_count.to_string().as_str());
        rendered.push_str(" row(s), revision ");
        rendered.push_str(table.row_revision.as_str());
        rendered.push('\n');
    }
    rendered
}

fn render_read_model_snapshot_text_report(report: &SemanticReadModelSnapshotReport) -> String {
    let mut rendered = format!(
        "Semantic read-model snapshot: {} from {}.\n",
        report.snapshot.snapshot_revision,
        report.root.display()
    );
    rendered.push_str("- authority: ");
    rendered.push_str(report.snapshot.authority.as_str());
    rendered.push_str("\n- tables: ");
    rendered.push_str(report.snapshot.catalog.table_count.to_string().as_str());
    rendered.push_str(" table(s), ");
    rendered.push_str(report.snapshot.catalog.total_row_count.to_string().as_str());
    rendered.push_str(" row(s)\n");
    for table in &report.snapshot.tables {
        rendered.push_str("- ");
        rendered.push_str(table.name.as_str());
        rendered.push_str(": ");
        rendered.push_str(table.row_count.to_string().as_str());
        rendered.push_str(" row(s), ");
        rendered.push_str(table.column_count.to_string().as_str());
        rendered.push_str(" column(s), revision ");
        rendered.push_str(table.row_revision.as_str());
        rendered.push('\n');
    }
    rendered
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
    .context("failed to serialize semantic report")?;
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
