//! Orgize-backed document tooling for Wendao client surfaces.

use std::fs;
use std::path::{Path, PathBuf};

use orgize::Org;
use orgize::ast::{
    AgendaDate, AgendaQuery, AgentPlanningQuery, PriorityProfile, PriorityValue, RepeaterKind,
    SparseTreeQuery, TimeUnit, Timestamp, TimestampRepeater, TodoState,
};
use orgize::fmt::{FormatOptions, format_org};
use orgize::lint::{LintOptions, LintReport, lint_org_with_options};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by Orgize-backed Wendao tooling.
#[derive(Debug, Error)]
pub enum OrgizeToolError {
    /// A path cannot be read, written, or inspected.
    #[error("{path}: {source}")]
    Io {
        /// Path associated with the filesystem error.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A supplied path is not an Org file.
    #[error("{path}: expected .org file")]
    NotOrgFile {
        /// Path that failed the `.org` extension check.
        path: PathBuf,
    },
    /// A supplied path is neither a regular file nor a directory.
    #[error("{path}: unsupported path type")]
    UnsupportedPath {
        /// Unsupported path.
        path: PathBuf,
    },
    /// A date does not use the supported `YYYY-MM-DD` form.
    #[error("invalid date `{value}`; expected YYYY-MM-DD")]
    InvalidDate {
        /// Raw date value.
        value: String,
    },
    /// A priority flag value is invalid.
    #[error("unsupported priority value `{value}`")]
    InvalidPriority {
        /// Raw priority value.
        value: String,
    },
    /// Priority profile bounds are not a valid Org priority profile.
    #[error(
        "priority profile must use one priority family and satisfy highest <= default <= lowest"
    )]
    InvalidPriorityProfile,
    /// An Org agenda match expression failed to parse.
    #[error("invalid agenda match expression `{expression}`: {message}")]
    InvalidMatchExpression {
        /// Raw match expression.
        expression: String,
        /// Parser diagnostic.
        message: String,
    },
}

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

/// Options for Org source formatting.
#[derive(Clone, Debug)]
pub struct OrgizeFormatRequest {
    /// Files or directories to format.
    pub paths: Vec<PathBuf>,
    /// Check formatting without writing changes.
    pub check: bool,
}

/// Result of Org source formatting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrgizeFormatReport {
    /// Files that would change or were changed.
    pub changed_paths: Vec<PathBuf>,
}

impl OrgizeFormatReport {
    /// Returns true when at least one file needs formatting.
    #[must_use]
    pub fn changed(&self) -> bool {
        !self.changed_paths.is_empty()
    }
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
}

/// Result of Org source linting.
#[derive(Clone, Debug)]
pub struct OrgizeLintRunReport {
    /// Per-file lint reports.
    pub files: Vec<OrgizeLintFileReport>,
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
        let rendered = self
            .files
            .iter()
            .filter(|file| !file.report.is_clean())
            .map(|file| file.report.to_compact_text(&file.path, &file.source))
            .collect::<Vec<_>>();

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
        format!("{{\"files\":[{files}]}}\n")
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

/// Options for agent planning snapshots derived from Org agenda syntax.
#[derive(Clone, Debug)]
pub struct OrgizeAgentPlanningRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Inclusive start date in `YYYY-MM-DD` form.
    pub start_date: String,
    /// Optional inclusive end date in `YYYY-MM-DD` form.
    pub end_date: Option<String>,
    /// Include DONE-state tasks.
    pub include_done: bool,
    /// Include archived tasks.
    pub include_archived: bool,
    /// Include COMMENT tasks.
    pub include_comments: bool,
    /// Optional Org agenda match expression.
    pub match_expression: Option<String>,
}

/// Options for sparse-tree projections derived from Org search syntax.
#[derive(Clone, Debug)]
pub struct OrgizeSparseTreeRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Optional text search term.
    pub text: Option<String>,
    /// Optional Org agenda match expression.
    pub match_expression: Option<String>,
    /// Sparse-tree visibility controls.
    pub visibility: OrgizeSparseTreeVisibility,
    /// Include COMMENT tasks.
    pub include_comments: bool,
    /// Sparse-tree render controls.
    pub render: OrgizeSparseTreeRenderOptions,
}

/// Visibility controls for sparse-tree projections.
#[derive(Clone, Debug, Default)]
pub struct OrgizeSparseTreeVisibility {
    /// Exclude DONE-state tasks.
    pub exclude_done: bool,
    /// Exclude archived tasks.
    pub exclude_archived: bool,
}

/// Render controls for sparse-tree projections.
#[derive(Clone, Debug, Default)]
pub struct OrgizeSparseTreeRenderOptions {
    /// Render skipped section receipts.
    pub explain_skips: bool,
}

/// Options for Org-native SDD status projections.
#[derive(Clone, Debug)]
pub struct OrgizeSddStatusRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
}

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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeAgentTaskRow {
    /// Stable task row identifier derived from source path and section range.
    pub task_id: String,
    /// Source Org file path.
    pub source_path: String,
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

/// Formats Org files with the upstream Orgize formatter.
///
/// # Errors
///
/// Returns an error when a target cannot be inspected, read, or written.
pub fn format_org_files(
    request: &OrgizeFormatRequest,
) -> Result<OrgizeFormatReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let options = FormatOptions::default();
    let mut changed_paths = Vec::new();

    for path in files {
        let source = read_to_string(&path)?;
        let formatted = format_org(&source, &options);
        if formatted.changed {
            changed_paths.push(path.clone());
            if !request.check {
                fs::write(&path, formatted.output).map_err(|source| OrgizeToolError::Io {
                    path: path.clone(),
                    source,
                })?;
            }
        }
    }

    Ok(OrgizeFormatReport { changed_paths })
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
    for path in files {
        let source = read_to_string(&path)?;
        let lint_options = LintOptions {
            include_base_dir: path.parent().map(Path::to_path_buf),
            attachment_base_dir: path.parent().map(Path::to_path_buf),
            file_base_dir: path.parent().map(Path::to_path_buf),
            ..base_lint_options.clone()
        };
        let report = lint_org_with_options(&source, &lint_options);
        reports.push(OrgizeLintFileReport {
            path: path.display().to_string(),
            source,
            report,
        });
    }

    Ok(OrgizeLintRunReport { files: reports })
}

/// Renders agent planning cards from Org agenda semantics.
///
/// # Errors
///
/// Returns an error when a path cannot be read, a date is invalid, or a match
/// expression cannot be parsed.
pub fn render_agent_planning(
    request: &OrgizeAgentPlanningRequest,
) -> Result<String, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let start = parse_agenda_date(&request.start_date)?;
    let end = request
        .end_date
        .as_deref()
        .map(parse_agenda_date)
        .transpose()?
        .unwrap_or(start);
    let mut agenda = AgendaQuery::new(start, end)
        .include_done(request.include_done)
        .include_archived(request.include_archived)
        .include_comments(request.include_comments);
    if let Some(expression) = request.match_expression.as_deref() {
        agenda = agenda.match_expression(expression).map_err(|error| {
            OrgizeToolError::InvalidMatchExpression {
                expression: expression.to_string(),
                message: error.to_string(),
            }
        })?;
    }
    let base_query = AgentPlanningQuery::new(agenda);

    let mut rendered = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        let snapshot = document.agent_planning_snapshot(&AgentPlanningQuery::new(
            base_query
                .agenda
                .clone()
                .source_file(path.display().to_string()),
        ));
        rendered.push(snapshot.to_compact_text(&path.display().to_string()));
    }
    Ok(join_projection_text(
        rendered,
        "[ok] orgize agent planning\n",
    ))
}

/// Renders sparse-tree cards from Org search semantics.
///
/// # Errors
///
/// Returns an error when a path cannot be read or a match expression cannot be
/// parsed.
pub fn render_sparse_tree(request: &OrgizeSparseTreeRequest) -> Result<String, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let mut query = SparseTreeQuery::new()
        .include_done(!request.visibility.exclude_done)
        .include_archived(!request.visibility.exclude_archived)
        .include_comments(request.include_comments)
        .explain_skips(request.render.explain_skips);
    if let Some(text) = request.text.as_deref() {
        query = query.text(text);
    }
    if let Some(expression) = request.match_expression.as_deref() {
        query = query.match_expression(expression).map_err(|error| {
            OrgizeToolError::InvalidMatchExpression {
                expression: expression.to_string(),
                message: error.to_string(),
            }
        })?;
    }

    let mut rendered = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        let projection =
            document.sparse_tree_projection(&query.clone().source_file(path.display().to_string()));
        rendered.push(projection.to_compact_text(&path.display().to_string()));
    }
    Ok(join_projection_text(rendered, "[ok] orgize sparse tree\n"))
}

/// Renders Org-native SDD status cards.
///
/// # Errors
///
/// Returns an error when a path cannot be read.
pub fn render_sdd_status(request: &OrgizeSddStatusRequest) -> Result<String, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let mut rendered = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        rendered.push(
            document
                .sdd_status()
                .to_compact_text(&path.display().to_string()),
        );
    }
    Ok(join_projection_text(
        rendered,
        "[ok] orgize sdd status: no SDD nodes\n",
    ))
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
        let document = Org::parse(&source).document();
        let projection =
            document.sparse_tree_projection(&query.clone().source_file(path.display().to_string()));
        rows.extend(
            projection
                .cards
                .iter()
                .filter(|card| card.todo.is_some() && card.tags.iter().any(|tag| tag == "agent"))
                .map(|card| agent_task_row_from_card(&path, card)),
        );
    }
    Ok(OrgizeAgentTaskReadModelReport { rows })
}

fn agent_task_row_from_card(path: &Path, card: &orgize::ast::SparseTreeCard) -> OrgizeAgentTaskRow {
    let task_id = stable_task_id(path, card);
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

    OrgizeAgentTaskRow {
        task_id,
        source_path: path.display().to_string(),
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
    }
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

fn stable_task_id(path: &Path, card: &orgize::ast::SparseTreeCard) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(path.display().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(card.source.range_start.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(card.source.range_end.to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(card.title.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn collect_org_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, OrgizeToolError> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(path, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), OrgizeToolError> {
    let metadata = fs::metadata(path).map_err(|source| OrgizeToolError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(OrgizeToolError::NotOrgFile {
                path: path.to_path_buf(),
            });
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(OrgizeToolError::UnsupportedPath {
            path: path.to_path_buf(),
        });
    }

    let mut entries = fs::read_dir(path)
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| OrgizeToolError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry.file_type().map_err(|source| OrgizeToolError::Io {
            path: entry_path.clone(),
            source,
        })?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

fn read_to_string(path: &Path) -> Result<String, OrgizeToolError> {
    fs::read_to_string(path).map_err(|source| OrgizeToolError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_agenda_date(value: &str) -> Result<AgendaDate, OrgizeToolError> {
    let parts = value.split('-').collect::<Vec<_>>();
    let invalid = || OrgizeToolError::InvalidDate {
        value: value.to_string(),
    };
    let [year, month, day] = parts.as_slice() else {
        return Err(invalid());
    };
    let year = year.parse::<u16>().map_err(|_| invalid())?;
    let month = month.parse::<u8>().map_err(|_| invalid())?;
    let day = day.parse::<u8>().map_err(|_| invalid())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(invalid());
    }
    Ok(AgendaDate::new(year, month, day))
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

fn join_projection_text(rendered: Vec<String>, empty_text: &str) -> String {
    let non_empty = rendered
        .into_iter()
        .filter(|text| text.trim() != empty_text.trim())
        .collect::<Vec<_>>();
    if non_empty.is_empty() {
        empty_text.to_string()
    } else {
        non_empty.join("\n")
    }
}
