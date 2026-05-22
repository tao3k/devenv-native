//! Shared row display helpers for Orgize read-model commands.

use std::path::Path;

use crate::ClientContext;
use crate::orgize::OrgizeTaskListView;

use super::model::AgentOrgTaskListRow;

pub(super) fn task_repeater_labels(row: &AgentOrgTaskListRow) -> Vec<String> {
    let mut labels = Vec::new();
    if let Some(repeater) = &row.scheduled_repeater {
        labels.push(format!("scheduled {} ({})", repeater.cookie, repeater.kind));
    }
    if let Some(repeater) = &row.deadline_repeater {
        labels.push(format!("deadline {} ({})", repeater.cookie, repeater.kind));
    }
    labels
}

pub(crate) fn task_list_view_label(view: OrgizeTaskListView) -> &'static str {
    match view {
        OrgizeTaskListView::Active => "active",
        OrgizeTaskListView::Done => "done",
        OrgizeTaskListView::Archived => "archived",
        OrgizeTaskListView::Achievement => "achievement",
        OrgizeTaskListView::ArchiveCandidate => "archive-candidate",
        OrgizeTaskListView::ClosureNeeded => "closure-needed",
        OrgizeTaskListView::Repeating => "repeating",
    }
}

pub(super) fn display_source_path(source_path: &str, context: &ClientContext) -> String {
    let path = Path::new(source_path);
    path.strip_prefix(context.root()).map_or_else(
        |_| source_path.to_string(),
        |path| path.display().to_string(),
    )
}

pub(super) fn property_value<'a>(row: &'a AgentOrgTaskListRow, key: &str) -> Option<&'a str> {
    row.properties
        .iter()
        .find(|property| property.key == key)
        .map(|property| property.value.as_str())
}
