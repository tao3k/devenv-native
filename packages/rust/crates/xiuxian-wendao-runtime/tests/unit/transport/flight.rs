use super::{ArrowFlightTransportClient, DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES};
use crate::transport::{
    REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_PATH_COLUMN,
    REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN, RERANK_ROUTE, WendaoFlightService,
};
use std::sync::Arc;
use std::time::Duration;

use arrow_flight::flight_service_server::FlightServiceServer;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use xiuxian_vector::{
    EngineRecordBatch, LanceDataType, LanceField, LanceFloat64Array,
    LanceFloat64Array as Float64Array, LanceInt32Array as Int32Array, LanceRecordBatch,
    LanceSchema, LanceStringArray as StringArray, engine_batches_to_lance_batches,
};

fn build_rerank_request_batch() -> EngineRecordBatch {
    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("doc_id", DataType::Utf8, false),
            Field::new("vector_score", DataType::Float32, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
                false,
            ),
            Field::new(
                "query_embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
                false,
            ),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["doc-0", "doc-1"])),
            Arc::new(Float32Array::from(vec![0.5_f32, 0.8_f32])),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    vec![
                        Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                        Some(vec![Some(0.0_f32), Some(1.0_f32), Some(0.0_f32)]),
                    ],
                    3,
                ),
            ),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    vec![
                        Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                        Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)]),
                    ],
                    3,
                ),
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("request batch should build: {error}"))
}

fn build_large_rerank_request_batch() -> EngineRecordBatch {
    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
    use arrow_schema::{DataType, Field, Schema};

    let large_doc_id = "doc-".to_string() + &"x".repeat(5 * 1024 * 1024);

    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("doc_id", DataType::Utf8, false),
            Field::new("vector_score", DataType::Float32, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
                false,
            ),
            Field::new(
                "query_embedding",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
                false,
            ),
        ])),
        vec![
            Arc::new(StringArray::from(vec![large_doc_id])),
            Arc::new(Float32Array::from(vec![0.5_f32])),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    vec![Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)])],
                    3,
                ),
            ),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    vec![Some(vec![Some(1.0_f32), Some(0.0_f32), Some(0.0_f32)])],
                    3,
                ),
            ),
        ],
    )
    .unwrap_or_else(|error| panic!("large request batch should build: {error}"))
}

#[tokio::test]
async fn flight_transport_client_roundtrips_batches_over_lance_arrow_line() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("listener should bind: {error}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("listener should expose a local address: {error}"));
    let query_response_batch = LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new(REPO_SEARCH_DOC_ID_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_PATH_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_TITLE_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_SCORE_COLUMN, LanceDataType::Float64, false),
            LanceField::new(REPO_SEARCH_LANGUAGE_COLUMN, LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["doc-1"])),
            Arc::new(StringArray::from(vec!["src/lib.rs"])),
            Arc::new(StringArray::from(vec!["Repo Search Result"])),
            Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
            Arc::new(StringArray::from(vec!["rust"])),
        ],
    )
    .unwrap_or_else(|error| panic!("query response batch should build: {error}"));
    let service = WendaoFlightService::new("v2", query_response_batch, 3)
        .unwrap_or_else(|error| panic!("runtime-owned Flight service should build: {error}"));
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(FlightServiceServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap_or_else(|error| panic!("mock Flight server should serve: {error}"));
    });

    let client = ArrowFlightTransportClient::new(
        format!("http://{address}"),
        RERANK_ROUTE,
        "v2",
        Duration::from_secs(5),
        32,
    )
    .unwrap_or_else(|error| panic!("flight client should build: {error}"));
    let request_batch = build_rerank_request_batch();
    let response_batches = client
        .process_batch(&request_batch)
        .await
        .unwrap_or_else(|error| panic!("flight roundtrip should succeed: {error}"));
    let lance_response_batches =
        engine_batches_to_lance_batches(&response_batches).unwrap_or_else(|error| {
            panic!("response batches should convert onto Lance Arrow: {error}")
        });

    assert_eq!(response_batches.len(), 1);
    assert_eq!(response_batches[0].num_rows(), 2);
    let doc_ids = lance_response_batches[0]
        .column_by_name("doc_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response doc_id column should decode as Utf8"));
    let vector_scores = lance_response_batches[0]
        .column_by_name("vector_score")
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .unwrap_or_else(|| panic!("response vector_score column should decode as Float64"));
    let semantic_scores = lance_response_batches[0]
        .column_by_name("semantic_score")
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .unwrap_or_else(|| panic!("response semantic_score column should decode as Float64"));
    let final_scores = lance_response_batches[0]
        .column_by_name("final_score")
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .unwrap_or_else(|| panic!("response final_score column should decode as Float64"));
    let ranks = lance_response_batches[0]
        .column_by_name("rank")
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .unwrap_or_else(|| panic!("response rank column should decode as Int32"));
    assert_eq!(doc_ids.value(0), "doc-0");
    assert_eq!(doc_ids.value(1), "doc-1");
    assert!((vector_scores.value(0) - 0.5).abs() < 1e-6);
    assert!((vector_scores.value(1) - 0.8).abs() < 1e-6);
    assert!((semantic_scores.value(0) - 1.0).abs() < 1e-6);
    assert!((semantic_scores.value(1) - 0.5).abs() < 1e-6);
    assert!((final_scores.value(0) - 0.8).abs() < 1e-6);
    assert!((final_scores.value(1) - 0.62).abs() < 1e-6);
    assert_eq!(ranks.value(0), 1);
    assert_eq!(ranks.value(1), 2);
    assert_eq!(client.base_url(), format!("http://{address}"));
    assert_eq!(client.route(), RERANK_ROUTE);
    assert_eq!(client.schema_version(), "v2");
    assert_eq!(client.timeout().as_secs(), 5);

    server.abort();
}

#[tokio::test]
async fn flight_transport_client_accepts_responses_larger_than_default_tonic_limit() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("listener should bind: {error}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("listener should expose a local address: {error}"));
    let query_response_batch = LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new(REPO_SEARCH_DOC_ID_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_PATH_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_TITLE_COLUMN, LanceDataType::Utf8, false),
            LanceField::new(REPO_SEARCH_SCORE_COLUMN, LanceDataType::Float64, false),
            LanceField::new(REPO_SEARCH_LANGUAGE_COLUMN, LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["doc-1"])),
            Arc::new(StringArray::from(vec!["src/lib.rs"])),
            Arc::new(StringArray::from(vec!["Repo Search Result"])),
            Arc::new(LanceFloat64Array::from(vec![0.91_f64])),
            Arc::new(StringArray::from(vec!["rust"])),
        ],
    )
    .unwrap_or_else(|error| panic!("query response batch should build: {error}"));
    let service = WendaoFlightService::new("v2", query_response_batch, 3)
        .unwrap_or_else(|error| panic!("runtime-owned Flight service should build: {error}"));
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(
                FlightServiceServer::new(service)
                    .max_decoding_message_size(DEFAULT_FLIGHT_MESSAGE_SIZE_BYTES),
            )
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap_or_else(|error| panic!("mock Flight server should serve: {error}"));
    });

    let client = ArrowFlightTransportClient::new(
        format!("http://{address}"),
        RERANK_ROUTE,
        "v2",
        Duration::from_secs(5),
        32,
    )
    .unwrap_or_else(|error| panic!("flight client should build: {error}"));
    let request_batch = build_large_rerank_request_batch();
    let response_batches = client
        .process_batch(&request_batch)
        .await
        .unwrap_or_else(|error| panic!("large flight roundtrip should succeed: {error}"));
    let lance_response_batches =
        engine_batches_to_lance_batches(&response_batches).unwrap_or_else(|error| {
            panic!("response batches should convert onto Lance Arrow: {error}")
        });

    let doc_ids = lance_response_batches[0]
        .column_by_name("doc_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response doc_id column should decode as Utf8"));
    assert!(doc_ids.value(0).len() > 4 * 1024 * 1024);

    server.abort();
}

#[tokio::test]
async fn flight_transport_client_gate_blocks_requests_above_budget() {
    let client = ArrowFlightTransportClient::new(
        "http://127.0.0.1:18815",
        RERANK_ROUTE,
        "v2",
        Duration::from_secs(5),
        1,
    )
    .unwrap_or_else(|error| panic!("flight client should build: {error}"));
    let permit = client
        .request_gate()
        .acquire_owned()
        .await
        .unwrap_or_else(|error| panic!("first gate permit should acquire: {error}"));

    let second_attempt = timeout(
        Duration::from_millis(50),
        client.request_gate().acquire_owned(),
    )
    .await;
    assert!(
        second_attempt.is_err(),
        "second permit should block at budget"
    );

    drop(permit);

    let _ = timeout(
        Duration::from_secs(1),
        client.request_gate().acquire_owned(),
    )
    .await
    .unwrap_or_else(|error| panic!("released permit should become available: {error}"))
    .unwrap_or_else(|error| panic!("second gate acquisition should succeed: {error}"));
}
