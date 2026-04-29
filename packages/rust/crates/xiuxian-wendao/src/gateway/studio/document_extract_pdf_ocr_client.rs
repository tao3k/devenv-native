use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::FlightDescriptor;
use arrow_flight::client::FlightClient;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_client::FlightServiceClient as TonicFlightServiceClient;
use futures::{TryStreamExt, stream};
use tonic::transport::{Channel, Endpoint};
use xiuxian_wendao_attachments::pdf::ocr::{
    PdfOcrShardInput, PdfOcrShardResult, build_ocr_result_resource_batch,
    build_ocr_shard_input_batch, decode_ocr_shard_result_batches,
};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_PDF_OCR_SHARDS_ROUTE, WENDAO_SCHEMA_VERSION_HEADER,
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
        request_pdf_ocr_shards_on_channel(self.channel.clone(), inputs).await
    }
}

async fn request_pdf_ocr_shards_on_channel(
    channel: Channel,
    inputs: &[PdfOcrShardInput],
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
mod tests {
    use std::path::{Path, PathBuf};
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
    use xiuxian_wendao_attachments::pdf::ocr::{
        PdfOcrWorkerProfile, build_ocr_shard_inputs, build_ocr_shard_result_batch,
    };
    use xiuxian_wendao_attachments::pdf::render::{
        PdfPageBox, PdfPageRenderProfile, PdfPageShardManifestInput, RenderedRasterIdentity,
        build_shard_manifest,
    };

    use super::*;

    type BoxFlightStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

    #[derive(Debug, Clone, Default)]
    struct ObservedOcrShardRequest {
        descriptor_path: Vec<String>,
        row_count: usize,
        page_index: i32,
    }

    #[derive(Clone)]
    struct PdfOcrShardTestFlightService {
        response_batch: EngineRecordBatch,
        observed: Arc<Mutex<Option<ObservedOcrShardRequest>>>,
    }

    #[async_trait]
    impl FlightService for PdfOcrShardTestFlightService {
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
            _request: Request<FlightDescriptor>,
        ) -> Result<Response<FlightInfo>, Status> {
            Err(Status::unimplemented(
                "get_flight_info is not used by this test",
            ))
        }

        async fn poll_flight_info(
            &self,
            _request: Request<FlightDescriptor>,
        ) -> Result<Response<PollInfo>, Status> {
            Err(Status::unimplemented(
                "poll_flight_info is not used by this test",
            ))
        }

        async fn get_schema(
            &self,
            _request: Request<FlightDescriptor>,
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
            let (descriptor_path, batches) = collect_request(request.into_inner()).await?;
            let batch = batches
                .first()
                .ok_or_else(|| Status::invalid_argument("missing OCR shard request batch"))?;
            let page_index = int32_column(batch, "pageIndex")?.value(0);
            *self
                .observed
                .lock()
                .map_err(|_| Status::internal("observed request lock poisoned"))? =
                Some(ObservedOcrShardRequest {
                    descriptor_path,
                    row_count: batch.num_rows(),
                    page_index,
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
    async fn pdf_ocr_shard_flight_client_roundtrips_results_and_resources() -> Result<(), String> {
        let input = sample_input();
        let success = PdfOcrShardResult::succeeded(&input, "page text", 0.93);
        let response_batch = build_ocr_shard_result_batch(std::slice::from_ref(&success))?;
        let observed = Arc::new(Mutex::new(None));
        let (endpoint, server_handle) =
            spawn_ocr_shard_service(response_batch, Arc::clone(&observed)).await;

        let client = PdfOcrShardFlightClient::connect(endpoint.as_str()).await?;
        assert_eq!(client.endpoint_url(), endpoint);
        let response = client.request(std::slice::from_ref(&input)).await?;

        assert_eq!(response.results, vec![success]);
        assert_eq!(response.resource_batch.num_rows(), 1);
        assert_eq!(
            string_column(&response.resource_batch, "resourceType")?.value(0),
            "ocr_text"
        );
        assert_eq!(
            string_column(&response.resource_batch, "content")?.value(0),
            "page text"
        );

        let observed = observed
            .lock()
            .map_err(|_| "observed request lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "test service did not observe a request".to_string())?;
        assert_eq!(observed.descriptor_path, vec!["analysis", "pdf-ocr-shards"]);
        assert_eq!(observed.row_count, 1);
        assert_eq!(observed.page_index, 0);

        server_handle.abort();
        Ok(())
    }

    #[tokio::test]
    async fn pdf_ocr_shard_flight_client_rejects_empty_input() -> Result<(), String> {
        let input = sample_input();
        let response_batch =
            build_ocr_shard_result_batch(&[PdfOcrShardResult::skipped(&input, "unused")])?;
        let (endpoint, server_handle) =
            spawn_ocr_shard_service(response_batch, Arc::new(Mutex::new(None))).await;
        let client = PdfOcrShardFlightClient::connect(endpoint.as_str()).await?;

        let Err(error) = client.request(&[]).await else {
            return Err("empty input should be rejected".to_string());
        };

        assert_eq!(error, "PDF OCR shard request inputs cannot be empty");
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
            .ok_or_else(|| Status::invalid_argument("missing first OCR shard Flight frame"))?;
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

    async fn spawn_ocr_shard_service(
        response_batch: EngineRecordBatch,
        observed: Arc<Mutex<Option<ObservedOcrShardRequest>>>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|error| panic!("listener should bind: {error}"));
        let address = listener
            .local_addr()
            .unwrap_or_else(|error| panic!("listener should expose an address: {error}"));
        let service = PdfOcrShardTestFlightService {
            response_batch,
            observed,
        };
        let handle = tokio::spawn(async move {
            Server::builder()
                .add_service(FlightServiceServer::new(service))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap_or_else(|error| panic!("test Flight server failed: {error}"));
        });
        (format!("http://{address}"), handle)
    }

    fn sample_input() -> PdfOcrShardInput {
        let profile = PdfPageRenderProfile::ocr_default();
        let manifest = build_shard_manifest(PdfPageShardManifestInput {
            source_path: Path::new("/tmp/source.pdf"),
            source_content_hash: "sourcehash",
            page_index: 0,
            profile: &profile,
            media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
            crop_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
            rotation_degrees: 0,
            raster: RenderedRasterIdentity {
                path: PathBuf::from("/tmp/page-00000.png"),
                sha256: "rasterhash".to_string(),
                width_px: 2400,
                height_px: 3100,
            },
        });
        build_ocr_shard_inputs(&[manifest], &PdfOcrWorkerProfile::docling_compatible()).remove(0)
    }

    fn string_column<'a>(
        batch: &'a EngineRecordBatch,
        name: &str,
    ) -> Result<&'a StringArray, String> {
        batch
            .column_by_name(name)
            .ok_or_else(|| format!("missing `{name}` column"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| format!("`{name}` column is not Utf8"))
    }

    fn int32_column<'a>(
        batch: &'a EngineRecordBatch,
        name: &str,
    ) -> Result<&'a Int32Array, Status> {
        batch
            .column_by_name(name)
            .ok_or_else(|| Status::invalid_argument(format!("missing `{name}` column")))?
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| Status::invalid_argument(format!("`{name}` column is not Int32")))
    }
}
