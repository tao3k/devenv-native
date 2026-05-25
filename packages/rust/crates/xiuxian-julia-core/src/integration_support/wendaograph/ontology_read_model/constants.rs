//! Service constants for the `WendaoGraph` ontology read-model bridge.

/// `WendaoGraph` service name for ontology read-model quality scoring.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SERVICE: &str =
    "wendao.graph.v1.OntologyReadModelQuality";
/// `WendaoGraph` service method for ontology read-model quality scoring.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_METHOD: &str = "RunOntologyReadModelQuality";
/// `WendaoGraph` ontology read-model quality service schema version.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION: &str =
    "xiuxian_wendao.graph.ontology_read_model_quality.service.v1";
/// MIME type used by the `WendaoGraph` ontology read-model quality Arrow IPC service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ARROW_IPC_MIME: &str =
    "application/vnd.apache.arrow.stream";
/// Flight descriptor path for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_FLIGHT_DESCRIPTOR_PATH: [&str; 3] =
    ["wendao", "graph", "ontology_read_model_quality"];
/// Canonical route form used by runtime Flight transport negotiation.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE: &str =
    "/wendao/graph/ontology_read_model_quality";
/// Stable provider id for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID: &str = "wendaograph";
/// Stable capability id for the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID: &str =
    "ontology-read-model-quality";
/// Polyglot Julia profile id used when scheduling ontology read-model quality Flight work.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROFILE_ID: &str =
    "wendaograph.ontology_read_model_quality";
/// Single request table name used to bundle the three read-model Arrow payloads over Flight.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_BUNDLE_TABLE: &str =
    "ontology_read_model_quality_request";
/// Request table names expected by the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_REQUEST_TABLES: [&str; 3] = [
    "semantic_objects",
    "semantic_relations",
    "semantic_projection_state",
];
/// Response table name returned by the `WendaoGraph` ontology read-model quality service.
pub const WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_RESPONSE_TABLE: &str = "ontology_quality_rows";
/// Optional parent object type request table accepted by extension proof mode.
pub const WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE: &str = "parent_object_types";
/// Optional parent link type request table accepted by extension proof mode.
pub const WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE: &str = "parent_link_types";
/// Request table names used when extension proof mode is explicitly selected.
pub const WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_REQUEST_TABLES: [&str; 5] = [
    "semantic_objects",
    "semantic_relations",
    "semantic_projection_state",
    WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_TABLE,
    WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_TABLE,
];
/// Response table name returned by the `WendaoGraph` ontology extension proof mode.
pub const WENDAO_GRAPH_ONTOLOGY_EXTENSION_PROOF_RESPONSE_TABLE: &str =
    "ontology_extension_proof_rows";
/// Bundle column containing the `semantic_objects` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_OBJECTS_PAYLOAD_COLUMN: &str = "semantic_objects_payload";
/// Bundle column containing the `semantic_relations` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_RELATIONS_PAYLOAD_COLUMN: &str =
    "semantic_relations_payload";
/// Bundle column containing the `semantic_projection_state` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_SEMANTIC_PROJECTION_STATE_PAYLOAD_COLUMN: &str =
    "semantic_projection_state_payload";
/// Bundle column containing the `parent_object_types` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_PARENT_OBJECT_TYPES_PAYLOAD_COLUMN: &str =
    "parent_object_types_payload";
/// Bundle column containing the `parent_link_types` Arrow IPC payload.
pub const WENDAO_GRAPH_ONTOLOGY_PARENT_LINK_TYPES_PAYLOAD_COLUMN: &str =
    "parent_link_types_payload";
/// Bundle scalar column containing the extension domain prefix.
pub const WENDAO_GRAPH_ONTOLOGY_EXTENSION_DOMAIN_PREFIX_COLUMN: &str = "extension_domain_prefix";
/// Bundle scalar column containing the optional RDF namespace hint.
pub const WENDAO_GRAPH_ONTOLOGY_RDF_NAMESPACE_COLUMN: &str = "rdf_namespace";

pub(super) const SERVICE_METADATA_KEY: &str = "wendao.service";
pub(super) const METHOD_METADATA_KEY: &str = "wendao.method";
pub(super) const SCHEMA_VERSION_METADATA_KEY: &str = "wendao.schema_version";
pub(super) const TABLE_METADATA_KEY: &str = "wendao.table";
