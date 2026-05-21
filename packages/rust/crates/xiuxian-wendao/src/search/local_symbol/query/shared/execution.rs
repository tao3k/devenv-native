//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::duckdb::ParquetQueryEngine;
use crate::search::ranking::{RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry};

use super::columns::{collect_candidates, collect_suggestions};
use super::types::{
    LocalSymbolAutocompleteExecution, LocalSymbolSearchError, LocalSymbolSearchExecution,
};

pub(crate) async fn execute_local_symbol_search(
    engine: &ParquetQueryEngine,
    table_names: &[String],
    query_lower: &str,
    window: RetainedWindow,
) -> Result<LocalSymbolSearchExecution, LocalSymbolSearchError> {
    let mut telemetry = StreamingRerankTelemetry::new(window, None, None);
    let mut candidates = Vec::with_capacity(window.target);
    for table_name in table_names {
        let sql = format!(
            "SELECT {} FROM {table_name}",
            crate::search::local_symbol::schema::projected_columns().join(", "),
        );
        let batches = engine.query_batches(sql.as_str()).await?;
        for batch in batches {
            collect_candidates(
                table_name.as_str(),
                &batch,
                query_lower,
                &mut candidates,
                window,
                &mut telemetry,
            )?;
        }
    }
    Ok(LocalSymbolSearchExecution {
        candidates,
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

pub(crate) async fn execute_local_symbol_autocomplete(
    engine: &ParquetQueryEngine,
    table_names: &[String],
    normalized_prefix: &str,
    window: RetainedWindow,
) -> Result<LocalSymbolAutocompleteExecution, LocalSymbolSearchError> {
    let mut telemetry = StreamingRerankTelemetry::new(window, None, None);
    let mut suggestions = Vec::with_capacity(window.target);
    let mut seen = std::collections::HashSet::new();
    for table_name in table_names {
        let sql = format!(
            "SELECT {} FROM {table_name}",
            crate::search::local_symbol::schema::suggestion_columns().join(", "),
        );
        let batches = engine.query_batches(sql.as_str()).await?;
        for batch in batches {
            collect_suggestions(
                &batch,
                normalized_prefix,
                &mut suggestions,
                &mut seen,
                window,
                &mut telemetry,
            )?;
        }
    }
    Ok(LocalSymbolAutocompleteExecution {
        suggestions,
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}
