use std::collections::HashSet;

use crate::analyzers::{ImportSearchQuery, ImportSearchResult};
use crate::search::SearchCorpusKind;
use crate::search::SearchPlaneService;
use crate::search::repo_entity::query::hydrate::build_import_search_result;
use crate::search::repo_entity::query::lookup::{
    PreparedRepoEntitySearch, RepoEntitySearchError, fixed_kind_filters, prepare_repo_entity_search,
};
use crate::search::repo_entity::query::results::shared::load_hydrated_rows;

pub(crate) async fn search_repo_entity_import_results(
    service: &SearchPlaneService,
    query: &ImportSearchQuery,
) -> Result<ImportSearchResult, RepoEntitySearchError> {
    let language_filters = HashSet::new();
    let kind_filters = fixed_kind_filters("import");
    let canonical_query = crate::analyzers::canonical_import_query_text(query);
    let Some(prepared) = prepare_repo_entity_search(
        service,
        query.repo_id.as_str(),
        canonical_query.as_str(),
        &language_filters,
        &kind_filters,
        query.limit,
    )
    .await?
    else {
        return Ok(empty_import_search_result(query.repo_id.as_str()));
    };
    let PreparedRepoEntitySearch {
        _read_permit,
        query_engine,
        engine_table_name,
        candidates,
        telemetry,
        source,
    } = prepared;
    let rows = load_hydrated_rows(
        &query_engine,
        engine_table_name.as_str(),
        candidates.as_slice(),
    )
    .await?;
    let result = build_import_search_result(query.repo_id.as_str(), candidates, &rows)?;
    service.record_query_telemetry(
        SearchCorpusKind::RepoEntity,
        telemetry.finish(
            source,
            Some(query.repo_id.clone()),
            result.import_hits.len(),
        ),
    );
    Ok(result)
}

fn empty_import_search_result(repo_id: &str) -> ImportSearchResult {
    ImportSearchResult {
        repo_id: repo_id.to_string(),
        imports: Vec::new(),
        import_hits: Vec::new(),
    }
}
