//! Request and response types for the `WendaoGraph` ontology read-model bridge.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_runtime::transport::NegotiatedTransportSelection;

/// Semantic read-model Arrow tables accepted by the `WendaoGraph` quality service.
#[derive(Debug, Clone)]
pub struct WendaoGraphOntologyReadModelQualityRequestBatches {
    /// Accepted `semantic_objects` read-model table.
    pub objects: RecordBatch,
    /// Accepted `semantic_relations` read-model table.
    pub relations: RecordBatch,
    /// Accepted `semantic_projection_state` read-model table.
    pub projection_state: RecordBatch,
}

/// Semantic read-model plus parent registry tables accepted by extension proof mode.
#[derive(Debug, Clone)]
pub struct WendaoGraphOntologyExtensionProofRequestBatches {
    /// Compiled parent ontology object type table.
    pub parent_object_types: RecordBatch,
    /// Compiled parent ontology link type table.
    pub parent_link_types: RecordBatch,
    /// Accepted semantic read-model tables for the extension.
    pub read_model: WendaoGraphOntologyReadModelQualityRequestBatches,
}

impl WendaoGraphOntologyExtensionProofRequestBatches {
    /// Create an extension proof request bundle from compiled parent registry and read-model tables.
    #[must_use]
    pub fn new(
        parent_object_types: RecordBatch,
        parent_link_types: RecordBatch,
        read_model: WendaoGraphOntologyReadModelQualityRequestBatches,
    ) -> Self {
        Self {
            parent_object_types,
            parent_link_types,
            read_model,
        }
    }

    /// Return the row counts for the five request tables in service order.
    #[must_use]
    pub fn row_counts(&self) -> [usize; 5] {
        [
            self.read_model.objects.num_rows(),
            self.read_model.relations.num_rows(),
            self.read_model.projection_state.num_rows(),
            self.parent_object_types.num_rows(),
            self.parent_link_types.num_rows(),
        ]
    }
}

impl WendaoGraphOntologyReadModelQualityRequestBatches {
    /// Create a request batch bundle from already materialized read-model tables.
    #[must_use]
    pub fn new(
        semantic_objects: RecordBatch,
        semantic_relations: RecordBatch,
        semantic_projection_state: RecordBatch,
    ) -> Self {
        Self {
            objects: semantic_objects,
            relations: semantic_relations,
            projection_state: semantic_projection_state,
        }
    }

    /// Return the row counts for the request tables in service order.
    #[must_use]
    pub fn row_counts(&self) -> [usize; 3] {
        [
            self.objects.num_rows(),
            self.relations.num_rows(),
            self.projection_state.num_rows(),
        ]
    }
}

/// Arrow IPC request payloads for the `WendaoGraph` ontology quality service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityArrowRequest {
    /// Service schema version expected by `WendaoGraph`.
    pub schema_version: &'static str,
    /// Request MIME type for every payload.
    pub request_mime_type: &'static str,
    /// Response MIME type expected from `WendaoGraph`.
    pub response_mime_type: &'static str,
    /// Request table names in payload order.
    pub request_tables: [&'static str; 3],
    /// Response table name expected from `WendaoGraph`.
    pub response_table: &'static str,
    /// Arrow IPC stream for `semantic_objects`.
    pub semantic_objects_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_relations`.
    pub semantic_relations_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_projection_state`.
    pub semantic_projection_state_payload: Vec<u8>,
}

/// Arrow IPC request payloads for the `WendaoGraph` ontology extension proof mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyExtensionProofArrowRequest {
    /// Service schema version expected by `WendaoGraph`.
    pub schema_version: &'static str,
    /// Request MIME type for every payload.
    pub request_mime_type: &'static str,
    /// Response MIME type expected from `WendaoGraph`.
    pub response_mime_type: &'static str,
    /// Request table names in payload order.
    pub request_tables: [&'static str; 5],
    /// Response table name expected from `WendaoGraph`.
    pub response_table: &'static str,
    /// Arrow IPC stream for `semantic_objects`.
    pub semantic_objects_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_relations`.
    pub semantic_relations_payload: Vec<u8>,
    /// Arrow IPC stream for `semantic_projection_state`.
    pub semantic_projection_state_payload: Vec<u8>,
    /// Arrow IPC stream for `parent_object_types`.
    pub parent_object_types_payload: Vec<u8>,
    /// Arrow IPC stream for `parent_link_types`.
    pub parent_link_types_payload: Vec<u8>,
    /// Extension domain prefix used by the proof contract.
    pub extension_domain_prefix: String,
    /// Optional RDF namespace hint used by the proof contract.
    pub rdf_namespace: String,
}

/// Runtime binding options for the `WendaoGraph` ontology quality Flight route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityFlightBindingOptions {
    /// Flight service base URL, for example `http://127.0.0.1:41082`.
    pub base_url: String,
    /// Optional health route for service readiness probes.
    pub health_route: Option<String>,
    /// Optional request timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Optional maximum in-flight requests for one transport client.
    pub max_in_flight_requests: Option<u64>,
}

/// Response from one negotiated ontology quality Flight exchange.
#[derive(Debug, Clone)]
pub struct WendaoGraphOntologyReadModelQualityRoundtrip {
    /// Runtime transport selected for the exchange.
    pub selection: NegotiatedTransportSelection,
    /// Raw Arrow response batches returned by `WendaoGraph`.
    pub response_batches: Vec<RecordBatch>,
}

/// Error returned when an ontology quality Flight exchange fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoGraphOntologyReadModelQualityRoundtripError {
    /// Runtime selection, when the exchange reached a negotiated transport.
    pub selection: Option<NegotiatedTransportSelection>,
    /// Human-readable failure detail.
    pub error: String,
}

impl WendaoGraphOntologyReadModelQualityArrowRequest {
    /// Return the encoded payload byte sizes in service request order.
    #[must_use]
    pub fn payload_byte_sizes(&self) -> [usize; 3] {
        [
            self.semantic_objects_payload.len(),
            self.semantic_relations_payload.len(),
            self.semantic_projection_state_payload.len(),
        ]
    }
}

impl WendaoGraphOntologyExtensionProofArrowRequest {
    /// Return the encoded payload byte sizes in service request order.
    #[must_use]
    pub fn payload_byte_sizes(&self) -> [usize; 5] {
        [
            self.semantic_objects_payload.len(),
            self.semantic_relations_payload.len(),
            self.semantic_projection_state_payload.len(),
            self.parent_object_types_payload.len(),
            self.parent_link_types_payload.len(),
        ]
    }
}
