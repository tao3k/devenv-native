use crate::gateway::studio::types::AstSearchHit;
use crate::search::local_symbol::query::shared::{
    LocalSymbolSearchError, compare_candidates, decode_local_symbol_hits,
    execute_local_symbol_search, prepare_local_symbol_read_tables, retained_window,
};
use crate::search::ranking::sort_by_rank;
use crate::search::{SearchCorpusKind, SearchPlaneService};

pub(crate) async fn search_local_symbols(
    service: &SearchPlaneService,
    query: &str,
    limit: usize,
) -> Result<Vec<AstSearchHit>, LocalSymbolSearchError> {
    let query_lower = query.trim().to_ascii_lowercase();
    if query_lower.is_empty() {
        return Ok(Vec::new());
    }

    let prepared = prepare_local_symbol_read_tables(service).await?;
    if prepared.table_names.is_empty() {
        return Ok(Vec::new());
    }

    let execution = execute_local_symbol_search(
        &prepared.query_engine,
        prepared.table_names.as_slice(),
        query_lower.as_str(),
        retained_window(limit),
    )
    .await?;
    let mut candidates = execution.candidates;
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);
    let hits = decode_local_symbol_hits(&prepared.query_engine, candidates).await?;
    service.record_query_telemetry(
        SearchCorpusKind::LocalSymbol,
        execution
            .telemetry
            .finish(execution.source, Some("search".to_string()), hits.len()),
    );
    Ok(hits)
}
