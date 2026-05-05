#[path = "hydrate/mod.rs"]
mod hydrate;
#[path = "lookup/mod.rs"]
mod lookup;
#[path = "results/mod.rs"]
mod results;

#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_entity/query/mod.rs"]
mod tests;

pub use lookup::RepoEntitySearchError;
pub(crate) use lookup::search_repo_entities;
pub use results::{RepoEntityOverviewSummary, summarize_repo_entity_overview};
pub(crate) use results::{
    search_repo_entity_example_results, search_repo_entity_import_results,
    search_repo_entity_module_results, search_repo_entity_symbol_results,
};
