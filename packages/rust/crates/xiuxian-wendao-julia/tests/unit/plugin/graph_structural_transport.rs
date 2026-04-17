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
#[test]
fn build_graph_structural_flight_transport_client_returns_none_without_config() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("missing graph-structural config should be ignored: {error}"));
    assert!(client.is_none());
}

#[test]
fn build_graph_structural_flight_transport_client_reads_common_options() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "health_route": "/ready",
                    "timeout_secs": 25,
                    "max_in_flight_requests": 4
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("graph-structural config should parse: {error}"))
    .unwrap_or_else(|| panic!("graph-structural client should exist"));

    assert_eq!(client.flight_base_url(), "http://127.0.0.1:9101");
    assert_eq!(client.flight_route(), GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(
        client.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );
    let binding = build_graph_structural_flight_transport_binding(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("graph-structural binding should parse: {error}"))
    .unwrap_or_else(|| panic!("graph-structural binding should exist"));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(4));
}

#[test]
fn build_graph_structural_flight_transport_client_reads_route_specific_overrides() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "structural_rerank": {
                        "route": "graph/structural/rerank",
                        "schema_version": "v0-custom",
                        "timeout_secs": 30
                    },
                    "constraint_filter": {
                        "route": "/graph/structural/filter",
                        "timeout_secs": 12
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let rerank_client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::StructuralRerank,
    )
    .unwrap_or_else(|error| panic!("rerank config should parse: {error}"))
    .unwrap_or_else(|| panic!("rerank client should exist"));
    let filter_client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("filter config should parse: {error}"))
    .unwrap_or_else(|| panic!("filter client should exist"));

    assert_eq!(rerank_client.flight_route(), GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(filter_client.flight_route(), GRAPH_STRUCTURAL_FILTER_ROUTE);
}

#[test]
fn build_graph_structural_flight_transport_client_honors_enabled_false() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": "http://127.0.0.1:9101",
                    "constraint_filter": {
                        "enabled": false
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let client = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    )
    .unwrap_or_else(|error| panic!("disabled route-specific config should parse: {error}"));
    assert!(client.is_none());
}

#[test]
fn build_graph_structural_flight_transport_client_rejects_invalid_field_types() {
    let repository = RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "constraint_filter": {
                        "timeout_secs": "fast"
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    };

    let Err(error) = build_graph_structural_flight_transport_client(
        &repository,
        GraphStructuralRouteKind::ConstraintFilter,
    ) else {
        panic!("invalid timeout type must fail");
    };
    assert!(
        error
            .to_string()
            .contains("Julia plugin field `timeout_secs` must be an unsigned integer"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_graph_structural_request_batches_accepts_staged_shapes() {
    let rerank = structural_rerank_request_batch();
    let filter = constraint_filter_request_batch();

    assert!(
        validate_graph_structural_request_batches(
            GraphStructuralRouteKind::StructuralRerank,
            &[rerank]
        )
        .is_ok()
    );
    assert!(
        validate_graph_structural_request_batches(
            GraphStructuralRouteKind::ConstraintFilter,
            &[filter]
        )
        .is_ok()
    );
}

#[test]
fn validate_graph_structural_response_batches_accepts_staged_shapes() {
    let rerank = structural_rerank_response_batch();
    let filter = constraint_filter_response_batch();

    assert!(
        validate_graph_structural_response_batches(
            GraphStructuralRouteKind::StructuralRerank,
            &[rerank]
        )
        .is_ok()
    );
    assert!(
        validate_graph_structural_response_batches(
            GraphStructuralRouteKind::ConstraintFilter,
            &[filter]
        )
        .is_ok()
    );
}

#[test]
fn validate_graph_structural_response_batches_rejects_wrong_shape() {
    let error = validate_graph_structural_response_batches(
        GraphStructuralRouteKind::ConstraintFilter,
        &[structural_rerank_response_batch()],
    )
    .err_or_panic("wrong graph-structural response shape must fail");
    assert!(
        error
            .to_string()
            .contains("Julia graph-structural response contract"),
        "unexpected error: {error}"
    );
}

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
