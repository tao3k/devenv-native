use arrow::datatypes::DataType;

use super::{
    MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN, MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN,
    MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN, MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN,
    MODELICA_AST_QUERY_NAME_EQUALS_COLUMN, MODELICA_AST_QUERY_NODE_KIND_COLUMN,
    MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN, MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN,
    ModelicaAstQueryRequest, build_modelica_ast_query_request_batch,
};

#[test]
fn build_modelica_ast_query_request_batch_uses_null_columns_for_blank_filters() {
    let batch = build_modelica_ast_query_request_batch(&[ModelicaAstQueryRequest {
        request_id: "req-blank-filters".to_string(),
        source_id: "Modelica/Blocks/package.mo".to_string(),
        source_text: "within Modelica; package Blocks end Blocks;".to_string(),
        limit: Some(128),
    }])
    .expect("build ast-query request batch");

    for column_name in [
        MODELICA_AST_QUERY_NODE_KIND_COLUMN,
        MODELICA_AST_QUERY_NAME_EQUALS_COLUMN,
        MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN,
        MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN,
        MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN,
        MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN,
        MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN,
        MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN,
    ] {
        let column = batch
            .column_by_name(column_name)
            .unwrap_or_else(|| panic!("missing `{column_name}` column"));
        assert_eq!(
            column.data_type(),
            &DataType::Null,
            "expected `{column_name}` to use Null request encoding",
        );
    }
}
