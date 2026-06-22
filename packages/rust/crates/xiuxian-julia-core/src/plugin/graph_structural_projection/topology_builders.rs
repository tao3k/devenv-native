//! Builder helpers for explicit-edge graph topology request rows.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::core::{
    GraphStructuralCandidateSubgraph, GraphStructuralCandidateSubgraphInput,
    GraphStructuralQueryContext, GraphStructuralRerankSignals,
};
use super::overlap::GraphStructuralFilterConstraint;
use super::rows::{
    build_graph_structural_filter_request_row, build_graph_structural_rerank_request_row,
};
use super::topology::{
    GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralRawConnectedPairCandidateInput,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs,
};
use crate::{
    GraphStructuralFilterRequestRow, GraphStructuralRerankRequestRow,
    build_graph_structural_filter_request_batch, build_graph_structural_rerank_request_batch,
};

/// Build one generic explicit-edge topology candidate subgraph from staged
/// metadata inputs.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the candidate id is blank, the node
/// list is invalid, or the explicit edge endpoints or edge kinds violate the
/// staged graph-structural candidate rules.
pub fn build_graph_structural_generic_topology_candidate_subgraph(
    metadata_inputs: GraphStructuralGenericTopologyCandidateMetadataInputs,
) -> Result<GraphStructuralCandidateSubgraph, RepoIntelligenceError> {
    let GraphStructuralGenericTopologyCandidateMetadataInputs {
        candidate_id,
        node_ids,
        edge_sources,
        edge_destinations,
        edge_kinds,
    } = metadata_inputs;
    GraphStructuralCandidateSubgraph::from_input(GraphStructuralCandidateSubgraphInput {
        candidate_id,
        node_ids,
        edge_sources,
        edge_destinations,
        edge_kinds,
    })
}

/// Build one staged structural-rerank request row from one generic explicit-edge
/// topology candidate bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the generic topology metadata or
/// staged plane scores violate the underlying graph-structural normalization
/// rules.
pub fn build_graph_structural_generic_topology_rerank_request_row(
    query: &GraphStructuralQueryContext,
    candidate: GraphStructuralGenericTopologyCandidateInputs,
) -> Result<GraphStructuralRerankRequestRow, RepoIntelligenceError> {
    let GraphStructuralGenericTopologyCandidateInputs {
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_score,
        tag_score,
    } = candidate;
    let candidate = build_graph_structural_generic_topology_candidate_subgraph(metadata_inputs)?;
    let signals = GraphStructuralRerankSignals::new(
        semantic_score,
        dependency_score,
        keyword_score,
        tag_score,
    )?;
    Ok(build_graph_structural_rerank_request_row(
        query, &candidate, &signals,
    ))
}

/// Build one staged structural-rerank request batch from one shared query plus
/// generic explicit-edge topology candidates.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any candidate violates the staged
/// graph-structural normalization rules or when the final Arrow batch fails
/// contract validation.
pub fn build_graph_structural_generic_topology_rerank_request_batch(
    query: &GraphStructuralQueryContext,
    candidates: &[GraphStructuralGenericTopologyCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            build_graph_structural_generic_topology_rerank_request_row(query, candidate)
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_graph_structural_rerank_request_batch(&rows)
}

/// Build one staged structural-rerank request batch from one shared query plus
/// raw connected-pair collections.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any raw connected-pair collection
/// fails staged Julia-owned generic-topology normalization or when the final
/// Arrow batch fails contract validation.
pub fn build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections(
    query: &GraphStructuralQueryContext,
    candidates: &[GraphStructuralRawConnectedPairCollectionCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let candidates = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
                GraphStructuralRawConnectedPairCandidateInput {
                    candidate_id: candidate.candidate_id,
                    pair_candidates: candidate.pair_candidates,
                    fallback_edge_kind: candidate.fallback_edge_kind.into(),
                    dependency_score: candidate.dependency_score,
                    keyword_score: candidate.keyword_score,
                    tag_score: candidate.tag_score,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_graph_structural_generic_topology_rerank_request_batch(query, candidates.as_slice())
}

/// Build one staged constraint-filter request row from one generic
/// explicit-edge topology candidate bundle.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the generic topology metadata or the
/// staged constraint input violates the underlying graph-structural
/// normalization rules.
pub fn build_graph_structural_generic_topology_filter_request_row(
    query: &GraphStructuralQueryContext,
    candidate: GraphStructuralGenericTopologyCandidateInputs,
    constraint: &GraphStructuralFilterConstraint,
) -> Result<GraphStructuralFilterRequestRow, RepoIntelligenceError> {
    let GraphStructuralGenericTopologyCandidateInputs {
        metadata_inputs, ..
    } = candidate;
    let candidate = build_graph_structural_generic_topology_candidate_subgraph(metadata_inputs)?;
    Ok(build_graph_structural_filter_request_row(
        query, &candidate, constraint,
    ))
}

/// Build one staged constraint-filter request batch from one shared query plus
/// generic explicit-edge topology candidates.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any candidate violates the staged
/// graph-structural normalization rules or when the final Arrow batch fails
/// contract validation.
pub fn build_graph_structural_generic_topology_filter_request_batch(
    query: &GraphStructuralQueryContext,
    constraint: &GraphStructuralFilterConstraint,
    candidates: &[GraphStructuralGenericTopologyCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let rows = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            build_graph_structural_generic_topology_filter_request_row(query, candidate, constraint)
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_graph_structural_filter_request_batch(&rows)
}

/// Build one staged constraint-filter request batch from one shared query plus
/// raw connected-pair collections.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any raw connected-pair collection
/// fails staged Julia-owned generic-topology normalization or when the final
/// Arrow batch fails contract validation.
pub fn build_graph_structural_generic_topology_filter_request_batch_from_raw_connected_pair_collections(
    query: &GraphStructuralQueryContext,
    constraint: &GraphStructuralFilterConstraint,
    candidates: &[GraphStructuralRawConnectedPairCollectionCandidateInputs],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let candidates = candidates
        .iter()
        .cloned()
        .map(|candidate| {
            build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
                GraphStructuralRawConnectedPairCandidateInput {
                    candidate_id: candidate.candidate_id,
                    pair_candidates: candidate.pair_candidates,
                    fallback_edge_kind: candidate.fallback_edge_kind.into(),
                    dependency_score: candidate.dependency_score,
                    keyword_score: candidate.keyword_score,
                    tag_score: candidate.tag_score,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_graph_structural_generic_topology_filter_request_batch(
        query,
        constraint,
        candidates.as_slice(),
    )
}
