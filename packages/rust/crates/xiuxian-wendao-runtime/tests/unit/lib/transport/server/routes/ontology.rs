use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::{Code, Request};

use crate::tests::transport::server::assertions::{
    metadata_value, must_err, must_ok, parse_json, route_descriptor,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::metadata::ontology::{
    healthcare_manifest, insert_dataset_ontology_metadata,
};
use crate::tests::transport::server::providers::RecordingDatasetOntologyMaterializeProvider;
use crate::transport::{ONTOLOGY_DATASET_MATERIALIZE_ROUTE, WENDAO_SCHEMA_VERSION_HEADER};

#[tokio::test]
async fn dataset_ontology_materialize_route_returns_unimplemented_without_provider() {
    let service = build_service_with_route_providers(|_route_providers| {});
    let mut request = Request::new(route_descriptor(ONTOLOGY_DATASET_MATERIALIZE_ROUTE));
    insert_dataset_ontology_metadata(request.metadata_mut(), &healthcare_manifest());

    let error = must_err(
        service.get_flight_info(request).await,
        "dataset ontology route should be admitted then stop at provider execution",
    );

    assert_eq!(error.code(), Code::Unimplemented);
    assert_eq!(
        error.message(),
        "dataset ontology materialize Flight route `/ontology/dataset/materialize` is not configured for this runtime host"
    );
}

#[tokio::test]
async fn dataset_ontology_materialize_route_calls_configured_provider() {
    let provider = Arc::new(RecordingDatasetOntologyMaterializeProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.dataset_ontology_materialize = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ONTOLOGY_DATASET_MATERIALIZE_ROUTE));
    insert_dataset_ontology_metadata(request.metadata_mut(), &healthcare_manifest());

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "dataset ontology route should call the configured provider",
    )
    .into_inner();
    let app_metadata = parse_json(
        &flight_info.app_metadata,
        "dataset ontology app metadata should decode",
    );

    assert_eq!(flight_info.total_records, 1);
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some((
            "healthcare.synthetic_care_delivery.contract.v1".to_string(),
            "healthcare.synthetic_care_delivery.v1".to_string(),
            vec!["raw_patients".to_string(), "raw_providers".to_string()],
        ))
    );
    assert_eq!(
        app_metadata["contractId"],
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(
        app_metadata["mappingId"],
        "healthcare.synthetic_care_delivery.v1"
    );
    assert_eq!(app_metadata["tableCount"], 2);
}

#[tokio::test]
async fn dataset_ontology_materialize_route_rejects_invalid_manifest_metadata() {
    let service = build_service_with_route_providers(|_route_providers| {});
    let mut request = Request::new(route_descriptor(ONTOLOGY_DATASET_MATERIALIZE_ROUTE));
    request.metadata_mut().insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata"),
    );

    let error = must_err(
        service.get_flight_info(request).await,
        "missing dataset ontology metadata should fail admission",
    );

    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(
        error.message(),
        "missing required header `x-wendao-dataset-ontology-contract-id`"
    );
}
