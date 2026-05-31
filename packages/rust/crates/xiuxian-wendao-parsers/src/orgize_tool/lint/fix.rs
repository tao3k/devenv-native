use std::path::Path;

use chrono::Local;
use orgize::ParseConfig;
use orgize::ast::{SparseTreeQuery, TodoState};
use orgize::lint::LintFinding;
use uuid::Uuid;

use crate::orgize_tool::OrgizeToolError;

use super::agent_tracking::{
    ProgressCookie, REQUIRED_AGENT_ORG_KEYWORDS, agent_org_date_has_seconds, has_org_keyword,
    heading_line_end, is_agent_tracking_org_source, leading_org_keyword_lines, org_keyword_value,
    progress_cookie_from_title,
};

struct AgentTaskIdFix {
    insert_at: usize,
    content: String,
}

struct AgentTaskIdFixReport {
    added_ids: usize,
    updated_source: Option<String>,
}

#[derive(Default)]
pub(super) struct AgentTaskLintFixReport {
    pub(super) added_ids: usize,
    pub(super) removed_redundant_properties: usize,
    pub(super) fixed_metadata_lines: usize,
    pub(super) updated_lifecycle_keywords: usize,
    pub(super) fixed_closed_timestamps: usize,
    pub(super) updated_source: Option<String>,
}

impl AgentTaskLintFixReport {
    fn fixed_count(&self) -> usize {
        self.added_ids
            + self.removed_redundant_properties
            + self.fixed_metadata_lines
            + self.updated_lifecycle_keywords
            + self.fixed_closed_timestamps
    }
}

pub(super) fn fix_agent_task_lint_findings(
    path: &Path,
    source: &str,
) -> Result<AgentTaskLintFixReport, OrgizeToolError> {
    let mut updated = source.to_string();
    let mut report = AgentTaskLintFixReport::default();

    let metadata_report = fix_agent_org_file_metadata(path, &updated)?;
    report.fixed_metadata_lines = metadata_report.changed_count;
    if let Some(next_source) = metadata_report.updated_source {
        updated = next_source;
    }

    let id_report = fix_missing_agent_task_ids(path, &updated)?;
    report.added_ids = id_report.added_ids;
    if let Some(next_source) = id_report.updated_source {
        updated = next_source;
    }

    let property_report = fix_redundant_agent_task_properties(path, &updated)?;
    report.removed_redundant_properties = property_report.changed_count;
    if let Some(next_source) = property_report.updated_source {
        updated = next_source;
    }

    let lifecycle_report = fix_agent_task_lifecycle_keywords(path, &updated)?;
    report.updated_lifecycle_keywords = lifecycle_report.changed_count;
    if let Some(next_source) = lifecycle_report.updated_source {
        updated = next_source;
    }

    let closed_report = fix_agent_task_closed_timestamps(path, &updated)?;
    report.fixed_closed_timestamps = closed_report.changed_count;
    if let Some(next_source) = closed_report.updated_source {
        updated = next_source;
    }

    if report.fixed_count() > 0 {
        report.updated_source = Some(updated);
    }
    Ok(report)
}

struct AgentTaskSourceFixReport {
    changed_count: usize,
    updated_source: Option<String>,
}

struct SourceRangeFix {
    start: usize,
    end: usize,
    replacement: String,
}

fn fix_agent_org_file_metadata(
    path: &Path,
    source: &str,
) -> Result<AgentTaskSourceFixReport, OrgizeToolError> {
    if is_agent_raw_archive_sink_path(path) {
        return Ok(AgentTaskSourceFixReport {
            changed_count: 0,
            updated_source: None,
        });
    }
    let keyword_lines = leading_org_keyword_lines(source);
    let keywords = keyword_lines
        .iter()
        .map(|line| (line.key.clone(), line.value.clone()))
        .collect::<Vec<_>>();
    if agent_task_cards(path, source)?.is_empty()
        && !is_agent_tracking_org_source(&keywords, source)
    {
        return Ok(AgentTaskSourceFixReport {
            changed_count: 0,
            updated_source: None,
        });
    }

    let missing_count = REQUIRED_AGENT_ORG_KEYWORDS
        .iter()
        .filter(|required| !has_org_keyword(&keywords, required.key))
        .count();
    let date_needs_fix =
        org_keyword_value(&keywords, "DATE").is_some_and(|date| !agent_org_date_has_seconds(date));
    if missing_count == 0 && !date_needs_fix {
        return Ok(AgentTaskSourceFixReport {
            changed_count: 0,
            updated_source: None,
        });
    }

    let title = org_keyword_value(&keywords, "TITLE")
        .map(ToString::to_string)
        .or_else(|| first_heading_title(source))
        .unwrap_or_else(|| default_title_from_path(path));
    let author = org_keyword_value(&keywords, "AUTHOR")
        .unwrap_or("CyberXiuXian Artisan workshop")
        .to_string();
    let filetags = org_keyword_value(&keywords, "FILETAGS")
        .map_or_else(|| inferred_agent_filetags(source), ToString::to_string);
    let date = org_keyword_value(&keywords, "DATE")
        .filter(|date| agent_org_date_has_seconds(date))
        .map_or_else(current_org_datetime, ToString::to_string);

    let fixed_header =
        format!("#+TITLE: {title}\n#+AUTHOR: {author}\n#+FILETAGS: {filetags}\n#+DATE: {date}\n\n");
    let mut updated = source.to_string();
    let mut required_lines = keyword_lines
        .iter()
        .filter(|line| {
            REQUIRED_AGENT_ORG_KEYWORDS
                .iter()
                .any(|required| required.key == line.key)
        })
        .map(|line| (line.start, line.end))
        .collect::<Vec<_>>();
    required_lines.sort_by_key(|(start, _)| *start);
    for (start, end) in required_lines.into_iter().rev() {
        updated.replace_range(start..end, "");
    }
    let updated = format!("{fixed_header}{}", updated.trim_start_matches('\n'));

    Ok(AgentTaskSourceFixReport {
        changed_count: missing_count + usize::from(date_needs_fix),
        updated_source: Some(updated),
    })
}

pub(super) fn suppress_raw_archive_sink_findings(path: &Path, findings: &mut Vec<LintFinding>) {
    if !is_agent_raw_archive_sink_path(path) {
        return;
    }
    findings.retain(|finding| finding.code != "ORG002");
}

pub(super) fn is_agent_raw_archive_sink_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "archives")
}

fn first_heading_title(source: &str) -> Option<String> {
    source
        .lines()
        .find_map(|line| {
            let heading = line.trim_start();
            if !heading.starts_with('*') {
                return None;
            }
            let mut title = heading.trim_start_matches('*').trim_start();
            for keyword in ["TODO", "DOING", "NEXT", "WAITING", "DONE", "CANCELLED"] {
                if let Some(rest) = title.strip_prefix(keyword) {
                    title = rest.trim_start();
                    break;
                }
            }
            let tagless = title.split(" :").next().unwrap_or(title).trim();
            Some(strip_trailing_progress_cookies(tagless).to_string())
        })
        .filter(|title| !title.trim().is_empty())
}

fn strip_trailing_progress_cookies(mut title: &str) -> &str {
    loop {
        let trimmed = title.trim_end();
        if !trimmed.ends_with(']') {
            return trimmed;
        }
        let Some(start) = trimmed.rfind('[') else {
            return trimmed;
        };
        let cookie = &trimmed[start + 1..trimmed.len() - 1];
        if cookie.contains(' ') || !(cookie.contains('/') || cookie.ends_with('%')) {
            return trimmed;
        }
        title = &trimmed[..start];
    }
}

fn default_title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['_', '-'], " "))
        .filter(|stem| !stem.trim().is_empty())
        .unwrap_or_else(|| "Agent task".to_string())
}

fn inferred_agent_filetags(source: &str) -> String {
    if source.contains(":SDD_KIND:") || source.contains(":sdd:") {
        ":agent:sdd:architecture:".to_string()
    } else if source.contains(":execplan:") {
        ":agent:execplan:".to_string()
    } else {
        ":agent:".to_string()
    }
}

fn current_org_datetime() -> String {
    Local::now().format("%Y-%m-%d %a %H:%M:%S").to_string()
}

fn fix_redundant_agent_task_properties(
    path: &Path,
    source: &str,
) -> Result<AgentTaskSourceFixReport, OrgizeToolError> {
    let cards = agent_task_cards(path, source)?;
    let mut fixes = Vec::new();
    for card in cards {
        let Some((section_start, section_end)) = card_section_range(source, &card) else {
            continue;
        };
        let mut in_properties = false;
        for (line_start, line_end, line) in
            source_lines_in_range(source, section_start, section_end)
        {
            let trimmed = line.trim_start();
            if trimmed.trim_end().eq_ignore_ascii_case(":PROPERTIES:") {
                in_properties = true;
                continue;
            }
            if in_properties && trimmed.trim_end().eq_ignore_ascii_case(":END:") {
                in_properties = false;
                continue;
            }
            if in_properties
                && (trimmed.starts_with(":STATUS:") || trimmed.starts_with(":EXECPLAN:"))
            {
                fixes.push(SourceRangeFix {
                    start: line_start,
                    end: line_end,
                    replacement: String::new(),
                });
            }
        }
    }
    Ok(apply_source_fixes(source, fixes))
}

fn fix_agent_task_lifecycle_keywords(
    path: &Path,
    source: &str,
) -> Result<AgentTaskSourceFixReport, OrgizeToolError> {
    let cards = agent_task_cards(path, source)?;
    let fixes = cards
        .into_iter()
        .filter(|card| {
            matches!(
                progress_cookie_from_title(card.title.as_str()),
                Some(ProgressCookie::Partial)
            )
        })
        .filter(|card| card.todo.as_ref().is_some_and(|todo| todo.name == "TODO"))
        .filter_map(|card| usize::try_from(card.source.range_start).ok())
        .filter_map(|heading_start| heading_keyword_fix(source, heading_start, "TODO", "DOING"))
        .collect();
    Ok(apply_source_fixes(source, fixes))
}

fn fix_agent_task_closed_timestamps(
    path: &Path,
    source: &str,
) -> Result<AgentTaskSourceFixReport, OrgizeToolError> {
    let cards = agent_task_cards(path, source)?;
    let mut fixes = Vec::new();
    for card in cards {
        if !matches!(
            progress_cookie_from_title(card.title.as_str()),
            Some(ProgressCookie::Complete)
        ) || !card
            .todo
            .as_ref()
            .is_some_and(|todo| matches!(todo.state, TodoState::Done))
        {
            continue;
        }
        let Some((section_start, section_end)) = card_section_range(source, &card) else {
            continue;
        };
        for (line_start, line_end, line) in
            source_lines_in_range(source, section_start, section_end)
        {
            let Some(relative_closed) = line.find("CLOSED:") else {
                continue;
            };
            let closed_rest = &line[relative_closed + "CLOSED:".len()..];
            let Some(open_relative) = closed_rest.find('<') else {
                continue;
            };
            let timestamp_start = relative_closed + "CLOSED:".len() + open_relative;
            let Some(close_relative) = line[timestamp_start..].find('>') else {
                continue;
            };
            let timestamp_end = timestamp_start + close_relative;
            let replacement = format!(
                "{}[{}]{}",
                &line[..timestamp_start],
                &line[timestamp_start + 1..timestamp_end],
                &line[timestamp_end + 1..]
            );
            fixes.push(SourceRangeFix {
                start: line_start,
                end: line_end,
                replacement,
            });
        }
    }
    Ok(apply_source_fixes(source, fixes))
}

fn fix_missing_agent_task_ids(
    path: &Path,
    source: &str,
) -> Result<AgentTaskIdFixReport, OrgizeToolError> {
    let cards = agent_task_cards(path, source)?;
    let mut fixes = cards
        .iter()
        .filter(|card| {
            !card
                .properties
                .iter()
                .any(|property| property.key == "ID" && !property.value.trim().is_empty())
        })
        .map(|card| {
            usize::try_from(card.source.range_start)
                .map(|heading_offset| agent_task_id_fix(source, heading_offset))
                .map_err(|error| OrgizeToolError::InvalidMatchExpression {
                    expression: "+agent".to_string(),
                    message: format!("source range start does not fit usize: {error}"),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    fixes.sort_by_key(|fix| fix.insert_at);

    if fixes.is_empty() {
        return Ok(AgentTaskIdFixReport {
            added_ids: 0,
            updated_source: None,
        });
    }

    let mut updated = source.to_string();
    for fix in fixes.iter().rev() {
        updated.insert_str(fix.insert_at, &fix.content);
    }
    Ok(AgentTaskIdFixReport {
        added_ids: fixes.len(),
        updated_source: Some(updated),
    })
}

pub(super) fn agent_task_cards(
    path: &Path,
    source: &str,
) -> Result<Vec<orgize::ast::SparseTreeCard>, OrgizeToolError> {
    let document = agent_task_parse_config().parse(source).document();
    let query = SparseTreeQuery::new()
        .include_done(true)
        .include_archived(true)
        .match_expression("+agent")
        .map_err(|error| OrgizeToolError::InvalidMatchExpression {
            expression: "+agent".to_string(),
            message: error.to_string(),
        })?;
    let query = query.source_file(path.display().to_string());
    let projection = document.sparse_tree_projection(&query);
    Ok(projection
        .cards
        .into_iter()
        .filter(|card| card.todo.is_some() && card.tags.iter().any(|tag| tag == "agent"))
        .collect())
}

pub(super) fn agent_task_parse_config() -> ParseConfig {
    ParseConfig {
        todo_keywords: (
            ["TODO", "DOING", "NEXT", "WAITING"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            ["DONE", "CANCELLED"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
        ),
        ..ParseConfig::default()
    }
}

fn agent_task_id_fix(source: &str, heading_offset: usize) -> AgentTaskIdFix {
    let heading_end = line_end(source, heading_offset);
    let indent = heading_indent(source, heading_offset, heading_end);
    let insert_at = planning_end(source, heading_end);
    let id_line = format!("{indent}:ID: {}\n", Uuid::new_v4());

    if source[insert_at..]
        .lines()
        .next()
        .is_some_and(|line| line.trim().eq_ignore_ascii_case(":PROPERTIES:"))
    {
        AgentTaskIdFix {
            insert_at: line_end(source, insert_at),
            content: id_line,
        }
    } else {
        AgentTaskIdFix {
            insert_at,
            content: format!("{indent}:PROPERTIES:\n{id_line}{indent}:END:\n"),
        }
    }
}

fn planning_end(source: &str, mut offset: usize) -> usize {
    while offset < source.len() {
        let end = line_end(source, offset);
        let line = source[offset..end].trim();
        if line.starts_with("SCHEDULED:")
            || line.starts_with("DEADLINE:")
            || line.starts_with("CLOSED:")
        {
            offset = end;
        } else {
            break;
        }
    }
    offset
}

fn heading_indent(source: &str, start: usize, end: usize) -> &str {
    let line = &source[start..end];
    let prefix_len = line.find('*').unwrap_or(0);
    &line[..prefix_len]
}

fn line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |relative| start + relative + 1)
}

fn card_section_range(source: &str, card: &orgize::ast::SparseTreeCard) -> Option<(usize, usize)> {
    let start = usize::try_from(card.source.range_start)
        .ok()?
        .min(source.len());
    let end = usize::try_from(card.source.range_end)
        .ok()?
        .min(source.len());
    (start < end).then_some((start, end))
}

fn source_lines_in_range(
    source: &str,
    start: usize,
    end: usize,
) -> impl Iterator<Item = (usize, usize, &str)> {
    let mut offset = start;
    std::iter::from_fn(move || {
        if offset >= end {
            return None;
        }
        let line_start = offset;
        let line_end = line_end(source, line_start).min(end);
        offset = line_end;
        Some((line_start, line_end, &source[line_start..line_end]))
    })
}

fn heading_keyword_fix(
    source: &str,
    heading_start: usize,
    old_keyword: &str,
    new_keyword: &str,
) -> Option<SourceRangeFix> {
    let heading_end = heading_line_end(source, heading_start);
    let heading = source.get(heading_start..heading_end)?;
    let keyword_start = heading.find(old_keyword)?;
    let before_keyword = &heading[..keyword_start];
    if !before_keyword.trim_start().starts_with('*') {
        return None;
    }
    let absolute_start = heading_start + keyword_start;
    let absolute_end = absolute_start + old_keyword.len();
    Some(SourceRangeFix {
        start: absolute_start,
        end: absolute_end,
        replacement: new_keyword.to_string(),
    })
}

fn apply_source_fixes(source: &str, mut fixes: Vec<SourceRangeFix>) -> AgentTaskSourceFixReport {
    if fixes.is_empty() {
        return AgentTaskSourceFixReport {
            changed_count: 0,
            updated_source: None,
        };
    }
    fixes.sort_by_key(|fix| fix.start);
    fixes.dedup_by(|left, right| left.start == right.start && left.end == right.end);
    let mut updated = source.to_string();
    for fix in fixes.iter().rev() {
        updated.replace_range(fix.start..fix.end, &fix.replacement);
    }
    AgentTaskSourceFixReport {
        changed_count: fixes.len(),
        updated_source: Some(updated),
    }
}
