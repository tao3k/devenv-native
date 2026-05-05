//! Modelica AST-query request batch contract.

use std::sync::Arc;

use arrow::array::{Array, Int64Array, NullArray, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::values::ast_query_request_error;
use super::{
    MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN, MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN,
    MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN, MODELICA_AST_QUERY_LIMIT_COLUMN,
    MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN, MODELICA_AST_QUERY_NAME_EQUALS_COLUMN,
    MODELICA_AST_QUERY_NODE_KIND_COLUMN, MODELICA_AST_QUERY_REQUEST_ID_COLUMN,
    MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN, MODELICA_AST_QUERY_SOURCE_ID_COLUMN,
    MODELICA_AST_QUERY_SOURCE_TEXT_COLUMN, MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN,
    ModelicaAstQueryRequest,
};

pub(crate) fn build_modelica_ast_query_request_batch(
    requests: &[ModelicaAstQueryRequest],
) -> Result<RecordBatch, RepoIntelligenceError> {
    if requests.is_empty() {
        return Err(ast_query_request_error(
            "request batches must contain at least one row",
        ));
    }

    let request_ids = requests
        .iter()
        .map(|request| request.request_id.as_str())
        .collect::<Vec<_>>();
    let source_ids = requests
        .iter()
        .map(|request| request.source_id.as_str())
        .collect::<Vec<_>>();
    let source_texts = requests
        .iter()
        .map(|request| request.source_text.as_str())
        .collect::<Vec<_>>();
    let limits = requests
        .iter()
        .map(|request| request.limit)
        .collect::<Vec<_>>();
    let empty_filters = vec![None::<&str>; requests.len()];

    RecordBatch::try_from_iter(vec![
        (
            MODELICA_AST_QUERY_REQUEST_ID_COLUMN,
            Arc::new(StringArray::from(request_ids)) as Arc<dyn Array>,
        ),
        (
            MODELICA_AST_QUERY_SOURCE_ID_COLUMN,
            Arc::new(StringArray::from(source_ids)) as Arc<dyn Array>,
        ),
        (
            MODELICA_AST_QUERY_SOURCE_TEXT_COLUMN,
            Arc::new(StringArray::from(source_texts)) as Arc<dyn Array>,
        ),
        (
            MODELICA_AST_QUERY_NODE_KIND_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_NAME_EQUALS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN,
            nullable_utf8_request_array(&empty_filters),
        ),
        (
            MODELICA_AST_QUERY_LIMIT_COLUMN,
            Arc::new(Int64Array::from(limits)) as Arc<dyn Array>,
        ),
    ])
    .map_err(|error| ast_query_request_error(error.to_string()))
}

fn nullable_utf8_request_array(values: &[Option<&str>]) -> Arc<dyn Array> {
    if values.iter().all(Option::is_none) {
        Arc::new(NullArray::new(values.len())) as Arc<dyn Array>
    } else {
        Arc::new(StringArray::from(values.to_vec())) as Arc<dyn Array>
    }
}
