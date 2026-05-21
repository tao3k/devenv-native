//! Arrow record batch helpers for Lance-backed vector data.

use std::sync::Arc;

use lance::deps::arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchReader, StringArray,
};
use lance::deps::arrow_schema::{ArrowError, DataType, Field, Schema};

/// Default vector embedding dimension.
pub const DEFAULT_DIMENSION: usize = 1536;
/// Canonical primary key column name.
pub const ID_COLUMN: &str = "id";
/// Canonical dense-vector column name.
pub const VECTOR_COLUMN: &str = "vector";
/// Canonical content column name.
pub const CONTENT_COLUMN: &str = "content";
/// Canonical metadata JSON column name.
pub const METADATA_COLUMN: &str = "metadata";
/// Canonical thread-id column name for checkpoint rows.
pub const THREAD_ID_COLUMN: &str = "thread_id";
/// Canonical skill-name column name.
pub const SKILL_NAME_COLUMN: &str = "skill_name";
/// Canonical category column name.
pub const CATEGORY_COLUMN: &str = "category";
/// Canonical tool-name column name.
pub const TOOL_NAME_COLUMN: &str = "tool_name";
/// Canonical file-path column name.
pub const FILE_PATH_COLUMN: &str = "file_path";
/// Canonical routing-keywords column name.
pub const ROUTING_KEYWORDS_COLUMN: &str = "routing_keywords";
/// Canonical intents column name.
pub const INTENTS_COLUMN: &str = "intents";

/// A record batch reader for vector store data.
pub struct VectorRecordBatchReader {
    schema: Arc<Schema>,
    batches: Vec<RecordBatch>,
    current_batch: usize,
}

/// Named input for creating a vector record batch reader.
#[derive(Debug, Clone)]
pub struct VectorBatchInput {
    /// Primary keys for each vector row.
    pub ids: Vec<String>,
    /// Dense vector values.
    pub vectors: Vec<Vec<f32>>,
    /// Row content values.
    pub contents: Vec<String>,
    /// Metadata JSON values.
    pub metadatas: Vec<String>,
    /// Fixed vector dimension.
    pub dimension: usize,
}

impl VectorRecordBatchReader {
    /// Create a new reader from a vector store batch.
    #[must_use]
    pub fn new(schema: Arc<Schema>, batches: Vec<RecordBatch>) -> Self {
        Self {
            schema,
            batches,
            current_batch: 0,
        }
    }

    /// Create a reader from individual vectors.
    ///
    /// # Errors
    ///
    /// Returns an error if `dimension` exceeds the Arrow fixed-size-list range
    /// or if building the underlying Arrow arrays or record batch fails.
    pub fn from_vectors(
        input: VectorBatchInput,
    ) -> Result<Self, lance::deps::arrow_schema::ArrowError> {
        let id_array = StringArray::from(input.ids);
        let content_array = StringArray::from(input.contents);
        let metadata_array = StringArray::from(input.metadatas);
        let schema = Self::default_schema(input.dimension)?;
        let dimension = i32::try_from(input.dimension).map_err(|_| {
            ArrowError::SchemaError("vector dimension exceeds i32 range".to_string())
        })?;

        let flat_values: Vec<f32> = input.vectors.into_iter().flatten().collect();
        let vector_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            dimension,
            Arc::new(Float32Array::from(flat_values)),
            None,
        )?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(content_array),
                Arc::new(metadata_array),
            ],
        )?;

        Ok(Self {
            schema,
            batches: vec![batch],
            current_batch: 0,
        })
    }

    /// Build the default schema used by vector-store record batches.
    ///
    /// # Errors
    ///
    /// Returns an error if `dimension` exceeds the Arrow fixed-size-list range.
    pub fn default_schema(dimension: usize) -> Result<Arc<Schema>, ArrowError> {
        let dimension = i32::try_from(dimension).map_err(|_| {
            ArrowError::SchemaError("vector dimension exceeds i32 range".to_string())
        })?;
        Ok(Arc::new(Schema::new(vec![
            Field::new(ID_COLUMN, DataType::Utf8, false),
            Field::new(
                VECTOR_COLUMN,
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dimension,
                ),
                false,
            ),
            Field::new(CONTENT_COLUMN, DataType::Utf8, false),
            Field::new(METADATA_COLUMN, DataType::Utf8, true),
        ])))
    }
}

impl Iterator for VectorRecordBatchReader {
    type Item = Result<RecordBatch, lance::deps::arrow_schema::ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_batch >= self.batches.len() {
            return None;
        }
        let batch = self.batches[self.current_batch].clone();
        self.current_batch += 1;
        Some(Ok(batch))
    }
}

impl RecordBatchReader for VectorRecordBatchReader {
    fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
}

/// Extract string values from a `StringArray` at a specific index.
#[must_use]
pub fn extract_string(array: &StringArray, index: usize) -> String {
    if array.is_null(index) {
        String::new()
    } else {
        array.value(index).to_string()
    }
}

/// Extract optional string from metadata column.
#[must_use]
pub fn extract_optional_string(array: Option<&StringArray>, index: usize) -> Option<String> {
    array.and_then(|arr| {
        if arr.is_null(index) {
            None
        } else {
            Some(arr.value(index).to_string())
        }
    })
}
