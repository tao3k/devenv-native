use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use arrow::array::{
    BinaryArray, Int32Array, Int64Array, ListBuilder, StringArray, TimestampMillisecondArray,
    UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, arrow_field_for_column,
    build_arrow_schema, encode_record_batch_ipc, validate_arrow_ipc_stream,
    validate_record_batch_schema, validate_schema_against_contract_with_options,
};

const TEST_TABLE: &str = "db_store_arrow_schema_test";
const TEST_COLUMNS: [ArrowSchemaColumn; 3] = [
    ArrowSchemaColumn::new("id", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::nullable("payload", ArrowSchemaDataType::BinaryPayload),
];

#[test]
fn builds_arrow_schema_from_contract() {
    let schema = build_arrow_schema(
        &test_contract(true),
        [(WENDAO_TABLE_METADATA_KEY.to_owned(), TEST_TABLE.to_owned())]
            .into_iter()
            .collect::<HashMap<_, _>>(),
    );

    assert_eq!(
        schema.metadata().get(WENDAO_TABLE_METADATA_KEY),
        Some(&TEST_TABLE.to_owned())
    );
    assert_eq!(schema.fields().len(), 3);
    assert_eq!(schema.fields()[0].name(), "id");
    assert_eq!(schema.fields()[0].data_type(), &DataType::Utf8);
    assert!(!schema.fields()[0].is_nullable());
    assert_eq!(schema.fields()[2].name(), "payload");
    assert_eq!(schema.fields()[2].data_type(), &DataType::Binary);
    assert!(schema.fields()[2].is_nullable());
}

#[test]
fn validates_record_batch_against_contract() -> Result<(), Box<dyn Error>> {
    let batch = valid_batch_with_table(TEST_TABLE)?;

    validate_record_batch_schema(&batch, &test_contract(true))?;

    Ok(())
}

#[test]
fn rejects_wrong_table_metadata() -> Result<(), Box<dyn Error>> {
    let batch = valid_batch_with_table("wrong_table")?;

    let error = validate_record_batch_schema(&batch, &test_contract(true))
        .err()
        .ok_or("validation should reject wrong table metadata")?;

    assert!(error.to_string().contains("table metadata must be"));
    assert!(error.to_string().contains("wrong_table"));
    Ok(())
}

#[test]
fn rejects_missing_required_column() -> Result<(), Box<dyn Error>> {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![Field::new("id", DataType::Utf8, false)],
        table_metadata(TEST_TABLE),
    ));
    let batch = RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec!["row-1"]))])?;

    let error = validate_record_batch_schema(&batch, &test_contract(false))
        .err()
        .ok_or("validation should reject missing columns")?;

    assert!(
        error
            .to_string()
            .contains("missing required column `count`")
    );
    Ok(())
}

#[test]
fn rejects_exact_column_order_mismatch() -> Result<(), Box<dyn Error>> {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("count", DataType::Int64, false),
            Field::new("id", DataType::Utf8, false),
            Field::new("payload", DataType::Binary, true),
        ],
        table_metadata(TEST_TABLE),
    ));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![1])),
            Arc::new(StringArray::from(vec!["row-1"])),
            Arc::new(BinaryArray::from_iter([Some(b"bytes".as_slice())])),
        ],
    )?;

    let error = validate_record_batch_schema(&batch, &test_contract(true))
        .err()
        .ok_or("validation should reject exact order mismatch")?;

    assert!(
        error
            .to_string()
            .contains("column 0 must be `id` but was `count`")
    );
    Ok(())
}

#[test]
fn validates_arrow_ipc_stream_against_contract() -> Result<(), Box<dyn Error>> {
    let batch = valid_batch_with_table(TEST_TABLE)?;
    let payload = encode_record_batch_ipc(&batch)?;

    validate_arrow_ipc_stream(payload.as_slice(), &test_contract(true))?;

    Ok(())
}

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
    let schema = Arc::new(build_arrow_schema(&contract, HashMap::new()));
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
    let schema = Arc::new(build_arrow_schema(&contract, HashMap::new()));
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
    let schema = Arc::new(build_arrow_schema(&contract, HashMap::new()));
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

#[test]
fn rejects_empty_arrow_ipc_payload() -> Result<(), Box<dyn Error>> {
    let error = validate_arrow_ipc_stream(&[], &test_contract(true))
        .err()
        .ok_or("validation should reject an empty IPC payload")?;

    assert!(
        error
            .to_string()
            .contains("Arrow IPC payload must not be empty")
    );
    Ok(())
}

#[test]
fn rejects_nullability_mismatch_with_exact_policy() -> Result<(), Box<dyn Error>> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Utf8, true),
        Field::new("count", DataType::Int64, false),
        Field::new("payload", DataType::Binary, true),
    ]);

    let Err(error) = validate_schema_against_contract_with_options(
        &schema,
        &test_contract(true),
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    ) else {
        return Err("exact nullability validation should reject mismatch".into());
    };

    assert!(
        error
            .to_string()
            .contains("column `id` expected nullable=false but received nullable=true")
    );
    Ok(())
}

#[test]
fn allows_nullability_widening_for_sql_outputs() -> Result<(), Box<dyn Error>> {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Utf8, true),
        Field::new("count", DataType::Int64, true),
        Field::new("payload", DataType::Binary, true),
    ]);

    validate_schema_against_contract_with_options(
        &schema,
        &test_contract(true),
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::AllowWidening),
    )?;

    Ok(())
}

fn test_contract(exact_column_set: bool) -> ArrowSchemaContract {
    ArrowSchemaContract::new(TEST_TABLE, exact_column_set, TEST_COLUMNS.to_vec())
}

fn valid_batch_with_table(table_name: &str) -> Result<RecordBatch, Box<dyn Error>> {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("count", DataType::Int64, false),
            Field::new("payload", DataType::Binary, true),
        ],
        table_metadata(table_name),
    ));
    Ok(RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["row-1"])),
            Arc::new(Int64Array::from(vec![1])),
            Arc::new(BinaryArray::from_iter([Some(b"bytes".as_slice())])),
        ],
    )?)
}

fn table_metadata(table_name: &str) -> HashMap<String, String> {
    [(WENDAO_TABLE_METADATA_KEY.to_owned(), table_name.to_owned())]
        .into_iter()
        .collect()
}

fn utf8_list_array<'a>(rows: impl IntoIterator<Item = &'a [&'a str]>) -> arrow::array::ListArray {
    let mut builder = ListBuilder::new(arrow::array::StringBuilder::new());
    for row in rows {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
