use crate::memory::transport::fetch::{
    fetch_memory_julia_calibration_artifact_rows, fetch_memory_julia_episodic_recall_score_rows,
    fetch_memory_julia_gate_score_recommendation_rows, fetch_memory_julia_plan_tuning_advice_rows,
};
use crate::memory::{
    MemoryJuliaCalibrationRequestRow, MemoryJuliaEpisodicRecallRequestRow,
    MemoryJuliaGateScoreRequestRow, MemoryJuliaPlanTuningRequestRow,
    memory_julia_calibration_response_schema, memory_julia_episodic_recall_response_schema,
    memory_julia_gate_score_response_schema, memory_julia_plan_tuning_response_schema,
};
use arrow::array::{Float32Array, StringArray, UInt32Array};
use arrow::record_batch::RecordBatch;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_server::FlightServiceServer;
use arrow_flight::{
    Action, Criteria, Empty, FlightData, FlightDescriptor, FlightInfo, HandshakeRequest,
    HandshakeResponse, PollInfo, SchemaResult, Ticket, flight_service_server::FlightService,
};
use async_trait::async_trait;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::{Stream, StreamExt, wrappers::TcpListenerStream};
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_wendao_runtime::{
    config::MemoryJuliaComputeRuntimeConfig, transport::WENDAO_SCHEMA_VERSION_HEADER,
};

type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
type PutResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::PutResult, Status>> + Send>>;
type ActionResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
type ActionTypeStream =
    Pin<Box<dyn Stream<Item = Result<arrow_flight::ActionType, Status>> + Send>>;

#[derive(Clone)]
struct MemoryTestFlightService {
    expected_schema_version: String,
    response_batch: RecordBatch,
}

impl MemoryTestFlightService {
    fn new(expected_schema_version: &str, response_batch: RecordBatch) -> Self {
        Self {
            expected_schema_version: expected_schema_version.to_string(),
            response_batch,
        }
    }

    fn validate_schema_version(
        &self,
        request: &Request<tonic::Streaming<FlightData>>,
    ) -> Result<(), Status> {
        let schema_version = request
            .metadata()
            .get(WENDAO_SCHEMA_VERSION_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if schema_version != self.expected_schema_version {
            return Err(Status::invalid_argument(format!(
                "unexpected schema version header: {schema_version}"
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl FlightService for MemoryTestFlightService {
    type HandshakeStream = HandshakeStream;
    type ListFlightsStream = FlightInfoStream;
    type DoGetStream = FlightDataStream;
    type DoPutStream = PutResultStream;
    type DoExchangeStream = FlightDataStream;
    type DoActionStream = ActionResultStream;
    type ListActionsStream = ActionTypeStream;

    async fn handshake(
        &self,
        _request: Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented(
            "handshake is not used by the memory transport test service",
        ))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented(
            "list_flights is not used by the memory transport test service",
        ))
    }

    async fn get_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented(
            "get_flight_info is not used by the memory transport test service",
        ))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented(
            "poll_flight_info is not used by the memory transport test service",
        ))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented(
            "get_schema is not used by the memory transport test service",
        ))
    }

    async fn do_get(
        &self,
        _request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        Err(Status::unimplemented(
            "do_get is not used by the memory transport test service",
        ))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented(
            "do_put is not used by the memory transport test service",
        ))
    }

    async fn do_exchange(
        &self,
        request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        self.validate_schema_version(&request)?;
        let response_stream = FlightDataEncoderBuilder::new()
            .build(tokio_stream::iter(vec![Ok::<
                RecordBatch,
                arrow_flight::error::FlightError,
            >(
                self.response_batch.clone()
            )]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented(
            "do_action is not used by the memory transport test service",
        ))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented(
            "list_actions is not used by the memory transport test service",
        ))
    }
}

fn runtime_for_test(base_url: String, route: &str) -> MemoryJuliaComputeRuntimeConfig {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.base_url = base_url;
    runtime.plugin_id = "wendao.memory".into();
    runtime.schema_version = "v1".to_string();
    runtime.routes.episodic_recall = route.to_string();
    runtime.routes.memory_gate_score = route.to_string();
    runtime.routes.memory_plan_tuning = route.to_string();
    runtime.routes.memory_calibration = route.to_string();
    runtime
}

async fn spawn_memory_service(
    response_batch: RecordBatch,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("listener should bind: {error}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("listener should expose an address: {error}"));
    let service = MemoryTestFlightService::new("v1", response_batch);
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(FlightServiceServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap_or_else(|error| panic!("memory flight test server should serve: {error}"));
    });
    (format!("http://{address}"), server)
}

fn episodic_recall_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        memory_julia_episodic_recall_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["query-1"])),
            Arc::new(StringArray::from(vec!["episode-1"])),
            Arc::new(Float32Array::from(vec![0.8_f32])),
            Arc::new(Float32Array::from(vec![0.7_f32])),
            Arc::new(Float32Array::from(vec![0.75_f32])),
            Arc::new(Float32Array::from(vec![0.9_f32])),
            Arc::new(StringArray::from(vec![Some("semantic+utility")])),
            Arc::new(StringArray::from(vec![Some("two_phase")])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("episodic recall response batch should build: {error}"))
}

fn gate_score_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        memory_julia_gate_score_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["memory-1"])),
            Arc::new(StringArray::from(vec!["retain"])),
            Arc::new(Float32Array::from(vec![0.92_f32])),
            Arc::new(Float32Array::from(vec![0.78_f32])),
            Arc::new(Float32Array::from(vec![0.74_f32])),
            Arc::new(StringArray::from(vec!["keep"])),
            Arc::new(StringArray::from(vec!["stable and recently validated"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("gate score response batch should build: {error}"))
}

fn plan_tuning_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        memory_julia_plan_tuning_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["repo"])),
            Arc::new(UInt32Array::from(vec![12_u32])),
            Arc::new(UInt32Array::from(vec![6_u32])),
            Arc::new(Float32Array::from(vec![0.65_f32])),
            Arc::new(Float32Array::from(vec![0.12_f32])),
            Arc::new(UInt32Array::from(vec![1024_u32])),
            Arc::new(StringArray::from(vec!["increase recall window"])),
            Arc::new(Float32Array::from(vec![0.88_f32])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("plan tuning response batch should build: {error}"))
}

fn calibration_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        memory_julia_calibration_response_schema(),
        vec![
            Arc::new(StringArray::from(vec!["calibration-1"])),
            Arc::new(StringArray::from(vec![Some("searchinfra")])),
            Arc::new(StringArray::from(vec!["artifact://memory/calibration-1"])),
            Arc::new(StringArray::from(vec!["{\"precision\":0.82}"])),
            Arc::new(StringArray::from(vec!["{\"retain\":0.7}"])),
            Arc::new(StringArray::from(vec!["{\"semantic\":0.6}"])),
            Arc::new(StringArray::from(vec!["v1"])),
        ],
    )
    .unwrap_or_else(|error| panic!("calibration response batch should build: {error}"))
}

#[tokio::test]
async fn fetch_memory_julia_episodic_recall_score_rows_roundtrips() {
    let route = "/memory/episodic_recall";
    let (base_url, server) = spawn_memory_service(episodic_recall_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_memory_julia_episodic_recall_score_rows(
        &runtime,
        &[MemoryJuliaEpisodicRecallRequestRow {
            query_id: "query-1".into(),
            scenario_pack: Some("searchinfra".to_string()),
            scope: "repo".to_string(),
            query_text: Some("fix memory lane".to_string()),
            query_embedding: vec![0.1, 0.2, 0.3],
            candidate_id: "episode-1".into(),
            intent_embedding: vec![0.1, 0.2, 0.3],
            q_value: 0.7,
            success_count: 3,
            failure_count: 1,
            retrieval_count: 4,
            created_at_ms: 100.into(),
            updated_at_ms: 200.into(),
            k1: 1.0,
            k2: 0.5,
            lambda: 0.6,
            min_score: 0.1,
        }],
    )
    .await
    .unwrap_or_else(|error| panic!("episodic recall fetch should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].candidate_id, "episode-1");
    assert!((rows[0].confidence - 0.9).abs() < 1e-6);

    server.abort();
}

#[tokio::test]
async fn fetch_memory_julia_gate_score_recommendation_rows_roundtrips() {
    let route = "/memory/gate_score";
    let (base_url, server) = spawn_memory_service(gate_score_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_memory_julia_gate_score_recommendation_rows(
        &runtime,
        &[MemoryJuliaGateScoreRequestRow {
            memory_id: "memory-1".to_string(),
            scenario_pack: Some("searchinfra".to_string()),
            react_revalidation_score: 0.9,
            graph_consistency_score: 0.8,
            omega_alignment_score: 0.85,
            q_value: 0.75,
            usage_count: 4,
            failure_rate: 0.25,
            ttl_score: 0.7,
            current_state: "active".into(),
        }],
    )
    .await
    .unwrap_or_else(|error| panic!("gate score fetch should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].verdict, "retain");

    server.abort();
}

#[tokio::test]
async fn fetch_memory_julia_plan_tuning_advice_rows_roundtrips() {
    let route = "/memory/plan_tuning";
    let (base_url, server) = spawn_memory_service(plan_tuning_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_memory_julia_plan_tuning_advice_rows(
        &runtime,
        &[MemoryJuliaPlanTuningRequestRow {
            scope: "repo".to_string(),
            scenario_pack: Some("searchinfra".to_string()),
            current_k1: 8,
            current_k2: 4,
            current_lambda: 0.7,
            current_min_score: 0.18,
            current_max_context_chars: 960,
            feedback_bias: 0.2,
            recent_success_rate: 0.4,
            recent_failure_rate: 0.3,
            recent_latency_ms: 250,
        }],
    )
    .await
    .unwrap_or_else(|error| panic!("plan tuning fetch should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].next_k1, 12);

    server.abort();
}

#[tokio::test]
async fn fetch_memory_julia_calibration_artifact_rows_roundtrips() {
    let route = "/memory/calibration";
    let (base_url, server) = spawn_memory_service(calibration_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_memory_julia_calibration_artifact_rows(
        &runtime,
        &[MemoryJuliaCalibrationRequestRow {
            calibration_job_id: "calibration-1".to_string(),
            scenario_pack: Some("searchinfra".to_string()),
            dataset_ref: "dataset://memory/searchinfra/latest".to_string(),
            objective: "maximize_precision".to_string(),
            hyperparam_config: "{\"max_iter\":32}".to_string(),
        }],
    )
    .await
    .unwrap_or_else(|error| panic!("calibration fetch should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].artifact_ref, "artifact://memory/calibration-1");

    server.abort();
}
