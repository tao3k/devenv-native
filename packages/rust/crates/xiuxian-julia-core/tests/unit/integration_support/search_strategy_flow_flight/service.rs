use std::env;
use std::io;
use std::io::Cursor;
use std::process::{Command, Stdio};
use std::sync::Arc;

use arrow::array::{Array, BinaryArray, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use xiuxian_polyglot_orchestrator::JuliaScheduleAction;
use xiuxian_wendao_core::transport::PluginTransportKind;

use super::{
    SearchStrategyFlowServiceFlightBindingOptions,
    build_search_strategy_flow_service_flight_binding, decode_search_strategy_flow_frontier_rows,
    roundtrip_search_strategy_flow_frontier_with_service,
};
use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowCandidateInputBatch,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};
use crate::integration_support::service_runtime::{
    JuliaServiceGuard, reserve_service_port, wait_for_service_ready_with_attempts,
};
use crate::integration_support::wendaograph::wendaograph_julia_project;
use crate::integration_support::{
    SearchStrategyFlowServiceRequestOptions, build_search_strategy_flow_service_arrow_request,
    build_search_strategy_flow_service_flight_request_batch,
    roundtrip_search_strategy_flow_frontier_with_service_request,
};

const RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_TEST";

#[test]
fn search_strategy_flow_service_binding_uses_arrow_route_and_orchestrator_admission() {
    let candidate_batch = fixture_candidate_batch();
    let options = SearchStrategyFlowServiceFlightBindingOptions::new("http://127.0.0.1:8815")
        .expect("base URL should parse")
        .with_max_in_flight_requests(4);

    let binding = build_search_strategy_flow_service_flight_binding(options, &candidate_batch)
        .expect("service binding should build");

    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/wendao/graph/search_strategy_flow")
    );
    assert_eq!(
        binding.contract_version.0,
        "xiuxian_wendao.graph.search_strategy_flow.service.v1"
    );
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(4));
    assert_eq!(binding.transport, PluginTransportKind::ArrowFlight);
    assert!(
        !candidate_batch.candidate_input_arrow_ipc_stream.is_empty(),
        "candidate batch should carry Arrow IPC"
    );
}

#[test]
fn search_strategy_flow_service_binding_rejects_empty_candidate_batch() {
    let mut candidate_batch = fixture_candidate_batch();
    candidate_batch.row_count = 0;
    let options = SearchStrategyFlowServiceFlightBindingOptions::new("http://127.0.0.1:8815")
        .expect("base URL should parse");

    let error = build_search_strategy_flow_service_flight_binding(options, &candidate_batch)
        .expect_err("empty candidate service request should be rejected");

    assert!(
        error.contains("must include candidate rows"),
        "unexpected error: {error}"
    );
}

#[test]
fn search_strategy_flow_frontier_decoder_maps_arrow_rows() {
    let batch = frontier_response_batch();

    let rows = decode_search_strategy_flow_frontier_rows(&[batch])
        .expect("frontier response should decode");

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
fn search_strategy_flow_frontier_decoder_accepts_response_bundle() {
    let frontier_batch = frontier_response_batch();
    let frontier_payload = arrow_ipc_stream(&frontier_batch);
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("strategy_candidates_payload", DataType::Binary, false),
            Field::new("strategy_transitions_payload", DataType::Binary, false),
            Field::new("strategy_frontier_payload", DataType::Binary, false),
            Field::new("strategy_planner_actions_payload", DataType::Binary, false),
        ],
        [(
            "wendao.table".to_string(),
            "search_strategy_flow_response".to_string(),
        )]
        .into_iter()
        .collect(),
    ));
    let bundle_batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(BinaryArray::from(vec![frontier_payload.as_slice()])),
            Arc::new(BinaryArray::from(vec![frontier_payload.as_slice()])),
            Arc::new(BinaryArray::from(vec![frontier_payload.as_slice()])),
            Arc::new(BinaryArray::from(vec![frontier_payload.as_slice()])),
        ],
    )
    .expect("response bundle should build");

    let rows = decode_search_strategy_flow_frontier_rows(&[bundle_batch])
        .expect("frontier response bundle should decode");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].flow_id, "flight-service-flow");
    assert_eq!(rows[0].judgement_kind, "authority");
    assert!(rows[0].selected);
    assert!(!rows[1].selected);
}

#[test]
fn search_strategy_flow_service_schedule_allows_candidate_batch() {
    let candidate_batch = fixture_candidate_batch();
    let options = SearchStrategyFlowServiceFlightBindingOptions::new("http://127.0.0.1:8815")
        .expect("base URL should parse");

    let plan = super::build_search_strategy_flow_service_orchestrator_schedule_plan(
        &options,
        candidate_batch.row_count,
        candidate_batch.candidate_input_arrow_ipc_byte_len(),
    );

    assert!(matches!(
        plan.action,
        JuliaScheduleAction::Dispatch | JuliaScheduleAction::Queue
    ));
    assert_eq!(plan.profile_id.as_str(), "wendaograph.search_strategy_flow");
}

#[test]
fn search_strategy_flow_service_request_bundle_wraps_candidate_and_optional_payloads() {
    let candidate_batch = fixture_candidate_batch();
    let branch_payload = branch_judgement_arrow_ipc(
        "flight-service-flow",
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
    );
    let request_options = SearchStrategyFlowServiceRequestOptions::default()
        .with_branch_judgements_arrow_ipc_stream(branch_payload.clone());

    let request =
        build_search_strategy_flow_service_arrow_request(&candidate_batch, request_options)
            .expect("service request should build");
    let request_batch = build_search_strategy_flow_service_flight_request_batch(&request)
        .expect("request bundle should build");

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
        .expect("query payload column should be binary");
    let branch_payloads = request_batch
        .column_by_name("branch_judgements_payload")
        .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
        .expect("branch payload column should be binary");

    assert!(query_payloads.is_null(0));
    assert_eq!(branch_payloads.value(0), branch_payload.as_slice());
}

#[tokio::test]
async fn search_strategy_flow_service_live_loopback_uses_real_wendaograph_arrow_flight()
-> io::Result<()> {
    if env::var_os(RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV).is_none() {
        eprintln!(
            "skipping live WendaoGraph SearchStrategyFlow service loopback; set {RUN_SEARCH_STRATEGY_FLOW_SERVICE_LIVE_LOOPBACK_ENV}=1"
        );
        return Ok(());
    }

    let candidate_batch = fixture_candidate_batch();
    let project = wendaograph_julia_project().map_err(io::Error::other)?;
    let runner = project
        .join("scripts")
        .join("run_search_strategy_flow_service.jl");
    if !runner.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing WendaoGraph SearchStrategyFlow runner `{}`",
                runner.display()
            ),
        ));
    }

    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut guard = JuliaServiceGuard::new(
        Command::new("julia")
            .arg(format!("--project={}", project.display()))
            .arg(&runner)
            .arg("--host=127.0.0.1")
            .arg(format!("--port={port}"))
            .arg("--flow-id=search-strategy-flow-rust-live")
            .arg("--max-active-requests=4")
            .arg("--request-capacity=4")
            .arg("--response-capacity=4")
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?,
    );

    wait_for_service_ready_with_attempts(&base_url, 300)
        .await
        .map_err(io::Error::other)?;
    let options = SearchStrategyFlowServiceFlightBindingOptions::new(base_url)
        .map_err(io::Error::other)?
        .with_max_in_flight_requests(1);
    let roundtrip =
        roundtrip_search_strategy_flow_frontier_with_service(&candidate_batch, options.clone())
            .await
            .map_err(io::Error::other)?;

    assert_eq!(roundtrip.flight_route, "/wendao/graph/search_strategy_flow");
    assert_eq!(roundtrip.rows.len(), 1);
    assert_eq!(roundtrip.rows[0].flow_id, "search-strategy-flow-rust-live");
    assert_eq!(
        roundtrip.rows[0].candidate_id,
        "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary"
    );
    assert!(roundtrip.rows[0].selected);

    let judged_request_options = SearchStrategyFlowServiceRequestOptions::default()
        .with_branch_judgements_arrow_ipc_stream(branch_judgement_arrow_ipc(
            "search-strategy-flow-rust-live",
            "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
        ));
    let judged_roundtrip = roundtrip_search_strategy_flow_frontier_with_service_request(
        &candidate_batch,
        judged_request_options,
        options,
    )
    .await
    .map_err(io::Error::other)?;

    assert_eq!(judged_roundtrip.rows.len(), 1);
    assert!(!judged_roundtrip.rows[0].selected);

    guard.kill();
    Ok(())
}

fn fixture_candidate_batch() -> SearchStrategyFlowCandidateInputBatch {
    let candidates = vec![SearchStrategyFlowCandidateInput {
        relative_path: "docs/30_search_strategy/30.01_search_strategy_flow.md".to_owned(),
        heading_anchor: "ownership-boundary".to_owned(),
        title: "Ownership Boundary".to_owned(),
        line_start: 10,
        line_end: 24,
        context_cost: 180,
        evidence_coverage: 0.94,
        graph_score: 0.91,
        authority_score: 0.95,
        structural_score: 0.9,
        uncertainty: 0.08,
        blocked: false,
        edge_kinds: vec!["authority".to_owned(), "anchor".to_owned()],
    }];
    search_strategy_flow_candidate_input_batch_with_discovery_receipt(
        "rust-code-intelligence-inventory",
        &candidates,
        &serde_json::json!({
            "receiptSource": "rust-code-intelligence-inventory",
            "candidateInputSource": "rust-code-intelligence-inventory",
            "candidateInputCount": candidates.len(),
            "transport": "unit-arrow-service",
            "route": "unit-arrow-service",
            "attemptCount": 1,
            "mergedCandidateCount": candidates.len()
        }),
    )
    .expect("candidate batch should build")
}

fn frontier_response_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("frontier_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("revision_id", DataType::Utf8, false),
        Field::new("rank", DataType::Int64, false),
        Field::new("selected", DataType::Boolean, false),
        Field::new("final_score", DataType::Float64, false),
        Field::new("action", DataType::Utf8, false),
        Field::new("context_budget", DataType::Int64, false),
        Field::new("judgement_kind", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![
                "flight-service-flow",
                "flight-service-flow",
            ])),
            Arc::new(StringArray::from(vec![
                "flight-service-flow-frontier-1",
                "flight-service-flow-frontier-2",
            ])),
            Arc::new(StringArray::from(vec![
                "docs/30_search_strategy/30.01_search_strategy_flow.md#ownership-boundary",
                "docs/90_validation/90.01_validation.md#promotion-boundary",
            ])),
            Arc::new(StringArray::from(vec!["revision-1", "revision-2"])),
            Arc::new(Int64Array::from(vec![1, 2])),
            Arc::new(BooleanArray::from(vec![true, false])),
            Arc::new(Float64Array::from(vec![0.95, 0.12])),
            Arc::new(StringArray::from(vec!["keep", "prune"])),
            Arc::new(Int64Array::from(vec![180, 0])),
            Arc::new(StringArray::from(vec!["authority", "not_selected"])),
        ],
    )
    .expect("frontier batch should build")
}

fn branch_judgement_arrow_ipc(flow_id: &str, candidate_id: &str) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flow_id", DataType::Utf8, false),
        Field::new("candidate_id", DataType::Utf8, false),
        Field::new("branch_role", DataType::Utf8, false),
        Field::new("judgement_score", DataType::Float64, false),
        Field::new("confidence", DataType::Float64, false),
        Field::new("decision", DataType::Utf8, false),
        Field::new("blocked", DataType::Boolean, false),
        Field::new("reason", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![flow_id])),
            Arc::new(StringArray::from(vec![candidate_id])),
            Arc::new(StringArray::from(vec!["authority"])),
            Arc::new(Float64Array::from(vec![0.1])),
            Arc::new(Float64Array::from(vec![1.0])),
            Arc::new(StringArray::from(vec!["reject"])),
            Arc::new(BooleanArray::from(vec![true])),
            Arc::new(StringArray::from(vec!["negative guard"])),
        ],
    )
    .expect("branch judgement batch should build");
    arrow_ipc_stream(&batch)
}

fn arrow_ipc_stream(batch: &RecordBatch) -> Vec<u8> {
    let mut writer = StreamWriter::try_new(Cursor::new(Vec::new()), batch.schema().as_ref())
        .expect("Arrow IPC stream writer should build");
    writer.write(batch).expect("Arrow IPC batch should write");
    writer.finish().expect("Arrow IPC stream should finish");
    writer
        .into_inner()
        .map(Cursor::into_inner)
        .expect("Arrow IPC stream should finalize")
}
