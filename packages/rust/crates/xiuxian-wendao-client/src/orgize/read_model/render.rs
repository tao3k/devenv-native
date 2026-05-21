//! Terminal rendering for Orgize read-model commands.

use std::collections::BTreeMap;

use crate::ClientContext;

use super::archive::archive_target_for_row;
use super::model::{AgentOrgTaskListRow, ResolvedReadModelSettings};
use super::row_view::{display_source_path, property_value, task_repeater_labels};

pub(super) fn render_task_list_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    context: &ClientContext,
) {
    println!();
    println!("[TASK{index:03}] {}", row.title);
    if let Some(todo_state) = row.todo_state.as_deref() {
        println!("state: {todo_state}");
    }
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    if let Some(scheduled) = row.scheduled.as_deref() {
        println!("scheduled: {scheduled}");
    }
    if let Some(deadline) = row.deadline.as_deref() {
        println!("deadline: {deadline}");
    }
    let repeaters = task_repeater_labels(row);
    if !repeaters.is_empty() {
        println!("repeat: {}", repeaters.join(", "));
    }
    if let Some(closed) = row.closed.as_deref() {
        println!("closed: {closed}");
    }
    if let Some(next_action) = property_value(row, "NEXT_ACTION") {
        println!("next: {next_action}");
    }
    if let Some(resume_query) = property_value(row, "RESUME_QUERY") {
        println!("resume: {resume_query}");
    }
}

pub(super) fn render_tag_counts(rows: &[&AgentOrgTaskListRow]) {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        for tag in &row.effective_tags {
            *counts.entry(tag.clone()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return;
    }

    println!();
    println!("Tags");
    for (tag, count) in counts {
        println!("{tag}: {count}");
    }
}

pub(super) fn render_report_section(
    title: &str,
    rows: &[&AgentOrgTaskListRow],
    limit: usize,
    context: &ClientContext,
) {
    println!();
    println!("{title}: {}", rows.len());
    for (index, row) in rows.iter().take(limit).enumerate() {
        render_task_list_row(index + 1, row, context);
    }
}

pub(super) fn render_archive_plan_row(
    index: usize,
    row: &AgentOrgTaskListRow,
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) {
    println!();
    println!("[ARCHIVE{index:03}] {}", row.title);
    println!(
        "source: {}:{}",
        display_source_path(&row.source_path, context),
        row.source_line
    );
    println!(
        "range: {}..{}",
        row.source_range_start, row.source_range_end
    );
    println!(
        "target: {}",
        display_source_path(
            archive_target_for_row(row, settings, context)
                .to_string_lossy()
                .as_ref(),
            context
        )
    );
    if !row.effective_tags.is_empty() {
        println!("tags: {}", row.effective_tags.join(":"));
    }
}

pub(super) fn render_archive_target_summary(
    rows: &[&AgentOrgTaskListRow],
    settings: &ResolvedReadModelSettings,
    context: &ClientContext,
) {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        let target = archive_target_for_row(row, settings, context);
        let target = display_source_path(target.to_string_lossy().as_ref(), context);
        *counts.entry(target).or_default() += 1;
    }
    if counts.is_empty() {
        return;
    }

    println!();
    println!("Archive Targets");
    for (target, count) in counts {
        println!("{target}: {count}");
    }
}
