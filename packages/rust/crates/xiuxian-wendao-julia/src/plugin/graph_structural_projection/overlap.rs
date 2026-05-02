//! Keyword-overlap DTOs for graph-structural projection.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::core::{
    GraphStructuralQueryAnchor, GraphStructuralQueryContext, GraphStructuralRerankSignals,
};
use super::pair::{
    GraphStructuralKeywordTagQueryInputs, GraphStructuralPairCandidateInputs,
    build_graph_structural_pair_candidate_inputs,
};
use super::support::{binary_plane_score, graph_structural_projection_error, normalize_non_blank};

/// Raw node-metadata inputs for the graph-structural projection helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralNodeMetadataInputs {
    pub(super) tags: Vec<String>,
}

impl GraphStructuralNodeMetadataInputs {
    /// Store one node-metadata input bundle for later normalization.
    #[must_use]
    pub fn new(tags: Vec<String>) -> Self {
        Self { tags }
    }
}

/// Raw keyword-overlap inputs that combine query, metadata, and pair data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordOverlapPairInputs {
    pub(super) query_inputs: GraphStructuralKeywordTagQueryInputs,
    pub(super) left_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) right_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) pair_inputs: GraphStructuralPairCandidateInputs,
}

impl GraphStructuralKeywordOverlapPairInputs {
    /// Store one keyword-overlap input bundle for later normalization.
    #[must_use]
    pub fn new(
        query_inputs: GraphStructuralKeywordTagQueryInputs,
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
    ) -> Self {
        Self {
            query_inputs,
            left_metadata,
            right_metadata,
            pair_inputs,
        }
    }
}

/// Raw scored inputs for one metadata-aware rerank request.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapPairRerankInputs {
    pub(super) metadata_inputs: GraphStructuralKeywordOverlapPairInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_match: bool,
}

impl GraphStructuralKeywordOverlapPairRerankInputs {
    /// Store one metadata-aware rerank input bundle for later normalization.
    #[must_use]
    pub fn new(
        metadata_inputs: GraphStructuralKeywordOverlapPairInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }
}

/// Higher-level metadata-aware request inputs for one keyword-overlap rerank request.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapPairRequestInputs {
    pub(super) metadata_inputs: GraphStructuralKeywordOverlapPairInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_match: bool,
}

impl GraphStructuralKeywordOverlapPairRequestInputs {
    /// Store one higher-level keyword-overlap request input bundle.
    #[must_use]
    pub fn new(
        query_inputs: GraphStructuralKeywordTagQueryInputs,
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self {
            metadata_inputs: GraphStructuralKeywordOverlapPairInputs::new(
                query_inputs,
                left_metadata,
                right_metadata,
                pair_inputs,
            ),
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }
}

/// Shared query inputs reused across keyword-overlap pair requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordOverlapQueryInputs {
    pub(super) query_id: String,
    pub(super) retrieval_layer: i32,
    pub(super) query_max_layers: i32,
    pub(super) keyword_anchors: Vec<String>,
    pub(super) edge_constraint_kinds: Vec<String>,
}

impl GraphStructuralKeywordOverlapQueryInputs {
    /// Store one shared keyword-overlap query input bundle.
    #[must_use]
    pub fn new(
        query_id: impl Into<String>,
        retrieval_layer: i32,
        query_max_layers: i32,
        keyword_anchors: Vec<String>,
        edge_constraint_kinds: Vec<String>,
    ) -> Self {
        Self {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        }
    }
}

/// Build one shared keyword-overlap query input bundle from raw query fields.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually constructing the shared-query DTO layer.
#[must_use]
pub fn build_graph_structural_keyword_overlap_query_inputs(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> GraphStructuralKeywordOverlapQueryInputs {
    GraphStructuralKeywordOverlapQueryInputs::new(
        query_id,
        retrieval_layer,
        query_max_layers,
        keyword_anchors,
        edge_constraint_kinds,
    )
}

/// Raw per-candidate inputs reused with one shared keyword-overlap query.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapRawCandidateInputs {
    pub(super) metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_match: bool,
}

impl GraphStructuralKeywordOverlapRawCandidateInputs {
    /// Store one raw keyword-overlap candidate input bundle.
    #[must_use]
    pub fn new(
        metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }
}

/// Build one raw keyword-overlap candidate bundle from pair metadata and
/// retrieval scores.
///
/// This keeps host consumers on a plugin-owned raw staging seam instead of
/// manually assembling per-candidate raw DTOs inline.
#[must_use]
pub fn build_graph_structural_keyword_overlap_raw_candidate_inputs(
    metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapRawCandidateInputs {
    GraphStructuralKeywordOverlapRawCandidateInputs::new(
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    )
}

/// Per-candidate normalized inputs reused with one shared keyword-overlap query.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapCandidateInputs {
    pub(super) left_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) right_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) pair_inputs: GraphStructuralPairCandidateInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_match: bool,
}

impl GraphStructuralKeywordOverlapCandidateInputs {
    /// Store one keyword-overlap candidate input bundle.
    #[must_use]
    pub fn new(
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self {
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }
}

/// Raw metadata inputs for one keyword-overlap candidate before score attachment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordOverlapCandidateMetadataInputs {
    pub(super) left_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) right_metadata: GraphStructuralNodeMetadataInputs,
    pub(super) pair_inputs: GraphStructuralPairCandidateInputs,
}

impl GraphStructuralKeywordOverlapCandidateMetadataInputs {
    /// Store one keyword-overlap candidate metadata bundle.
    #[must_use]
    pub fn new(
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
    ) -> Self {
        Self {
            left_metadata,
            right_metadata,
            pair_inputs,
        }
    }
}

/// Build one keyword-overlap candidate input bundle from raw pair metadata.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually assembling node-metadata and pair-candidate DTOs.
#[must_use]
pub fn build_graph_structural_keyword_overlap_candidate_inputs(
    metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    let GraphStructuralKeywordOverlapCandidateMetadataInputs {
        left_metadata,
        right_metadata,
        pair_inputs,
    } = metadata_inputs;
    GraphStructuralKeywordOverlapCandidateInputs::new(
        left_metadata,
        right_metadata,
        pair_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    )
}

/// Build one keyword-overlap candidate metadata bundle from raw pair ids, edge
/// kinds, and node tags.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually assembling node-metadata and pair-candidate DTOs.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
) -> GraphStructuralKeywordOverlapCandidateMetadataInputs {
    GraphStructuralKeywordOverlapCandidateMetadataInputs::new(
        GraphStructuralNodeMetadataInputs::new(left_tags),
        GraphStructuralNodeMetadataInputs::new(right_tags),
        build_graph_structural_pair_candidate_inputs(left_id, right_id, edge_kinds),
    )
}

/// Build one keyword-overlap candidate input bundle from one staged raw
/// candidate bundle.
///
/// This preserves the narrower plugin-owned seam for callers that already hold
/// the raw candidate bundle.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(
    raw_candidate_inputs: GraphStructuralKeywordOverlapRawCandidateInputs,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    let GraphStructuralKeywordOverlapRawCandidateInputs {
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    } = raw_candidate_inputs;
    build_graph_structural_keyword_overlap_pair_candidate_inputs(
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    )
}

/// Build one keyword-overlap candidate input bundle from staged pair metadata
/// and rerank scores.
///
/// This preserves the narrower plugin-owned seam for callers that already hold
/// the metadata bundle.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_candidate_inputs(
    metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    build_graph_structural_keyword_overlap_candidate_inputs(
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    )
}

/// Build one query context from keyword and tag anchor values.
///
/// Keyword anchors are emitted first, followed by tag anchors.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the query id is blank, the layer
/// bounds are invalid, both anchor lists are empty, or any anchor or
/// edge-constraint value is blank.
pub fn build_graph_structural_keyword_tag_query_context(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    tag_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> Result<GraphStructuralQueryContext, RepoIntelligenceError> {
    let mut anchors = Vec::with_capacity(keyword_anchors.len() + tag_anchors.len());
    for keyword in keyword_anchors {
        anchors.push(GraphStructuralQueryAnchor::new("keyword", keyword)?);
    }
    for tag in tag_anchors {
        anchors.push(GraphStructuralQueryAnchor::new("tag", tag)?);
    }
    GraphStructuralQueryContext::new(
        query_id,
        retrieval_layer,
        query_max_layers,
        anchors,
        edge_constraint_kinds,
    )
}

/// Build one rerank-signal set from semantic scores plus binary keyword or tag matches.
///
/// `keyword_match` and `tag_match` are normalized to `1.0` when true and
/// `0.0` when false.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when `semantic_score` or
/// `dependency_score` is negative or not finite.
pub fn build_graph_structural_keyword_tag_rerank_signals(
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
    tag_match: bool,
) -> Result<GraphStructuralRerankSignals, RepoIntelligenceError> {
    GraphStructuralRerankSignals::new(
        semantic_score,
        dependency_score,
        binary_plane_score(keyword_match),
        binary_plane_score(tag_match),
    )
}

/// Constraint settings that feed structural filter evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralFilterConstraint {
    constraint_kind: String,
    required_boundary_size: i32,
}

impl GraphStructuralFilterConstraint {
    /// Create one normalized structural filter constraint.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the constraint kind is blank or
    /// the required boundary size is negative.
    pub fn new(
        constraint_kind: impl Into<String>,
        required_boundary_size: i32,
    ) -> Result<Self, RepoIntelligenceError> {
        if required_boundary_size < 0 {
            return Err(graph_structural_projection_error(format!(
                "required boundary size must be non-negative; found {required_boundary_size}"
            )));
        }
        Ok(Self {
            constraint_kind: normalize_non_blank(constraint_kind.into(), "constraint kind")?,
            required_boundary_size,
        })
    }

    /// Return the normalized constraint kind.
    #[must_use]
    pub fn constraint_kind(&self) -> &str {
        &self.constraint_kind
    }

    /// Return the required boundary size.
    #[must_use]
    pub fn required_boundary_size(&self) -> i32 {
        self.required_boundary_size
    }
}
