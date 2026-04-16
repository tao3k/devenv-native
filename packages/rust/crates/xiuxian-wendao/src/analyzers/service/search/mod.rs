//! Repository search functions (overview, module, symbol, example, import, doc coverage).
#[cfg(feature = "studio")]
mod artifacts;
mod contracts;
mod coverage;
mod documents;
mod example;
mod imports;
#[cfg(feature = "repo-lexical-index")]
mod indexed_exact;
#[cfg(feature = "repo-lexical-index")]
mod indexed_fuzzy;
mod legacy;
mod module;
mod overview;
mod ranking;
mod symbol;

#[cfg(test)]
#[path = "../../../../tests/unit/analyzers/service/search/mod.rs"]
mod tests;

pub use coverage::{
    build_doc_coverage, doc_coverage_from_config, doc_coverage_from_config_with_registry,
};
pub use example::{
    build_example_search, example_search_from_config, example_search_from_config_with_registry,
};
pub use imports::{
    build_import_search, import_search_from_config, import_search_from_config_with_registry,
};
pub use module::{
    build_module_search, module_search_from_config, module_search_from_config_with_registry,
};
pub use overview::{
    build_repo_overview, repo_overview_from_config, repo_overview_from_config_with_registry,
};
pub use symbol::{
    build_symbol_search, symbol_search_from_config, symbol_search_from_config_with_registry,
};

#[cfg(feature = "studio")]
pub(crate) use artifacts::repository_search_artifacts;
#[cfg(feature = "search-runtime")]
pub(crate) use contracts::canonical_import_query_text;
#[cfg(feature = "studio")]
pub(crate) use contracts::{
    RepoAnalysisFallbackContract, example_fallback_contract, import_fallback_contract,
    module_fallback_contract, symbol_fallback_contract,
};
#[cfg(feature = "studio")]
pub(crate) use documents::ExampleSearchMetadata;
#[cfg(all(feature = "studio", feature = "repo-lexical-index"))]
pub(crate) use example::build_example_search_with_artifacts;
#[cfg(feature = "studio")]
pub(crate) use imports::build_import_search_with_artifacts;
#[cfg(all(feature = "studio", feature = "repo-lexical-index"))]
pub(crate) use module::build_module_search_with_artifacts;
#[cfg(all(feature = "studio", feature = "repo-lexical-index"))]
pub(crate) use symbol::build_symbol_search_with_artifacts;
