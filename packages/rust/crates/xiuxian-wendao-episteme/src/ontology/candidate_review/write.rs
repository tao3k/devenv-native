//! Writers for candidate review artifacts.

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};

use super::{
    model::{ReviewMetrics, ReviewRow},
    types::{EpistemeOntologyCandidateReviewReport, REVIEW_COLUMNS},
};

pub(super) fn write_review_tsv(path: &Path, rows: &[ReviewRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "{}", REVIEW_COLUMNS.join("\t"))?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            escape_tsv(&row.record_kind),
            row.review_decision,
            row.quality_score,
            row.evidence_strength,
            escape_tsv(&row.issue_codes.join(",")),
            row.promotion_precondition_met,
            escape_tsv(&row.source_file_id),
            escape_tsv(&row.source_queue_id),
            escape_tsv(&row.extraction_run_id),
            escape_tsv(&row.suggested_term_key),
            escape_tsv(&row.label)
        )?;
    }
    Ok(())
}

pub(super) fn write_review_org(
    path: &Path,
    rows: &[ReviewRow],
    metrics: &ReviewMetrics,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Private Ontology Candidate Review Ledger")?;
    writeln!(file)?;
    writeln!(file, "* Candidate review ledger")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_candidate_review_gate")?;
    writeln!(file, ":ONTOLOGY_KIND: dataset_mapping")?;
    writeln!(file, ":LIFECYCLE_STATE: review")?;
    writeln!(file, ":PROMOTION_STATE: blocked_pending_review")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This ledger is the authoritative candidate review input. TSV files are generated projections."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| review_row_count | {} |", rows.len())?;
    writeln!(
        file,
        "| promotion_precondition_met_count | {} |",
        metrics.promotion_precondition_met_count
    )?;
    writeln!(
        file,
        "| blocked_invalid_count | {} |",
        metrics.blocked_invalid_count
    )?;
    writeln!(
        file,
        "| needs_evidence_count | {} |",
        metrics.needs_evidence_count
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "| {} |",
        REVIEW_COLUMNS
            .iter()
            .map(|column| escape_org_table_cell(column))
            .collect::<Vec<_>>()
            .join(" | ")
    )?;
    writeln!(
        file,
        "| {} |",
        REVIEW_COLUMNS
            .iter()
            .map(|_| "-")
            .collect::<Vec<_>>()
            .join(" | ")
    )?;
    for row in rows {
        writeln!(
            file,
            "| {} |",
            review_row_org_cells(row)
                .iter()
                .map(|cell| escape_org_table_cell(cell))
                .collect::<Vec<_>>()
                .join(" | ")
        )?;
    }
    Ok(())
}

fn review_row_org_cells(row: &ReviewRow) -> [String; 12] {
    [
        row.record_id.clone(),
        row.record_kind.clone(),
        row.review_decision.to_string(),
        row.quality_score.to_string(),
        row.evidence_strength.to_string(),
        row.issue_codes.join(","),
        row.promotion_precondition_met.to_string(),
        row.source_file_id.clone(),
        row.source_queue_id.clone(),
        row.extraction_run_id.clone(),
        row.suggested_term_key.clone(),
        row.label.clone(),
    ]
}

pub(super) fn write_json(
    path: &Path,
    report: &EpistemeOntologyCandidateReviewReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn escape_org_table_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\vert{}")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
