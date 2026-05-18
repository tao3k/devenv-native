//! Shared extraction-run task DTOs for cache materializers.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

const TASKS_TSV_HEADER: [&str; 11] = [
    "queue_id",
    "file_id",
    "relative_path",
    "category",
    "language",
    "extraction_route",
    "priority",
    "source_sha256",
    "planned_output_path",
    "output_contract",
    "status",
];

/// Raw DTO boundary for a source-contract extraction cache task.
///
/// This mirrors the stable `tasks.tsv` contract instead of depending on a
/// external vertical-domain repository or a Studio CLI type.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeCacheTask {
    /// Queue row id.
    pub queue_id: String,
    /// Source file id.
    pub file_id: String,
    /// Source path relative to the corpus root.
    pub relative_path: String,
    /// Source category.
    pub category: EpistemeCacheTaskCategory,
    /// Source language.
    pub language: String,
    /// Extraction route.
    pub extraction_route: String,
    /// Queue priority.
    pub priority: u32,
    /// Source SHA-256 copied from `files.tsv`.
    pub source_sha256: String,
    /// Planned local output path relative to a run directory.
    pub planned_output_path: String,
    /// Output contract.
    pub output_contract: String,
    /// Planned task status.
    pub status: EpistemeCacheTaskStatus,
}

/// Typed source-category value copied from a source-contract task row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EpistemeCacheTaskCategory(String);

impl EpistemeCacheTaskCategory {
    /// Return the source category as stored in `tasks.tsv`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for EpistemeCacheTaskCategory {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for EpistemeCacheTaskCategory {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Typed task status value copied from a source-contract task row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EpistemeCacheTaskStatus(String);

impl EpistemeCacheTaskStatus {
    /// Return the task status as stored in `tasks.tsv`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for EpistemeCacheTaskStatus {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for EpistemeCacheTaskStatus {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub(crate) fn read_tasks_tsv(path: &Path, label: &str) -> Result<Vec<EpistemeCacheTask>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = raw.lines();
    let header = lines
        .next()
        .with_context(|| format!("{label} tasks TSV is missing a header"))?;
    let expected_header = TASKS_TSV_HEADER.join("\t");
    if header != expected_header {
        anyhow::bail!("{label} tasks TSV header mismatch in `{}`", path.display());
    }

    lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_task_line(path, index + 2, line, label))
        .collect()
}

fn parse_task_line(
    path: &Path,
    line_number: usize,
    line: &str,
    label: &str,
) -> Result<EpistemeCacheTask> {
    let columns = line.split('\t').collect::<Vec<_>>();
    if columns.len() != TASKS_TSV_HEADER.len() {
        anyhow::bail!(
            "{label} tasks TSV line {line_number} in `{}` has {} columns, expected {}",
            path.display(),
            columns.len(),
            TASKS_TSV_HEADER.len()
        );
    }
    let priority = columns[6].parse::<u32>().with_context(|| {
        format!(
            "{label} tasks TSV line {line_number} in `{}` has invalid priority",
            path.display()
        )
    })?;
    Ok(EpistemeCacheTask {
        queue_id: columns[0].to_string(),
        file_id: columns[1].to_string(),
        relative_path: columns[2].to_string(),
        category: EpistemeCacheTaskCategory::from(columns[3]),
        language: columns[4].to_string(),
        extraction_route: columns[5].to_string(),
        priority,
        source_sha256: columns[7].to_string(),
        planned_output_path: columns[8].to_string(),
        output_contract: columns[9].to_string(),
        status: EpistemeCacheTaskStatus::from(columns[10]),
    })
}

pub(crate) fn task_extension(task: &EpistemeCacheTask) -> String {
    Path::new(&task.relative_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}
