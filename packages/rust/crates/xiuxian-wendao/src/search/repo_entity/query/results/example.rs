use std::collections::HashSet;

use crate::analyzers::ExampleSearchResult;
use crate::search::SearchCorpusKind;
use crate::search::SearchPlaneService;
use crate::search::repo_entity::query::hydrate::build_example_search_result;
use crate::search::repo_entity::query::lookup::{
    PreparedRepoEntitySearch, RepoEntitySearchError, fixed_kind_filters, prepare_repo_entity_search,
};
use crate::search::repo_entity::query::results::shared::load_hydrated_rows;

pub(crate) async fn search_repo_entity_example_results(
    service: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    limit: usize,
) -> Result<ExampleSearchResult, RepoEntitySearchError> {
    let language_filters = HashSet::new();
    let kind_filters = fixed_kind_filters("example");
    let Some(prepared) = prepare_repo_entity_search(
        service,
        repo_id,
        query,
        &language_filters,
        &kind_filters,
        limit,
    )
    .await?
    else {
        return Ok(empty_example_search_result(repo_id));
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
    let result = build_example_search_result(repo_id, candidates, &rows)?;
    service.record_query_telemetry(
        SearchCorpusKind::RepoEntity,
        telemetry.finish(source, Some(repo_id.to_string()), result.example_hits.len()),
    );
    Ok(result)
}

fn empty_example_search_result(repo_id: &str) -> ExampleSearchResult {
    ExampleSearchResult {
        repo_id: repo_id.to_string(),
        examples: Vec::new(),
        example_hits: Vec::new(),
    }
}
