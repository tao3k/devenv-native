use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::EpistemeOntologySourcePatchRdfReadModelReport;
use crate::ontology::{
    EpistemeOntologySemanticEvidenceRow, EpistemeOntologySemanticObjectRow,
    EpistemeOntologySemanticRelationRow,
};

pub(super) fn write_objects_tsv(
    path: &Path,
    rows: &[EpistemeOntologySemanticObjectRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tkind\ttitle\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\trelation_count\tstatus\tread_model_projection_staleness"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.kind.as_str()),
            escape_tsv(row.title.as_str()),
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            row.evidence_status,
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.review_decision.as_str()),
            escape_tsv(row.promotion_decision.as_str()),
            escape_tsv(row.reviewer_id.as_str()),
            row.relation_count,
            row.status,
            row.read_model_projection_staleness
        )?;
    }
    Ok(())
}

pub(super) fn write_relations_tsv(
    path: &Path,
    rows: &[EpistemeOntologySemanticRelationRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tkind\tsource\ttarget\tdomain\tevidence_id\tevidence_status\ttarget_rdf_file\treview_decision\tpromotion_decision\treviewer_id\tstatus\tread_model_projection_staleness"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.kind.as_str()),
            escape_tsv(row.source.as_str()),
            escape_tsv(row.target.as_str()),
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            row.evidence_status,
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.review_decision.as_str()),
            escape_tsv(row.promotion_decision.as_str()),
            escape_tsv(row.reviewer_id.as_str()),
            row.status,
            row.read_model_projection_staleness
        )?;
    }
    Ok(())
}

pub(super) fn write_evidence_tsv(
    path: &Path,
    rows: &[EpistemeOntologySemanticEvidenceRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "id\tevidence_id\trecord_id\trecord_kind\tontology_target\ttarget\tstatus\tdomain\ttarget_rdf_file\treviewer_id"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(row.id.as_str()),
            escape_tsv(row.evidence_id.as_str()),
            escape_tsv(row.record_id.as_str()),
            escape_tsv(row.record_kind.as_str()),
            escape_tsv(row.ontology_target.as_str()),
            escape_tsv(row.target.as_str()),
            row.status,
            escape_tsv(row.domain.as_str()),
            escape_tsv(row.target_rdf_file.as_str()),
            escape_tsv(row.reviewer_id.as_str())
        )?;
    }
    Ok(())
}

pub(super) fn write_read_model_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchRdfReadModelReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology RDF Source Read-Model")?;
    writeln!(file)?;
    writeln!(file, "* RDF source read-model")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_rdf_read_model")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This read-model was compiled from applied source-patch records in ontology RDF source files."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| rdf_source_row_count | {} |",
        report.rdf_source_row_count
    )?;
    writeln!(
        file,
        "| semantic_object_count | {} |",
        report.semantic_object_count
    )?;
    writeln!(
        file,
        "| semantic_relation_count | {} |",
        report.semantic_relation_count
    )?;
    writeln!(
        file,
        "| semantic_evidence_count | {} |",
        report.semantic_evidence_count
    )?;
    writeln!(
        file,
        "| projection_quality_passed | {} |",
        report.projection_quality_passed
    )?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
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

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
