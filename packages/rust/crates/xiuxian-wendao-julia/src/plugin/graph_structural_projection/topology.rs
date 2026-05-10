//! Explicit-topology DTOs for graph-structural projection.

use std::collections::HashSet;

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::pair::{
    GraphStructuralPairCandidateInputs, GraphStructuralRawConnectedPairInputs,
    GraphStructuralScoredPairCandidateInputs, build_graph_structural_raw_connected_pair_inputs,
    build_graph_structural_scored_pair_candidate_inputs,
};
use super::support::{
    graph_structural_projection_error, normalize_non_blank, normalize_pair_endpoint_ids,
    normalize_string_list,
};
use crate::JuliaContractKind;

/// Raw generic explicit-edge topology inputs before score attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralGenericTopologyCandidateMetadataInputs {
    pub(super) candidate_id: String,
    pub(super) node_ids: Vec<String>,
    pub(super) edge_sources: Vec<String>,
    pub(super) edge_destinations: Vec<String>,
    pub(super) edge_kinds: Vec<String>,
}

/// Named inputs for one explicit-edge topology metadata bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralGenericTopologyCandidateMetadataInput {
    /// Stable candidate id.
    pub candidate_id: String,
    /// Candidate node ids.
    pub node_ids: Vec<String>,
    /// Candidate edge sources.
    pub edge_sources: Vec<String>,
    /// Candidate edge destinations.
    pub edge_destinations: Vec<String>,
    /// Candidate edge kinds.
    pub edge_kinds: Vec<String>,
}

impl GraphStructuralGenericTopologyCandidateMetadataInputs {
    /// Store one generic explicit-edge topology metadata bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralGenericTopologyCandidateMetadataInput) -> Self {
        let GraphStructuralGenericTopologyCandidateMetadataInput {
            candidate_id,
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        } = input;
        Self {
            candidate_id,
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        }
    }

    #[cfg(test)]
    pub(crate) fn new(
        candidate_id: impl Into<String>,
        node_ids: Vec<String>,
        edge_sources: Vec<String>,
        edge_destinations: Vec<String>,
        edge_kinds: Vec<String>,
    ) -> Self {
        Self::from_input(GraphStructuralGenericTopologyCandidateMetadataInput {
            candidate_id: candidate_id.into(),
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        })
    }
}

/// Build one generic explicit-edge topology metadata bundle from raw fields.
///
/// This keeps callers on a plugin-owned staging seam instead of manually
/// constructing generic topology metadata DTOs.
#[must_use]
pub fn build_graph_structural_generic_topology_candidate_metadata_inputs(
    input: GraphStructuralGenericTopologyCandidateMetadataInput,
) -> GraphStructuralGenericTopologyCandidateMetadataInputs {
    GraphStructuralGenericTopologyCandidateMetadataInputs::from_input(input)
}

/// Raw generic explicit-edge topology candidate inputs with staged plane
/// scores.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralGenericTopologyCandidateInputs {
    pub(super) metadata_inputs: GraphStructuralGenericTopologyCandidateMetadataInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_score: f64,
    pub(super) tag_score: f64,
}

/// Named inputs for one explicit-edge topology candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralGenericTopologyCandidateInput {
    /// Explicit topology metadata.
    pub metadata: GraphStructuralGenericTopologyCandidateMetadataInputs,
    /// Semantic plane score.
    pub semantic_score: f64,
    /// Dependency plane score.
    pub dependency_score: f64,
    /// Keyword plane score.
    pub keyword_score: f64,
    /// Tag plane score.
    pub tag_score: f64,
}

impl GraphStructuralGenericTopologyCandidateInputs {
    /// Store one generic explicit-edge topology candidate bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralGenericTopologyCandidateInput) -> Self {
        let GraphStructuralGenericTopologyCandidateInput {
            metadata,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        } = input;
        Self {
            metadata_inputs: metadata,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        }
    }

    #[cfg(test)]
    pub(crate) fn new(
        metadata_inputs: GraphStructuralGenericTopologyCandidateMetadataInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_score: f64,
        tag_score: f64,
    ) -> Self {
        Self::from_input(GraphStructuralGenericTopologyCandidateInput {
            metadata: metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        })
    }
}

/// Build one generic explicit-edge topology candidate bundle from raw topology
/// metadata and staged plane scores.
///
/// This keeps callers on a plugin-owned staging seam instead of manually
/// constructing generic topology candidate DTOs.
#[must_use]
pub fn build_graph_structural_generic_topology_candidate_inputs(
    input: GraphStructuralGenericTopologyCandidateInput,
) -> GraphStructuralGenericTopologyCandidateInputs {
    GraphStructuralGenericTopologyCandidateInputs::from_input(input)
}

/// Build one generic explicit-edge topology metadata bundle from a staged pair
/// collection.
///
/// Any pair with an empty `edge_kinds` list is promoted with the supplied
/// `fallback_edge_kind`, which keeps bounded pair collections from
/// `LinkGraphAgenticExpansionPlan` usable on the generic-topology live seam.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the candidate id or fallback edge
/// kind is blank, the pair collection is empty, any pair endpoint is invalid,
/// or any non-empty edge-kind list contains blank items.
pub fn build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection(
    candidate_id: impl Into<String>,
    pair_candidates: Vec<GraphStructuralPairCandidateInputs>,
    fallback_edge_kind: impl Into<String>,
) -> Result<GraphStructuralGenericTopologyCandidateMetadataInputs, RepoIntelligenceError> {
    let candidate_id = normalize_non_blank(candidate_id.into(), "generic topology candidate id")?;
    if pair_candidates.is_empty() {
        return Err(graph_structural_projection_error(
            "generic topology pair collection must contain at least one pair",
        ));
    }
    let fallback_edge_kind = normalize_non_blank(fallback_edge_kind.into(), "fallback edge kind")?;

    let mut node_ids = Vec::new();
    let mut seen_nodes = HashSet::new();
    let mut edge_sources = Vec::new();
    let mut edge_destinations = Vec::new();
    let mut edge_kinds = Vec::new();
    let mut seen_edges = HashSet::new();

    for pair in pair_candidates {
        push_pair_collection_topology_metadata(
            pair,
            &fallback_edge_kind,
            &mut node_ids,
            &mut seen_nodes,
            &mut edge_sources,
            &mut edge_destinations,
            &mut edge_kinds,
            &mut seen_edges,
        )?;
    }

    Ok(
        GraphStructuralGenericTopologyCandidateMetadataInputs::from_input(
            GraphStructuralGenericTopologyCandidateMetadataInput {
                candidate_id,
                node_ids,
                edge_sources,
                edge_destinations,
                edge_kinds,
            },
        ),
    )
}

fn push_pair_collection_topology_metadata(
    pair: GraphStructuralPairCandidateInputs,
    fallback_edge_kind: &str,
    node_ids: &mut Vec<String>,
    seen_nodes: &mut HashSet<String>,
    edge_sources: &mut Vec<String>,
    edge_destinations: &mut Vec<String>,
    edge_kinds: &mut Vec<String>,
    seen_edges: &mut HashSet<(String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let (left_id, right_id) = normalize_pair_endpoint_ids(pair.left_id, pair.right_id)?;
    push_unique_pair_nodes(&left_id, &right_id, node_ids, seen_nodes);
    let normalized_edge_kinds =
        normalize_pair_collection_edge_kinds(pair.edge_kinds, fallback_edge_kind)?;
    push_unique_pair_edges(
        &left_id,
        &right_id,
        normalized_edge_kinds,
        edge_sources,
        edge_destinations,
        edge_kinds,
        seen_edges,
    );
    Ok(())
}

fn push_unique_pair_nodes(
    left_id: &str,
    right_id: &str,
    node_ids: &mut Vec<String>,
    seen_nodes: &mut HashSet<String>,
) {
    if seen_nodes.insert(left_id.to_string()) {
        node_ids.push(left_id.to_string());
    }
    if seen_nodes.insert(right_id.to_string()) {
        node_ids.push(right_id.to_string());
    }
}

fn normalize_pair_collection_edge_kinds(
    edge_kinds: Vec<String>,
    fallback_edge_kind: &str,
) -> Result<Vec<String>, RepoIntelligenceError> {
    if edge_kinds.is_empty() {
        return Ok(vec![fallback_edge_kind.to_string()]);
    }
    normalize_string_list(edge_kinds, "pair edge kinds", false)
}

fn push_unique_pair_edges(
    left_id: &str,
    right_id: &str,
    normalized_edge_kinds: Vec<String>,
    edge_sources: &mut Vec<String>,
    edge_destinations: &mut Vec<String>,
    edge_kinds: &mut Vec<String>,
    seen_edges: &mut HashSet<(String, String, String)>,
) {
    for edge_kind in normalized_edge_kinds {
        let edge_key = (left_id.to_string(), right_id.to_string(), edge_kind.clone());
        if seen_edges.insert(edge_key) {
            edge_sources.push(left_id.to_string());
            edge_destinations.push(right_id.to_string());
            edge_kinds.push(edge_kind);
        }
    }
}

/// Named inputs for one generic-topology candidate built from a pair collection.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralGenericTopologyPairCollectionInput {
    /// Stable candidate id for the aggregated topology.
    pub candidate_id: String,
    /// Pair candidates to aggregate.
    pub pair_candidates: Vec<GraphStructuralPairCandidateInputs>,
    /// Edge kind used when a pair has no explicit edge kind.
    pub fallback_edge_kind: JuliaContractKind,
    /// Semantic plane score.
    pub semantic_score: f64,
    /// Dependency plane score.
    pub dependency_score: f64,
    /// Keyword plane score.
    pub keyword_score: f64,
    /// Tag plane score.
    pub tag_score: f64,
}

/// Build one generic explicit-edge topology candidate bundle from a staged
/// pair collection plus plane scores.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the pair-collection topology cannot
/// be normalized into the bounded generic-topology request shape.
pub fn build_graph_structural_generic_topology_candidate_inputs_from_pair_collection(
    input: GraphStructuralGenericTopologyPairCollectionInput,
) -> Result<GraphStructuralGenericTopologyCandidateInputs, RepoIntelligenceError> {
    Ok(build_graph_structural_generic_topology_candidate_inputs(
        GraphStructuralGenericTopologyCandidateInput {
            metadata:
                build_graph_structural_generic_topology_candidate_metadata_inputs_from_pair_collection(
                    input.candidate_id,
                    input.pair_candidates,
                    input.fallback_edge_kind.into_string(),
                )?,
            semantic_score: input.semantic_score,
            dependency_score: input.dependency_score,
            keyword_score: input.keyword_score,
            tag_score: input.tag_score,
        },
    ))
}

/// Named inputs for one generic-topology candidate built from scored pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralScoredPairCollectionCandidateInput {
    /// Stable candidate id for the aggregated topology.
    pub candidate_id: String,
    /// Scored pair candidates to aggregate.
    pub pair_candidates: Vec<GraphStructuralScoredPairCandidateInputs>,
    /// Edge kind used when a pair has no explicit edge kind.
    pub fallback_edge_kind: JuliaContractKind,
    /// Dependency plane score.
    pub dependency_score: f64,
    /// Keyword plane score.
    pub keyword_score: f64,
    /// Tag plane score.
    pub tag_score: f64,
}

/// Build one generic explicit-edge topology candidate bundle from a scored pair
/// collection and staged non-semantic plane scores.
///
/// The candidate-level semantic score is the arithmetic mean of the pair-level
/// semantic scores after score normalization.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the pair collection is empty, any
/// pair-level semantic score is negative or non-finite, or the staged
/// pair-collection topology cannot be normalized into the bounded
/// generic-topology request shape.
pub fn build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection(
    input: GraphStructuralScoredPairCollectionCandidateInput,
) -> Result<GraphStructuralGenericTopologyCandidateInputs, RepoIntelligenceError> {
    let GraphStructuralScoredPairCollectionCandidateInput {
        candidate_id,
        pair_candidates,
        fallback_edge_kind,
        dependency_score,
        keyword_score,
        tag_score,
    } = input;
    if pair_candidates.is_empty() {
        return Err(graph_structural_projection_error(
            "generic topology scored pair collection must contain at least one pair",
        ));
    }

    let pair_count = u32::try_from(pair_candidates.len()).map_err(|_error| {
        graph_structural_projection_error(
            "generic topology scored pair collection exceeds supported pair count",
        )
    })?;
    let semantic_score = pair_candidates
        .iter()
        .map(|pair| pair.semantic_score)
        .sum::<f64>()
        / f64::from(pair_count);
    let pair_candidates = pair_candidates
        .into_iter()
        .map(|pair| pair.pair_inputs)
        .collect::<Vec<_>>();

    build_graph_structural_generic_topology_candidate_inputs_from_pair_collection(
        GraphStructuralGenericTopologyPairCollectionInput {
            candidate_id,
            pair_candidates,
            fallback_edge_kind,
            semantic_score,
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

/// Named inputs for one generic-topology candidate built from raw connected pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRawConnectedPairCandidateInput {
    /// Stable candidate id for the aggregated topology.
    pub candidate_id: String,
    /// Raw connected pairs to aggregate.
    pub pair_candidates: Vec<GraphStructuralRawConnectedPairInputs>,
    /// Edge kind used when a pair has no explicit edge kind.
    pub fallback_edge_kind: JuliaContractKind,
    /// Dependency plane score.
    pub dependency_score: f64,
    /// Keyword plane score.
    pub keyword_score: f64,
    /// Tag plane score.
    pub tag_score: f64,
}

/// Build one generic explicit-edge topology candidate bundle from a raw
/// connected-pair collection and staged non-semantic plane scores.
///
/// Each connected pair is promoted into one scored pair candidate with an
/// empty edge-kind list; `fallback_edge_kind` supplies the bounded edge label
/// for the eventual generic-topology materialization.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the raw pair collection is empty,
/// any connected pair is invalid, or the staged pair-collection topology
/// cannot be normalized into the bounded generic-topology request shape.
pub fn build_graph_structural_generic_topology_candidate_inputs_from_raw_connected_pairs(
    input: GraphStructuralRawConnectedPairCandidateInput,
) -> Result<GraphStructuralGenericTopologyCandidateInputs, RepoIntelligenceError> {
    let GraphStructuralRawConnectedPairCandidateInput {
        candidate_id,
        pair_candidates,
        fallback_edge_kind,
        dependency_score,
        keyword_score,
        tag_score,
    } = input;
    let pair_candidates = pair_candidates
        .into_iter()
        .map(|pair| {
            build_graph_structural_scored_pair_candidate_inputs(
                pair.left_id,
                pair.right_id,
                Vec::new(),
                pair.semantic_score,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    build_graph_structural_generic_topology_candidate_inputs_from_scored_pair_collection(
        GraphStructuralScoredPairCollectionCandidateInput {
            candidate_id,
            pair_candidates,
            fallback_edge_kind,
            dependency_score,
            keyword_score,
            tag_score,
        },
    )
}

/// Raw connected-pair collection inputs for one generic-topology candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRawConnectedPairCollectionCandidateInputs {
    pub(super) candidate_id: String,
    pub(super) pair_candidates: Vec<GraphStructuralRawConnectedPairInputs>,
    pub(super) fallback_edge_kind: String,
    pub(super) dependency_score: f64,
    pub(super) keyword_score: f64,
    pub(super) tag_score: f64,
}

impl GraphStructuralRawConnectedPairCollectionCandidateInputs {
    /// Store one raw connected-pair collection bundle for later normalization.
    #[must_use]
    pub fn new(input: GraphStructuralRawConnectedPairCandidateInput) -> Self {
        Self {
            candidate_id: input.candidate_id,
            pair_candidates: input.pair_candidates,
            fallback_edge_kind: input.fallback_edge_kind.into_string(),
            dependency_score: input.dependency_score,
            keyword_score: input.keyword_score,
            tag_score: input.tag_score,
        }
    }
}

/// Build one raw connected-pair collection bundle from raw candidate-level
/// fields.
#[must_use]
pub fn build_graph_structural_raw_connected_pair_collection_candidate_inputs(
    input: GraphStructuralRawConnectedPairCandidateInput,
) -> GraphStructuralRawConnectedPairCollectionCandidateInputs {
    GraphStructuralRawConnectedPairCollectionCandidateInputs::new(input)
}

/// Named raw tuple inputs for one raw connected-pair collection.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRawConnectedPairCollectionRawTupleInput<I> {
    /// Stable candidate id for the aggregated topology.
    pub candidate_id: String,
    /// Raw pair tuples to normalize.
    pub pair_candidates: I,
    /// Edge kind used when a pair has no explicit edge kind.
    pub fallback_edge_kind: JuliaContractKind,
    /// Dependency plane score.
    pub dependency_score: f64,
    /// Keyword plane score.
    pub keyword_score: f64,
    /// Tag plane score.
    pub tag_score: f64,
}

/// Build one raw connected-pair collection bundle from raw tuple fields.
///
/// Each tuple is normalized through the existing raw connected-pair helper so
/// callers do not manually materialize `GraphStructuralRawConnectedPairInputs`
/// before using the collection-level generic-topology seam.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any tuple contains invalid pair
/// endpoints or a negative or non-finite semantic score.
pub fn build_graph_structural_raw_connected_pair_collection_candidate_inputs_from_raw_tuples<
    I,
    L,
    R,
>(
    input: GraphStructuralRawConnectedPairCollectionRawTupleInput<I>,
) -> Result<GraphStructuralRawConnectedPairCollectionCandidateInputs, RepoIntelligenceError>
where
    I: IntoIterator<Item = (L, R, f64)>,
    L: Into<String>,
    R: Into<String>,
{
    let pair_candidates = input
        .pair_candidates
        .into_iter()
        .map(|(left_id, right_id, semantic_score)| {
            build_graph_structural_raw_connected_pair_inputs(left_id, right_id, semantic_score)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(
        build_graph_structural_raw_connected_pair_collection_candidate_inputs(
            GraphStructuralRawConnectedPairCandidateInput {
                candidate_id: input.candidate_id,
                pair_candidates,
                fallback_edge_kind: input.fallback_edge_kind,
                dependency_score: input.dependency_score,
                keyword_score: input.keyword_score,
                tag_score: input.tag_score,
            },
        ),
    )
}
