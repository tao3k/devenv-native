use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

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
use tokio::sync::oneshot;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use xiuxian_julia_core::JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION;
use xiuxian_wendao_runtime::transport::WENDAO_SCHEMA_VERSION_HEADER;

use super::response::response_batch;

type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
type PutResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::PutResult, Status>> + Send>>;
type ActionResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
type ActionTypeStream =
    Pin<Box<dyn Stream<Item = Result<arrow_flight::ActionType, Status>> + Send>>;

#[derive(Clone)]
struct GraphStructuralFlightService {
    base_url: Arc<str>,
}

pub(crate) struct FakeGraphStructuralServiceGuard {
    shutdown: Option<oneshot::Sender<()>>,
    server: Option<thread::JoinHandle<()>>,
}

impl FakeGraphStructuralServiceGuard {
    pub(crate) fn kill(&mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

impl Drop for FakeGraphStructuralServiceGuard {
    fn drop(&mut self) {
        self.shutdown();
        let _ = self.server.take();
    }
}

pub(crate) fn spawn_fake_graph_structural_service()
-> Result<(String, FakeGraphStructuralServiceGuard), String> {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = thread::Builder::new()
        .name("wendao-graph-structural-fake".to_string())
        .spawn(move || run_service_thread(ready_tx, shutdown_rx))
        .map_err(|error| format!("spawn fake graph-structural service thread: {error}"))?;
    let base_url = ready_rx
        .recv()
        .map_err(|error| format!("wait for fake graph-structural service readiness: {error}"))??;
    Ok((
        base_url,
        FakeGraphStructuralServiceGuard {
            shutdown: Some(shutdown_tx),
            server: Some(server),
        },
    ))
}

fn run_service_thread(
    ready_tx: mpsc::Sender<Result<String, String>>,
    shutdown_rx: oneshot::Receiver<()>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| panic!("build fake graph-structural runtime: {error}"));
    runtime.block_on(async move {
        if let Err(error) = spawn_service(ready_tx, shutdown_rx).await {
            panic!("fake graph-structural service should serve: {error}");
        }
    });
}

async fn spawn_service(
    ready_tx: mpsc::Sender<Result<String, String>>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("bind fake graph-structural service: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("read fake graph-structural listener address: {error}"))?;
    let base_url = format!("http://{address}");
    let service = GraphStructuralFlightService {
        base_url: Arc::<str>::from(base_url.as_str()),
    };
    ready_tx
        .send(Ok(base_url))
        .map_err(|_| "publish fake graph-structural service readiness".to_string())?;
    Server::builder()
        .add_service(FlightServiceServer::new(service))
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
            let _ = shutdown_rx.await;
        })
        .await
        .map_err(|error| format!("serve fake graph-structural service: {error}"))?;
    Ok(())
}

#[async_trait]
impl FlightService for GraphStructuralFlightService {
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
        let response_batch = response_batch(self.base_url.as_ref(), request_batches.as_slice())?;
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
    if schema_version != JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION {
        return Err(Status::invalid_argument(format!(
            "unexpected graph-structural schema version `{schema_version}`"
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
            "graph-structural request stream returned no batches",
        ));
    }
    Ok(batches)
}
