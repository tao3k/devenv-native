//! CLI command flows for Orgize read-model commands.

use std::path::PathBuf;

use anyhow::Result;
use xiuxian_wendao_parsers::{
    OrgizeAgentTaskReadModelRequest, OrgizeAgentTaskRow, collect_agent_task_rows,
};

use crate::orgize::{
    OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs,
};
use crate::{ClientContext, CommandOutcome};

use super::archive::apply_archive_plan;
use super::filter::{
    filter_archive_rows, filter_report_rows, filter_task_rows, task_row_has_tag,
    task_row_is_archive_candidate, task_row_is_closure_needed, task_row_is_repeating,
};
use super::model::{
    AGENT_ORG_TASKS_TABLE, AgentOrgReadModelMaterializationReport, ResolvedReadModelSettings,
};
use super::render::{
    render_archive_plan_row, render_report_section, render_tag_counts, render_task_list_row,
};
use super::settings::{resolve_read_model_settings, resolve_source_paths};
use super::store::{
    open_read_model_connection, open_read_model_read_only_connection, query_agent_org_task_rows,
    refresh_agent_org_read_model,
};

struct TaskQuerySnapshot {
    settings: ResolvedReadModelSettings,
    source_paths: Vec<PathBuf>,
    materialized: Option<AgentOrgReadModelMaterializationReport>,
    snapshot_label: &'static str,
    refresh_warning: Option<String>,
    rows: Vec<super::model::AgentOrgTaskListRow>,
}

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
    let snapshot = open_task_query_snapshot(&args.paths, context)?;
    let filtered = filter_task_rows(&snapshot.rows, args);
    let limit = args.limit;
    let shown = filtered.iter().take(limit).collect::<Vec<_>>();

    println!("orgize agent task-list");
    println!("backend: duckdb");
    render_snapshot_header(&snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
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
    let snapshot = open_task_query_snapshot(&args.paths, context)?;
    let filtered = filter_report_rows(&snapshot.rows, args);
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
        .filter(|row| task_row_is_archive_candidate(row))
        .copied()
        .collect::<Vec<_>>();
    let achievements = filtered
        .iter()
        .filter(|row| task_row_has_tag(row, "achievement"))
        .copied()
        .collect::<Vec<_>>();
    let repeating_rows = filtered
        .iter()
        .filter(|row| task_row_is_repeating(row))
        .copied()
        .collect::<Vec<_>>();
    let closure_needed_rows = filtered
        .iter()
        .filter(|row| task_row_is_closure_needed(row))
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-report");
    println!("backend: duckdb");
    render_snapshot_header(&snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("rows: {}", filtered.len());
    println!("active: {}", active_rows.len());
    println!("done: {}", done_rows.len());
    println!("archived: {}", archived_rows.len());
    println!("achievements: {}", achievements.len());
    println!("archive-candidates: {}", archive_candidates.len());
    println!("repeating: {}", repeating_rows.len());
    println!("closure-needed: {}", closure_needed_rows.len());
    render_tag_counts(&filtered);
    render_report_section("Closure Needed", &closure_needed_rows, limit, context);
    render_report_section("Archive Candidates", &archive_candidates, limit, context);
    render_report_section("Achievements", &achievements, limit, context);
    render_report_section("Repeating Tasks", &repeating_rows, limit, context);
    Ok(CommandOutcome::success())
}

pub(crate) fn run_task_archive(
    args: &OrgizeTaskArchiveArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let snapshot = if args.apply {
        open_fresh_task_query_snapshot(&args.paths, context)?
    } else {
        open_task_query_snapshot(&args.paths, context)?
    };
    let candidates = filter_archive_rows(&snapshot.rows, args);
    let selected = candidates
        .iter()
        .take(args.limit)
        .copied()
        .collect::<Vec<_>>();

    println!("orgize agent task-archive");
    println!("backend: duckdb");
    println!("mode: {}", if args.apply { "apply" } else { "plan" });
    render_snapshot_header(&snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("candidates: {}", candidates.len());
    println!("selected: {}", selected.len());
    for (index, row) in selected.iter().enumerate() {
        render_archive_plan_row(index + 1, row, &snapshot.settings, context);
    }

    if args.apply {
        apply_archive_plan(&selected, &snapshot.settings, context)?;
        println!("applied: {}", selected.len());
    }

    Ok(CommandOutcome::success())
}

fn open_task_query_snapshot(
    paths: &[PathBuf],
    context: &ClientContext,
) -> Result<TaskQuerySnapshot> {
    match open_fresh_task_query_snapshot(paths, context) {
        Ok(snapshot) => Ok(snapshot),
        Err(error) if is_duckdb_lock_error(&error) => {
            open_read_only_snapshot_after_refresh_lock(paths, context, error_chain_message(&error))
        }
        Err(error) => Err(error),
    }
}

fn open_fresh_task_query_snapshot(
    paths: &[PathBuf],
    context: &ClientContext,
) -> Result<TaskQuerySnapshot> {
    let refreshed = refresh_agent_org_read_model(paths, context)?;
    let connection = open_snapshot_connection(&refreshed.settings)?;
    let rows = query_agent_org_task_rows(&connection)?;
    Ok(TaskQuerySnapshot {
        settings: refreshed.settings,
        source_paths: refreshed.source_paths,
        materialized: Some(refreshed.materialized),
        snapshot_label: "refreshed",
        refresh_warning: None,
        rows,
    })
}

fn open_read_only_snapshot_after_refresh_lock(
    paths: &[PathBuf],
    context: &ClientContext,
    refresh_error: String,
) -> Result<TaskQuerySnapshot> {
    let settings = resolve_read_model_settings(context)?;
    let source_paths = resolve_source_paths(paths, context, settings.cache_home.as_path());
    if let Some(connection) = open_read_model_read_only_connection(&settings)
        .ok()
        .flatten()
    {
        let rows = query_agent_org_task_rows(&connection)?;
        return Ok(TaskQuerySnapshot {
            settings,
            source_paths,
            materialized: None,
            snapshot_label: "read-only-fallback",
            refresh_warning: Some(refresh_error),
            rows,
        });
    }

    let report = collect_agent_task_rows(&OrgizeAgentTaskReadModelRequest {
        paths: source_paths.clone(),
        match_expression: Some("+agent".to_string()),
        include_comments: false,
    })?;
    let materialized = materialization_report_from_agent_rows(&report.rows);
    let rows = report
        .rows
        .into_iter()
        .map(agent_task_row_to_list_row)
        .collect();
    Ok(TaskQuerySnapshot {
        settings,
        source_paths,
        materialized: Some(materialized),
        snapshot_label: "in-memory-fallback",
        refresh_warning: Some(refresh_error),
        rows,
    })
}

fn open_snapshot_connection(
    settings: &ResolvedReadModelSettings,
) -> Result<xiuxian_db_store::duckdb_crate::Connection> {
    match open_read_model_read_only_connection(settings)? {
        Some(connection) => Ok(connection),
        None => open_read_model_connection(settings),
    }
}

fn render_snapshot_header(snapshot: &TaskQuerySnapshot) {
    println!("database: {}", snapshot.settings.database_path.display());
    println!("sources: {}", snapshot.source_paths.len());
    println!("snapshot: {}", snapshot.snapshot_label);
    if let Some(reason) = &snapshot.refresh_warning {
        println!("refresh-warning: {}", compact_refresh_warning(reason));
    } else if snapshot.snapshot_label == "refreshed"
        && let Some(materialized) = &snapshot.materialized
    {
        println!("snapshot-rows: {}", materialized.rows);
    }
}

fn is_duckdb_lock_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        let message = cause.to_string();
        message.contains("Could not set lock")
            || message.contains("Conflicting lock")
            || message.contains("open the file in read-only mode")
    })
}

fn error_chain_message(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

fn compact_refresh_warning(reason: &str) -> String {
    reason.lines().next().unwrap_or(reason).to_string()
}

fn materialization_report_from_agent_rows(
    rows: &[OrgizeAgentTaskRow],
) -> AgentOrgReadModelMaterializationReport {
    AgentOrgReadModelMaterializationReport {
        rows: rows.len(),
        active_rows: rows
            .iter()
            .filter(|row| !row.is_done && !row.archived)
            .count(),
        done_rows: rows.iter().filter(|row| row.is_done).count(),
        archived_rows: rows.iter().filter(|row| row.archived).count(),
    }
}

fn agent_task_row_to_list_row(row: OrgizeAgentTaskRow) -> super::model::AgentOrgTaskListRow {
    super::model::AgentOrgTaskListRow {
        source_path: row.source_path,
        source_line: row.source_line,
        source_range_start: row.source_range_start,
        source_range_end: row.source_range_end,
        level: row.level,
        title: row.title,
        todo_state: row.todo_state,
        is_done: row.is_done,
        archived: row.archived,
        tags: row.tags,
        effective_tags: row.effective_tags,
        scheduled: row.scheduled,
        scheduled_repeater: row.scheduled_repeater,
        deadline: row.deadline,
        deadline_repeater: row.deadline_repeater,
        closed: row.closed,
        outline_path: row.outline_path,
        properties: row.properties,
    }
}
