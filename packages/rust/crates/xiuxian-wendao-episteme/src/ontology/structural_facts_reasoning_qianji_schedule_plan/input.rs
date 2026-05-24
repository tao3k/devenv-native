use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::ontology::reasoning_target::{
    default_evidence_anchor_kind, default_evidence_structure_hint, default_evidence_target_intent,
};

const WORKFLOW_KEY: &str = "episteme_ontology_reasoning_fill";
const ACTIVITY_KIND: &str = "read_targeted_evidence_then_fill_org_proposal";
const STATUS_PENDING: &str = "pending_workflow_execution";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningFillPlanInputRow {
    pub fill_item_id: String,
    pub workflow_key: String,
    pub activity_kind: String,
    pub qianji_activity_contract: String,
    pub seed_id: String,
    pub seed_kind: String,
    pub packet_id: String,
    pub reasoning_task_kind: String,
    #[serde(default = "default_evidence_target_intent")]
    pub evidence_target_intent: String,
    #[serde(default = "default_evidence_anchor_kind")]
    pub evidence_anchor_kind: String,
    #[serde(default = "default_evidence_structure_hint")]
    pub evidence_structure_hint: String,
    pub document_id: String,
    pub document_anchor_id: String,
    pub file_id: String,
    pub domain_id: String,
    pub source_contract_id: String,
    pub relative_path: String,
    pub category: String,
    pub language: String,
    pub extraction_route: String,
    pub source_content_hash: String,
    pub evidence_id: String,
    pub target_ledger_field_group: String,
    pub output_contract: String,
    pub review_decision_required: bool,
    pub promotion_decision_required: bool,
    #[serde(flatten)]
    pub execution: ReasoningFillPlanInputExecutionFlags,
    #[serde(flatten)]
    pub safety: ReasoningFillPlanInputSafetyFlags,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningFillPlanInputExecutionFlags {
    pub source_text_read: bool,
    pub llm_executed: bool,
    pub workflow_executed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningFillPlanInputSafetyFlags {
    pub source_mutation_allowed: bool,
    pub rdf_mutation_allowed: bool,
    pub ontology_truth: bool,
}

pub(super) fn read_reasoning_fill_plan_rows(path: &Path) -> Result<Vec<ReasoningFillPlanInputRow>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let rows: Vec<ReasoningFillPlanInputRow> = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if rows.is_empty() {
        bail!("reasoning fill-plan JSON `{}` has no rows", path.display());
    }
    for row in &rows {
        validate_fill_plan_row(path, row)?;
    }
    Ok(rows)
}

fn validate_fill_plan_row(path: &Path, row: &ReasoningFillPlanInputRow) -> Result<()> {
    validate_required_fields(path, row)?;
    validate_workflow_shape(path, row)?;
    validate_safety_flags(path, row)?;
    Ok(())
}

fn validate_required_fields(path: &Path, row: &ReasoningFillPlanInputRow) -> Result<()> {
    for (field, value) in [
        ("fill_item_id", row.fill_item_id.as_str()),
        ("workflow_key", row.workflow_key.as_str()),
        ("activity_kind", row.activity_kind.as_str()),
        (
            "qianji_activity_contract",
            row.qianji_activity_contract.as_str(),
        ),
        ("seed_id", row.seed_id.as_str()),
        ("seed_kind", row.seed_kind.as_str()),
        ("packet_id", row.packet_id.as_str()),
        ("reasoning_task_kind", row.reasoning_task_kind.as_str()),
        (
            "evidence_target_intent",
            row.evidence_target_intent.as_str(),
        ),
        ("evidence_anchor_kind", row.evidence_anchor_kind.as_str()),
        (
            "evidence_structure_hint",
            row.evidence_structure_hint.as_str(),
        ),
        ("document_id", row.document_id.as_str()),
        ("document_anchor_id", row.document_anchor_id.as_str()),
        ("file_id", row.file_id.as_str()),
        ("domain_id", row.domain_id.as_str()),
        ("source_contract_id", row.source_contract_id.as_str()),
        ("relative_path", row.relative_path.as_str()),
        ("source_content_hash", row.source_content_hash.as_str()),
        ("evidence_id", row.evidence_id.as_str()),
        (
            "target_ledger_field_group",
            row.target_ledger_field_group.as_str(),
        ),
        ("output_contract", row.output_contract.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!(
                "reasoning fill-plan JSON `{}` row `{}` has blank {field}",
                path.display(),
                row.fill_item_id
            );
        }
    }
    Ok(())
}

fn validate_workflow_shape(path: &Path, row: &ReasoningFillPlanInputRow) -> Result<()> {
    if row.workflow_key != WORKFLOW_KEY {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` has unsupported workflow key `{}`",
            path.display(),
            row.fill_item_id,
            row.workflow_key
        );
    }
    if row.activity_kind != ACTIVITY_KIND {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` has unsupported activity kind `{}`",
            path.display(),
            row.fill_item_id,
            row.activity_kind
        );
    }
    if row.status != STATUS_PENDING {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` is not pending workflow execution",
            path.display(),
            row.fill_item_id
        );
    }
    if !row.review_decision_required || !row.promotion_decision_required {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` is missing review or promotion decision requirements",
            path.display(),
            row.fill_item_id
        );
    }
    Ok(())
}

fn validate_safety_flags(path: &Path, row: &ReasoningFillPlanInputRow) -> Result<()> {
    if row.execution.source_text_read {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` already read source text",
            path.display(),
            row.fill_item_id
        );
    }
    if row.execution.llm_executed {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` already executed an LLM",
            path.display(),
            row.fill_item_id
        );
    }
    if row.execution.workflow_executed {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` already executed workflow",
            path.display(),
            row.fill_item_id
        );
    }
    if row.safety.source_mutation_allowed {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` attempted to allow source mutation",
            path.display(),
            row.fill_item_id
        );
    }
    if row.safety.rdf_mutation_allowed {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` attempted to allow RDF mutation",
            path.display(),
            row.fill_item_id
        );
    }
    if row.safety.ontology_truth {
        bail!(
            "reasoning fill-plan JSON `{}` row `{}` attempted to mark ontology truth",
            path.display(),
            row.fill_item_id
        );
    }
    Ok(())
}
