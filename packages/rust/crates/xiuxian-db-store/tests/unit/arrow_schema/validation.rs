use std::error::Error;
use std::sync::Arc;

use arrow::array::{BinaryArray, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaNullabilityPolicy, ArrowSchemaValidationOptions, validate_record_batch_schema,
    validate_schema_against_contract_with_options,
};

use super::helpers::{TEST_TABLE, table_metadata, test_contract, valid_batch_with_table};

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
