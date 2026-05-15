use std::pin::Pin;
use std::sync::{Arc, Mutex};

use arrow::array::{Array, Int32Array, StringArray};
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_server::{FlightService, FlightServiceServer};
use arrow_flight::{
    Action, ActionType, Criteria, Empty, FlightData, FlightInfo, HandshakeRequest,
    HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt, future, stream};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_wendao_attachments::audio::{AudioShardResult, build_audio_shard_result_batch};

use super::{
    AudioShardFlightClient, AudioShardInput, EngineRecordBatch, FlightDataEncoderBuilder,
    WENDAO_AUDIO_WORKERS_HEADER,
};

type BoxFlightStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

#[derive(Debug, Clone, Default)]
struct ObservedAudioShardRequest {
    descriptor_path: Vec<String>,
    row_count: usize,
    sample_rate_hz: i32,
    source_path: String,
    worker_budget_header: Option<String>,
}

#[derive(Clone)]
struct AudioShardTestFlightService {
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedAudioShardRequest>>>,
}

#[async_trait]
impl FlightService for AudioShardTestFlightService {
    type HandshakeStream = BoxFlightStream<HandshakeResponse>;
    type ListFlightsStream = BoxFlightStream<FlightInfo>;
    type DoGetStream = BoxFlightStream<FlightData>;
    type DoPutStream = BoxFlightStream<PutResult>;
    type DoExchangeStream = BoxFlightStream<FlightData>;
    type DoActionStream = BoxFlightStream<arrow_flight::Result>;
    type ListActionsStream = BoxFlightStream<ActionType>;

    async fn handshake(
        &self,
        _request: Request<tonic::Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("handshake is not used by this test"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented(
            "list_flights is not used by this test",
        ))
    }

    async fn get_flight_info(
        &self,
        _request: Request<super::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented(
            "get_flight_info is not used by this test",
        ))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<super::FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented(
            "poll_flight_info is not used by this test",
        ))
    }

    async fn get_schema(
        &self,
        _request: Request<super::FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("get_schema is not used by this test"))
    }

    async fn do_get(
        &self,
        _request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        Err(Status::unimplemented("do_get is not used by this test"))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put is not used by this test"))
    }

    async fn do_exchange(
        &self,
        request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        let worker_budget_header = request
            .metadata()
            .get(WENDAO_AUDIO_WORKERS_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let (descriptor_path, batches) = collect_request(request.into_inner()).await?;
        let batch = batches
            .first()
            .ok_or_else(|| Status::invalid_argument("missing audio shard request batch"))?;
        *self
            .observed
            .lock()
            .map_err(|_| Status::internal("observed request lock poisoned"))? =
            Some(ObservedAudioShardRequest {
                descriptor_path,
                row_count: batch.num_rows(),
                sample_rate_hz: int32_column(batch, "sampleRateHz")?.value(0),
                source_path: string_column(batch, "sourcePath")?.value(0).to_owned(),
                worker_budget_header,
            });

        let response_stream = FlightDataEncoderBuilder::new()
            .build(stream::iter(vec![Ok::<
                EngineRecordBatch,
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
        Err(Status::unimplemented("do_action is not used by this test"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented(
            "list_actions is not used by this test",
        ))
    }
}

#[tokio::test]
async fn audio_shard_flight_client_roundtrips_results() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    assert_eq!(client.endpoint_url(), endpoint);
    let response = client.request(std::slice::from_ref(&input)).await?;

    assert_eq!(response.results, vec![success]);
    let merge_report = response.merge_for_inputs(std::slice::from_ref(&input))?;
    assert_eq!(merge_report.text, "audio text");
    assert!(merge_report.has_complete_success_coverage());
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.descriptor_path, vec!["analysis", "audio-shards"]);
    assert_eq!(observed.row_count, 1);
    assert_eq!(observed.sample_rate_hz, 16_000);
    assert_eq!(observed.source_path, "/tmp/source.mp3");
    assert_eq!(observed.worker_budget_header, None);

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_sends_worker_budget_header() -> Result<(), String> {
    let input = sample_input();
    let success = AudioShardResult::succeeded(&input, "audio text", 0.92);
    let response_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let observed = Arc::new(Mutex::new(None));
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::clone(&observed)).await?;

    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;
    let response = client
        .request_with_worker_budget(std::slice::from_ref(&input), Some(4))
        .await?;

    assert_eq!(response.results, vec![success]);
    let observed = observed
        .lock()
        .map_err(|_| "observed request lock poisoned".to_owned())?
        .clone()
        .ok_or_else(|| "test service did not observe a request".to_owned())?;
    assert_eq!(observed.worker_budget_header.as_deref(), Some("4"));

    server_handle.abort();
    Ok(())
}

#[tokio::test]
async fn audio_shard_flight_client_rejects_empty_input() -> Result<(), String> {
    let input = sample_input();
    let response_batch =
        build_audio_shard_result_batch(&[AudioShardResult::skipped(&input, "unused")])?;
    let (endpoint, server_handle) =
        spawn_audio_shard_service(response_batch, Arc::new(Mutex::new(None))).await?;
    let client = AudioShardFlightClient::connect(endpoint.as_str()).await?;

    let Err(error) = client.request(&[]).await else {
        return Err("empty input should be rejected".to_owned());
    };

    assert_eq!(error, "audio shard request inputs cannot be empty");
    server_handle.abort();
    Ok(())
}

async fn collect_request(
    mut request: tonic::Streaming<FlightData>,
) -> Result<(Vec<String>, Vec<EngineRecordBatch>), Status> {
    let first = request
        .message()
        .await
        .map_err(|error| Status::invalid_argument(error.to_string()))?
        .ok_or_else(|| Status::invalid_argument("missing first audio shard Flight frame"))?;
    let descriptor_path = first
        .flight_descriptor
        .as_ref()
        .map(|descriptor| descriptor.path.clone())
        .unwrap_or_default();
    let frames = stream::once(async move { Ok(first) })
        .chain(request.map(|frame| frame.map_err(arrow_flight::error::FlightError::from)))
        .try_filter(|frame| future::ready(!frame.data_header.is_empty()));
    let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(frames);
    let mut batches = Vec::new();
    while let Some(batch) = batch_stream
        .try_next()
        .await
        .map_err(|error| Status::invalid_argument(error.to_string()))?
    {
        batches.push(batch);
    }
    Ok((descriptor_path, batches))
}

async fn spawn_audio_shard_service(
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedAudioShardRequest>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("listener should bind: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("listener should expose an address: {error}"))?;
    let service = AudioShardTestFlightService {
        response_batch,
        observed,
    };
    let handle = tokio::spawn(async move {
        if let Err(error) = Server::builder()
            .add_service(FlightServiceServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
        {
            panic!("test Flight server failed: {error}");
        }
    });
    Ok((format!("http://{address}"), handle))
}

fn sample_input() -> AudioShardInput {
    AudioShardInput {
        contract_version: "xiuxian_wendao.audio_shard_input.v1".to_owned(),
        source_path: "/tmp/source.mp3".to_owned(),
        source_content_hash: "sourcehash".to_owned(),
        shard_path: "/tmp/audio.wav".to_owned(),
        shard_sha256: "shardhash".to_owned(),
        shard_profile: "audio-shards-v1".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio".to_owned(),
        preferred_languages: vec!["zh".to_owned()],
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        shard_element_id: "audio-shard-id".to_owned(),
        reading_order_key: "000000.000000000000".to_owned(),
    }
}

fn string_column<'a>(batch: &'a EngineRecordBatch, name: &str) -> Result<&'a StringArray, Status> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Status::invalid_argument(format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| Status::invalid_argument(format!("`{name}` column is not Utf8")))
}

fn int32_column<'a>(batch: &'a EngineRecordBatch, name: &str) -> Result<&'a Int32Array, Status> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Status::invalid_argument(format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| Status::invalid_argument(format!("`{name}` column is not Int32")))
}
