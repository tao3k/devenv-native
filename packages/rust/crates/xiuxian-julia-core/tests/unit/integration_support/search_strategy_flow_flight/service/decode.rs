use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::BinaryArray;
use arrow::record_batch::RecordBatch;

use super::fixtures::{
    arrow_ipc_stream, frontier_response_batch, frontier_response_batch_with_rank_float,
};
use crate::integration_support::{
    decode_search_strategy_flow_frontier_rows, search_strategy_flow_response_bundle_schema,
};

#[test]
fn search_strategy_flow_frontier_decoder_maps_arrow_rows() {
    let batch = frontier_response_batch();

    let rows = must(
        decode_search_strategy_flow_frontier_rows(&[batch]),
        "frontier response should decode",
    );

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].flow_id, "flight-service-flow");
    assert_eq!(
        rows[0].candidate_id,
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary"
    );
    assert_eq!(rows[0].rank, 1);
    assert!(rows[0].selected);
    assert_eq!(rows[0].judgement_kind, "authority");
    assert!(!rows[1].selected);
}

#[test]
fn search_strategy_flow_frontier_decoder_rejects_schema_drift() {
    let batch = frontier_response_batch_with_rank_float();

    let error = must_err(decode_search_strategy_flow_frontier_rows(&[batch]));

    assert!(
        error.contains("column `rank` must be Int64"),
        "unexpected error: {error}"
    );
}

#[test]
fn search_strategy_flow_frontier_decoder_accepts_response_bundle() {
    let frontier_batch = frontier_response_batch();
    let frontier_payload = arrow_ipc_stream(&frontier_batch);
    let bundle_batch = response_bundle_batch(frontier_payload.as_slice());

    let rows = must(
        decode_search_strategy_flow_frontier_rows(&[bundle_batch]),
        "frontier response bundle should decode",
    );

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].flow_id, "flight-service-flow");
    assert_eq!(rows[0].judgement_kind, "authority");
    assert!(rows[0].selected);
    assert!(!rows[1].selected);
}

#[test]
fn search_strategy_flow_frontier_decoder_rejects_bundled_payload_schema_drift() {
    let frontier_batch = frontier_response_batch_with_rank_float();
    let frontier_payload = arrow_ipc_stream(&frontier_batch);
    let bundle_batch = response_bundle_batch(frontier_payload.as_slice());

    let error = must_err(decode_search_strategy_flow_frontier_rows(&[bundle_batch]));

    assert!(
        error.contains("column `rank` must be Int64"),
        "unexpected error: {error}"
    );
}

#[test]
fn search_strategy_flow_response_decoder_uses_contract_owned_payload_routing() {
    let decoder_source =
        read_crate_source("src/integration_support/search_strategy_flow_flight/service/decode.rs");
    let contract_source =
        read_crate_source("src/integration_support/search_strategy_flow_flight/contract.rs");

    assert!(
        decoder_source.contains("search_strategy_flow_response_candidates_payload_column"),
        "decoder should consume response payload columns through contract accessors"
    );
    assert!(
        !decoder_source
            .contains("WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN"),
        "decoder must not own response payload column constants"
    );
    assert!(
        contract_source
            .contains("WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN"),
        "contract should own response payload column constants"
    );
}

fn response_bundle_batch(frontier_payload: &[u8]) -> RecordBatch {
    let schema = Arc::new(search_strategy_flow_response_bundle_schema(
        [(
            "wendao.table".to_string(),
            "search_strategy_flow_response".to_string(),
        )]
        .into_iter()
        .collect(),
    ));
    must(
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(BinaryArray::from(vec![frontier_payload])),
                Arc::new(BinaryArray::from(vec![frontier_payload])),
                Arc::new(BinaryArray::from(vec![frontier_payload])),
                Arc::new(BinaryArray::from(vec![frontier_payload])),
            ],
        ),
        "response bundle should build",
    )
}

fn read_crate_source(relative_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
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
