use std::sync::Arc;

use arrow::array::{BooleanArray, UInt64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::{
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
};
use crate::compatibility::link_graph::JULIA_PLUGIN_ID;

pub(crate) fn legacy_response_batch(
    capability_variant_field: Option<Field>,
    capability_variant_column: Option<Arc<dyn arrow::array::Array>>,
) -> RecordBatch {
    let row_count = capability_variant_column
        .as_ref()
        .map_or(1, |column| column.len());
    let (mut fields, mut columns) = legacy_response_fields_and_columns(row_count);

    if let Some(field) = capability_variant_field {
        fields.insert(2, field);
    }
    if let Some(column) = capability_variant_column {
        columns.insert(2, column);
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .unwrap_or_else(|error| panic!("legacy response batch should build: {error}"))
}

fn legacy_response_fields_and_columns(
    row_count: usize,
) -> (Vec<Field>, Vec<Arc<dyn arrow::array::Array>>) {
    let fields = vec![
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
            DataType::UInt64,
            true,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
            DataType::Boolean,
            false,
        ),
    ];
    let columns: Vec<Arc<dyn arrow::array::Array>> = vec![
        Arc::new(StringArray::from(vec![Some(JULIA_PLUGIN_ID); row_count])),
        Arc::new(StringArray::from(vec![Some("rerank"); row_count])),
        Arc::new(StringArray::from(vec![
            Some("arrow_flight");
            row_count
        ])),
        Arc::new(StringArray::from(vec![
            Some("http://127.0.0.1:8815");
            row_count
        ])),
        Arc::new(StringArray::from(vec![Some("/rerank"); row_count])),
        Arc::new(StringArray::from(vec![Some("/healthz"); row_count])),
        Arc::new(StringArray::from(vec![Some("v0-draft"); row_count])),
        Arc::new(UInt64Array::from(vec![Some(15); row_count])),
        Arc::new(BooleanArray::from(vec![true; row_count])),
    ];
    (fields, columns)
}

pub(crate) fn legacy_response_batch_with_replaced_column(
    column_name: &str,
    field: Field,
    column: Arc<dyn arrow::array::Array>,
) -> RecordBatch {
    let (mut fields, mut columns) = legacy_response_fields_and_columns(column.len());
    let index = fields
        .iter()
        .position(|field| field.name() == column_name)
        .unwrap_or_else(|| panic!("legacy response should contain `{column_name}`"));
    fields[index] = field;
    columns[index] = column;

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)
        .unwrap_or_else(|error| panic!("legacy response batch should build: {error}"))
}

pub(crate) fn legacy_response_batch_without_health_route() -> RecordBatch {
    legacy_response_batch_without_column(JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN)
}

pub(crate) fn legacy_response_batch_without_timeout_secs() -> RecordBatch {
    legacy_response_batch_without_column(JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN)
}

fn legacy_response_batch_without_column(column_name: &str) -> RecordBatch {
    let batch = legacy_response_batch(None, None);
    let schema = batch.schema();
    let index = schema
        .index_of(column_name)
        .unwrap_or_else(|error| panic!("legacy response should contain `{column_name}`: {error}"));
    let mut fields = schema
        .fields()
        .iter()
        .map(|field| field.as_ref().clone())
        .collect::<Vec<_>>();
    let mut columns = batch.columns().to_vec();
    fields.remove(index);
    columns.remove(index);

    RecordBatch::try_new(Arc::new(Schema::new(fields)), columns).unwrap_or_else(|error| {
        panic!("legacy response batch without `{column_name}` should build: {error}")
    })
}
