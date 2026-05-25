//! Folder-first tests for graph-structural projection helpers.

pub(super) use super::{
    GraphStructuralCandidateSubgraph, GraphStructuralFilterConstraint,
    GraphStructuralGenericTopologyCandidateInput, GraphStructuralGenericTopologyCandidateInputs,
    GraphStructuralGenericTopologyCandidateMetadataInput,
    GraphStructuralGenericTopologyCandidateMetadataInputs,
    GraphStructuralGenericTopologyPairCollectionInput,
    GraphStructuralKeywordOverlapCandidateInputs,
    GraphStructuralKeywordOverlapCandidateMetadataInput, GraphStructuralKeywordOverlapPairInputs,
    GraphStructuralKeywordOverlapPairRequestInputs, GraphStructuralKeywordOverlapPairRerankInputs,
    GraphStructuralKeywordOverlapPairRerankRowInput, GraphStructuralKeywordOverlapQueryInput,
    GraphStructuralKeywordOverlapQueryInputs, GraphStructuralKeywordOverlapRawCandidateInput,
    GraphStructuralKeywordOverlapRawCandidateInputs, GraphStructuralKeywordTagMatchFlags,
    GraphStructuralKeywordTagPairRerankRequestInput, GraphStructuralKeywordTagQueryContextInput,
    GraphStructuralKeywordTagQueryInputs, GraphStructuralKeywordTagRerankSignalInput,
    GraphStructuralNodeMetadataInputs, GraphStructuralPairCandidateInputs,
    GraphStructuralPairFilterRequestInput, GraphStructuralPairRerankRequestInput,
    GraphStructuralQueryAnchor, GraphStructuralQueryContext,
    GraphStructuralRawConnectedPairCandidateInput,
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    GraphStructuralRawConnectedPairCollectionRawTupleInput, GraphStructuralRawConnectedPairInputs,
    GraphStructuralRerankSignals, GraphStructuralScoredPairCandidateInputs,
    GraphStructuralScoredPairCollectionCandidateInput, build_graph_structural_filter_request_row,
    build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection,
    build_graph_structural_generic_topology_candidate_subgraph,
    build_graph_structural_generic_topology_rerank_request_batch,
    build_graph_structural_generic_topology_rerank_request_batch_from_raw_connected_pair_collections,
    build_graph_structural_generic_topology_rerank_request_row,
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    build_graph_structural_keyword_overlap_pair_request_input,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_inputs,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_metadata,
    build_graph_structural_keyword_overlap_pair_rerank_request_batch_from_raw_candidates,
    build_graph_structural_keyword_overlap_pair_rerank_request_row_from_metadata,
    build_graph_structural_pair_candidate_inputs, build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_rerank_request_row, build_graph_structural_scored_pair_candidate_inputs,
    graph_structural_pair_candidate_id, graph_structural_shared_tag_anchors,
};
pub(super) use crate::{
    GraphStructuralFilterRequestRow, GraphStructuralRerankRequestRow, JuliaContractKind,
    build_graph_structural_filter_request_batch, build_graph_structural_rerank_request_batch,
};

mod invariants;
mod keyword_overlap;
mod rows_and_pairs;
mod topology;

fn build_graph_structural_keyword_overlap_query_inputs(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> GraphStructuralKeywordOverlapQueryInputs {
    super::build_graph_structural_keyword_overlap_query_inputs(
        GraphStructuralKeywordOverlapQueryInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        },
    )
}

fn build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
) -> super::GraphStructuralKeywordOverlapCandidateMetadataInputs {
    super::build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
        GraphStructuralKeywordOverlapCandidateMetadataInput {
            left_id: left_id.into(),
            right_id: right_id.into(),
            edge_kinds,
            left_tags,
            right_tags,
        },
    )
}

fn build_graph_structural_keyword_overlap_raw_candidate_inputs(
    metadata_inputs: super::GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapRawCandidateInputs {
    super::build_graph_structural_keyword_overlap_raw_candidate_inputs(
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

fn build_graph_structural_keyword_overlap_candidate_inputs(
    metadata_inputs: super::GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    super::build_graph_structural_keyword_overlap_candidate_inputs(
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

fn build_graph_structural_keyword_overlap_pair_candidate_inputs(
    metadata_inputs: super::GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    super::build_graph_structural_keyword_overlap_pair_candidate_inputs(
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

fn build_graph_structural_keyword_tag_query_context(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    tag_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> Result<
    GraphStructuralQueryContext,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_keyword_tag_query_context(
        GraphStructuralKeywordTagQueryContextInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            tag_anchors,
            edge_constraint_kinds,
        },
    )
}

fn build_graph_structural_keyword_tag_rerank_signals(
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
    tag_match: bool,
) -> Result<
    GraphStructuralRerankSignals,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_keyword_tag_rerank_signals(
        GraphStructuralKeywordTagRerankSignalInput {
            semantic_score,
            dependency_score,
            matches: GraphStructuralKeywordTagMatchFlags {
                keyword_match,
                tag_match,
            },
        },
    )
}

fn build_graph_structural_generic_topology_candidate_metadata_inputs(
    candidate_id: impl Into<String>,
    node_ids: Vec<String>,
    edge_sources: Vec<String>,
    edge_destinations: Vec<String>,
    edge_kinds: Vec<String>,
) -> GraphStructuralGenericTopologyCandidateMetadataInputs {
    super::build_graph_structural_generic_topology_candidate_metadata_inputs(
        GraphStructuralGenericTopologyCandidateMetadataInput {
            candidate_id: candidate_id.into(),
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_inputs(
    metadata_inputs: GraphStructuralGenericTopologyCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> GraphStructuralGenericTopologyCandidateInputs {
    super::build_graph_structural_generic_topology_candidate_inputs(
        GraphStructuralGenericTopologyCandidateInput {
            metadata: metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_inputs_from_pair_collection(
    candidate_id: impl Into<String>,
    pair_candidates: Vec<GraphStructuralPairCandidateInputs>,
    fallback_edge_kind: impl Into<String>,
    semantic_score: f64,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<
    GraphStructuralGenericTopologyCandidateInputs,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_generic_topology_candidate_inputs_from_pair_collection(
        GraphStructuralGenericTopologyPairCollectionInput {
            candidate_id: candidate_id.into(),
            pair_candidates,
            fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection(
    candidate_id: impl Into<String>,
    pair_candidates: Vec<GraphStructuralScoredPairCandidateInputs>,
    fallback_edge_kind: impl Into<String>,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<
    GraphStructuralGenericTopologyCandidateInputs,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection(
        GraphStructuralScoredPairCollectionCandidateInput {
            candidate_id: candidate_id.into(),
            pair_candidates,
            fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
    candidate_id: impl Into<String>,
    pair_candidates: Vec<GraphStructuralRawConnectedPairInputs>,
    fallback_edge_kind: impl Into<String>,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<
    GraphStructuralGenericTopologyCandidateInputs,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
        GraphStructuralRawConnectedPairCandidateInput {
            candidate_id: candidate_id.into(),
            pair_candidates,
            fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples<I, L, R>(
    candidate_id: impl Into<String>,
    pair_candidates: I,
    fallback_edge_kind: impl Into<String>,
    dependency_score: f64,
    keyword_score: f64,
    tag_score: f64,
) -> Result<
    GraphStructuralRawConnectedPairCollectionCandidateInputs,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
>
where
    I: IntoIterator<Item = (L, R, f64)>,
    L: Into<String>,
    R: Into<String>,
{
    super::build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples(
        GraphStructuralRawConnectedPairCollectionRawTupleInput {
            candidate_id: candidate_id.into(),
            pair_candidates,
            fallback_edge_kind: JuliaContractKind::from(fallback_edge_kind.into()),
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

fn build_graph_structural_pair_rerank_request_row(
    query: &GraphStructuralQueryContext,
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    signals: &GraphStructuralRerankSignals,
) -> Result<
    GraphStructuralRerankRequestRow,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_pair_rerank_request_row(GraphStructuralPairRerankRequestInput {
        query: query.clone(),
        pair: build_graph_structural_pair_candidate_inputs(left_id, right_id, edge_kinds),
        signals: signals.clone(),
    })
}

fn build_graph_structural_pair_candidate_subgraph(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
) -> Result<
    GraphStructuralCandidateSubgraph,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_pair_candidate_subgraph(
        build_graph_structural_pair_candidate_inputs(left_id, right_id, edge_kinds),
    )
}

fn build_graph_structural_pair_filter_request_row(
    query: &GraphStructuralQueryContext,
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    constraint: &GraphStructuralFilterConstraint,
) -> Result<
    GraphStructuralFilterRequestRow,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_pair_filter_request_row(GraphStructuralPairFilterRequestInput {
        query: query.clone(),
        pair: build_graph_structural_pair_candidate_inputs(left_id, right_id, edge_kinds),
        constraint: constraint.clone(),
    })
}

fn build_graph_structural_keyword_tag_pair_rerank_request_row(
    query: GraphStructuralKeywordTagQueryInputs,
    pair: GraphStructuralPairCandidateInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
    tag_match: bool,
) -> Result<
    GraphStructuralRerankRequestRow,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_keyword_tag_pair_rerank_request_row(
        GraphStructuralKeywordTagPairRerankRequestInput {
            query,
            pair,
            semantic_score,
            dependency_score,
            keyword_match,
            tag_match,
        },
    )
}

fn build_graph_structural_keyword_overlap_pair_rerank_request_row(
    query: GraphStructuralKeywordTagQueryInputs,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
    pair: GraphStructuralPairCandidateInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> Result<
    GraphStructuralRerankRequestRow,
    xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError,
> {
    super::build_graph_structural_keyword_overlap_pair_rerank_request_row(
        GraphStructuralKeywordOverlapPairRerankRowInput {
            query,
            left_tags,
            right_tags,
            pair,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}
