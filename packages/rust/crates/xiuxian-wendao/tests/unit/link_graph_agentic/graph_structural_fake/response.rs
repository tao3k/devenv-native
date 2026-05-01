use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Float64Array, Int32Array, ListArray, ListBuilder, StringArray,
    StringBuilder, UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use tonic::Status;
use xiuxian_wendao_julia::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FILTER_ROUTE,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
    GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN,
    GRAPH_STRUCTURAL_RERANK_ROUTE, GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION, JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE, JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
};

const JULIA_PLUGIN_ID: &str = "xiuxian-wendao-julia";
const JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID: &str = "plugin-capabilities";
const JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID: &str = "graph-structural";
const ARROW_FLIGHT_TRANSPORT_KIND: &str = "arrow_flight";
const DEFAULT_HEALTH_ROUTE: &str = "/healthz";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

pub(super) fn response_batch(
    base_url: &str,
    requests: &[RecordBatch],
) -> Result<RecordBatch, Status> {
    let Some(first) = requests.first() else {
        return Err(Status::invalid_argument(
            "graph-structural fake request stream returned no batches",
        ));
    };
    if first
        .column_by_name(JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN)
        .is_some()
    {
        return manifest_response_batch(base_url);
    }
    if first
        .column_by_name(GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN)
        .is_some()
    {
        return filter_response_batch(requests);
    }
    rerank_response_batch(requests)
}

fn manifest_response_batch(base_url: &str) -> Result<RecordBatch, Status> {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
                false,
            ),
            utf8_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN, false),
            utf8_field(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
                true,
            ),
            utf8_field(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
                false,
            ),
            utf8_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN, false),
            utf8_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN, false),
            utf8_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN, true),
            utf8_field(
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
                false,
            ),
            u64_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN, true),
            bool_field(JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec![
                JULIA_PLUGIN_ID,
                JULIA_PLUGIN_ID,
                JULIA_PLUGIN_ID,
            ])),
            Arc::new(StringArray::from(vec![
                JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID,
                JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID,
                JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID,
            ])),
            Arc::new(StringArray::from(vec![
                None,
                Some("structural_rerank"),
                Some("constraint_filter"),
            ])),
            Arc::new(StringArray::from(vec![
                ARROW_FLIGHT_TRANSPORT_KIND,
                ARROW_FLIGHT_TRANSPORT_KIND,
                ARROW_FLIGHT_TRANSPORT_KIND,
            ])),
            Arc::new(StringArray::from(vec![base_url, base_url, base_url])),
            Arc::new(StringArray::from(vec![
                JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE,
                GRAPH_STRUCTURAL_RERANK_ROUTE,
                GRAPH_STRUCTURAL_FILTER_ROUTE,
            ])),
            Arc::new(StringArray::from(vec![
                Some(DEFAULT_HEALTH_ROUTE),
                Some(DEFAULT_HEALTH_ROUTE),
                Some(DEFAULT_HEALTH_ROUTE),
            ])),
            Arc::new(StringArray::from(vec![
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
                JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
                JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
            ])),
            Arc::new(UInt64Array::from(vec![
                Some(DEFAULT_TIMEOUT_SECS),
                Some(DEFAULT_TIMEOUT_SECS),
                Some(DEFAULT_TIMEOUT_SECS),
            ])),
            Arc::new(BooleanArray::from(vec![true, true, true])),
        ],
    )
    .map_err(|error| Status::internal(error.to_string()))
}

fn rerank_response_batch(requests: &[RecordBatch]) -> Result<RecordBatch, Status> {
    let mut rows = Vec::new();
    for batch in requests {
        let candidate_ids = string_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN)?;
        let candidate_node_ids = list_column(batch, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN)?;
        for row_index in 0..batch.num_rows() {
            let candidate_id = candidate_ids.value(row_index).to_string();
            let nodes = string_list_values(candidate_node_ids, row_index)?;
            let edge_count = edge_count(batch, row_index)?;
            let structural_score =
                0.45 + count_score(nodes.len(), 0.01)? + count_score(edge_count, 0.02)?;
            rows.push(RerankRow {
                candidate_id,
                structural_score,
                final_score: structural_score + 0.25,
                pin_assignment: first_pin(nodes.as_slice(), 1),
                explanation: format!(
                    "solver_demo feasible candidate via rydberg solve with {} nodes, {edge_count} explicit edges",
                    nodes.len()
                ),
            });
        }
    }
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, false),
            bool_field(GRAPH_STRUCTURAL_FEASIBLE_COLUMN, false),
            f64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, false),
            f64_field(GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, false),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, false),
            utf8_field(GRAPH_STRUCTURAL_EXPLANATION_COLUMN, false),
        ])),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.candidate_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(vec![true; rows.len()])),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.structural_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.final_score).collect::<Vec<_>>(),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.pin_assignment.as_slice()),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.explanation.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| Status::internal(error.to_string()))
}

fn filter_response_batch(requests: &[RecordBatch]) -> Result<RecordBatch, Status> {
    let mut rows = Vec::new();
    for batch in requests {
        let candidate_ids = string_column(batch, GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN)?;
        let required_boundary_sizes =
            int32_column(batch, GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN)?;
        let candidate_node_ids = list_column(batch, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN)?;
        for row_index in 0..batch.num_rows() {
            let nodes = string_list_values(candidate_node_ids, row_index)?;
            let required_boundary_size = usize::try_from(required_boundary_sizes.value(row_index))
                .map_err(|error| Status::invalid_argument(error.to_string()))?;
            rows.push(FilterRow {
                candidate_id: candidate_ids.value(row_index).to_string(),
                structural_score: 0.5 + count_score(nodes.len(), 0.01)?,
                pin_assignment: first_pin(nodes.as_slice(), required_boundary_size),
            });
        }
    }
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, false),
            bool_field(GRAPH_STRUCTURAL_ACCEPTED_COLUMN, false),
            f64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, false),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, false),
            utf8_field(GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, false),
        ])),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.candidate_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(vec![true; rows.len()])),
            Arc::new(Float64Array::from(
                rows.iter()
                    .map(|row| row.structural_score)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(build_utf8_list_array(
                rows.iter().map(|row| row.pin_assignment.as_slice()),
            )),
            Arc::new(StringArray::from(vec![""; rows.len()])),
        ],
    )
    .map_err(|error| Status::internal(error.to_string()))
}

struct RerankRow {
    candidate_id: String,
    structural_score: f64,
    final_score: f64,
    pin_assignment: Vec<String>,
    explanation: String,
}

struct FilterRow {
    candidate_id: String,
    structural_score: f64,
    pin_assignment: Vec<String>,
}

fn first_pin(nodes: &[String], size: usize) -> Vec<String> {
    nodes.iter().take(size.max(1)).cloned().collect::<Vec<_>>()
}

fn edge_count(batch: &RecordBatch, row_index: usize) -> Result<usize, Status> {
    let Some(column) = batch.column_by_name("candidate_edge_sources") else {
        return Ok(0);
    };
    let edges = column
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| Status::invalid_argument("candidate_edge_sources must be List<Utf8>"))?;
    usize::try_from(edges.value_length(row_index))
        .map_err(|error| Status::invalid_argument(error.to_string()))
}

fn count_score(count: usize, factor: f64) -> Result<f64, Status> {
    let count = u32::try_from(count)
        .map_err(|error| Status::invalid_argument(format!("count exceeds u32: {error}")))?;
    Ok(f64::from(count) * factor)
}

fn string_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a StringArray, Status> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| Status::invalid_argument(format!("missing Utf8 column `{column_name}`")))
}

fn int32_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a Int32Array, Status> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .ok_or_else(|| Status::invalid_argument(format!("missing Int32 column `{column_name}`")))
}

fn list_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a ListArray, Status> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| {
            Status::invalid_argument(format!("missing List<Utf8> column `{column_name}`"))
        })
}

fn string_list_values(column: &ListArray, row_index: usize) -> Result<Vec<String>, Status> {
    let values = column.value(row_index);
    let values = values
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Status::invalid_argument("list values must be Utf8"))?;
    Ok((0..values.len())
        .map(|index| values.value(index).to_string())
        .collect())
}

fn build_utf8_list_array<'a>(rows: impl IntoIterator<Item = &'a [String]>) -> ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for row in rows {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}

fn utf8_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

fn bool_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Boolean, nullable)
}

fn f64_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::Float64, nullable)
}

fn u64_field(name: &str, nullable: bool) -> Field {
    Field::new(name, DataType::UInt64, nullable)
}

fn list_utf8_field(name: &str, nullable: bool) -> Field {
    Field::new(
        name,
        DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
        nullable,
    )
}
