//! Task row filtering for Orgize read-model commands.

use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::{OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskReportArgs};

use super::model::AgentOrgTaskListRow;

pub(super) fn filter_task_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskListArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| args.include_done || !row.is_done)
        .filter(|row| args.include_archived || !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

pub(super) fn filter_report_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskReportArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| args.include_archived || !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

pub(super) fn filter_archive_rows<'a>(
    rows: &'a [AgentOrgTaskListRow],
    args: &OrgizeTaskArchiveArgs,
) -> Vec<&'a AgentOrgTaskListRow> {
    let text = args.text.as_ref().map(|text| text.to_lowercase());
    let tags = args
        .tags
        .iter()
        .map(|tag| normalize_tag_filter(tag))
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    rows.iter()
        .filter(|row| row.is_done && !row.archived)
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

fn task_row_matches_text(row: &AgentOrgTaskListRow, text: &str) -> bool {
    let property_match = row.properties.iter().any(|property| {
        property.key.to_lowercase().contains(text) || property.value.to_lowercase().contains(text)
    });
    let repeater_match = row
        .scheduled_repeater
        .iter()
        .chain(row.deadline_repeater.iter())
        .any(|repeater| repeater_matches_text(repeater, text));
    row.title.to_lowercase().contains(text)
        || row.source_path.to_lowercase().contains(text)
        || row.outline_path.join(" / ").to_lowercase().contains(text)
        || row.tags.iter().any(|tag| tag.to_lowercase().contains(text))
        || row
            .effective_tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(text))
        || property_match
        || repeater_match
}

fn normalize_tag_filter(tag: &str) -> String {
    tag.trim().trim_matches(':').to_lowercase()
}

pub(super) fn task_row_has_tag(row: &AgentOrgTaskListRow, tag: &str) -> bool {
    row.tags
        .iter()
        .chain(row.effective_tags.iter())
        .any(|candidate| candidate.to_lowercase() == tag)
}

fn repeater_matches_text(repeater: &OrgizeAgentTaskRepeater, text: &str) -> bool {
    repeater.kind.to_lowercase().contains(text)
        || repeater.unit.to_lowercase().contains(text)
        || repeater.cookie.to_lowercase().contains(text)
}
