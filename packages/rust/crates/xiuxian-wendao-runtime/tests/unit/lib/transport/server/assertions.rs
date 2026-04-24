use std::fmt::Display;

use arrow_array::RecordBatch;
use arrow_flight::{FlightDescriptor, FlightInfo};
use tonic::metadata::{Ascii, MetadataValue};
use xiuxian_db_store::LanceRecordBatch;

use crate::transport::flight_descriptor_path;

pub(super) fn must_ok<T, E: Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn must_err<T, E>(result: Result<T, E>, context: &str) -> E {
    match result {
        Ok(_) => panic!("{context}"),
        Err(error) => error,
    }
}

pub(super) fn must_some<T>(option: Option<T>, context: &str) -> T {
    option.unwrap_or_else(|| panic!("{context}"))
}

pub(super) fn route_descriptor(route: &str) -> FlightDescriptor {
    let path = must_ok(
        flight_descriptor_path(route),
        "Flight descriptor path should build",
    );
    FlightDescriptor::new_path(path)
}

pub(super) fn ticket_string(flight_info: &FlightInfo, context: &str) -> String {
    let ticket = must_some(
        flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.as_ref()),
        context,
    );
    String::from_utf8_lossy(ticket.ticket.as_ref()).into_owned()
}

pub(super) fn parse_json(bytes: &[u8], context: &str) -> serde_json::Value {
    must_ok(serde_json::from_slice(bytes), context)
}

pub(super) fn metadata_value(raw: &str, context: &str) -> MetadataValue<Ascii> {
    must_ok(MetadataValue::try_from(raw), context)
}

pub(super) fn batch_column<'a, T: 'static>(
    batch: &'a RecordBatch,
    name: &str,
    context: &str,
) -> &'a T {
    must_some(
        batch
            .column_by_name(name)
            .and_then(|column| column.as_any().downcast_ref::<T>()),
        context,
    )
}

pub(super) fn lance_batch_column<'a, T: 'static>(
    batch: &'a LanceRecordBatch,
    name: &str,
    context: &str,
) -> &'a T {
    must_some(
        batch
            .column_by_name(name)
            .and_then(|column| column.as_any().downcast_ref::<T>()),
        context,
    )
}
