//! Org-native SDD graph-diff projection.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use orgize::Org;
use orgize::ast::{SddNodeRecord, SddStatus};

use crate::orgize_tool::OrgizeToolError;
use crate::orgize_tool::io::{collect_org_paths, join_projection_text, read_to_string};

use super::identity::{non_blank, sdd_id_index, sdd_status_roots};

/// Options for Org-native SDD graph diff projections.
#[derive(Clone, Debug)]
pub struct OrgizeSddGraphDiffRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
}

/// Renders Org-native SDD graph diff cards.
///
/// # Errors
///
/// Returns an error when a path cannot be read.
pub fn render_sdd_graph_diff(
    request: &OrgizeSddGraphDiffRequest,
) -> Result<String, OrgizeToolError> {
    let files = collect_sdd_graph_diff_files(&request.paths)?;
    let rendered = files
        .iter()
        .map(SddGraphDiffFile::render)
        .collect::<Vec<_>>();
    Ok(join_projection_text(
        rendered,
        "[ok] orgize sdd graph-diff: no SDD nodes\n",
    ))
}

/// Counts SDD graph drift detected by the Org outline comparison.
///
/// Missing SDD paths count as one drift item so callers can use this as an
/// automation gate.
///
/// # Errors
///
/// Returns an error when a path cannot be read.
pub fn count_sdd_graph_drift(
    request: &OrgizeSddGraphDiffRequest,
) -> Result<usize, OrgizeToolError> {
    Ok(collect_sdd_graph_diff_files(&request.paths)?
        .iter()
        .map(SddGraphDiffFile::drift_count)
        .sum())
}

fn collect_sdd_graph_diff_files(
    paths: &[PathBuf],
) -> Result<Vec<SddGraphDiffFile>, OrgizeToolError> {
    let (existing_roots, mut files) =
        sdd_status_roots(paths)
            .into_iter()
            .fold((Vec::new(), Vec::new()), |mut acc, path| {
                if path.exists() {
                    acc.0.push(path);
                } else {
                    acc.1
                        .push(SddGraphDiffFile::missing(path.display().to_string()));
                }
                acc
            });
    files.extend(
        collect_org_paths(&existing_roots)?
            .into_iter()
            .filter_map(|path| collect_sdd_graph_diff_file(&path).transpose())
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(files)
}

fn collect_sdd_graph_diff_file(path: &Path) -> Result<Option<SddGraphDiffFile>, OrgizeToolError> {
    let source = read_to_string(path)?;
    let document = Org::parse(&source).document();
    let status = document.sdd_status();
    if status.records.is_empty() {
        return Ok(None);
    }
    Ok(Some(SddGraphDiffFile::from_status(
        path.display().to_string(),
        &status,
    )))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SddGraphDiffFile {
    path: String,
    exists: bool,
    nodes: usize,
    rows: Vec<SddGraphDiffRow>,
    recovery: Option<String>,
}

impl SddGraphDiffFile {
    fn missing(path: String) -> Self {
        Self {
            path,
            exists: false,
            nodes: 0,
            rows: Vec::new(),
            recovery: Some(
                "create the directory, then copy `.agent/sdd/_architecture_template.org` into an active SDD file under it".to_string(),
            ),
        }
    }

    fn from_status(path: String, status: &SddStatus) -> Self {
        Self {
            path,
            exists: true,
            nodes: status.records.len(),
            rows: sdd_graph_diff_rows(status),
            recovery: None,
        }
    }

    fn drift_count(&self) -> usize {
        usize::from(!self.exists) + self.rows.iter().filter(|row| row.is_drift()).count()
    }

    fn summary_counts(&self) -> BTreeMap<&'static str, usize> {
        let mut counts = BTreeMap::new();
        if !self.exists {
            counts.insert("missing-path", 1);
        }
        for row in &self.rows {
            *counts.entry(row.status).or_insert(0) += 1;
        }
        counts
    }

    fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("[SDD-GRAPH] ");
        output.push_str(&self.path);
        output.push('\n');
        output.push_str("nodes: ");
        output.push_str(&self.nodes.to_string());
        output.push('\n');
        output.push_str("summary: ");
        output.push_str(&render_graph_diff_summary(&self.summary_counts()));
        output.push_str("; drift=");
        output.push_str(&self.drift_count().to_string());
        output.push('\n');

        if let Some(recovery) = &self.recovery {
            output.push_str("status: missing-path\n");
            output.push_str("next: ");
            output.push_str(recovery);
            output.push('\n');
            return output;
        }

        output.push_str("edges:\n");
        for row in &self.rows {
            output.push_str("- ");
            output.push_str(row.status);
            output.push_str(": ");
            output.push_str(&row.title);
            output.push('\n');
            output.push_str("  semantic: ");
            output.push_str(&row.semantic_parent);
            output.push('\n');
            output.push_str("  outline: ");
            output.push_str(&row.outline_parent);
            output.push('\n');
            if let Some(message) = &row.message {
                output.push_str("  note: ");
                output.push_str(message);
                output.push('\n');
            }
        }
        output
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SddGraphDiffRow {
    status: &'static str,
    title: String,
    semantic_parent: String,
    outline_parent: String,
    message: Option<String>,
}

impl SddGraphDiffRow {
    fn is_drift(&self) -> bool {
        !matches!(self.status, "aligned" | "root")
    }
}

fn sdd_graph_diff_rows(status: &SddStatus) -> Vec<SddGraphDiffRow> {
    let id_index = sdd_id_index(status);
    let outline_index = sdd_outline_index(status);
    status
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let outline_parent = sdd_outline_parent_index(status, index, &outline_index);
            let semantic_parent = sdd_semantic_parent_index(record, &id_index);
            let status_label = sdd_graph_diff_status(record, semantic_parent, outline_parent);
            SddGraphDiffRow {
                status: status_label,
                title: record.title.clone(),
                semantic_parent: sdd_semantic_parent_label(record, status, semantic_parent),
                outline_parent: sdd_outline_parent_label(status, outline_parent),
                message: sdd_graph_diff_message(status_label, semantic_parent, outline_parent),
            }
        })
        .collect()
}

fn sdd_outline_index(status: &SddStatus) -> HashMap<Vec<String>, usize> {
    status
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| (record.outline_path.clone(), index))
        .collect()
}

fn sdd_outline_parent_index(
    status: &SddStatus,
    index: usize,
    outline_index: &HashMap<Vec<String>, usize>,
) -> Option<usize> {
    let outline_path = &status.records[index].outline_path;
    for ancestor_len in (1..outline_path.len()).rev() {
        if let Some(parent_index) = outline_index.get(&outline_path[..ancestor_len]) {
            return Some(*parent_index);
        }
    }
    None
}

fn sdd_semantic_parent_index(
    record: &SddNodeRecord,
    id_index: &HashMap<String, usize>,
) -> Option<usize> {
    record
        .parent
        .as_ref()
        .and_then(|parent| parent.target_id.as_ref())
        .and_then(|target_id| id_index.get(target_id))
        .copied()
}

fn sdd_graph_diff_status(
    record: &SddNodeRecord,
    semantic_parent: Option<usize>,
    outline_parent: Option<usize>,
) -> &'static str {
    let Some(parent) = &record.parent else {
        return if record.kind.can_omit_parent() {
            if outline_parent.is_some() {
                "outline-only"
            } else {
                "root"
            }
        } else {
            "missing-parent"
        };
    };
    if non_blank(parent.target_id.as_deref()).is_none() {
        return "invalid-parent";
    }
    if semantic_parent.is_none() {
        return "orphan-parent";
    }
    if semantic_parent == outline_parent {
        "aligned"
    } else {
        "semantic-move"
    }
}

fn sdd_semantic_parent_label(
    record: &SddNodeRecord,
    status: &SddStatus,
    parent_index: Option<usize>,
) -> String {
    if let Some(index) = parent_index {
        return status.records[index].title.clone();
    }
    record.parent.as_ref().map_or_else(
        || "<root>".to_string(),
        |parent| {
            parent.target_id.as_deref().map_or_else(
                || parent.raw.clone(),
                |target_id| {
                    parent
                        .label
                        .as_deref()
                        .map_or_else(|| target_id.to_string(), ToString::to_string)
                },
            )
        },
    )
}

fn sdd_outline_parent_label(status: &SddStatus, parent_index: Option<usize>) -> String {
    parent_index.map_or_else(
        || "<root>".to_string(),
        |index| status.records[index].title.clone(),
    )
}

fn sdd_graph_diff_message(
    status: &'static str,
    semantic_parent: Option<usize>,
    outline_parent: Option<usize>,
) -> Option<String> {
    match status {
        "semantic-move" => Some(format!(
            "SDD_PARENT and Org outline parent differ ({})",
            graph_diff_parent_shape(semantic_parent, outline_parent)
        )),
        "missing-parent" => Some("non-root SDD node is missing SDD_PARENT".to_string()),
        "invalid-parent" => Some("SDD_PARENT is not an Org id link".to_string()),
        "orphan-parent" => Some("SDD_PARENT target is absent from this status set".to_string()),
        "outline-only" => Some("root-capable node is nested under another SDD node".to_string()),
        _ => None,
    }
}

fn graph_diff_parent_shape(
    semantic_parent: Option<usize>,
    outline_parent: Option<usize>,
) -> String {
    match (semantic_parent, outline_parent) {
        (Some(_), Some(_)) => "semantic-parent != outline-parent".to_string(),
        (Some(_), None) => "semantic-parent set, outline-parent root".to_string(),
        (None, Some(_)) => "semantic-parent root, outline-parent set".to_string(),
        (None, None) => "both root".to_string(),
    }
}

fn render_graph_diff_summary(counts: &BTreeMap<&'static str, usize>) -> String {
    if counts.is_empty() {
        return "{}".to_string();
    }
    let entries = counts
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(", "))
}
