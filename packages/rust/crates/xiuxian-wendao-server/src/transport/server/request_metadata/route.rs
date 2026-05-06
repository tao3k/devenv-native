//! Route extraction and cache-key formatting helpers.

use std::collections::HashSet;

use arrow_flight::{FlightDescriptor, Ticket};
use tonic::Status;

use crate::transport::query_contract::{
    SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE,
    normalize_flight_route,
};

pub(crate) fn descriptor_route(descriptor: &FlightDescriptor) -> Result<String, Status> {
    let actual_path = descriptor
        .path
        .iter()
        .map(|segment| String::from_utf8_lossy(segment.as_ref()).into_owned())
        .collect::<Vec<_>>();
    normalize_flight_route(format!("/{}", actual_path.join("/"))).map_err(Status::invalid_argument)
}

pub(crate) fn ticket_route(ticket: &Ticket) -> Result<String, Status> {
    let route = String::from_utf8(ticket.ticket.to_vec())
        .map_err(|error| Status::invalid_argument(format!("invalid ticket bytes: {error}")))?;
    normalize_flight_route(route).map_err(Status::invalid_argument)
}

pub(crate) fn join_sorted_set(values: &HashSet<String>) -> String {
    let mut sorted = values.iter().cloned().collect::<Vec<_>>();
    sorted.sort();
    sorted.join(",")
}

pub(crate) fn is_search_family_route(route: &str) -> bool {
    matches!(
        route,
        SEARCH_INTENT_ROUTE
            | SEARCH_KNOWLEDGE_ROUTE
            | SEARCH_REFERENCES_ROUTE
            | SEARCH_SYMBOLS_ROUTE
    )
}
