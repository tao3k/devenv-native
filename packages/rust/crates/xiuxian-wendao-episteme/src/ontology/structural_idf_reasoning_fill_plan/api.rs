use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use super::{
    input::{ReasoningLedgerSeedInputRow, read_reasoning_ledger_seed_rows},
    types::{
        EpistemeOntologyStructuralIdfReasoningFillPlanExecutionFlags,
        EpistemeOntologyStructuralIdfReasoningFillPlanItem,
        EpistemeOntologyStructuralIdfReasoningFillPlanReport,
        EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
        EpistemeOntologyStructuralIdfReasoningFillPlanSafetyFlags,
        REASONING_FILL_PLAN_REPORT_SCHEMA_VERSION, ReasoningFillPlanOutputPaths,
    },
    write::{write_fill_plan_org, write_fill_plan_tsv, write_json},
};

const OBJECT_SEED_KIND: &str = "object_proposal_slot";
const RELATION_SEED_KIND: &str = "relation_proposal_slot";
const OBJECT_FIELD_GROUP: &str = "object_proposal";
const RELATION_FIELD_GROUP: &str = "relation_proposal";
const WORKFLOW_KEY: &str = "episteme_ontology_reasoning_fill";
const ACTIVITY_KIND: &str = "read_targeted_evidence_then_fill_org_proposal";
const QIANJI_ACTIVITY_CONTRACT: &str =
    "xiuxian.qianji.activity.episteme_ontology_reasoning_fill.v1";
const OUTPUT_CONTRACT: &str = "filled_org_proposal_row";
const STATUS_PENDING: &str = "pending_workflow_execution";

/// Compile a structural IDF reasoning ledger seed into workflow fill-plan rows.
///
/// # Errors
///
/// Returns an error when the ledger-seed artifact is missing, malformed, has no
/// selectable rows, attempts to mark ontology truth or mutation, contains
/// duplicate seed ids, or output artifacts cannot be written.
pub fn write_episteme_ontology_structural_idf_reasoning_fill_plan(
    request: &EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralIdfReasoningFillPlanReport> {
    validate_run_id(&request.run_id)?;
    if request.limit == 0 {
        bail!("reasoning fill-plan limit must be greater than zero");
    }

    let seed_rows = read_reasoning_ledger_seed_rows(request.reasoning_ledger_seed_json.as_path())?;
    let (fill_items, skipped_by_limit_count) =
        build_fill_plan_items(&seed_rows, request.run_id.as_str(), request.limit)?;
    let paths = ReasoningFillPlanOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    write_fill_plan_tsv(paths.fill_plan_tsv.as_path(), &fill_items)?;
    write_json(paths.fill_plan_json.as_path(), &fill_items)?;
    let report = build_report(request, &paths, &fill_items, skipped_by_limit_count);
    write_fill_plan_org(paths.fill_plan_org.as_path(), &report, &fill_items)?;
    write_json(paths.report_json.as_path(), &report)?;
    Ok(report)
}

fn build_fill_plan_items(
    seed_rows: &[ReasoningLedgerSeedInputRow],
    run_id: &str,
    limit: usize,
) -> Result<(
    Vec<EpistemeOntologyStructuralIdfReasoningFillPlanItem>,
    usize,
)> {
    let mut seen_seed_ids = BTreeSet::new();
    let mut seen_fill_item_ids = BTreeSet::new();
    let mut fill_items = Vec::new();
    let mut skipped_by_limit_count = 0;

    for seed in seed_rows {
        if !seen_seed_ids.insert(seed.seed_id.as_str()) {
            bail!("duplicate reasoning ledger seed id: {}", seed.seed_id);
        }
        if seen_seed_ids.len() > limit {
            skipped_by_limit_count += 1;
            continue;
        }
        let item = fill_plan_item(seed, run_id);
        if !seen_fill_item_ids.insert(item.fill_item_id.clone()) {
            bail!(
                "duplicate reasoning fill-plan item id: {}",
                item.fill_item_id
            );
        }
        fill_items.push(item);
    }

    if fill_items.is_empty() {
        bail!("reasoning fill-plan selection produced no rows");
    }
    Ok((fill_items, skipped_by_limit_count))
}

fn fill_plan_item(
    seed: &ReasoningLedgerSeedInputRow,
    run_id: &str,
) -> EpistemeOntologyStructuralIdfReasoningFillPlanItem {
    EpistemeOntologyStructuralIdfReasoningFillPlanItem {
        fill_item_id: stable_fill_item_id(run_id, seed.seed_id.as_str()),
        workflow_key: WORKFLOW_KEY,
        activity_kind: ACTIVITY_KIND,
        qianji_activity_contract: QIANJI_ACTIVITY_CONTRACT,
        seed_id: seed.seed_id.clone(),
        seed_kind: seed.seed_kind.clone(),
        packet_id: seed.packet_id.clone(),
        reasoning_task_kind: seed.reasoning_task_kind.clone(),
        document_id: seed.document_id.clone(),
        document_anchor_id: seed.document_anchor_id.clone(),
        file_id: seed.file_id.clone(),
        domain_id: seed.domain_id.clone(),
        source_contract_id: seed.source_contract_id.clone(),
        relative_path: seed.relative_path.clone(),
        category: seed.category.clone(),
        language: seed.language.clone(),
        extraction_route: seed.extraction_route.clone(),
        source_content_hash: seed.source_content_hash.clone(),
        evidence_id: seed.evidence_id.clone(),
        target_ledger_field_group: target_field_group(seed.seed_kind.as_str()),
        output_contract: OUTPUT_CONTRACT,
        review_decision_required: true,
        promotion_decision_required: true,
        execution: EpistemeOntologyStructuralIdfReasoningFillPlanExecutionFlags {
            source_text_read: false,
            llm_executed: false,
            workflow_executed: false,
        },
        safety: EpistemeOntologyStructuralIdfReasoningFillPlanSafetyFlags {
            source_mutation_allowed: false,
            rdf_mutation_allowed: false,
            ontology_truth: false,
        },
        status: STATUS_PENDING,
    }
}

fn target_field_group(seed_kind: &str) -> &'static str {
    match seed_kind {
        OBJECT_SEED_KIND => OBJECT_FIELD_GROUP,
        RELATION_SEED_KIND => RELATION_FIELD_GROUP,
        _ => unreachable!("seed kind is validated before fill-plan item generation"),
    }
}

fn build_report(
    request: &EpistemeOntologyStructuralIdfReasoningFillPlanRequest,
    paths: &ReasoningFillPlanOutputPaths,
    items: &[EpistemeOntologyStructuralIdfReasoningFillPlanItem],
    skipped_by_limit_count: usize,
) -> EpistemeOntologyStructuralIdfReasoningFillPlanReport {
    let object_fill_item_count = items
        .iter()
        .filter(|item| item.seed_kind == OBJECT_SEED_KIND)
        .count();
    let relation_fill_item_count = items
        .iter()
        .filter(|item| item.seed_kind == RELATION_SEED_KIND)
        .count();
    EpistemeOntologyStructuralIdfReasoningFillPlanReport {
        schema_version: REASONING_FILL_PLAN_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        reasoning_ledger_seed_json: request.reasoning_ledger_seed_json.clone(),
        run_dir: paths.run_dir.clone(),
        reasoning_fill_plan_tsv: paths.fill_plan_tsv.clone(),
        reasoning_fill_plan_json: paths.fill_plan_json.clone(),
        reasoning_fill_plan_org: paths.fill_plan_org.clone(),
        reasoning_fill_plan_report_json: paths.report_json.clone(),
        seed_row_count: items.len(),
        object_fill_item_count,
        relation_fill_item_count,
        fill_item_count: items.len(),
        skipped_by_limit_count,
        execution: EpistemeOntologyStructuralIdfReasoningFillPlanExecutionFlags {
            source_text_read: false,
            llm_executed: false,
            workflow_executed: false,
        },
        safety: EpistemeOntologyStructuralIdfReasoningFillPlanSafetyFlags {
            source_mutation_allowed: false,
            rdf_mutation_allowed: false,
            ontology_truth: false,
        },
    }
}

fn stable_fill_item_id(run_id: &str, seed_id: &str) -> String {
    let digest = Sha256::digest(format!("{run_id}:{seed_id}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("idf.reasoning_fill_plan.{suffix}")
}

fn validate_run_id(run_id: &str) -> Result<()> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("invalid run id `{run_id}`; use ASCII letters, digits, '.', '_', or '-'");
    }
    Ok(())
}
