//! Source-grounded Org agent task read-model extraction.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use orgize::ParseConfig;
use orgize::ast::{
    RepeaterKind, SparseTreeQuery, TimeUnit, Timestamp, TimestampRepeater, TodoState,
};
use serde::{Deserialize, Serialize};

use super::OrgizeToolError;
use super::io::{collect_org_paths, read_to_string};

/// Options for extracting source-grounded agent task rows from Org files.
#[derive(Clone, Debug)]
pub struct OrgizeAgentTaskReadModelRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Optional Org agenda match expression.
    pub match_expression: Option<String>,
    /// Include COMMENT tasks.
    pub include_comments: bool,
}

/// One property captured from an Org task property drawer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeAgentTaskProperty {
    /// Property key as written in the Org file.
    pub key: String,
    /// Property value as written in the Org file.
    pub value: String,
}

/// One Org timestamp repeater attached to agent task planning metadata.
///
/// Stringly state boundary: this is a serialized read-model DTO that preserves
/// Orgize's repeater labels for `DuckDB` and CLI consumers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeAgentTaskRepeater {
    /// Repeater kind as a stable lower-camel token.
    pub kind: String,
    /// Repeater numeric value.
    pub value: u32,
    /// Repeater unit as a stable singular token.
    pub unit: String,
    /// Original Org repeater cookie, for example `++1w`.
    pub cookie: String,
}

/// One source-grounded task row for agent read-model materialization.
///
/// Raw DTO boundary and stringly state boundary: this row is the lossless
/// source-grounded export shape consumed by the agent task read model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeAgentTaskRow {
    /// Stable Org section ID from the task property drawer.
    pub orgid: String,
    /// Source Org file path.
    pub source_path: String,
    /// Source Org file modified time in Unix milliseconds.
    pub source_modified_unix_ms: u64,
    /// One-based source line.
    pub source_line: u64,
    /// One-based source column.
    pub source_column: u64,
    /// Zero-based byte offset where the source subtree starts.
    pub source_range_start: u64,
    /// Zero-based byte offset where the source subtree ends.
    pub source_range_end: u64,
    /// Org headline level.
    pub level: u64,
    /// Org outline path.
    pub outline_path: Vec<String>,
    /// Headline title.
    pub title: String,
    /// TODO keyword text, when present.
    pub todo_state: Option<String>,
    /// Whether the TODO keyword belongs to the DONE class.
    pub is_done: bool,
    /// Local tags.
    pub tags: Vec<String>,
    /// Effective tags including inherited file or parent tags.
    pub effective_tags: Vec<String>,
    /// Raw scheduled timestamp, when present.
    pub scheduled: Option<String>,
    /// Parsed scheduled timestamp repeater, when present.
    pub scheduled_repeater: Option<OrgizeAgentTaskRepeater>,
    /// Raw deadline timestamp, when present.
    pub deadline: Option<String>,
    /// Parsed deadline timestamp repeater, when present.
    pub deadline_repeater: Option<OrgizeAgentTaskRepeater>,
    /// Raw closed timestamp, when present.
    pub closed: Option<String>,
    /// Whether Org archive state marks the task archived.
    pub archived: bool,
    /// Archive location, when known.
    pub archive_location: Option<String>,
    /// Local Org properties for checkpoint/resume metadata.
    pub properties: Vec<OrgizeAgentTaskProperty>,
}

/// Extracted task rows for agent read-model materialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeAgentTaskReadModelReport {
    /// Rows extracted from the requested Org files.
    pub rows: Vec<OrgizeAgentTaskRow>,
}

/// Collect source-grounded Org agent task rows for read-model materialization.
///
/// # Errors
///
/// Returns an error when a path cannot be read or a match expression cannot be
/// parsed.
pub fn collect_agent_task_rows(
    request: &OrgizeAgentTaskReadModelRequest,
) -> Result<OrgizeAgentTaskReadModelReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let mut query = SparseTreeQuery::new()
        .include_done(true)
        .include_archived(true)
        .include_comments(request.include_comments)
        .explain_skips(false);
    let expression = request.match_expression.as_deref().unwrap_or("+agent");
    query = query.match_expression(expression).map_err(|error| {
        OrgizeToolError::InvalidMatchExpression {
            expression: expression.to_string(),
            message: error.to_string(),
        }
    })?;

    let mut rows = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let source_modified_unix_ms = source_modified_unix_ms(&path)?;
        let document = agent_task_parse_config().parse(&source).document();
        let projection =
            document.sparse_tree_projection(&query.clone().source_file(path.display().to_string()));
        for card in projection
            .cards
            .iter()
            .filter(|card| card.todo.is_some() && card.tags.iter().any(|tag| tag == "agent"))
        {
            rows.push(agent_task_row_from_card(
                &source,
                &path,
                source_modified_unix_ms,
                card,
            )?);
        }
    }
    Ok(OrgizeAgentTaskReadModelReport { rows })
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

fn agent_task_row_from_card(
    source: &str,
    path: &Path,
    source_modified_unix_ms: u64,
    card: &orgize::ast::SparseTreeCard,
) -> Result<OrgizeAgentTaskRow, OrgizeToolError> {
    let todo_state = card.todo.as_ref().map(|todo| todo.name.clone());
    let is_done = card
        .todo
        .as_ref()
        .is_some_and(|todo| matches!(todo.state, TodoState::Done));
    let properties = card
        .properties
        .iter()
        .map(|property| OrgizeAgentTaskProperty {
            key: property.key.clone(),
            value: property.value.clone(),
        })
        .collect();
    let orgid = section_orgid(source, card).ok_or_else(|| OrgizeToolError::MissingAgentOrgid {
        path: path.to_path_buf(),
        line: card.source.start.line,
        title: card.title.clone(),
    })?;

    Ok(OrgizeAgentTaskRow {
        orgid,
        source_path: path.display().to_string(),
        source_modified_unix_ms,
        source_line: card.source.start.line as u64,
        source_column: card.source.start.column as u64,
        source_range_start: u64::from(card.source.range_start),
        source_range_end: u64::from(card.source.range_end),
        level: card.level as u64,
        outline_path: card.outline_path.clone(),
        title: card.title.clone(),
        todo_state,
        is_done,
        tags: card.tags.clone(),
        effective_tags: card.effective_tags.clone(),
        scheduled: card
            .planning
            .scheduled
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        scheduled_repeater: card
            .planning
            .scheduled
            .as_ref()
            .and_then(repeater_from_timestamp),
        deadline: card
            .planning
            .deadline
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        deadline_repeater: card
            .planning
            .deadline
            .as_ref()
            .and_then(repeater_from_timestamp),
        closed: card
            .planning
            .closed
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        archived: card.archive.archived,
        archive_location: card.archive.location.clone(),
        properties,
    })
}

fn source_modified_unix_ms(path: &Path) -> Result<u64, OrgizeToolError> {
    let modified = std::fs::metadata(path)
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .modified()
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX))
}

fn section_orgid(source: &str, card: &orgize::ast::SparseTreeCard) -> Option<String> {
    card.properties
        .iter()
        .find(|property| property.key == "ID")
        .map(|property| property.value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| section_text_orgid(source, card))
}

fn section_text_orgid(source: &str, card: &orgize::ast::SparseTreeCard) -> Option<String> {
    let start = card.source.range_start as usize;
    let end = card.source.range_end as usize;
    let section = source.get(start..end)?;
    let mut lines = section.lines();
    lines.next()?;
    let mut in_properties = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('*') {
            return None;
        }
        if trimmed.eq_ignore_ascii_case(":PROPERTIES:") {
            in_properties = true;
            continue;
        }
        if in_properties && trimmed.eq_ignore_ascii_case(":END:") {
            return None;
        }
        if in_properties && let Some(value) = trimmed.strip_prefix(":ID:") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn repeater_from_timestamp(timestamp: &Timestamp) -> Option<OrgizeAgentTaskRepeater> {
    let repeater = timestamp.repeater.as_ref()?;
    Some(OrgizeAgentTaskRepeater {
        kind: repeater_kind_label(repeater.kind).to_string(),
        value: repeater.value,
        unit: time_unit_label(repeater.unit).to_string(),
        cookie: repeater_cookie(repeater),
    })
}

fn repeater_cookie(repeater: &TimestampRepeater) -> String {
    format!(
        "{}{}{}",
        repeater_mark(repeater.kind),
        repeater.value,
        time_unit_cookie(repeater.unit)
    )
}

fn repeater_kind_label(kind: RepeaterKind) -> &'static str {
    match kind {
        RepeaterKind::Cumulate => "cumulate",
        RepeaterKind::CatchUp => "catchUp",
        RepeaterKind::Restart => "restart",
    }
}

fn repeater_mark(kind: RepeaterKind) -> &'static str {
    match kind {
        RepeaterKind::Cumulate => "+",
        RepeaterKind::CatchUp => "++",
        RepeaterKind::Restart => ".+",
    }
}

fn time_unit_label(unit: TimeUnit) -> &'static str {
    match unit {
        TimeUnit::Hour => "hour",
        TimeUnit::Day => "day",
        TimeUnit::Week => "week",
        TimeUnit::Month => "month",
        TimeUnit::Year => "year",
    }
}

fn time_unit_cookie(unit: TimeUnit) -> &'static str {
    match unit {
        TimeUnit::Hour => "h",
        TimeUnit::Day => "d",
        TimeUnit::Week => "w",
        TimeUnit::Month => "m",
        TimeUnit::Year => "y",
    }
}
