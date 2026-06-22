use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use arrow::array::{BinaryArray, Int64Array, ListBuilder, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, WENDAO_TABLE_METADATA_KEY,
};

pub(super) const TEST_TABLE: &str = "db_store_arrow_schema_test";
pub(super) const TEST_COLUMNS: [ArrowSchemaColumn; 3] = [
    ArrowSchemaColumn::new("id", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::nullable("payload", ArrowSchemaDataType::BinaryPayload),
];

pub(super) fn test_contract(exact_column_set: bool) -> ArrowSchemaContract {
    ArrowSchemaContract::new(TEST_TABLE, exact_column_set, TEST_COLUMNS.to_vec())
}

pub(super) fn valid_batch_with_table(table_name: &str) -> Result<RecordBatch, Box<dyn Error>> {
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

pub(super) fn table_metadata(table_name: &str) -> HashMap<String, String> {
    [(WENDAO_TABLE_METADATA_KEY.to_owned(), table_name.to_owned())]
        .into_iter()
        .collect()
}

pub(super) fn utf8_list_array<'a>(
    rows: impl IntoIterator<Item = &'a [&'a str]>,
) -> arrow::array::ListArray {
    let mut builder = ListBuilder::new(arrow::array::StringBuilder::new());
    for row in rows {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
