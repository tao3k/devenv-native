use tonic::metadata::MetadataMap;

use crate::tests::transport::server::assertions::{metadata_value, must_err, must_ok};
use crate::transport::{
    DatasetOntologyFlightManifest, DatasetOntologySourceTablePayload,
    WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER, WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
    WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    encode_dataset_ontology_manifest_header,
    validate_dataset_ontology_materialize_request_metadata,
};

#[test]
fn validate_dataset_ontology_materialize_request_metadata_accepts_manifest() {
    let metadata = dataset_ontology_metadata(&healthcare_manifest());

    let manifest = must_ok(
        validate_dataset_ontology_materialize_request_metadata(&metadata),
        "dataset ontology metadata should validate",
    );

    assert_eq!(
        manifest.contract_id,
        "healthcare.synthetic_care_delivery.contract.v1"
    );
    assert_eq!(manifest.tables.len(), 2);
}

#[test]
fn validate_dataset_ontology_materialize_request_metadata_rejects_missing_manifest() {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        metadata_value(
            "healthcare.synthetic_care_delivery.contract.v1",
            "contract id metadata",
        ),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        metadata_value(
            "healthcare.synthetic_care_delivery.v1",
            "mapping id metadata",
        ),
    );

    let error = must_err(
        validate_dataset_ontology_materialize_request_metadata(&metadata),
        "missing manifest should fail",
    );

    assert_eq!(
        error.message(),
        "missing required header `x-wendao-dataset-ontology-manifest`"
    );
}

#[test]
fn validate_dataset_ontology_materialize_request_metadata_rejects_header_manifest_mismatch() {
    let manifest = healthcare_manifest();
    let mut metadata = dataset_ontology_metadata(&manifest);
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        metadata_value("wrong.mapping", "mapping id metadata"),
    );

    let error = must_err(
        validate_dataset_ontology_materialize_request_metadata(&metadata),
        "mapping mismatch should fail",
    );

    assert_eq!(
        error.message(),
        "dataset ontology mapping header `x-wendao-dataset-ontology-mapping-id` does not match manifest mapping id"
    );
}

pub(crate) fn dataset_ontology_metadata(manifest: &DatasetOntologyFlightManifest) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    insert_dataset_ontology_metadata(&mut metadata, manifest);
    metadata
}

pub(crate) fn insert_dataset_ontology_metadata(
    metadata: &mut MetadataMap,
    manifest: &DatasetOntologyFlightManifest,
) {
    metadata.insert(
        WENDAO_SCHEMA_VERSION_HEADER,
        metadata_value("v2", "schema version metadata"),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        metadata_value(manifest.contract_id.as_str(), "contract id metadata"),
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        metadata_value(manifest.mapping_id.as_str(), "mapping id metadata"),
    );
    let manifest_header = must_ok(
        encode_dataset_ontology_manifest_header(manifest),
        "dataset ontology manifest should encode",
    );
    metadata.insert(
        WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
        metadata_value(manifest_header.as_str(), "manifest metadata"),
    );
}

pub(crate) fn healthcare_manifest() -> DatasetOntologyFlightManifest {
    DatasetOntologyFlightManifest::new(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        vec![
            DatasetOntologySourceTablePayload::new("raw_patients", "payload-raw-patients")
                .with_row_count(2),
            DatasetOntologySourceTablePayload::new("raw_providers", "payload-raw-providers")
                .with_row_count(2),
        ],
    )
}
