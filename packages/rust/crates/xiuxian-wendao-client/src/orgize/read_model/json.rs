//! JSON rendering for Orgize read-model commands.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde_json::json;

use crate::orgize::{
    OrgizeOrgidShowArgs, OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs,
};
use crate::{ClientContext, OutputFormat};

use super::archive::{ArchiveApplyReport, archive_target_for_row};
use super::memory::reflection_memory_objects_for_row;
use super::model::{
    AGENT_ORG_TASKS_TABLE, AgentOrgReadModelMaterializationReport, AgentOrgTaskListRow,
    ResolvedReadModelSettings, TaskQuerySnapshot,
};
use super::row_view::{
    display_source_path, property_value, task_list_view_label, task_repeater_labels,
};

pub(super) struct TaskReportCounts {
    pub(super) rows: usize,
    pub(super) active: usize,
    pub(super) done: usize,
    pub(super) archived: usize,
    pub(super) achievements: usize,
    pub(super) archive_candidates: usize,
    pub(super) repeating: usize,
    pub(super) closure_needed: usize,
}

pub(super) struct TaskListJsonContext<'a> {
    pub(super) args: &'a OrgizeTaskListArgs,
    pub(super) snapshot: &'a TaskQuerySnapshot,
    pub(super) rows: usize,
    pub(super) showing: usize,
    pub(super) active: usize,
    pub(super) done: usize,
    pub(super) archived: usize,
    pub(super) shown: &'a [&'a AgentOrgTaskListRow],
    pub(super) context: &'a ClientContext,
}

pub(super) struct ArchiveApplyJsonContext<'a> {
    pub(super) args: &'a OrgizeTaskArchiveArgs,
    pub(super) snapshot: &'a TaskQuerySnapshot,
    pub(super) candidates: usize,
    pub(super) selected: &'a [&'a AgentOrgTaskListRow],
    pub(super) apply_report: &'a ArchiveApplyReport,
    pub(super) post_apply: &'a AgentOrgReadModelMaterializationReport,
    pub(super) output: OutputFormat,
    pub(super) context: &'a ClientContext,
}

pub(super) fn emit_task_list_json(input: &TaskListJsonContext<'_>) -> Result<()> {
    let report = json!({
        "command": "orgize task-list",
        "backend": "duckdb",
        "table": AGENT_ORG_TASKS_TABLE,
        "view": input.args.view.map(task_list_view_label),
        "database": &input.snapshot.settings.database_path,
        "sources": &input.snapshot.source_paths,
        "snapshot": input.snapshot.snapshot_label,
        "snapshotRows": input.snapshot.materialized.as_ref().map(|materialized| materialized.rows),
        "refreshWarning": &input.snapshot.refresh_warning,
        "rows": input.rows,
        "showing": input.showing,
        "active": input.active,
        "done": input.done,
        "archived": input.archived,
        "tasks": input.shown
            .iter()
            .enumerate()
            .map(|(index, row)| task_list_row_json(index + 1, row, input.context))
            .collect::<Vec<_>>(),
    });
    emit_json_report(&report, input.context.output(), "orgize task-list")
}

pub(super) fn emit_orgid_show_json(
    args: &OrgizeOrgidShowArgs,
    snapshot: &TaskQuerySnapshot,
    row: &AgentOrgTaskListRow,
    section: &str,
    context: &ClientContext,
) -> Result<()> {
    let report = json!({
        "command": "orgize orgid-show",
        "backend": "duckdb",
        "table": AGENT_ORG_TASKS_TABLE,
        "orgid": args.id,
        "database": &snapshot.settings.database_path,
        "sources": &snapshot.source_paths,
        "snapshot": snapshot.snapshot_label,
        "snapshotRows": snapshot.materialized.as_ref().map(|materialized| materialized.rows),
        "refreshWarning": &snapshot.refresh_warning,
        "task": orgid_show_row_json(row, section, context),
    });
    emit_json_report(&report, context.output(), "orgize orgid-show")
}

pub(super) fn emit_task_report_json(
    args: &OrgizeTaskReportArgs,
    snapshot: &TaskQuerySnapshot,
    counts: &TaskReportCounts,
    filtered: &[&AgentOrgTaskListRow],
    output: OutputFormat,
) -> Result<()> {
    let report = json!({
        "command": "orgize task-report",
        "backend": "duckdb",
        "table": AGENT_ORG_TASKS_TABLE,
        "view": args.view.map(task_list_view_label),
        "summaryOnly": args.summary_only,
        "database": &snapshot.settings.database_path,
        "sources": &snapshot.source_paths,
        "snapshot": snapshot.snapshot_label,
        "snapshotRows": snapshot.materialized.as_ref().map(|materialized| materialized.rows),
        "refreshWarning": &snapshot.refresh_warning,
        "rows": counts.rows,
        "active": counts.active,
        "done": counts.done,
        "archived": counts.archived,
        "achievements": counts.achievements,
        "archiveCandidates": counts.archive_candidates,
        "repeating": counts.repeating,
        "closureNeeded": counts.closure_needed,
        "tags": task_report_tag_counts(filtered),
        "sections": {
            "closureNeeded": counts.closure_needed,
            "archiveCandidates": counts.archive_candidates,
            "achievements": counts.achievements,
            "repeating": counts.repeating,
        }
    });
    emit_json_report(&report, output, "orgize task-report")
}

pub(super) fn emit_task_archive_plan_json(
    args: &OrgizeTaskArchiveArgs,
    snapshot: &TaskQuerySnapshot,
    candidates: usize,
    selected: &[&AgentOrgTaskListRow],
    output: OutputFormat,
    context: &ClientContext,
) -> Result<()> {
    let report = json!({
        "command": "orgize task-archive",
        "backend": "duckdb",
        "mode": "plan",
        "table": AGENT_ORG_TASKS_TABLE,
        "database": &snapshot.settings.database_path,
        "sources": &snapshot.source_paths,
        "snapshot": snapshot.snapshot_label,
        "snapshotRows": snapshot.materialized.as_ref().map(|materialized| materialized.rows),
        "refreshWarning": &snapshot.refresh_warning,
        "targetFilter": args.target,
        "closedBefore": args.closed_before,
        "expectSelected": args.expect_selected,
        "limit": args.limit,
        "candidates": candidates,
        "selected": selected.len(),
        "archiveTargets": archive_target_counts_json(selected, &snapshot.settings, context),
        "items": selected
            .iter()
            .enumerate()
            .map(|(index, row)| archive_plan_row_json(index + 1, row, &snapshot.settings, context))
            .collect::<Vec<_>>(),
    });
    emit_json_report(&report, output, "orgize task-archive plan")
}

pub(super) fn emit_task_archive_apply_json(input: &ArchiveApplyJsonContext<'_>) -> Result<()> {
    let report = json!({
        "command": "orgize task-archive",
        "backend": "duckdb",
        "mode": "apply",
        "table": AGENT_ORG_TASKS_TABLE,
        "database": &input.snapshot.settings.database_path,
        "sources": &input.snapshot.source_paths,
        "snapshot": input.snapshot.snapshot_label,
        "snapshotRows": input.snapshot.materialized.as_ref().map(|materialized| materialized.rows),
        "refreshWarning": &input.snapshot.refresh_warning,
        "targetFilter": input.args.target,
        "closedBefore": input.args.closed_before,
        "expectSelected": input.args.expect_selected,
        "limit": input.args.limit,
        "candidates": input.candidates,
        "selected": input.selected.len(),
        "archiveTargets": archive_target_counts_json(input.selected, &input.snapshot.settings, input.context),
        "items": input.selected
            .iter()
            .enumerate()
            .map(|(index, row)| archive_plan_row_json(index + 1, row, &input.snapshot.settings, input.context))
            .collect::<Vec<_>>(),
        "applied": input.apply_report.rows,
        "sourcesUpdated": input.apply_report
            .sources_updated
            .iter()
            .map(|path| display_source_path(path.to_string_lossy().as_ref(), input.context))
            .collect::<Vec<_>>(),
        "targetsUpdated": input.apply_report
            .targets_updated
            .iter()
            .map(|path| display_source_path(path.to_string_lossy().as_ref(), input.context))
            .collect::<Vec<_>>(),
        "postApplyRefresh": "refreshed",
        "postApplyRows": input.post_apply.rows,
        "postApplyActive": input.post_apply.active_rows,
        "postApplyDone": input.post_apply.done_rows,
        "postApplyArchived": input.post_apply.archived_rows,
    });
    emit_json_report(&report, input.output, "orgize task-archive apply")
}

fn emit_json_report(report: &serde_json::Value, output: OutputFormat, command: &str) -> Result<()> {
    let rendered = match output {
        OutputFormat::Text => unreachable!("text output is rendered by the text command path"),
        OutputFormat::Json => serde_json::to_string(report)
            .with_context(|| format!("failed to serialize {command} as JSON"))?,
        OutputFormat::Pretty => serde_json::to_string_pretty(report)
            .with_context(|| format!("failed to serialize {command} as JSON"))?,
    };
    println!("{rendered}");
    Ok(())
}

fn task_list_row_json(
    index: usize,
    row: &AgentOrgTaskListRow,
    context: &ClientContext,
) -> serde_json::Value {
    json!({
        "index": index,
        "orgid": row.orgid,
        "title": row.title,
        "state": row.todo_state,
        "isDone": row.is_done,
        "archived": row.archived,
        "tags": row.effective_tags,
        "source": display_source_path(&row.source_path, context),
        "sourceModifiedUnixMs": row.source_modified_unix_ms,
        "sourceLine": row.source_line,
        "scheduled": row.scheduled,
        "deadline": row.deadline,
        "closed": row.closed,
        "repeat": task_repeater_labels(row),
        "next": property_value(row, "NEXT_ACTION"),
        "resume": property_value(row, "RESUME_QUERY"),
        "memoryObjects": memory_objects_json(row),
    })
}

fn orgid_show_row_json(
    row: &AgentOrgTaskListRow,
    section: &str,
    context: &ClientContext,
) -> serde_json::Value {
    json!({
        "orgid": row.orgid,
        "title": row.title,
        "state": row.todo_state,
        "isDone": row.is_done,
        "archived": row.archived,
        "tags": row.effective_tags,
        "source": display_source_path(&row.source_path, context),
        "sourceModifiedUnixMs": row.source_modified_unix_ms,
        "sourceLine": row.source_line,
        "sourceRangeStart": row.source_range_start,
        "sourceRangeEnd": row.source_range_end,
        "outline": row.outline_path,
        "next": property_value(row, "NEXT_ACTION"),
        "resume": property_value(row, "RESUME_QUERY"),
        "memoryObjects": memory_objects_json(row),
        "section": section,
    })
}

fn memory_objects_json(row: &AgentOrgTaskListRow) -> Vec<serde_json::Value> {
    reflection_memory_objects_for_row(row)
        .into_iter()
        .enumerate()
        .map(|(index, object)| {
            json!({
                "index": index + 1,
                "kind": object.kind.name(),
                "facet": object.kind.facet_label(),
                "question": object.question,
                "value": object.value,
            })
        })
        .collect()
}

fn task_report_tag_counts(rows: &[&AgentOrgTaskListRow]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        for tag in &row.effective_tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    counts
}

fn archive_target_counts_json(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        let target = archive_target_for_row(row, settings, context);
        let target = display_source_path(target.to_string_lossy().as_ref(), context);
        *counts.entry(target).or_default() += 1;
    }
    counts
}

fn archive_plan_row_json(
    index: usize,
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) -> serde_json::Value {
    let target = archive_target_for_row(row, settings, context);
    json!({
        "index": index,
        "title": row.title,
        "state": row.todo_state,
        "tags": row.effective_tags,
        "source": display_source_path(&row.source_path, context),
        "sourceLine": row.source_line,
        "sourceRangeStart": row.source_range_start,
        "sourceRangeEnd": row.source_range_end,
        "target": display_source_path(target.to_string_lossy().as_ref(), context),
        "closed": row.closed,
    })
}
