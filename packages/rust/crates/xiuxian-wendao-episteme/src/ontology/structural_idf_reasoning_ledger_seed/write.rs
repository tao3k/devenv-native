use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{
    EpistemeOntologyStructuralIdfReasoningLedgerSeedReport,
    EpistemeOntologyStructuralIdfReasoningLedgerSeedRow,
};

pub(super) fn write_seed_tsv(
    path: &Path,
    rows: &[EpistemeOntologyStructuralIdfReasoningLedgerSeedRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "seed_id\tseed_kind\tpacket_id\treasoning_task_kind\tdocument_id\tdocument_anchor_id\tfile_id\tdomain_id\tsource_contract_id\trelative_path\tcategory\tlanguage\textraction_route\tsource_content_hash\tevidence_id\tproposed_object_id\tproposed_object_type\tproposed_label\tproposed_relation_id\tproposed_source_object_id\tproposed_predicate\tproposed_target_object_id\treview_decision\tpromotion_decision\treviewer_id\tsource_text_read\tllm_executed\tsource_mutation_allowed\tontology_truth\tstatus"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.seed_id),
            row.seed_kind,
            escape_tsv(&row.packet_id),
            escape_tsv(&row.reasoning_task_kind),
            escape_tsv(&row.document_id),
            escape_tsv(&row.document_anchor_id),
            escape_tsv(&row.file_id),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.relative_path),
            escape_tsv(&row.category),
            escape_tsv(&row.language),
            escape_tsv(&row.extraction_route),
            escape_tsv(&row.source_content_hash),
            escape_tsv(&row.evidence_id),
            escape_tsv(&row.proposed_object_id),
            escape_tsv(&row.proposed_object_type),
            escape_tsv(&row.proposed_label),
            escape_tsv(&row.proposed_relation_id),
            escape_tsv(&row.proposed_source_object_id),
            escape_tsv(&row.proposed_predicate),
            escape_tsv(&row.proposed_target_object_id),
            row.review_decision,
            row.promotion_decision,
            escape_tsv(&row.reviewer_id),
            row.execution.source_text_read,
            row.execution.llm_executed,
            row.safety.source_mutation_allowed,
            row.safety.ontology_truth,
            row.status
        )?;
    }
    Ok(())
}

pub(super) fn write_seed_org(
    path: &Path,
    report: &EpistemeOntologyStructuralIdfReasoningLedgerSeedReport,
    rows: &[EpistemeOntologyStructuralIdfReasoningLedgerSeedRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "#+TITLE: Episteme Structural IDF Reasoning Ledger Seed"
    )?;
    writeln!(file)?;
    writeln!(file, "* Reasoning ledger seed")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(
        file,
        ":WENDAO_KIND: episteme_structural_idf_reasoning_ledger_seed"
    )?;
    writeln!(file, ":SOURCE_TEXT_READ: false")?;
    writeln!(file, ":LLM_EXECUTED: false")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This generated seed gives reviewers fillable proposal rows. It does not infer semantic object ids, relation predicates, or labels."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| run_id | {} |", org_cell(&report.run_id))?;
    writeln!(
        file,
        "| reasoning_packet_json | {} |",
        org_cell(&report.reasoning_packet_json.display().to_string())
    )?;
    writeln!(file, "| packet_row_count | {} |", report.packet_row_count)?;
    writeln!(
        file,
        "| object_seed_row_count | {} |",
        report.object_seed_row_count
    )?;
    writeln!(
        file,
        "| relation_seed_row_count | {} |",
        report.relation_seed_row_count
    )?;
    writeln!(file, "| seed_row_count | {} |", report.seed_row_count)?;
    writeln!(file, "| source_text_read | false |")?;
    writeln!(file, "| llm_executed | false |")?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    write_object_seed_table(&mut file, rows)?;
    write_relation_seed_table(&mut file, rows)?;
    Ok(())
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn write_object_seed_table(
    file: &mut File,
    rows: &[EpistemeOntologyStructuralIdfReasoningLedgerSeedRow],
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "** Object proposal slots")?;
    writeln!(file)?;
    writeln!(
        file,
        "| seed_id | packet_id | evidence_id | document_id | anchor | file_id | proposed_object_id | proposed_object_type | proposed_label | review_decision | promotion_decision | reviewer_id |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|-|-|-|-|")?;
    for row in rows
        .iter()
        .filter(|row| row.seed_kind == "object_proposal_slot")
    {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} |  |  |  | {} | {} |  |",
            org_cell(&row.seed_id),
            org_cell(&row.packet_id),
            org_cell(&row.evidence_id),
            org_cell(&row.document_id),
            org_cell(&row.document_anchor_id),
            org_cell(&row.file_id),
            row.review_decision,
            row.promotion_decision
        )?;
    }
    Ok(())
}

fn write_relation_seed_table(
    file: &mut File,
    rows: &[EpistemeOntologyStructuralIdfReasoningLedgerSeedRow],
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "** Relation proposal slots")?;
    writeln!(file)?;
    writeln!(
        file,
        "| seed_id | packet_id | evidence_id | document_id | anchor | file_id | proposed_relation_id | proposed_source_object_id | proposed_predicate | proposed_target_object_id | review_decision | promotion_decision | reviewer_id |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|-|-|-|-|-|")?;
    for row in rows
        .iter()
        .filter(|row| row.seed_kind == "relation_proposal_slot")
    {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} |  |  |  |  | {} | {} |  |",
            org_cell(&row.seed_id),
            org_cell(&row.packet_id),
            org_cell(&row.evidence_id),
            org_cell(&row.document_id),
            org_cell(&row.document_anchor_id),
            org_cell(&row.file_id),
            row.review_decision,
            row.promotion_decision
        )?;
    }
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn org_cell(value: &str) -> String {
    value.replace('|', "\\vert{}").replace('\n', " ")
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
