//! Arrow IPC bridge contract for `WendaoGraph` ontology read-model quality checks.

mod constants;
mod envelope;
mod flight;
mod ipc;
mod types;

pub use constants::{
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE, WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN,
    WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN,
};
pub use envelope::build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope;
pub use flight::{
    build_wendaograph_ontology_read_model_quality_flight_binding,
    build_wendaograph_ontology_read_model_quality_flight_descriptor,
    roundtrip_wendaograph_ontology_extension_proof_with_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
    wendaograph_ontology_read_model_quality_provider_selector,
};
pub use ipc::{
    build_wendaograph_ontology_extension_proof_arrow_request,
    build_wendaograph_ontology_extension_proof_flight_request_batch,
    build_wendaograph_ontology_read_model_quality_arrow_request,
    build_wendaograph_ontology_read_model_quality_flight_request_batch,
};
pub use types::{
    WendaoGraphOntologyExtensionProofArrowRequest, WendaoGraphOntologyExtensionProofRequestBatches,
    WendaoGraphOntologyReadModelQualityArrowRequest,
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    WendaoGraphOntologyReadModelQualityRequestBatches,
    WendaoGraphOntologyReadModelQualityRoundtrip,
    WendaoGraphOntologyReadModelQualityRoundtripError,
};
