use anyhow::Result;
use xiuxian_db_store::duckdb::{
    DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, open_duckdb_connection,
};

use crate::orgize::read_model::model::ResolvedReadModelSettings;

pub(in crate::orgize::read_model) fn open_read_model_connection(
    settings: &ResolvedReadModelSettings,
) -> Result<xiuxian_db_store::duckdb_crate::Connection> {
    let runtime = DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::File(settings.database_path.clone()),
        temp_directory: settings.temp_directory.clone(),
        threads: settings.threads,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: true,
            prefer_virtual_arrow: true,
        },
        memory_limit: settings.memory_limit.clone(),
        max_temp_directory_size: settings.max_temp_directory_size.clone(),
        materialize_threshold_rows: settings.materialize_threshold_rows,
    };
    open_duckdb_connection(&runtime).map_err(anyhow::Error::msg)
}
