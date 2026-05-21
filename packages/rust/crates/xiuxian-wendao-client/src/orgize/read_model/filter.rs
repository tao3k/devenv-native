//! Task row filtering for Orgize read-model commands.

use xiuxian_wendao_parsers::OrgizeAgentTaskRepeater;

use crate::orgize::{
    OrgizeTaskArchiveArgs, OrgizeTaskListArgs, OrgizeTaskListView, OrgizeTaskReportArgs,
};

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
        .filter(|row| {
            task_row_matches_list_visibility(
                row,
                args.view,
                args.include_done,
                args.include_archived,
            )
        })
        .filter(|row| task_row_matches_view(row, args.view))
        .filter(|row| tags.iter().all(|tag| task_row_has_tag(row, tag)))
        .filter(|row| {
            text.as_ref()
                .is_none_or(|text| task_row_matches_text(row, text))
        })
        .collect()
}

fn task_row_matches_list_visibility(
    row: &AgentOrgTaskListRow,
    view: Option<OrgizeTaskListView>,
    include_done: bool,
    include_archived: bool,
) -> bool {
    match view {
        Some(OrgizeTaskListView::Active) | None => {
            (include_done || !row.is_done) && (include_archived || !row.archived)
        }
        Some(OrgizeTaskListView::Done) => !row.archived,
        Some(OrgizeTaskListView::Archived) => true,
        Some(
            OrgizeTaskListView::Achievement
            | OrgizeTaskListView::ArchiveCandidate
            | OrgizeTaskListView::ClosureNeeded
            | OrgizeTaskListView::Repeating,
        ) => include_archived || !row.archived,
    }
}

fn task_row_matches_view(row: &AgentOrgTaskListRow, view: Option<OrgizeTaskListView>) -> bool {
    match view {
        None => true,
        Some(OrgizeTaskListView::Active) => !row.is_done && !row.archived,
        Some(OrgizeTaskListView::Done) => row.is_done && !row.archived,
        Some(OrgizeTaskListView::Archived) => row.archived,
        Some(OrgizeTaskListView::Achievement) => task_row_has_tag(row, "achievement"),
        Some(OrgizeTaskListView::ArchiveCandidate) => task_row_is_archive_candidate(row),
        Some(OrgizeTaskListView::ClosureNeeded) => task_row_is_closure_needed(row),
        Some(OrgizeTaskListView::Repeating) => task_row_is_repeating(row),
    }
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
        .filter(|row| task_row_matches_view(row, args.view))
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
        .filter(|row| task_row_is_archive_candidate(row))
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

pub(super) fn task_row_is_archive_candidate(row: &AgentOrgTaskListRow) -> bool {
    row.is_done && !row.archived && !task_row_is_repeating(row)
}

pub(super) fn task_row_is_closure_needed(row: &AgentOrgTaskListRow) -> bool {
    row.level == 1
        && !row.is_done
        && !row.archived
        && !task_row_is_repeating(row)
        && title_has_complete_progress_cookie(&row.title)
}

pub(super) fn task_row_is_repeating(row: &AgentOrgTaskListRow) -> bool {
    row.scheduled_repeater.is_some() || row.deadline_repeater.is_some()
}

fn repeater_matches_text(repeater: &OrgizeAgentTaskRepeater, text: &str) -> bool {
    repeater.kind.to_lowercase().contains(text)
        || repeater.unit.to_lowercase().contains(text)
        || repeater.cookie.to_lowercase().contains(text)
}

fn title_has_complete_progress_cookie(title: &str) -> bool {
    title.contains("[100%]")
        || title
            .split_whitespace()
            .any(token_has_complete_ratio_cookie)
}

fn token_has_complete_ratio_cookie(token: &str) -> bool {
    let Some(cookie) = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((done, total)) = cookie.split_once('/') else {
        return false;
    };
    let Ok(done) = done.parse::<u64>() else {
        return false;
    };
    let Ok(total) = total.parse::<u64>() else {
        return false;
    };
    done > 0 && done == total
}
