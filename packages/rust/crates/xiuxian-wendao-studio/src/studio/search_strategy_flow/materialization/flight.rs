use arrow_flight::FlightData;
use arrow_flight::decode::FlightRecordBatchStream;
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightDescriptor, FlightInfo};
use futures::{StreamExt, TryStreamExt};
use tonic::metadata::MetadataMap;
use tonic::{Request, Status};
use xiuxian_db_store::LanceArray;

use crate::studio::arrow_types::{LanceRecordBatch, LanceStringArray, LanceUInt64Array};
use crate::transport::{
    WENDAO_GRAPH_DIRECTION_HEADER, WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER,
    WENDAO_GRAPH_NODE_ID_HEADER, WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_QUERY_HEADER, WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    WendaoFlightService, flight_descriptor_path,
};

use super::receipt::{
    RouteDecodedPayloadReceipt, RouteMaterializationReceipt, SearchStrategyFlowMaterializationError,
};

pub(super) async fn collect_route_batches<F>(
    service: &WendaoFlightService,
    route: &str,
    context: &str,
    populate: F,
) -> Result<Vec<LanceRecordBatch>, SearchStrategyFlowMaterializationError>
where
    F: Fn(&mut MetadataMap) -> Result<(), SearchStrategyFlowMaterializationError>,
{
    let (_, flight_info) = fetch_flight_info(service, route, |metadata| populate(metadata)).await?;
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!(
                "{context} should emit one ticket"
            ))
        })?;

    let mut request = Request::new(ticket);
    populate(request.metadata_mut())?;
    let frames = service
        .do_get(request)
        .await
        .map_err(|error| {
            SearchStrategyFlowMaterializationError::message(format!(
                "{context} should stream batches: {error}"
            ))
        })?
        .into_inner()
        .collect::<Vec<_>>()
        .await;
    decode_flight_batches(frames, context).await
}

pub(super) fn first_string(
    batch: &LanceRecordBatch,
    column: &str,
) -> Result<String, SearchStrategyFlowMaterializationError> {
    Ok(string_array(batch, column)?.value(0).to_string())
}

pub(super) fn first_u64(
    batch: &LanceRecordBatch,
    column: &str,
) -> Result<u64, SearchStrategyFlowMaterializationError> {
    Ok(batch
        .column_by_name(column)
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!("missing column `{column}`"))
        })?
        .as_any()
        .downcast_ref::<LanceUInt64Array>()
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!(
                "column `{column}` should be uint64"
            ))
        })?
        .value(0))
}

pub(super) fn string_values(
    batch: &LanceRecordBatch,
    column: &str,
) -> Result<Vec<String>, SearchStrategyFlowMaterializationError> {
    let array = string_array(batch, column)?;
    Ok((0..array.len())
        .filter(|index| !array.is_null(*index))
        .map(|index| array.value(index).to_string())
        .collect())
}

pub(super) fn route_receipt(
    route: &str,
    batches: &[LanceRecordBatch],
) -> Result<RouteMaterializationReceipt, SearchStrategyFlowMaterializationError> {
    RouteMaterializationReceipt::new(route, row_count(batches))
}

pub(super) fn decoded_payload_receipt(
    route: &str,
    batches: &[LanceRecordBatch],
    decoded_columns: Vec<&str>,
    evidence_anchor: String,
) -> Result<RouteDecodedPayloadReceipt, SearchStrategyFlowMaterializationError> {
    RouteDecodedPayloadReceipt::new(
        route,
        row_count(batches),
        decoded_columns.into_iter().map(str::to_string).collect(),
        evidence_anchor,
    )
}

pub(super) fn populate_repo_search_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    query_text: &str,
    limit: usize,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_REPO_HEADER, repo_id)?;
    insert_header(metadata, WENDAO_REPO_SEARCH_QUERY_HEADER, query_text)?;
    insert_header(
        metadata,
        WENDAO_REPO_SEARCH_LIMIT_HEADER,
        &limit.to_string(),
    )
}

pub(super) fn populate_repo_projected_page_index_tree_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
        page_id,
    )
}

pub(super) fn populate_repo_projected_retrieval_context_headers(
    metadata: &mut MetadataMap,
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: usize,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    populate_schema_headers(metadata)?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        repo_id,
    )?;
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        page_id,
    )?;
    if let Some(node_id) = node_id {
        insert_header(
            metadata,
            WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
            node_id,
        )?;
    }
    insert_header(
        metadata,
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        &related_limit.to_string(),
    )
}

pub(super) fn populate_graph_neighbors_headers(
    metadata: &mut MetadataMap,
    node_id: &str,
    direction: &str,
    hops: usize,
    limit: usize,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    populate_schema_headers(metadata)?;
    insert_header(metadata, WENDAO_GRAPH_NODE_ID_HEADER, node_id)?;
    insert_header(metadata, WENDAO_GRAPH_DIRECTION_HEADER, direction)?;
    insert_header(metadata, WENDAO_GRAPH_HOPS_HEADER, &hops.to_string())?;
    insert_header(metadata, WENDAO_GRAPH_LIMIT_HEADER, &limit.to_string())
}

pub(super) fn find_node_id_by_title(value: &serde_json::Value, title: &str) -> Option<String> {
    match value {
        serde_json::Value::Array(nodes) => nodes
            .iter()
            .find_map(|node| find_node_id_by_title(node, title)),
        serde_json::Value::Object(node) => {
            if node
                .get("title")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|node_title| node_title == title)
            {
                return node
                    .get("node_id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
            }
            node.get("children")
                .and_then(|children| find_node_id_by_title(children, title))
        }
        _ => None,
    }
}

async fn fetch_flight_info<F>(
    service: &WendaoFlightService,
    route: &str,
    populate: F,
) -> Result<(Vec<String>, FlightInfo), SearchStrategyFlowMaterializationError>
where
    F: FnOnce(&mut MetadataMap) -> Result<(), SearchStrategyFlowMaterializationError>,
{
    let descriptor_path = flight_descriptor_path(route).map_err(|error| {
        SearchStrategyFlowMaterializationError::message(format!("descriptor path: {error}"))
    })?;
    let mut request = Request::new(FlightDescriptor::new_path(descriptor_path.clone()));
    populate(request.metadata_mut())?;
    let response = service.get_flight_info(request).await.map_err(|error| {
        SearchStrategyFlowMaterializationError::message(format!(
            "route `{route}` should resolve: {error}"
        ))
    })?;
    Ok((descriptor_path, response.into_inner()))
}

async fn decode_flight_batches(
    frames: Vec<Result<FlightData, Status>>,
    context: &str,
) -> Result<Vec<LanceRecordBatch>, SearchStrategyFlowMaterializationError> {
    let stream = futures::stream::iter(
        frames
            .into_iter()
            .map(|frame| frame.map_err(arrow_flight::error::FlightError::from)),
    );
    let mut batch_stream = FlightRecordBatchStream::new_from_flight_data(stream);
    let mut batches = Vec::new();
    while let Some(batch) = batch_stream.try_next().await.map_err(|error| {
        SearchStrategyFlowMaterializationError::message(format!(
            "{context} should decode Flight batches: {error}"
        ))
    })? {
        batches.push(batch);
    }
    Ok(batches)
}

fn populate_schema_headers(
    metadata: &mut MetadataMap,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    insert_header(metadata, WENDAO_SCHEMA_VERSION_HEADER, "v2")
}

fn insert_header(
    metadata: &mut MetadataMap,
    header: &'static str,
    value: &str,
) -> Result<(), SearchStrategyFlowMaterializationError> {
    metadata.insert(
        header,
        value.parse().map_err(|error| {
            SearchStrategyFlowMaterializationError::message(format!(
                "invalid metadata value for `{header}`: {error}"
            ))
        })?,
    );
    Ok(())
}

fn string_array<'a>(
    batch: &'a LanceRecordBatch,
    column: &str,
) -> Result<&'a LanceStringArray, SearchStrategyFlowMaterializationError> {
    batch
        .column_by_name(column)
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!("missing column `{column}`"))
        })?
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .ok_or_else(|| {
            SearchStrategyFlowMaterializationError::message(format!(
                "column `{column}` should be utf8"
            ))
        })
}

fn row_count(batches: &[LanceRecordBatch]) -> usize {
    batches.iter().map(LanceRecordBatch::num_rows).sum()
}
