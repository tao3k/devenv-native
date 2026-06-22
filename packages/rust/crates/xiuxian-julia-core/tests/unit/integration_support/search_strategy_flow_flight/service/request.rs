use arrow::array::{Array, BinaryArray};

use super::fixtures::{
    branch_judgement_arrow_ipc, branch_judgement_arrow_ipc_without_reason, fixture_candidate_batch,
};
use crate::integration_support::{
    SearchStrategyFlowServiceRequestOptions, build_search_strategy_flow_service_arrow_request,
    build_search_strategy_flow_service_flight_request_batch,
};

#[test]
fn search_strategy_flow_service_request_bundle_wraps_candidate_and_optional_payloads() {
    let candidate_batch = fixture_candidate_batch();
    let branch_payload = branch_judgement_arrow_ipc(
        "flight-service-flow",
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
    );
    let request_options = SearchStrategyFlowServiceRequestOptions::default()
        .with_branch_judgements_arrow_ipc_stream(branch_payload.clone());

    let request = must(
        build_search_strategy_flow_service_arrow_request(&candidate_batch, request_options),
        "service request should build",
    );
    let request_batch = must(
        build_search_strategy_flow_service_flight_request_batch(&request),
        "request bundle should build",
    );

    assert_eq!(request.request_table, "search_strategy_flow_request");
    assert_eq!(
        request.schema_version,
        "xiuxian_wendao.graph.search_strategy_flow.service.v1"
    );
    assert!(request.payload_byte_size() > candidate_batch.candidate_input_arrow_ipc_byte_len());
    assert_eq!(request_batch.num_rows(), 1);
    assert_eq!(
        request_batch
            .schema()
            .metadata()
            .get("wendao.table")
            .map(String::as_str),
        Some("search_strategy_flow_request")
    );

    let query_payloads = request_batch
        .column_by_name("query_understanding_payload")
        .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
        .unwrap_or_else(|| panic!("query payload column should be binary"));
    let branch_payloads = request_batch
        .column_by_name("branch_judgements_payload")
        .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
        .unwrap_or_else(|| panic!("branch payload column should be binary"));

    assert!(query_payloads.is_null(0));
    assert_eq!(branch_payloads.value(0), branch_payload.as_slice());
}

#[test]
fn search_strategy_flow_service_request_rejects_side_table_schema_drift() {
    let candidate_batch = fixture_candidate_batch();
    let request_options = SearchStrategyFlowServiceRequestOptions::default()
        .with_branch_judgements_arrow_ipc_stream(branch_judgement_arrow_ipc_without_reason());

    let error = must_err(build_search_strategy_flow_service_arrow_request(
        &candidate_batch,
        request_options,
    ));

    assert!(
        error.contains("must have 8 columns"),
        "unexpected error: {error}"
    );
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected error, got {value:?}"),
        Err(error) => error,
    }
}
