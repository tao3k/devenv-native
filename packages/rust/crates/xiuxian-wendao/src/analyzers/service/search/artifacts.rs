//! `analyzers::service::search::artifacts` owns Wendao service search artifacts behavior.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::cache::{
    RepositoryAnalysisCacheKey, RepositorySearchArtifacts, load_cached_repository_search_artifacts,
    store_cached_repository_search_artifacts,
};
use crate::analyzers::projection::{build_projected_page_search_index, build_projected_pages};
use crate::search::SearchDocumentIndex;

use super::documents::{
    build_example_metadata_lookup, build_search_document_index, example_search_document,
    module_search_document, symbol_search_document,
};

/// Return cached or freshly built repository search artifacts for one analysis.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when cached artifacts cannot be loaded or
/// freshly generated artifacts cannot be stored.
pub fn repository_search_artifacts(
    cache_key: &RepositoryAnalysisCacheKey,
    analysis: &RepositoryAnalysisOutput,
) -> Result<Arc<RepositorySearchArtifacts>, RepoIntelligenceError> {
    if let Some(cached) = load_cached_repository_search_artifacts(cache_key)? {
        return Ok(cached);
    }

    let example_metadata = build_example_metadata_lookup(analysis);
    let projected_pages = build_projected_pages(analysis);
    let (projected_page_index, projected_pages_by_id) = if projected_pages.is_empty() {
        (SearchDocumentIndex::new(), HashMap::new())
    } else {
        build_projected_page_search_index(projected_pages.as_slice())
            .map_err(|message| RepoIntelligenceError::AnalysisFailed { message })?
    };

    store_cached_repository_search_artifacts(
        cache_key.clone(),
        RepositorySearchArtifacts {
            module_index: build_search_document_index(
                analysis.modules.iter().map(module_search_document),
            )
            .unwrap_or_else(SearchDocumentIndex::new),
            symbol_index: build_search_document_index(
                analysis.symbols.iter().map(symbol_search_document),
            )
            .unwrap_or_else(SearchDocumentIndex::new),
            example_index: build_search_document_index(analysis.examples.iter().map(|example| {
                let metadata = example_metadata
                    .get(example.example_id.as_str())
                    .cloned()
                    .unwrap_or_default();
                example_search_document(example, &metadata)
            }))
            .unwrap_or_else(SearchDocumentIndex::new),
            projected_page_index,
            modules_by_id: analysis
                .modules
                .iter()
                .map(|module| (module.module_id.to_string(), module.clone()))
                .collect::<BTreeMap<_, _>>(),
            symbols_by_id: analysis
                .symbols
                .iter()
                .map(|symbol| (symbol.symbol_id.to_string(), symbol.clone()))
                .collect::<BTreeMap<_, _>>(),
            examples_by_id: analysis
                .examples
                .iter()
                .map(|example| (example.example_id.to_string(), example.clone()))
                .collect::<BTreeMap<_, _>>(),
            example_metadata,
            projected_pages_by_id,
            projected_pages,
        },
    )
}
