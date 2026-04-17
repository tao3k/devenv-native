use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int32Array, NullArray, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::{
    MODELICA_PARSER_SUMMARY_BACKEND_COLUMN, MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_CLASS_PATH_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_COMPONENT_KIND_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_DEFAULT_VALUE_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_GROUP_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_IS_ENCAPSULATED_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_IS_FINAL_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_IS_PARTIAL_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_KIND_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_TEXT_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_TYPE_NAME_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_UNIT_COLUMN, MODELICA_PARSER_SUMMARY_ITEM_VARIABILITY_COLUMN,
    MODELICA_PARSER_SUMMARY_ITEM_VISIBILITY_COLUMN, MODELICA_PARSER_SUMMARY_KIND_COLUMN,
    MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
    MODELICA_PARSER_SUMMARY_RESTRICTION_COLUMN, MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
    MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN, decode_modelica_parser_file_summary,
    decode_modelica_parser_summary_response_rows,
};
use crate::modelica_plugin::parser_summary::transport::ParserSummaryRouteKind;

#[test]
fn decode_modelica_parser_summary_rows_accepts_null_optional_columns() {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(
                MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
                DataType::Utf8,
                false,
            ),
            Field::new(
                MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
                DataType::Utf8,
                false,
            ),
            Field::new(MODELICA_PARSER_SUMMARY_KIND_COLUMN, DataType::Utf8, false),
            Field::new(
                MODELICA_PARSER_SUMMARY_BACKEND_COLUMN,
                DataType::Utf8,
                false,
            ),
            Field::new(
                MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN,
                DataType::Boolean,
                false,
            ),
            Field::new(
                MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN,
                DataType::Null,
                true,
            ),
            Field::new(
                MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
                DataType::Null,
                true,
            ),
            Field::new(
                MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN,
                DataType::Utf8,
                true,
            ),
        ])),
        vec![
            Arc::new(StringArray::from(vec![Some("req-1")])),
            Arc::new(StringArray::from(vec![Some("Demo.mo")])),
            Arc::new(StringArray::from(vec![Some("modelica_file_summary")])),
            Arc::new(StringArray::from(vec![Some("OMParser.jl")])),
            Arc::new(BooleanArray::from(vec![Some(true)])),
            Arc::new(NullArray::new(1)),
            Arc::new(NullArray::new(1)),
            Arc::new(StringArray::from(vec![Some("Demo")])),
        ],
    )
    .unwrap_or_else(|error| panic!("build sample batch: {error}"));

    let rows = decode_modelica_parser_summary_response_rows(&[batch])
        .unwrap_or_else(|error| panic!("decode modelica parser-summary response rows: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].request_id, "req-1");
    assert_eq!(rows[0].primary_name, None);
    assert_eq!(rows[0].error_message, None);
    assert_eq!(rows[0].class_name.as_deref(), Some("Demo"));
}

#[test]
fn decode_modelica_parser_file_summary_preserves_declaration_attributes() {
    let batch = sample_modelica_declaration_attribute_batch();

    let rows = decode_modelica_parser_summary_response_rows(&[batch])
        .unwrap_or_else(|error| panic!("decode modelica parser-summary response rows: {error}"));
    let summary = decode_modelica_parser_file_summary(ParserSummaryRouteKind::FileSummary, &rows)
        .unwrap_or_else(|error| panic!("decode modelica parser file summary: {error}"));

    assert_eq!(summary.declarations.len(), 1);
    let declaration = &summary.declarations[0];
    assert_eq!(declaration.name, "k");
    assert_eq!(
        declaration.attributes.get("visibility").map(String::as_str),
        Some("public"),
    );
    assert_eq!(
        declaration
            .attributes
            .get("variability")
            .map(String::as_str),
        Some("parameter"),
    );
    assert_eq!(
        declaration.attributes.get("type_name").map(String::as_str),
        Some("Real"),
    );
    assert_eq!(
        declaration.attributes.get("unit").map(String::as_str),
        Some("kg"),
    );
    assert_eq!(
        declaration.attributes.get("owner_path").map(String::as_str),
        Some("PI"),
    );
    assert_eq!(
        declaration
            .attributes
            .get("equation_latex")
            .map(String::as_str),
        Some("y = k;"),
    );
}

fn sample_modelica_declaration_attribute_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(sample_modelica_declaration_attribute_fields())),
        sample_modelica_declaration_attribute_columns(),
    )
    .unwrap_or_else(|error| panic!("build sample batch: {error}"))
}

fn sample_modelica_declaration_attribute_fields() -> Vec<Field> {
    vec![
        required_field(MODELICA_PARSER_SUMMARY_REQUEST_ID_COLUMN, DataType::Utf8),
        required_field(MODELICA_PARSER_SUMMARY_SOURCE_ID_COLUMN, DataType::Utf8),
        required_field(MODELICA_PARSER_SUMMARY_KIND_COLUMN, DataType::Utf8),
        required_field(MODELICA_PARSER_SUMMARY_BACKEND_COLUMN, DataType::Utf8),
        required_field(MODELICA_PARSER_SUMMARY_SUCCESS_COLUMN, DataType::Boolean),
        optional_field(MODELICA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, DataType::Null),
        optional_field(MODELICA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN, DataType::Null),
        optional_field(MODELICA_PARSER_SUMMARY_CLASS_NAME_COLUMN, DataType::Utf8),
        optional_field(MODELICA_PARSER_SUMMARY_RESTRICTION_COLUMN, DataType::Utf8),
        optional_field(MODELICA_PARSER_SUMMARY_ITEM_GROUP_COLUMN, DataType::Utf8),
        optional_field(MODELICA_PARSER_SUMMARY_ITEM_NAME_COLUMN, DataType::Utf8),
        optional_field(MODELICA_PARSER_SUMMARY_ITEM_KIND_COLUMN, DataType::Utf8),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN,
            DataType::Utf8,
        ),
        optional_field(MODELICA_PARSER_SUMMARY_ITEM_TEXT_COLUMN, DataType::Utf8),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_OWNER_NAME_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_OWNER_PATH_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_VISIBILITY_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_TYPE_NAME_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_VARIABILITY_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_COMPONENT_KIND_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_DEFAULT_VALUE_COLUMN,
            DataType::Utf8,
        ),
        optional_field(MODELICA_PARSER_SUMMARY_ITEM_UNIT_COLUMN, DataType::Utf8),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_CLASS_PATH_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
            DataType::Boolean,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_IS_PARTIAL_COLUMN,
            DataType::Boolean,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_IS_FINAL_COLUMN,
            DataType::Boolean,
        ),
        optional_field(
            MODELICA_PARSER_SUMMARY_ITEM_IS_ENCAPSULATED_COLUMN,
            DataType::Boolean,
        ),
    ]
}

fn sample_modelica_declaration_attribute_columns() -> Vec<ArrayRef> {
    vec![
        modelica_utf8([Some("req-1"), Some("req-1")]),
        modelica_utf8([Some("Demo.mo"), Some("Demo.mo")]),
        modelica_utf8([Some("modelica_file_summary"), Some("modelica_file_summary")]),
        modelica_utf8([Some("OMParser.jl"), Some("OMParser.jl")]),
        modelica_bool([Some(true), Some(true)]),
        modelica_null(2),
        modelica_null(2),
        modelica_utf8([Some("PI"), Some("PI")]),
        modelica_utf8([Some("model"), Some("model")]),
        modelica_utf8([Some("symbol"), Some("equation")]),
        modelica_utf8([Some("k"), Some("PI")]),
        modelica_utf8([Some("parameter"), None]),
        modelica_utf8([Some("parameter Real k = 1;"), None]),
        modelica_utf8([None, Some("y = k;")]),
        modelica_i32([Some(2), Some(4)]),
        modelica_i32([Some(2), Some(4)]),
        modelica_utf8([Some("PI"), Some("PI")]),
        modelica_utf8([Some("PI"), Some("PI")]),
        modelica_utf8([Some("public"), None]),
        modelica_utf8([Some("Real"), None]),
        modelica_utf8([Some("parameter"), None]),
        modelica_utf8([Some("component"), None]),
        modelica_utf8([Some("1"), None]),
        modelica_utf8([Some("kg"), None]),
        modelica_utf8([Some("PI"), Some("PI")]),
        modelica_bool([Some(false), None]),
        modelica_bool([Some(false), None]),
        modelica_bool([Some(true), None]),
        modelica_bool([Some(false), None]),
    ]
}

fn required_field(name: &str, data_type: DataType) -> Field {
    Field::new(name, data_type, false)
}

fn optional_field(name: &str, data_type: DataType) -> Field {
    Field::new(name, data_type, true)
}

fn modelica_utf8(values: [Option<&str>; 2]) -> ArrayRef {
    Arc::new(StringArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn modelica_i32(values: [Option<i32>; 2]) -> ArrayRef {
    Arc::new(Int32Array::from(values.into_iter().collect::<Vec<_>>()))
}

fn modelica_bool(values: [Option<bool>; 2]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn modelica_null(row_count: usize) -> ArrayRef {
    Arc::new(NullArray::new(row_count))
}
