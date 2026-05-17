use super::{
    DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION, DatasetOntologyFlightManifest,
    DatasetOntologySourceTablePayload, ONTOLOGY_DATASET_MATERIALIZE_ROUTE,
    WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER, WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
    WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER, WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER,
    decode_dataset_ontology_manifest_header, encode_dataset_ontology_manifest_header,
    validate_dataset_ontology_flight_manifest,
};

#[test]
fn dataset_ontology_flight_contract_exposes_route_and_headers() {
    assert_eq!(
        ONTOLOGY_DATASET_MATERIALIZE_ROUTE,
        "/ontology/dataset/materialize"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER,
        "x-wendao-dataset-ontology-contract-id"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER,
        "x-wendao-dataset-ontology-mapping-id"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
        "x-wendao-dataset-ontology-manifest"
    );
    assert_eq!(
        WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER,
        "x-wendao-dataset-ontology-payload-id"
    );
}

#[test]
fn dataset_ontology_flight_manifest_roundtrips_as_json_metadata() {
    let manifest = healthcare_manifest();

    let encoded = encode_dataset_ontology_manifest_header(&manifest)
        .unwrap_or_else(|error| panic!("valid dataset ontology manifest should encode: {error}"));
    let decoded = decode_dataset_ontology_manifest_header(&encoded)
        .unwrap_or_else(|error| panic!("valid dataset ontology manifest should decode: {error}"));

    assert_eq!(decoded, manifest);
}

#[test]
fn dataset_ontology_flight_manifest_rejects_invalid_identity() {
    let mut manifest = healthcare_manifest();
    manifest.contract_id = " ".to_string();
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err("dataset ontology contract id must not be blank".to_string())
    );

    let mut manifest = healthcare_manifest();
    manifest.mapping_id.clear();
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err("dataset ontology mapping id must not be blank".to_string())
    );
}

#[test]
fn dataset_ontology_flight_manifest_rejects_invalid_tables() {
    let mut manifest = healthcare_manifest();
    manifest.tables.clear();
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err("dataset ontology manifest must include at least one source table".to_string())
    );

    let mut manifest = healthcare_manifest();
    manifest.tables[1].table_name = "raw_patients".to_string();
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err("dataset ontology manifest contains duplicate source table `raw_patients`".to_string())
    );

    let mut manifest = healthcare_manifest();
    manifest.tables[1].payload_id = "payload-raw-patients".to_string();
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err(
            "dataset ontology manifest contains duplicate payload id `payload-raw-patients`"
                .to_string()
        )
    );

    let mut manifest = healthcare_manifest();
    manifest.tables[0].row_count = Some(0);
    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err(
            "dataset ontology source table `raw_patients` row count must be positive when provided"
                .to_string()
        )
    );
}

#[test]
fn dataset_ontology_flight_manifest_rejects_unsupported_schema_version() {
    let mut manifest = healthcare_manifest();
    manifest.schema_version = "xiuxian_wendao.dataset_ontology_handoff.v0".to_string();

    assert_eq!(
        validate_dataset_ontology_flight_manifest(&manifest),
        Err("unsupported dataset ontology manifest schema version `xiuxian_wendao.dataset_ontology_handoff.v0`".to_string())
    );
}

fn healthcare_manifest() -> DatasetOntologyFlightManifest {
    DatasetOntologyFlightManifest::new(
        "healthcare.synthetic_care_delivery.contract.v1",
        "healthcare.synthetic_care_delivery.v1",
        vec![
            DatasetOntologySourceTablePayload::new("raw_patients", "payload-raw-patients")
                .with_row_count(2)
                .with_content_sha256("sha256:patients")
                .with_schema_fingerprint("schema:patients"),
            DatasetOntologySourceTablePayload::new("raw_providers", "payload-raw-providers")
                .with_row_count(2)
                .with_content_sha256("sha256:providers")
                .with_schema_fingerprint("schema:providers"),
        ],
    )
}

#[test]
fn dataset_ontology_flight_manifest_uses_current_schema_version() {
    assert_eq!(
        healthcare_manifest().schema_version,
        DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION
    );
}
