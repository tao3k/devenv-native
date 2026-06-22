//! Row projection helpers for staged graph-structural request DTOs.

use super::core::{
    GraphStructuralCandidateSubgraph, GraphStructuralQueryContext, GraphStructuralRerankSignals,
};
use super::overlap::GraphStructuralFilterConstraint;
use crate::{GraphStructuralFilterRequestRow, GraphStructuralRerankRequestRow};

/// Build one staged structural-rerank request row from normalized semantic DTOs.
#[must_use]
pub fn build_graph_structural_rerank_request_row(
    query: &GraphStructuralQueryContext,
    candidate: &GraphStructuralCandidateSubgraph,
    signals: &GraphStructuralRerankSignals,
) -> GraphStructuralRerankRequestRow {
    GraphStructuralRerankRequestRow {
        query_id: query.query_id().to_string(),
        candidate_id: candidate.candidate_id().to_string(),
        retrieval_layer: query.retrieval_layer(),
        query_max_layers: query.query_max_layers(),
        semantic_score: signals.semantic_score(),
        dependency_score: signals.dependency_score(),
        keyword_score: signals.keyword_score(),
        tag_score: signals.tag_score(),
        anchor_planes: query
            .anchors()
            .iter()
            .map(|anchor| anchor.plane().to_string())
            .collect(),
        anchor_values: query
            .anchors()
            .iter()
            .map(|anchor| anchor.value().to_string())
            .collect(),
        edge_constraint_kinds: query.edge_constraint_kinds().to_vec(),
        candidate_node_ids: candidate.node_ids().to_vec(),
        candidate_edge_sources: candidate.edge_sources().to_vec(),
        candidate_edge_destinations: candidate.edge_destinations().to_vec(),
        candidate_edge_kinds: candidate.edge_kinds().to_vec(),
    }
}

/// Build one staged constraint-filter request row from normalized semantic DTOs.
#[must_use]
pub fn build_graph_structural_filter_request_row(
    query: &GraphStructuralQueryContext,
    candidate: &GraphStructuralCandidateSubgraph,
    constraint: &GraphStructuralFilterConstraint,
) -> GraphStructuralFilterRequestRow {
    GraphStructuralFilterRequestRow {
        query_id: query.query_id().to_string(),
        candidate_id: candidate.candidate_id().to_string(),
        retrieval_layer: query.retrieval_layer(),
        query_max_layers: query.query_max_layers(),
        constraint_kind: constraint.constraint_kind().into(),
        required_boundary_size: constraint.required_boundary_size(),
        anchor_planes: query
            .anchors()
            .iter()
            .map(|anchor| anchor.plane().to_string())
            .collect(),
        anchor_values: query
            .anchors()
            .iter()
            .map(|anchor| anchor.value().to_string())
            .collect(),
        edge_constraint_kinds: query.edge_constraint_kinds().to_vec(),
        candidate_node_ids: candidate.node_ids().to_vec(),
        candidate_edge_sources: candidate.edge_sources().to_vec(),
        candidate_edge_destinations: candidate.edge_destinations().to_vec(),
        candidate_edge_kinds: candidate.edge_kinds().to_vec(),
    }
}
