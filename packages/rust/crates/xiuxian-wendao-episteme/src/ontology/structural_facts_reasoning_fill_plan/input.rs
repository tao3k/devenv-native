use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::ontology::reasoning_target::{
    OBJECT_INSTANCE_SEED_KIND, OBJECT_SEED_KIND, RELATION_SEED_KIND, SERVICE_CATALOG_SEED_KIND,
    default_evidence_anchor_kind, default_evidence_structure_hint, default_evidence_target_intent,
};

const REVIEW_PENDING: &str = "pending_reasoning";
const PROMOTION_BLOCKED: &str = "blocked_until_review";
const STATUS_PENDING: &str = "pending_reasoning";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningLedgerSeedInputRow {
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
    pub proposed_object_id: String,
    pub proposed_object_type: String,
    pub proposed_label: String,
    pub proposed_relation_id: String,
    pub proposed_source_object_id: String,
    pub proposed_predicate: String,
    pub proposed_target_object_id: String,
    pub review_decision: String,
    pub promotion_decision: String,
    pub reviewer_id: String,
    #[serde(flatten)]
    pub execution: ReasoningLedgerSeedInputExecutionFlags,
    #[serde(flatten)]
    pub safety: ReasoningLedgerSeedInputSafetyFlags,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningLedgerSeedInputExecutionFlags {
    pub source_text_read: bool,
    pub llm_executed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReasoningLedgerSeedInputSafetyFlags {
    pub source_mutation_allowed: bool,
    pub ontology_truth: bool,
}

pub(super) fn read_reasoning_ledger_seed_rows(
    path: &Path,
) -> Result<Vec<ReasoningLedgerSeedInputRow>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let rows: Vec<ReasoningLedgerSeedInputRow> = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse `{}`", path.display()))?;
    if rows.is_empty() {
        bail!(
            "reasoning ledger seed JSON `{}` has no rows",
            path.display()
        );
    }
    for row in &rows {
        validate_seed_row(path, row)?;
    }
    Ok(rows)
}

fn validate_seed_row(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    validate_required_fields(path, row)?;
    validate_seed_kind(path, row)?;
    validate_pending_state(path, row)?;
    validate_blank_semantic_fields(path, row)?;
    validate_safety_flags(path, row)?;
    Ok(())
}

fn validate_required_fields(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    for (field, value) in [
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
    ] {
        if value.trim().is_empty() {
            bail!(
                "reasoning ledger seed JSON `{}` row `{}` has blank {field}",
                path.display(),
                row.seed_id
            );
        }
    }
    Ok(())
}

fn validate_seed_kind(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    if !matches!(
        row.seed_kind.as_str(),
        OBJECT_SEED_KIND
            | RELATION_SEED_KIND
            | SERVICE_CATALOG_SEED_KIND
            | OBJECT_INSTANCE_SEED_KIND
    ) {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` has unsupported seed kind `{}`",
            path.display(),
            row.seed_id,
            row.seed_kind
        );
    }
    Ok(())
}

fn validate_pending_state(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    if row.review_decision != REVIEW_PENDING
        || row.promotion_decision != PROMOTION_BLOCKED
        || row.status != STATUS_PENDING
    {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` is not a pending reasoning seed",
            path.display(),
            row.seed_id
        );
    }
    Ok(())
}

fn validate_blank_semantic_fields(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    for (field, value) in [
        ("proposed_object_id", row.proposed_object_id.as_str()),
        ("proposed_object_type", row.proposed_object_type.as_str()),
        ("proposed_label", row.proposed_label.as_str()),
        ("proposed_relation_id", row.proposed_relation_id.as_str()),
        (
            "proposed_source_object_id",
            row.proposed_source_object_id.as_str(),
        ),
        ("proposed_predicate", row.proposed_predicate.as_str()),
        (
            "proposed_target_object_id",
            row.proposed_target_object_id.as_str(),
        ),
        ("reviewer_id", row.reviewer_id.as_str()),
    ] {
        if !value.trim().is_empty() {
            bail!(
                "reasoning ledger seed JSON `{}` row `{}` already contains {field}",
                path.display(),
                row.seed_id
            );
        }
    }
    Ok(())
}

fn validate_safety_flags(path: &Path, row: &ReasoningLedgerSeedInputRow) -> Result<()> {
    if row.execution.source_text_read {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` already read source text",
            path.display(),
            row.seed_id
        );
    }
    if row.execution.llm_executed {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` already executed an LLM",
            path.display(),
            row.seed_id
        );
    }
    if row.safety.source_mutation_allowed {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` attempted to allow source mutation",
            path.display(),
            row.seed_id
        );
    }
    if row.safety.ontology_truth {
        bail!(
            "reasoning ledger seed JSON `{}` row `{}` attempted to mark ontology truth",
            path.display(),
            row.seed_id
        );
    }
    Ok(())
}
