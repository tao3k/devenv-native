//! Shared `SearchStrategyFlow` candidate contracts.

use serde::Serialize;

pub(crate) const MAX_CANDIDATES: usize = 12;
pub(crate) const MARKDOWN_HEADING_CANDIDATE_SOURCE: &str = "rust-markdown-headings";
pub(crate) const CODE_INTELLIGENCE_CANDIDATE_SOURCE: &str = "rust-code-intelligence-inventory";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchStrategyFlowCandidateInput {
    pub(crate) relative_path: String,
    pub(crate) heading_anchor: String,
    pub(crate) title: String,
    pub(crate) line_start: usize,
    pub(crate) line_end: usize,
    pub(crate) context_cost: usize,
    pub(crate) evidence_coverage: f64,
    pub(crate) graph_score: f64,
    pub(crate) authority_score: f64,
    pub(crate) structural_score: f64,
    pub(crate) uncertainty: f64,
    pub(crate) blocked: bool,
    pub(crate) edge_kinds: Vec<String>,
}

/// TSV candidate batch passed from Rust discovery into `SearchStrategyFlow`.
/// Serialized candidate TSV batch passed to the `WendaoGraph` search-strategy replay host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStrategyFlowCandidateInputBatch {
    pub(crate) source: &'static str,
    pub(crate) row_count: usize,
    pub(crate) tsv: String,
    pub(crate) discovery_receipt_json: String,
}

pub(crate) struct SearchStrategyFlowRepoSearchHit<'a> {
    pub(crate) relative_path: &'a str,
    pub(crate) title: Option<&'a str>,
    pub(crate) best_section: Option<&'a str>,
    pub(crate) line_start: Option<usize>,
    pub(crate) line_end: Option<usize>,
    pub(crate) score: Option<f64>,
}
