use std::pin::Pin;

use arrow::array::StringArray;
use arrow::record_batch::RecordBatch;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_server::{FlightService, FlightServiceServer};
use arrow_flight::{
    Action, Criteria, Empty, FlightData, FlightDescriptor, FlightInfo, HandshakeRequest,
    HandshakeResponse, PollInfo, SchemaResult, Ticket,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_wendao_runtime::transport::WENDAO_SCHEMA_VERSION_HEADER;

use super::modelica;
use super::parser::build_response_rows;
use super::rows::{ParserSummaryRequest, rows_to_batch};
use super::schema::{REQUEST_ID, SOURCE_ID, SOURCE_TEXT};

type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
type PutResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::PutResult, Status>> + Send>>;
type ActionResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
type ActionTypeStream =
    Pin<Box<dyn Stream<Item = Result<arrow_flight::ActionType, Status>> + Send>>;

#[derive(Clone)]
struct ParserSummaryFlightService;

pub(crate) struct FakeParserSummaryServiceGuard {
    runtime: tokio::runtime::Runtime,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for FakeParserSummaryServiceGuard {
    fn drop(&mut self) {
        self.server.abort();
        let _ = &self.runtime;
    }
}

pub(crate) fn spawn_fake_julia_parser_summary_service()
-> Result<(String, FakeParserSummaryServiceGuard), String> {
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("wendao-repo-parser-summary-fake")
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        let (base_url, server) = runtime.block_on(spawn_service())?;
        Ok((base_url, FakeParserSummaryServiceGuard { runtime, server }))
    })
    .join()
    .map_err(|_| "fake parser-summary service thread panicked".to_string())?
}

async fn spawn_service() -> Result<(String, tokio::task::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("bind fake parser-summary service: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("read fake parser-summary listener address: {error}"))?;
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(FlightServiceServer::new(ParserSummaryFlightService))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap_or_else(|error| panic!("fake parser-summary service should serve: {error}"));
    });
    Ok((format!("http://{address}"), server))
}

#[async_trait]
impl FlightService for ParserSummaryFlightService {
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
        Err(Status::unimplemented("handshake is not used"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("list_flights is not used"))
    }

    async fn get_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented("get_flight_info is not used"))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented("poll_flight_info is not used"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("get_schema is not used"))
    }

    async fn do_get(
        &self,
        _request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        Err(Status::unimplemented("do_get is not used"))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented("do_put is not used"))
    }

    async fn do_exchange(
        &self,
        request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        validate_schema_version(&request)?;
        let request_batches = decode_request_batches(request.into_inner()).await?;
        let response_batch = response_batch_for_requests(request_batches.as_slice())?;
        let response_stream = FlightDataEncoderBuilder::new()
            .build(tokio_stream::iter(vec![Ok::<
                RecordBatch,
                arrow_flight::error::FlightError,
            >(response_batch)]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("do_action is not used"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("list_actions is not used"))
    }
}

fn validate_schema_version(request: &Request<tonic::Streaming<FlightData>>) -> Result<(), Status> {
    let schema_version = request
        .metadata()
        .get(WENDAO_SCHEMA_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if schema_version != "v3" {
        return Err(Status::invalid_argument(format!(
            "unexpected parser-summary schema version `{schema_version}`"
        )));
    }
    Ok(())
}

async fn decode_request_batches(
    stream: tonic::Streaming<FlightData>,
) -> Result<Vec<RecordBatch>, Status> {
    let flight_data = stream.map_err(arrow_flight::error::FlightError::from);
    let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(flight_data);
    let mut batches = Vec::new();
    while let Some(batch) = batch_stream.next().await {
        batches.push(batch.map_err(|error| Status::invalid_argument(error.to_string()))?);
    }
    if batches.is_empty() {
        return Err(Status::invalid_argument(
            "parser-summary request stream returned no batches",
        ));
    }
    Ok(batches)
}

fn response_batch_for_requests(request_batches: &[RecordBatch]) -> Result<RecordBatch, Status> {
    let mut requests = Vec::new();
    for batch in request_batches {
        requests.extend(request_rows(batch)?);
    }
    if requests
        .first()
        .is_some_and(|request| request.request_id.starts_with("modelica-file-summary:"))
    {
        return modelica::response_batch_for_requests(requests.as_slice())
            .map_err(|error| Status::internal(error.to_string()));
    }
    if requests
        .first()
        .is_some_and(|request| request.request_id.starts_with("modelica-ast-query:"))
    {
        return modelica::ast_query_response_batch_for_requests(requests.as_slice())
            .map_err(|error| Status::internal(error.to_string()));
    }
    let mut rows = Vec::new();
    for request in requests {
        rows.extend(build_response_rows(&request));
    }
    rows_to_batch(rows.as_slice()).map_err(|error| Status::internal(error.to_string()))
}

fn request_rows(batch: &RecordBatch) -> Result<Vec<ParserSummaryRequest>, Status> {
    let request_ids = string_column(batch, REQUEST_ID)?;
    let source_ids = string_column(batch, SOURCE_ID)?;
    let source_texts = string_column(batch, SOURCE_TEXT)?;
    Ok((0..batch.num_rows())
        .map(|index| ParserSummaryRequest {
            request_id: request_ids.value(index).to_string(),
            source_id: source_ids.value(index).to_string(),
            source_text: source_texts.value(index).to_string(),
        })
        .collect())
}

fn string_column<'a>(batch: &'a RecordBatch, column_name: &str) -> Result<&'a StringArray, Status> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| Status::invalid_argument(format!("missing Utf8 column `{column_name}`")))
}
