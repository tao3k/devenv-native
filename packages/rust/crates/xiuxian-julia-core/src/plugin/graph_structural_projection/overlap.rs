//! Keyword-overlap DTOs for graph-structural projection.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::core::{
    GraphStructuralQueryAnchor, GraphStructuralQueryContext, GraphStructuralQueryContextInput,
    GraphStructuralRerankSignals,
};
use super::pair::{
    GraphStructuralKeywordTagQueryInput, GraphStructuralKeywordTagQueryInputs,
    GraphStructuralPairCandidateInputs, build_graph_structural_pair_candidate_inputs,
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

/// Named input bundle for one metadata-aware keyword-overlap rerank row.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapPairRerankInput {
    /// Metadata, query, and pair inputs for the row.
    pub metadata_inputs: GraphStructuralKeywordOverlapPairInputs,
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Whether the pair matched a keyword anchor.
    pub keyword_match: bool,
}

impl GraphStructuralKeywordOverlapPairRerankInputs {
    /// Store one metadata-aware rerank input bundle for later normalization.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordOverlapPairRerankInput) -> Self {
        let GraphStructuralKeywordOverlapPairRerankInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        } = input;
        Self {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }

    /// Store one metadata-aware rerank input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        metadata_inputs: GraphStructuralKeywordOverlapPairInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordOverlapPairRerankInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        })
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

/// Named input bundle for one higher-level keyword-overlap request.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapPairRequestInput {
    /// Shared query inputs for this candidate.
    pub query_inputs: GraphStructuralKeywordTagQueryInputs,
    /// Left node metadata.
    pub left_metadata: GraphStructuralNodeMetadataInputs,
    /// Right node metadata.
    pub right_metadata: GraphStructuralNodeMetadataInputs,
    /// Pair candidate data.
    pub pair_inputs: GraphStructuralPairCandidateInputs,
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Whether the pair matched a keyword anchor.
    pub keyword_match: bool,
}

impl GraphStructuralKeywordOverlapPairRequestInputs {
    /// Store one higher-level keyword-overlap request input bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordOverlapPairRequestInput) -> Self {
        let GraphStructuralKeywordOverlapPairRequestInput {
            query_inputs,
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        } = input;
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

    /// Store one higher-level keyword-overlap request input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        query_inputs: GraphStructuralKeywordTagQueryInputs,
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordOverlapPairRequestInput {
            query_inputs,
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        })
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

/// Named input bundle for one shared keyword-overlap query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordOverlapQueryInput {
    /// Query id attached to the structural request.
    pub query_id: String,
    /// Retrieval layer used by the graph route.
    pub retrieval_layer: i32,
    /// Maximum layer depth accepted for the query.
    pub query_max_layers: i32,
    /// Keyword anchors for the request.
    pub keyword_anchors: Vec<String>,
    /// Edge-kind constraints for the request.
    pub edge_constraint_kinds: Vec<String>,
}

impl GraphStructuralKeywordOverlapQueryInputs {
    /// Store one shared keyword-overlap query input bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordOverlapQueryInput) -> Self {
        let GraphStructuralKeywordOverlapQueryInput {
            query_id,
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        } = input;
        Self {
            query_id,
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        }
    }

    /// Store one shared keyword-overlap query input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        query_id: impl Into<String>,
        retrieval_layer: i32,
        query_max_layers: i32,
        keyword_anchors: Vec<String>,
        edge_constraint_kinds: Vec<String>,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordOverlapQueryInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        })
    }
}

/// Build one shared keyword-overlap query input bundle from raw query fields.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually constructing the shared-query DTO layer.
#[must_use]
pub fn build_graph_structural_keyword_overlap_query_inputs(
    input: GraphStructuralKeywordOverlapQueryInput,
) -> GraphStructuralKeywordOverlapQueryInputs {
    GraphStructuralKeywordOverlapQueryInputs::from_input(input)
}

/// Raw per-candidate inputs reused with one shared keyword-overlap query.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapRawCandidateInputs {
    pub(super) metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    pub(super) semantic_score: f64,
    pub(super) dependency_score: f64,
    pub(super) keyword_match: bool,
}

/// Named input bundle for one raw keyword-overlap candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapRawCandidateInput {
    /// Candidate metadata before score attachment.
    pub metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Whether the pair matched a keyword anchor.
    pub keyword_match: bool,
}

impl GraphStructuralKeywordOverlapRawCandidateInputs {
    /// Store one raw keyword-overlap candidate input bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordOverlapRawCandidateInput) -> Self {
        let GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        } = input;
        Self {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }

    /// Store one raw keyword-overlap candidate input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        metadata_inputs: GraphStructuralKeywordOverlapCandidateMetadataInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        })
    }
}

/// Build one raw keyword-overlap candidate bundle from pair metadata and
/// retrieval scores.
///
/// This keeps host consumers on a plugin-owned raw staging seam instead of
/// manually assembling per-candidate raw DTOs inline.
#[must_use]
pub fn build_graph_structural_keyword_overlap_raw_candidate_inputs(
    input: GraphStructuralKeywordOverlapRawCandidateInput,
) -> GraphStructuralKeywordOverlapRawCandidateInputs {
    GraphStructuralKeywordOverlapRawCandidateInputs::from_input(input)
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

/// Named input bundle for one normalized keyword-overlap candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralKeywordOverlapCandidateInput {
    /// Left node metadata.
    pub left_metadata: GraphStructuralNodeMetadataInputs,
    /// Right node metadata.
    pub right_metadata: GraphStructuralNodeMetadataInputs,
    /// Pair candidate data.
    pub pair_inputs: GraphStructuralPairCandidateInputs,
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Whether the pair matched a keyword anchor.
    pub keyword_match: bool,
}

impl GraphStructuralKeywordOverlapCandidateInputs {
    /// Store one keyword-overlap candidate input bundle.
    #[must_use]
    pub fn from_input(input: GraphStructuralKeywordOverlapCandidateInput) -> Self {
        let GraphStructuralKeywordOverlapCandidateInput {
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        } = input;
        Self {
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        }
    }

    /// Store one keyword-overlap candidate input bundle for tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(
        left_metadata: GraphStructuralNodeMetadataInputs,
        right_metadata: GraphStructuralNodeMetadataInputs,
        pair_inputs: GraphStructuralPairCandidateInputs,
        semantic_score: f64,
        dependency_score: f64,
        keyword_match: bool,
    ) -> Self {
        Self::from_input(GraphStructuralKeywordOverlapCandidateInput {
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        })
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
    input: GraphStructuralKeywordOverlapRawCandidateInput,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    let GraphStructuralKeywordOverlapRawCandidateInput {
        metadata_inputs,
        semantic_score,
        dependency_score,
        keyword_match,
    } = input;
    let GraphStructuralKeywordOverlapCandidateMetadataInputs {
        left_metadata,
        right_metadata,
        pair_inputs,
    } = metadata_inputs;
    GraphStructuralKeywordOverlapCandidateInputs::from_input(
        GraphStructuralKeywordOverlapCandidateInput {
            left_metadata,
            right_metadata,
            pair_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

/// Named input bundle for one keyword-overlap candidate metadata row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordOverlapCandidateMetadataInput {
    /// Left node id.
    pub left_id: String,
    /// Right node id.
    pub right_id: String,
    /// Edge kinds joining the two nodes.
    pub edge_kinds: Vec<String>,
    /// Left node tags.
    pub left_tags: Vec<String>,
    /// Right node tags.
    pub right_tags: Vec<String>,
}

/// Build one keyword-overlap candidate metadata bundle from raw pair ids, edge
/// kinds, and node tags.
///
/// This keeps host consumers on the plugin-owned staging seam instead of
/// manually assembling node-metadata and pair-candidate DTOs.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
    input: GraphStructuralKeywordOverlapCandidateMetadataInput,
) -> GraphStructuralKeywordOverlapCandidateMetadataInputs {
    let GraphStructuralKeywordOverlapCandidateMetadataInput {
        left_id,
        right_id,
        edge_kinds,
        left_tags,
        right_tags,
    } = input;
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
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

/// Build one keyword-overlap candidate input bundle from staged pair metadata
/// and rerank scores.
///
/// This preserves the narrower plugin-owned seam for callers that already hold
/// the metadata bundle.
#[must_use]
pub fn build_graph_structural_keyword_overlap_pair_candidate_inputs(
    input: GraphStructuralKeywordOverlapRawCandidateInput,
) -> GraphStructuralKeywordOverlapCandidateInputs {
    build_graph_structural_keyword_overlap_candidate_inputs(input)
}

/// Named input bundle for building keyword/tag query contexts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralKeywordTagQueryContextInput {
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
    input: GraphStructuralKeywordTagQueryContextInput,
) -> Result<GraphStructuralQueryContext, RepoIntelligenceError> {
    let GraphStructuralKeywordTagQueryContextInput {
        query_id,
        retrieval_layer,
        query_max_layers,
        keyword_anchors,
        tag_anchors,
        edge_constraint_kinds,
    } = input;
    let mut anchors = Vec::with_capacity(keyword_anchors.len() + tag_anchors.len());
    for keyword in keyword_anchors {
        anchors.push(GraphStructuralQueryAnchor::new("keyword", keyword)?);
    }
    for tag in tag_anchors {
        anchors.push(GraphStructuralQueryAnchor::new("tag", tag)?);
    }
    GraphStructuralQueryContext::from_input(GraphStructuralQueryContextInput {
        query_id,
        retrieval_layer,
        query_max_layers,
        anchors,
        edge_constraint_kinds,
    })
}

/// Binary match flags used by keyword/tag rerank signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphStructuralKeywordTagMatchFlags {
    /// Whether the row matched a keyword anchor.
    pub keyword_match: bool,
    /// Whether the row matched a tag anchor.
    pub tag_match: bool,
}

/// Named input bundle for keyword/tag rerank signal construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GraphStructuralKeywordTagRerankSignalInput {
    /// Semantic score before Julia rerank.
    pub semantic_score: f64,
    /// Dependency score before Julia rerank.
    pub dependency_score: f64,
    /// Binary keyword/tag match flags.
    pub matches: GraphStructuralKeywordTagMatchFlags,
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
    input: GraphStructuralKeywordTagRerankSignalInput,
) -> Result<GraphStructuralRerankSignals, RepoIntelligenceError> {
    let GraphStructuralKeywordTagRerankSignalInput {
        semantic_score,
        dependency_score,
        matches,
    } = input;
    GraphStructuralRerankSignals::new(
        semantic_score,
        dependency_score,
        binary_plane_score(matches.keyword_match),
        binary_plane_score(matches.tag_match),
    )
}

impl From<GraphStructuralKeywordOverlapQueryInputs> for GraphStructuralKeywordTagQueryInput {
    fn from(input: GraphStructuralKeywordOverlapQueryInputs) -> Self {
        Self {
            query_id: input.query_id,
            retrieval_layer: input.retrieval_layer,
            query_max_layers: input.query_max_layers,
            keyword_anchors: input.keyword_anchors,
            tag_anchors: Vec::new(),
            edge_constraint_kinds: input.edge_constraint_kinds,
        }
    }
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
