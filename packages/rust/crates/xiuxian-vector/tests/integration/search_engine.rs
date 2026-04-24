//! Integration coverage for the `DataFusion` search-engine foundation.

use std::sync::Arc;

use anyhow::Result;
use arrow::array::{ArrayRef, StringArray, StringViewArray};
use arrow::datatypes::DataType;
use xiuxian_vector::{
    EngineRecordBatch, LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
    SearchEngineContext, engine_batch_to_lance_batch, lance_batch_to_engine_batch,
    write_lance_batches_to_parquet_file,
};

fn local_symbol_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new_with_metadata(
        vec![
            LanceField::new("id", LanceDataType::Utf8, false),
            LanceField::new("name", LanceDataType::Utf8, false),
        ],
        std::collections::HashMap::from([("domain".to_string(), "local_symbol".to_string())]),
    ))
}

fn local_symbol_batch(rows: &[(&str, &str)], schema: Arc<LanceSchema>) -> Result<LanceRecordBatch> {
    let ids = rows.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let names = rows.iter().map(|(_, name)| *name).collect::<Vec<_>>();
    Ok(LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(names)),
        ],
    )?)
}

fn string_value_at(column: &ArrayRef, row: usize) -> String {
    match column.data_type() {
        DataType::Utf8 => match column.as_any().downcast_ref::<StringArray>() {
            Some(values) => values.value(row).to_string(),
            None => panic!("utf8 column should decode as StringArray"),
        },
        DataType::Utf8View => match column.as_any().downcast_ref::<StringViewArray>() {
            Some(values) => values.value(row).to_string(),
            None => panic!("utf8 view column should decode as StringViewArray"),
        },
        other => panic!("expected utf8-like column, got {other:?}"),
    }
}

fn rows_from_engine_batch(batch: &EngineRecordBatch) -> Vec<(String, String)> {
    (0..batch.num_rows())
        .map(|row| {
            (
                string_value_at(batch.column(0), row),
                string_value_at(batch.column(1), row),
            )
        })
        .collect()
}

fn rows_from_lance_batch(batch: &LanceRecordBatch) -> Vec<(String, String)> {
    let Some(ids) = batch.column(0).as_any().downcast_ref::<LanceStringArray>() else {
        panic!("id column should decode as LanceStringArray");
    };
    let Some(names) = batch.column(1).as_any().downcast_ref::<LanceStringArray>() else {
        panic!("name column should decode as LanceStringArray");
    };

    (0..batch.num_rows())
        .map(|row| (ids.value(row).to_string(), names.value(row).to_string()))
        .collect()
}

#[test]
fn lance_batches_round_trip_into_engine_batches() -> Result<()> {
    let schema = local_symbol_schema();
    let batch = local_symbol_batch(&[("sym-1", "AlphaSymbol"), ("sym-2", "BetaSymbol")], schema)?;

    let engine_batch = lance_batch_to_engine_batch(&batch)?;

    assert_eq!(
        rows_from_engine_batch(&engine_batch),
        vec![
            ("sym-1".to_string(), "AlphaSymbol".to_string()),
            ("sym-2".to_string(), "BetaSymbol".to_string()),
        ]
    );
    assert_eq!(
        engine_batch
            .schema()
            .metadata()
            .get("domain")
            .map(String::as_str),
        Some("local_symbol")
    );
    Ok(())
}

#[test]
fn engine_batches_round_trip_back_into_lance_batches() -> Result<()> {
    let schema = local_symbol_schema();
    let batch = local_symbol_batch(&[("sym-1", "AlphaSymbol"), ("sym-2", "BetaSymbol")], schema)?;
    let engine_batch = lance_batch_to_engine_batch(&batch)?;
    let lance_batch = engine_batch_to_lance_batch(&engine_batch)?;

    assert_eq!(
        rows_from_lance_batch(&lance_batch),
        vec![
            ("sym-1".to_string(), "AlphaSymbol".to_string()),
            ("sym-2".to_string(), "BetaSymbol".to_string()),
        ]
    );
    assert_eq!(
        lance_batch
            .schema()
            .metadata()
            .get("domain")
            .map(String::as_str),
        Some("local_symbol")
    );
    Ok(())
}

#[tokio::test]
async fn search_engine_registers_parquet_exports() -> Result<()> {
    let schema = local_symbol_schema();
    let batch = local_symbol_batch(&[("sym-1", "AlphaSymbol"), ("sym-2", "BetaSymbol")], schema)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("xiuxian_vector_search_engine_")
        .tempdir()?;
    let parquet_path = temp_dir.path().join("local_symbol.parquet");

    write_lance_batches_to_parquet_file(&parquet_path, &[batch])?;

    let engine = SearchEngineContext::new();
    engine
        .register_parquet_table("local_symbol", &parquet_path, &[])
        .await?;
    let batches = engine
        .sql_batches("SELECT id, name FROM local_symbol ORDER BY id")
        .await?;

    let rows = batches
        .iter()
        .flat_map(rows_from_engine_batch)
        .collect::<Vec<_>>();

    assert_eq!(
        rows,
        vec![
            ("sym-1".to_string(), "AlphaSymbol".to_string()),
            ("sym-2".to_string(), "BetaSymbol".to_string()),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn search_engine_ensure_registration_is_idempotent() -> Result<()> {
    let schema = local_symbol_schema();
    let batch = local_symbol_batch(&[("sym-1", "AlphaSymbol")], schema)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("xiuxian_vector_search_engine_ensure_")
        .tempdir()?;
    let parquet_path = temp_dir.path().join("local_symbol.parquet");

    write_lance_batches_to_parquet_file(&parquet_path, &[batch])?;

    let engine = SearchEngineContext::new();
    engine
        .ensure_parquet_table_registered("local_symbol", &parquet_path, &[])
        .await?;
    engine
        .ensure_parquet_table_registered("local_symbol", &parquet_path, &[])
        .await?;

    let batches = engine
        .sql_batches("SELECT id, name FROM local_symbol ORDER BY id")
        .await?;
    let rows = batches
        .iter()
        .flat_map(rows_from_engine_batch)
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![("sym-1".to_string(), "AlphaSymbol".to_string())]);
    Ok(())
}

#[tokio::test]
async fn search_engine_ensure_registration_tolerates_concurrent_callers() -> Result<()> {
    let schema = local_symbol_schema();
    let batch = local_symbol_batch(&[("sym-1", "AlphaSymbol")], schema)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("xiuxian_vector_search_engine_concurrent_ensure_")
        .tempdir()?;
    let parquet_path = temp_dir.path().join("local_symbol.parquet");

    write_lance_batches_to_parquet_file(&parquet_path, &[batch])?;

    let engine = SearchEngineContext::new();
    let (left, right) = tokio::join!(
        engine.ensure_parquet_table_registered("local_symbol", &parquet_path, &[]),
        engine.ensure_parquet_table_registered("local_symbol", &parquet_path, &[]),
    );
    left?;
    right?;

    let batches = engine
        .sql_batches("SELECT id, name FROM local_symbol ORDER BY id")
        .await?;
    let rows = batches
        .iter()
        .flat_map(rows_from_engine_batch)
        .collect::<Vec<_>>();

    assert_eq!(rows, vec![("sym-1".to_string(), "AlphaSymbol".to_string())]);
    Ok(())
}
