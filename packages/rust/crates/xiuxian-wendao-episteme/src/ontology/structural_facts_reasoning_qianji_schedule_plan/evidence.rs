use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::input::ReasoningFillPlanInputRow;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ContextEvidenceRow {
    pub extraction_run_id: String,
    pub cache_output_path: String,
    pub queue_id: String,
    pub file_id: String,
    pub relative_path: String,
    pub category: String,
    pub language: String,
    pub extraction_route: String,
    pub source_sha256: String,
    pub text_sha256: String,
    pub text_char_count: usize,
    pub extracted_text: String,
}

#[derive(Debug, Deserialize)]
struct CacheOutputRow {
    status: String,
    queue_id: String,
    file_id: String,
    relative_path: String,
    category: String,
    language: String,
    extraction_route: String,
    source_sha256: String,
    text_sha256: Option<String>,
    text_char_count: Option<usize>,
    extracted_text: Option<String>,
    ontology_truth: Option<bool>,
    raw_to_rdf_promotion_allowed: Option<bool>,
}

pub(super) type ContextEvidenceByFileId = BTreeMap<String, Vec<ContextEvidenceRow>>;

pub(super) fn read_context_evidence_by_file_id(
    extraction_run_root: &Path,
    extraction_run_ids: &[String],
    fill_rows: &[ReasoningFillPlanInputRow],
) -> Result<ContextEvidenceByFileId> {
    let file_ids = fill_rows
        .iter()
        .map(|row| row.file_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for run_id in extraction_run_ids {
        validate_run_id(run_id)?;
        let outputs_dir = extraction_run_root.join(run_id).join("outputs");
        if !outputs_dir.is_dir() {
            bail!(
                "context evidence extraction run `{}` has no outputs directory `{}`",
                run_id,
                outputs_dir.display()
            );
        }
        collect_context_evidence_for_run(outputs_dir.as_path(), run_id, &file_ids, &mut rows)?;
    }
    rows.sort_by(|left, right| {
        left.file_id
            .cmp(&right.file_id)
            .then_with(|| left.extraction_run_id.cmp(&right.extraction_run_id))
            .then_with(|| left.queue_id.cmp(&right.queue_id))
    });

    let mut by_file_id = BTreeMap::<String, Vec<ContextEvidenceRow>>::new();
    for row in rows {
        by_file_id.entry(row.file_id.clone()).or_default().push(row);
    }
    Ok(by_file_id)
}

fn collect_context_evidence_for_run(
    outputs_dir: &Path,
    run_id: &str,
    file_ids: &BTreeSet<&str>,
    rows: &mut Vec<ContextEvidenceRow>,
) -> Result<()> {
    for entry in fs::read_dir(outputs_dir)
        .with_context(|| format!("failed to read `{}`", outputs_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read `{}`", outputs_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Some(row) = read_context_evidence_output(path.as_path(), run_id)? else {
            continue;
        };
        if file_ids.contains(row.file_id.as_str()) {
            rows.push(row);
        }
    }
    Ok(())
}

fn read_context_evidence_output(path: &Path, run_id: &str) -> Result<Option<ContextEvidenceRow>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let row = serde_json::from_str::<CacheOutputRow>(&raw)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if row.status != "succeeded" {
        return Ok(None);
    }
    if row.ontology_truth.unwrap_or(false) || row.raw_to_rdf_promotion_allowed.unwrap_or(false) {
        bail!(
            "context evidence cache output `{}` must not be ontology truth or raw-to-RDF promotable",
            path.display()
        );
    }
    let extracted_text = row.extracted_text.unwrap_or_default();
    if extracted_text.trim().is_empty() {
        bail!(
            "context evidence cache output `{}` has empty extracted_text",
            path.display()
        );
    }
    let text_sha256 = row
        .text_sha256
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| sha256_text(&extracted_text));
    let text_char_count = row
        .text_char_count
        .unwrap_or_else(|| extracted_text.chars().count());
    Ok(Some(ContextEvidenceRow {
        extraction_run_id: run_id.to_owned(),
        cache_output_path: path.display().to_string(),
        queue_id: row.queue_id,
        file_id: row.file_id,
        relative_path: row.relative_path,
        category: row.category,
        language: row.language,
        extraction_route: row.extraction_route,
        source_sha256: row.source_sha256,
        text_sha256,
        text_char_count,
        extracted_text,
    }))
}

fn sha256_text(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("invalid context evidence extraction run id `{run_id}`");
    }
    Ok(())
}
