//! Arrow Flight service for qianji run-console read-model batches.

use std::pin::Pin;
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{
    Action, ActionType, Criteria, Empty, FlightData, FlightDescriptor, FlightEndpoint, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use async_trait::async_trait;
use futures::stream;
use futures::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use xiuxian_qianji_control::{ControlLedger, RunId};

use crate::bpmn::run_console_read_model::{
    QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, QIANJI_RUN_CONSOLE_EVENT_ROUTE,
    QIANJI_RUN_CONSOLE_SCHEMA_VERSION, qianji_run_console_arrow_read_model,
};

/// Metadata header selecting the durable qianji control run.
pub const QIANJI_RUN_CONSOLE_RUN_ID_HEADER: &str = "x-qianji-run-id";
const WENDAO_SCHEMA_VERSION_HEADER: &str = "x-wendao-schema-version";

type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
type PutResultStream = Pin<Box<dyn Stream<Item = Result<PutResult, Status>> + Send>>;
type ActionResultStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
type ActionTypeStream = Pin<Box<dyn Stream<Item = Result<ActionType, Status>> + Send>>;

/// Read-only Arrow Flight service for qianji run-console rows.
#[derive(Clone)]
pub struct QianjiRunConsoleFlightService {
    ledger: Arc<dyn ControlLedger>,
}

impl std::fmt::Debug for QianjiRunConsoleFlightService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("QianjiRunConsoleFlightService")
            .finish_non_exhaustive()
    }
}

impl QianjiRunConsoleFlightService {
    /// Create a read-only qianji run-console Flight service.
    #[must_use]
    pub fn new(ledger: Arc<dyn ControlLedger>) -> Self {
        Self { ledger }
    }

    fn read_batch(
        &self,
        route: &str,
        metadata: &tonic::metadata::MetadataMap,
    ) -> Result<RecordBatch, Status> {
        validate_schema_version(metadata)?;
        let run_id = run_id_from_metadata(metadata)?;
        let events = self
            .ledger
            .load_events(&run_id)
            .map_err(|error| Status::internal(error.to_string()))?;
        let read_model = qianji_run_console_arrow_read_model(&run_id, &events)
            .map_err(|error| Status::internal(error.to_string()))?;
        match route {
            QIANJI_RUN_CONSOLE_EVENT_ROUTE => Ok(read_model.events),
            QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE => Ok(read_model.element_states),
            other => Err(Status::invalid_argument(format!(
                "unsupported qianji run-console Flight route `{other}`"
            ))),
        }
    }
}

#[async_trait]
impl FlightService for QianjiRunConsoleFlightService {
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
            "handshake is not used by qianji run-console Flight",
        ))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented(
            "list_flights is not used by qianji run-console Flight",
        ))
    }

    async fn get_flight_info(
        &self,
        request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        let metadata = request.metadata().clone();
        let descriptor = request.into_inner();
        let route = route_from_descriptor(&descriptor)?;
        let batch = self.read_batch(route.as_str(), &metadata)?;
        let total_records =
            i64::try_from(batch.num_rows()).map_err(|error| Status::internal(error.to_string()))?;
        let endpoint = FlightEndpoint::new().with_ticket(Ticket::new(route));
        let flight_info = FlightInfo::new()
            .try_with_schema(batch.schema().as_ref())
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
            "poll_flight_info is not used by qianji run-console Flight",
        ))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented(
            "get_schema is not used by qianji run-console Flight",
        ))
    }

    async fn do_get(
        &self,
        request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        let metadata = request.metadata().clone();
        let ticket = request.into_inner();
        let route = route_from_ticket(&ticket)?;
        let batch = self.read_batch(route.as_str(), &metadata)?;
        let response_stream = FlightDataEncoderBuilder::new()
            .build(stream::iter(vec![Ok::<
                RecordBatch,
                arrow_flight::error::FlightError,
            >(batch)]))
            .map(|item| item.map_err(|error| Status::internal(error.to_string())));
        Ok(Response::new(Box::pin(response_stream)))
    }

    async fn do_put(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        Err(Status::unimplemented(
            "do_put is not used by qianji run-console Flight",
        ))
    }

    async fn do_exchange(
        &self,
        _request: Request<tonic::Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented(
            "do_exchange is not used by qianji run-console Flight",
        ))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented(
            "do_action is not used by qianji run-console Flight",
        ))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented(
            "list_actions is not used by qianji run-console Flight",
        ))
    }
}

fn validate_schema_version(metadata: &tonic::metadata::MetadataMap) -> Result<(), Status> {
    let Some(value) = metadata.get(WENDAO_SCHEMA_VERSION_HEADER) else {
        return Ok(());
    };
    let value = value
        .to_str()
        .map_err(|error| Status::invalid_argument(error.to_string()))?;
    if value == QIANJI_RUN_CONSOLE_SCHEMA_VERSION {
        return Ok(());
    }
    Err(Status::invalid_argument(format!(
        "qianji run-console Flight schema version `{value}` does not match `{QIANJI_RUN_CONSOLE_SCHEMA_VERSION}`"
    )))
}

fn run_id_from_metadata(metadata: &tonic::metadata::MetadataMap) -> Result<RunId, Status> {
    let value = metadata
        .get(QIANJI_RUN_CONSOLE_RUN_ID_HEADER)
        .ok_or_else(|| Status::invalid_argument("missing x-qianji-run-id metadata"))?
        .to_str()
        .map_err(|error| Status::invalid_argument(error.to_string()))?;
    RunId::new(value.to_string()).map_err(|_| {
        Status::invalid_argument("x-qianji-run-id metadata must be a non-empty run id")
    })
}

fn route_from_descriptor(descriptor: &FlightDescriptor) -> Result<String, Status> {
    let route = descriptor.path.join("/");
    validate_route(&route)
}

fn route_from_ticket(ticket: &Ticket) -> Result<String, Status> {
    let route = std::str::from_utf8(ticket.ticket.as_ref())
        .map_err(|error| Status::invalid_argument(error.to_string()))?;
    validate_route(route)
}

fn validate_route(route: &str) -> Result<String, Status> {
    let normalized = route.trim_matches('/').to_string();
    match normalized.as_str() {
        QIANJI_RUN_CONSOLE_EVENT_ROUTE | QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE => Ok(normalized),
        _ => Err(Status::invalid_argument(format!(
            "unsupported qianji run-console Flight route `{route}`"
        ))),
    }
}
