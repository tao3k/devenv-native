//! Stable `WendaoGraph.jl` algorithm catalog facts.

use xiuxian_polyglot_orchestrator::julia_runtime::facts;

pub use xiuxian_polyglot_orchestrator::julia_runtime::{
    WendaoGraphAlgorithmComplexity, WendaoGraphAlgorithmRef,
};
use xiuxian_polyglot_orchestrator::julia_runtime::{WendaoGraphAlgorithmId, WendaoGraphProfileId};

/// Returns the `WendaoGraph.jl` `LinkGraph` algorithm catalog.
#[must_use]
pub const fn wendaograph_link_graph_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    facts::wendaograph_fact_link_graph_algorithm_refs()
}

/// Returns the `WendaoGraph.jl` relationship-search algorithm catalog.
#[must_use]
pub const fn wendaograph_relationship_search_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef]
{
    facts::wendaograph_fact_relationship_search_algorithm_refs()
}

/// Returns the `WendaoGraph.jl` `PageIndex` algorithm catalog.
#[must_use]
pub const fn wendaograph_page_index_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    facts::wendaograph_fact_page_index_algorithm_refs()
}

/// Returns the `WendaoGraph.jl` `SearchStrategyFlow` algorithm catalog.
#[must_use]
pub const fn wendaograph_search_strategy_flow_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef]
{
    facts::wendaograph_fact_search_strategy_flow_algorithm_refs()
}

/// Returns the `WendaoGraph.jl` GNN algorithm catalog.
#[must_use]
pub const fn wendaograph_gnn_algorithm_refs() -> &'static [WendaoGraphAlgorithmRef] {
    facts::wendaograph_fact_gnn_algorithm_refs()
}

/// Returns all staged `WendaoGraph.jl` algorithm catalog entries.
#[must_use]
pub fn wendaograph_algorithm_refs() -> Vec<WendaoGraphAlgorithmRef> {
    facts::wendaograph_fact_algorithm_refs()
}

/// Finds one staged `WendaoGraph.jl` algorithm catalog entry by id.
#[must_use]
pub fn wendaograph_algorithm_ref(
    algorithm_id: WendaoGraphAlgorithmId,
) -> Option<WendaoGraphAlgorithmRef> {
    facts::wendaograph_fact_algorithm_ref(algorithm_id)
}

/// Returns the staged `WendaoGraph.jl` algorithm that owns one reasoning-tree
/// backend frontier evidence kind.
#[must_use]
pub fn wendaograph_frontier_algorithm_ref(evidence_kind: &str) -> Option<WendaoGraphAlgorithmRef> {
    facts::wendaograph_fact_frontier_algorithm_ref(evidence_kind)
}

/// Returns staged `WendaoGraph.jl` algorithm entries for one Julia profile id.
#[must_use]
pub fn wendaograph_algorithm_refs_for_profile(
    profile_id: WendaoGraphProfileId,
) -> Vec<WendaoGraphAlgorithmRef> {
    facts::wendaograph_fact_algorithm_refs_for_profile(profile_id)
}
