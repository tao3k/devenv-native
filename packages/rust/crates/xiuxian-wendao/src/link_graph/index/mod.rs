//! Core index build + query algorithms for markdown link graph.

pub(in crate::link_graph::index) use super::models::{
    LinkGraphDirection, LinkGraphDocument, LinkGraphEdgeType, LinkGraphHit, LinkGraphLinkFilter,
    LinkGraphMatchStrategy, LinkGraphMetadata, LinkGraphNeighbor, LinkGraphPprSubgraphMode,
    LinkGraphRelatedFilter, LinkGraphRelatedPprDiagnostics, LinkGraphRelatedPprOptions,
    LinkGraphScope, LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSortField,
    LinkGraphSortOrder, LinkGraphSortTerm, LinkGraphStats, PageIndexNode,
};

#[path = "agentic_expansion/mod.rs"]
mod agentic_expansion;
#[path = "agentic_overlay.rs"]
mod agentic_overlay;
#[path = "build/mod.rs"]
mod build;
#[path = "constants.rs"]
mod constants;
#[path = "ids.rs"]
mod ids;
#[path = "lookup.rs"]
mod lookup;
#[path = "page_indices.rs"]
mod page_indices;
#[path = "passages.rs"]
mod passages;
#[path = "pattern_symbols.rs"]
mod pattern_symbols;
#[path = "ppr/mod.rs"]
mod ppr;
#[path = "rank.rs"]
mod rank;
#[path = "scoring/mod.rs"]
mod scoring;
#[path = "search/mod.rs"]
pub(crate) mod search;
#[path = "semantic_documents.rs"]
mod semantic_documents;
#[path = "shared.rs"]
mod shared;
#[path = "symbol_cache.rs"]
mod symbol_cache;
#[path = "traversal/mod.rs"]
mod traversal;
#[path = "types.rs"]
mod types;

#[cfg(feature = "vector-store")]
pub use search::quantum_fusion::orchestrate::QuantumContextBuildError;
#[cfg(feature = "vector-store")]
pub use search::quantum_fusion::semantic_ignition::{
    QuantumSemanticIgnition, QuantumSemanticIgnitionError, QuantumSemanticIgnitionFuture,
};
pub use xiuxian_wendao_core::LinkGraphRefreshMode;

pub(in crate::link_graph::index) use constants::{
    DEFAULT_MIN_SECTION_WORDS, DEFAULT_PER_DOC_SECTION_CAP, INCOMING_RANK_FACTOR,
    INCREMENTAL_REBUILD_THRESHOLD, MAX_GRAPH_RANK_BOOST, OUTGOING_RANK_FACTOR,
    SECTION_AGGREGATION_BETA, WEIGHT_FTS_LEXICAL, WEIGHT_FTS_PATH, WEIGHT_FTS_SECTION,
    WEIGHT_PATH_FUZZY_PATH, WEIGHT_PATH_FUZZY_SECTION,
};
pub(in crate::link_graph::index) use scoring::{
    normalize_with_case, score_document, score_document_exact, score_document_regex,
    score_path_fields, section_tree_distance, token_match_ratio, tokenize,
};
pub(in crate::link_graph::index) use shared::{
    ScoredSearchRow, deterministic_random_key, doc_contains_phrase, doc_sort_key,
    normalize_path_filter, path_matches_filter, sort_hits,
};
pub(crate) use types::{IndexedSection, SectionCandidate, SectionMatch};
pub use types::{
    LinkGraphCacheBuildMeta, LinkGraphIndex, LinkGraphVirtualNode, PageIndexParent,
    SymbolCacheStats, SymbolRef,
};

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/index.rs"]
mod tests;
