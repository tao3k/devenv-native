//! CLI command flows for Orgize read-model commands.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{
    OrgizeAgentTaskReadModelRequest, OrgizeAgentTaskRow, collect_agent_task_rows,
};

use crate::orgize::{
    OrgizeReadModelArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs,
};
use crate::{ClientContext, CommandOutcome, OutputFormat};

use super::archive::{ArchiveApplyReport, apply_archive_plan};
use super::filter::{
    filter_archive_rows, filter_report_rows, filter_task_rows, task_row_has_tag,
    task_row_is_archive_candidate, task_row_is_closure_needed, task_row_is_repeating,
};
use super::json::{
    ArchiveApplyJsonContext, TaskListJsonContext, TaskReportCounts, emit_task_archive_apply_json,
    emit_task_archive_plan_json, emit_task_list_json, emit_task_report_json,
};
use super::model::{
    AGENT_ORG_TASKS_TABLE, AgentOrgReadModelMaterializationReport, ResolvedReadModelSettings,
    TaskQuerySnapshot,
};
use super::render::{
    render_archive_plan_row, render_archive_target_summary, render_report_section,
    render_tag_counts, render_task_list_row,
};
use super::row_view::{display_source_path, task_list_view_label};
use super::settings::{resolve_read_model_settings, resolve_source_paths};
use super::store::{
    open_read_model_connection, open_read_model_read_only_connection,
    query_active_agent_org_task_row_window, query_agent_org_task_rows,
    refresh_agent_org_read_model,
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
    if let Some(outcome) = try_run_cached_active_task_list_fast_path(args, context)? {
        return Ok(outcome);
    }

    let snapshot = open_task_query_snapshot(&args.paths, context, args.cached)?;
    let filtered = filter_task_rows(&snapshot.rows, args);
    let limit = args.limit;
    let shown = filtered.iter().take(limit).copied().collect::<Vec<_>>();
    let active = filtered
        .iter()
        .filter(|row| !row.is_done && !row.archived)
        .count();
    let done = filtered.iter().filter(|row| row.is_done).count();
    let archived = filtered.iter().filter(|row| row.archived).count();

    if context.output() != OutputFormat::Text {
        emit_task_list_json(&TaskListJsonContext {
            args,
            snapshot: &snapshot,
            rows: filtered.len(),
            showing: shown.len(),
            active,
            done,
            archived,
            shown: &shown,
            context,
        })?;
        return Ok(CommandOutcome::success());
    }

    println!("orgize agent task-list");
    println!("backend: duckdb");
    if let Some(view) = args.view {
        println!("view: {}", task_list_view_label(view));
    }
    render_snapshot_header(&snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("rows: {}", filtered.len());
    println!("showing: {}", shown.len());
    println!("active: {active}");
    println!("done: {done}");
    println!("archived: {archived}");
    for (index, row) in shown.iter().enumerate() {
        render_task_list_row(index + 1, row, context);
    }
    Ok(CommandOutcome::success())
}

fn try_run_cached_active_task_list_fast_path(
    args: &OrgizeTaskListArgs,
    context: &ClientContext,
) -> Result<Option<CommandOutcome>> {
    if !can_use_cached_active_task_list_fast_path(args) {
        return Ok(None);
    }
    let settings = resolve_read_model_settings(context)?;
    let Some(connection) = open_read_model_read_only_connection(&settings)? else {
        return Ok(None);
    };
    let source_paths = resolve_source_paths(&args.paths, context, settings.cache_home.as_path());
    let window = query_active_agent_org_task_row_window(&connection, &source_paths, args.limit)?;
    let active = window.total_rows;
    let rows = window.rows;
    let snapshot = TaskQuerySnapshot {
        settings,
        source_paths,
        materialized: None,
        snapshot_label: "cached",
        refresh_warning: None,
        rows: rows.clone(),
    };
    let shown = rows.iter().collect::<Vec<_>>();

    if context.output() != OutputFormat::Text {
        emit_task_list_json(&TaskListJsonContext {
            args,
            snapshot: &snapshot,
            rows: active,
            showing: shown.len(),
            active,
            done: 0,
            archived: 0,
            shown: &shown,
            context,
        })?;
        return Ok(Some(CommandOutcome::success()));
    }

    println!("orgize agent task-list");
    println!("backend: duckdb");
    if let Some(view) = args.view {
        println!("view: {}", task_list_view_label(view));
    }
    render_snapshot_header(&snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    println!("rows: {active}");
    println!("showing: {}", shown.len());
    println!("active: {active}");
    println!("done: 0");
    println!("archived: 0");
    for (index, row) in shown.iter().enumerate() {
        render_task_list_row(index + 1, row, context);
    }
    Ok(Some(CommandOutcome::success()))
}

fn can_use_cached_active_task_list_fast_path(args: &OrgizeTaskListArgs) -> bool {
    args.cached
        && args.text.is_none()
        && args.tags.is_empty()
        && !args.include_done
        && !args.include_archived
        && args
            .view
            .is_none_or(|view| view == crate::orgize::OrgizeTaskListView::Active)
}

pub(crate) fn run_task_report(
    args: &OrgizeTaskReportArgs,
    context: &ClientContext,
) -> Result<CommandOutcome> {
    let snapshot = open_task_query_snapshot(&args.paths, context, args.cached)?;
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

    if context.output() != OutputFormat::Text {
        let counts = TaskReportCounts {
            rows: filtered.len(),
            active: active_rows.len(),
            done: done_rows.len(),
            archived: archived_rows.len(),
            achievements: achievements.len(),
            archive_candidates: archive_candidates.len(),
            repeating: repeating_rows.len(),
            closure_needed: closure_needed_rows.len(),
        };
        emit_task_report_json(args, &snapshot, &counts, &filtered, context.output())?;
        return Ok(CommandOutcome::success());
    }

    println!("orgize agent task-report");
    println!("backend: duckdb");
    if let Some(view) = args.view {
        println!("view: {}", task_list_view_label(view));
    }
    if args.summary_only {
        println!("summary-only: true");
    }
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
    if args.summary_only {
        return Ok(CommandOutcome::success());
    }
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
        open_task_query_snapshot(&args.paths, context, false)?
    };
    let selection = select_archive_rows(&snapshot, args, context)?;
    ensure_archive_selected_count(args, selection.selected.len())?;
    if context.output() != OutputFormat::Text {
        render_task_archive_json(args, &snapshot, &selection, context)?;
        return Ok(CommandOutcome::success());
    }

    render_task_archive_text(args, &snapshot, &selection, context)?;
    Ok(CommandOutcome::success())
}

struct TaskArchiveSelection<'a> {
    candidates: Vec<&'a super::model::AgentOrgTaskListRow>,
    selected: Vec<&'a super::model::AgentOrgTaskListRow>,
}

fn select_archive_rows<'a>(
    snapshot: &'a TaskQuerySnapshot,
    args: &OrgizeTaskArchiveArgs,
    context: &ClientContext,
) -> Result<TaskArchiveSelection<'a>> {
    let target = args.target.as_ref().map(|target| target.to_lowercase());
    let closed_before = args
        .closed_before
        .as_deref()
        .map(parse_iso_date_filter)
        .transpose()?;
    let candidates = filter_archive_rows(&snapshot.rows, args)
        .into_iter()
        .filter(|row| {
            target.as_ref().is_none_or(|target| {
                super::archive::archive_target_for_row(row, &snapshot.settings, context)
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(target)
            })
        })
        .filter(|row| {
            closed_before.is_none_or(|closed_before| {
                row.closed
                    .as_deref()
                    .and_then(timestamp_iso_date)
                    .is_some_and(|closed| closed < closed_before)
            })
        })
        .collect::<Vec<_>>();
    let selected = candidates
        .iter()
        .take(args.limit)
        .copied()
        .collect::<Vec<_>>();
    Ok(TaskArchiveSelection {
        candidates,
        selected,
    })
}

fn ensure_archive_selected_count(args: &OrgizeTaskArchiveArgs, selected: usize) -> Result<()> {
    if let Some(expected) = args.expect_selected {
        anyhow::ensure!(
            selected == expected,
            "archive selected row count mismatch: expected {expected}, selected {selected}",
        );
    }
    Ok(())
}

fn render_task_archive_json(
    args: &OrgizeTaskArchiveArgs,
    snapshot: &TaskQuerySnapshot,
    selection: &TaskArchiveSelection<'_>,
    context: &ClientContext,
) -> Result<()> {
    if args.apply {
        let apply_report = apply_archive_plan(&selection.selected, &snapshot.settings, context)?;
        let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
        emit_task_archive_apply_json(&ArchiveApplyJsonContext {
            args,
            snapshot,
            candidates: selection.candidates.len(),
            selected: &selection.selected,
            apply_report: &apply_report,
            post_apply: &refreshed.materialized,
            output: context.output(),
            context,
        })?;
    } else {
        emit_task_archive_plan_json(
            args,
            snapshot,
            selection.candidates.len(),
            &selection.selected,
            context.output(),
            context,
        )?;
    }
    Ok(())
}

fn render_task_archive_text(
    args: &OrgizeTaskArchiveArgs,
    snapshot: &TaskQuerySnapshot,
    selection: &TaskArchiveSelection<'_>,
    context: &ClientContext,
) -> Result<()> {
    println!("orgize agent task-archive");
    println!("backend: duckdb");
    println!("mode: {}", if args.apply { "apply" } else { "plan" });
    render_snapshot_header(snapshot);
    println!("table: {AGENT_ORG_TASKS_TABLE}");
    if let Some(target) = args.target.as_deref() {
        println!("target-filter: {target}");
    }
    if let Some(closed_before) = args.closed_before.as_deref() {
        println!("closed-before: {closed_before}");
    }
    if let Some(expected) = args.expect_selected {
        println!("expect-selected: {expected}");
    }
    println!("candidates: {}", selection.candidates.len());
    println!("selected: {}", selection.selected.len());
    render_archive_target_summary(&selection.selected, &snapshot.settings, context);
    for (index, row) in selection.selected.iter().enumerate() {
        render_archive_plan_row(index + 1, row, &snapshot.settings, context);
    }

    if args.apply {
        let apply_report = apply_archive_plan(&selection.selected, &snapshot.settings, context)?;
        render_apply_report(&apply_report, context);
        let refreshed = refresh_agent_org_read_model(&args.paths, context)?;
        println!("post-apply-refresh: refreshed");
        println!("post-apply-rows: {}", refreshed.materialized.rows);
        println!("post-apply-active: {}", refreshed.materialized.active_rows);
        println!("post-apply-done: {}", refreshed.materialized.done_rows);
        println!(
            "post-apply-archived: {}",
            refreshed.materialized.archived_rows
        );
    }

    Ok(())
}

fn render_apply_report(report: &ArchiveApplyReport, context: &ClientContext) {
    println!("applied: {}", report.rows);
    println!("sources-updated: {}", report.sources_updated.len());
    for source in &report.sources_updated {
        println!(
            "- source: {}",
            display_source_path(source.to_string_lossy().as_ref(), context)
        );
    }
    println!("targets-updated: {}", report.targets_updated.len());
    for target in &report.targets_updated {
        println!(
            "- target: {}",
            display_source_path(target.to_string_lossy().as_ref(), context)
        );
    }
}

fn parse_iso_date_filter(value: &str) -> Result<&str> {
    let valid = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(index, value)| index == 4 || index == 7 || value.is_ascii_digit());
    anyhow::ensure!(
        valid,
        "closed-before must be formatted as YYYY-MM-DD, got `{value}`"
    );
    Ok(value)
}

fn timestamp_iso_date(value: &str) -> Option<&str> {
    let start = value.find(|candidate: char| candidate.is_ascii_digit())?;
    let date = value.get(start..start + 10)?;
    parse_iso_date_filter(date)
        .with_context(|| "timestamp date is not an ISO date")
        .ok()
}

fn open_task_query_snapshot(
    paths: &[PathBuf],
    context: &ClientContext,
    cached: bool,
) -> Result<TaskQuerySnapshot> {
    if cached && let Some(snapshot) = open_cached_task_query_snapshot(paths, context)? {
        return Ok(snapshot);
    }

    match open_fresh_task_query_snapshot(paths, context) {
        Ok(snapshot) => Ok(snapshot),
        Err(error) if is_duckdb_lock_error(&error) => {
            open_read_only_snapshot_after_refresh_lock(paths, context, error_chain_message(&error))
        }
        Err(error) => Err(error),
    }
}

fn open_cached_task_query_snapshot(
    paths: &[PathBuf],
    context: &ClientContext,
) -> Result<Option<TaskQuerySnapshot>> {
    let settings = resolve_read_model_settings(context)?;
    let Some(connection) = open_read_model_read_only_connection(&settings)? else {
        return Ok(None);
    };
    let source_paths = resolve_source_paths(paths, context, settings.cache_home.as_path());
    let rows = filter_snapshot_rows_by_source_paths(
        query_agent_org_task_rows(&connection)?,
        &source_paths,
    );
    Ok(Some(TaskQuerySnapshot {
        settings,
        source_paths,
        materialized: None,
        snapshot_label: "cached",
        refresh_warning: None,
        rows,
    }))
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
        let rows = filter_snapshot_rows_by_source_paths(
            query_agent_org_task_rows(&connection)?,
            &source_paths,
        );
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

fn filter_snapshot_rows_by_source_paths(
    rows: Vec<super::model::AgentOrgTaskListRow>,
    source_paths: &[PathBuf],
) -> Vec<super::model::AgentOrgTaskListRow> {
    rows.into_iter()
        .filter(|row| {
            source_paths
                .iter()
                .any(|source_path| row_matches_source_path(row.source_path.as_str(), source_path))
        })
        .collect()
}

fn row_matches_source_path(row_source_path: &str, source_path: &Path) -> bool {
    let row_source_path = Path::new(row_source_path);
    if source_path.is_dir() {
        row_source_path.starts_with(source_path)
    } else {
        row_source_path == source_path
    }
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
