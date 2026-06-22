use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use xiuxian_wendao_parsers::{OrgOntologyAuthoringTable, compile_org_ontology_authoring_document};

use super::model::{
    CandidateEvidenceRecord, CandidateObjectRecord, CandidateRelationRecord, DraftInputs,
    EVIDENCE_TSV, OBJECTS_TSV, QUALITY_REPORT_JSON, QualityReport, RELATIONS_TSV, REVIEW_ORG,
    ReviewRecord, TsvTable,
};

pub(super) fn read_draft_inputs(run_dir: &Path) -> Result<DraftInputs> {
    let quality = read_quality_report(run_dir.join(QUALITY_REPORT_JSON).as_path())?;
    let objects = read_candidate_objects(run_dir.join(OBJECTS_TSV).as_path())?;
    let relations = read_candidate_relations(run_dir.join(RELATIONS_TSV).as_path())?;
    let evidence = read_candidate_evidence(run_dir.join(EVIDENCE_TSV).as_path())?;
    let reviews = read_candidate_review(run_dir.join(REVIEW_ORG).as_path())?;
    Ok(DraftInputs {
        objects,
        relations,
        evidence,
        reviews_by_id: reviews
            .into_iter()
            .map(|review| (review.record_id.clone(), review))
            .collect(),
        quality,
    })
}

fn read_quality_report(path: &Path) -> Result<QualityReport> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(raw.as_str())
        .with_context(|| format!("failed to parse `{}`", path.display()))
}

fn read_candidate_objects(path: &Path) -> Result<Vec<CandidateObjectRecord>> {
    read_tsv(path)?
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
                source_path: optional(row, "source_path"),
                category: optional(row, "category"),
                language: optional(row, "language"),
                extraction_route: optional(row, "extraction_route"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
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

fn read_candidate_relations(path: &Path) -> Result<Vec<CandidateRelationRecord>> {
    read_tsv(path)?
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

fn read_candidate_evidence(path: &Path) -> Result<Vec<CandidateEvidenceRecord>> {
    read_tsv(path)?
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

fn read_candidate_review(path: &Path) -> Result<Vec<ReviewRecord>> {
    read_candidate_review_org_table(path)?
        .rows
        .iter()
        .map(|row| {
            Ok(ReviewRecord {
                record_id: required_org(row, "record_id", path)?,
                record_kind: required_org(row, "record_kind", path)?,
                review_decision: required_org(row, "review_decision", path)?,
                quality_score: parse_usize_org(row, "quality_score", path)?,
                evidence_strength: required_org(row, "evidence_strength", path)?,
                issue_codes: optional_org(row, "issue_codes"),
                promotion_precondition_met: parse_bool_org(
                    row,
                    "promotion_precondition_met",
                    path,
                )?,
                suggested_term_key: optional_org(row, "suggested_term_key"),
                label: optional_org(row, "label"),
            })
        })
        .collect()
}

fn read_candidate_review_org_table(path: &Path) -> Result<OrgOntologyAuthoringTable> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let document = compile_org_ontology_authoring_document(&content, path.display().to_string())
        .with_context(|| format!("failed to parse candidate review Org `{}`", path.display()))?;
    document
        .sections
        .into_iter()
        .flat_map(|section| section.tables)
        .find(|table| table.kind == "candidate_review")
        .with_context(|| {
            format!(
                "candidate review Org `{}` has no candidate_review table",
                path.display()
            )
        })
}

fn read_tsv(path: &Path) -> Result<TsvTable> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .with_context(|| format!("candidate TSV `{}` is empty", path.display()))?;
    let headers = header_line
        .split('\t')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
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
    row_number: usize,
    line: &str,
) -> Result<HashMap<String, String>> {
    let values = line.split('\t').collect::<Vec<_>>();
    if values.len() != headers.len() {
        anyhow::bail!(
            "candidate TSV `{}` row {} has {} columns, expected {}",
            path.display(),
            row_number,
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

fn required_org(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "candidate review Org `{}` missing `{name}` column",
            path.display()
        )
    })
}

fn optional_org(row: &BTreeMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_usize_org(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<usize> {
    let value = required_org(row, name, path)?;
    value.parse::<usize>().with_context(|| {
        format!(
            "candidate review Org `{}` has invalid `{name}` value `{value}`",
            path.display()
        )
    })
}

fn parse_bool_org(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<bool> {
    let value = required_org(row, name, path)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "candidate review Org `{}` has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn unescape_tsv(value: &str) -> String {
    value.to_string()
}
