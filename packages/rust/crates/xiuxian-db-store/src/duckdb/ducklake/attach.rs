//! Runtime attach helpers for embedded `DuckDB` `DuckLake` catalogs.

use std::fs;

use super::{DuckLakeAttachConfig, DuckLakeCatalog, build_ducklake_attach_sql};

/// Attach one DuckLake catalog to an existing embedded DuckDB connection.
///
/// # Errors
///
/// Returns an error when required local directories cannot be created, when
/// DuckLake SQL cannot be rendered, or when DuckDB rejects extension
/// installation, loading, or attachment.
pub fn attach_ducklake(
    connection: &::duckdb::Connection,
    config: &DuckLakeAttachConfig,
) -> Result<(), String> {
    prepare_ducklake_paths(config)?;
    let sql = build_ducklake_attach_sql(config)?;
    connection.execute_batch(sql.as_str()).map_err(|error| {
        format!(
            "failed to attach DuckLake catalog `{}` through DuckDB: {error}",
            config.alias
        )
    })
}

fn prepare_ducklake_paths(config: &DuckLakeAttachConfig) -> Result<(), String> {
    if let Some(data_path) = config.data_path.local_path() {
        fs::create_dir_all(data_path).map_err(|error| {
            format!(
                "failed to create DuckLake data directory `{}`: {error}",
                data_path.display()
            )
        })?;
    }

    if let DuckLakeCatalog::LocalMetadataFile(metadata_path) = &config.catalog {
        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create DuckLake metadata directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
    }
    Ok(())
}
