//! Shared row display helpers for Orgize read-model commands.

use std::path::Path;

use crate::ClientContext;

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
