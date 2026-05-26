//! Schema and batch validators for the `WendaoSearch.jl` graph-structural contract.

use std::collections::{BTreeSet, HashMap};

use arrow::array::{Array, BooleanArray, Float64Array, Int32Array, ListArray, StringArray};
use arrow::datatypes::{DataType, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, build_arrow_schema,
    validate_schema_against_contract,
};

use super::columns::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FILTER_REQUEST_INT32_COLUMNS,
    GRAPH_STRUCTURAL_FILTER_REQUEST_LIST_UTF8_COLUMNS,
    GRAPH_STRUCTURAL_FILTER_REQUEST_UTF8_COLUMNS, GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
    GRAPH_STRUCTURAL_QUERY_ID_COLUMN, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
    GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
    GRAPH_STRUCTURAL_RERANK_REQUEST_FLOAT64_COLUMNS, GRAPH_STRUCTURAL_RERANK_REQUEST_INT32_COLUMNS,
    GRAPH_STRUCTURAL_RERANK_REQUEST_LIST_UTF8_COLUMNS,
    GRAPH_STRUCTURAL_RERANK_REQUEST_UTF8_COLUMNS, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
    GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
};

/// Build the staged structural-rerank request Arrow schema.
#[must_use]
pub fn graph_structural_rerank_request_schema() -> Schema {
    build_arrow_schema(&graph_structural_rerank_request_contract(), HashMap::new())
}

/// Validate the staged structural-rerank request schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged structural-rerank
/// request contract.
pub fn validate_graph_structural_rerank_request_schema(schema: &Schema) -> Result<(), String> {
    validate_schema_against_contract(schema, &graph_structural_rerank_request_contract())
        .map_err(|error| format!("graph structural rerank request {error}"))?;
    validate_utf8_fields(schema, GRAPH_STRUCTURAL_RERANK_REQUEST_UTF8_COLUMNS)?;
    validate_int32_fields(schema, GRAPH_STRUCTURAL_RERANK_REQUEST_INT32_COLUMNS)?;
    validate_float64_fields(schema, GRAPH_STRUCTURAL_RERANK_REQUEST_FLOAT64_COLUMNS)?;
    validate_list_utf8_fields(schema, GRAPH_STRUCTURAL_RERANK_REQUEST_LIST_UTF8_COLUMNS)?;
    Ok(())
}

/// Validate one staged structural-rerank request batch.
///
/// # Errors
///
/// Returns an error when the batch does not satisfy the staged structural-rerank
/// request semantics.
pub fn validate_graph_structural_rerank_request_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_graph_structural_rerank_request_schema(batch.schema().as_ref())?;
    require_non_empty_graph_structural_batch(batch, "graph structural rerank request")?;
    validate_graph_structural_rerank_request_scalar_columns(batch)?;
    validate_graph_structural_request_list_columns(batch, "graph structural rerank request")
}

fn validate_graph_structural_rerank_request_scalar_columns(
    batch: &RecordBatch,
) -> Result<(), String> {
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_QUERY_ID_COLUMN, false)?;
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, true)?;
    require_int32_column(batch, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, 0)?;
    require_int32_column(batch, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, 1)?;
    require_non_negative_float64_column(batch, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN)?;
    require_non_negative_float64_column(batch, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN)?;
    require_non_negative_float64_column(batch, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN)?;
    require_non_negative_float64_column(batch, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN)?;
    Ok(())
}

fn validate_graph_structural_request_list_columns(
    batch: &RecordBatch,
    request_label: &str,
) -> Result<(), String> {
    let anchor_planes =
        require_utf8_list_column(batch, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN, false)?;
    let anchor_values =
        require_utf8_list_column(batch, GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, false)?;
    require_utf8_list_column(batch, GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, true)?;
    let candidate_node_ids =
        require_utf8_list_column(batch, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, false)?;
    let candidate_edge_sources =
        require_utf8_list_column(batch, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN, true)?;
    let candidate_edge_destinations = require_utf8_list_column(
        batch,
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
        true,
    )?;
    let candidate_edge_kinds =
        require_utf8_list_column(batch, GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, true)?;
    for row_index in 0..batch.num_rows() {
        if anchor_planes[row_index].len() != anchor_values[row_index].len() {
            return Err(format!(
                "{request_label} anchor columns must stay aligned; row {row_index} has {} planes but {} values",
                anchor_planes[row_index].len(),
                anchor_values[row_index].len(),
            ));
        }
        validate_candidate_edge_lists(
            request_label,
            row_index,
            &candidate_node_ids[row_index],
            &candidate_edge_sources[row_index],
            &candidate_edge_destinations[row_index],
            &candidate_edge_kinds[row_index],
        )?;
    }
    Ok(())
}

/// Validate the staged structural-rerank response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged structural-rerank
/// response contract.
pub fn validate_graph_structural_rerank_response_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN)?;
    validate_bool_field(schema, GRAPH_STRUCTURAL_FEASIBLE_COLUMN)?;
    validate_float64_field(schema, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN)?;
    validate_float64_field(schema, GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN)?;
    validate_list_utf8_field(schema, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN)?;
    validate_utf8_field(schema, GRAPH_STRUCTURAL_EXPLANATION_COLUMN)?;
    Ok(())
}

/// Validate one staged structural-rerank response batch.
///
/// # Errors
///
/// Returns an error when the batch does not satisfy the staged structural-rerank
/// response semantics.
pub fn validate_graph_structural_rerank_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_graph_structural_rerank_response_schema(batch.schema().as_ref())?;
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, true)?;
    require_bool_column(batch, GRAPH_STRUCTURAL_FEASIBLE_COLUMN)?;
    require_finite_float64_column(batch, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN)?;
    require_finite_float64_column(batch, GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN)?;
    require_utf8_list_column(batch, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, true)?;
    require_utf8_column(batch, GRAPH_STRUCTURAL_EXPLANATION_COLUMN, true)?;
    Ok(())
}

/// Validate the staged constraint-filter request schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged constraint-filter
/// request contract.
pub fn validate_graph_structural_filter_request_schema(schema: &Schema) -> Result<(), String> {
    validate_schema_against_contract(schema, &graph_structural_filter_request_contract())
        .map_err(|error| format!("graph structural filter request {error}"))?;
    validate_utf8_fields(schema, GRAPH_STRUCTURAL_FILTER_REQUEST_UTF8_COLUMNS)?;
    validate_int32_fields(schema, GRAPH_STRUCTURAL_FILTER_REQUEST_INT32_COLUMNS)?;
    validate_list_utf8_fields(schema, GRAPH_STRUCTURAL_FILTER_REQUEST_LIST_UTF8_COLUMNS)?;
    Ok(())
}

/// Build the staged constraint-filter request Arrow schema.
#[must_use]
pub fn graph_structural_filter_request_schema() -> Schema {
    build_arrow_schema(&graph_structural_filter_request_contract(), HashMap::new())
}

/// Validate one staged constraint-filter request batch.
///
/// # Errors
///
/// Returns an error when the batch does not satisfy the staged constraint-filter
/// request semantics.
pub fn validate_graph_structural_filter_request_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_graph_structural_filter_request_schema(batch.schema().as_ref())?;
    require_non_empty_graph_structural_batch(batch, "graph structural filter request")?;
    validate_graph_structural_filter_request_scalar_columns(batch)?;
    validate_graph_structural_request_list_columns(batch, "graph structural filter request")
}

fn validate_graph_structural_filter_request_scalar_columns(
    batch: &RecordBatch,
) -> Result<(), String> {
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_QUERY_ID_COLUMN, false)?;
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, true)?;
    require_int32_column(batch, GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, 0)?;
    require_int32_column(batch, GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, 1)?;
    require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, false)?;
    require_int32_column(batch, GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, 0)?;
    Ok(())
}

fn graph_structural_rerank_request_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "graph_structural_rerank_request",
        true,
        vec![
            column(GRAPH_STRUCTURAL_QUERY_ID_COLUMN, ArrowSchemaDataType::Utf8),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            column(
                GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
                ArrowSchemaDataType::Int32,
            ),
            column(
                GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
                ArrowSchemaDataType::Int32,
            ),
            column(
                GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
                ArrowSchemaDataType::Float64,
            ),
            column(
                GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
                ArrowSchemaDataType::Float64,
            ),
            column(
                GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
                ArrowSchemaDataType::Float64,
            ),
            column(
                GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
                ArrowSchemaDataType::Float64,
            ),
            column(
                GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
        ],
    )
}

fn graph_structural_filter_request_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        "graph_structural_filter_request",
        true,
        vec![
            column(GRAPH_STRUCTURAL_QUERY_ID_COLUMN, ArrowSchemaDataType::Utf8),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            column(
                GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN,
                ArrowSchemaDataType::Int32,
            ),
            column(
                GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN,
                ArrowSchemaDataType::Int32,
            ),
            column(
                GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN,
                ArrowSchemaDataType::Utf8,
            ),
            column(
                GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
                ArrowSchemaDataType::Int32,
            ),
            column(
                GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
            column(
                GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
                ArrowSchemaDataType::Utf8List,
            ),
        ],
    )
}

fn column(name: &'static str, data_type: ArrowSchemaDataType) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, data_type)
}

/// Validate the staged constraint-filter response schema.
///
/// # Errors
///
/// Returns an error when the schema does not match the staged constraint-filter
/// response contract.
pub fn validate_graph_structural_filter_response_schema(schema: &Schema) -> Result<(), String> {
    validate_utf8_field(schema, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN)?;
    validate_bool_field(schema, GRAPH_STRUCTURAL_ACCEPTED_COLUMN)?;
    validate_float64_field(schema, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN)?;
    validate_list_utf8_field(schema, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN)?;
    validate_utf8_field(schema, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN)?;
    Ok(())
}

/// Validate one staged constraint-filter response batch.
///
/// # Errors
///
/// Returns an error when the batch does not satisfy the staged constraint-filter
/// response semantics.
pub fn validate_graph_structural_filter_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_graph_structural_filter_response_schema(batch.schema().as_ref())?;
    let candidate_ids =
        require_non_blank_utf8_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, true)?;
    let accepted = require_bool_column(batch, GRAPH_STRUCTURAL_ACCEPTED_COLUMN)?;
    require_non_negative_float64_column(batch, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN)?;
    require_utf8_list_column(batch, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, true)?;
    let rejection_reason =
        require_utf8_column(batch, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, true)?;
    for row_index in 0..batch.num_rows() {
        if accepted.value(row_index) {
            if !rejection_reason.value(row_index).trim().is_empty() {
                return Err(format!(
                    "graph structural filter response column `{GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN}` must be blank for accepted candidate `{}` at row {row_index}",
                    candidate_ids.value(row_index),
                ));
            }
        } else if rejection_reason.value(row_index).trim().is_empty() {
            return Err(format!(
                "graph structural filter response column `{GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN}` must be non-blank for rejected candidate `{}` at row {row_index}",
                candidate_ids.value(row_index),
            ));
        }
    }
    Ok(())
}

fn validate_utf8_field(schema: &Schema, field_name: &str) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing graph structural column `{field_name}`"))?;
    if field.data_type() != &DataType::Utf8 {
        return Err(format!(
            "graph structural column `{field_name}` must be Utf8"
        ));
    }
    Ok(())
}

fn validate_utf8_fields(schema: &Schema, field_names: &[&str]) -> Result<(), String> {
    for field_name in field_names {
        validate_utf8_field(schema, field_name)?;
    }
    Ok(())
}

fn validate_bool_field(schema: &Schema, field_name: &str) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing graph structural column `{field_name}`"))?;
    if field.data_type() != &DataType::Boolean {
        return Err(format!(
            "graph structural column `{field_name}` must be Boolean"
        ));
    }
    Ok(())
}

fn validate_int32_field(schema: &Schema, field_name: &str) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing graph structural column `{field_name}`"))?;
    if field.data_type() != &DataType::Int32 {
        return Err(format!(
            "graph structural column `{field_name}` must be Int32"
        ));
    }
    Ok(())
}

fn validate_int32_fields(schema: &Schema, field_names: &[&str]) -> Result<(), String> {
    for field_name in field_names {
        validate_int32_field(schema, field_name)?;
    }
    Ok(())
}

fn validate_float64_field(schema: &Schema, field_name: &str) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing graph structural column `{field_name}`"))?;
    if field.data_type() != &DataType::Float64 {
        return Err(format!(
            "graph structural column `{field_name}` must be Float64"
        ));
    }
    Ok(())
}

fn validate_float64_fields(schema: &Schema, field_names: &[&str]) -> Result<(), String> {
    for field_name in field_names {
        validate_float64_field(schema, field_name)?;
    }
    Ok(())
}

fn validate_list_utf8_field(schema: &Schema, field_name: &str) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing graph structural column `{field_name}`"))?;
    match field.data_type() {
        DataType::List(inner) if inner.data_type() == &DataType::Utf8 => Ok(()),
        _ => Err(format!(
            "graph structural column `{field_name}` must be List<Utf8>"
        )),
    }
}

fn validate_list_utf8_fields(schema: &Schema, field_names: &[&str]) -> Result<(), String> {
    for field_name in field_names {
        validate_list_utf8_field(schema, field_name)?;
    }
    Ok(())
}

fn require_non_empty_graph_structural_batch(
    batch: &RecordBatch,
    label: &str,
) -> Result<(), String> {
    if batch.num_rows() == 0 {
        return Err(format!("{label} batch must contain at least one row"));
    }
    Ok(())
}

fn require_non_blank_utf8_column<'a>(
    batch: &'a RecordBatch,
    field_name: &str,
    unique: bool,
) -> Result<&'a StringArray, String> {
    let column = utf8_column(batch, field_name)?;
    let mut seen = BTreeSet::new();
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
        let value = column.value(row_index).trim();
        if value.is_empty() {
            return Err(format!(
                "graph structural column `{field_name}` must not contain blank values; row {row_index} is blank"
            ));
        }
        if unique && !seen.insert(value.to_string()) {
            return Err(format!(
                "graph structural column `{field_name}` must be unique across one batch; row {row_index} duplicates `{value}`"
            ));
        }
    }
    Ok(column)
}

fn require_utf8_column<'a>(
    batch: &'a RecordBatch,
    field_name: &str,
    allow_blank: bool,
) -> Result<&'a StringArray, String> {
    let column = utf8_column(batch, field_name)?;
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
        if !allow_blank && column.value(row_index).trim().is_empty() {
            return Err(format!(
                "graph structural column `{field_name}` must not contain blank values; row {row_index} is blank"
            ));
        }
    }
    Ok(column)
}

fn require_bool_column<'a>(
    batch: &'a RecordBatch,
    field_name: &str,
) -> Result<&'a BooleanArray, String> {
    let column = bool_column(batch, field_name)?;
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
    }
    Ok(column)
}

fn require_int32_column(
    batch: &RecordBatch,
    field_name: &str,
    min_value: i32,
) -> Result<(), String> {
    let column = int32_column(batch, field_name)?;
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
        let value = column.value(row_index);
        if value < min_value {
            return Err(format!(
                "graph structural column `{field_name}` must be greater than or equal to {min_value}; row {row_index} is {value}"
            ));
        }
    }
    Ok(())
}

fn require_non_negative_float64_column(
    batch: &RecordBatch,
    field_name: &str,
) -> Result<(), String> {
    let column = float64_column(batch, field_name)?;
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
        let value = column.value(row_index);
        if !value.is_finite() {
            return Err(format!(
                "graph structural column `{field_name}` must contain finite values; row {row_index} is {value}"
            ));
        }
        if value < 0.0 {
            return Err(format!(
                "graph structural column `{field_name}` must be greater than or equal to 0.0; row {row_index} is {value}"
            ));
        }
    }
    Ok(())
}

fn require_finite_float64_column(batch: &RecordBatch, field_name: &str) -> Result<(), String> {
    let column = float64_column(batch, field_name)?;
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null values; row {row_index} is null"
            ));
        }
        let value = column.value(row_index);
        if !value.is_finite() {
            return Err(format!(
                "graph structural column `{field_name}` must contain finite values; row {row_index} is {value}"
            ));
        }
    }
    Ok(())
}

fn require_utf8_list_column(
    batch: &RecordBatch,
    field_name: &str,
    allow_empty_lists: bool,
) -> Result<Vec<Vec<String>>, String> {
    let column = list_utf8_column(batch, field_name)?;
    let mut rows = Vec::with_capacity(batch.num_rows());
    for row_index in 0..batch.num_rows() {
        if column.is_null(row_index) {
            return Err(format!(
                "graph structural column `{field_name}` must not contain null lists; row {row_index} is null"
            ));
        }
        let values = column.value(row_index);
        let strings = values
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                format!("graph structural column `{field_name}` must decode as List<Utf8>")
            })?;
        let mut items = Vec::with_capacity(strings.len());
        for value_index in 0..strings.len() {
            if strings.is_null(value_index) {
                return Err(format!(
                    "graph structural column `{field_name}` must not contain null string values; row {row_index} item {value_index} is null"
                ));
            }
            let value = strings.value(value_index).trim();
            if value.is_empty() {
                return Err(format!(
                    "graph structural column `{field_name}` must not contain blank string values; row {row_index} item {value_index} is blank"
                ));
            }
            items.push(value.to_string());
        }
        if !allow_empty_lists && items.is_empty() {
            return Err(format!(
                "graph structural column `{field_name}` must contain at least one item; row {row_index} is empty"
            ));
        }
        rows.push(items);
    }
    Ok(rows)
}

fn validate_candidate_edge_lists(
    subject: &str,
    row_index: usize,
    node_ids: &[String],
    edge_sources: &[String],
    edge_destinations: &[String],
    edge_kinds: &[String],
) -> Result<(), String> {
    if edge_sources.len() != edge_destinations.len() {
        return Err(format!(
            "{subject} edge endpoint columns must stay aligned; row {row_index} has {} sources but {} destinations",
            edge_sources.len(),
            edge_destinations.len(),
        ));
    }
    if edge_sources.len() != edge_kinds.len() {
        return Err(format!(
            "{subject} edge columns must align with edge kinds; row {row_index} has {} edge endpoints but {} edge kinds",
            edge_sources.len(),
            edge_kinds.len(),
        ));
    }
    let node_ids = node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for (edge_index, (src_id, dst_id)) in edge_sources
        .iter()
        .zip(edge_destinations.iter())
        .enumerate()
    {
        if src_id == dst_id {
            return Err(format!(
                "{subject} edge endpoints must not be identical; row {row_index} edge {edge_index} repeats `{src_id}`",
            ));
        }
        if !node_ids.contains(src_id.as_str()) {
            return Err(format!(
                "{subject} edge source `{src_id}` is not present in candidate nodes at row {row_index}",
            ));
        }
        if !node_ids.contains(dst_id.as_str()) {
            return Err(format!(
                "{subject} edge destination `{dst_id}` is not present in candidate nodes at row {row_index}",
            ));
        }
    }
    Ok(())
}

fn utf8_column<'a>(batch: &'a RecordBatch, field_name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(field_name)
        .ok_or_else(|| format!("missing graph structural column `{field_name}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("graph structural column `{field_name}` must decode as Utf8"))
}

fn bool_column<'a>(batch: &'a RecordBatch, field_name: &str) -> Result<&'a BooleanArray, String> {
    batch
        .column_by_name(field_name)
        .ok_or_else(|| format!("missing graph structural column `{field_name}`"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("graph structural column `{field_name}` must decode as Boolean"))
}

fn int32_column<'a>(batch: &'a RecordBatch, field_name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(field_name)
        .ok_or_else(|| format!("missing graph structural column `{field_name}`"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("graph structural column `{field_name}` must decode as Int32"))
}

fn float64_column<'a>(
    batch: &'a RecordBatch,
    field_name: &str,
) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(field_name)
        .ok_or_else(|| format!("missing graph structural column `{field_name}`"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("graph structural column `{field_name}` must decode as Float64"))
}

fn list_utf8_column<'a>(batch: &'a RecordBatch, field_name: &str) -> Result<&'a ListArray, String> {
    batch
        .column_by_name(field_name)
        .ok_or_else(|| format!("missing graph structural column `{field_name}`"))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| format!("graph structural column `{field_name}` must decode as List<Utf8>"))
}
