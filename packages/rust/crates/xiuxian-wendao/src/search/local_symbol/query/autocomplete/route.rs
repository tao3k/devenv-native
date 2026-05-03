use crate::search::contracts::AutocompleteSuggestion;
use crate::search::local_symbol::query::shared::{
    LocalSymbolSearchError, compare_suggestions, execute_local_symbol_autocomplete,
    prepare_local_symbol_read_tables, suggestion_window,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};

pub(crate) async fn autocomplete_local_symbols(
    service: &SearchPlaneService,
    prefix: &str,
    limit: usize,
) -> Result<Vec<AutocompleteSuggestion>, LocalSymbolSearchError> {
    let normalized_prefix = prefix.trim().to_ascii_lowercase();
    if normalized_prefix.is_empty() {
        return Ok(Vec::new());
    }

    let prepared = prepare_local_symbol_read_tables(service).await?;
    if prepared.table_names.is_empty() {
        return Ok(Vec::new());
    }

    let execution = execute_local_symbol_autocomplete(
        &prepared.query_engine,
        prepared.table_names.as_slice(),
        normalized_prefix.as_str(),
        suggestion_window(limit),
    )
    .await?;
    let mut suggestions = execution.suggestions;
    suggestions.sort_by(compare_suggestions);
    suggestions.truncate(limit);
    service.record_query_telemetry(
        SearchCorpusKind::LocalSymbol,
        execution.telemetry.finish(
            execution.source,
            Some("autocomplete".to_string()),
            suggestions.len(),
        ),
    );
    Ok(suggestions)
}
