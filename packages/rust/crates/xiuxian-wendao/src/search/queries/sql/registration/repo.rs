use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::search::SearchPlaneService;
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use super::{RegisteredSqlTable, naming};

pub(super) async fn collect_repo_tables(
    service: &SearchPlaneService,
    tables: &mut BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &mut BTreeMap<String, PathBuf>,
) {
    let repo_records = service.repo_corpus_snapshot_for_reads().await;
    for ((corpus, repo_id), record) in repo_records {
        let Some(publication) = record.publication else {
            continue;
        };
        if !publication.is_parquet_query_readable() {
            continue;
        }

        let parquet_path =
            service.repo_publication_parquet_path(corpus, publication.table_name.as_str());
        if !parquet_path.exists() {
            continue;
        }

        let engine_table_name = SearchPlaneService::repo_publication_engine_table_name(
            corpus,
            publication.publication_id.as_str(),
        );
        let sql_table_name = naming::repo_sql_table_name(corpus, repo_id.as_str());
        parquet_paths.insert(sql_table_name.clone(), parquet_path);
        tables.insert(
            sql_table_name.clone(),
            RegisteredSqlTable::repo(corpus, repo_id.as_str(), sql_table_name, engine_table_name),
        );
    }
}

#[cfg(not(feature = "duckdb"))]
pub(super) async fn register_repo_tables(
    query_engine: &SearchEngineContext,
    tables: &BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    for table in tables
        .values()
        .filter(|table| table.scope == "repo" && table.sql_object_kind == "table")
    {
        let parquet_path = parquet_paths
            .get(table.sql_table_name.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "repo SQL surface should carry parquet path for `{}`",
                    table.sql_table_name
                )
            });
        query_engine
            .ensure_parquet_table_registered(
                table.sql_table_name.as_str(),
                parquet_path.as_path(),
                &[],
            )
            .await
            .map_err(|error| {
                format!(
                    "studio SQL Flight provider failed to register `{}` for corpus `{}`: {error}",
                    table.sql_table_name, table.corpus
                )
            })?;
    }
    Ok(())
}
