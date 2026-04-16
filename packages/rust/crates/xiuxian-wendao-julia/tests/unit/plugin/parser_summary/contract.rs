use std::collections::BTreeMap;
use std::sync::Arc;

use arrow::array::{BooleanArray, Int32Array, StringArray};
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
    let error = validate_julia_parser_summary_response_batches(&[])
        .expect_err("empty response batch lists should fail validation");
    let message = error.to_string();
    assert!(
        message.contains("response stream returned no record batches"),
        "unexpected error message: {message}"
    );
}

fn sample_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        sample_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![
                Some("req-1"),
                Some("req-1"),
                Some("req-1"),
                Some("req-1"),
                Some("req-1"),
                Some("req-1"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("Demo.jl"),
                Some("Demo.jl"),
                Some("Demo.jl"),
                Some("Demo.jl"),
                Some("Demo.jl"),
                Some("Demo.jl"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("julia_file_summary"),
                Some("julia_file_summary"),
                Some("julia_file_summary"),
                Some("julia_file_summary"),
                Some("julia_file_summary"),
                Some("julia_file_summary"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("JuliaSyntax.jl"),
                Some("JuliaSyntax.jl"),
                Some("JuliaSyntax.jl"),
                Some("JuliaSyntax.jl"),
                Some("JuliaSyntax.jl"),
                Some("JuliaSyntax.jl"),
            ])),
            Arc::new(BooleanArray::from(vec![true, true, true, true, true, true])),
            Arc::new(StringArray::from(vec![
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
            ])),
            Arc::new(StringArray::from(vec![
                None::<&str>,
                None::<&str>,
                None::<&str>,
                None::<&str>,
                None::<&str>,
                None::<&str>,
            ])),
            Arc::new(StringArray::from(vec![
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
                Some("Demo"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("module"),
                Some("module"),
                Some("module"),
                Some("module"),
                Some("module"),
                Some("module"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("export"),
                Some("import"),
                Some("symbol"),
                Some("symbol"),
                Some("docstring"),
                Some("include"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("solve"),
                None,
                Some("solve"),
                Some("LIMIT"),
                Some("solve"),
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                Some("function"),
                Some("binding"),
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                Some("solve(problem::Problem)"),
                Some("const LIMIT = 1"),
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                None,
                Some("function"),
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                None,
                Some("solve"),
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                None,
                Some("Demo.solve"),
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                None,
                None,
                Some(5),
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                None,
                None,
                Some(7),
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("using"),
                None,
                None,
                None,
                Some("include"),
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("aliased_member"),
                None,
                None,
                None,
                Some("include"),
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("..Core.solve"),
                None,
                None,
                None,
                Some("solvers.jl"),
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                Some(true),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                Some(2),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("solver"),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("..Core"),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("solve"),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("solver"),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                None,
                Some("Solve docs."),
                None,
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                Some(true),
                None,
                None,
                None,
                None,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                None,
                None,
                Some("solvers.jl"),
            ])),
            Arc::new(StringArray::from(vec![
                None,
                None,
                None,
                Some("const"),
                None,
                None,
            ])),
            Arc::new(BooleanArray::from(vec![
                None,
                None,
                Some(true),
                Some(true),
                None,
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                Some(5),
                Some(3),
                None,
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                Some(7),
                Some(3),
                None,
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                Some(1),
                None,
                None,
                None,
            ])),
            Arc::new(Int32Array::from(vec![
                None,
                None,
                Some(0),
                None,
                None,
                None,
            ])),
        ],
    )
    .unwrap_or_else(|error| panic!("sample response batch should build: {error}"))
}

fn sample_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, DataType::Utf8, false),
        Field::new(JULIA_PARSER_SUMMARY_KIND_COLUMN, DataType::Utf8, false),
        Field::new(JULIA_PARSER_SUMMARY_BACKEND_COLUMN, DataType::Utf8, false),
        Field::new(
            JULIA_PARSER_SUMMARY_SUCCESS_COLUMN,
            DataType::Boolean,
            false,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_PRIMARY_NAME_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ERROR_MESSAGE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_MODULE_NAME_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_MODULE_KIND_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(JULIA_PARSER_SUMMARY_ITEM_GROUP_COLUMN, DataType::Utf8, true),
        Field::new(JULIA_PARSER_SUMMARY_ITEM_NAME_COLUMN, DataType::Utf8, true),
        Field::new(JULIA_PARSER_SUMMARY_ITEM_KIND_COLUMN, DataType::Utf8, true),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_SIGNATURE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_KIND_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_NAME_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_PATH_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_START_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TARGET_LINE_END_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_KIND_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_FORM_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_TARGET_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_IS_RELATIVE_COLUMN,
            DataType::Boolean,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_RELATIVE_LEVEL_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_LOCAL_NAME_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_PARENT_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_MEMBER_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_DEPENDENCY_ALIAS_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_CONTENT_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_REEXPORTED_COLUMN,
            DataType::Boolean,
            true,
        ),
        Field::new(JULIA_PARSER_SUMMARY_ITEM_PATH_COLUMN, DataType::Utf8, true),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_BINDING_KIND_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_TOP_LEVEL_COLUMN,
            DataType::Boolean,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_LINE_START_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_LINE_END_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_FUNCTION_POSITIONAL_ARITY_COLUMN,
            DataType::Int32,
            true,
        ),
        Field::new(
            JULIA_PARSER_SUMMARY_ITEM_FUNCTION_KEYWORD_ARITY_COLUMN,
            DataType::Int32,
            true,
        ),
    ]))
}
