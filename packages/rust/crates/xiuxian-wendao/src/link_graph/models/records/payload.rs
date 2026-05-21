//! `link_graph::models::records::payload` owns Wendao models records payload behavior.

use super::hits::{LinkGraphDisplayHit, LinkGraphHit};
use super::quantum_fusion::QuantumContext;
use super::retrieval_plan::{
    LinkGraphConfidenceLevel, LinkGraphRetrievalMode, LinkGraphRetrievalPlanRecord,
};
use crate::link_graph::agentic::LinkGraphSuggestedLink;
use crate::link_graph::models::query::LinkGraphSearchOptions;
use serde::{Deserialize, Serialize};
use xiuxian_wendao_core::transport::PluginTransportKind;

/// Canonical planned-search payload used by CLI/bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphPlannedSearchPayload {
    /// Parsed/normalized query after directive extraction.
    pub query: String,
    /// Effective search options used during execution.
    pub options: LinkGraphSearchOptions,
    /// Display-ready hits.
    pub hits: Vec<LinkGraphDisplayHit>,
    /// Number of matched rows before external truncation.
    pub hit_count: usize,
    /// Number of hits that matched at section/heading level.
    pub section_hit_count: usize,
    /// Retrieval mode requested by runtime policy.
    #[serde(default)]
    pub requested_mode: LinkGraphRetrievalMode,
    /// Retrieval mode selected after graph confidence gating.
    #[serde(default)]
    pub selected_mode: LinkGraphRetrievalMode,
    /// Human-readable policy reason.
    #[serde(default)]
    pub reason: String,
    /// Number of graph hits evaluated by policy.
    #[serde(default)]
    pub graph_hit_count: usize,
    /// Number of source hints derived from graph hits.
    #[serde(default)]
    pub source_hint_count: usize,
    /// Confidence score derived from graph hit quality.
    #[serde(default)]
    pub graph_confidence_score: f64,
    /// Confidence level bucket derived from graph confidence score.
    #[serde(default)]
    pub graph_confidence_level: LinkGraphConfidenceLevel,
    /// Full schema-aligned retrieval plan record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_plan: Option<LinkGraphRetrievalPlanRecord>,
    /// Semantic ignition telemetry for quantum enrichment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_ignition: Option<LinkGraphSemanticIgnitionTelemetry>,
    /// Julia rerank telemetry for optional Arrow IPC post-processing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub julia_rerank: Option<LinkGraphJuliaRerankTelemetry>,
    /// Optional precomputed query embedding retained for runtime semantic ignition.
    #[serde(default, skip)]
    pub query_vector: Option<Vec<f32>>,
    /// Quantum contexts derived from semantic ignition plus Arrow-native fusion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quantum_contexts: Vec<QuantumContext>,
    /// Raw hit rows for backward compatibility.
    pub results: Vec<LinkGraphHit>,
    /// Optional provisional suggested-link rows for hybrid/agentic surfaces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provisional_suggestions: Vec<LinkGraphSuggestedLink>,
    /// Optional retrieval error when provisional suggestions were requested but unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provisional_error: Option<String>,
    /// Query-time promoted-edge overlay telemetry for observability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promoted_overlay: Option<LinkGraphPromotedOverlayTelemetry>,
    /// CCS (Context Completeness Score) audit result for persona-style anchor coverage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ccs_audit: Option<LinkGraphCcsAudit>,
}

/// Context Completeness Score audit result for search payloads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinkGraphCcsAudit {
    /// Context Completeness Score (0.0-1.0).
    pub ccs_score: f64,
    /// Whether CCS passed the threshold (>= 0.70).
    pub passed: bool,
    /// Whether compensation was applied (re-search with expanded parameters).
    #[serde(default)]
    pub compensated: bool,
    /// Anchors missing from search evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_anchors: Vec<String>,
}

/// Promoted-edge overlay telemetry emitted on search payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphPromotedOverlayTelemetry {
    /// Whether a promoted-edge overlay was applied for this query.
    pub applied: bool,
    /// Overlay source identifier.
    pub source: String,
    /// Number of candidate suggestion rows scanned from source.
    pub scanned_rows: usize,
    /// Number of scanned rows in `promoted` state.
    pub promoted_rows: usize,
    /// Number of distinct directed edges materialized into the overlay graph.
    pub added_edges: usize,
}

/// Semantic ignition telemetry emitted on planned search payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphSemanticIgnitionTelemetry {
    /// Runtime-selected semantic ignition backend alias.
    pub backend: String,
    /// Stable backend name reported by the ignition implementation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_name: Option<String>,
    /// Number of quantum contexts emitted for this payload.
    #[serde(default)]
    pub context_count: usize,
    /// Backend or orchestration error, when enrichment failed closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Julia rerank telemetry emitted on planned search payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphJuliaRerankTelemetry {
    /// Whether the remote Julia rerank path was actually executed.
    pub applied: bool,
    /// Number of score rows returned by the Julia service.
    #[serde(default)]
    pub response_row_count: usize,
    /// Runtime-selected transport surfaced by the negotiation seam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_transport: Option<PluginTransportKind>,
    /// Higher-preference transport skipped before selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_from: Option<PluginTransportKind>,
    /// Reason the runtime fell back from a higher-preference transport.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    /// Optional trace identifiers surfaced from additive Julia response columns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace_ids: Vec<String>,
    /// Transport, contract, or application error emitted by the rerank step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
