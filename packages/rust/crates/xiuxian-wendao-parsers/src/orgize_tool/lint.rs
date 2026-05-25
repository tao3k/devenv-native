//! Org source linting adapter.

use std::path::{Path, PathBuf};

use chrono::Local;
use orgize::ParseConfig;
use orgize::ast::{PriorityProfile, PriorityValue};
use orgize::ast::{SourcePosition, SparseTreeQuery, TodoState};
use orgize::lint::{
    LintFinding, LintLocation, LintOptions, LintReport, LintSeverity, lint_org_with_options,
};
use uuid::Uuid;

use super::OrgizeToolError;
use super::io::{collect_org_paths, read_to_string};

/// Output format for Orgize lint reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrgizeLintOutputFormat {
    /// Compact diagnostics for agents.
    Compact,
    /// Human-readable text diagnostics.
    Text,
    /// JSON diagnostics.
    Json,
}

/// Options for Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRequest {
    /// Files or directories to lint.
    pub paths: Vec<PathBuf>,
    /// Rendered lint output format.
    pub output_format: OrgizeLintOutputFormat,
    /// Optional highest priority bound.
    pub priority_highest: Option<String>,
    /// Optional lowest priority bound.
    pub priority_lowest: Option<String>,
    /// Optional default priority value.
    pub priority_default: Option<String>,
    /// Apply safe source fixes before rendering diagnostics.
    pub fix: bool,
}

/// Result of Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRunReport {
    /// Per-file lint reports.
    pub files: Vec<OrgizeLintFileReport>,
    /// Safe fixes applied before linting.
    pub fixed: Vec<OrgizeLintFixReport>,
}

impl OrgizeLintRunReport {
    /// Returns true when no lint finding was emitted.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.files.iter().all(|file| file.report.is_clean())
    }

    /// Renders this lint report using the requested format.
    #[must_use]
    pub fn render(&self, output_format: OrgizeLintOutputFormat) -> String {
        match output_format {
            OrgizeLintOutputFormat::Compact => self.render_compact(),
            OrgizeLintOutputFormat::Text => self.render_text(),
            OrgizeLintOutputFormat::Json => self.render_json(),
        }
    }

    fn render_compact(&self) -> String {
        let mut rendered = self
            .files
            .iter()
            .filter(|file| !file.report.is_clean())
            .map(|file| file.report.to_compact_text(&file.path, &file.source))
            .collect::<Vec<_>>();

        if self.fixed_count() > 0 {
            rendered.insert(
                0,
                format!("[fixed] orgize lint: {}\n", self.fixed_summary()),
            );
        }

        if rendered.is_empty() {
            "[ok] orgize lint\n".to_string()
        } else {
            rendered.join("\n")
        }
    }

    fn render_text(&self) -> String {
        self.files
            .iter()
            .map(|file| file.report.to_text(&file.path))
            .collect::<String>()
    }

    fn render_json(&self) -> String {
        let files = self
            .files
            .iter()
            .map(|file| file.report.to_json_file(&file.path))
            .collect::<Vec<_>>()
            .join(",");
        let fixed = self
            .fixed
            .iter()
            .map(OrgizeLintFixReport::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"fixed\":[{fixed}],\"files\":[{files}]}}\n")
    }

    fn fixed_count(&self) -> usize {
        self.fixed
            .iter()
            .map(OrgizeLintFixReport::fixed_count)
            .sum()
    }

    fn fixed_summary(&self) -> String {
        let added_ids: usize = self.fixed.iter().map(|report| report.added_ids).sum();
        let removed_redundant_properties: usize = self
            .fixed
            .iter()
            .map(|report| report.removed_redundant_properties)
            .sum();
        let fixed_metadata_lines: usize = self
            .fixed
            .iter()
            .map(|report| report.fixed_metadata_lines)
            .sum();
        let updated_lifecycle_keywords: usize = self
            .fixed
            .iter()
            .map(|report| report.updated_lifecycle_keywords)
            .sum();
        let fixed_closed_timestamps: usize = self
            .fixed
            .iter()
            .map(|report| report.fixed_closed_timestamps)
            .sum();
        let mut parts = Vec::new();
        if added_ids > 0 {
            parts.push(format!("added {added_ids} missing ID properties"));
        }
        if removed_redundant_properties > 0 {
            parts.push(format!(
                "removed {removed_redundant_properties} redundant properties"
            ));
        }
        if fixed_metadata_lines > 0 {
            parts.push(format!(
                "fixed {fixed_metadata_lines} agent Org metadata lines"
            ));
        }
        if updated_lifecycle_keywords > 0 {
            parts.push(format!(
                "updated {updated_lifecycle_keywords} lifecycle keywords"
            ));
        }
        if fixed_closed_timestamps > 0 {
            parts.push(format!("fixed {fixed_closed_timestamps} CLOSED timestamps"));
        }
        parts.join(", ")
    }
}

/// One safe Org lint fix report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeLintFixReport {
    /// Display path.
    pub path: String,
    /// Count of inserted ID properties.
    pub added_ids: usize,
    /// Count of removed redundant agent task properties.
    pub removed_redundant_properties: usize,
    /// Count of inserted or replaced agent Org file metadata lines.
    pub fixed_metadata_lines: usize,
    /// Count of lifecycle keyword updates.
    pub updated_lifecycle_keywords: usize,
    /// Count of converted CLOSED timestamps.
    pub fixed_closed_timestamps: usize,
}

impl OrgizeLintFixReport {
    fn fixed_count(&self) -> usize {
        self.added_ids
            + self.removed_redundant_properties
            + self.fixed_metadata_lines
            + self.updated_lifecycle_keywords
            + self.fixed_closed_timestamps
    }

    fn to_json(&self) -> String {
        format!(
            "{{\"path\":{},\"addedIds\":{},\"removedRedundantProperties\":{},\"fixedMetadataLines\":{},\"updatedLifecycleKeywords\":{},\"fixedClosedTimestamps\":{}}}",
            serde_json::to_string(&self.path).unwrap_or_else(|_| "\"\"".to_string()),
            self.added_ids,
            self.removed_redundant_properties,
            self.fixed_metadata_lines,
            self.updated_lifecycle_keywords,
            self.fixed_closed_timestamps
        )
    }
}

/// One Orgize lint report with source context.
#[derive(Clone, Debug)]
pub struct OrgizeLintFileReport {
    /// Display path used by rendered diagnostics.
    pub path: String,
    /// Original source text.
    pub source: String,
    /// Orgize lint report.
    pub report: LintReport,
}

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

fn org_filetags_include_agent(keywords: &[(String, String)]) -> bool {
    org_keyword_value(keywords, "FILETAGS").is_some_and(|filetags| {
        filetags
            .split(':')
            .any(|tag| tag.trim().eq_ignore_ascii_case("agent"))
    })
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

fn has_org_keyword(keywords: &[(String, String)], target: &str) -> bool {
    keywords.iter().any(|(key, _)| key == target)
}

fn org_keyword_value<'a>(keywords: &'a [(String, String)], target: &str) -> Option<&'a str> {
    keywords
        .iter()
        .find_map(|(key, value)| (key == target).then_some(value.as_str()))
}

fn agent_org_date_has_seconds(value: &str) -> bool {
    if value.trim() == "YYYY-MM-DD Day HH:MM:SS" {
        return true;
    }

    let mut parts = value.split_whitespace();
    let Some(date) = parts.next() else {
        return false;
    };
    let Some(day) = parts.next() else {
        return false;
    };
    let Some(time) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && org_date_token_has_ymd(date)
        && !day.is_empty()
        && org_time_token_has_seconds(time)
}

fn org_date_token_has_ymd(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn org_time_token_has_seconds(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5) || byte.is_ascii_digit())
}

struct OrgKeywordLine {
    key: String,
    value: String,
    start: usize,
    end: usize,
}

fn leading_org_keyword_lines(source: &str) -> Vec<OrgKeywordLine> {
    let mut keys = Vec::new();
    let mut offset = 0;
    while offset < source.len() {
        let line_start = offset;
        let line_end = line_end(source, line_start);
        offset = line_end;
        let line = &source[line_start..line_end];
        let trimmed = line.trim_start();
        if trimmed.starts_with('*') {
            break;
        }
        let Some((raw_key, value)) = trimmed
            .strip_prefix("#+")
            .and_then(|line| line.split_once(':'))
        else {
            continue;
        };
        if !value.trim().is_empty() {
            keys.push(OrgKeywordLine {
                key: raw_key.trim().to_ascii_uppercase(),
                value: value.trim().to_string(),
                start: line_start,
                end: line_end,
            });
        }
    }
    keys
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
            message: "agent task is 100% complete; change lifecycle state to DONE, add inactive CLOSED: [YYYY-MM-DD Day], and complete Closure Questions"
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
                message: "completed agent task is missing Closure Questions answers; fill non-empty Value cells before archive"
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
                level == target_level && title.eq_ignore_ascii_case("closure questions");
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

fn closed_line_uses_inactive_timestamp(line: &str) -> Option<bool> {
    let (_, rest) = line.split_once("CLOSED:")?;
    Some(rest.trim_start().starts_with('['))
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

fn agent_task_todo_keyword(value: &str) -> bool {
    matches!(
        value,
        "TODO" | "DOING" | "NEXT" | "WAITING" | "DONE" | "CANCELLED"
    )
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

    let mut question_rows = 0usize;
    for row in rows {
        let question = row.get(question_index).map_or("", String::as_str).trim();
        if question.is_empty() {
            continue;
        }
        question_rows += 1;
        let value = row.get(value_index).map_or("", String::as_str).trim();
        if value.is_empty() {
            return false;
        }
    }

    question_rows > 0
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProgressCookie {
    Zero,
    Partial,
    Complete,
}

fn progress_cookie_from_title(title: &str) -> Option<ProgressCookie> {
    title
        .split_whitespace()
        .filter_map(progress_cookie_from_token)
        .max_by_key(|progress| match progress {
            ProgressCookie::Zero => 0,
            ProgressCookie::Partial => 1,
            ProgressCookie::Complete => 2,
        })
}

fn progress_cookie_from_token(token: &str) -> Option<ProgressCookie> {
    let cookie = token.strip_prefix('[')?.strip_suffix(']')?;
    if let Some(percent) = cookie.strip_suffix('%') {
        let percent = percent.parse::<u64>().ok()?;
        return Some(progress_cookie_from_percent(percent));
    }
    let (done, total) = cookie.split_once('/')?;
    let done = done.parse::<u64>().ok()?;
    let total = total.parse::<u64>().ok()?;
    if total == 0 || done == 0 {
        Some(ProgressCookie::Zero)
    } else if done >= total {
        Some(ProgressCookie::Complete)
    } else {
        Some(ProgressCookie::Partial)
    }
}

fn progress_cookie_from_percent(percent: u64) -> ProgressCookie {
    if percent == 0 {
        ProgressCookie::Zero
    } else if percent >= 100 {
        ProgressCookie::Complete
    } else {
        ProgressCookie::Partial
    }
}

fn heading_line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset)
}

fn lint_location_for_offsets(source: &str, start: usize, end: usize) -> LintLocation {
    let start = start.min(source.len());
    let end = end.min(source.len());
    LintLocation {
        start: source_position_for_offset(source, start),
        end: source_position_for_offset(source, end),
        range_start: start,
        range_end: end,
    }
}

fn source_position_for_offset(source: &str, offset: usize) -> SourcePosition {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    SourcePosition { line, column }
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

struct AgentTaskIdFix {
    insert_at: usize,
    content: String,
}

struct AgentTaskIdFixReport {
    added_ids: usize,
    updated_source: Option<String>,
}

#[derive(Default)]
struct AgentTaskLintFixReport {
    added_ids: usize,
    removed_redundant_properties: usize,
    fixed_metadata_lines: usize,
    updated_lifecycle_keywords: usize,
    fixed_closed_timestamps: usize,
    updated_source: Option<String>,
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

fn fix_agent_task_lint_findings(
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

fn suppress_raw_archive_sink_findings(path: &Path, findings: &mut Vec<LintFinding>) {
    if !is_agent_raw_archive_sink_path(path) {
        return;
    }
    findings.retain(|finding| finding.code != "ORG002");
}

fn is_agent_raw_archive_sink_path(path: &Path) -> bool {
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
    let mut fixes = Vec::new();
    for card in cards {
        if !matches!(
            progress_cookie_from_title(card.title.as_str()),
            Some(ProgressCookie::Partial)
        ) {
            continue;
        }
        let Some(todo) = card.todo.as_ref() else {
            continue;
        };
        if todo.name != "TODO" {
            continue;
        }
        let Ok(heading_start) = usize::try_from(card.source.range_start) else {
            continue;
        };
        if let Some(fix) = heading_keyword_fix(source, heading_start, "TODO", "DOING") {
            fixes.push(fix);
        }
    }
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

fn agent_task_cards(
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

fn agent_task_parse_config() -> ParseConfig {
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
