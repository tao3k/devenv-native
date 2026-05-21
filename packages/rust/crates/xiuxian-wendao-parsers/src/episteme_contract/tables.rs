//! TSV table DTOs and parsers for episteme source contracts.

use serde::Serialize;

use super::EpistemeSourceContractParseError;
use super::tsv::{parse_number, read_tsv};

const FILE_FIELDS: [&str; 8] = [
    "file_id",
    "relative_path",
    "extension",
    "byte_size",
    "sha256",
    "category",
    "language",
    "extraction_route",
];

const EXTRACTION_QUEUE_FIELDS: [&str; 9] = [
    "queue_id",
    "file_id",
    "relative_path",
    "category",
    "language",
    "extraction_route",
    "priority",
    "output_contract",
    "status",
];

/// Raw DTO boundary: this mirrors one row from `files.tsv`.
/// Stringly state boundary: `category` and route fields remain source-catalog tokens.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeFileRow {
    /// Stable source file id.
    pub file_id: String,
    /// Source path relative to the source corpus root.
    pub relative_path: String,
    /// Lowercase extension without dot.
    pub extension: String,
    /// Recorded source size in bytes.
    pub byte_size: u64,
    /// Recorded source SHA-256.
    pub sha256: String,
    /// Corpus category.
    pub category: String,
    /// Source language tag.
    pub language: String,
    /// Planned extraction route.
    pub extraction_route: String,
}

/// Raw DTO boundary: this mirrors one row from `extraction_queue.tsv`.
/// Stringly state boundary: queue status and category fields remain source-catalog tokens.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeExtractionQueueRow {
    /// Stable queue row id.
    pub queue_id: String,
    /// Stable source file id.
    pub file_id: String,
    /// Source path relative to the source corpus root.
    pub relative_path: String,
    /// Corpus category.
    pub category: String,
    /// Source language tag.
    pub language: String,
    /// Planned extraction route.
    pub extraction_route: String,
    /// Queue priority; lower values are planned first.
    pub priority: u32,
    /// Output contract for this row.
    pub output_contract: String,
    /// Queue row status.
    pub status: String,
}

/// Parse episteme source-contract `files.tsv` rows from TSV text.
///
/// # Errors
///
/// Returns an error when the TSV header, row width, or numeric fields are
/// invalid.
pub fn parse_episteme_files_tsv(
    raw: &str,
) -> Result<Vec<EpistemeFileRow>, EpistemeSourceContractParseError> {
    read_tsv(raw, &FILE_FIELDS)?
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            Ok(EpistemeFileRow {
                file_id: row[0].clone(),
                relative_path: row[1].clone(),
                extension: row[2].clone(),
                byte_size: parse_number(index + 2, "byte_size", &row[3])?,
                sha256: row[4].clone(),
                category: row[5].clone(),
                language: row[6].clone(),
                extraction_route: row[7].clone(),
            })
        })
        .collect()
}

/// Parse episteme source-contract `extraction_queue.tsv` rows from TSV text.
///
/// # Errors
///
/// Returns an error when the TSV header, row width, or numeric fields are
/// invalid.
pub fn parse_episteme_extraction_queue_tsv(
    raw: &str,
) -> Result<Vec<EpistemeExtractionQueueRow>, EpistemeSourceContractParseError> {
    read_tsv(raw, &EXTRACTION_QUEUE_FIELDS)?
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            Ok(EpistemeExtractionQueueRow {
                queue_id: row[0].clone(),
                file_id: row[1].clone(),
                relative_path: row[2].clone(),
                category: row[3].clone(),
                language: row[4].clone(),
                extraction_route: row[5].clone(),
                priority: parse_number(index + 2, "priority", &row[6])?,
                output_contract: row[7].clone(),
                status: row[8].clone(),
            })
        })
        .collect()
}
