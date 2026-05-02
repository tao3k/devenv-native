//! Core graph-structural query and candidate DTOs.

use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::support::{
    graph_structural_projection_error, normalize_non_blank, normalize_non_negative_score,
    normalize_string_list,
};

/// One query anchor used to align graph-structural request rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralQueryAnchor {
    plane: String,
    value: String,
}

impl GraphStructuralQueryAnchor {
    /// Create one normalized graph-structural query anchor.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when `plane` or `value` is blank after
    /// normalization.
    pub fn new(
        plane: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            plane: normalize_non_blank(plane.into(), "query anchor plane")?,
            value: normalize_non_blank(value.into(), "query anchor value")?,
        })
    }

    /// Return the normalized anchor plane.
    #[must_use]
    pub fn plane(&self) -> &str {
        &self.plane
    }

    /// Return the normalized anchor value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Query-scoped context shared by graph-structural request rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralQueryContext {
    query_id: String,
    retrieval_layer: i32,
    query_max_layers: i32,
    anchors: Vec<GraphStructuralQueryAnchor>,
    edge_constraint_kinds: Vec<String>,
}

impl GraphStructuralQueryContext {
    /// Create normalized query context for graph-structural request rows.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the query id is blank, the layer
    /// bounds are invalid, the anchor list is empty, or any edge-constraint
    /// value is blank.
    pub fn new(
        query_id: impl Into<String>,
        retrieval_layer: i32,
        query_max_layers: i32,
        anchors: Vec<GraphStructuralQueryAnchor>,
        edge_constraint_kinds: Vec<String>,
    ) -> Result<Self, RepoIntelligenceError> {
        if retrieval_layer < 0 {
            return Err(graph_structural_projection_error(format!(
                "retrieval layer must be non-negative; found {retrieval_layer}"
            )));
        }
        if query_max_layers < 1 {
            return Err(graph_structural_projection_error(format!(
                "query max layers must be at least 1; found {query_max_layers}"
            )));
        }
        if anchors.is_empty() {
            return Err(graph_structural_projection_error(
                "at least one query anchor is required",
            ));
        }
        Ok(Self {
            query_id: normalize_non_blank(query_id.into(), "query id")?,
            retrieval_layer,
            query_max_layers,
            anchors,
            edge_constraint_kinds: normalize_string_list(
                edge_constraint_kinds,
                "edge constraint kinds",
                true,
            )?,
        })
    }

    /// Return the normalized query id.
    #[must_use]
    pub fn query_id(&self) -> &str {
        &self.query_id
    }

    /// Return the retrieval layer for this query context.
    #[must_use]
    pub fn retrieval_layer(&self) -> i32 {
        self.retrieval_layer
    }

    /// Return the maximum retrieval depth for this query context.
    #[must_use]
    pub fn query_max_layers(&self) -> i32 {
        self.query_max_layers
    }

    /// Return the normalized anchor list.
    #[must_use]
    pub fn anchors(&self) -> &[GraphStructuralQueryAnchor] {
        &self.anchors
    }

    /// Return the normalized edge-constraint kinds.
    #[must_use]
    pub fn edge_constraint_kinds(&self) -> &[String] {
        &self.edge_constraint_kinds
    }
}

/// Bounded candidate subgraph selected for structural evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStructuralCandidateSubgraph {
    candidate_id: String,
    node_ids: Vec<String>,
    edge_sources: Vec<String>,
    edge_destinations: Vec<String>,
    edge_kinds: Vec<String>,
}

impl GraphStructuralCandidateSubgraph {
    /// Create one normalized graph-structural candidate subgraph.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the candidate id is blank, the
    /// node list is empty, or any node or edge-kind value is blank.
    pub fn new(
        candidate_id: impl Into<String>,
        node_ids: Vec<String>,
        edge_sources: Vec<String>,
        edge_destinations: Vec<String>,
        edge_kinds: Vec<String>,
    ) -> Result<Self, RepoIntelligenceError> {
        let node_ids = normalize_string_list(node_ids, "candidate node ids", false)?;
        let edge_sources = normalize_string_list(edge_sources, "candidate edge sources", true)?;
        let edge_destinations =
            normalize_string_list(edge_destinations, "candidate edge destinations", true)?;
        let edge_kinds = normalize_string_list(edge_kinds, "candidate edge kinds", true)?;
        if edge_sources.len() != edge_destinations.len() {
            return Err(graph_structural_projection_error(
                "candidate edge endpoints must stay aligned",
            ));
        }
        if edge_sources.len() != edge_kinds.len() {
            return Err(graph_structural_projection_error(
                "candidate edge endpoints must align with edge kinds",
            ));
        }
        let node_ids_set = node_ids
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        for (src_id, dst_id) in edge_sources.iter().zip(edge_destinations.iter()) {
            if src_id == dst_id {
                return Err(graph_structural_projection_error(
                    "candidate edge endpoints must not be identical",
                ));
            }
            if !node_ids_set.contains(src_id.as_str()) {
                return Err(graph_structural_projection_error(format!(
                    "candidate edge source `{src_id}` is not present in candidate node ids",
                )));
            }
            if !node_ids_set.contains(dst_id.as_str()) {
                return Err(graph_structural_projection_error(format!(
                    "candidate edge destination `{dst_id}` is not present in candidate node ids",
                )));
            }
        }
        Ok(Self {
            candidate_id: normalize_non_blank(candidate_id.into(), "candidate id")?,
            node_ids,
            edge_sources,
            edge_destinations,
            edge_kinds,
        })
    }

    /// Return the normalized candidate id.
    #[must_use]
    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    /// Return the normalized candidate node ids.
    #[must_use]
    pub fn node_ids(&self) -> &[String] {
        &self.node_ids
    }

    /// Return the normalized candidate edge sources.
    #[must_use]
    pub fn edge_sources(&self) -> &[String] {
        &self.edge_sources
    }

    /// Return the normalized candidate edge destinations.
    #[must_use]
    pub fn edge_destinations(&self) -> &[String] {
        &self.edge_destinations
    }

    /// Return the normalized candidate edge kinds.
    #[must_use]
    pub fn edge_kinds(&self) -> &[String] {
        &self.edge_kinds
    }
}

/// Per-plane scores that feed structural rerank evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphStructuralRerankSignals {
    semantic: f64,
    dependency: f64,
    keyword: f64,
    tag: f64,
}

impl GraphStructuralRerankSignals {
    /// Create one normalized set of structural rerank plane scores.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when any score is negative or not
    /// finite.
    pub fn new(
        semantic_score: f64,
        dependency_score: f64,
        keyword_score: f64,
        tag_score: f64,
    ) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            semantic: normalize_non_negative_score(semantic_score, "semantic score")?,
            dependency: normalize_non_negative_score(dependency_score, "dependency score")?,
            keyword: normalize_non_negative_score(keyword_score, "keyword score")?,
            tag: normalize_non_negative_score(tag_score, "tag score")?,
        })
    }

    /// Return the normalized semantic score.
    #[must_use]
    pub fn semantic_score(&self) -> f64 {
        self.semantic
    }

    /// Return the normalized dependency score.
    #[must_use]
    pub fn dependency_score(&self) -> f64 {
        self.dependency
    }

    /// Return the normalized keyword score.
    #[must_use]
    pub fn keyword_score(&self) -> f64 {
        self.keyword
    }

    /// Return the normalized tag score.
    #[must_use]
    pub fn tag_score(&self) -> f64 {
        self.tag
    }
}
