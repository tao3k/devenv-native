use std::collections::HashMap;

use arrow::datatypes::DataType;
use xiuxian_db_store::{WENDAO_TABLE_METADATA_KEY, build_arrow_schema};

use super::helpers::{TEST_TABLE, test_contract};

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
