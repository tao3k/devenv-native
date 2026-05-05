use std::sync::Arc;

use arrow_flight::Ticket;
use arrow_flight::flight_service_server::FlightService;
use futures::StreamExt;
use tonic::Request;

use crate::transport::QUERY_SQL_ROUTE;

use crate::tests::transport::server::assertions::{
    batch_column, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::{
    build_service_with_route_providers, decode_flight_batches,
};
use crate::tests::transport::server::providers::RecordingSqlProvider;
use crate::tests::transport::server::request_headers::populate_schema_and_sql_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_sql_provider() {
    let provider = Arc::new(RecordingSqlProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.sql = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(QUERY_SQL_ROUTE));
    populate_schema_and_sql_headers(
        request.metadata_mut(),
        "SELECT table_name, row_id FROM repo_entity",
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "SQL route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "SQL route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, QUERY_SQL_ROUTE);
    assert_eq!(flight_info.total_records, 2);
    assert_eq!(
        app_metadata["query"],
        "SELECT table_name, row_id FROM repo_entity"
    );
    assert_eq!(app_metadata["batchCount"], 2);
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_do_get_streams_multi_batch_sql_response() {
    let provider = Arc::new(RecordingSqlProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.sql = Some(provider.clone());
    });
    let mut request = Request::new(Ticket::new(QUERY_SQL_ROUTE.to_string()));
    populate_schema_and_sql_headers(
        request.metadata_mut(),
        "SELECT table_name, row_id FROM repo_entity",
    );

    let frames = must_ok(
        service.do_get(request).await,
        "SQL route should stream through the pluggable provider",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;
    let batches = decode_flight_batches(frames).await;
    let first_names = batch_column::<arrow_array::StringArray>(
        &batches[0],
        "table_name",
        "table_name should decode as Utf8",
    );
    let second_names = batch_column::<arrow_array::StringArray>(
        &batches[1],
        "table_name",
        "table_name should decode as Utf8",
    );

    assert_eq!(batches.len(), 2);
    assert_eq!(first_names.value(0), "repo_entity");
    assert_eq!(second_names.value(0), "repo_content_chunk");
    assert_eq!(provider.call_count(), 1);
}
