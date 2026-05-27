use std::pin::Pin;
use std::sync::{Arc, Mutex};

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_server::{FlightService, FlightServiceServer};
use arrow_flight::{
    Action, ActionType, Criteria, Empty, FlightData, FlightDescriptor, FlightEndpoint, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt, stream};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_llm::model_routing::{
    WENDAO_ROUTE_ID_HEADER, WENDAO_ROUTE_MODALITY_HEADER, WENDAO_ROUTE_PRECISION_TIER_HEADER,
    WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER, WENDAO_ROUTE_SELECTED_MODEL_HEADER,
    WENDAO_ROUTE_SELECTED_PROVIDER_HEADER, WENDAO_ROUTE_TASK_KIND_HEADER,
};
use xiuxian_wendao_server::transport::{
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
};

type BoxFlightStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedDocumentExtractRequest {
    pub(crate) descriptor_path: Vec<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) output_dir: Option<String>,
    pub(crate) profile: Option<String>,
    pub(crate) route_id: Option<String>,
    pub(crate) route_task_kind: Option<String>,
    pub(crate) route_modality: Option<String>,
    pub(crate) route_selected_provider: Option<String>,
    pub(crate) route_selected_model: Option<String>,
    pub(crate) route_selected_backend_profile: Option<String>,
    pub(crate) route_precision_tier: Option<String>,
}

#[derive(Clone)]
struct DocumentExtractTestFlightService {
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedDocumentExtractRequest>>>,
    observed_requests: Arc<Mutex<Vec<ObservedDocumentExtractRequest>>>,
}

#[async_trait]
impl FlightService for DocumentExtractTestFlightService {
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
        request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let metadata = request.metadata().clone();
        let descriptor = request.into_inner();
        let observed_request = ObservedDocumentExtractRequest {
            descriptor_path: descriptor.path.clone(),
            source_path: metadata
                .get(WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned),
            output_dir: metadata
                .get(WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned),
            profile: metadata
                .get(WENDAO_DOCUMENT_EXTRACT_PROFILE_HEADER)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned),
            route_id: metadata_header(&metadata, WENDAO_ROUTE_ID_HEADER),
            route_task_kind: metadata_header(&metadata, WENDAO_ROUTE_TASK_KIND_HEADER),
            route_modality: metadata_header(&metadata, WENDAO_ROUTE_MODALITY_HEADER),
            route_selected_provider: metadata_header(
                &metadata,
                WENDAO_ROUTE_SELECTED_PROVIDER_HEADER,
            ),
            route_selected_model: metadata_header(&metadata, WENDAO_ROUTE_SELECTED_MODEL_HEADER),
            route_selected_backend_profile: metadata_header(
                &metadata,
                WENDAO_ROUTE_SELECTED_BACKEND_PROFILE_HEADER,
            ),
            route_precision_tier: metadata_header(&metadata, WENDAO_ROUTE_PRECISION_TIER_HEADER),
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

        let endpoint = FlightEndpoint::new().with_ticket(Ticket::new("document-extract"));
        let total_records = i64::try_from(self.response_batch.num_rows())
            .map_err(|_| Status::internal("response batch row count exceeds i64"))?;
        let flight_info = FlightInfo::new()
            .try_with_schema(self.response_batch.schema().as_ref())
            .map_err(|error| Status::internal(error.to_string()))?
            .with_endpoint(endpoint)
            .with_descriptor(descriptor)
            .with_total_records(total_records);
        Ok(Response::new(flight_info))
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
        let batch = self.response_batch.clone();
        let response_stream = FlightDataEncoderBuilder::new()
            .build(stream::iter(vec![Ok::<
                EngineRecordBatch,
                arrow_flight::error::FlightError,
            >(batch)]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put is not used by this test"))
    }

    async fn do_exchange(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented(
            "do_exchange is not used by this test",
        ))
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

pub(crate) async fn spawn_document_extract_service(
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedDocumentExtractRequest>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    spawn_document_extract_service_with_observed_requests(
        response_batch,
        observed,
        Arc::new(Mutex::new(Vec::new())),
    )
    .await
}

pub(crate) async fn spawn_document_extract_service_with_observed_requests(
    response_batch: EngineRecordBatch,
    observed: Arc<Mutex<Option<ObservedDocumentExtractRequest>>>,
    observed_requests: Arc<Mutex<Vec<ObservedDocumentExtractRequest>>>,
) -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("listener should bind: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("listener should expose an address: {error}"))?;
    let service = DocumentExtractTestFlightService {
        response_batch,
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

fn metadata_header(metadata: &tonic::metadata::MetadataMap, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}
