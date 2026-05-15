//! Dataset-to-ontology transport contracts.

mod dataset;

pub use dataset::{
    DATASET_ONTOLOGY_HANDOFF_SCHEMA_VERSION, DatasetOntologyFlightManifest,
    DatasetOntologySourceTablePayload, ONTOLOGY_DATASET_MATERIALIZE_ROUTE,
    WENDAO_DATASET_ONTOLOGY_CONTRACT_ID_HEADER, WENDAO_DATASET_ONTOLOGY_MANIFEST_HEADER,
    WENDAO_DATASET_ONTOLOGY_MAPPING_ID_HEADER, WENDAO_DATASET_ONTOLOGY_PAYLOAD_ID_HEADER,
    decode_dataset_ontology_manifest_header, encode_dataset_ontology_manifest_header,
    validate_dataset_ontology_flight_manifest,
};
