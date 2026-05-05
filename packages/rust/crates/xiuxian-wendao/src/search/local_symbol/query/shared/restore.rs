use crate::search::SearchPlaneService;
use crate::search::contracts::AstSearchHit;
use crate::search::local_symbol::query::shared::{
    LocalSymbolSearchError, PreparedLocalSymbolRead, prepare_local_symbol_read_tables,
};

/// Restore all published local-symbol hits from the active search-plane tables.
///
/// # Errors
///
/// Returns a local-symbol search error when published tables cannot be opened
/// or decoded.
pub async fn restore_local_symbol_hits(
    service: &SearchPlaneService,
) -> Result<Vec<AstSearchHit>, LocalSymbolSearchError> {
    let prepared = prepare_local_symbol_read_tables(service).await?;
    if prepared.table_names.is_empty() {
        return Ok(Vec::new());
    }

    let mut hits = Vec::new();
    for table_name in &prepared.table_names {
        hits.extend(load_table_hits(&prepared, table_name.as_str()).await?);
    }
    Ok(hits)
}

async fn load_table_hits(
    prepared: &PreparedLocalSymbolRead,
    table_name: &str,
) -> Result<Vec<AstSearchHit>, LocalSymbolSearchError> {
    let sql = format!(
        "SELECT {hit_json_column} FROM {table_name}",
        hit_json_column = crate::search::local_symbol::schema::hit_json_column(),
    );
    let batches = prepared.query_engine.query_batches(sql.as_str()).await?;
    let mut hits = Vec::new();
    for batch in batches {
        let hit_json = string_column(
            &batch,
            crate::search::local_symbol::schema::hit_json_column(),
        )?;
        for row in 0..batch.num_rows() {
            hits.push(
                serde_json::from_str(hit_json.value(row))
                    .map_err(|error| LocalSymbolSearchError::Decode(error.to_string()))?,
            );
        }
    }
    Ok(hits)
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
