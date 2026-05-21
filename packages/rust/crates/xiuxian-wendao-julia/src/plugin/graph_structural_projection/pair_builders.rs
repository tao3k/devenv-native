//! Builder helpers for pair-shaped graph-structural candidates.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::core::{
    GraphStructuralCandidateSubgraph, GraphStructuralCandidateSubgraphInput,
    GraphStructuralQueryContext, GraphStructuralRerankSignals,
};
use super::overlap::{
    GraphStructuralFilterConstraint, GraphStructuralKeywordTagMatchFlags,
    GraphStructuralKeywordTagQueryContextInput, GraphStructuralKeywordTagRerankSignalInput,
    build_graph_structural_keyword_tag_query_context,
    build_graph_structural_keyword_tag_rerank_signals,
};
use super::pair::{GraphStructuralKeywordTagQueryInputs, GraphStructuralPairCandidateInputs};
use super::rows::{
    build_graph_structural_filter_request_row, build_graph_structural_rerank_request_row,
};
use super::support::{normalize_pair_endpoint_ids, stable_pair_candidate_id};
use crate::{GraphStructuralFilterRequestRow, GraphStructuralRerankRequestRow};

/// Named inputs for one pair-shaped graph-structural rerank request row.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralPairRerankRequestInput {
    /// Query context shared by the request row.
    pub query: GraphStructuralQueryContext,
    /// Pair candidate data.
    pub pair: GraphStructuralPairCandidateInputs,
    /// Rerank signals for this pair.
    pub signals: GraphStructuralRerankSignals,
}

/// Named inputs for one pair-shaped graph-structural filter request row.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralPairFilterRequestInput {
    /// Query context shared by the request row.
    pub query: GraphStructuralQueryContext,
    /// Pair candidate data.
    pub pair: GraphStructuralPairCandidateInputs,
    /// Filter constraint for this pair.
    pub constraint: GraphStructuralFilterConstraint,
}

/// Named inputs for one keyword-or-tag pair rerank request row.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordTagPairRerankRequestInput {
    /// Raw query anchors and retrieval bounds.
    pub query: GraphStructuralKeywordTagQueryInputs,
    /// Pair candidate data.
    pub pair: GraphStructuralPairCandidateInputs,
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Whether the pair matched a keyword anchor.
    pub keyword_match: bool,
    /// Whether the pair matched a tag anchor.
    pub tag_match: bool,
}

/// Build the stable candidate id used for one two-node graph-structural pair.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization or when both endpoints resolve to the same id.
pub fn graph_structural_pair_candidate_id(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
) -> Result<String, RepoIntelligenceError> {
    let (left_id, right_id) = normalize_pair_endpoint_ids(left_id.into(), right_id.into())?;
    Ok(stable_pair_candidate_id(&left_id, &right_id))
}

/// Build one normalized candidate subgraph from a two-node graph pair.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization, both endpoints resolve to the same id, or any edge kind is
/// blank.
pub fn build_graph_structural_pair_candidate_subgraph(
    pair: GraphStructuralPairCandidateInputs,
) -> Result<GraphStructuralCandidateSubgraph, RepoIntelligenceError> {
    let (left_id, right_id) = normalize_pair_endpoint_ids(pair.left_id, pair.right_id)?;
    let edge_kinds = pair.edge_kinds;
    let edge_count = edge_kinds.len();
    GraphStructuralCandidateSubgraph::from_input(GraphStructuralCandidateSubgraphInput {
        candidate_id: stable_pair_candidate_id(&left_id, &right_id),
        node_ids: vec![left_id.clone(), right_id.clone()],
        edge_sources: vec![left_id; edge_count],
        edge_destinations: vec![right_id; edge_count],
        edge_kinds,
    })
}

/// Build one staged structural-rerank request row from a two-node graph pair.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization, both endpoints resolve to the same id, or any edge kind is
/// blank.
pub fn build_graph_structural_pair_rerank_request_row(
    input: GraphStructuralPairRerankRequestInput,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    let candidate = build_graph_structural_pair_candidate_subgraph(input.pair)?;
    Ok(build_graph_structural_rerank_request_row(
        &input.query,
        &candidate,
        &input.signals,
    ))
}

/// Build one staged constraint-filter request row from a two-node graph pair.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization, both endpoints resolve to the same id, or any edge kind is
/// blank.
pub fn build_graph_structural_pair_filter_request_row(
    input: GraphStructuralPairFilterRequestInput,
) -> Result<GraphStructuralFilterRequestRow, RepoIntelligenceError> {
    let candidate = build_graph_structural_pair_candidate_subgraph(input.pair)?;
    Ok(build_graph_structural_filter_request_row(
        &input.query,
        &candidate,
        &input.constraint,
    ))
}

/// Build one staged structural-rerank request row from keyword-or-tag query inputs
/// plus one two-node graph pair.
///
/// This convenience helper keeps the host on a thin consumption seam by
/// composing the Julia-owned keyword-or-tag query builder, binary rerank-signal
/// builder, and pair-rerank request-row projection.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any query, anchor, edge-constraint,
/// endpoint, edge-kind, or score input fails the underlying Julia-owned
/// normalization rules.
pub fn build_graph_structural_keyword_tag_pair_rerank_request_row(
    input: GraphStructuralKeywordTagPairRerankRequestInput,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    let GraphStructuralKeywordTagPairRerankRequestInput {
        query,
        pair,
        semantic_score,
        dependency_score,
        keyword_match,
        tag_match,
    } = input;
    let query = build_graph_structural_keyword_tag_query_context(
        GraphStructuralKeywordTagQueryContextInput {
            query_id: query.query_id,
            retrieval_layer: query.retrieval_layer,
            query_max_layers: query.query_max_layers,
            keyword_anchors: query.keyword_anchors,
            tag_anchors: query.tag_anchors,
            edge_constraint_kinds: query.edge_constraint_kinds,
        },
    )?;
    let signals = build_graph_structural_keyword_tag_rerank_signals(
        GraphStructuralKeywordTagRerankSignalInput {
            semantic_score,
            dependency_score,
            matches: GraphStructuralKeywordTagMatchFlags {
                keyword_match,
                tag_match,
            },
        },
    )?;
    build_graph_structural_pair_rerank_request_row(GraphStructuralPairRerankRequestInput {
        query,
        pair,
        signals,
    })
}
