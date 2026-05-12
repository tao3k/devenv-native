//! `link_graph::models::semantic_policy` owns Wendao link graph models semantic policy behavior.

use serde::{Deserialize, Serialize};
/// Scope of semantic documents allowed in retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LinkGraphSemanticDocumentScope {
    /// Allow both summary- and section-level semantic documents.
    #[default]
    All,
    /// Allow only summary-level semantic documents.
    SummaryOnly,
}

/// Semantic retrieval policy embedded in search options and retrieval plans.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSemanticSearchPolicy {
    /// Which semantic document scope is eligible for retrieval.
    #[serde(default)]
    pub document_scope: LinkGraphSemanticDocumentScope,
    /// Optional minimum vector score required for semantic ignition.
    #[serde(default)]
    pub min_vector_score: Option<f64>,
}

impl Default for LinkGraphSemanticSearchPolicy {
    fn default() -> Self {
        Self {
            document_scope: LinkGraphSemanticDocumentScope::All,
            min_vector_score: None,
        }
    }
}

impl LinkGraphSemanticSearchPolicy {
    /// Normalize optional thresholds into safe bounds.
    #[must_use]
    pub fn normalized(&self) -> Self {
        Self {
            document_scope: self.document_scope,
            min_vector_score: self.min_vector_score.map(|score| score.clamp(0.0, 1.0)),
        }
    }
}
