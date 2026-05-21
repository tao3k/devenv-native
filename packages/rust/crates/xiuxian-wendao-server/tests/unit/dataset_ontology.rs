use std::sync::{Arc, Mutex};

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_flight::FlightDescriptor;
use arrow_flight::flight_service_server::FlightService;
use arrow_schema::{DataType, Field, Schema};
use async_trait::async_trait;
use tonic::{Code, Request};
use xiuxian_wendao_server::transport::{
    DatasetOntologyFlightManifest, DatasetOntologyMaterializeFlightRouteProvider,
    DatasetOntologyMaterializeFlightRouteResponse, DatasetOntologySourceTablePayload,
    ONTOLOGY_DATASET_MATERIALIZE_ROUTE, RepoSearchFlightRequest, RepoSearchFlightRouteProvider,
    RerankScoreWeights, WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
    WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER, WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
    WENDAO_SCHEMA_VERSION_HEADER, WendaoFlightRouteProviders, WendaoFlightService,
    encode_dataset_ontology_manifest_header, flight_descriptor_path,
    validate_dataset_ontology_flight_manifest,
};

const TEST_SCHEMA_VERSION: &str = "dataset-ontology-smoke";

#[test]
fn dataset_ontology_manifest_contract_rejects_duplicate_payloads() {
    let manifest = DatasetOntologyFlightManifest::new(
        "healthcare-fixture",
        "healthcare-v1",
        vec![
            DatasetOntologySourceTablePayload::new("patients", "raw-table"),
            DatasetOntologySourceTablePayload::new("encounters", "raw-table"),
        ],
    );

    let error = validate_dataset_ontology_flight_manifest(&manifest)
        .expect_err("duplicate payload ids must be rejected");

    assert!(error.contains("duplicate payload id"));
}

#[tokio::test]
async fn dataset_ontology_route_requires_provider_after_metadata_admission() {
    let providers = WendaoFlightRouteProviders::new(Arc::new(FakeRepoSearchProvider));
    let service = service_with_providers(providers);
    let mut request = Request::new(dataset_ontology_descriptor());
    insert_dataset_ontology_metadata(request.metadata_mut(), &dataset_ontology_manifest());

    let error = service
        .get_flight_info(request)
        .await
        .expect_err("missing dataset ontology provider must fail");

    assert_eq!(error.code(), Code::Unimplemented);
    assert!(error.message().contains("dataset ontology materialize"));
}

#[tokio::test]
async fn dataset_ontology_route_rejects_mismatched_contract_metadata() {
    let mut providers = WendaoFlightRouteProviders::new(Arc::new(FakeRepoSearchProvider));
    providers.dataset_ontology_materialize = Some(Arc::new(FakeDatasetOntologyProvider::default()));
    let service = service_with_providers(providers);
    let manifest = dataset_ontology_manifest();
    let mut request = Request::new(dataset_ontology_descriptor());
    insert_dataset_ontology_metadata(request.metadata_mut(), &manifest);
    request.metadata_mut().insert(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        "wrong-contract"
            .parse()
            .unwrap_or_else(|error| panic!("contract metadata: {error}")),
    );

    let error = service
        .get_flight_info(request)
        .await
        .expect_err("mismatched contract metadata must fail");

    assert_eq!(error.code(), Code::InvalidArgument);
    assert!(error.message().contains("does not match manifest"));
}

#[tokio::test]
async fn dataset_ontology_route_calls_configured_provider_and_preserves_metadata() {
    let observed_manifest = Arc::new(Mutex::new(None));
    let mut providers = WendaoFlightRouteProviders::new(Arc::new(FakeRepoSearchProvider));
    providers.dataset_ontology_materialize = Some(Arc::new(FakeDatasetOntologyProvider {
        observed_manifest: Arc::clone(&observed_manifest),
    }));
    let service = service_with_providers(providers);
    let manifest = dataset_ontology_manifest();
    let mut request = Request::new(dataset_ontology_descriptor());
    insert_dataset_ontology_metadata(request.metadata_mut(), &manifest);

    let flight_info = service
        .get_flight_info(request)
        .await
        .unwrap_or_else(|error| panic!("dataset ontology get_flight_info: {error}"))
        .into_inner();

    assert_eq!(flight_info.total_records, 1);
    assert_eq!(
        flight_info.app_metadata.as_ref(),
        br#"{"datasetOntologyMaterialization":{"status":"ready"}}"#
    );
    let observed = observed_manifest
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_else(|| panic!("dataset ontology provider should receive manifest"));
    assert_eq!(observed, manifest);
}

#[derive(Debug)]
struct FakeRepoSearchProvider;

#[async_trait]
impl RepoSearchFlightRouteProvider for FakeRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        _request: &RepoSearchFlightRequest,
    ) -> Result<RecordBatch, String> {
        Ok(empty_batch())
    }
}

#[derive(Debug, Default)]
struct FakeDatasetOntologyProvider {
    observed_manifest: Arc<Mutex<Option<DatasetOntologyFlightManifest>>>,
}

#[async_trait]
impl DatasetOntologyMaterializeFlightRouteProvider for FakeDatasetOntologyProvider {
    async fn dataset_ontology_materialize_batch(
        &self,
        manifest: &DatasetOntologyFlightManifest,
    ) -> Result<DatasetOntologyMaterializeFlightRouteResponse, String> {
        *self
            .observed_manifest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(manifest.clone());
        Ok(
            DatasetOntologyMaterializeFlightRouteResponse::new(dataset_ontology_batch())
                .with_app_metadata(
                    br#"{"datasetOntologyMaterialization":{"status":"ready"}}"#.as_slice(),
                ),
        )
    }
}

fn service_with_providers(providers: WendaoFlightRouteProviders) -> WendaoFlightService {
    WendaoFlightService::new_with_route_providers(
        TEST_SCHEMA_VERSION,
        providers,
        1,
        RerankScoreWeights::default(),
    )
    .unwrap_or_else(|error| panic!("build dataset ontology Flight service: {error}"))
}

fn dataset_ontology_descriptor() -> FlightDescriptor {
    FlightDescriptor::new_path(
        flight_descriptor_path(ONTOLOGY_DATASET_MATERIALIZE_ROUTE)
            .unwrap_or_else(|error| panic!("dataset ontology descriptor path: {error}")),
    )
}

fn insert_dataset_ontology_metadata(
    metadata: &mut tonic::metadata::MetadataMap,
    manifest: &DatasetOntologyFlightManifest,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        TEST_SCHEMA_VERSION
            .parse()
            .unwrap_or_else(|error| panic!("schema metadata value: {error}")),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        manifest
            .contract_id
            .parse()
            .unwrap_or_else(|error| panic!("contract metadata value: {error}")),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        manifest
            .mapping_id
            .parse()
            .unwrap_or_else(|error| panic!("mapping metadata value: {error}")),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
        encode_dataset_ontology_manifest_header(manifest)
            .unwrap_or_else(|error| panic!("encode manifest metadata: {error}"))
            .parse()
            .unwrap_or_else(|error| panic!("manifest metadata value: {error}")),
    );
}

fn dataset_ontology_manifest() -> DatasetOntologyFlightManifest {
    DatasetOntologyFlightManifest::new(
        "healthcare-fixture",
        "healthcare-v1",
        vec![
            DatasetOntologySourceTablePayload::new("patients", "patients-arrow")
                .with_row_count(2)
                .with_content_sha256(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                )
                .with_schema_fingerprint("schema-patients-v1"),
            DatasetOntologySourceTablePayload::new("encounters", "encounters-arrow")
                .with_row_count(3)
                .with_content_sha256(
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                )
                .with_schema_fingerprint("schema-encounters-v1"),
        ],
    )
}

fn empty_batch() -> RecordBatch {
    RecordBatch::new_empty(Arc::new(Schema::empty()))
}

fn dataset_ontology_batch() -> RecordBatch {
    let object_ids: ArrayRef = Arc::new(StringArray::from(vec!["patient/patient-001"]));
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            "semanticObjectId",
            DataType::Utf8,
            false,
        )])),
        vec![object_ids],
    )
    .unwrap_or_else(|error| panic!("dataset ontology test batch: {error}"))
}
