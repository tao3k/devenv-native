//! Builder helpers for pair-shaped graph-structural candidates.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::core::{
    GraphStructuralCandidateSubgraph, GraphStructuralQueryContext, GraphStructuralRerankSignals,
};
use super::overlap::{
    GraphStructuralFilterConstraint, build_graph_structural_keyword_tag_query_context,
    build_graph_structural_keyword_tag_rerank_signals,
};
use super::pair::{GraphStructuralKeywordTagQueryInputs, GraphStructuralPairCandidateInputs};
use super::rows::{
    build_graph_structural_filter_request_row, build_graph_structural_rerank_request_row,
};
use super::support::{normalize_pair_endpoint_ids, stable_pair_candidate_id};
use crate::{GraphStructuralFilterRequestRow, GraphStructuralRerankRequestRow};

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
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
) -> Result<GraphStructuralCandidateSubgraph, RepoIntelligenceError> {
    let (left_id, right_id) = normalize_pair_endpoint_ids(left_id.into(), right_id.into())?;
    let edge_count = edge_kinds.len();
    GraphStructuralCandidateSubgraph::new(
        stable_pair_candidate_id(&left_id, &right_id),
        vec![left_id.clone(), right_id.clone()],
        vec![left_id; edge_count],
        vec![right_id; edge_count],
        edge_kinds,
    )
}

/// Build one staged structural-rerank request row from a two-node graph pair.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when either endpoint id is blank after
/// normalization, both endpoints resolve to the same id, or any edge kind is
/// blank.
pub fn build_graph_structural_pair_rerank_request_row(
    query: &GraphStructuralQueryContext,
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    signals: &GraphStructuralRerankSignals,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    let candidate = build_graph_structural_pair_candidate_subgraph(left_id, right_id, edge_kinds)?;
    Ok(build_graph_structural_rerank_request_row(
        query, &candidate, signals,
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
    query: &GraphStructuralQueryContext,
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    constraint: &GraphStructuralFilterConstraint,
) -> Result<GraphStructuralFilterRequestRow, RepoIntelligenceError> {
    let candidate = build_graph_structural_pair_candidate_subgraph(left_id, right_id, edge_kinds)?;
    Ok(build_graph_structural_filter_request_row(
        query, &candidate, constraint,
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
    query_inputs: GraphStructuralKeywordTagQueryInputs,
    pair_inputs: GraphStructuralPairCandidateInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
    tag_match: bool,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    let query = build_graph_structural_keyword_tag_query_context(
        query_inputs.query_id,
        query_inputs.retrieval_layer,
        query_inputs.query_max_layers,
        query_inputs.keyword_anchors,
        query_inputs.tag_anchors,
        query_inputs.edge_constraint_kinds,
    )?;
    let signals = build_graph_structural_keyword_tag_rerank_signals(
        semantic_score,
        dependency_score,
        keyword_match,
        tag_match,
    )?;
    build_graph_structural_pair_rerank_request_row(
        &query,
        pair_inputs.left_id,
        pair_inputs.right_id,
        pair_inputs.edge_kinds,
        &signals,
    )
}
