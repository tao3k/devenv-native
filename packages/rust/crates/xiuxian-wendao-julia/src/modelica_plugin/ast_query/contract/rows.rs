//! Modelica AST-query response row decoding.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::values::{
    ast_query_contract_error, optional_bool_values, optional_int_values, optional_utf8_values,
    required_bool_values, required_utf8_values,
};
use super::{
    MODELICA_AST_QUERY_BACKEND_COLUMN, MODELICA_AST_QUERY_ERROR_MESSAGE_COLUMN,
    MODELICA_AST_QUERY_MATCH_ARRAY_DIMENSIONS_COLUMN, MODELICA_AST_QUERY_MATCH_CLASS_PATH_COLUMN,
    MODELICA_AST_QUERY_MATCH_COMPONENT_KIND_COLUMN, MODELICA_AST_QUERY_MATCH_COUNT_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEFAULT_VALUE_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_ALIAS_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_FORM_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_KIND_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_LOCAL_NAME_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_MEMBER_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_PARENT_COLUMN,
    MODELICA_AST_QUERY_MATCH_DEPENDENCY_TARGET_COLUMN, MODELICA_AST_QUERY_MATCH_DIRECTION_COLUMN,
    MODELICA_AST_QUERY_MATCH_INDEX_COLUMN, MODELICA_AST_QUERY_MATCH_IS_ENCAPSULATED_COLUMN,
    MODELICA_AST_QUERY_MATCH_IS_FINAL_COLUMN, MODELICA_AST_QUERY_MATCH_IS_PARTIAL_COLUMN,
    MODELICA_AST_QUERY_MATCH_LINE_END_COLUMN, MODELICA_AST_QUERY_MATCH_LINE_START_COLUMN,
    MODELICA_AST_QUERY_MATCH_MODIFIER_NAMES_COLUMN, MODELICA_AST_QUERY_MATCH_NAME_COLUMN,
    MODELICA_AST_QUERY_MATCH_NODE_KIND_COLUMN, MODELICA_AST_QUERY_MATCH_OWNER_NAME_COLUMN,
    MODELICA_AST_QUERY_MATCH_OWNER_PATH_COLUMN, MODELICA_AST_QUERY_MATCH_PATH_COLUMN,
    MODELICA_AST_QUERY_MATCH_SIGNATURE_COLUMN, MODELICA_AST_QUERY_MATCH_START_VALUE_COLUMN,
    MODELICA_AST_QUERY_MATCH_TEXT_COLUMN, MODELICA_AST_QUERY_MATCH_TOP_LEVEL_COLUMN,
    MODELICA_AST_QUERY_MATCH_TYPE_NAME_COLUMN, MODELICA_AST_QUERY_MATCH_UNIT_COLUMN,
    MODELICA_AST_QUERY_MATCH_VARIABILITY_COLUMN, MODELICA_AST_QUERY_MATCH_VISIBILITY_COLUMN,
    MODELICA_AST_QUERY_PRIMARY_NAME_COLUMN, MODELICA_AST_QUERY_REQUEST_ID_COLUMN,
    MODELICA_AST_QUERY_SOURCE_ID_COLUMN, MODELICA_AST_QUERY_SUCCESS_COLUMN,
    MODELICA_AST_QUERY_SUMMARY_KIND_COLUMN, ModelicaAstQueryResponseRow,
};

pub(crate) fn decode_modelica_ast_query_response_rows(
    batches: &[RecordBatch],
) -> Result<Vec<ModelicaAstQueryResponseRow>, RepoIntelligenceError> {
    if batches.is_empty() {
        return Err(ast_query_contract_error(
            "response",
            "ast-query response stream returned no record batches",
        ));
    }

    let mut rows = Vec::new();
    for batch in batches {
        rows.extend(decode_modelica_ast_query_response_batch_rows(batch)?);
    }

    Ok(rows)
}

struct ModelicaAstQueryResponseBatchCoreColumns {
    request_id: Vec<String>,
    source_id: Vec<String>,
    summary_kind: Vec<String>,
    backend: Vec<String>,
    success: Vec<bool>,
    primary_name: Vec<Option<String>>,
    match_count: Vec<Option<i64>>,
    error_message: Vec<Option<String>>,
}

struct ModelicaAstQueryResponseBatchNodeColumns {
    index: Vec<Option<i64>>,
    node_kind: Vec<Option<String>>,
    name: Vec<Option<String>>,
    text: Vec<Option<String>>,
    signature: Vec<Option<String>>,
    path: Vec<Option<String>>,
    line_start: Vec<Option<i64>>,
    line_end: Vec<Option<i64>>,
}

struct ModelicaAstQueryResponseBatchDependencyColumns {
    kind: Vec<Option<String>>,
    form: Vec<Option<String>>,
    target: Vec<Option<String>>,
    local_name: Vec<Option<String>>,
    parent: Vec<Option<String>>,
    member: Vec<Option<String>>,
    alias: Vec<Option<String>>,
}

struct ModelicaAstQueryResponseBatchAttributeColumns {
    owner_name: Vec<Option<String>>,
    owner_path: Vec<Option<String>>,
    class_path: Vec<Option<String>>,
    top_level: Vec<Option<bool>>,
    visibility: Vec<Option<String>>,
    type_name: Vec<Option<String>>,
    variability: Vec<Option<String>>,
    direction: Vec<Option<String>>,
    component_kind: Vec<Option<String>>,
    array_dimensions: Vec<Option<String>>,
    default_value: Vec<Option<String>>,
    start_value: Vec<Option<String>>,
    modifier_names: Vec<Option<String>>,
    unit: Vec<Option<String>>,
    is_partial: Vec<Option<bool>>,
    is_final: Vec<Option<bool>>,
    is_encapsulated: Vec<Option<bool>>,
}

struct ModelicaAstQueryResponseBatchMatchColumns {
    node: ModelicaAstQueryResponseBatchNodeColumns,
    dependency: ModelicaAstQueryResponseBatchDependencyColumns,
    attributes: ModelicaAstQueryResponseBatchAttributeColumns,
}

fn decode_modelica_ast_query_response_batch_rows(
    batch: &RecordBatch,
) -> Result<Vec<ModelicaAstQueryResponseRow>, RepoIntelligenceError> {
    let core = decode_modelica_ast_query_response_batch_core_columns(batch)?;
    let matches = decode_modelica_ast_query_response_batch_match_columns(batch)?;
    Ok((0..batch.num_rows())
        .map(|index| ModelicaAstQueryResponseRow {
            request_id: core.request_id[index].clone(),
            source_id: core.source_id[index].clone(),
            summary_kind: core.summary_kind[index].clone(),
            backend: core.backend[index].clone(),
            success: core.success[index],
            primary_name: core.primary_name[index].clone(),
            match_count: core.match_count[index],
            error_message: core.error_message[index].clone(),
            match_index: matches.node.index[index],
            match_node_kind: matches.node.node_kind[index].clone(),
            match_name: matches.node.name[index].clone(),
            match_text: matches.node.text[index].clone(),
            match_signature: matches.node.signature[index].clone(),
            match_path: matches.node.path[index].clone(),
            match_dependency_kind: matches.dependency.kind[index].clone(),
            match_dependency_form: matches.dependency.form[index].clone(),
            match_dependency_target: matches.dependency.target[index].clone(),
            match_dependency_local_name: matches.dependency.local_name[index].clone(),
            match_dependency_parent: matches.dependency.parent[index].clone(),
            match_dependency_member: matches.dependency.member[index].clone(),
            match_dependency_alias: matches.dependency.alias[index].clone(),
            match_line_start: matches.node.line_start[index],
            match_line_end: matches.node.line_end[index],
            match_owner_name: matches.attributes.owner_name[index].clone(),
            match_owner_path: matches.attributes.owner_path[index].clone(),
            match_class_path: matches.attributes.class_path[index].clone(),
            match_top_level: matches.attributes.top_level[index],
            match_visibility: matches.attributes.visibility[index].clone(),
            match_type_name: matches.attributes.type_name[index].clone(),
            match_variability: matches.attributes.variability[index].clone(),
            match_direction: matches.attributes.direction[index].clone(),
            match_component_kind: matches.attributes.component_kind[index].clone(),
            match_array_dimensions: matches.attributes.array_dimensions[index].clone(),
            match_default_value: matches.attributes.default_value[index].clone(),
            match_start_value: matches.attributes.start_value[index].clone(),
            match_modifier_names: matches.attributes.modifier_names[index].clone(),
            match_unit: matches.attributes.unit[index].clone(),
            match_is_partial: matches.attributes.is_partial[index],
            match_is_final: matches.attributes.is_final[index],
            match_is_encapsulated: matches.attributes.is_encapsulated[index],
        })
        .collect())
}

fn decode_modelica_ast_query_response_batch_core_columns(
    batch: &RecordBatch,
) -> Result<ModelicaAstQueryResponseBatchCoreColumns, RepoIntelligenceError> {
    Ok(ModelicaAstQueryResponseBatchCoreColumns {
        request_id: required_utf8_values(batch, MODELICA_AST_QUERY_REQUEST_ID_COLUMN, "response")?,
        source_id: required_utf8_values(batch, MODELICA_AST_QUERY_SOURCE_ID_COLUMN, "response")?,
        summary_kind: required_utf8_values(
            batch,
            MODELICA_AST_QUERY_SUMMARY_KIND_COLUMN,
            "response",
        )?,
        backend: required_utf8_values(batch, MODELICA_AST_QUERY_BACKEND_COLUMN, "response")?,
        success: required_bool_values(batch, MODELICA_AST_QUERY_SUCCESS_COLUMN, "response")?,
        primary_name: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_PRIMARY_NAME_COLUMN,
            "response",
        )?,
        match_count: optional_int_values(batch, MODELICA_AST_QUERY_MATCH_COUNT_COLUMN, "response")?,
        error_message: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_ERROR_MESSAGE_COLUMN,
            "response",
        )?,
    })
}

fn decode_modelica_ast_query_response_batch_match_columns(
    batch: &RecordBatch,
) -> Result<ModelicaAstQueryResponseBatchMatchColumns, RepoIntelligenceError> {
    Ok(ModelicaAstQueryResponseBatchMatchColumns {
        node: decode_modelica_ast_query_response_batch_node_columns(batch)?,
        dependency: decode_modelica_ast_query_response_batch_dependency_columns(batch)?,
        attributes: decode_modelica_ast_query_response_batch_attribute_columns(batch)?,
    })
}

fn decode_modelica_ast_query_response_batch_node_columns(
    batch: &RecordBatch,
) -> Result<ModelicaAstQueryResponseBatchNodeColumns, RepoIntelligenceError> {
    Ok(ModelicaAstQueryResponseBatchNodeColumns {
        index: optional_int_values(batch, MODELICA_AST_QUERY_MATCH_INDEX_COLUMN, "response")?,
        node_kind: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_NODE_KIND_COLUMN,
            "response",
        )?,
        name: optional_utf8_values(batch, MODELICA_AST_QUERY_MATCH_NAME_COLUMN, "response")?,
        text: optional_utf8_values(batch, MODELICA_AST_QUERY_MATCH_TEXT_COLUMN, "response")?,
        signature: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_SIGNATURE_COLUMN,
            "response",
        )?,
        path: optional_utf8_values(batch, MODELICA_AST_QUERY_MATCH_PATH_COLUMN, "response")?,
        line_start: optional_int_values(
            batch,
            MODELICA_AST_QUERY_MATCH_LINE_START_COLUMN,
            "response",
        )?,
        line_end: optional_int_values(batch, MODELICA_AST_QUERY_MATCH_LINE_END_COLUMN, "response")?,
    })
}

fn decode_modelica_ast_query_response_batch_dependency_columns(
    batch: &RecordBatch,
) -> Result<ModelicaAstQueryResponseBatchDependencyColumns, RepoIntelligenceError> {
    Ok(ModelicaAstQueryResponseBatchDependencyColumns {
        kind: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_KIND_COLUMN,
            "response",
        )?,
        form: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_FORM_COLUMN,
            "response",
        )?,
        target: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_TARGET_COLUMN,
            "response",
        )?,
        local_name: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_LOCAL_NAME_COLUMN,
            "response",
        )?,
        parent: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_PARENT_COLUMN,
            "response",
        )?,
        member: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_MEMBER_COLUMN,
            "response",
        )?,
        alias: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEPENDENCY_ALIAS_COLUMN,
            "response",
        )?,
    })
}

fn decode_modelica_ast_query_response_batch_attribute_columns(
    batch: &RecordBatch,
) -> Result<ModelicaAstQueryResponseBatchAttributeColumns, RepoIntelligenceError> {
    Ok(ModelicaAstQueryResponseBatchAttributeColumns {
        owner_name: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_OWNER_NAME_COLUMN,
            "response",
        )?,
        owner_path: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_OWNER_PATH_COLUMN,
            "response",
        )?,
        class_path: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_CLASS_PATH_COLUMN,
            "response",
        )?,
        top_level: optional_bool_values(
            batch,
            MODELICA_AST_QUERY_MATCH_TOP_LEVEL_COLUMN,
            "response",
        )?,
        visibility: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_VISIBILITY_COLUMN,
            "response",
        )?,
        type_name: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_TYPE_NAME_COLUMN,
            "response",
        )?,
        variability: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_VARIABILITY_COLUMN,
            "response",
        )?,
        direction: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DIRECTION_COLUMN,
            "response",
        )?,
        component_kind: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_COMPONENT_KIND_COLUMN,
            "response",
        )?,
        array_dimensions: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_ARRAY_DIMENSIONS_COLUMN,
            "response",
        )?,
        default_value: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_DEFAULT_VALUE_COLUMN,
            "response",
        )?,
        start_value: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_START_VALUE_COLUMN,
            "response",
        )?,
        modifier_names: optional_utf8_values(
            batch,
            MODELICA_AST_QUERY_MATCH_MODIFIER_NAMES_COLUMN,
            "response",
        )?,
        unit: optional_utf8_values(batch, MODELICA_AST_QUERY_MATCH_UNIT_COLUMN, "response")?,
        is_partial: optional_bool_values(
            batch,
            MODELICA_AST_QUERY_MATCH_IS_PARTIAL_COLUMN,
            "response",
        )?,
        is_final: optional_bool_values(
            batch,
            MODELICA_AST_QUERY_MATCH_IS_FINAL_COLUMN,
            "response",
        )?,
        is_encapsulated: optional_bool_values(
            batch,
            MODELICA_AST_QUERY_MATCH_IS_ENCAPSULATED_COLUMN,
            "response",
        )?,
    })
}
