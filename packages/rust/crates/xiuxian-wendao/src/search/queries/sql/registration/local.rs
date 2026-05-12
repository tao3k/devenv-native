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
    SearchCorpusKind::ALL
        .into_iter()
        .filter(|corpus| !corpus.is_repo_backed())
        .for_each(|corpus| collect_local_corpus_tables(service, tables, parquet_paths, corpus));
}

fn collect_local_corpus_tables(
    service: &SearchPlaneService,
    tables: &mut BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &mut BTreeMap<String, PathBuf>,
    corpus: SearchCorpusKind,
) {
    let status = service.coordinator().status_for(corpus);
    let Some(active_epoch) = status.active_epoch else {
        return;
    };

    if corpus == SearchCorpusKind::LocalSymbol {
        collect_local_symbol_tables(service, tables, parquet_paths, corpus, active_epoch);
        return;
    }

    let parquet_path = service.local_epoch_parquet_path(corpus, active_epoch);
    if !parquet_path.exists() {
        return;
    }

    let engine_table_name = SearchPlaneService::local_epoch_engine_table_name(corpus, active_epoch);
    let sql_table_name = naming::local_sql_table_name(corpus, engine_table_name.as_str());
    parquet_paths.insert(sql_table_name.clone(), parquet_path);
    tables.insert(
        sql_table_name.clone(),
        RegisteredSqlTable::local(corpus, sql_table_name, engine_table_name),
    );
}

fn collect_local_symbol_tables(
    service: &SearchPlaneService,
    tables: &mut BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &mut BTreeMap<String, PathBuf>,
    corpus: SearchCorpusKind,
    active_epoch: u64,
) {
    service
        .local_epoch_table_names_for_reads(corpus, active_epoch)
        .into_iter()
        .for_each(|table_name| {
            let parquet_path = service.local_table_parquet_path(corpus, table_name.as_str());
            if !parquet_path.exists() {
                return;
            }
            parquet_paths.insert(table_name.clone(), parquet_path);
            tables.insert(
                table_name.clone(),
                RegisteredSqlTable::local(corpus, table_name.clone(), table_name),
            );
        });
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
