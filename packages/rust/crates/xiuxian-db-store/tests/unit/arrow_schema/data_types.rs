use std::error::Error;
use std::sync::Arc;

use arrow::array::{Int32Array, StringArray, TimestampMillisecondArray, UInt64Array};
use arrow::datatypes::{DataType, Field, TimeUnit};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, arrow_field_for_column,
    build_arrow_schema, validate_record_batch_schema,
};

use super::helpers::{TEST_TABLE, utf8_list_array};

#[test]
fn builds_uint64_fields_from_column_contract() -> Result<(), Box<dyn Error>> {
    let field = arrow_field_for_column(ArrowSchemaColumn::nullable(
        "line",
        ArrowSchemaDataType::UInt64,
    ));
    assert_eq!(field.name(), "line");
    assert_eq!(field.data_type(), &DataType::UInt64);
    assert!(field.is_nullable());

    let contract = ArrowSchemaContract::new(
        TEST_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new("id", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::nullable("line", ArrowSchemaDataType::UInt64),
        ],
    );
    let schema = Arc::new(build_arrow_schema(
        &contract,
        std::collections::HashMap::new(),
    ));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["row-1"])),
            Arc::new(UInt64Array::from(vec![Some(7)])),
        ],
    )?;

    validate_record_batch_schema(&batch, &contract)?;
    Ok(())
}

#[test]
fn builds_graph_structural_int32_and_utf8_list_fields_from_contract() -> Result<(), Box<dyn Error>>
{
    let contract = ArrowSchemaContract::new(
        TEST_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new("layer", ArrowSchemaDataType::Int32),
            ArrowSchemaColumn::new("anchors", ArrowSchemaDataType::Utf8List),
        ],
    );
    let schema = Arc::new(build_arrow_schema(
        &contract,
        std::collections::HashMap::new(),
    ));
    assert_eq!(schema.field(0).data_type(), &DataType::Int32);
    assert_eq!(
        schema.field(1).data_type(),
        &DataType::List(Arc::new(Field::new("item", DataType::Utf8, true)))
    );

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![1])),
            Arc::new(utf8_list_array([["plane", "value"].as_slice()])),
        ],
    )?;

    validate_record_batch_schema(&batch, &contract)?;
    Ok(())
}

#[test]
fn builds_timestamp_millisecond_fields_from_contract() -> Result<(), Box<dyn Error>> {
    let contract = ArrowSchemaContract::new(
        TEST_TABLE,
        true,
        vec![
            ArrowSchemaColumn::new("id", ArrowSchemaDataType::Utf8),
            ArrowSchemaColumn::new("created_at", ArrowSchemaDataType::TimestampMillisecond),
        ],
    );
    let schema = Arc::new(build_arrow_schema(
        &contract,
        std::collections::HashMap::new(),
    ));
    assert_eq!(
        schema.field(1).data_type(),
        &DataType::Timestamp(TimeUnit::Millisecond, None)
    );

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["row-1"])),
            Arc::new(TimestampMillisecondArray::from(vec![1_778_000_000_000])),
        ],
    )?;

    validate_record_batch_schema(&batch, &contract)?;
    Ok(())
}
