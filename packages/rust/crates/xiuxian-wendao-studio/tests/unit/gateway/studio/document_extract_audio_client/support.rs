use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use arrow::array::{Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::encode::FlightDataEncoderBuilder;
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
use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardManifestItem, AudioShardMaterializationSource,
    AudioShardMaterializedItem, AudioShardPlan, AudioSourceIdentity, AudioSpeechSegment,
    AudioSpeechWindowPlannerInput,
};
use xiuxian_wendao_server::transport::{
    WENDAO_AUDIO_HOSTED_BASE_URL_HEADER, WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER,
    WENDAO_AUDIO_HOSTED_MODEL_HEADER, WENDAO_AUDIO_HOSTED_PROVIDER_HEADER,
    WENDAO_AUDIO_WORKER_HEADER, WENDAO_AUDIO_WORKERS_HEADER,
};

type BoxFlightStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

#[derive(Debug, Clone, Default)]
pub(crate) struct ObservedAudioShardRequest {
    pub(crate) descriptor_path: Vec<String>,
    pub(crate) row_count: usize,
    pub(crate) sample_rate_hz: i32,
    pub(crate) start_ms: i64,
    pub(crate) duration_ms: i64,
    pub(crate) media_start_ms: i64,
    pub(crate) media_duration_ms: i64,
    pub(crate) source_path: String,
    pub(crate) backend_profile: String,
    pub(crate) worker_budget_header: Option<String>,
    pub(crate) audio_worker_header: Option<String>,
    pub(crate) hosted_provider_header: Option<String>,
    pub(crate) hosted_base_url_header: Option<String>,
    pub(crate) hosted_endpoint_header: Option<String>,
    pub(crate) hosted_model_header: Option<String>,
    pub(crate) windows: Vec<ObservedAudioShardWindow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedAudioShardWindow {
    pub(crate) start_ms: i64,
    pub(crate) duration_ms: i64,
    pub(crate) media_start_ms: i64,
    pub(crate) media_duration_ms: i64,
    pub(crate) reading_order_key: String,
}

#[derive(Clone)]
struct AudioShardTestFlightService {
    response_batches: Arc<Mutex<VecDeque<EngineRecordBatch>>>,
    observed: Arc<Mutex<Option<ObservedAudioShardRequest>>>,
    observed_requests: Arc<Mutex<Vec<ObservedAudioShardRequest>>>,
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
        _request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented(
            "get_flight_info is not used by this test",
        ))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<arrow_flight::FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented(
            "poll_flight_info is not used by this test",
        ))
    }

    async fn get_schema(
        &self,
        _request: Request<arrow_flight::FlightDescriptor>,
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
        let metadata = request.metadata().clone();
        let worker_budget_header = metadata_value(&metadata, WENDAO_AUDIO_WORKERS_HEADER);
        let audio_worker_header = metadata_value(&metadata, WENDAO_AUDIO_WORKER_HEADER);
        let hosted_provider_header = metadata_value(&metadata, WENDAO_AUDIO_HOSTED_PROVIDER_HEADER);
        let hosted_base_url_header = metadata_value(&metadata, WENDAO_AUDIO_HOSTED_BASE_URL_HEADER);
        let hosted_endpoint_header = metadata_value(&metadata, WENDAO_AUDIO_HOSTED_ENDPOINT_HEADER);
        let hosted_model_header = metadata_value(&metadata, WENDAO_AUDIO_HOSTED_MODEL_HEADER);
        let (descriptor_path, batches) = collect_request(request.into_inner()).await?;
        let batch = batches
            .first()
            .ok_or_else(|| Status::invalid_argument("missing audio shard request batch"))?;
        if batch.num_rows() == 0 {
            return Err(Status::invalid_argument("empty audio shard request batch"));
        }
        let sample_rate_hz = int32_column(batch, "sampleRateHz")?;
        let start_ms = int64_column(batch, "startMs")?;
        let duration_ms = int64_column(batch, "durationMs")?;
        let media_start_ms = int64_column(batch, "mediaStartMs")?;
        let media_duration_ms = int64_column(batch, "mediaDurationMs")?;
        let reading_order_key = string_column(batch, "readingOrderKey")?;
        let windows = (0..batch.num_rows())
            .map(|row| ObservedAudioShardWindow {
                start_ms: start_ms.value(row),
                duration_ms: duration_ms.value(row),
                media_start_ms: media_start_ms.value(row),
                media_duration_ms: media_duration_ms.value(row),
                reading_order_key: reading_order_key.value(row).to_owned(),
            })
            .collect::<Vec<_>>();
        let observed_request = ObservedAudioShardRequest {
            descriptor_path,
            row_count: batch.num_rows(),
            sample_rate_hz: sample_rate_hz.value(0),
            start_ms: windows[0].start_ms,
            duration_ms: windows[0].duration_ms,
            media_start_ms: windows[0].media_start_ms,
            media_duration_ms: windows[0].media_duration_ms,
            source_path: string_column(batch, "sourcePath")?.value(0).to_owned(),
            backend_profile: string_column(batch, "backendProfile")?.value(0).to_owned(),
            worker_budget_header,
            audio_worker_header,
            hosted_provider_header,
            hosted_base_url_header,
            hosted_endpoint_header,
            hosted_model_header,
            windows,
        };
        *self
            .observed
            .lock()
            .map_err(|_| Status::internal("observed request lock poisoned"))? =
            Some(observed_request.clone());
        self.observed_requests
            .lock()
            .map_err(|_| Status::internal("observed request sequence lock poisoned"))?
            .push(observed_request);

        let response_batch = next_response_batch(&self.response_batches)?;

        let response_stream = FlightDataEncoderBuilder::new()
            .build(stream::iter(vec![Ok::<
                EngineRecordBatch,
                arrow_flight::error::FlightError,
            >(response_batch)]))
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

pub(super) async fn collect_request(
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

pub(crate) async fn spawn_audio_shard_service(
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedAudioShardRequest>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    spawn_audio_shard_sequence_service(
        vec![response_batch],
        observed,
        Arc::new(Mutex::new(Vec::new())),
    )
    .await
}

pub(crate) async fn spawn_audio_shard_sequence_service(
    response_batches: Vec<EngineRecordBatch>,
    observed: Arc<Mutex<Option<ObservedAudioShardRequest>>>,
    observed_requests: Arc<Mutex<Vec<ObservedAudioShardRequest>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    if response_batches.is_empty() {
        return Err("audio shard test service needs at least one response batch".to_owned());
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("listener should bind: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("listener should expose an address: {error}"))?;
    let service = AudioShardTestFlightService {
        response_batches: Arc::new(Mutex::new(VecDeque::from(response_batches))),
        observed,
        observed_requests,
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

fn next_response_batch(
    response_batches: &Arc<Mutex<VecDeque<EngineRecordBatch>>>,
) -> Result<EngineRecordBatch, Status> {
    let mut response_batches = response_batches
        .lock()
        .map_err(|_| Status::internal("response batch lock poisoned"))?;
    if response_batches.len() > 1 {
        return response_batches
            .pop_front()
            .ok_or_else(|| Status::internal("missing audio shard response batch"));
    }
    response_batches
        .front()
        .cloned()
        .ok_or_else(|| Status::internal("missing audio shard response batch"))
}

fn metadata_value(metadata: &tonic::metadata::MetadataMap, key: &'static str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

pub(crate) fn sample_input() -> AudioShardInput {
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

pub(crate) fn sample_variable_window_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(30_000),
        },
        chunk_duration_ms: 30_000,
        start_offsets_ms: vec![9_000],
        window_durations_ms: vec![8_000],
        context_before_ms: 500,
        context_after_ms: 700,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
        strategy: "speech-segments".to_owned(),
    }
}

pub(crate) fn sample_speech_window_planner_input() -> AudioSpeechWindowPlannerInput {
    AudioSpeechWindowPlannerInput {
        profile: "audio-shards-v1".to_owned(),
        source: AudioSourceIdentity {
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            duration_ms: Some(30_000),
        },
        chunk_duration_ms: 30_000,
        limit_chunks: 8,
        speech_segments: vec![
            AudioSpeechSegment {
                index: 0,
                start_ms: 0,
                duration_ms: 4_000,
            },
            AudioSpeechSegment {
                index: 1,
                start_ms: 9_000,
                duration_ms: 3_000,
            },
            AudioSpeechSegment {
                index: 2,
                start_ms: 14_000,
                duration_ms: 3_000,
            },
        ],
        merge_gap_ms: 1_000,
        min_window_ms: 8_000,
        short_merge_gap_ms: Some(3_000),
        max_window_ms: Some(30_000),
        boundary_snap_tolerance_ms: 0,
        context_before_ms: 500,
        context_after_ms: 700,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        audio_bitrate: None,
    }
}

pub(crate) fn sample_materialized_item() -> AudioShardMaterializedItem {
    AudioShardMaterializedItem {
        manifest: AudioShardManifestItem {
            shard_id: "materialized-audio-shard-id".to_owned(),
            source_id: "/tmp/source.mp3".to_owned(),
            source_sha256: "sourcehash".to_owned(),
            chunk_index: 1,
            start_ms: 9_000,
            duration_ms: 8_000,
            media_start_ms: 8_500,
            media_duration_ms: 9_200,
            context_before_ms: 500,
            context_after_ms: 700,
            sample_rate_hz: 16_000,
            channels: 1,
            audio_format: "wav".to_owned(),
            audio_bitrate: None,
            cache_key: "audio-shards-v1:materialized-audio-shard-id".to_owned(),
            reading_order_key: "000001.000000009000".to_owned(),
        },
        output_path: std::path::PathBuf::from("/tmp/materialized.wav"),
        shard_sha256: "materialized-shardhash".to_owned(),
        shard_byte_len: 128,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    }
}

#[cfg(unix)]
pub(crate) fn make_executable(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(error_to_string)?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).map_err(error_to_string)
}

#[cfg(not(unix))]
pub(crate) fn make_executable(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

pub(crate) fn error_to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
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

fn int64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a arrow::array::Int64Array, Status> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Status::invalid_argument(format!("missing `{name}` column")))?
        .as_any()
        .downcast_ref::<arrow::array::Int64Array>()
        .ok_or_else(|| Status::invalid_argument(format!("`{name}` column is not Int64")))
}
