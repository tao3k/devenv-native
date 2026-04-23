use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use arrow::datatypes::Schema;
use arrow_flight::encode::FlightDataEncoderBuilder;
use arrow_flight::error::FlightError;
use arrow_flight::sql::{ProstMessageExt, TicketStatementQuery};
use arrow_flight::{FlightData, FlightDescriptor, FlightEndpoint, FlightInfo, Ticket};
use prost::Message;
use tokio_stream::StreamExt;
use tonic::{Response, Status};
use uuid::Uuid;
use xiuxian_db_store::EngineRecordBatch;

pub(super) type StatementCache = Arc<Mutex<HashMap<String, Vec<EngineRecordBatch>>>>;
pub(super) type DoGetResponseStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<FlightData, Status>> + Send + 'static>>;

pub(super) fn new_statement_cache() -> StatementCache {
    Arc::new(Mutex::new(HashMap::new()))
}

pub(super) fn new_statement_handle() -> String {
    Uuid::new_v4().to_string()
}

pub(super) fn cache_statement_batches(
    cache: &StatementCache,
    statement_handle: String,
    batches: Vec<EngineRecordBatch>,
) {
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(statement_handle, batches);
}

pub(super) fn statement_flight_info(
    descriptor: FlightDescriptor,
    statement_handle: &str,
    batches: &[EngineRecordBatch],
) -> Result<FlightInfo, Status> {
    let schema = batches
        .first()
        .map_or_else(|| Arc::new(Schema::empty()), EngineRecordBatch::schema);
    let total_records = batches.iter().fold(0_i64, |total, batch| {
        total.saturating_add(i64::try_from(batch.num_rows()).unwrap_or(i64::MAX))
    });
    let total_bytes = batches.iter().fold(0_i64, |total, batch| {
        total.saturating_add(i64::try_from(batch.get_array_memory_size()).unwrap_or(i64::MAX))
    });
    let ticket = Ticket::new(
        TicketStatementQuery {
            statement_handle: statement_handle.as_bytes().to_vec().into(),
        }
        .as_any()
        .encode_to_vec(),
    );
    let endpoint = FlightEndpoint::new().with_ticket(ticket);

    FlightInfo::new()
        .try_with_schema(schema.as_ref())
        .map_err(|error| {
            Status::internal(format!(
                "FlightSQL failed to encode statement schema: {error}"
            ))
        })
        .map(|flight_info| {
            flight_info
                .with_endpoint(endpoint)
                .with_descriptor(descriptor)
                .with_total_records(total_records)
                .with_total_bytes(total_bytes)
        })
}

pub(super) fn take_statement_batches(
    cache: &StatementCache,
    ticket: &TicketStatementQuery,
) -> Result<Vec<EngineRecordBatch>, Status> {
    let statement_handle =
        String::from_utf8(ticket.statement_handle.to_vec()).map_err(|error| {
            Status::invalid_argument(format!(
                "FlightSQL statement ticket handle must be valid UTF-8: {error}"
            ))
        })?;
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&statement_handle)
        .ok_or_else(|| {
            Status::not_found(format!(
                "FlightSQL statement handle `{statement_handle}` is unknown or already consumed"
            ))
        })
}

pub(super) fn response_stream(
    schema: Arc<Schema>,
    batches: Vec<EngineRecordBatch>,
) -> Response<DoGetResponseStream> {
    let stream = FlightDataEncoderBuilder::new()
        .with_schema(schema)
        .build(tokio_stream::iter(
            batches
                .into_iter()
                .map(Ok::<EngineRecordBatch, FlightError>),
        ))
        .map(|frame| frame.map_err(Status::from));
    Response::new(Box::pin(stream))
}
