use std::collections::BTreeMap;
use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::{
    JULIA_PARSER_SUMMARY_BACKEND_COLUMN, JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN, JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN, JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN, JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN, JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN, JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN, JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN, JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN,
    JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN, JULIA_PARSER_SUMMARY_KIND_COLUMN,
    JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN, JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN,
    JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
    JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN,
    JULIA_PARSER_SUMMARY_SUCCESS_COLUMN, JuliaParserSummaryRequestRow,
    build_julia_parser_summary_request_batch, decode_julia_parser_file_summary,
    decode_julia_parser_root_summary, decode_julia_parser_summary_response_rows,
    validate_julia_parser_summary_response_batches,
};
use crate::plugin::parser_summary::transport::ParserSummaryRouteKind;
use crate::plugin::parser_summary::types::{
    JuliaParserDocAttachment, JuliaParserDocTargetKind, JuliaParserImport,
    JuliaParserSourceSummary, JuliaParserSymbol, JuliaParserSymbolKind,
};

#[test]
fn parser_summary_request_batch_materializes_rows() {
    let batch = build_julia_parser_summary_request_batch(&[JuliaParserSummaryRequestRow {
        request_id: "req-1".to_string(),
        source_id: "Demo.jl".to_string(),
        source_text: "module Demo\nend\n".to_string(),
    }])
    .unwrap_or_else(|error| panic!("request batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        batch.schema().field(0).name(),
        JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(1).name(),
        JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(2).name(),
        JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN
    );
}

#[test]
fn decode_parser_summary_rows_materializes_file_and_root_summaries() {
    let rows = decode_julia_parser_summary_response_rows(&[sample_response_batch()])
        .unwrap_or_else(|error| panic!("response rows should decode: {error}"));
    let file_summary =
        decode_julia_parser_file_summary(ParserSummaryRouteKind::FileSummary, rows.as_slice())
            .unwrap_or_else(|error| panic!("file summary should decode: {error}"));
    let root_summary =
        decode_julia_parser_root_summary(ParserSummaryRouteKind::FileSummary, rows.as_slice())
            .unwrap_or_else(|error| panic!("root summary should decode: {error}"));

    assert_eq!(
        root_summary,
        JuliaParserSourceSummary {
            module_name: "Demo".to_string(),
            exports: vec!["solve".to_string()],
            imports: vec![JuliaParserImport {
                module: "..Core.solve".to_string(),
                reexported: true,
                dependency_kind: "using".to_string(),
                dependency_form: "aliased_member".to_string(),
                dependency_is_relative: true,
                dependency_relative_level: 2,
                dependency_local_name: Some("solver".to_string()),
                dependency_parent: Some("..Core".to_string()),
                dependency_member: Some("solve".to_string()),
                dependency_alias: Some("solver".to_string()),
            }],
            symbols: vec![
                JuliaParserSymbol {
                    name: "LIMIT".to_string(),
                    kind: JuliaParserSymbolKind::Constant,
                    signature: Some("const LIMIT = 1".to_string()),
                    line_start: Some(3),
                    line_end: Some(3),
                    attributes: BTreeMap::from([
                        ("binding_kind".to_string(), "const".to_string()),
                        ("module_kind".to_string(), "module".to_string()),
                        ("parser_kind".to_string(), "binding".to_string()),
                        ("top_level".to_string(), "true".to_string()),
                    ]),
                },
                JuliaParserSymbol {
                    name: "solve".to_string(),
                    kind: JuliaParserSymbolKind::Function,
                    signature: Some("solve(problem::Problem)".to_string()),
                    line_start: Some(5),
                    line_end: Some(7),
                    attributes: BTreeMap::from([
                        ("function_keyword_arity".to_string(), "0".to_string()),
                        ("function_positional_arity".to_string(), "1".to_string()),
                        ("module_kind".to_string(), "module".to_string()),
                        ("parser_kind".to_string(), "function".to_string()),
                        ("top_level".to_string(), "true".to_string()),
                    ]),
                },
            ],
            docstrings: vec![JuliaParserDocAttachment {
                target_name: "solve".to_string(),
                target_kind: JuliaParserDocTargetKind::Symbol,
                target_path: Some("Demo.solve".to_string()),
                target_line_start: Some(5),
                target_line_end: Some(7),
                content: "Solve docs.".to_string(),
            }],
            includes: vec!["solvers.jl".to_string()],
        }
    );
    assert_eq!(file_summary.module_name.as_deref(), Some("Demo"));
}

#[test]
fn parser_summary_response_batches_reject_empty_batch_lists() {
    let validation = validate_julia_parser_summary_response_batches(&[]);
    let Err(error) = validation else {
        panic!("empty response batch lists should fail validation");
    };
    let message = error.to_string();
    assert!(
        message.contains("response stream returned no record batches"),
        "unexpected error message: {message}"
    );
}

fn sample_response_batch() -> RecordBatch {
    RecordBatch::try_new(sample_response_schema(), sample_response_columns())
        .unwrap_or_else(|error| panic!("sample response batch should build: {error}"))
}

fn sample_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(sample_response_schema_fields()))
}

fn sample_response_columns() -> Vec<ArrayRef> {
    vec![
        repeated_utf8(Some("req-1")),
        repeated_utf8(Some("Demo.jl")),
        repeated_utf8(Some("julia_file_summary")),
        repeated_utf8(Some("JuliaSyntax.jl")),
        repeated_bool(true),
        repeated_utf8(Some("Demo")),
        utf8_values([None::<&str>; 6]),
        repeated_utf8(Some("Demo")),
        repeated_utf8(Some("module")),
        utf8_values([
            Some("export"),
            Some("import"),
            Some("symbol"),
            Some("symbol"),
            Some("docstring"),
            Some("include"),
        ]),
        utf8_values([
            Some("solve"),
            None,
            Some("solve"),
            Some("LIMIT"),
            Some("solve"),
            None,
        ]),
        utf8_values([None, None, Some("function"), Some("binding"), None, None]),
        utf8_values([
            None,
            None,
            Some("solve(problem::Problem)"),
            Some("const LIMIT = 1"),
            None,
            None,
        ]),
        utf8_values([None, None, None, None, Some("function"), None]),
        utf8_values([None, None, None, None, Some("solve"), None]),
        utf8_values([None, None, None, None, Some("Demo.solve"), None]),
        i32_values([None, None, None, None, Some(5), None]),
        i32_values([None, None, None, None, Some(7), None]),
        utf8_values([None, Some("using"), None, None, None, Some("include")]),
        utf8_values([
            None,
            Some("aliased_member"),
            None,
            None,
            None,
            Some("include"),
        ]),
        utf8_values([
            None,
            Some("..Core.solve"),
            None,
            None,
            None,
            Some("solvers.jl"),
        ]),
        bool_values([None, Some(true), None, None, None, None]),
        i32_values([None, Some(2), None, None, None, None]),
        utf8_values([None, Some("solver"), None, None, None, None]),
        utf8_values([None, Some("..Core"), None, None, None, None]),
        utf8_values([None, Some("solve"), None, None, None, None]),
        utf8_values([None, Some("solver"), None, None, None, None]),
        utf8_values([None, None, None, None, Some("Solve docs."), None]),
        bool_values([None, Some(true), None, None, None, None]),
        utf8_values([None, None, None, None, None, Some("solvers.jl")]),
        utf8_values([None, None, None, Some("const"), None, None]),
        bool_values([None, None, Some(true), Some(true), None, None]),
        i32_values([None, None, Some(5), Some(3), None, None]),
        i32_values([None, None, Some(7), Some(3), None, None]),
        i32_values([None, None, Some(1), None, None, None]),
        i32_values([None, None, Some(0), None, None, None]),
    ]
}

fn sample_response_schema_fields() -> Vec<Field> {
    vec![
        required_field(JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN, DataType::Utf8),
        required_field(JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, DataType::Utf8),
        required_field(JULIA_PARSER_SUMMARY_KIND_COLUMN, DataType::Utf8),
        required_field(JULIA_PARSER_SUMMARY_BACKEND_COLUMN, DataType::Utf8),
        required_field(JULIA_PARSER_SUMMARY_SUCCESS_COLUMN, DataType::Boolean),
        optional_field(JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN, DataType::Utf8),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN, DataType::Utf8),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN,
            DataType::Boolean,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
            DataType::Utf8,
        ),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN, DataType::Utf8),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN,
            DataType::Boolean,
        ),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN, DataType::Utf8),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN,
            DataType::Utf8,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
            DataType::Boolean,
        ),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN, DataType::Int32),
        optional_field(JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN, DataType::Int32),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN,
            DataType::Int32,
        ),
        optional_field(
            JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN,
            DataType::Int32,
        ),
    ]
}

fn required_field(name: &str, data_type: DataType) -> Field {
    Field::new(name, data_type, false)
}

fn optional_field(name: &str, data_type: DataType) -> Field {
    Field::new(name, data_type, true)
}

fn repeated_utf8(value: Option<&str>) -> ArrayRef {
    Arc::new(StringArray::from(vec![value; 6]))
}

fn repeated_bool(value: bool) -> ArrayRef {
    Arc::new(BooleanArray::from(vec![value; 6]))
}

fn utf8_values(values: [Option<&str>; 6]) -> ArrayRef {
    Arc::new(StringArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn bool_values(values: [Option<bool>; 6]) -> ArrayRef {
    Arc::new(BooleanArray::from(values.into_iter().collect::<Vec<_>>()))
}

fn i32_values(values: [Option<i32>; 6]) -> ArrayRef {
    Arc::new(Int32Array::from(values.into_iter().collect::<Vec<_>>()))
}
