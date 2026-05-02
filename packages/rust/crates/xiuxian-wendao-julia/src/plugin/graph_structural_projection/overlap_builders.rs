//! Builder helpers for keyword-overlap graph-structural request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::overlap::{
    GraphStructuralKeywordOverlapCandidateInputs, GraphStructuralKeywordOverlapPairInputs,
    GraphStructuralKeywordOverlapPairRequestInputs, GraphStructuralKeywordOverlapPairRerankInputs,
    GraphStructuralKeywordOverlapQueryInputs, GraphStructuralKeywordOverlapRawCandidateInputs,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
};
use super::pair::{GraphStructuralKeywordTagQueryInputs, GraphStructuralPairCandidateInputs};
use super::pair_builders::build_graph_structural_keyword_tag_pair_rerank_request_row;
use super::support::normalize_string_list;
use crate::{GraphStructuralRerankRequestRow, build_graph_structural_rerank_request_batch};

/// Return normalized shared tags between the left and right metadata slices.
///
/// The output preserves normalized left-tag order and removes duplicates.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any tag value is blank after
/// normalization.
pub fn graph_structural_shared_tag_anchors(
    left_tags: Vec<String>,
    right_tags: Vec<String>,
) -> Result<Vec<String>, RepoIntelligenceError> {
    let left_tags = normalize_string_list(left_tags, "left tags", true)?;
    let right_tags = normalize_string_list(right_tags, "right tags", true)?;
    let right_set: std::collections::HashSet<String> = right_tags.into_iter().collect();
    let mut seen = std::collections::HashSet::new();
    let mut shared = Vec::new();
    for tag in left_tags {
        if right_set.contains(&tag) && seen.insert(tag.clone()) {
            shared.push(tag);
        }
    }
    Ok(shared)
}

/// Build one staged structural-rerank request row from keyword anchors,
/// raw left or right tag metadata, and one two-node graph pair.
///
/// Shared tag anchors are derived inside the Julia-owned helper layer, and the
/// tag-score signal is inferred from whether any shared tags remain.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any query, tag, edge-constraint,
/// endpoint, edge-kind, or score input fails the underlying Julia-owned
/// normalization rules.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_row(
    mut query_inputs: GraphStructuralKeywordTagQueryInputs,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
    pair_inputs: GraphStructuralPairCandidateInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    query_inputs.tag_anchors = graph_structural_shared_tag_anchors(left_tags, right_tags)?;
    let tag_match = !query_inputs.tag_anchors.is_empty();
    build_graph_structural_keyword_tag_pair_rerank_request_row(
        query_inputs,
        pair_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
        tag_match,
    )
}

/// Build one staged structural-rerank request row from a plugin-owned
/// metadata-and-pair input bundle.
///
/// This convenience helper keeps the host on a thinner Julia-owned seam by
/// deferring both shared-tag extraction and pair-row assembly to the plugin
/// layer.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any query, metadata, edge-constraint,
/// endpoint, edge-kind, or score input fails the underlying Julia-owned
/// normalization rules.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata(
    inputs: GraphStructuralKeywordOverlapPairInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    build_graph_structural_keyword_overlap_pair_rerank_request_row(
        inputs.query_inputs,
        inputs.left_metadata.tags,
        inputs.right_metadata.tags,
        inputs.pair_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    )
}

/// Build one staged structural-rerank request batch from plugin-owned
/// metadata-aware rerank input bundles.
///
/// This convenience helper keeps the host on the thinnest currently available
/// Julia-owned seam by composing metadata-aware row projection and Arrow batch
/// materialization inside the plugin crate.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any metadata-aware rerank input fails
/// the underlying Julia-owned normalization rules or when the final Arrow batch
/// fails staged contract validation.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata(
    inputs: &[GraphStructuralKeywordOverlapPairRerankInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = inputs
        .iter()
        .cloned()
        .map(|input| {
            build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata(
                input.metadata_inputs,
                input.semantic_score,
                input.dependency_score,
                input.keyword_match,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_graph_structural_rerank_request_batch(&rows)
}

/// Build one staged structural-rerank request batch from higher-level
/// keyword-overlap candidate inputs.
///
/// This helper keeps host consumers on a thinner Julia-owned seam by composing
/// query-input, metadata-input, pair-input, and metadata-aware batch assembly
/// inside the plugin crate.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any higher-level candidate input
/// fails the underlying Julia-owned normalization rules or when the final
/// Arrow batch fails staged contract validation.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs(
    inputs: &[GraphStructuralKeywordOverlapPairRequestInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let metadata_inputs = inputs
        .iter()
        .cloned()
        .map(|input| {
            GraphStructuralKeywordOverlapPairRerankInputs::new(
                input.metadata_inputs,
                input.semantic_score,
                input.dependency_score,
                input.keyword_match,
            )
        })
        .collect::<Vec<_>>();
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata(&metadata_inputs)
}

/// Build one higher-level keyword-overlap request input from a shared query
/// bundle and one candidate bundle.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_request_input(
    query: &GraphStructuralKeywordOverlapQueryInputs,
    candidate: GraphStructuralKeywordOverlapCandidateInputs,
) -> GraphStructuralKeywordOverlapPairRequestInputs {
    let query_inputs = GraphStructuralKeywordTagQueryInputs::new(
        query.query_id.clone(),
        query.retrieval_layer,
        query.query_max_layers,
        query.keyword_anchors.clone(),
        Vec::new(),
        query.edge_constraint_kinds.clone(),
    );

    GraphStructuralKeywordOverlapPairRequestInputs::new(
        query_inputs,
        candidate.left_metadata,
        candidate.right_metadata,
        candidate.pair_inputs,
        candidate.semantic_score,
        candidate.dependency_score,
        candidate.keyword_match,
    )
}

/// Build one staged structural-rerank request batch from one shared
/// keyword-overlap query bundle plus per-candidate inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any derived request input fails the
/// underlying Julia-owned normalization rules or when the final Arrow batch
/// fails staged contract validation.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_batch(
    query: &GraphStructuralKeywordOverlapQueryInputs,
    candidates: &[GraphStructuralKeywordOverlapCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let request_inputs = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            build_graph_structural_keyword_overlap_pair_request_input(query, candidate)
        })
        .collect::<Vec<_>>();
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs(&request_inputs)
}

/// Build one staged structural-rerank request batch from one shared
/// keyword-overlap query bundle plus raw per-candidate inputs.
///
/// This helper keeps host consumers on a thinner Julia-owned seam by
/// composing raw candidate normalization, higher-level request-input
/// projection, and Arrow batch materialization inside the plugin crate.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any raw candidate input fails the
/// underlying Julia-owned normalization rules or when the final Arrow batch
/// fails staged contract validation.
pub fn build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates(
    query: &GraphStructuralKeywordOverlapQueryInputs,
    candidates: &[GraphStructuralKeywordOverlapRawCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let normalized_candidates = candidates
        .iter()
        .cloned()
        .map(build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw)
        .collect::<Vec<_>>();
    build_graph_structural_keyword_overlap_pair_rerank_request_batch(query, &normalized_candidates)
}
