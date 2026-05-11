use arrow_flight::{FlightDescriptor, Ticket};
use tonic::Status;
use tonic::metadata::MetadataMap;

use crate::transport::query_contract::{
    SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE,
    WENDAO_RERANK_DIMENSION_HEADER, WENDAO_RERANK_MIN_FINAL_SCORE_HEADER,
    WENDAO_RERANK_TOP_K_HEADER, WENDAO_SCHEMA_VERSION_HEADER, normalize_flight_route,
};

pub(crate) fn validate_schema_version(
    metadata: &MetadataMap,
    expected_schema_version: &str,
) -> Result<(), Status> {
    let schema_version = metadata
        .get(WENDAO_SCHEMA_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if schema_version != expected_schema_version {
        return Err(Status::invalid_argument(format!(
            "unexpected schema version header: {schema_version}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_rerank_dimension_header(metadata: &MetadataMap) -> Result<usize, Status> {
    let dimension = metadata
        .get(WENDAO_RERANK_DIMENSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let parsed_dimension = dimension.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank dimension header `{WENDAO_RERANK_DIMENSION_HEADER}`: {dimension}"
        ))
    })?;
    if parsed_dimension == 0 {
        return Err(Status::invalid_argument(format!(
            "rerank dimension header `{WENDAO_RERANK_DIMENSION_HEADER}` must be greater than zero"
        )));
    }
    Ok(parsed_dimension)
}

pub(crate) fn validate_rerank_top_k_header(
    metadata: &MetadataMap,
) -> Result<Option<usize>, Status> {
    let Some(raw_value) = metadata.get(WENDAO_RERANK_TOP_K_HEADER) else {
        return Ok(None);
    };
    let top_k = raw_value.to_str().unwrap_or_default();
    let parsed_top_k = top_k.parse::<usize>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank top_k header `{WENDAO_RERANK_TOP_K_HEADER}`: {top_k}"
        ))
    })?;
    if parsed_top_k == 0 {
        return Err(Status::invalid_argument(format!(
            "rerank top_k header `{WENDAO_RERANK_TOP_K_HEADER}` must be greater than zero"
        )));
    }
    Ok(Some(parsed_top_k))
}

pub(crate) fn validate_rerank_min_final_score_header(
    metadata: &MetadataMap,
) -> Result<Option<f64>, Status> {
    let Some(raw_value) = metadata.get(WENDAO_RERANK_MIN_FINAL_SCORE_HEADER) else {
        return Ok(None);
    };
    let min_final_score = raw_value.to_str().unwrap_or_default();
    let parsed_min_final_score = min_final_score.parse::<f64>().map_err(|_| {
        Status::invalid_argument(format!(
            "invalid rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}`: {min_final_score}"
        ))
    })?;
    if !parsed_min_final_score.is_finite() {
        return Err(Status::invalid_argument(format!(
            "rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}` must be finite"
        )));
    }
    if !(0.0..=1.0).contains(&parsed_min_final_score) {
        return Err(Status::invalid_argument(format!(
            "rerank min_final_score header `{WENDAO_RERANK_MIN_FINAL_SCORE_HEADER}` must stay within inclusive range [0.0, 1.0]"
        )));
    }
    Ok(Some(parsed_min_final_score))
}

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

pub(crate) fn join_sorted_set(values: &std::collections::HashSet<String>) -> String {
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

pub(super) fn split_non_empty_header_values(
    metadata: &MetadataMap,
    header: &'static str,
) -> Vec<String> {
    metadata
        .get(header)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|value| !value.is_empty() || metadata.contains_key(header))
        .map(ToString::to_string)
        .collect()
}
