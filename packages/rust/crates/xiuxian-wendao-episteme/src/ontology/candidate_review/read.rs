//! TSV readers for ontology candidate review inputs.

use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result};

use super::model::{CandidateEvidenceRecord, CandidateObjectRecord, CandidateRelationRecord};

pub(super) fn read_candidate_objects(path: &Path) -> Result<Vec<CandidateObjectRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateObjectRecord {
                candidate_id: required(row, "candidate_id", path)?,
                candidate_kind: required(row, "candidate_kind", path)?,
                label: required(row, "label", path)?,
                suggested_term_key: optional(row, "suggested_term_key"),
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                text_char_count: parse_usize(row, "text_char_count", path)?,
                raw_to_rdf_promotion_allowed: parse_bool(
                    row,
                    "raw_to_rdf_promotion_allowed",
                    path,
                )?,
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
            })
        })
        .collect()
}

pub(super) fn read_candidate_relations(path: &Path) -> Result<Vec<CandidateRelationRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateRelationRecord {
                candidate_id: required(row, "candidate_id", path)?,
                relation_kind: required(row, "relation_kind", path)?,
                source_candidate_id: required(row, "source_candidate_id", path)?,
                target_candidate_id: required(row, "target_candidate_id", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
            })
        })
        .collect()
}

pub(super) fn read_candidate_evidence(path: &Path) -> Result<Vec<CandidateEvidenceRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateEvidenceRecord {
                evidence_id: required(row, "evidence_id", path)?,
                evidence_kind: required(row, "evidence_kind", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                text_char_count: parse_usize(row, "text_char_count", path)?,
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
            })
        })
        .collect()
}

struct TsvTable {
    rows: Vec<HashMap<String, String>>,
}

fn read_tsv(path: &Path) -> Result<TsvTable> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .with_context(|| format!("candidate TSV `{}` is empty", path.display()))?;
    let headers: Vec<String> = header_line.split('\t').map(ToString::to_string).collect();
    if headers.is_empty() {
        anyhow::bail!("candidate TSV `{}` has no header", path.display());
    }
    let rows = lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_tsv_row(path, &headers, index + 2, line))
        .collect::<Result<Vec<_>>>()?;
    Ok(TsvTable { rows })
}

fn parse_tsv_row(
    path: &Path,
    headers: &[String],
    line_number: usize,
    line: &str,
) -> Result<HashMap<String, String>> {
    let values: Vec<&str> = line.split('\t').collect();
    if values.len() != headers.len() {
        anyhow::bail!(
            "candidate TSV `{}` row {} has {} columns, expected {}",
            path.display(),
            line_number,
            values.len(),
            headers.len()
        );
    }
    Ok(headers
        .iter()
        .cloned()
        .zip(values.into_iter().map(unescape_tsv))
        .collect())
}

fn required(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<String> {
    row.get(name)
        .cloned()
        .with_context(|| format!("candidate TSV `{}` missing `{name}` column", path.display()))
}

fn optional(row: &HashMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_usize(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<usize> {
    let value = required(row, name, path)?;
    value.parse::<usize>().with_context(|| {
        format!(
            "candidate TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        )
    })
}

fn parse_bool(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<bool> {
    let value = required(row, name, path)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "candidate TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('t') => output.push('\t'),
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('\\') | None => output.push('\\'),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
        }
    }
    output
}
