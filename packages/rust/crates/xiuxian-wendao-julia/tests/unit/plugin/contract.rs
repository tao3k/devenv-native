use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{FixedSizeListArray, Float32Array, Float64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::julia_arrow_request_schema;
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_SCHEMA_VERSION, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
};

pub(crate) fn request_batch() -> RecordBatch {
    let vector_dim = 2;
    let batch = RecordBatch::try_new(
        julia_arrow_request_schema(vector_dim),
        vec![
            Arc::new(StringArray::from(vec!["doc-a", "doc-b"])),
            Arc::new(Float64Array::from(vec![0.4_f64, 0.7_f64])),
            Arc::new(fixed_size_vector_array(
                vector_dim,
                vec![0.1_f32, 0.2_f32, 0.3_f32, 0.4_f32],
            )),
            Arc::new(fixed_size_vector_array(
                vector_dim,
                vec![0.9_f32, 0.8_f32, 0.9_f32, 0.8_f32],
            )),
        ],
    )
    .unwrap_or_else(|error| panic!("request batch: {error}"));
    attach_record_batch_metadata(
        &batch,
        [(
            FLIGHT_SCHEMA_VERSION_METADATA_KEY,
            DEFAULT_FLIGHT_SCHEMA_VERSION,
        )],
    )
    .unwrap_or_else(|error| panic!("attach request schema metadata: {error}"))
}

pub(crate) fn request_batch_with_trace_id(trace_id: &str) -> RecordBatch {
    attach_record_batch_metadata(&request_batch(), [("trace_id", trace_id)])
        .unwrap_or_else(|error| panic!("attach trace metadata: {error}"))
}

fn attach_record_batch_metadata<K, V, I>(
    batch: &RecordBatch,
    metadata: I,
) -> Result<RecordBatch, ArrowError>
where
    K: Into<String>,
    V: Into<String>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut merged: HashMap<String, String> = batch.schema().metadata().clone();
    merged.extend(
        metadata
            .into_iter()
            .map(|(key, value)| (key.into(), value.into())),
    );

    let schema = Arc::new(Schema::new_with_metadata(
        batch.schema().fields().clone(),
        merged,
    ));
    RecordBatch::try_new(schema, batch.columns().to_vec())
}

fn fixed_size_vector_array(vector_dim: i32, values: Vec<f32>) -> FixedSizeListArray {
    FixedSizeListArray::try_new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        vector_dim,
        Arc::new(Float32Array::from(values)),
        None,
    )
    .unwrap_or_else(|error| panic!("fixed-size vector array: {error}"))
}
