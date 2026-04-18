use std::sync::Arc;

use arrow::array::{
    BooleanArray, Float64Array, Int32Array, ListArray, ListBuilder, StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::{
    repo_intelligence::{RegisteredRepository, RepositoryPluginConfig},
    transport::PluginTransportKind,
};
use xiuxian_wendao_runtime::transport::FLIGHT_SCHEMA_VERSION_METADATA_KEY;

use super::{
    build_graph_structural_flight_transport_binding,
    build_graph_structural_flight_transport_client, validate_graph_structural_request_batches,
    validate_graph_structural_response_batches,
};
use crate::plugin::graph_structural::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN,
    GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN,
    GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN, GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN,
    GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FILTER_ROUTE,
    GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN, GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
    GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN, GRAPH_STRUCTURAL_RERANK_ROUTE,
    GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, GRAPH_STRUCTURAL_TAG_SCORE_COLUMN,
    GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
};
use crate::plugin::test_support::common::ResultTestExt;
include!("graph_structural_transport/config.rs");
include!("graph_structural_transport/validation.rs");

fn structural_rerank_request_batch() -> RecordBatch {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_QUERY_ID_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            int32_field(GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN),
            int32_field(GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN),
            float64_field(GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_DEPENDENCY_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_KEYWORD_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_TAG_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["query-1"])),
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(Int32Array::from(vec![0])),
            Arc::new(Int32Array::from(vec![2])),
            Arc::new(Float64Array::from(vec![0.7])),
            Arc::new(Float64Array::from(vec![0.6])),
            Arc::new(Float64Array::from(vec![0.5])),
            Arc::new(Float64Array::from(vec![0.4])),
            Arc::new(list_utf8_array(vec![vec!["semantic", "dependency"]])),
            Arc::new(list_utf8_array(vec![vec!["symbol:foo", "symbol:bar"]])),
            Arc::new(list_utf8_array(vec![vec!["depends_on"]])),
            Arc::new(list_utf8_array(vec![vec!["n1", "n2"]])),
            Arc::new(list_utf8_array(vec![vec!["n1"]])),
            Arc::new(list_utf8_array(vec![vec!["n2"]])),
            Arc::new(list_utf8_array(vec![vec!["depends_on"]])),
        ],
    )
    .unwrap_or_else(|error| panic!("structural rerank request batch: {error}"));
    attach_schema_metadata(&batch)
}

fn constraint_filter_request_batch() -> RecordBatch {
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_QUERY_ID_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            int32_field(GRAPH_STRUCTURAL_RETRIEVAL_LAYER_COLUMN),
            int32_field(GRAPH_STRUCTURAL_QUERY_MAX_LAYERS_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_CONSTRAINT_KIND_COLUMN),
            int32_field(GRAPH_STRUCTURAL_REQUIRED_BOUNDARY_SIZE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_PLANES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_ANCHOR_VALUES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_EDGE_CONSTRAINT_KINDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_NODE_IDS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_SOURCES_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_DESTINATIONS_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["query-1"])),
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(Int32Array::from(vec![1])),
            Arc::new(Int32Array::from(vec![3])),
            Arc::new(StringArray::from(vec!["boundary-match"])),
            Arc::new(Int32Array::from(vec![2])),
            Arc::new(list_utf8_array(vec![vec!["semantic", "tag"]])),
            Arc::new(list_utf8_array(vec![vec!["symbol:foo", "tag:core"]])),
            Arc::new(list_utf8_array(vec![vec!["depends_on"]])),
            Arc::new(list_utf8_array(vec![vec!["n1", "n2"]])),
            Arc::new(list_utf8_array(vec![vec!["n1"]])),
            Arc::new(list_utf8_array(vec![vec!["n2"]])),
            Arc::new(list_utf8_array(vec![vec!["depends_on"]])),
        ],
    )
    .unwrap_or_else(|error| panic!("constraint filter request batch: {error}"));
    attach_schema_metadata(&batch)
}

fn structural_rerank_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            bool_field(GRAPH_STRUCTURAL_FEASIBLE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN),
            float64_field(GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_EXPLANATION_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(BooleanArray::from(vec![true])),
            Arc::new(Float64Array::from(vec![0.91])),
            Arc::new(Float64Array::from(vec![0.87])),
            Arc::new(list_utf8_array(vec![vec!["pin:entry", "pin:exit"]])),
            Arc::new(StringArray::from(vec!["structural rerank accepted"])),
        ],
    )
    .unwrap_or_else(|error| panic!("structural rerank response batch: {error}"))
}

fn constraint_filter_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            utf8_field(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN),
            bool_field(GRAPH_STRUCTURAL_ACCEPTED_COLUMN),
            float64_field(GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN),
            list_utf8_field(GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN),
            utf8_field(GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(BooleanArray::from(vec![true])),
            Arc::new(Float64Array::from(vec![0.73])),
            Arc::new(list_utf8_array(vec![vec!["pin:entry"]])),
            Arc::new(StringArray::from(vec![""])),
        ],
    )
    .unwrap_or_else(|error| panic!("constraint filter response batch: {error}"))
}

fn attach_schema_metadata(batch: &RecordBatch) -> RecordBatch {
    let metadata = std::collections::HashMap::from([(
        FLIGHT_SCHEMA_VERSION_METADATA_KEY.to_string(),
        JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION.to_string(),
    )]);
    let schema = Arc::new(batch.schema().as_ref().clone().with_metadata(metadata));
    RecordBatch::try_new(schema, batch.columns().to_vec())
        .unwrap_or_else(|error| panic!("schema metadata batch: {error}"))
}

fn utf8_field(name: &str) -> Field {
    Field::new(name, DataType::Utf8, false)
}

fn bool_field(name: &str) -> Field {
    Field::new(name, DataType::Boolean, false)
}

fn float64_field(name: &str) -> Field {
    Field::new(name, DataType::Float64, false)
}

fn int32_field(name: &str) -> Field {
    Field::new(name, DataType::Int32, false)
}

fn list_utf8_field(name: &str) -> Field {
    Field::new(
        name,
        DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
        false,
    )
}

fn list_utf8_array(values: Vec<Vec<&str>>) -> ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for row in values {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
