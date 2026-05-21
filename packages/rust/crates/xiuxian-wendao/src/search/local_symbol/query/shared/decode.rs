//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeMap;

use crate::duckdb::ParquetQueryEngine;
use crate::search::contracts::AstSearchHit;

use super::types::{LocalSymbolCandidate, LocalSymbolSearchError};

pub(crate) async fn decode_local_symbol_hits(
    engine: &ParquetQueryEngine,
    candidates: Vec<LocalSymbolCandidate>,
) -> Result<Vec<AstSearchHit>, LocalSymbolSearchError> {
    let payloads = load_hit_payloads(engine, candidates.as_slice()).await?;
    candidates
        .into_iter()
        .map(|candidate| {
            let hit_json = payloads
                .get(candidate.table_name.as_str())
                .and_then(|entries| entries.get(candidate.id.as_str()))
                .ok_or_else(|| {
                    LocalSymbolSearchError::Decode(format!(
                        "local symbol hydration missing payload for table `{}` id `{}`",
                        candidate.table_name, candidate.id
                    ))
                })?;
            let mut hit: AstSearchHit = serde_json::from_str(hit_json.as_str())
                .map_err(|error| LocalSymbolSearchError::Decode(error.to_string()))?;
            hit.score = candidate.score;
            Ok(hit)
        })
        .collect()
}

async fn load_hit_payloads(
    engine: &ParquetQueryEngine,
    candidates: &[LocalSymbolCandidate],
) -> Result<BTreeMap<String, BTreeMap<String, String>>, LocalSymbolSearchError> {
    let mut ids_by_table = BTreeMap::<String, Vec<String>>::new();
    for candidate in candidates {
        ids_by_table
            .entry(candidate.table_name.clone())
            .or_default()
            .push(candidate.id.clone());
    }

    let mut payloads = BTreeMap::<String, BTreeMap<String, String>>::new();
    for (table_name, ids) in ids_by_table {
        let sql = format!(
            "SELECT {id_column}, {hit_json_column} FROM {table_name} WHERE {id_column} IN ({ids})",
            id_column = crate::search::local_symbol::schema::id_column(),
            hit_json_column = crate::search::local_symbol::schema::hit_json_column(),
            ids = ids
                .iter()
                .map(|id| sql_string_literal(id.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let batches = engine.query_batches(sql.as_str()).await?;
        let table_payloads = payloads.entry(table_name.clone()).or_default();
        for batch in batches {
            let id = string_column(&batch, crate::search::local_symbol::schema::id_column())?;
            let hit_json = string_column(
                &batch,
                crate::search::local_symbol::schema::hit_json_column(),
            )?;
            for row in 0..batch.num_rows() {
                table_payloads.insert(id.value(row).to_string(), hit_json.value(row).to_string());
            }
        }
    }

    Ok(payloads)
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn string_column<'a>(
    batch: &'a xiuxian_db_store::EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, LocalSymbolSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        LocalSymbolSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;
    if let Some(array) = column.as_any().downcast_ref::<arrow::array::StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column
        .as_any()
        .downcast_ref::<arrow::array::StringViewArray>()
    {
        return Ok(EngineStringColumn::Utf8View(array));
    }
    Err(LocalSymbolSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

#[derive(Clone, Copy)]
enum EngineStringColumn<'a> {
    Utf8(&'a arrow::array::StringArray),
    Utf8View(&'a arrow::array::StringViewArray),
}

impl<'a> EngineStringColumn<'a> {
    fn value(self, row: usize) -> &'a str {
        match self {
            Self::Utf8(column) => column.value(row),
            Self::Utf8View(column) => column.value(row),
        }
    }
}
