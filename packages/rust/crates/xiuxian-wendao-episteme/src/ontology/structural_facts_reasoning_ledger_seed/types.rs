//! Data contracts for structural-facts reasoning ledger seeds.

use std::path::{Path, PathBuf};

use serde::Serialize;

pub(super) const REASONING_LEDGER_SEED_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts_reasoning_ledger_seed_report.v1";

/// Request for compiling a reasoning packet into a fillable Org ledger seed.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest {
    pub(super) reasoning_packet_json: PathBuf,
    pub(super) run_id: String,
    pub(super) limit: usize,
}

impl EpistemeOntologyStructuralFactsReasoningLedgerSeedRequest {
    /// Create a reasoning ledger seed request.
    #[must_use]
    pub fn new(reasoning_packet_json: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            reasoning_packet_json: reasoning_packet_json.into(),
            run_id: run_id.into(),
            limit: 512,
        }
    }

    /// Set the maximum number of input packet rows to seed.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeOntologyStructuralFactsReasoningLedgerSeedRow {
    pub seed_id: String,
    pub seed_kind: String,
    pub packet_id: String,
    pub reasoning_task_kind: String,
    pub evidence_target_intent: String,
    pub evidence_anchor_kind: String,
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
    pub review_decision: &'static str,
    pub promotion_decision: &'static str,
    pub reviewer_id: String,
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags,
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags,
    pub status: &'static str,
}

/// Report emitted after reasoning-ledger seed generation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningLedgerSeedReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source reasoning packet JSON artifact.
    pub reasoning_packet_json: PathBuf,
    /// Concrete ledger-seed run directory.
    pub run_dir: PathBuf,
    /// Generated ledger-seed TSV path.
    pub reasoning_ledger_seed_tsv: PathBuf,
    /// Generated ledger-seed JSON path.
    pub reasoning_ledger_seed_json: PathBuf,
    /// Generated ledger-seed Org path.
    pub reasoning_ledger_seed_org: PathBuf,
    /// Generated report JSON path.
    pub reasoning_ledger_seed_report_json: PathBuf,
    /// Number of packet rows consumed.
    pub packet_row_count: usize,
    /// Number of object proposal slot rows emitted.
    pub object_seed_row_count: usize,
    /// Number of relation proposal slot rows emitted.
    pub relation_seed_row_count: usize,
    /// Number of service-catalog review rows emitted.
    pub service_catalog_seed_row_count: usize,
    /// Number of object-instance review rows emitted.
    pub object_instance_seed_row_count: usize,
    /// Total seed rows emitted.
    pub seed_row_count: usize,
    /// Number of matching packet rows skipped by the limit.
    pub skipped_by_limit_count: usize,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags,
}

/// Execution flags preserved in ledger-seed reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningLedgerSeedExecutionFlags {
    /// Whether this seed read private source text.
    pub source_text_read: bool,
    /// Whether this seed called a live LLM.
    pub llm_executed: bool,
}

/// Safety flags preserved in ledger-seed reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningLedgerSeedSafetyFlags {
    /// Whether this seed authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct ReasoningLedgerSeedOutputPaths {
    pub run_dir: PathBuf,
    pub seed_tsv: PathBuf,
    pub seed_json: PathBuf,
    pub seed_org: PathBuf,
    pub report_json: PathBuf,
}

impl ReasoningLedgerSeedOutputPaths {
    pub fn new(run_root: &Path, run_key: &str) -> Self {
        let run_dir = run_root.join(run_key);
        Self {
            seed_tsv: run_dir.join("reasoning_ledger_seed.tsv"),
            seed_json: run_dir.join("reasoning_ledger_seed.json"),
            seed_org: run_dir.join("reasoning_ledger_seed.org"),
            report_json: run_dir.join("reasoning_ledger_seed_report.json"),
            run_dir,
        }
    }
}
