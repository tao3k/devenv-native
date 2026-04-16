use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Int32Array, Int64Array, LargeStringArray, NullArray, StringArray,
    StringViewArray,
};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::{
    ImportKind, ModuleRecord, RepoIntelligenceError, RepoSymbolKind, RepositoryAnalysisOutput,
    SymbolRecord,
};

const MODELICA_AST_QUERY_REQUEST_ID_COLUMN: &str = "request_id";
const MODELICA_AST_QUERY_SOURCE_ID_COLUMN: &str = "source_id";
const MODELICA_AST_QUERY_SOURCE_TEXT_COLUMN: &str = "source_text";
const MODELICA_AST_QUERY_NODE_KIND_COLUMN: &str = "node_kind";
const MODELICA_AST_QUERY_NAME_EQUALS_COLUMN: &str = "name_equals";
const MODELICA_AST_QUERY_NAME_CONTAINS_COLUMN: &str = "name_contains";
const MODELICA_AST_QUERY_TEXT_CONTAINS_COLUMN: &str = "text_contains";
const MODELICA_AST_QUERY_SIGNATURE_CONTAINS_COLUMN: &str = "signature_contains";
const MODELICA_AST_QUERY_ATTRIBUTE_KEY_COLUMN: &str = "attribute_key";
const MODELICA_AST_QUERY_ATTRIBUTE_EQUALS_COLUMN: &str = "attribute_equals";
const MODELICA_AST_QUERY_ATTRIBUTE_CONTAINS_COLUMN: &str = "attribute_contains";
const MODELICA_AST_QUERY_LIMIT_COLUMN: &str = "limit";

const MODELICA_AST_QUERY_SUMMARY_KIND_COLUMN: &str = "summary_kind";
const MODELICA_AST_QUERY_BACKEND_COLUMN: &str = "backend";
const MODELICA_AST_QUERY_SUCCESS_COLUMN: &str = "success";
const MODELICA_AST_QUERY_PRIMARY_NAME_COLUMN: &str = "primary_name";
const MODELICA_AST_QUERY_MATCH_COUNT_COLUMN: &str = "match_count";
const MODELICA_AST_QUERY_ERROR_MESSAGE_COLUMN: &str = "error_message";
const MODELICA_AST_QUERY_MATCH_INDEX_COLUMN: &str = "match_index";
const MODELICA_AST_QUERY_MATCH_NODE_KIND_COLUMN: &str = "match_node_kind";
const MODELICA_AST_QUERY_MATCH_NAME_COLUMN: &str = "match_name";
const MODELICA_AST_QUERY_MATCH_TEXT_COLUMN: &str = "match_text";
const MODELICA_AST_QUERY_MATCH_SIGNATURE_COLUMN: &str = "match_signature";
const MODELICA_AST_QUERY_MATCH_PATH_COLUMN: &str = "match_path";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_KIND_COLUMN: &str = "match_dependency_kind";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_FORM_COLUMN: &str = "match_dependency_form";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_TARGET_COLUMN: &str = "match_dependency_target";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_LOCAL_NAME_COLUMN: &str = "match_dependency_local_name";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_PARENT_COLUMN: &str = "match_dependency_parent";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_MEMBER_COLUMN: &str = "match_dependency_member";
const MODELICA_AST_QUERY_MATCH_DEPENDENCY_ALIAS_COLUMN: &str = "match_dependency_alias";
const MODELICA_AST_QUERY_MATCH_LINE_START_COLUMN: &str = "match_line_start";
const MODELICA_AST_QUERY_MATCH_LINE_END_COLUMN: &str = "match_line_end";
const MODELICA_AST_QUERY_MATCH_OWNER_NAME_COLUMN: &str = "match_owner_name";
const MODELICA_AST_QUERY_MATCH_OWNER_PATH_COLUMN: &str = "match_owner_path";
const MODELICA_AST_QUERY_MATCH_CLASS_PATH_COLUMN: &str = "match_class_path";
const MODELICA_AST_QUERY_MATCH_TOP_LEVEL_COLUMN: &str = "match_top_level";
const MODELICA_AST_QUERY_MATCH_VISIBILITY_COLUMN: &str = "match_visibility";
const MODELICA_AST_QUERY_MATCH_TYPE_NAME_COLUMN: &str = "match_type_name";
const MODELICA_AST_QUERY_MATCH_VARIABILITY_COLUMN: &str = "match_variability";
const MODELICA_AST_QUERY_MATCH_DIRECTION_COLUMN: &str = "match_direction";
const MODELICA_AST_QUERY_MATCH_COMPONENT_KIND_COLUMN: &str = "match_component_kind";
const MODELICA_AST_QUERY_MATCH_ARRAY_DIMENSIONS_COLUMN: &str = "match_array_dimensions";
const MODELICA_AST_QUERY_MATCH_DEFAULT_VALUE_COLUMN: &str = "match_default_value";
const MODELICA_AST_QUERY_MATCH_START_VALUE_COLUMN: &str = "match_start_value";
const MODELICA_AST_QUERY_MATCH_MODIFIER_NAMES_COLUMN: &str = "match_modifier_names";
const MODELICA_AST_QUERY_MATCH_UNIT_COLUMN: &str = "match_unit";
const MODELICA_AST_QUERY_MATCH_IS_PARTIAL_COLUMN: &str = "match_is_partial";
const MODELICA_AST_QUERY_MATCH_IS_FINAL_COLUMN: &str = "match_is_final";
const MODELICA_AST_QUERY_MATCH_IS_ENCAPSULATED_COLUMN: &str = "match_is_encapsulated";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaAstQueryRequest {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) source_text: String,
    pub(crate) limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelicaAstQueryResponseRow {
    pub(crate) request_id: String,
    pub(crate) source_id: String,
    pub(crate) summary_kind: String,
    pub(crate) backend: String,
    pub(crate) success: bool,
    pub(crate) primary_name: Option<String>,
    pub(crate) match_count: Option<i64>,
    pub(crate) error_message: Option<String>,
    pub(crate) match_index: Option<i64>,
    pub(crate) match_node_kind: Option<String>,
    pub(crate) match_name: Option<String>,
    pub(crate) match_text: Option<String>,
    pub(crate) match_signature: Option<String>,
    pub(crate) match_path: Option<String>,
    pub(crate) match_dependency_kind: Option<String>,
    pub(crate) match_dependency_form: Option<String>,
    pub(crate) match_dependency_target: Option<String>,
    pub(crate) match_dependency_local_name: Option<String>,
    pub(crate) match_dependency_parent: Option<String>,
    pub(crate) match_dependency_member: Option<String>,
    pub(crate) match_dependency_alias: Option<String>,
    pub(crate) match_line_start: Option<i64>,
    pub(crate) match_line_end: Option<i64>,
    pub(crate) match_owner_name: Option<String>,
    pub(crate) match_owner_path: Option<String>,
    pub(crate) match_class_path: Option<String>,
    pub(crate) match_top_level: Option<bool>,
    pub(crate) match_visibility: Option<String>,
    pub(crate) match_type_name: Option<String>,
    pub(crate) match_variability: Option<String>,
    pub(crate) match_direction: Option<String>,
    pub(crate) match_component_kind: Option<String>,
    pub(crate) match_array_dimensions: Option<String>,
    pub(crate) match_default_value: Option<String>,
    pub(crate) match_start_value: Option<String>,
    pub(crate) match_modifier_names: Option<String>,
    pub(crate) match_unit: Option<String>,
    pub(crate) match_is_partial: Option<bool>,
    pub(crate) match_is_final: Option<bool>,
    pub(crate) match_is_encapsulated: Option<bool>,
}

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

pub(crate) fn decode_modelica_ast_query_analysis(
    repo_id: &str,
    source_id: &str,
    rows: &[ModelicaAstQueryResponseRow],
) -> Result<RepositoryAnalysisOutput, RepoIntelligenceError> {
    let first_row = validate_modelica_ast_query_analysis_rows(rows)?;
    let (module_id, mut output) =
        initialize_modelica_ast_query_analysis_output(repo_id, source_id, first_row);
    let mut seen_symbols = BTreeSet::<(String, String, String)>::new();
    let mut seen_imports = BTreeSet::<(String, String, String, String)>::new();
    for row in rows {
        append_modelica_ast_query_analysis_row(
            repo_id,
            source_id,
            &module_id,
            row,
            &mut output,
            &mut seen_symbols,
            &mut seen_imports,
        )?;
    }

    Ok(output)
}

fn validate_modelica_ast_query_analysis_rows(
    rows: &[ModelicaAstQueryResponseRow],
) -> Result<&ModelicaAstQueryResponseRow, RepoIntelligenceError> {
    let Some(first_row) = rows.first() else {
        return Err(ast_query_contract_error(
            "response",
            "ast-query response contained no rows",
        ));
    };
    if first_row.summary_kind != "modelica_ast_query" {
        return Err(ast_query_contract_error(
            "response",
            format!(
                "expected `modelica_ast_query` summary kind, found `{}`",
                first_row.summary_kind
            ),
        ));
    }
    if first_row.backend != "OMParser.jl" {
        return Err(ast_query_contract_error(
            "response",
            format!(
                "expected `OMParser.jl` backend, found `{}`",
                first_row.backend
            ),
        ));
    }
    if rows.iter().any(|row| !row.success) {
        let message = rows
            .iter()
            .find_map(|row| row.error_message.clone())
            .unwrap_or_else(|| "unknown Modelica AST query failure".to_string());
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!("Modelica AST query request failed: {message}"),
        });
    }
    Ok(first_row)
}

fn initialize_modelica_ast_query_analysis_output(
    repo_id: &str,
    source_id: &str,
    first_row: &ModelicaAstQueryResponseRow,
) -> (String, RepositoryAnalysisOutput) {
    let primary_name = first_row
        .primary_name
        .clone()
        .or_else(|| {
            std::path::Path::new(source_id)
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "modelica".to_string());
    let module_id = format!("repo:{repo_id}:module:{primary_name}");
    let output = RepositoryAnalysisOutput {
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string(),
            module_id: module_id.clone(),
            qualified_name: primary_name,
            path: source_id.to_string(),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    (module_id, output)
}

fn append_modelica_ast_query_analysis_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_symbols: &mut BTreeSet<(String, String, String)>,
    seen_imports: &mut BTreeSet<(String, String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(node_kind) = row.match_node_kind.as_deref() else {
        return Ok(());
    };
    if matches!(node_kind, "import" | "extends") {
        append_modelica_ast_query_import_row(
            repo_id,
            source_id,
            module_id,
            row,
            output,
            seen_imports,
        )?;
        return Ok(());
    }
    append_modelica_ast_query_symbol_row(
        repo_id,
        source_id,
        module_id,
        node_kind,
        row,
        output,
        seen_symbols,
    )
}

fn append_modelica_ast_query_import_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_imports: &mut BTreeSet<(String, String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(import_record) = import_record_from_ast_row(repo_id, source_id, module_id, row)?
    else {
        return Ok(());
    };
    let key = (
        import_record.source_module.clone(),
        import_record.import_name.clone(),
        import_record.path.clone(),
        import_record
            .attributes
            .get("dependency_form")
            .cloned()
            .unwrap_or_default(),
    );
    if seen_imports.insert(key) {
        output.imports.push(import_record);
    }
    Ok(())
}

fn append_modelica_ast_query_symbol_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    node_kind: &str,
    row: &ModelicaAstQueryResponseRow,
    output: &mut RepositoryAnalysisOutput,
    seen_symbols: &mut BTreeSet<(String, String, String)>,
) -> Result<(), RepoIntelligenceError> {
    let Some(kind) = ast_row_symbol_kind(row) else {
        return Ok(());
    };
    let name = row.match_name.clone().unwrap_or_default();
    if name.trim().is_empty() {
        return Ok(());
    }
    let owner_path = row.match_owner_path.clone().unwrap_or_default();
    let symbol_key = (name.clone(), node_kind.to_string(), owner_path.clone());
    if !seen_symbols.insert(symbol_key) {
        return Ok(());
    }

    let qualified_name = ast_row_qualified_name(row, &name);
    output.symbols.push(SymbolRecord {
        repo_id: repo_id.to_string(),
        symbol_id: format!("repo:{repo_id}:symbol:{qualified_name}"),
        module_id: Some(module_id.to_string()),
        name,
        qualified_name,
        kind,
        path: source_id.to_string(),
        line_start: ast_line_number(row.match_line_start)?,
        line_end: ast_line_number(row.match_line_end)?,
        signature: row
            .match_signature
            .clone()
            .or_else(|| row.match_text.clone())
            .or_else(|| row.match_name.clone()),
        audit_status: None,
        verification_state: None,
        attributes: ast_row_symbol_attributes(row),
    });
    Ok(())
}

fn ast_row_symbol_kind(row: &ModelicaAstQueryResponseRow) -> Option<RepoSymbolKind> {
    match row.match_node_kind.as_deref() {
        Some("function") => Some(RepoSymbolKind::Function),
        Some(
            "package"
            | "model"
            | "record"
            | "block"
            | "connector"
            | "expandable_connector"
            | "type"
            | "enumeration"
            | "operator"
            | "operator_record"
            | "uniontype"
            | "metarecord"
            | "class",
        ) => Some(RepoSymbolKind::Type),
        Some("component") => match row.match_component_kind.as_deref() {
            Some("constant" | "parameter") => Some(RepoSymbolKind::Constant),
            _ => Some(RepoSymbolKind::Other),
        },
        _ => None,
    }
}

fn import_record_from_ast_row(
    repo_id: &str,
    source_id: &str,
    module_id: &str,
    row: &ModelicaAstQueryResponseRow,
) -> Result<Option<xiuxian_wendao_core::repo_intelligence::ImportRecord>, RepoIntelligenceError> {
    let source_module = row
        .match_dependency_target
        .clone()
        .or_else(|| row.match_name.clone())
        .unwrap_or_default();
    if source_module.trim().is_empty() {
        return Ok(None);
    }
    let import_name = row
        .match_dependency_local_name
        .clone()
        .or_else(|| row.match_dependency_alias.clone())
        .or_else(|| source_module.rsplit('.').next().map(str::to_string))
        .unwrap_or_else(|| source_module.clone());
    let target_package = source_module
        .split('.')
        .next()
        .unwrap_or(source_module.as_str())
        .to_string();

    let mut attributes = BTreeMap::new();
    insert_text_attribute(
        &mut attributes,
        "dependency_kind",
        row.match_dependency_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_form",
        row.match_dependency_form.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_target",
        row.match_dependency_target.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_alias",
        row.match_dependency_alias.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_local_name",
        row.match_dependency_local_name.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_parent",
        row.match_dependency_parent.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "dependency_member",
        row.match_dependency_member.as_ref(),
    );
    insert_text_attribute(&mut attributes, "owner_name", row.match_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.match_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "class_path", row.match_class_path.as_ref());

    Ok(Some(xiuxian_wendao_core::repo_intelligence::ImportRecord {
        repo_id: repo_id.to_string(),
        module_id: module_id.to_string(),
        path: source_id.to_string(),
        import_name,
        target_package,
        source_module,
        kind: ast_row_import_kind(row),
        line_start: ast_line_number(row.match_line_start)?,
        resolved_id: None,
        attributes,
    }))
}

fn ast_row_import_kind(row: &ModelicaAstQueryResponseRow) -> ImportKind {
    match row.match_dependency_form.as_deref() {
        Some("named_import" | "unqualified_import" | "group_import" | "extends") => {
            ImportKind::Module
        }
        _ => ImportKind::Symbol,
    }
}

fn ast_row_qualified_name(row: &ModelicaAstQueryResponseRow, name: &str) -> String {
    if let Some(class_path) = row
        .match_class_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if class_path == name {
            return class_path.to_string();
        }
        return format!("{class_path}.{name}");
    }
    if let Some(owner_path) = row
        .match_owner_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return format!("{owner_path}.{name}");
    }
    name.to_string()
}

fn ast_row_symbol_attributes(row: &ModelicaAstQueryResponseRow) -> BTreeMap<String, String> {
    let mut attributes = BTreeMap::new();
    insert_text_attribute(&mut attributes, "parser_kind", row.match_node_kind.as_ref());
    if let Some(restriction) = row.match_node_kind.as_deref().filter(|value| {
        matches!(
            *value,
            "package"
                | "model"
                | "record"
                | "block"
                | "connector"
                | "expandable_connector"
                | "type"
                | "enumeration"
                | "operator"
                | "operator_record"
                | "uniontype"
                | "metarecord"
                | "class"
        )
    }) {
        attributes.insert("restriction".to_string(), restriction.to_string());
    }
    insert_text_attribute(&mut attributes, "visibility", row.match_visibility.as_ref());
    insert_text_attribute(&mut attributes, "type_name", row.match_type_name.as_ref());
    insert_text_attribute(
        &mut attributes,
        "variability",
        row.match_variability.as_ref(),
    );
    insert_text_attribute(&mut attributes, "direction", row.match_direction.as_ref());
    insert_text_attribute(
        &mut attributes,
        "component_kind",
        row.match_component_kind.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "array_dimensions",
        row.match_array_dimensions.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "default_value",
        row.match_default_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "start_value",
        row.match_start_value.as_ref(),
    );
    insert_text_attribute(
        &mut attributes,
        "modifier_names",
        row.match_modifier_names.as_ref(),
    );
    insert_text_attribute(&mut attributes, "unit", row.match_unit.as_ref());
    insert_text_attribute(&mut attributes, "owner_name", row.match_owner_name.as_ref());
    insert_text_attribute(&mut attributes, "owner_path", row.match_owner_path.as_ref());
    insert_text_attribute(&mut attributes, "class_path", row.match_class_path.as_ref());
    insert_bool_attribute(&mut attributes, "top_level", row.match_top_level);
    insert_bool_attribute(&mut attributes, "is_partial", row.match_is_partial);
    insert_bool_attribute(&mut attributes, "is_final", row.match_is_final);
    insert_bool_attribute(
        &mut attributes,
        "is_encapsulated",
        row.match_is_encapsulated,
    );
    attributes
}

fn ast_line_number(value: Option<i64>) -> Result<Option<usize>, RepoIntelligenceError> {
    value
        .map(usize::try_from)
        .transpose()
        .map_err(|error| ast_query_contract_error("response", error.to_string()))
}

fn insert_text_attribute(
    attributes: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<&String>,
) {
    let Some(value) = value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    attributes.insert(key.to_string(), value.to_string());
}

fn insert_bool_attribute(
    attributes: &mut BTreeMap<String, String>,
    key: &str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        attributes.insert(key.to_string(), value.to_string());
    }
}

fn required_utf8_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<String>, RepoIntelligenceError> {
    let array = batch.column_by_name(column).ok_or_else(|| {
        ast_query_contract_error(stage, format!("missing required column `{column}`"))
    })?;
    let values = utf8_values(array, column, stage)?;
    if values.iter().any(Option::is_none) {
        return Err(ast_query_contract_error(
            stage,
            format!("required column `{column}` contains null rows"),
        ));
    }
    Ok(values
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect::<Vec<_>>())
}

fn optional_utf8_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    match batch.column_by_name(column) {
        Some(array) => utf8_values(array, column, stage),
        None => Ok(vec![None; batch.num_rows()]),
    }
}

fn utf8_values(
    array: &Arc<dyn Array>,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    if let Some(values) = array.as_any().downcast_ref::<StringArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<LargeStringArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<StringViewArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    Err(ast_query_contract_error(
        stage,
        format!(
            "column `{column}` expected Utf8-compatible values but found {:?}",
            array.data_type()
        ),
    ))
}

fn required_bool_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<bool>, RepoIntelligenceError> {
    let array = batch.column_by_name(column).ok_or_else(|| {
        ast_query_contract_error(stage, format!("missing required column `{column}`"))
    })?;
    let values = array
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| {
            ast_query_contract_error(
                stage,
                format!(
                    "column `{column}` expected Boolean values but found {:?}",
                    array.data_type()
                ),
            )
        })?;
    if (0..values.len()).any(|index| values.is_null(index)) {
        return Err(ast_query_contract_error(
            stage,
            format!("required column `{column}` contains null rows"),
        ));
    }
    Ok((0..values.len()).map(|index| values.value(index)).collect())
}

fn optional_bool_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<bool>>, RepoIntelligenceError> {
    let Some(array) = batch.column_by_name(column) else {
        return Ok(vec![None; batch.num_rows()]);
    };
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    let values = array
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| {
            ast_query_contract_error(
                stage,
                format!(
                    "column `{column}` expected Boolean values but found {:?}",
                    array.data_type()
                ),
            )
        })?;
    Ok((0..values.len())
        .map(|index| (!values.is_null(index)).then(|| values.value(index)))
        .collect())
}

fn optional_int_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<i64>>, RepoIntelligenceError> {
    let Some(array) = batch.column_by_name(column) else {
        return Ok(vec![None; batch.num_rows()]);
    };
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    if let Some(values) = array.as_any().downcast_ref::<Int32Array>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| i64::from(values.value(index))))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<Int64Array>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index)))
            .collect());
    }
    Err(ast_query_contract_error(
        stage,
        format!(
            "column `{column}` expected Int32 or Int64 values but found {:?}",
            array.data_type()
        ),
    ))
}

fn ast_query_request_error(message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to build Modelica ast-query request batch: {}",
            message.into()
        ),
    }
}

fn ast_query_contract_error(stage: &str, message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "invalid Modelica ast-query {stage} contract: {}",
            message.into()
        ),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/ast_query_contract.rs"]
mod contract_tests;
