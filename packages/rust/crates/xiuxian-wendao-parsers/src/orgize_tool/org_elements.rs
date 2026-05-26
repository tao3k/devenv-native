//! Source-grounded Org elements read-model extraction.

use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use orgize::Org;
use serde::{Deserialize, Serialize};

use super::OrgizeToolError;
use super::io::{collect_org_paths, read_to_string};

/// Options for extracting SQL-shaped Org element rows from Org files.
#[derive(Clone, Debug)]
pub struct OrgizeOrgElementReadModelRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
}

/// One SQL-shaped Org element row with source-file provenance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeOrgElementRow {
    /// Source Org file path.
    pub source_path: String,
    /// Source Org file modified time in Unix milliseconds.
    pub source_modified_unix_ms: u64,
    /// Document-local org-elements ordinal.
    pub ordinal: u64,
    /// Org element category, for example `section`, `element`, or `property`.
    pub category: String,
    /// Org element kind, for example `headline`, `paragraph`, or `node-property`.
    pub kind: String,
    /// Optional affiliated `#+name` value.
    pub affiliated_name: Option<String>,
    /// JSON-encoded outline path from the org-elements SQL projection.
    pub outline_path_json: String,
    /// Org element context label from the flat index.
    pub context: String,
    /// JSON-encoded org-elements summary map.
    pub summary_json: String,
    /// Optional source block or inline source language.
    pub language: Option<String>,
    /// One-based source start line.
    pub source_start_line: u64,
    /// One-based source start column.
    pub source_start_column: u64,
    /// One-based source end line.
    pub source_end_line: u64,
    /// One-based source end column.
    pub source_end_column: u64,
    /// Zero-based byte offset where the element starts.
    pub source_range_start: u64,
    /// Zero-based byte offset where the element ends.
    pub source_range_end: u64,
    /// Raw source slice for the element.
    pub source_raw: String,
}

/// Extracted Org element rows for read-model materialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OrgizeOrgElementReadModelReport {
    /// Rows extracted from the requested Org files.
    pub rows: Vec<OrgizeOrgElementRow>,
}

/// Collect source-grounded Org element rows for read-model materialization.
///
/// # Errors
///
/// Returns an error when a requested path cannot be read.
pub fn collect_org_element_rows(
    request: &OrgizeOrgElementReadModelRequest,
) -> Result<OrgizeOrgElementReadModelReport, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let mut rows = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let source_modified_unix_ms = source_modified_unix_ms(&path)?;
        let document = Org::parse(&source).document();
        for row in document.org_elements_sql_rows() {
            rows.push(OrgizeOrgElementRow {
                source_path: path.display().to_string(),
                source_modified_unix_ms,
                ordinal: row.ordinal as u64,
                category: row.category,
                kind: row.kind,
                affiliated_name: row.affiliated_name,
                outline_path_json: row.outline_path_json,
                context: row.context,
                summary_json: row.summary_json,
                language: row.language,
                source_start_line: row.source_start_line as u64,
                source_start_column: row.source_start_column as u64,
                source_end_line: row.source_end_line as u64,
                source_end_column: row.source_end_column as u64,
                source_range_start: u64::from(row.source_range_start),
                source_range_end: u64::from(row.source_range_end),
                source_raw: row.source_raw,
            });
        }
    }
    Ok(OrgizeOrgElementReadModelReport { rows })
}

fn source_modified_unix_ms(path: &std::path::Path) -> Result<u64, OrgizeToolError> {
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
