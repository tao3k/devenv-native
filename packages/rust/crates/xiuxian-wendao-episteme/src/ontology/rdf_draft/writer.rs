use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};

use super::EpistemeOntologyRdfDraftExportReport;

pub(super) fn write_promotion_proposal_org(
    path: &Path,
    report: &EpistemeOntologyRdfDraftExportReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    write_proposal_header(&mut file)?;
    write_proposal_table(&mut file, report)?;
    Ok(())
}

pub(super) fn write_json(path: &Path, report: &EpistemeOntologyRdfDraftExportReport) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

pub(super) fn write_string(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn write_proposal_header(file: &mut impl Write) -> Result<()> {
    writeln!(file, "#+TITLE: Private Ontology Promotion Proposal")?;
    writeln!(file)?;
    writeln!(file, "* RDF draft export")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_promotion_proposal")?;
    writeln!(file, ":PROMOTION_STATE: draft_pending_review")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":RAW_TO_RDF_PROMOTION_ALLOWED: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This proposal was generated from a deterministic candidate review gate."
    )?;
    writeln!(
        file,
        "It is a review artifact and does not mutate source ontology RDF."
    )?;
    writeln!(file)?;
    Ok(())
}

fn write_proposal_table(
    file: &mut impl Write,
    report: &EpistemeOntologyRdfDraftExportReport,
) -> Result<()> {
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| candidate_object_count | {} |",
        report.candidate_object_count
    )?;
    writeln!(
        file,
        "| candidate_relation_count | {} |",
        report.candidate_relation_count
    )?;
    writeln!(
        file,
        "| candidate_evidence_count | {} |",
        report.candidate_evidence_count
    )?;
    writeln!(file, "| review_row_count | {} |", report.review_row_count)?;
    writeln!(
        file,
        "| draft_resource_count | {} |",
        report.draft_resource_count
    )?;
    writeln!(
        file,
        "| draft_statement_count | {} |",
        report.draft_statement_count
    )?;
    writeln!(
        file,
        "| review_gate_passed | {} |",
        report.review_gate_passed
    )?;
    writeln!(
        file,
        "| raw_to_rdf_promotion_allowed | {} |",
        report.raw_to_rdf_promotion_allowed
    )?;
    writeln!(file, "| ontology_truth | {} |", report.ontology_truth)?;
    Ok(())
}
