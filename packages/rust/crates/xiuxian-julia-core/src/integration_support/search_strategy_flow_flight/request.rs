//! Arrow request-bundle builders for the `WendaoGraph` `SearchStrategyFlow` service.

use std::sync::Arc;

use arrow::array::{ArrayRef, BinaryArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInputBatch;

use super::constants::{
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ARROW_IPC_MIME,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_BRANCH_JUDGEMENTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_METHOD,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ONTOLOGY_REGISTRY_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_QUERY_UNDERSTANDING_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION, WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SERVICE,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
};

/// Optional Arrow IPC side-table payloads for one `SearchStrategyFlow` service request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchStrategyFlowServiceRequestOptions {
    /// Arrow IPC stream for precomputed query-understanding rows.
    pub query_understanding: Option<Vec<u8>>,
    /// Arrow IPC stream for ontology registry rows.
    pub ontology_registry: Option<Vec<u8>>,
    /// Arrow IPC stream for branch judgement rows.
    pub branch_judgements: Option<Vec<u8>>,
}

impl SearchStrategyFlowServiceRequestOptions {
    /// Return options with query-understanding Arrow IPC rows.
    #[must_use]
    pub fn with_query_understanding_arrow_ipc_stream(mut self, payload: Vec<u8>) -> Self {
        self.query_understanding = Some(payload);
        self
    }

    /// Return options with ontology-registry Arrow IPC rows.
    #[must_use]
    pub fn with_ontology_registry_arrow_ipc_stream(mut self, payload: Vec<u8>) -> Self {
        self.ontology_registry = Some(payload);
        self
    }

    /// Return options with branch-judgement Arrow IPC rows.
    #[must_use]
    pub fn with_branch_judgements_arrow_ipc_stream(mut self, payload: Vec<u8>) -> Self {
        self.branch_judgements = Some(payload);
        self
    }
}

/// Arrow IPC payload bundle for one `SearchStrategyFlow` service request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStrategyFlowServiceArrowRequest {
    /// Service schema version expected by `WendaoGraph`.
    pub schema_version: &'static str,
    /// Request MIME type for every payload.
    pub request_mime_type: &'static str,
    /// Response MIME type expected from `WendaoGraph`.
    pub response_mime_type: &'static str,
    /// Single Flight request-bundle table name.
    pub request_table: &'static str,
    /// Arrow IPC stream for `strategy_candidates`.
    pub strategy_candidates_payload: Vec<u8>,
    /// Optional Arrow IPC stream for `query_understanding`.
    pub query_understanding_payload: Option<Vec<u8>>,
    /// Optional Arrow IPC stream for `ontology_registry`.
    pub ontology_registry_payload: Option<Vec<u8>>,
    /// Optional Arrow IPC stream for `branch_judgements`.
    pub branch_judgements_payload: Option<Vec<u8>>,
}

impl SearchStrategyFlowServiceArrowRequest {
    /// Return total encoded payload bytes for scheduling/admission estimates.
    #[must_use]
    pub fn payload_byte_size(&self) -> usize {
        self.strategy_candidates_payload.len()
            + optional_len(self.query_understanding_payload.as_ref())
            + optional_len(self.ontology_registry_payload.as_ref())
            + optional_len(self.branch_judgements_payload.as_ref())
    }
}

/// Build the Arrow IPC payload bundle for the `SearchStrategyFlow` Flight service.
///
/// # Errors
///
/// Returns an error when the candidate batch has no Arrow IPC payload or any
/// optional side-table payload is present but empty.
pub fn build_search_strategy_flow_service_arrow_request(
    candidate_batch: &SearchStrategyFlowCandidateInputBatch,
    options: SearchStrategyFlowServiceRequestOptions,
) -> Result<SearchStrategyFlowServiceArrowRequest, String> {
    if candidate_batch.candidate_input_arrow_ipc_stream.is_empty() {
        return Err("SearchStrategyFlow candidate Arrow IPC payload must not be empty".to_string());
    }
    validate_optional_payload("query_understanding", options.query_understanding.as_ref())?;
    validate_optional_payload("ontology_registry", options.ontology_registry.as_ref())?;
    validate_optional_payload("branch_judgements", options.branch_judgements.as_ref())?;

    Ok(SearchStrategyFlowServiceArrowRequest {
        schema_version: WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION,
        request_mime_type: WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ARROW_IPC_MIME,
        response_mime_type: WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ARROW_IPC_MIME,
        request_table: WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE,
        strategy_candidates_payload: candidate_batch.candidate_input_arrow_ipc_stream.clone(),
        query_understanding_payload: options.query_understanding,
        ontology_registry_payload: options.ontology_registry,
        branch_judgements_payload: options.branch_judgements,
    })
}

/// Build the single-table Arrow Flight request bundle for `SearchStrategyFlow`.
///
/// # Errors
///
/// Returns an error when the request bundle cannot be represented as an Arrow
/// `RecordBatch`.
pub fn build_search_strategy_flow_service_flight_request_batch(
    request: &SearchStrategyFlowServiceArrowRequest,
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
                DataType::Binary,
                false,
            ),
            Field::new(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_QUERY_UNDERSTANDING_PAYLOAD_COLUMN,
                DataType::Binary,
                true,
            ),
            Field::new(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ONTOLOGY_REGISTRY_PAYLOAD_COLUMN,
                DataType::Binary,
                true,
            ),
            Field::new(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_BRANCH_JUDGEMENTS_PAYLOAD_COLUMN,
                DataType::Binary,
                true,
            ),
        ],
        [
            (
                "wendao.service".to_string(),
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SERVICE.to_string(),
            ),
            (
                "wendao.method".to_string(),
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_METHOD.to_string(),
            ),
            (
                "wendao.schema_version".to_string(),
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION.to_string(),
            ),
            (
                "wendao.table".to_string(),
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
    ));

    RecordBatch::try_new(
        schema,
        vec![
            required_binary(request.strategy_candidates_payload.as_slice()),
            optional_binary(request.query_understanding_payload.as_deref()),
            optional_binary(request.ontology_registry_payload.as_deref()),
            optional_binary(request.branch_judgements_payload.as_deref()),
        ],
    )
    .map_err(|error| format!("build SearchStrategyFlow Flight request bundle: {error}"))
}

fn required_binary(payload: &[u8]) -> ArrayRef {
    Arc::new(BinaryArray::from(vec![payload]))
}

fn optional_binary(payload: Option<&[u8]>) -> ArrayRef {
    Arc::new(BinaryArray::from_iter([payload]))
}

fn validate_optional_payload(label: &str, payload: Option<&Vec<u8>>) -> Result<(), String> {
    if matches!(payload, Some(bytes) if bytes.is_empty()) {
        return Err(format!(
            "SearchStrategyFlow optional `{label}` Arrow IPC payload must not be empty"
        ));
    }
    Ok(())
}

fn optional_len(payload: Option<&Vec<u8>>) -> usize {
    payload.map_or(0, Vec::len)
}
