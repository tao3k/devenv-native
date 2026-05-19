//! Org-native SDD status projection.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use orgize::Org;
use orgize::ast::{SddKind, SddNodeRecord, SddStatus};
use serde::Serialize;

use crate::orgize_tool::OrgizeToolError;
use crate::orgize_tool::io::{collect_org_paths, join_projection_text, read_to_string};

use super::identity::{non_blank, sdd_id_index, sdd_status_label, sdd_status_roots};

/// Options for Org-native SDD status projections.
#[derive(Clone, Debug)]
pub struct OrgizeSddStatusRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Render only files that have diagnostics.
    pub issues_only: bool,
}

/// Renders Org-native SDD status cards.
///
/// # Errors
///
/// Returns an error when a path cannot be read.
pub fn render_sdd_status(request: &OrgizeSddStatusRequest) -> Result<String, OrgizeToolError> {
    let roots = sdd_status_roots(&request.paths);
    let mut existing_roots = Vec::new();
    let mut rendered = Vec::new();

    for path in roots {
        if path.exists() {
            existing_roots.push(path);
        } else {
            rendered.push(render_missing_sdd_path(&path));
        }
    }

    let files = collect_org_paths(&existing_roots)?;
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        let status = document.sdd_status();
        let path = path.display().to_string();
        if request.issues_only {
            let diagnostics = sdd_diagnostics(&status, &path);
            if diagnostics.is_empty() {
                continue;
            }
            rendered.push(render_sdd_status_issues(&status, &path, &diagnostics));
        } else {
            rendered.push(render_sdd_status_tree(&status, &path));
        }
    }
    let empty_text = if request.issues_only {
        "[ok] orgize sdd status: no issues\n"
    } else {
        "[ok] orgize sdd status: no SDD nodes\n"
    };
    Ok(join_projection_text(rendered, empty_text))
}

fn render_missing_sdd_path(path: &Path) -> String {
    format!(
        concat!(
            "[SDD] {path}\n",
            "architecture nodes: 0\n",
            "status: missing-path\n",
            "diagnostics:\n",
            "- [missing-path] no SDD directory or Org file exists at `{path}`\n",
            "next: create the directory, then copy `.agent/sdd/_architecture_template.org` into an active SDD file under it.\n",
        ),
        path = path.display()
    )
}

fn render_sdd_status_tree(status: &SddStatus, path: &str) -> String {
    if status.records.is_empty() {
        return "[ok] orgize sdd status: no SDD nodes\n".to_string();
    }

    let diagnostics = sdd_diagnostics(status, path);
    let mut output = String::new();
    output.push_str("[SDD] ");
    output.push_str(path);
    output.push('\n');
    output.push_str("architecture nodes: ");
    output.push_str(&status.records.len().to_string());
    output.push('\n');
    output.push_str("summary: kinds=");
    output.push_str(&render_count_map(&sdd_kind_counts(status)));
    output.push_str("; statuses=");
    output.push_str(&render_count_map(&sdd_status_counts(status)));
    output.push_str("; issues=");
    output.push_str(&diagnostics.len().to_string());
    output.push('\n');
    output.push_str("tree:\n");
    push_sdd_tree(&mut output, status, path);
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("- no issues\n");
    } else {
        for diagnostic in diagnostics {
            output.push_str("- [");
            output.push_str(diagnostic.code);
            output.push_str("] ");
            output.push_str(&diagnostic.message);
            output.push('\n');
        }
    }
    output
}

fn render_sdd_status_issues(
    status: &SddStatus,
    path: &str,
    diagnostics: &[SddDiagnostic],
) -> String {
    let mut output = String::new();
    output.push_str("[SDD] ");
    output.push_str(path);
    output.push('\n');
    output.push_str("architecture nodes: ");
    output.push_str(&status.records.len().to_string());
    output.push('\n');
    output.push_str("summary: kinds=");
    output.push_str(&render_count_map(&sdd_kind_counts(status)));
    output.push_str("; statuses=");
    output.push_str(&render_count_map(&sdd_status_counts(status)));
    output.push_str("; issues=");
    output.push_str(&diagnostics.len().to_string());
    output.push('\n');
    output.push_str("diagnostics:\n");
    for diagnostic in diagnostics {
        output.push_str("- [");
        output.push_str(diagnostic.code);
        output.push_str("] ");
        output.push_str(&diagnostic.message);
        output.push('\n');
    }
    output
}

fn sdd_kind_counts(status: &SddStatus) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in &status.records {
        *counts.entry(record.kind.as_str().to_string()).or_insert(0) += 1;
    }
    counts
}

fn sdd_status_counts(status: &SddStatus) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in &status.records {
        let status = sdd_status_label(record).unwrap_or("missing");
        *counts.entry(status.to_string()).or_insert(0) += 1;
    }
    counts
}

fn render_count_map(counts: &BTreeMap<String, usize>) -> String {
    let entries = counts
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(", "))
}

fn push_sdd_tree(output: &mut String, status: &SddStatus, path: &str) {
    let id_index = sdd_id_index(status);
    let children = sdd_child_index(status, &id_index);
    let roots = status
        .records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            let parent_index = record
                .parent
                .as_ref()
                .and_then(|parent| parent.target_id.as_ref())
                .and_then(|target_id| id_index.get(target_id));
            parent_index.is_none().then_some(index)
        })
        .collect::<Vec<_>>();

    let mut visited = HashSet::new();
    for root in roots {
        push_sdd_tree_node(output, status, path, &children, root, 0, &mut visited);
    }
    let unvisited = (0..status.records.len())
        .filter(|index| !visited.contains(index))
        .collect::<Vec<_>>();
    for index in unvisited {
        push_sdd_tree_node(output, status, path, &children, index, 0, &mut visited);
    }
}

fn sdd_child_index(
    status: &SddStatus,
    id_index: &HashMap<String, usize>,
) -> HashMap<usize, Vec<usize>> {
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for (index, record) in status.records.iter().enumerate() {
        let Some(parent_index) = record
            .parent
            .as_ref()
            .and_then(|parent| parent.target_id.as_ref())
            .and_then(|target_id| id_index.get(target_id))
        else {
            continue;
        };
        children.entry(*parent_index).or_default().push(index);
    }
    children
}

fn push_sdd_tree_node(
    output: &mut String,
    status: &SddStatus,
    path: &str,
    children: &HashMap<usize, Vec<usize>>,
    index: usize,
    depth: usize,
    visited: &mut HashSet<usize>,
) {
    let indent = "  ".repeat(depth);
    let record = &status.records[index];
    if !visited.insert(index) {
        output.push_str(&indent);
        output.push_str("- cycle: ");
        output.push_str(&record.title);
        output.push('\n');
        return;
    }

    output.push_str(&indent);
    output.push_str("- ");
    output.push_str(record.kind.as_str());
    if let Some(status) = sdd_status_label(record) {
        output.push(' ');
        output.push_str(status);
    }
    output.push_str(": ");
    output.push_str(&record.title);
    output.push('\n');
    push_sdd_record_details(output, path, record, children.get(&index), depth + 1);

    if let Some(child_indices) = children.get(&index) {
        for child in child_indices {
            push_sdd_tree_node(output, status, path, children, *child, depth + 1, visited);
        }
    }
}

fn push_sdd_record_details(
    output: &mut String,
    path: &str,
    record: &SddNodeRecord,
    children: Option<&Vec<usize>>,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    output.push_str(&indent);
    output.push_str("@ ");
    output.push_str(path);
    output.push(':');
    output.push_str(&record.source.start.line.to_string());
    output.push(':');
    output.push_str(&record.source.start.column.to_string());
    output.push('\n');
    if let Some(id) = non_blank(record.id.as_deref()) {
        push_sdd_field(output, &indent, "id", id);
    }
    if let Some(parent) = &record.parent {
        let parent_text = parent.target_id.as_deref().map_or_else(
            || parent.raw.as_str().to_string(),
            |target_id| {
                parent.label.as_deref().map_or_else(
                    || target_id.to_string(),
                    |label| format!("{target_id} ({label})"),
                )
            },
        );
        push_sdd_field(output, &indent, "parent", &parent_text);
    }
    if let Some(summary) = children.and_then(|children| sdd_child_summary(children)) {
        push_sdd_field(output, &indent, "children", &summary);
    }
    if let Some(capability) = non_blank(record.capability.as_deref()) {
        push_sdd_field(output, &indent, "capability", capability);
    }
    if let Some(viewpoint) = non_blank(record.viewpoint.as_deref()) {
        push_sdd_field(output, &indent, "viewpoint", viewpoint);
    }
    if let Some(concern) = non_blank(record.concern.as_deref()) {
        push_sdd_field(output, &indent, "concern", concern);
    }
    if let Some(quality) = non_blank(record.quality.as_deref()) {
        push_sdd_field(output, &indent, "quality", quality);
    }
    if let Some(rationale) = non_blank(record.rationale.as_deref()) {
        push_sdd_field(output, &indent, "rationale", rationale);
    }
    if let Some(slug) = non_blank(record.slug.as_deref()) {
        push_sdd_field(output, &indent, "slug", slug);
    }
}

fn push_sdd_field(output: &mut String, indent: &str, label: &str, value: &str) {
    output.push_str(indent);
    output.push_str(label);
    output.push_str(": ");
    output.push_str(value);
    output.push('\n');
}

fn sdd_child_summary(children: &[usize]) -> Option<String> {
    if children.is_empty() {
        None
    } else {
        Some(format!("nodes={}", children.len()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SddDiagnostic {
    code: &'static str,
    message: String,
}

fn sdd_diagnostics(status: &SddStatus, path: &str) -> Vec<SddDiagnostic> {
    let id_index = sdd_id_index(status);
    let mut diagnostics = Vec::new();
    for record in &status.records {
        push_sdd_record_diagnostics(&mut diagnostics, record, path, &id_index);
    }
    diagnostics
}

fn push_sdd_record_diagnostics(
    diagnostics: &mut Vec<SddDiagnostic>,
    record: &SddNodeRecord,
    path: &str,
    id_index: &HashMap<String, usize>,
) {
    if !record.kind.is_known() {
        let value = record.kind.as_str();
        let message = if value.is_empty() {
            "missing SDD_KIND".to_string()
        } else {
            format!("unsupported SDD_KIND `{value}`")
        };
        push_sdd_diagnostic(diagnostics, "invalid-kind", record, path, &message);
    }
    if !record.has_id() {
        push_sdd_diagnostic(diagnostics, "missing-id", record, path, "missing ID");
    }
    if record.id.as_deref().is_some_and(is_template_sdd_id) {
        push_sdd_diagnostic(
            diagnostics,
            "template-id",
            record,
            path,
            "replace template placeholder ID",
        );
    }
    if sdd_status_label(record).is_none() {
        push_sdd_diagnostic(
            diagnostics,
            "missing-status",
            record,
            path,
            "missing SDD_STATUS",
        );
    }
    push_sdd_parent_diagnostics(diagnostics, record, path, id_index);
    push_sdd_required_field_diagnostics(diagnostics, record, path);
}

fn push_sdd_parent_diagnostics(
    diagnostics: &mut Vec<SddDiagnostic>,
    record: &SddNodeRecord,
    path: &str,
    id_index: &HashMap<String, usize>,
) {
    if record.kind.can_omit_parent() {
        return;
    }
    let Some(parent) = &record.parent else {
        push_sdd_diagnostic(
            diagnostics,
            "missing-parent",
            record,
            path,
            "missing SDD_PARENT id link",
        );
        return;
    };
    let Some(target_id) = non_blank(parent.target_id.as_deref()) else {
        push_sdd_diagnostic(
            diagnostics,
            "invalid-parent",
            record,
            path,
            "SDD_PARENT must be an id link",
        );
        return;
    };
    if !id_index.contains_key(target_id) {
        let message = format!("SDD_PARENT target `{target_id}` is not present in this status set");
        push_sdd_diagnostic(diagnostics, "orphan-parent", record, path, &message);
    }
}

fn push_sdd_required_field_diagnostics(
    diagnostics: &mut Vec<SddDiagnostic>,
    record: &SddNodeRecord,
    path: &str,
) {
    match record.kind {
        SddKind::System | SddKind::Audit => require_sdd_field(
            diagnostics,
            record,
            path,
            "missing-concern",
            "SDD_CONCERN",
            record.concern.as_deref(),
        ),
        SddKind::Capability => require_sdd_field(
            diagnostics,
            record,
            path,
            "missing-capability",
            "SDD_CAPABILITY",
            record.capability.as_deref(),
        ),
        SddKind::View => {
            require_sdd_field(
                diagnostics,
                record,
                path,
                "missing-viewpoint",
                "SDD_VIEWPOINT",
                record.viewpoint.as_deref(),
            );
            require_sdd_field(
                diagnostics,
                record,
                path,
                "missing-concern",
                "SDD_CONCERN",
                record.concern.as_deref(),
            );
        }
        SddKind::Decision => require_sdd_field(
            diagnostics,
            record,
            path,
            "missing-rationale",
            "SDD_RATIONALE",
            record.rationale.as_deref(),
        ),
        SddKind::Unknown(_) => {}
    }
}

fn require_sdd_field(
    diagnostics: &mut Vec<SddDiagnostic>,
    record: &SddNodeRecord,
    path: &str,
    code: &'static str,
    field: &str,
    value: Option<&str>,
) {
    if non_blank(value).is_none() {
        let message = format!("missing {field}");
        push_sdd_diagnostic(diagnostics, code, record, path, &message);
    }
}

fn push_sdd_diagnostic(
    diagnostics: &mut Vec<SddDiagnostic>,
    code: &'static str,
    record: &SddNodeRecord,
    path: &str,
    message: &str,
) {
    diagnostics.push(SddDiagnostic {
        code,
        message: format!(
            "{} @ {}:{}:{}: {}",
            record.title, path, record.source.start.line, record.source.start.column, message
        ),
    });
}

fn is_template_sdd_id(id: &str) -> bool {
    id.starts_with("00000000-0000-7000-8000-")
}

/// Renders Org-native SDD status as JSON for machine consumers.
///
/// # Errors
///
/// Returns an error when a path cannot be read or the payload cannot be
/// serialized.
pub fn render_sdd_status_json(request: &OrgizeSddStatusRequest) -> Result<String, OrgizeToolError> {
    let payload = SddStatusJsonPayload {
        format: "orgize.sdd.status.v1",
        files: collect_sdd_status_json_files(&request.paths, request.issues_only)?,
    };
    let mut rendered =
        serde_json::to_string_pretty(&payload).map_err(|source| OrgizeToolError::Io {
            path: PathBuf::from("<sdd-status-json>"),
            source: std::io::Error::other(source),
        })?;
    rendered.push('\n');
    Ok(rendered)
}

/// Counts diagnostics that would be emitted by Org-native SDD status.
///
/// Missing SDD paths count as one issue so callers can use this as an
/// automation gate.
///
/// # Errors
///
/// Returns an error when a path cannot be read.
pub fn count_sdd_status_issues(request: &OrgizeSddStatusRequest) -> Result<usize, OrgizeToolError> {
    Ok(collect_sdd_status_json_files(&request.paths, false)?
        .iter()
        .map(|file| file.summary.issues)
        .sum())
}

fn collect_sdd_status_json_files(
    paths: &[PathBuf],
    issues_only: bool,
) -> Result<Vec<SddStatusFileJson>, OrgizeToolError> {
    let (existing_roots, mut files) =
        sdd_status_roots(paths)
            .into_iter()
            .fold((Vec::new(), Vec::new()), |mut acc, path| {
                if path.exists() {
                    acc.0.push(path);
                } else {
                    acc.1
                        .push(SddStatusFileJson::missing(&path.display().to_string()));
                }
                acc
            });
    files.extend(
        collect_org_paths(&existing_roots)?
            .into_iter()
            .filter_map(|path| collect_sdd_status_json_file(&path, issues_only).transpose())
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(files)
}

fn collect_sdd_status_json_file(
    path: &Path,
    issues_only: bool,
) -> Result<Option<SddStatusFileJson>, OrgizeToolError> {
    let source = read_to_string(path)?;
    let document = Org::parse(&source).document();
    let status = document.sdd_status();
    let path = path.display().to_string();
    if issues_only && sdd_diagnostics(&status, &path).is_empty() {
        return Ok(None);
    }
    Ok(Some(SddStatusFileJson::from_status(path, &status)))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddStatusJsonPayload {
    format: &'static str,
    files: Vec<SddStatusFileJson>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddStatusFileJson {
    path: String,
    exists: bool,
    architecture_nodes: usize,
    summary: SddStatusSummaryJson,
    nodes: Vec<SddNodeJson>,
    diagnostics: Vec<SddDiagnostic>,
    recovery: Option<String>,
}

impl SddStatusFileJson {
    fn missing(path: &str) -> Self {
        Self {
            path: path.to_string(),
            exists: false,
            architecture_nodes: 0,
            summary: SddStatusSummaryJson {
                kinds: BTreeMap::new(),
                statuses: BTreeMap::new(),
                issues: 1,
            },
            nodes: Vec::new(),
            diagnostics: vec![SddDiagnostic {
                code: "missing-path",
                message: format!("no SDD directory or Org file exists at `{path}`"),
            }],
            recovery: Some(
                "create the directory, then copy `.agent/sdd/_architecture_template.org` into an active SDD file under it".to_string(),
            ),
        }
    }

    fn from_status(path: String, status: &SddStatus) -> Self {
        let diagnostics = sdd_diagnostics(status, &path);
        Self {
            path,
            exists: true,
            architecture_nodes: status.records.len(),
            summary: SddStatusSummaryJson {
                kinds: sdd_kind_counts(status),
                statuses: sdd_status_counts(status),
                issues: diagnostics.len(),
            },
            nodes: status.records.iter().map(sdd_node_json).collect(),
            diagnostics,
            recovery: None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddStatusSummaryJson {
    kinds: BTreeMap<String, usize>,
    statuses: BTreeMap<String, usize>,
    issues: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddNodeJson {
    title: String,
    kind: String,
    status: Option<String>,
    id: Option<String>,
    parent: Option<SddParentJson>,
    source: SddSourceJson,
    outline_path: Vec<String>,
    capability: Option<String>,
    viewpoint: Option<String>,
    concern: Option<String>,
    quality: Option<String>,
    rationale: Option<String>,
    slug: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddParentJson {
    raw: String,
    target_id: Option<String>,
    label: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SddSourceJson {
    line: usize,
    column: usize,
}

fn sdd_node_json(record: &SddNodeRecord) -> SddNodeJson {
    SddNodeJson {
        title: record.title.clone(),
        kind: record.kind.as_str().to_string(),
        status: sdd_status_label(record).map(str::to_string),
        id: record.id.clone(),
        parent: record.parent.as_ref().map(|parent| SddParentJson {
            raw: parent.raw.clone(),
            target_id: parent.target_id.clone(),
            label: parent.label.clone(),
        }),
        source: SddSourceJson {
            line: record.source.start.line,
            column: record.source.start.column,
        },
        outline_path: record.outline_path.clone(),
        capability: record.capability.clone(),
        viewpoint: record.viewpoint.clone(),
        concern: record.concern.clone(),
        quality: record.quality.clone(),
        rationale: record.rationale.clone(),
        slug: record.slug.clone(),
    }
}
