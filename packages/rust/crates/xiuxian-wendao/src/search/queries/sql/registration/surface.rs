use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::PathBuf;

use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::catalog::{columns_catalog_schema, tables_catalog_schema, view_sources_catalog_schema};
use super::{
    RegisteredSqlColumn, RegisteredSqlTable, RegisteredSqlViewSource,
    STUDIO_SQL_CATALOG_TABLE_NAME, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
    STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME, SqlQuerySurface, local, repo, views,
};

pub(crate) struct SqlSurfaceAssembly {
    pub(crate) surface: SqlQuerySurface,
    pub(crate) parquet_paths: BTreeMap<String, PathBuf>,
}

pub(crate) async fn build_sql_query_surface(
    service: &SearchPlaneService,
) -> Result<SqlQuerySurface, String> {
    build_sql_surface_assembly(service)
        .await
        .map(|assembly| assembly.surface)
}

pub(crate) async fn build_sql_surface_assembly(
    service: &SearchPlaneService,
) -> Result<SqlSurfaceAssembly, String> {
    let mut tables = BTreeMap::<String, RegisteredSqlTable>::new();
    let mut parquet_paths = BTreeMap::<String, PathBuf>::new();
    local::collect_local_tables(service, &mut tables, &mut parquet_paths);
    let mut view_sources = views::collect_local_logical_views(&mut tables);
    repo::collect_repo_tables(service, &mut tables, &mut parquet_paths).await;
    view_sources.extend(views::collect_repo_logical_views(&mut tables));
    tables.insert(
        STUDIO_SQL_CATALOG_TABLE_NAME.to_string(),
        RegisteredSqlTable::system(STUDIO_SQL_CATALOG_TABLE_NAME),
    );
    tables.insert(
        STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME.to_string(),
        RegisteredSqlTable::system(STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME),
    );
    tables.insert(
        STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME.to_string(),
        RegisteredSqlTable::system(STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME),
    );

    let tables = tables.into_values().collect::<Vec<_>>();
    let columns = collect_registered_columns_from_surface(
        tables.as_slice(),
        view_sources.as_slice(),
        &parquet_paths,
    )?;
    Ok(SqlSurfaceAssembly {
        surface: SqlQuerySurface::new(
            STUDIO_SQL_CATALOG_TABLE_NAME,
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
            tables,
            columns,
            view_sources,
        ),
        parquet_paths,
    })
}

fn collect_registered_columns_from_surface(
    tables: &[RegisteredSqlTable],
    view_sources: &[RegisteredSqlViewSource],
    parquet_paths: &BTreeMap<String, PathBuf>,
) -> Result<Vec<RegisteredSqlColumn>, String> {
    let mut columns_by_table = BTreeMap::<String, Vec<RegisteredSqlColumn>>::new();
    for table in tables
        .iter()
        .filter(|table| table.sql_object_kind != "view")
    {
        let columns = match table.sql_table_name.as_str() {
            STUDIO_SQL_CATALOG_TABLE_NAME => {
                columns_from_schema(table, tables_catalog_schema().as_ref())
            }
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME => {
                columns_from_schema(table, columns_catalog_schema().as_ref())
            }
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME => {
                columns_from_schema(table, view_sources_catalog_schema().as_ref())
            }
            _ => {
                let parquet_path = parquet_paths
                    .get(table.sql_table_name.as_str())
                    .unwrap_or_else(|| {
                        panic!(
                            "SQL surface should carry parquet path for `{}`",
                            table.sql_table_name
                        )
                    });
                columns_from_parquet(table, parquet_path)?
            }
        };
        columns_by_table.insert(table.sql_table_name.clone(), columns);
    }

    for table in tables
        .iter()
        .filter(|table| table.sql_object_kind == "view")
    {
        let columns = columns_for_logical_view(table, view_sources, &columns_by_table)?;
        columns_by_table.insert(table.sql_table_name.clone(), columns);
    }

    let mut columns = Vec::new();
    for table in tables {
        columns.extend(
            columns_by_table
                .get(table.sql_table_name.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "SQL surface should carry registered columns for `{}`",
                        table.sql_table_name
                    )
                })
                .clone(),
        );
    }
    Ok(columns)
}

fn columns_from_schema(table: &RegisteredSqlTable, schema: &Schema) -> Vec<RegisteredSqlColumn> {
    schema
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| RegisteredSqlColumn::from_arrow_field(table, index + 1, field))
        .collect()
}

fn columns_from_parquet(
    table: &RegisteredSqlTable,
    parquet_path: &PathBuf,
) -> Result<Vec<RegisteredSqlColumn>, String> {
    let schema_path = parquet_schema_source_path(table, parquet_path)?;
    let parquet_file = File::open(schema_path.as_path()).map_err(|error| {
        format!(
            "studio SQL surface failed to open parquet schema for `{}` at `{}`: {error}",
            table.sql_table_name,
            schema_path.display()
        )
    })?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(parquet_file).map_err(|error| {
        format!(
            "studio SQL surface failed to read parquet schema for `{}` at `{}`: {error}",
            table.sql_table_name,
            schema_path.display()
        )
    })?;
    Ok(columns_from_schema(table, builder.schema().as_ref()))
}

fn parquet_schema_source_path(
    table: &RegisteredSqlTable,
    parquet_path: &PathBuf,
) -> Result<PathBuf, String> {
    if parquet_path.is_file() {
        return Ok(parquet_path.clone());
    }
    if !parquet_path.is_dir() {
        return Err(format!(
            "studio SQL surface expected parquet file or directory for `{}` at `{}`",
            table.sql_table_name,
            parquet_path.display()
        ));
    }

    let mut partition_paths = fs::read_dir(parquet_path)
        .map_err(|error| {
            format!(
                "studio SQL surface failed to inspect parquet directory for `{}` at `{}`: {error}",
                table.sql_table_name,
                parquet_path.display()
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_parquet_file = entry
                .file_type()
                .ok()
                .is_some_and(|file_type| file_type.is_file())
                && path
                    .extension()
                    .is_some_and(|extension| extension == "parquet");
            is_parquet_file.then_some(path)
        })
        .collect::<Vec<_>>();
    partition_paths.sort();
    partition_paths.into_iter().next().ok_or_else(|| {
        format!(
            "studio SQL surface found no parquet partitions for `{}` at `{}`",
            table.sql_table_name,
            parquet_path.display()
        )
    })
}

fn columns_for_logical_view(
    table: &RegisteredSqlTable,
    view_sources: &[RegisteredSqlViewSource],
    columns_by_table: &BTreeMap<String, Vec<RegisteredSqlColumn>>,
) -> Result<Vec<RegisteredSqlColumn>, String> {
    let mut source_tables = view_sources
        .iter()
        .filter(|view_source| view_source.sql_view_name == table.sql_table_name)
        .collect::<Vec<_>>();
    source_tables.sort_by_key(|view_source| view_source.source_ordinal);
    let Some(first_source) = source_tables.first() else {
        return Err(format!(
            "studio SQL surface missing logical-view sources for `{}`",
            table.sql_table_name
        ));
    };
    let source_columns = columns_by_table
        .get(first_source.source_sql_table_name.as_str())
        .ok_or_else(|| {
            format!(
                "studio SQL surface missing source columns for logical view `{}`",
                table.sql_table_name
            )
        })?;
    let fields = logical_view_fields(table, source_columns)?;
    Ok(fields
        .iter()
        .enumerate()
        .map(|(index, field)| RegisteredSqlColumn::from_arrow_field(table, index + 1, field))
        .collect())
}

fn logical_view_fields(
    table: &RegisteredSqlTable,
    source_columns: &[RegisteredSqlColumn],
) -> Result<Vec<Field>, String> {
    let projected_fields = source_columns
        .iter()
        .map(|column| column.arrow_field.clone())
        .collect::<Vec<_>>();
    match (
        table.scope.as_str(),
        table.corpus.as_str(),
        table.sql_table_name.as_str(),
    ) {
        ("local_logical", _, _) => Ok(projected_fields),
        ("repo_logical", corpus, _) if corpus == SearchCorpusKind::RepoEntity.to_string() => {
            let mut fields = vec![Field::new("repo_id", DataType::Utf8, false)];
            fields.extend(projected_fields);
            Ok(fields)
        }
        ("repo_logical", corpus, _) if corpus == SearchCorpusKind::RepoContentChunk.to_string() => {
            let mut fields = vec![
                Field::new("repo_id", DataType::Utf8, false),
                Field::new("title", DataType::Utf8, false),
                Field::new("doc_type", DataType::Utf8, false),
                Field::new("code_tag", DataType::Utf8, false),
                Field::new("file_tag", DataType::Utf8, false),
                Field::new("kind_tag", DataType::Utf8, false),
                Field::new("language_tag", DataType::Utf8, true),
            ];
            fields.extend(projected_fields);
            Ok(fields)
        }
        _ => Err(format!(
            "studio SQL surface cannot derive logical-view schema for `{}`",
            table.sql_table_name
        )),
    }
}
