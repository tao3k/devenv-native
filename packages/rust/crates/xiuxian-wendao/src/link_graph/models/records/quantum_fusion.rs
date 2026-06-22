//! `link_graph::models::records::quantum_fusion` owns Wendao models records quantum fusion behavior.

use super::retrieval_plan::LinkGraphRetrievalBudget;
use crate::link_graph::models::LinkGraphRelatedPprOptions;
use crate::link_graph::models::{LinkGraphSemanticDocumentScope, LinkGraphSemanticSearchPolicy};
use serde::{Deserialize, Serialize};

const MISSING_SEMANTIC_POLICY_FALLBACK_REASON: &str =
    "semantic policy omitted; use unrestricted semantic ignition thresholds";

/// One semantic anchor hit returned by a vector backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumAnchorHit {
    /// Canonical anchor identifier (`doc_id#anchor`).
    pub anchor_id: String,
    /// Vector similarity score in `[0.0, 1.0]`.
    pub vector_score: f64,
}

/// Options that control quantum-fusion scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantumFusionOptions {
    /// Blend coefficient applied to vector similarity.
    pub alpha: f64,
    /// Maximum graph traversal distance.
    pub max_distance: usize,
    /// Max number of related candidates to consider.
    pub related_limit: usize,
    /// Optional PPR tuning for related traversal.
    #[serde(default)]
    pub ppr: Option<LinkGraphRelatedPprOptions>,
}

impl Default for QuantumFusionOptions {
    fn default() -> Self {
        Self {
            alpha: 0.5,
            max_distance: 2,
            related_limit: 8,
            ppr: None,
        }
    }
}

impl QuantumFusionOptions {
    /// Normalize the fusion options into a safe runtime range.
    #[must_use]
    pub fn normalized(&self) -> Self {
        Self {
            alpha: self.alpha.clamp(0.0, 1.0),
            max_distance: self.max_distance.max(1),
            related_limit: self.related_limit.max(1),
            ppr: self.ppr.clone(),
        }
    }
}

/// One fused quantum context derived from anchor hits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct QuantumContext {
    /// Canonical anchor id used as the semantic seed.
    pub anchor_id: String,
    /// Canonical document id containing the anchor.
    pub doc_id: String,
    /// Path or stem for the anchored document.
    pub path: String,
    /// Ordered semantic breadcrumb trail.
    pub semantic_path: Vec<String>,
    /// Traceability tag derived from the semantic path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_label: Option<String>,
    /// Related cluster doc ids identified by topology expansion.
    pub related_clusters: Vec<String>,
    /// Final fused saliency score.
    pub saliency_score: f64,
    /// Source vector similarity score.
    pub vector_score: f64,
    /// Topology-derived score (for observability).
    pub topology_score: f64,
}

impl QuantumContext {
    /// Render a stable trace label for diagnostics.
    #[must_use]
    pub fn trace_label(&self) -> String {
        self.trace_label
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map_or_else(
                || format!("quantum:{}:{:.3}", self.anchor_id, self.saliency_score),
                str::to_string,
            )
    }

    /// Build a traceability label from a semantic path.
    #[cfg(feature = "vector-store")]
    #[must_use]
    pub(crate) fn trace_label_from_semantic_path(semantic_path: &[String]) -> Option<String> {
        if semantic_path.is_empty() {
            None
        } else {
            Some(format!("[Path: {}]", semantic_path.join(" > ")))
        }
    }
}

/// Request payload for semantic ignition backends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumSemanticSearchRequest<'a> {
    /// Optional user query text (for telemetry).
    pub query_text: Option<&'a str>,
    /// Query embedding vector.
    pub query_vector: &'a [f32],
    /// Maximum number of anchor hits to request.
    pub candidate_limit: usize,
    /// Optional minimum similarity filter.
    pub min_vector_score: Option<f64>,
    /// Optional maximum similarity filter.
    pub max_vector_score: Option<f64>,
}

impl QuantumSemanticSearchRequest<'_> {
    /// Build a semantic-ignition request from retrieval-plan budget and policy.
    #[must_use]
    pub fn from_retrieval_budget<'a>(
        query_text: Option<&'a str>,
        query_vector: &'a [f32],
        budget: Option<&LinkGraphRetrievalBudget>,
        semantic_policy: Option<LinkGraphSemanticSearchPolicy>,
    ) -> QuantumSemanticSearchRequest<'a> {
        let semantic_policy = semantic_policy_with_explicit_fallback(semantic_policy).normalized();
        QuantumSemanticSearchRequest {
            query_text,
            query_vector,
            candidate_limit: budget.map_or(1, |value| value.candidate_limit.max(1)),
            min_vector_score: semantic_policy.min_vector_score,
            max_vector_score: None,
        }
        .normalized()
    }

    /// Normalize request parameters into safe bounds.
    #[must_use]
    pub fn normalized(&self) -> Self {
        let query_text = self.query_text.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let min_vector_score = self.min_vector_score.filter(|value| value.is_finite());
        let max_vector_score = self.max_vector_score.filter(|value| value.is_finite());
        Self {
            query_text,
            query_vector: self.query_vector,
            candidate_limit: self.candidate_limit.max(1),
            min_vector_score,
            max_vector_score,
        }
    }

    /// Return true when no search should be performed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.candidate_limit == 0 || (self.query_vector.is_empty() && self.query_text.is_none())
    }

    /// Check whether a vector score passes the configured filter.
    #[must_use]
    pub fn allows_vector_score(&self, score: f64) -> bool {
        let min_ok = self.min_vector_score.is_none_or(|min| score >= min);
        let max_ok = self.max_vector_score.is_none_or(|max| score <= max);
        min_ok && max_ok
    }
}

fn semantic_policy_with_explicit_fallback(
    semantic_policy: Option<LinkGraphSemanticSearchPolicy>,
) -> LinkGraphSemanticSearchPolicy {
    match semantic_policy {
        Some(policy) => policy,
        None => semantic_policy_missing_explicit_fallback(MISSING_SEMANTIC_POLICY_FALLBACK_REASON),
    }
}

fn semantic_policy_missing_explicit_fallback(
    fallback_reason: &'static str,
) -> LinkGraphSemanticSearchPolicy {
    debug_assert!(!fallback_reason.trim().is_empty());
    LinkGraphSemanticSearchPolicy {
        document_scope: LinkGraphSemanticDocumentScope::All,
        min_vector_score: None,
    }
}

/// Telemetry summary emitted by quantum-fusion searches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct QuantumFusionTelemetry {
    /// Number of anchor hits processed.
    pub anchor_count: usize,
    /// Number of contexts emitted.
    pub context_count: usize,
}
