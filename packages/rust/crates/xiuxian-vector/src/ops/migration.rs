//! One-shot schema migration: infer version from schema, apply v1→v2 (`TOOL_NAME` Utf8→Dictionary), etc.
//!
//! Version history:
//! - v1: Original schema (`TOOL_NAME`: Utf8)
//! - v2: `TOOL_NAME` Dictionary; `SKILL_NAME/CATEGORY` already Dictionary
//! - v3 (planned): `routing_keywords/intents` as List<Utf8>
//! - v4 (planned): metadata as Struct

use futures::TryStreamExt;
use lance::dataset::WriteParams;
use lance::deps::arrow_array::builder::{ListBuilder, StringBuilder};
use lance::deps::arrow_array::types::Int32Type;
use lance::deps::arrow_array::{
    DictionaryArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use lance::deps::arrow_schema::DataType;
use std::sync::Arc;

use crate::error::VectorStoreError;
use crate::ops::column_read::{get_intents_at, get_routing_keywords_at, get_utf8_at};
use crate::{
    CATEGORY_COLUMN, FILE_PATH_COLUMN, ID_COLUMN, SKILL_NAME_COLUMN, TOOL_NAME_COLUMN,
    VECTOR_COLUMN,
};
use crate::{CONTENT_COLUMN, INTENTS_COLUMN, ROUTING_KEYWORDS_COLUMN};

fn build_string_dictionary(
    values: &[String],
) -> Result<DictionaryArray<Int32Type>, VectorStoreError> {
    let mut uniq: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for s in values {
        if !map.contains_key(s) {
            let idx = i32::try_from(uniq.len()).map_err(|_| {
                VectorStoreError::General(
                    "tool_name dictionary exceeds i32 key capacity".to_string(),
                )
            })?;
            map.insert(s.clone(), idx);
            uniq.push(s.clone());
        }
    }
    let keys: Vec<i32> = values
        .iter()
        .map(|s| {
            map.get(s).copied().ok_or_else(|| {
                VectorStoreError::General(format!("missing tool_name dictionary key for '{s}'"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value_arr = StringArray::from(uniq);
    DictionaryArray::<Int32Type>::try_new(Int32Array::from(keys), Arc::new(value_arr))
        .map_err(VectorStoreError::Arrow)
}

/// Current target schema version. New tables are created at this version.
pub const XIUXIAN_SCHEMA_VERSION: u32 = 2;

/// One migration step (`from_version` → `to_version`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationItem {
    /// Schema version before this migration.
    pub from_version: u32,
    /// Schema version after this migration.
    pub to_version: u32,
    /// Human-readable description of the change.
    pub description: String,
}

/// Result of running migrations.
#[derive(Debug, Default, serde::Serialize)]
pub struct MigrateResult {
    /// Pairs (`from_version`, `to_version`) for each applied migration.
    pub applied: Vec<(u32, u32)>,
    /// Total rows processed across all batches.
    pub rows_processed: u64,
}

/// Infer schema version from a dataset's schema (by inspecting `TOOL_NAME` column type).
#[must_use]
pub fn schema_version_from_schema(schema: &lance::deps::arrow_schema::Schema) -> u32 {
    let Ok(field) = schema.field_with_name(TOOL_NAME_COLUMN) else {
        return 1;
    };
    match field.data_type() {
        DataType::Dictionary(_, _) => 2,
        _ => 1,
    }
}

/// Write params for migration (same as default writer in `writer_impl`).
fn migration_write_params() -> WriteParams {
    WriteParams {
        data_storage_version: Some(lance_file::version::LanceFileVersion::V2_1),
        ..WriteParams::default()
    }
}

/// Convert a single `RecordBatch` from v1 (`TOOL_NAME` Utf8) to v2 (`TOOL_NAME` Dictionary).
/// Other columns are passed through by reference. Caller must ensure `batch` has all 10 columns.
fn migrate_batch_v1_to_v2(
    batch: &RecordBatch,
    schema_v2: &Arc<lance::deps::arrow_schema::Schema>,
) -> Result<RecordBatch, VectorStoreError> {
    let tool_col = batch
        .column_by_name(TOOL_NAME_COLUMN)
        .ok_or_else(|| VectorStoreError::General("TOOL_NAME column not found".to_string()))?;
    let tool_names: Vec<String> = (0..batch.num_rows())
        .map(|i| get_utf8_at(tool_col.as_ref(), i))
        .collect();
    let tool_name_dict = Arc::new(build_string_dictionary(&tool_names)?);

    let rk_col = batch
        .column_by_name(ROUTING_KEYWORDS_COLUMN)
        .ok_or_else(|| {
            VectorStoreError::General("routing_keywords column not found".to_string())
        })?;
    let in_col = batch
        .column_by_name(INTENTS_COLUMN)
        .ok_or_else(|| VectorStoreError::General("intents column not found".to_string()))?;
    let mut rk_builder = ListBuilder::new(StringBuilder::new());
    let mut in_builder = ListBuilder::new(StringBuilder::new());
    for i in 0..batch.num_rows() {
        for s in get_routing_keywords_at(rk_col.as_ref(), i) {
            rk_builder.values().append_value(s.as_str());
        }
        rk_builder.append(true);
        for s in get_intents_at(in_col.as_ref(), i) {
            in_builder.values().append_value(s.as_str());
        }
        in_builder.append(true);
    }
    let routing_keywords_array = Arc::new(rk_builder.finish());
    let intents_array = Arc::new(in_builder.finish());

    // v1 tables have no metadata column; add empty metadata for v2 schema (10 columns).
    let n = batch.num_rows();
    let metadata_empty: Arc<dyn lance::deps::arrow_array::Array> =
        Arc::new(lance::deps::arrow_array::StringArray::from(vec![""; n]));

    let columns: Vec<Arc<dyn lance::deps::arrow_array::Array>> = vec![
        batch
            .column_by_name(ID_COLUMN)
            .ok_or_else(|| VectorStoreError::General("id column not found".to_string()))?
            .clone(),
        batch
            .column_by_name(VECTOR_COLUMN)
            .ok_or_else(|| VectorStoreError::General("vector column not found".to_string()))?
            .clone(),
        batch
            .column_by_name(CONTENT_COLUMN)
            .ok_or_else(|| VectorStoreError::General("content column not found".to_string()))?
            .clone(),
        batch
            .column_by_name(SKILL_NAME_COLUMN)
            .ok_or_else(|| VectorStoreError::General("skill_name column not found".to_string()))?
            .clone(),
        batch
            .column_by_name(CATEGORY_COLUMN)
            .ok_or_else(|| VectorStoreError::General("category column not found".to_string()))?
            .clone(),
        tool_name_dict,
        batch
            .column_by_name(FILE_PATH_COLUMN)
            .ok_or_else(|| VectorStoreError::General("file_path column not found".to_string()))?
            .clone(),
        routing_keywords_array,
        intents_array,
        metadata_empty,
    ];

    RecordBatch::try_new(schema_v2.clone(), columns).map_err(VectorStoreError::Arrow)
}

impl crate::VectorStore {
    /// List pending migrations for a table (based on current schema version vs `XIUXIAN_SCHEMA_VERSION`).
    ///
    /// # Errors
    ///
    /// Returns an error when opening the dataset or reading schema metadata fails.
    pub async fn check_migrations(
        &self,
        table_name: &str,
    ) -> Result<Vec<MigrationItem>, VectorStoreError> {
        let table_path = self.table_path(table_name);
        if !table_path.exists() {
            return Ok(vec![]);
        }
        let dataset = self
            .open_dataset_at_uri(table_path.to_string_lossy().as_ref())
            .await?;
        let arrow_schema = Arc::new(lance::deps::arrow_schema::Schema::from(dataset.schema()));
        // Only migrate tables that have the tool/skills schema (TOOL_NAME column).
        if arrow_schema.field_with_name(TOOL_NAME_COLUMN).is_err() {
            return Ok(vec![]);
        }
        let current = schema_version_from_schema(arrow_schema.as_ref());
        let mut out = Vec::new();
        let mut v = current;
        while v < XIUXIAN_SCHEMA_VERSION {
            let next = v + 1;
            let description = match (v, next) {
                (1, 2) => "TOOL_NAME Utf8 → Dictionary".to_string(),
                _ => format!("Schema v{v} → v{next}"),
            };
            out.push(MigrationItem {
                from_version: v,
                to_version: next,
                description,
            });
            v = next;
        }
        Ok(out)
    }

    /// Run all pending migrations for the table (detect version, apply v1→v2, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error when reading source data, converting batches, recreating
    /// the destination table, or appending migrated batches fails.
    pub async fn migrate(&mut self, table_name: &str) -> Result<MigrateResult, VectorStoreError> {
        let pending = self.check_migrations(table_name).await?;
        if pending.is_empty() {
            return Ok(MigrateResult::default());
        }

        if self.current_migration_schema_version(table_name).await? != 1 {
            return Ok(MigrateResult::default());
        }

        self.migrate_v1_table_to_v2(table_name).await
    }

    async fn current_migration_schema_version(
        &self,
        table_name: &str,
    ) -> Result<u32, VectorStoreError> {
        let table_path = self.table_path(table_name);
        let uri = table_path.to_string_lossy();
        let dataset = self.open_dataset_at_uri(uri.as_ref()).await?;
        let arrow_schema_v1 = Arc::new(lance::deps::arrow_schema::Schema::from(dataset.schema()));
        Ok(schema_version_from_schema(arrow_schema_v1.as_ref()))
    }

    async fn migrate_v1_table_to_v2(
        &mut self,
        table_name: &str,
    ) -> Result<MigrateResult, VectorStoreError> {
        let schema_v2 = self.create_schema();
        let table_path = self.table_path(table_name);
        let uri = table_path.to_string_lossy();
        let dataset = self.open_dataset_at_uri(uri.as_ref()).await?;
        let mut scanner = dataset.scan();
        scanner.project(&v1_migration_columns())?;
        let mut stream = scanner.try_into_stream().await?;

        let Some(first_batch) = stream.try_next().await? else {
            return Ok(MigrateResult {
                applied: vec![],
                rows_processed: 0,
            });
        };
        let mut rows_processed = first_batch.num_rows() as u64;
        let first_v2 = migrate_batch_v1_to_v2(&first_batch, &schema_v2)?;

        // Robustness note: drop-then-write. If append fails midway, table may be partial.
        // Migration is one-shot; caller should ensure table is backed up for critical data.
        self.drop_table(table_name).await?;

        let (mut dataset, created) = self
            .get_or_create_dataset(table_name, false, Some((schema_v2.clone(), first_v2)))
            .await?;
        if !created {
            return Err(VectorStoreError::General(
                "Expected new table after migrate drop".to_string(),
            ));
        }

        while let Some(batch) = stream.try_next().await? {
            rows_processed += batch.num_rows() as u64;
            let batch_v2 = migrate_batch_v1_to_v2(&batch, &schema_v2)?;
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(
                        vec![Ok(batch_v2)],
                        schema_v2.clone(),
                    )),
                    Some(migration_write_params()),
                )
                .await?;
        }

        {
            let mut cache = self.datasets.write().await;
            cache.insert(table_name.to_string(), dataset);
        }

        Ok(MigrateResult {
            applied: vec![(1, 2)],
            rows_processed,
        })
    }
}

fn v1_migration_columns() -> [&'static str; 9] {
    [
        ID_COLUMN,
        VECTOR_COLUMN,
        CONTENT_COLUMN,
        SKILL_NAME_COLUMN,
        CATEGORY_COLUMN,
        TOOL_NAME_COLUMN,
        FILE_PATH_COLUMN,
        ROUTING_KEYWORDS_COLUMN,
        INTENTS_COLUMN,
    ]
}
