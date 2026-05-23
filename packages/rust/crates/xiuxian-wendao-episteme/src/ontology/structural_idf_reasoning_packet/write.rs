use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{
    EpistemeOntologyStructuralIdfReasoningPacketReport,
    EpistemeOntologyStructuralIdfReasoningPacketRow,
};

pub(super) fn write_packet_tsv(
    path: &Path,
    rows: &[EpistemeOntologyStructuralIdfReasoningPacketRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "packet_id\tpacket_kind\treasoning_task_kind\tdocument_id\tdocument_anchor_id\tfile_id\tdomain_id\tsource_contract_id\trelative_path\tcategory\tlanguage\textraction_route\tsource_content_hash\tevidence_action\tontology_truth\tstatus"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.packet_id),
            row.packet_kind,
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
            row.evidence_action,
            row.ontology_truth,
            row.status
        )?;
    }
    Ok(())
}

pub(super) fn write_packet_org(
    path: &Path,
    report: &EpistemeOntologyStructuralIdfReasoningPacketReport,
    rows: &[EpistemeOntologyStructuralIdfReasoningPacketRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Episteme Structural IDF Reasoning Packet")?;
    writeln!(file)?;
    writeln!(file, "* Reasoning packet")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(
        file,
        ":WENDAO_KIND: episteme_structural_idf_reasoning_packet"
    )?;
    writeln!(file, ":SOURCE_TEXT_READ: false")?;
    writeln!(file, ":LLM_EXECUTED: false")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This packet is a bounded proposal input surface. It cites generated structural IDF rows and does not contain raw private source text."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| run_id | {} |", org_cell(&report.run_id))?;
    writeln!(
        file,
        "| structural_idf_json | {} |",
        org_cell(&report.structural_idf_json.display().to_string())
    )?;
    writeln!(file, "| packet_row_count | {} |", report.packet_row_count)?;
    writeln!(
        file,
        "| skipped_by_filter_count | {} |",
        report.skipped_by_filter_count
    )?;
    writeln!(
        file,
        "| skipped_by_limit_count | {} |",
        report.skipped_by_limit_count
    )?;
    writeln!(file, "| source_text_read | false |")?;
    writeln!(file, "| llm_executed | false |")?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file)?;
    writeln!(file, "** Packet rows")?;
    writeln!(file)?;
    writeln!(
        file,
        "| packet_id | task | file_id | relative_path | category | route | anchor | hash |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|")?;
    for row in rows {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            org_cell(&row.packet_id),
            org_cell(&row.reasoning_task_kind),
            org_cell(&row.file_id),
            org_cell(&row.relative_path),
            org_cell(&row.category),
            org_cell(&row.extraction_route),
            org_cell(&row.document_anchor_id),
            org_cell(&row.source_content_hash)
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
