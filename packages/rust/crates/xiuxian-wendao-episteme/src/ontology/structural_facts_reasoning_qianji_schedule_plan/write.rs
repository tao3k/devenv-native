use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::types::{
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem,
    EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport,
};

pub(super) fn write_schedule_plan_tsv(
    path: &Path,
    items: &[EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "schedule_item_id\tschedule_contract\tadmission_kind\tqianji_run_id\tfill_item_id\tworkflow_key\tactivity_kind\tseed_id\tpacket_id\tevidence_target_intent\tevidence_anchor_kind\tevidence_structure_hint\tdocument_id\tdocument_anchor_id\tfile_id\tevidence_id\tfield_group\treasoning_context_shard_id\treasoning_context_shard_index\treasoning_context_shard_count\treasoning_context_shard_row_start\treasoning_context_shard_row_end\tactivity_id\tactivity_type\ttask_queue\tinput_artifact_id\tinput_artifact_kind\tidempotency_key\tsource_text_read\tllm_executed\tworkflow_executed\tqianji_ledger_mutated\thot_state_enqueued\tsource_mutation_allowed\trdf_mutation_allowed\tontology_truth\tstatus"
    )?;
    for item in items {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&item.schedule_item_id),
            item.schedule_contract,
            item.admission_kind,
            escape_tsv(&item.qianji_run_id),
            escape_tsv(&item.fill_item_id),
            escape_tsv(&item.workflow_key),
            escape_tsv(&item.activity_kind),
            escape_tsv(&item.seed_id),
            escape_tsv(&item.packet_id),
            escape_tsv(&item.evidence_target_intent),
            escape_tsv(&item.evidence_anchor_kind),
            escape_tsv(&item.evidence_structure_hint),
            escape_tsv(&item.document_id),
            escape_tsv(&item.document_anchor_id),
            escape_tsv(&item.file_id),
            escape_tsv(&item.evidence_id),
            escape_tsv(&item.field_group),
            escape_tsv(item.reasoning_context_shard_id.as_deref().unwrap_or("")),
            item.reasoning_context_shard_index
                .map(|value| value.to_string())
                .unwrap_or_default(),
            item.reasoning_context_shard_count
                .map(|value| value.to_string())
                .unwrap_or_default(),
            item.reasoning_context_shard_row_start
                .map(|value| value.to_string())
                .unwrap_or_default(),
            item.reasoning_context_shard_row_end
                .map(|value| value.to_string())
                .unwrap_or_default(),
            escape_tsv(&item.activity_task.activity_id),
            escape_tsv(&item.activity_task.activity_type),
            escape_tsv(&item.activity_task.task_queue),
            escape_tsv(&item.activity_task.input_ref.artifact_id),
            escape_tsv(&item.activity_task.input_ref.artifact_kind),
            escape_tsv(&item.activity_task.idempotency_key),
            item.execution.input.source_text_read,
            item.execution.input.llm_executed,
            item.execution.runtime.workflow_executed,
            item.execution.runtime.qianji_ledger_mutated,
            item.execution.runtime.hot_state_enqueued,
            item.safety.source_mutation_allowed,
            item.safety.rdf_mutation_allowed,
            item.safety.ontology_truth,
            item.status
        )?;
    }
    Ok(())
}

pub(super) fn write_schedule_plan_org(
    path: &Path,
    report: &EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport,
    items: &[EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem],
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Episteme Qianji Reasoning Schedule Plan")?;
    writeln!(file)?;
    writeln!(file, "* Qianji reasoning schedule plan")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(
        file,
        ":WENDAO_KIND: episteme_qianji_reasoning_schedule_plan"
    )?;
    writeln!(
        file,
        ":SCHEDULE_CONTRACT: xiuxian.qianji.control.activity_schedule_admission_plan.v1"
    )?;
    writeln!(file, ":SOURCE_TEXT_READ: false")?;
    writeln!(file, ":LLM_EXECUTED: false")?;
    writeln!(file, ":WORKFLOW_EXECUTED: false")?;
    writeln!(file, ":QIANJI_LEDGER_MUTATED: false")?;
    writeln!(file, ":HOT_STATE_ENQUEUED: false")?;
    writeln!(file, ":RDF_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This generated plan describes Qianji activity schedule inputs. It does not append control ledger events, enqueue hot-state work, call a model, read source text, or mutate RDF."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| run_id | {} |", org_cell(&report.run_id))?;
    writeln!(
        file,
        "| qianji_run_id | {} |",
        org_cell(&report.qianji_run_id)
    )?;
    writeln!(
        file,
        "| reasoning_fill_plan_json | {} |",
        org_cell(&report.reasoning_fill_plan_json.display().to_string())
    )?;
    writeln!(file, "| fill_item_count | {} |", report.fill_item_count)?;
    writeln!(
        file,
        "| schedule_item_count | {} |",
        report.schedule_item_count
    )?;
    writeln!(
        file,
        "| skipped_by_filter_count | {} |",
        report.skipped_by_filter_count
    )?;
    writeln!(
        file,
        "| reasoning_context_shard_mode | {} |",
        org_cell(report.reasoning_context_shard_mode.as_str())
    )?;
    writeln!(
        file,
        "| reasoning_context_shard_row_limit | {} |",
        report.reasoning_context_shard_row_limit
    )?;
    writeln!(
        file,
        "| reasoning_context_shard_count | {} |",
        report.reasoning_context_shard_count
    )?;
    writeln!(
        file,
        "| target_ledger_field_group | {} |",
        org_cell(report.target_ledger_field_group.as_deref().unwrap_or(""))
    )?;
    writeln!(
        file,
        "| evidence_target_intent | {} |",
        org_cell(report.evidence_target_intent.as_deref().unwrap_or(""))
    )?;
    writeln!(file, "| source_text_read | false |")?;
    writeln!(file, "| llm_executed | false |")?;
    writeln!(file, "| workflow_executed | false |")?;
    writeln!(file, "| qianji_ledger_mutated | false |")?;
    writeln!(file, "| hot_state_enqueued | false |")?;
    writeln!(file, "| rdf_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    write_schedule_item_table(&mut file, items)?;
    Ok(())
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn write_schedule_item_table(
    file: &mut File,
    items: &[EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem],
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "** Schedule admission items")?;
    writeln!(file)?;
    writeln!(
        file,
        "| schedule_item_id | qianji_run_id | fill_item_id | target_intent | structure_hint | reasoning_context_shard_id | shard_rows | activity_id | activity_type | task_queue | evidence_id | field_group | status |"
    )?;
    writeln!(file, "|-|-|-|-|-|-|-|-|-|-|-|-|-|")?;
    for item in items {
        let shard_rows = match (
            item.reasoning_context_shard_row_start,
            item.reasoning_context_shard_row_end,
        ) {
            (Some(start), Some(end)) => format!("{start}-{end}"),
            _ => String::new(),
        };
        writeln!(
            file,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            org_cell(&item.schedule_item_id),
            org_cell(&item.qianji_run_id),
            org_cell(&item.fill_item_id),
            org_cell(&item.evidence_target_intent),
            org_cell(&item.evidence_structure_hint),
            org_cell(item.reasoning_context_shard_id.as_deref().unwrap_or("")),
            org_cell(&shard_rows),
            org_cell(&item.activity_task.activity_id),
            item.activity_task.activity_type,
            item.activity_task.task_queue,
            org_cell(&item.evidence_id),
            org_cell(&item.field_group),
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
