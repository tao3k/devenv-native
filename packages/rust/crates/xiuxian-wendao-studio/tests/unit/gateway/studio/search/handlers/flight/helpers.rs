use crate::studio::arrow_types::{LanceRecordBatch, LanceStringArray};
use crate::transport::{WendaoFlightService, flight_descriptor_path};
use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightDescriptor, FlightInfo};
use serde::Serialize;
use tonic::Request;
use tonic::metadata::MetadataMap;

pub(super) fn assert_studio_flight_snapshot(name: &str, value: impl Serialize) {
    insta::with_settings!({
        snapshot_path => concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/gateway/studio"),
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}

pub(super) async fn fetch_flight_info<F>(
    service: &WendaoFlightService,
    route: &str,
    populate: F,
) -> (Vec<String>, FlightInfo)
where
    F: FnOnce(&mut MetadataMap),
{
    let descriptor_path =
        flight_descriptor_path(route).unwrap_or_else(|error| panic!("descriptor path: {error}"));
    let mut request = Request::new(FlightDescriptor::new_path(descriptor_path.clone()));
    populate(request.metadata_mut());
    let response = service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("route `{route}` should resolve: {error}"));
    (descriptor_path, response.into_inner())
}

pub(super) fn first_ticket(flight_info: &FlightInfo, context: &str) -> String {
    let Some(endpoint) = flight_info.endpoint.first() else {
        panic!("{context} should emit one ticket");
    };
    let Some(ticket) = endpoint.ticket.as_ref() else {
        panic!("{context} should emit one ticket");
    };
    String::from_utf8_lossy(ticket.ticket.as_ref()).into_owned()
}

pub(super) async fn assert_route_ticket<F>(
    service: &WendaoFlightService,
    route: &str,
    context: &str,
    populate: F,
) where
    F: FnOnce(&mut MetadataMap),
{
    let (_, flight_info) = fetch_flight_info(service, route, populate).await;
    assert_eq!(first_ticket(&flight_info, context), route);
}

pub(super) fn first_string(batch: &LanceRecordBatch, column: &str) -> String {
    batch
        .column_by_name(column)
        .unwrap_or_else(|| panic!("missing column `{column}`"))
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .unwrap_or_else(|| panic!("column `{column}` should be utf8"))
        .value(0)
        .to_string()
}
