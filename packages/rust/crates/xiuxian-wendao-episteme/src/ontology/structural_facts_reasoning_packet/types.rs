//! Data contracts for structural-facts reasoning packets.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Serialize;

pub(super) const STRUCTURAL_FACTS_REASONING_PACKET_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts_reasoning_packet_report.v1";

/// Request for compiling structural facts rows into a reasoning packet.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsReasoningPacketRequest {
    pub(super) structural_facts_json: PathBuf,
    pub(super) run_id: String,
    pub(super) limit: usize,
    pub(super) category: Option<String>,
    pub(super) route: Option<String>,
}

impl EpistemeOntologyStructuralFactsReasoningPacketRequest {
    /// Create a reasoning-packet request from a structural facts JSON artifact.
    #[must_use]
    pub fn new(structural_facts_json: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            structural_facts_json: structural_facts_json.into(),
            run_id: run_id.into(),
            limit: 256,
            category: None,
            route: None,
        }
    }

    /// Set the maximum number of packet rows to emit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Restrict packet rows to one source category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Restrict packet rows to one extraction route.
    #[must_use]
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeOntologyStructuralFactsReasoningPacketRow {
    pub packet_id: String,
    pub packet_kind: &'static str,
    pub reasoning_task_kind: String,
    pub evidence_target_intent: &'static str,
    pub evidence_anchor_kind: String,
    pub evidence_structure_hint: &'static str,
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
    pub evidence_action: &'static str,
    pub ontology_truth: bool,
    pub status: &'static str,
}

/// Report emitted after reasoning-packet generation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningPacketReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source structural facts JSON artifact.
    pub structural_facts_json: PathBuf,
    /// Concrete packet run directory.
    pub run_dir: PathBuf,
    /// Generated packet TSV path.
    pub reasoning_packet_tsv: PathBuf,
    /// Generated packet rows JSON path.
    pub reasoning_packet_json: PathBuf,
    /// Generated packet Org path.
    pub reasoning_packet_org: PathBuf,
    /// Generated report JSON path.
    pub reasoning_packet_report_json: PathBuf,
    /// Number of packet rows emitted.
    pub packet_row_count: usize,
    /// Number of document rows selected.
    pub selected_document_count: usize,
    /// Number of document rows skipped by category or route filters.
    pub skipped_by_filter_count: usize,
    /// Number of matching document rows skipped by the limit.
    pub skipped_by_limit_count: usize,
    /// Source category counts in emitted packet rows.
    pub category_counts: BTreeMap<String, usize>,
    /// Extraction route counts in emitted packet rows.
    pub route_counts: BTreeMap<String, usize>,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningPacketExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningPacketSafetyFlags,
}

/// Execution flags preserved in reasoning-packet reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningPacketExecutionFlags {
    /// Whether this packet read private source text.
    pub source_text_read: bool,
    /// Whether this packet called a live LLM.
    pub llm_executed: bool,
}

/// Safety flags preserved in reasoning-packet reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningPacketSafetyFlags {
    /// Whether this packet authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct ReasoningPacketOutputPaths {
    pub run_dir: PathBuf,
    pub packet_tsv: PathBuf,
    pub packet_json: PathBuf,
    pub packet_org: PathBuf,
    pub report_json: PathBuf,
}

impl ReasoningPacketOutputPaths {
    pub fn new(run_root: &Path, run_key: &str) -> Self {
        let run_dir = run_root.join(run_key);
        Self {
            packet_tsv: run_dir.join("reasoning_packet.tsv"),
            packet_json: run_dir.join("reasoning_packet.json"),
            packet_org: run_dir.join("reasoning_packet.org"),
            report_json: run_dir.join("reasoning_packet_report.json"),
            run_dir,
        }
    }
}
