//! CLI command flows for Orgize read-model commands.

use anyhow::Result;

use crate::orgize::{
    OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs,
};
use crate::{ClientContext, CommandOutcome};

use super::archive::apply_archive_plan;
use super::filter::{filter_archive_rows, filter_report_rows, filter_task_rows, task_row_has_tag};
use super::model::AGENT_ORG_TASKS_TABLE;
use super::render::{
    render_archive_plan_row, render_report_section, render_tag_counts, render_task_list_row,
};
use super::store::{
    open_read_model_connection, query_agent_org_task_rows, refresh_agent_org_read_model,
};

pub(crate) fn run_read_model(
    args: &OrgizeReadModelArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;

    println!("orgize agent read-model materialized");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", refreshed.materialized.rows);
    println!("active: {}", refreshed.materialized.active_rows);
    println!("done: {}", refreshed.materialized.done_rows);
    println!("archived: {}", refreshed.materialized.archived_rows);
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_list(
    args: &OrgizeTaskListArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let filtered = filter_task_rows(&rows, args);
    let limit = args.limit;
    let shown = filtered.iter().take(limit).collect::<Vec<_>>();

    println!("orgize agent task-list");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", filtered.len());
    println!("showing: {}", shown.len());
    println!(
        "active: {}",
        filtered
            .iter()
            .filter(|row| !row.is_done && !row.archived)
            .count()
    );
    println!(
        "done: {}",
        filtered.iter().filter(|row| row.is_done).count()
    );
    println!(
        "archived: {}",
        filtered.iter().filter(|row| row.archived).count()
    );
    for (index, row) in shown.iter().enumerate() {
        render_task_list_row(index + 1, row, context);
    }
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_report(
    args: &OrgizeTaskReportArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let filtered = filter_report_rows(&rows, args);
    let limit = args.limit;
    let active_rows = filtered
        .iter()
        .filter(|row| !row.is_done && !row.archived)
        .copied()
        .collect::<Vec<_>>();
    let done_rows = filtered
        .iter()
        .filter(|row| row.is_done)
        .copied()
        .collect::<Vec<_>>();
    let archived_rows = filtered
        .iter()
        .filter(|row| row.archived)
        .copied()
        .collect::<Vec<_>>();
    let archive_candidates = filtered
        .iter()
        .filter(|row| row.is_done && !row.archived)
        .copied()
        .collect::<Vec<_>>();
    let achievements = filtered
        .iter()
        .filter(|row| task_row_has_tag(row, "achievement"))
        .copied()
        .collect::<Vec<_>>();
    let repeating_rows = filtered
        .iter()
        .filter(|row| row.scheduled_repeater.is_some() || row.deadline_repeater.is_some())
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-report");
    println!("backend: duckdb");
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("rows: {}", filtered.len());
    println!("active: {}", active_rows.len());
    println!("done: {}", done_rows.len());
    println!("archived: {}", archived_rows.len());
    println!("achievements: {}", achievements.len());
    println!("archive-candidates: {}", archive_candidates.len());
    println!("repeating: {}", repeating_rows.len());
    render_tag_counts(&filtered);
    render_report_section("Archive Candidates", &archive_candidates, limit, context);
    render_report_section("Achievements", &achievements, limit, context);
    render_report_section("Repeating Tasks", &repeating_rows, limit, context);
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_archive(
    args: &OrgizeTaskArchiveArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
    let connection = open_read_model_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    let candidates = filter_archive_rows(&rows, args);
    let selected = candidates
        .iter()
        .take(args.limit)
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-archive");
    println!("backend: duckdb");
    println!("mode: {}", if args.apply { "apply" } else { "plan" });
    println!("database: {}", refreshed.settings.database_path.display());
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("sources: {}", refreshed.source_paths.len());
    println!("candidates: {}", candidates.len());
    println!("selected: {}", selected.len());
    for (index, row) in selected.iter().enumerate() {
        render_archive_plan_row(index + 1, row, &refreshed.settings, context);
    }

    if args.apply {
        apply_archive_plan(&selected, &refreshed.settings, context)?;
        println!("applied: {}", selected.len());
    }

    Ok(CommandOutcome::success())
}
