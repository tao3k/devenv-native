//! API surface for compiling structural-facts reasoning ledger seeds.

use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::ontology::reasoning_target::{
    OBJECT_INSTANCE_SEED_KIND, OBJECT_SEED_KIND, RELATION_SEED_KIND, SERVICE_CATALOG_SEED_KIND,
    seed_kinds_for_target_intent,
};

use super::{
    input::{ReasoningPacketInputRow, read_reasoning_packet_rows},
    types::{
        EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags,
        EpistemeOntologyStructuralFactsReasoningLedgerSeedReport,
        EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
        EpistemeOntologyStructuralFactsReasoningLedgerSeedRow,
        EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags,
        REASONING_LEDGER_SEED_REPORT_SCHEMA_VERSION, ReasoningLedgerSeedOutputPaths,
    },
    write::{write_json, write_seed_org, write_seed_tsv},
};

const REVIEW_PENDING: &str = "pending_reasoning";
const PROMOTION_BLOCKED: &str = "blocked_until_review";
const STATUS_PENDING: &str = "pending_reasoning";

/// Compile a structural facts reasoning packet into a fillable Org ledger seed.
///
/// # Errors
///
/// Returns an error when the reasoning-packet artifact is missing, malformed,
/// attempts to mark ontology truth, contains duplicate packet ids, has no
/// selectable rows, or output artifacts cannot be written.
pub fn write_episteme_ontology_structural_facts_reasoning_ledger_seed(
    request: &EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeOntologyStructuralFactsReasoningLedgerSeedReport> {
    validate_run_id(&request.run_id)?;
    if request.limit == 0 {
        bail!("reasoning ledger seed limit must be greater than zero");
    }

    let packet_rows = read_reasoning_packet_rows(request.reasoning_packet_json.as_path())?;
    let (seed_rows, skipped_by_limit_count) =
        build_seed_rows(&packet_rows, request.run_id.as_str(), request.limit)?;
    let paths = ReasoningLedgerSeedOutputPaths::new(run_root.as_ref(), request.run_id.as_str());
    fs::create_dir_all(paths.run_dir.as_path())
        .with_context(|| format!("failed to create `{}`", paths.run_dir.display()))?;
    write_seed_tsv(paths.seed_tsv.as_path(), &seed_rows)?;
    write_json(paths.seed_json.as_path(), &seed_rows)?;
    let report = build_report(request, &paths, &seed_rows, skipped_by_limit_count);
    write_seed_org(paths.seed_org.as_path(), &report, &seed_rows)?;
    write_json(paths.report_json.as_path(), &report)?;
    Ok(report)
}

fn build_seed_rows(
    packet_rows: &[ReasoningPacketInputRow],
    run_id: &str,
    limit: usize,
) -> Result<(
    Vec<EpistemeOntologyStructuralFactsReasoningLedgerSeedRow>,
    usize,
)> {
    let mut seen_packet_ids = BTreeSet::new();
    let mut seen_seed_ids = BTreeSet::new();
    let mut seed_rows = Vec::new();
    let mut skipped_by_limit_count = 0;

    for packet in packet_rows {
        if !seen_packet_ids.insert(packet.packet_id.as_str()) {
            bail!("duplicate reasoning packet id: {}", packet.packet_id);
        }
        if seen_packet_ids.len() > limit {
            skipped_by_limit_count += 1;
            continue;
        }
        for seed_kind in seed_kinds_for_target_intent(packet.evidence_target_intent.as_str()) {
            let seed = seed_row(packet, run_id, seed_kind);
            if !seen_seed_ids.insert(seed.seed_id.clone()) {
                bail!("duplicate reasoning ledger seed id: {}", seed.seed_id);
            }
            seed_rows.push(seed);
        }
    }

    if seed_rows.is_empty() {
        bail!("reasoning ledger seed selection produced no rows");
    }
    Ok((seed_rows, skipped_by_limit_count))
}

fn seed_row(
    packet: &ReasoningPacketInputRow,
    run_id: &str,
    seed_kind: &str,
) -> EpistemeOntologyStructuralFactsReasoningLedgerSeedRow {
    EpistemeOntologyStructuralFactsReasoningLedgerSeedRow {
        seed_id: stable_seed_id(run_id, packet.packet_id.as_str(), seed_kind),
        seed_kind: seed_kind.to_owned(),
        packet_id: packet.packet_id.clone(),
        reasoning_task_kind: packet.reasoning_task_kind.clone(),
        evidence_target_intent: packet.evidence_target_intent.clone(),
        evidence_anchor_kind: packet.evidence_anchor_kind.clone(),
        evidence_structure_hint: packet.evidence_structure_hint.clone(),
        document_id: packet.document_id.clone(),
        document_anchor_id: packet.document_anchor_id.clone(),
        file_id: packet.file_id.clone(),
        domain_id: packet.domain_id.clone(),
        source_contract_id: packet.source_contract_id.clone(),
        relative_path: packet.relative_path.clone(),
        category: packet.category.clone(),
        language: packet.language.clone(),
        extraction_route: packet.extraction_route.clone(),
        source_content_hash: packet.source_content_hash.clone(),
        evidence_id: packet.packet_id.clone(),
        proposed_object_id: String::new(),
        proposed_object_type: String::new(),
        proposed_label: String::new(),
        proposed_relation_id: String::new(),
        proposed_source_object_id: String::new(),
        proposed_predicate: String::new(),
        proposed_target_object_id: String::new(),
        review_decision: REVIEW_PENDING,
        promotion_decision: PROMOTION_BLOCKED,
        reviewer_id: String::new(),
        execution: EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags {
            source_text_read: false,
            llm_executed: false,
        },
        safety: EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags {
            source_mutation_allowed: false,
            ontology_truth: false,
        },
        status: STATUS_PENDING,
    }
}

fn build_report(
    request: &EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest,
    paths: &ReasoningLedgerSeedOutputPaths,
    rows: &[EpistemeOntologyStructuralFactsReasoningLedgerSeedRow],
    skipped_by_limit_count: usize,
) -> EpistemeOntologyStructuralFactsReasoningLedgerSeedReport {
    let object_seed_row_count = rows
        .iter()
        .filter(|row| row.seed_kind == OBJECT_SEED_KIND)
        .count();
    let relation_seed_row_count = rows
        .iter()
        .filter(|row| row.seed_kind == RELATION_SEED_KIND)
        .count();
    let service_catalog_seed_row_count = rows
        .iter()
        .filter(|row| row.seed_kind == SERVICE_CATALOG_SEED_KIND)
        .count();
    let object_instance_seed_row_count = rows
        .iter()
        .filter(|row| row.seed_kind == OBJECT_INSTANCE_SEED_KIND)
        .count();
    EpistemeOntologyStructuralFactsReasoningLedgerSeedReport {
        schema_version: REASONING_LEDGER_SEED_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        reasoning_packet_json: request.reasoning_packet_json.clone(),
        run_dir: paths.run_dir.clone(),
        reasoning_ledger_seed_tsv: paths.seed_tsv.clone(),
        reasoning_ledger_seed_json: paths.seed_json.clone(),
        reasoning_ledger_seed_org: paths.seed_org.clone(),
        reasoning_ledger_seed_report_json: paths.report_json.clone(),
        packet_row_count: rows
            .iter()
            .map(|row| row.packet_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        object_seed_row_count,
        relation_seed_row_count,
        service_catalog_seed_row_count,
        object_instance_seed_row_count,
        seed_row_count: rows.len(),
        skipped_by_limit_count,
        execution: EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags {
            source_text_read: false,
            llm_executed: false,
        },
        safety: EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags {
            source_mutation_allowed: false,
            ontology_truth: false,
        },
    }
}

fn stable_seed_id(run_id: &str, packet_id: &str, seed_kind: &str) -> String {
    let digest = Sha256::digest(format!("{run_id}:{packet_id}:{seed_kind}").as_bytes());
    let suffix = format!("{digest:x}").chars().take(16).collect::<String>();
    format!("structural_facts.reasoning_ledger_seed.{suffix}")
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
