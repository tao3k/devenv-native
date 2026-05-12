//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::duckdb::ParquetQueryEngine;
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::{LocalSymbolSearchError, PreparedLocalSymbolRead};

pub(crate) async fn prepare_local_symbol_read_tables(
    service: &SearchPlaneService,
) -> Result<PreparedLocalSymbolRead, LocalSymbolSearchError> {
    let status = service
        .coordinator()
        .status_for(SearchCorpusKind::LocalSymbol);
    let Some(active_epoch) = status.active_epoch else {
        return Err(LocalSymbolSearchError::NotReady);
    };

    let table_names =
        service.local_epoch_table_names_for_reads(SearchCorpusKind::LocalSymbol, active_epoch);
    if table_names.is_empty() {
        #[cfg(feature = "duckdb")]
        let query_engine = ParquetQueryEngine::configured()?;
        #[cfg(not(feature = "duckdb"))]
        let query_engine =
            ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
        return Ok(PreparedLocalSymbolRead {
            query_engine,
            table_names,
        });
    }
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    for table_name in &table_names {
        let parquet_path =
            service.local_table_parquet_path(SearchCorpusKind::LocalSymbol, table_name.as_str());
        if !parquet_path.exists() {
            return Err(LocalSymbolSearchError::NotReady);
        }
        query_engine
            .ensure_parquet_table_registered(table_name.as_str(), parquet_path.as_path())
            .await?;
    }

    Ok(PreparedLocalSymbolRead {
        query_engine,
        table_names,
    })
}
