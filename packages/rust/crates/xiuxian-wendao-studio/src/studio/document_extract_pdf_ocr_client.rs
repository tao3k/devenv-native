use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::{TryStreamExt, stream};
use serde::Serialize;
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, build_ocr_result_resource_batch,
    build_ocr_shard_input_batch, decode_ocr_shard_result_batches,
};
use xiuxian_wendao_server::transport::{
    ANALYSIS_PDF_OCR_SHARDS_ROUTE, WENDAO_PDF_OCR_WORKERS_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
};

const PDF_OCR_SHARD_FLIGHT_MESSAGE_SIZE_BYTES: usize = 256 * 1024 * 1024;

/// Feature-gated Arrow Flight client for the internal PDF OCR shard exchange.
#[derive(Debug, Clone)]
pub struct PdfOcrShardFlightClient {
    endpoint_url: String,
    channel: Channel,
}

/// OCR shard worker response decoded into typed rows and stable resource rows.
#[derive(Debug, Clone)]
pub struct PdfOcrShardFlightResponse {
    /// Typed OCR result rows returned by the Python analyzer worker.
    pub results: Vec<PdfOcrShardResult>,
    /// Stable document resource batch materialized from OCR result rows.
    pub resource_batch: EngineRecordBatch,
    /// Internal Rust scheduler diagnostics for live shard batches.
    pub scheduler_trace: Vec<PdfOcrShardSchedulerTrace>,
}

/// Internal scheduler trace for one live OCR shard request chunk.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOcrShardSchedulerTrace {
    /// Scheduler lane used for the live OCR request.
    pub lane: &'static str,
    /// Number of shard rows sent in this request chunk.
    pub shard_count: usize,
    /// Lowest source page index in the chunk.
    pub page_start: Option<u32>,
    /// Highest source page index in the chunk.
    pub page_end: Option<u32>,
    /// Shard type for the chunk when it is homogeneous.
    pub shard_type: Option<String>,
    /// OCR profile for the chunk when it is homogeneous.
    pub ocr_profile: Option<String>,
    /// Queue wait before this scheduler lane acquired worker permits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_wait_ms: Option<f64>,
    /// Milliseconds from lane dispatch start to this chunk request start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatch_start_ms: Option<f64>,
    /// Milliseconds from lane dispatch start to this chunk request completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatch_end_ms: Option<f64>,
    /// Wall-clock request latency for this chunk.
    pub latency_ms: f64,
    /// Character count returned by successful rows in this chunk.
    pub text_char_count: usize,
}

impl PdfOcrShardFlightClient {
    /// Connect to the Python analyzer Flight endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error when the endpoint URL is invalid or cannot be reached.
    pub async fn connect(endpoint_url: impl Into<String>) -> Result<Self, String> {
        let endpoint_url = endpoint_url.into();
        let endpoint = Endpoint::from_shared(endpoint_url.clone())
            .map_err(|error| format!("invalid PDF OCR shard endpoint `{endpoint_url}`: {error}"))?;
        let channel = endpoint.connect().await.map_err(|error| {
            format!("failed to connect PDF OCR shard endpoint `{endpoint_url}`: {error}")
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

    /// Send OCR shard input rows and decode OCR worker result rows.
    ///
    /// # Errors
    ///
    /// Returns an error when input rows are empty, Arrow encoding fails, the
    /// Flight exchange fails, or the worker response does not match the stable
    /// OCR shard result contract.
    pub async fn request(
        &self,
        inputs: &[PdfOcrShardInput],
    ) -> Result<PdfOcrShardFlightResponse, String> {
        self.request_with_worker_budget(inputs, None).await
    }

    /// Send OCR shard inputs with an optional Python worker budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the Flight exchange fails or the worker response
    /// does not match the stable OCR shard result contract.
    pub async fn request_with_worker_budget(
        &self,
        inputs: &[PdfOcrShardInput],
        worker_budget: Option<usize>,
    ) -> Result<PdfOcrShardFlightResponse, String> {
        request_pdf_ocr_shards_on_channel(self.channel.clone(), inputs, worker_budget).await
    }
}

async fn request_pdf_ocr_shards_on_channel(
    channel: Channel,
    inputs: &[PdfOcrShardInput],
    worker_budget: Option<usize>,
) -> Result<PdfOcrShardFlightResponse, String> {
    if inputs.is_empty() {
        return Err("PDF OCR shard request inputs cannot be empty".to_string());
    }

    let input_batch = build_ocr_shard_input_batch(inputs)?;
    let request_stream = FlightDataEncoderBuilder::new()
        .with_schema(input_batch.schema())
        .with_flight_descriptor(Some(pdf_ocr_shards_descriptor()))
        .with_max_flight_data_size(PDF_OCR_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .build(stream::iter(vec![Ok::<
            EngineRecordBatch,
            arrow_flight::error::FlightError,
        >(input_batch)]));

    let inner_client = TonicFlightServiceClient::new(channel)
        .max_encoding_message_size(PDF_OCR_SHARD_FLIGHT_MESSAGE_SIZE_BYTES)
        .max_decoding_message_size(PDF_OCR_SHARD_FLIGHT_MESSAGE_SIZE_BYTES);
    let mut client = FlightClient::new_from_inner(inner_client);
    client
        .add_header(WENDAO_SCHEMA_VERSION_HEADER, "v2")
        .map_err(|error| format!("invalid PDF OCR shard schema-version header: {error}"))?;
    let worker_budget_header = worker_budget
        .filter(|budget| *budget > 0)
        .map(|budget| budget.to_string());
    if let Some(worker_budget_header) = worker_budget_header.as_deref() {
        client
            .add_header(WENDAO_PDF_OCR_WORKERS_HEADER, worker_budget_header)
            .map_err(|error| format!("invalid PDF OCR workers header: {error}"))?;
    }

    let response_batches = client
        .do_exchange(request_stream)
        .await
        .map_err(|error| format!("PDF OCR shard exchange failed: {error}"))?
        .try_collect::<Vec<EngineRecordBatch>>()
        .await
        .map_err(|error| format!("failed to decode PDF OCR shard response: {error}"))?;
    if response_batches.is_empty() {
        return Err("PDF OCR shard exchange returned no record batches".to_string());
    }

    let results = decode_ocr_shard_result_batches(&response_batches)?;
    let resource_batch = build_ocr_result_resource_batch(&results)?;
    Ok(PdfOcrShardFlightResponse {
        results,
        resource_batch,
        scheduler_trace: Vec::new(),
    })
}

fn pdf_ocr_shards_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        ANALYSIS_PDF_OCR_SHARDS_ROUTE
            .trim_start_matches('/')
            .split('/')
            .map(ToString::to_string)
            .collect(),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/gateway/studio/document_extract_pdf_ocr_client.rs"]
mod tests;
