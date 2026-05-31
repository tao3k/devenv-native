//! Org lint execution and diagnostics.

use std::path::Path;

use orgize::ast::{PriorityProfile, PriorityValue};
use orgize::ast::{SparseTreeQuery, TodoState};
use orgize::lint::{LintFinding, LintOptions, LintSeverity, lint_org_with_options};

use crate::orgize_tool::{OrgizeToolError, collect_org_paths, read_to_string};

use super::agent_tracking::{
    ProgressCookie, agent_org_date_has_seconds, agent_task_todo_keyword,
    closed_line_uses_inactive_timestamp, has_org_keyword, heading_line_end,
    leading_org_keyword_lines, lint_location_for_offsets, org_filetags_include_agent,
    progress_cookie_from_title,
};
use super::fix::{
    AgentTaskLintFixReport, agent_task_cards, agent_task_parse_config,
    fix_agent_task_lint_findings, is_agent_raw_archive_sink_path,
    suppress_raw_archive_sink_findings,
};
use super::report::{
    OrgizeLintFileReport, OrgizeLintFixReport, OrgizeLintRequest, OrgizeLintRunReport,
};

/// Lints Org files with the upstream Orgize linter.
///
/// # Errors
///
/// Returns an error when a target cannot be inspected/read or when priority
/// profile flags are invalid.
pub fn lint_org_files(request: &OrgizeLintRequest) -> Result<OrgizeLintRunReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let priority_profile = priority_profile_from_flags(
        request.priority_highest.as_deref(),
        request.priority_lowest.as_deref(),
        request.priority_default.as_deref(),
    )?;
    let base_lint_options = LintOptions {
        priority_profile,
        ..LintOptions::default()
    };

    let mut reports = Vec::new();
    let mut fixed = Vec::new();
    for path in files {
        let mut source = read_to_string(&path)?;
        if request.fix {
            let AgentTaskLintFixReport {
                added_ids,
                removed_redundant_properties,
                fixed_metadata_lines,
                updated_lifecycle_keywords,
                fixed_closed_timestamps,
                updated_source,
            } = fix_agent_task_lint_findings(&path, &source)?;
            if let Some(updated) = updated_source {
                std::fs::write(&path, updated.as_bytes()).map_err(|source| {
                    OrgizeToolError::Io {
                        path: path.clone(),
                        source,
                    }
                })?;
                source = updated;
            }
            if added_ids
                + removed_redundant_properties
                + fixed_metadata_lines
                + updated_lifecycle_keywords
                + fixed_closed_timestamps
                > 0
            {
                fixed.push(OrgizeLintFixReport {
                    path: path.display().to_string(),
                    added_ids,
                    removed_redundant_properties,
                    fixed_metadata_lines,
                    updated_lifecycle_keywords,
                    fixed_closed_timestamps,
                });
            }
        }
        let lint_options = LintOptions {
            include_base_dir: path.parent().map(Path::to_path_buf),
            attachment_base_dir: path.parent().map(Path::to_path_buf),
            file_base_dir: path.parent().map(Path::to_path_buf),
            ..base_lint_options.clone()
        };
        let mut report = lint_org_with_options(&source, &lint_options);
        suppress_raw_archive_sink_findings(&path, &mut report.findings);
        report
            .findings
            .extend(agent_org_file_metadata_findings(&path, &source)?);
        report
            .findings
            .extend(agent_task_lifecycle_findings(&path, &source)?);
        sort_lint_findings(&mut report.findings);
        reports.push(OrgizeLintFileReport {
            path: path.display().to_string(),
            source,
            report,
        });
    }

    Ok(OrgizeLintRunReport {
        files: reports,
        fixed,
    })
}

struct RequiredAgentOrgKeyword {
    key: &'static str,
    code: &'static str,
    message: &'static str,
}

const REQUIRED_AGENT_ORG_KEYWORDS: &[RequiredAgentOrgKeyword] = &[
    RequiredAgentOrgKeyword {
        key: "TITLE",
        code: "agent-org-title-missing",
        message: "agent Org file is missing required #+TITLE metadata; start from .agent/org/_task_template.org",
    },
    RequiredAgentOrgKeyword {
        key: "AUTHOR",
        code: "agent-org-author-missing",
        message: "agent Org file is missing required #+AUTHOR metadata; use the project author line from .agent/org/_task_template.org",
    },
    RequiredAgentOrgKeyword {
        key: "FILETAGS",
        code: "agent-org-filetags-missing",
        message: "agent Org file is missing required #+FILETAGS metadata; include the :agent: file tag",
    },
    RequiredAgentOrgKeyword {
        key: "DATE",
        code: "agent-org-date-missing",
        message: "agent Org file is missing required #+DATE metadata; record task creation time as #+DATE: YYYY-MM-DD Day HH:MM:SS",
    },
];

fn agent_org_file_metadata_findings(
    path: &Path,
    source: &str,
) -> Result<Vec<LintFinding>, OrgizeToolError> {
    if is_agent_raw_archive_sink_path(path) {
        return Ok(Vec::new());
    }
    let keywords = leading_org_keywords(source);
    if agent_task_cards(path, source)?.is_empty()
        && !is_agent_tracking_org_source(&keywords, source)
    {
        return Ok(Vec::new());
    }

    let location = lint_location_for_offsets(source, 0, heading_line_end(source, 0));
    let mut findings = REQUIRED_AGENT_ORG_KEYWORDS
        .iter()
        .filter(|required| !has_org_keyword(&keywords, required.key))
        .map(|required| LintFinding {
            code: required.code,
            severity: LintSeverity::Warning,
            message: required.message.to_string(),
            location: location.clone(),
        })
        .collect::<Vec<_>>();

    if let Some(date) = org_keyword_value(&keywords, "DATE")
        && !agent_org_date_has_seconds(date)
    {
        findings.push(LintFinding {
            code: "agent-org-date-precision",
            severity: LintSeverity::Warning,
            message: "agent Org #+DATE must include seconds for memory ordering; use #+DATE: YYYY-MM-DD Day HH:MM:SS".to_string(),
            location,
        });
    }

    Ok(findings)
}

fn is_agent_tracking_org_source(keywords: &[(String, String)], source: &str) -> bool {
    org_filetags_include_agent(keywords) || source_contains_agent_tracking_markers(source)
}

fn source_contains_agent_tracking_markers(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(":SDD_KIND:")
            || trimmed.starts_with(":SDD_STATUS:")
            || (trimmed.starts_with('*')
                && (trimmed.contains(":agent:")
                    || trimmed.contains(":sdd:")
                    || trimmed.contains(":execplan:")))
    })
}

fn leading_org_keywords(source: &str) -> Vec<(String, String)> {
    leading_org_keyword_lines(source)
        .into_iter()
        .map(|line| (line.key, line.value))
        .collect()
}

fn org_keyword_value<'a>(keywords: &'a [(String, String)], target: &str) -> Option<&'a str> {
    keywords
        .iter()
        .find_map(|(key, value)| (key == target).then_some(value.as_str()))
}

fn agent_task_lifecycle_findings(
    path: &Path,
    source: &str,
) -> Result<Vec<LintFinding>, OrgizeToolError> {
    let document = agent_task_parse_config().parse(source).document();
    let query = SparseTreeQuery::new()
        .include_done(true)
        .include_archived(true)
        .match_expression("+agent")
        .map_err(|error| OrgizeToolError::InvalidMatchExpression {
            expression: "+agent".to_string(),
            message: error.to_string(),
        })?;
    let projection =
        document.sparse_tree_projection(&query.source_file(path.display().to_string()));
    Ok(projection
        .cards
        .iter()
        .filter(|card| card.todo.is_some() && card.tags.iter().any(|tag| tag == "agent"))
        .flat_map(|card| agent_task_lifecycle_findings_for_card(source, card))
        .collect())
}

fn agent_task_lifecycle_findings_for_card(
    source: &str,
    card: &orgize::ast::SparseTreeCard,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    if let Some(finding) = agent_task_status_property_finding(source, card) {
        findings.push(finding);
    }
    if let Some(finding) = agent_task_progress_finding(source, card) {
        findings.push(finding);
    }
    findings
}

fn agent_task_status_property_finding(
    source: &str,
    card: &orgize::ast::SparseTreeCard,
) -> Option<LintFinding> {
    let has_status = card
        .properties
        .iter()
        .any(|property| property.key.eq_ignore_ascii_case("STATUS"));
    let has_execplan = card
        .properties
        .iter()
        .any(|property| property.key.eq_ignore_ascii_case("EXECPLAN"));
    let (code, message) = if has_status {
        (
            "agent-task-status-property",
            "agent task has redundant STATUS property; use the Org lifecycle keyword instead",
        )
    } else if has_execplan {
        (
            "agent-task-execplan-property",
            "agent task has redundant EXECPLAN property; use SDD plus the task-local Org checklist, and let ExecPlan files link back with ORG_TASK when they exist",
        )
    } else {
        return None;
    };
    let range_start = usize::try_from(card.source.range_start).ok()?;
    let location =
        lint_location_for_offsets(source, range_start, heading_line_end(source, range_start));
    Some(LintFinding {
        code,
        severity: LintSeverity::Warning,
        message: message.to_string(),
        location,
    })
}

fn agent_task_progress_finding(
    source: &str,
    card: &orgize::ast::SparseTreeCard,
) -> Option<LintFinding> {
    let progress = progress_cookie_from_title(card.title.as_str())?;
    let todo = card.todo.as_ref()?;
    let range_start = usize::try_from(card.source.range_start).ok()?;
    let location =
        lint_location_for_offsets(source, range_start, heading_line_end(source, range_start));
    match progress {
        ProgressCookie::Partial if todo.name == "TODO" => Some(LintFinding {
            code: "agent-task-progress-state",
            severity: LintSeverity::Warning,
            message: "agent task has nonzero progress; change lifecycle state from TODO to DOING"
                .to_string(),
            location,
        }),
        ProgressCookie::Complete if !matches!(todo.state, TodoState::Done) => Some(LintFinding {
            code: "agent-task-progress-complete",
            severity: LintSeverity::Warning,
            message: "agent task is 100% complete; change lifecycle state to DONE, add inactive CLOSED: [YYYY-MM-DD Day], and complete Reflection Questions"
                .to_string(),
            location,
        }),
        ProgressCookie::Complete
            if matches!(todo.state, TodoState::Done) && card.planning.closed.is_none() =>
        {
            Some(LintFinding {
                code: "agent-task-closed-missing",
                severity: LintSeverity::Warning,
                message: "completed agent task is missing CLOSED; add inactive CLOSED: [YYYY-MM-DD Day] before archive"
                    .to_string(),
                location,
            })
        }
        ProgressCookie::Complete
            if matches!(todo.state, TodoState::Done)
                && !agent_task_closed_uses_inactive_timestamp(source, card) =>
        {
            Some(LintFinding {
                code: "agent-task-closed-active-timestamp",
                severity: LintSeverity::Warning,
                message: "completed agent task uses an active CLOSED timestamp; use CLOSED: [YYYY-MM-DD Day], not CLOSED: <YYYY-MM-DD Day>"
                    .to_string(),
                location,
            })
        }
        ProgressCookie::Complete
            if matches!(todo.state, TodoState::Done)
                && !agent_task_has_answered_question_table(source, card) =>
        {
            Some(LintFinding {
                code: "agent-task-closure-questions-missing",
                severity: LintSeverity::Warning,
                message: "completed agent task is missing Reflection Questions answers; fill non-empty Value cells before archive"
                    .to_string(),
                location,
            })
        }
        ProgressCookie::Complete
            if matches!(todo.state, TodoState::Done) && !agent_task_is_archived(card) =>
        {
            Some(LintFinding {
                code: "agent-task-archive-candidate",
                severity: LintSeverity::Warning,
                message: "closed reflected agent task remains in active Org files; archive it to keep memory recovery clean"
                    .to_string(),
                location,
            })
        }
        ProgressCookie::Zero | ProgressCookie::Partial | ProgressCookie::Complete => None,
    }
}

fn agent_task_is_archived(card: &orgize::ast::SparseTreeCard) -> bool {
    card.tags.iter().any(|tag| tag == "ARCHIVE")
}

fn agent_task_has_answered_question_table(
    source: &str,
    card: &orgize::ast::SparseTreeCard,
) -> bool {
    let Ok(section_start) = usize::try_from(card.source.range_start) else {
        return false;
    };
    let Ok(section_end) = usize::try_from(card.source.range_end) else {
        return false;
    };
    let section_start = section_start.min(source.len());
    let section_end = section_end.min(source.len());
    if section_start >= section_end {
        return false;
    }

    let target_level = card.level.saturating_add(1);
    let mut in_closure_questions = false;
    let mut table_lines = Vec::<&str>::new();

    for line in source[section_start..section_end].lines() {
        if let Some((level, title)) = org_heading_level_and_title(line) {
            if in_closure_questions
                && !table_lines.is_empty()
                && question_table_has_all_values(&table_lines)
            {
                return true;
            }
            table_lines.clear();
            if in_closure_questions && level <= target_level {
                break;
            }
            in_closure_questions =
                level == target_level && is_agent_task_reflection_table_title(title);
            continue;
        }

        if !in_closure_questions {
            continue;
        }

        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            table_lines.push(trimmed);
        } else if !table_lines.is_empty() {
            if question_table_has_all_values(&table_lines) {
                return true;
            }
            table_lines.clear();
        }
    }

    !table_lines.is_empty() && question_table_has_all_values(&table_lines)
}

fn is_agent_task_reflection_table_title(title: &str) -> bool {
    title.eq_ignore_ascii_case("reflection questions")
        || title.eq_ignore_ascii_case("closure questions")
}

fn agent_task_closed_uses_inactive_timestamp(
    source: &str,
    card: &orgize::ast::SparseTreeCard,
) -> bool {
    let Ok(section_start) = usize::try_from(card.source.range_start) else {
        return true;
    };
    let Ok(section_end) = usize::try_from(card.source.range_end) else {
        return true;
    };
    let section_start = section_start.min(source.len());
    let section_end = section_end.min(source.len());
    source[section_start..section_end]
        .lines()
        .find_map(closed_line_uses_inactive_timestamp)
        .unwrap_or(true)
}

fn org_heading_level_and_title(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.bytes().take_while(|byte| *byte == b'*').count();
    if level == 0
        || !trimmed
            .as_bytes()
            .get(level)
            .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }

    let mut title = trimmed[level..].trim();
    if let Some(rest) = title
        .split_once(char::is_whitespace)
        .and_then(|(head, rest)| agent_task_todo_keyword(head).then_some(rest.trim()))
    {
        title = rest;
    }
    if title.starts_with("[#") && title.get(3..4) == Some("]") {
        title = title[4..].trim_start();
    }
    Some((level, strip_org_heading_tags(title).trim()))
}

fn strip_org_heading_tags(title: &str) -> &str {
    let Some((before, after)) = title.rsplit_once(' ') else {
        return title;
    };
    if after.starts_with(':')
        && after.ends_with(':')
        && after.trim_matches(':').split(':').all(|tag| {
            !tag.is_empty()
                && tag
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '@' || ch == '#')
        })
    {
        before
    } else {
        title
    }
}

fn question_table_has_all_values(table_lines: &[&str]) -> bool {
    let rows = table_lines
        .iter()
        .map(|line| parse_org_table_row(line))
        .filter(|cells| !cells.is_empty() && !org_table_separator_row(cells))
        .collect::<Vec<_>>();
    let Some((headers, rows)) = rows.split_first() else {
        return false;
    };
    let Some(question_index) = org_table_column_index(headers, "question") else {
        return false;
    };
    let Some(value_index) = org_table_column_index(headers, "value") else {
        return false;
    };

    rows.iter()
        .filter(|row| {
            !row.get(question_index)
                .map_or("", String::as_str)
                .trim()
                .is_empty()
        })
        .try_fold(0usize, |question_rows, row| {
            let value = row.get(value_index).map_or("", String::as_str).trim();
            (!value.is_empty()).then_some(question_rows + 1)
        })
        .is_some_and(|question_rows| question_rows > 0)
}

fn parse_org_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn org_table_separator_row(cells: &[String]) -> bool {
    cells.iter().all(|cell| {
        !cell.is_empty()
            && cell
                .chars()
                .all(|character| matches!(character, '-' | '+' | ' '))
    })
}

fn org_table_column_index(headers: &[String], target: &str) -> Option<usize> {
    headers
        .iter()
        .position(|header| header.trim().eq_ignore_ascii_case(target))
}

fn sort_lint_findings(findings: &mut [LintFinding]) {
    findings.sort_by(|left, right| {
        left.location
            .range_start
            .cmp(&right.location.range_start)
            .then_with(|| left.location.range_end.cmp(&right.location.range_end))
            .then_with(|| left.code.cmp(right.code))
    });
}

fn parse_priority(value: &str) -> Result<PriorityValue, OrgizeToolError> {
    PriorityValue::parse(value).ok_or_else(|| OrgizeToolError::InvalidPriority {
        value: value.to_string(),
    })
}

fn priority_profile_from_flags(
    highest: Option<&str>,
    lowest: Option<&str>,
    default: Option<&str>,
) -> Result<PriorityProfile, OrgizeToolError> {
    if highest.is_none() && lowest.is_none() && default.is_none() {
        return Ok(PriorityProfile::org_default());
    }
    let profile = PriorityProfile::org_default();
    let highest = highest
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.highest().clone());
    let lowest = lowest
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.lowest().clone());
    let default = default
        .map(parse_priority)
        .transpose()?
        .unwrap_or_else(|| profile.default_priority().clone());
    PriorityProfile::new(highest, lowest, default).ok_or(OrgizeToolError::InvalidPriorityProfile)
}
