use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{
    EpistemeOntologyStructuralFactsReasoningFillPlanItem,
    EpistemeOntologyStructuralFactsReasoningFillPlanReport,
};

pub(super) fn write_fill_plan_tsv(
    path: &Path,
    items: &[EpistemeOntologyStructuralFactsReasoningFillPlanItem],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "fill_item_id\tworkflow_key\tactivity_kind\tqianji_activity_contract\tseed_id\tseed_kind\tpacket_id\treasoning_task_kind\tevidence_target_intent\tevidence_anchor_kind\tevidence_structure_hint\tdocument_id\tdocument_anchor_id\tfile_id\tdomain_id\tsource_contract_id\trelative_path\tcategory\tlanguage\textraction_route\tsource_content_hash\tevidence_id\ttarget_ledger_field_group\toutput_contract\treview_decision_required\tpromotion_decision_required\tsource_text_read\tllm_executed\tworkflow_executed\tsource_mutation_allowed\trdf_mutation_allowed\tontology_truth\tstatus"
    )?;
    for item in items {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&item.fill_item_id),
            item.workflow_key,
            item.activity_kind,
            item.qianji_activity_contract,
            escape_tsv(&item.seed_id),
            escape_tsv(&item.seed_kind),
            escape_tsv(&item.packet_id),
            escape_tsv(&item.reasoning_task_kind),
            escape_tsv(&item.evidence_target_intent),
            escape_tsv(&item.evidence_anchor_kind),
            escape_tsv(&item.evidence_structure_hint),
            escape_tsv(&item.document_id),
            escape_tsv(&item.document_anchor_id),
            escape_tsv(&item.file_id),
            escape_tsv(&item.domain_id),
            escape_tsv(&item.source_contract_id),
            escape_tsv(&item.relative_path),
            escape_tsv(&item.category),
            escape_tsv(&item.language),
            escape_tsv(&item.extraction_route),
            escape_tsv(&item.source_content_hash),
            escape_tsv(&item.evidence_id),
            item.target_ledger_field_group,
            item.output_contract,
            item.review_decision_required,
            item.promotion_decision_required,
            item.execution.source_text_read,
            item.execution.llm_executed,
            item.execution.workflow_executed,
            item.safety.source_mutation_allowed,
            item.safety.rdf_mutation_allowed,
            item.safety.ontology_truth,
            item.status
        )?;
    }
    Ok(())
}

pub(super) fn write_fill_plan_org(
    path: &Path,
    report: &EpistemeOntologyStructuralFactsReasoningFillPlanReport,
    items: &[EpistemeOntologyStructuralFactsReasoningFillPlanItem],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "#+TITLE: Episteme Structural Facts Reasoning Fill Plan"
    )?;
    writeln!(file)?;
    writeln!(file, "* Reasoning fill plan")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(
        file,
        ":WENDAO_KIND: episteme_structural_facts_reasoning_fill_plan"
    )?;
    writeln!(file, ":WORKFLOW_KEY: episteme_ontology_reasoning_fill")?;
    writeln!(file, ":SOURCE_TEXT_READ: false")?;
    writeln!(file, ":LLM_EXECUTED: false")?;
    writeln!(file, ":WORKFLOW_EXECUTED: false")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":RDF_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This generated plan describes workflow inputs for filling pending Org proposal rows. It does not execute Qianji, call a model, read source text, or mutate RDF."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| run_id | {} |", org_cell(&report.run_id))?;
    writeln!(
        file,
        "| reasoning_ledger_seed_json | {} |",
        org_cell(&report.reasoning_ledger_seed_json.display().to_string())
    )?;
    writeln!(file, "| seed_row_count | {} |", report.seed_row_count)?;
    writeln!(
        file,
        "| object_fill_item_count | {} |",
        report.object_fill_item_count
    )?;
    writeln!(
        file,
        "| relation_fill_item_count | {} |",
        report.relation_fill_item_count
    )?;
    writeln!(
        file,
        "| service_catalog_fill_item_count | {} |",
        report.service_catalog_fill_item_count
    )?;
    writeln!(
        file,
        "| object_instance_fill_item_count | {} |",
        report.object_instance_fill_item_count
    )?;
    writeln!(file, "| fill_item_count | {} |", report.fill_item_count)?;
    writeln!(file, "| source_text_read | false |")?;
    writeln!(file, "| llm_executed | false |")?;
    writeln!(file, "| workflow_executed | false |")?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| rdf_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    write_work_item_table(&mut file, items)?;
    Ok(())
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn write_work_item_table(
    file: &mut File,
    items: &[EpistemeOntologyStructuralFactsReasoningFillPlanItem],
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "** Workflow fill items")?;
    writeln!(file)?;
    writeln!(
        file,
        "| fill_item_id | workflow_key | activity_kind | seed_id | seed_kind | target_intent | structure_hint | packet_id | document_id | anchor | file_id | field_group | status |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|-|-|-|-|-|")?;
    for item in items {
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            org_cell(&item.fill_item_id),
            item.workflow_key,
            item.activity_kind,
            org_cell(&item.seed_id),
            org_cell(&item.seed_kind),
            org_cell(&item.evidence_target_intent),
            org_cell(&item.evidence_structure_hint),
            org_cell(&item.packet_id),
            org_cell(&item.document_id),
            org_cell(&item.document_anchor_id),
            org_cell(&item.file_id),
            item.target_ledger_field_group,
            item.status
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
