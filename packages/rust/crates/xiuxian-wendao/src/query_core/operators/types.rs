//! `query_core::operators::types` owns Wendao query core operators types behavior.

use std::collections::{BTreeSet, HashSet};

use crate::query_core::types::WendaoRelation;

/// Retrieval corpus routed through the Phase-1 vector-search operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetrievalCorpus {
    /// Repo content chunk search.
    #[default]
    RepoContent,
    /// Repo entity search.
    RepoEntity,
}

/// Graph traversal direction used by Phase-1 neighbor queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphDirection {
    /// Traverse incoming edges only.
    Incoming,
    /// Traverse outgoing edges only.
    Outgoing,
    /// Traverse both incoming and outgoing edges.
    #[default]
    Both,
}

/// Typed request for retrieval-first candidate search.
#[derive(Debug, Clone)]
pub struct VectorSearchOp {
    /// Search corpus targeted by the request.
    pub corpus: RetrievalCorpus,
    /// Repository identifier to search within.
    pub repo_id: String,
    /// User-provided search term.
    pub search_term: String,
    /// Optional language filters.
    pub language_filters: HashSet<String>,
    /// Optional kind filters.
    pub kind_filters: HashSet<String>,
    /// Maximum number of candidates to return.
    pub limit: usize,
}

impl Default for VectorSearchOp {
    fn default() -> Self {
        Self {
            corpus: RetrievalCorpus::RepoContent,
            repo_id: String::new(),
            search_term: String::new(),
            language_filters: HashSet::new(),
            kind_filters: HashSet::new(),
            limit: 10,
        }
    }
}

/// Typed request for graph-neighbor lookup.
#[derive(Debug, Clone)]
pub struct GraphNeighborsOp {
    /// Seed node identifier.
    pub node_id: String,
    /// Traversal direction.
    pub direction: GraphDirection,
    /// Hop count.
    pub hops: usize,
    /// Maximum number of neighbors to return.
    pub limit: usize,
}

impl Default for GraphNeighborsOp {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            direction: GraphDirection::Both,
            hops: 1,
            limit: 20,
        }
    }
}

/// Narrow-column predicates supported by the Phase-1 column-mask operator.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnMaskPredicate {
    /// Keep rows whose ids are present in the set.
    IdIn(BTreeSet<String>),
    /// Keep rows for a specific repository.
    RepoEquals(String),
    /// Keep rows whose path contains the provided fragment.
    PathContains(String),
    /// Keep rows at or above the provided score threshold.
    ScoreAtLeast(f64),
}

/// Typed request for narrow-column filtering over an existing relation.
#[derive(Debug, Clone)]
pub struct ColumnMaskOp {
    /// Input relation to filter.
    pub relation: WendaoRelation,
    /// Narrow predicates applied before payload hydration.
    pub predicates: Vec<ColumnMaskPredicate>,
    /// Optional output limit after filtering.
    pub limit: Option<usize>,
}

/// Typed request for payload hydration and projection.
#[derive(Debug, Clone)]
pub struct PayloadFetchOp {
    /// Input candidate relation.
    pub relation: WendaoRelation,
    /// Requested payload columns.
    pub columns: Vec<String>,
    /// Optional candidate id filter for hydration.
    pub ids: Option<BTreeSet<String>>,
}
