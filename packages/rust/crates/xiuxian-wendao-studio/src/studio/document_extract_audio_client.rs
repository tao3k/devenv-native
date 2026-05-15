//! Owns the Studio document extract audio shard Flight client surface.

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::{TryStreamExt, stream};
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_attachments::audio::{
    AudioShardInput, AudioShardMergeReport, AudioShardResult, build_audio_shard_input_batch,
    decode_audio_shard_result_batches, merge_audio_shard_results,
};
use xiuxian_wendao_server::transport::{
    ANALYSIS_AUDIO_SHARDS_ROUTE, WENDAO_AUDIO_WORKERS_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
};

const AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;

/// Feature-gated Arrow Flight client for the internal audio shard exchange.
#[derive(Debug, Clone)]
pub struct AudioShardFlightClient {
    endpoint_url: String,
    channel: Channel,
}

/// Audio shard worker response decoded into typed rows.
#[derive(Debug, Clone)]
pub struct AudioShardFlightResponse {
    /// Typed audio result rows returned by the Python analyzer worker.
    pub results: Vec<AudioShardResult>,
}

impl AudioShardFlightResponse {
    /// Merge the response rows against the submitted shard inputs.
    ///
    /// # Errors
    ///
    /// Returns an error when result rows fail identity, fingerprint, profile,
    /// or text MIME validation.
    pub fn merge_for_inputs(
        &self,
        inputs: &[AudioShardInput],
    ) -> Result<AudioShardMergeReport, String> {
        merge_audio_shard_results(inputs, self.results.as_slice())
    }
}

impl AudioShardFlightClient {
    /// Connect to the Python analyzer Flight endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint URL is invalid or cannot be reached.
    pub async fn connect(endpoint_url: impl Into<String>) -> Result<Self, String> {
        let endpoint_url = endpoint_url.into();
        let endpoint = Endpoint::from_shared(endpoint_url.clone())
            .map_err(|error| format!("invalid audio shard endpoint `{endpoint_url}`: {error}"))?;
        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect audio shard endpoint `{endpoint_url}`: {error}")
        })?;
        Ok(Self {
            endpoint_url,
            channel,
        })
    }

    /// Return the connected endpoint URL.
    #[must_use]
    pub fn endpoint_url(&self) -> &str {
        self.endpoint_url.as_str()
    }

    /// Send audio shard input rows and decode worker result rows.
    ///
    /// # Errors
    ///
    /// Returns an error when input rows are empty, Arrow encoding fails, the
    /// Flight exchange fails, or the worker response does not match the stable
    /// audio shard result contract.
    pub async fn request(
        &self,
        inputs: &[AudioShardInput],
    ) -> Result<AudioShardFlightResponse, String> {
        self.request_with_worker_budget(inputs, None).await
    }

    /// Send audio shard inputs with an optional Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the Flight exchange fails or the worker response
    /// does not match the stable audio shard result contract.
    pub async fn request_with_worker_budget(
        &self,
        inputs: &[AudioShardInput],
        worker_budget: Option<usize>,
    ) -> Result<AudioShardFlightResponse, String> {
        request_audio_shards_on_channel(self.channel.clone(), inputs, worker_budget).await
    }
}

async fn request_audio_shards_on_channel(
    channel: Channel,
    inputs: &[AudioShardInput],
    worker_budget: Option<usize>,
) -> Result<AudioShardFlightResponse, String> {
    if inputs.is_empty() {
        return Err("audio shard request inputs cannot be empty".to_owned());
    }

    let input_batch = build_audio_shard_input_batch(inputs)?;
    let request_stream = FlightDataEncoderBuilder::new()
        .with_schema(input_batch.schema())
        .with_flight_descriptor(Some(audio_shards_descriptor()))
        .with_max_flight_data_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .build(stream::iter(vec![Ok::<
            EngineRecordBatch,
            arrow_flight::error::FlightError,
        >(input_batch)]));

    let inner_client = TonicFlightServiceClient::new(channel)
        .max_encoding_message_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .max_decoding_message_size(AUDIO_SHARD_FLIGHT_MESSAGE_SIZE_BYTES);
    let mut client = FlightClient::new_from_inner(inner_client);
    client
        .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
        .map_err(|error| format!("invalid audio shard schema-version header: {error}"))?;
    let worker_budget_header = worker_budget
        .filter(|budget| *budget > 0)
        .map(|budget| budget.to_string());
    if let Some(worker_budget_header) = worker_budget_header.as_deref() {
        client
            .add_header(WENDAO_AUDIO_WORKERS_HEADER, worker_budget_header)
            .map_err(|error| format!("invalid audio workers header: {error}"))?;
    }

    let response_batches = client
        .do_exchange(request_stream)
        .await
        .map_err(|error| format!("audio shard exchange failed: {error}"))?
        .try_collect::<Vec<EngineRecordBatch>>()
        .await
        .map_err(|error| format!("failed to decode audio shard response: {error}"))?;
    if response_batches.is_empty() {
        return Err("audio shard exchange returned no record batches".to_owned());
    }

    Ok(AudioShardFlightResponse {
        results: decode_audio_shard_result_batches(response_batches.as_slice())?,
    })
}

fn audio_shards_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        ANALYSIS_AUDIO_SHARDS_ROUTE
            .trim_start_matches('/')
            .split('/')
            .map(ToString::to_string)
            .collect(),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/gateway/studio/document_extract_audio_client.rs"]
mod tests;
