//! `link_graph::models::records::retrieval_plan` owns Wendao models records retrieval plan behavior.

use serde::{Deserialize, Serialize};

/// Canonical schema version for `LinkGraph` retrieval-plan records.
pub const LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION: &str =
    "xiuxian_wendao.link_graph.retrieval_plan.v1";
/// Policy reason emitted when the backend cannot be initialized.
pub const LINK_GRAPH_REASON_BACKEND_UNAVAILABLE: &str = "backend_unavailable";
/// Policy reason emitted when caller explicitly requests vector-only routing.
pub const LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED: &str = "vector_only_requested";
/// Policy reason emitted when graph-only mode has graph hits.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED: &str = "graph_only_requested";
/// Policy reason emitted when graph-only mode has no graph hits.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY: &str = "graph_only_requested_empty";
/// Policy reason emitted when graph-only routing timed out.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT: &str = "graph_only_search_timeout";
/// Policy reason emitted when graph-only mode overrides non-graph payload mode.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN: &str = "graph_only_payload_overridden";
/// Policy reason emitted when payload requested mode conflicts in graph-only mode.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT: &str =
    "graph_only_payload_mode_conflict";
/// Policy reason emitted when graph-only policy payload is missing.
pub const LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING: &str = "graph_only_policy_missing";
/// Policy reason emitted when graph confidence is sufficient.
pub const LINK_GRAPH_REASON_GRAPH_SUFFICIENT: &str = "graph_sufficient";
/// Policy reason emitted when graph confidence is insufficient.
pub const LINK_GRAPH_REASON_GRAPH_INSUFFICIENT: &str = "graph_insufficient";
/// Policy reason emitted when policy selects hybrid execution.
pub const LINK_GRAPH_REASON_HYBRID_SELECTED: &str = "hybrid_selected";
/// Policy reason emitted when graph search timed out.
pub const LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT: &str = "graph_search_timeout";
/// Policy reason emitted when payload mode conflicts with requested mode.
pub const LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT: &str = "graph_policy_mode_conflict";
/// Policy reason emitted when payload decision is missing.
pub const LINK_GRAPH_REASON_GRAPH_POLICY_MISSING: &str = "graph_policy_missing";
/// Canonical policy-reason vocabulary shared across Rust/Python layers.
pub const LINK_GRAPH_POLICY_REASON_VOCAB: &[&str] = &[
    LINK_GRAPH_REASON_BACKEND_UNAVAILABLE,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY,
    LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_SUFFICIENT,
    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT,
    LINK_GRAPH_REASON_HYBRID_SELECTED,
    LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_POLICY_MISSING,
];

/// Retrieval mode selected/requested by graph-first policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphRetrievalMode {
    /// Execute and trust graph retrieval output.
    GraphOnly,
    /// Start graph-first and allow bounded escalation.
    #[default]
    Hybrid,
    /// Skip graph retrieval and route directly to vector stage.
    VectorOnly,
}

impl LinkGraphRetrievalMode {
    /// Parse retrieval mode aliases from runtime configuration.
    #[must_use]
    pub fn from_alias(raw: &str) -> Option<Self> {
        match raw.trim().to_lowercase().as_str() {
            "graph" | "graph_only" => Some(Self::GraphOnly),
            "hybrid" => Some(Self::Hybrid),
            "vector" | "vector_only" => Some(Self::VectorOnly),
            _ => None,
        }
    }
}

/// Confidence bucket for graph retrieval quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphConfidenceLevel {
    /// No confidence signal (for example no hits).
    #[default]
    None,
    /// Low confidence.
    Low,
    /// Medium confidence.
    Medium,
    /// High confidence.
    High,
}

/// Bounded retrieval budget emitted in policy records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphRetrievalBudget {
    /// Planned candidate query size before post-truncation.
    pub candidate_limit: usize,
    /// Max graph source hints forwarded to later stages.
    pub max_sources: usize,
    /// Max rows requested per source hint.
    pub rows_per_source: usize,
}

impl Default for LinkGraphRetrievalBudget {
    fn default() -> Self {
        Self {
            candidate_limit: 1,
            max_sources: 1,
            rows_per_source: 1,
        }
    }
}

/// Schema-aligned retrieval-plan record for graph-first policy telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphRetrievalPlanRecord {
    /// Version tag for stable contract validation.
    pub schema: String,
    /// Retrieval mode requested by caller/runtime configuration.
    pub requested_mode: LinkGraphRetrievalMode,
    /// Retrieval mode selected by policy after confidence evaluation.
    pub selected_mode: LinkGraphRetrievalMode,
    /// Human-readable selection reason.
    pub reason: String,
    /// Backend identifier used by this policy decision.
    pub backend_name: String,
    /// Graph hit count used by confidence gate.
    pub graph_hit_count: usize,
    /// Distinct source hints extracted from graph hits.
    pub source_hint_count: usize,
    /// Graph confidence score in [0, 1].
    pub graph_confidence_score: f64,
    /// Graph confidence bucket.
    pub graph_confidence_level: LinkGraphConfidenceLevel,
    /// Bounded policy budget.
    pub budget: LinkGraphRetrievalBudget,
}

/// Construction payload for [`LinkGraphRetrievalPlanRecord`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphRetrievalPlanInput {
    /// Retrieval mode requested by caller/runtime policy.
    pub requested_mode: LinkGraphRetrievalMode,
    /// Retrieval mode selected after policy evaluation.
    pub selected_mode: LinkGraphRetrievalMode,
    /// Canonical policy reason string.
    pub reason: String,
    /// Backend identifier emitting this plan record.
    pub backend_name: String,
    /// Total graph hits considered by policy.
    pub graph_hit_count: usize,
    /// Distinct source hints extracted from graph hits.
    pub source_hint_count: usize,
    /// Graph confidence score in [0, 1].
    pub graph_confidence_score: f64,
    /// Confidence bucket for the score.
    pub graph_confidence_level: LinkGraphConfidenceLevel,
    /// Bounded retrieval budget derived by runtime policy.
    pub budget: LinkGraphRetrievalBudget,
}

impl LinkGraphRetrievalPlanRecord {
    /// Build a schema-aligned retrieval plan record with bounded score.
    #[must_use]
    pub fn new(input: LinkGraphRetrievalPlanInput) -> Self {
        debug_assert!(
            LINK_GRAPH_POLICY_REASON_VOCAB.contains(&input.reason.as_str()),
            "link_graph retrieval plan reason `{}` is not in policy vocab",
            input.reason
        );
        Self {
            schema: LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION.to_string(),
            requested_mode: input.requested_mode,
            selected_mode: input.selected_mode,
            reason: input.reason,
            backend_name: input.backend_name,
            graph_hit_count: input.graph_hit_count,
            source_hint_count: input.source_hint_count,
            graph_confidence_score: input.graph_confidence_score.clamp(0.0, 1.0),
            graph_confidence_level: input.graph_confidence_level,
            budget: input.budget,
        }
    }
}
