use crate::studio::types::{SearchHit, SourceSymbolHit};
#[cfg(all(test, feature = "duckdb"))]
use xiuxian_wendao::duckdb::ParquetQueryEngine;
#[cfg(test)]
use xiuxian_wendao::search::SearchPlaneService;

#[derive(Debug, Clone, Default)]
pub(crate) struct IntentSearchTransportMetadata {
    #[cfg(test)]
    pub(crate) knowledge_query_engine: Option<&'static str>,
    #[cfg(test)]
    pub(crate) local_symbol_query_engine: Option<&'static str>,
    #[cfg(all(test, feature = "duckdb"))]
    pub(crate) repo_query_engine: Option<&'static str>,
    #[cfg(test)]
    pub(crate) repo_content_transport: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntentIndexState {
    pub(crate) knowledge_config_missing: bool,
    pub(crate) symbol_config_missing: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct IntentSourceHits {
    pub(crate) knowledge_hits: Vec<SearchHit>,
    pub(crate) local_symbol_hits: Vec<SourceSymbolHit>,
    pub(crate) knowledge_indexing: bool,
    pub(crate) local_symbol_indexing: bool,
    #[cfg(test)]
    pub(crate) knowledge_query_engine: Option<&'static str>,
    #[cfg(test)]
    pub(crate) local_symbol_query_engine: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntentMergedResults {
    pub(crate) hits: Vec<SearchHit>,
    pub(crate) knowledge_hit_count: usize,
    pub(crate) local_symbol_hit_count: usize,
    pub(crate) repo_hit_count: usize,
    pub(crate) transport: IntentSearchTransportMetadata,
    pub(crate) partial: bool,
    pub(crate) pending_repos: Vec<String>,
    pub(crate) skipped_repos: Vec<String>,
}

#[cfg(all(test, feature = "duckdb"))]
pub(crate) fn configured_parquet_query_engine_label(
    _service: &SearchPlaneService,
) -> Result<&'static str, String> {
    let query_engine = ParquetQueryEngine::configured().map_err(|error| error.to_string())?;
    Ok(query_engine.kind().as_str())
}

#[cfg(all(test, not(feature = "duckdb")))]
pub(crate) fn configured_parquet_query_engine_label(_service: &SearchPlaneService) -> &'static str {
    "datafusion"
}
