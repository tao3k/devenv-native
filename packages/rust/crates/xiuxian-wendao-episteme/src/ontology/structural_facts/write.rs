use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{
    EpistemeOntologyStructuralFactsAnchorRow, EpistemeOntologyStructuralFactsDocumentRow,
    EpistemeOntologyStructuralFactsRelationRow, EpistemeOntologyStructuralFactsReport,
};

pub(super) fn write_documents_tsv(
    path: &Path,
    rows: &[EpistemeOntologyStructuralFactsDocumentRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "document_id\tfile_id\tdomain_id\tsource_contract_id\tsource_manifest_path\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\tsource_exists\tbyte_size_matches\tsha256_matches\tontology_truth\tstatus"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.document_id),
            escape_tsv(&row.file_id),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.source_manifest_path),
            escape_tsv(&row.relative_path),
            escape_tsv(&row.extension),
            row.byte_size,
            escape_tsv(&row.sha256),
            escape_tsv(&row.category),
            escape_tsv(&row.language),
            escape_tsv(&row.extraction_route),
            row.source_exists,
            row.byte_size_matches,
            option_bool(row.sha256_matches),
            row.ontology_truth,
            escape_tsv(&row.status)
        )?;
    }
    Ok(())
}

pub(super) fn write_anchors_tsv(
    path: &Path,
    rows: &[EpistemeOntologyStructuralFactsAnchorRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "anchor_id\tanchor_kind\tdocument_id\tfile_id\tparent_anchor_id\tdomain_id\tsource_contract_id\trelative_path\tpath_depth\torder_key\tlanguage\textraction_route\tsource_content_hash\tontology_truth\tstatus"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.anchor_id),
            escape_tsv(&row.anchor_kind),
            escape_tsv(&row.document_id),
            escape_tsv(&row.file_id),
            escape_tsv(&row.parent_anchor_id),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.relative_path),
            row.path_depth,
            row.order_key,
            escape_tsv(&row.language),
            escape_tsv(&row.extraction_route),
            escape_tsv(&row.source_content_hash),
            row.ontology_truth,
            escape_tsv(&row.status)
        )?;
    }
    Ok(())
}

pub(super) fn write_relations_tsv(
    path: &Path,
    rows: &[EpistemeOntologyStructuralFactsRelationRow],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "relation_id\trelation_kind\tsource_anchor_id\ttarget_anchor_id\tdocument_id\tfile_id\tdomain_id\tsource_contract_id\tevidence_path\torder_key\tontology_truth\tstatus"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.relation_id),
            escape_tsv(&row.relation_kind),
            escape_tsv(&row.source_anchor_id),
            escape_tsv(&row.target_anchor_id),
            escape_tsv(&row.document_id),
            escape_tsv(&row.file_id),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.source_contract_id),
            escape_tsv(&row.evidence_path),
            row.order_key,
            row.ontology_truth,
            escape_tsv(&row.status)
        )?;
    }
    Ok(())
}

pub(super) fn write_structural_facts_org(
    path: &Path,
    report: &EpistemeOntologyStructuralFactsReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Episteme Structural Facts Seed")?;
    writeln!(file)?;
    writeln!(file, "* Structural Facts seed")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: episteme_structural_facts_seed")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":EXTRACTION_EXECUTED: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This deterministic seed records private source structure only. It does not promote raw rows into ontology truth."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| run_id | {} |", org_cell(&report.run_id))?;
    writeln!(file, "| domain_count | {} |", report.domain_count)?;
    writeln!(
        file,
        "| source_manifest_count | {} |",
        report.source_manifest_count
    )?;
    writeln!(file, "| file_count | {} |", report.file_count)?;
    writeln!(file, "| document_count | {} |", report.document_count)?;
    writeln!(file, "| anchor_count | {} |", report.anchor_count)?;
    writeln!(file, "| relation_count | {} |", report.relation_count)?;
    writeln!(
        file,
        "| read_model_object_count | {} |",
        report.read_model_object_count
    )?;
    writeln!(
        file,
        "| read_model_relation_count | {} |",
        report.read_model_relation_count
    )?;
    writeln!(
        file,
        "| read_model_projection_state_count | {} |",
        report.read_model_projection_state_count
    )?;
    writeln!(
        file,
        "| read_model_quality_passed | {} |",
        report.read_model_quality_passed
    )?;
    writeln!(file, "| extraction_executed | false |")?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file, "| validation_mode | {:?} |", report.validation_mode)?;
    writeln!(file, "| full_hash_checked | {} |", report.full_hash_checked)?;
    writeln!(file, "| rdf_seed_ttl | {} |", report.rdf_seed_ttl.display())?;
    writeln!(
        file,
        "| read_model_objects_parquet | {} |",
        report.read_model_objects_parquet.display()
    )?;
    writeln!(
        file,
        "| read_model_relations_parquet | {} |",
        report.read_model_relations_parquet.display()
    )?;
    Ok(())
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

pub(super) fn write_string(path: &Path, value: &str) -> Result<()> {
    let mut file = create_file(path)?;
    file.write_all(value.as_bytes())
        .with_context(|| format!("failed to write `{}`", path.display()))
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn option_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    }
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
