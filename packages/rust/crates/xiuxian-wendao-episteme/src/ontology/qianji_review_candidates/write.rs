//! TSV and JSON writers for Qianji review candidate import artifacts.

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{CandidateEvidenceRow, CandidateObjectRow, CandidateRelationRow};

pub(super) fn write_objects_tsv(path: &Path, rows: &[CandidateObjectRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\tcandidate_kind\tstatus\tlabel\tsuggested_term_key\tsuggested_term_label\tsource_file_id\tsource_queue_id\tsource_path\tcategory\tlanguage\textraction_route\textraction_run_id\tsource_sha256\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\traw_to_rdf_promotion_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\tontology_candidate.qianji_object_patch\tcandidate\t{}\t{}\t{}\t{}\t\t{}\t\t\tqianji_episteme_review\t\t\t{}\t{}\treview_required\tblocked_pending_review\tfalse\tfalse",
            tsv(row.candidate_id.as_str()),
            tsv(row.label.as_str()),
            tsv(row.suggested_term_key.as_str()),
            tsv(row.suggested_term_key.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.source_path.as_str()),
            tsv(row.evidence_sha256.as_str()),
            row.text_char_count
        )?;
    }
    Ok(())
}

pub(super) fn write_relations_tsv(path: &Path, rows: &[CandidateRelationRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "candidate_id\trelation_kind\tsource_candidate_id\ttarget_candidate_id\tsource_file_id\tsource_queue_id\textraction_run_id\tevidence_sha256\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t\t\t{}\treview_required\tblocked_pending_review\tfalse",
            tsv(row.candidate_id.as_str()),
            tsv(row.relation_kind.as_str()),
            tsv(row.source_candidate_id.as_str()),
            tsv(row.target_candidate_id.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.evidence_sha256.as_str())
        )?;
    }
    Ok(())
}

pub(super) fn write_evidence_tsv(path: &Path, rows: &[CandidateEvidenceRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "evidence_id\tevidence_kind\tsource_file_id\tsource_queue_id\tsource_path\tsource_sha256\textraction_run_id\tcache_output_path\tevidence_sha256\ttext_char_count\treview_status\tpromotion_status\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\tontology_candidate.qianji_review_evidence\t{}\t\t{}\t\t\t\t{}\t{}\treview_required\tblocked_pending_review\tfalse",
            tsv(row.evidence_id.as_str()),
            tsv(row.source_file_id.as_str()),
            tsv(row.source_path.as_str()),
            tsv(row.evidence_sha256.as_str()),
            row.text_char_count
        )?;
    }
    Ok(())
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
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

fn tsv(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ").trim().to_owned()
}
