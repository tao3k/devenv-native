//! Pair-shaped DTOs for graph-structural projection.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::support::{normalize_non_negative_score, normalize_pair_endpoint_ids};

/// Raw query inputs for the keyword-or-tag graph-structural helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordTagQueryInputs {
    pub(super) query_id: String,
    pub(super) retrieval_layer: i32,
    pub(super) query_max_layers: i32,
    pub(super) keyword_anchors: Vec<String>,
    pub(super) tag_anchors: Vec<String>,
    pub(super) edge_constraint_kinds: Vec<String>,
}

/// Named input bundle for one keyword-or-tag graph-structural query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordTagQueryInput {
    /// Query id attached to the structural request.
    pub query_id: String,
    /// Retrieval layer used by the graph route.
    pub retrieval_layer: i32,
    /// Maximum layer depth accepted for the query.
    pub query_max_layers: i32,
    /// Keyword anchors for the request.
    pub keyword_anchors: Vec<String>,
    /// Tag anchors for the request.
    pub tag_anchors: Vec<String>,
    /// Edge-kind constraints for the request.
    pub edge_constraint_kinds: Vec<String>,
}

impl GraphStructuralKeywordTagQueryInputs {
    /// Store one keyword-or-tag query input bundle for later normalization.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordTagQueryInput) -> Self {
        let GraphStructuralKeywordTagQueryInput {
            query_id,
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        } = input;
        Self {
            query_id,
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        }
    }

    /// Store one keyword-or-tag query input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        query_id: impl Into<String>,
        retrieval_layer: i32,
        query_max_layers: i32,
        keyword_anchors: Vec<String>,
        tag_anchors: Vec<String>,
        edge_constraint_kinds: Vec<String>,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordTagQueryInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        })
    }
}

/// Raw pair-candidate inputs for the keyword-or-tag graph-structural helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralPairCandidateInputs {
    pub(super) left_id: String,
    pub(super) right_id: String,
    pub(super) edge_kinds: Vec<String>,
}

impl GraphStructuralPairCandidateInputs {
    /// Store one pair-candidate input bundle for later normalization.
    #[must_use]
    pub fn new(
        left_id: impl Into<String>,
        right_id: impl Into<String>,
        edge_kinds: Vec<String>,
    ) -> Self {
        Self {
            left_id: left_id.into(),
            right_id: right_id.into(),
            edge_kinds,
        }
    }
}

/// Build one pair-candidate input bundle from raw pair ids and edge kinds.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually constructing the pair-input DTO layer.
#[must_use]
pub fn build_graph_structural_pair_candidate_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
) -> GraphStructuralPairCandidateInputs {
    GraphStructuralPairCandidateInputs::new(left_id, right_id, edge_kinds)
}

/// One scored pair-candidate input bundle used to aggregate a generic topology
/// candidate above the pair seam.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralScoredPairCandidateInputs {
    pub(super) pair_inputs: GraphStructuralPairCandidateInputs,
    pub(super) semantic_score: f64,
}

impl GraphStructuralScoredPairCandidateInputs {
    /// Store one scored pair-candidate bundle for later normalization.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when `semantic_score` is negative or
    /// non-finite.
    pub fn new(
        pair_inputs: GraphStructuralPairCandidateInputs,
        semantic_score: f64,
    ) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            pair_inputs,
            semantic_score: normalize_non_negative_score(semantic_score, "pair semantic score")?,
        })
    }
}

/// Build one scored pair-candidate input bundle from raw pair ids, edge kinds,
/// and one pair-level semantic score.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when `semantic_score` is negative or
/// non-finite.
pub fn build_graph_structural_scored_pair_candidate_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    semantic_score: f64,
) -> Result<GraphStructuralScoredPairCandidateInputs, RepoIntelligenceError> {
    GraphStructuralScoredPairCandidateInputs::new(
        build_graph_structural_pair_candidate_inputs(left_id, right_id, edge_kinds),
        semantic_score,
    )
}

/// One raw connected-pair bundle used to aggregate a generic topology
/// candidate without exposing the scored pair DTO seam to callers.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRawConnectedPairInputs {
    pub(super) left_id: String,
    pub(super) right_id: String,
    pub(super) semantic_score: f64,
}

impl GraphStructuralRawConnectedPairInputs {
    /// Store one raw connected-pair bundle for later normalization.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
    /// normalization, both endpoints resolve to the same id, or
    /// `semantic_score` is negative or non-finite.
    pub fn new(
        left_id: impl Into<String>,
        right_id: impl Into<String>,
        semantic_score: f64,
    ) -> Result<Self, RepoIntelligenceError> {
        let (left_id, right_id) = normalize_pair_endpoint_ids(left_id.into(), right_id.into())?;
        Ok(Self {
            left_id,
            right_id,
            semantic_score: normalize_non_negative_score(
                semantic_score,
                "connected pair semantic score",
            )?,
        })
    }
}

/// Build one raw connected-pair bundle from endpoint ids and one pair-level
/// semantic score.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization, both endpoints resolve to the same id, or
/// `semantic_score` is negative or non-finite.
pub fn build_graph_structural_raw_connected_pair_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    semantic_score: f64,
) -> Result<GraphStructuralRawConnectedPairInputs, RepoIntelligenceError> {
    GraphStructuralRawConnectedPairInputs::new(left_id, right_id, semantic_score)
}
