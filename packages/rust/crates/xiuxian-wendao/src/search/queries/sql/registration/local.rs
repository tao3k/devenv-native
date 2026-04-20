use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::search::{SearchCorpusKind, SearchPlaneService};
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use super::{RegisteredSqlTable, naming};

pub(super) fn collect_local_tables(
    service: &SearchPlaneService,
    tables: &mut BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &mut BTreeMap<String, PathBuf>,
) {
    for corpus in SearchCorpusKind::ALL
        .into_iter()
        .filter(|corpus| !corpus.is_repo_backed())
    {
        let status = service.coordinator().status_for(corpus);
        let Some(active_epoch) = status.active_epoch else {
            continue;
        };

        if corpus == SearchCorpusKind::LocalSymbol {
            for table_name in service.local_epoch_table_names_for_reads(corpus, active_epoch) {
                let parquet_path = service.local_table_parquet_path(corpus, table_name.as_str());
                if !parquet_path.exists() {
                    continue;
                }
                parquet_paths.insert(table_name.clone(), parquet_path);
                tables.insert(
                    table_name.clone(),
                    RegisteredSqlTable::local(corpus, table_name.clone(), table_name),
                );
            }
            continue;
        }

        let parquet_path = service.local_epoch_parquet_path(corpus, active_epoch);
        if !parquet_path.exists() {
            continue;
        }

        let engine_table_name =
            SearchPlaneService::local_epoch_engine_table_name(corpus, active_epoch);
        let sql_table_name = naming::local_sql_table_name(corpus, engine_table_name.as_str());
        parquet_paths.insert(sql_table_name.clone(), parquet_path);
        tables.insert(
            sql_table_name.clone(),
            RegisteredSqlTable::local(corpus, sql_table_name, engine_table_name),
        );
    }
}

#[cfg(not(feature = "duckdb"))]
pub(super) async fn register_local_tables(
    query_engine: &SearchEngineContext,
    tables: &BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &BTreeMap<String, PathBuf>,
) -> Result<(), String> {
    for table in tables
        .values()
        .filter(|table| table.scope == "local" && table.sql_object_kind == "table")
    {
        let parquet_path = parquet_paths
            .get(table.sql_table_name.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "local SQL surface should carry parquet path for `{}`",
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
