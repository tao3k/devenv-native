#[path = "hydrate/mod.rs"]
mod hydrate;
#[path = "lookup/mod.rs"]
mod lookup;
#[path = "results/mod.rs"]
mod results;

#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_entity/query/mod.rs"]
mod tests;

pub(crate) use lookup::{RepoEntitySearchError, search_repo_entities};
pub(crate) use results::{
    RepoEntityOverviewSummary, search_repo_entity_example_results,
    search_repo_entity_import_results, search_repo_entity_module_results,
    search_repo_entity_symbol_results, summarize_repo_entity_overview,
};
