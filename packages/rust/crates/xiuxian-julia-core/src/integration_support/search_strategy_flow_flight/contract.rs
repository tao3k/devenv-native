//! Arrow table contracts for the `WendaoGraph` `SearchStrategyFlow` service.

use std::collections::HashMap;

use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    build_arrow_schema, validate_arrow_ipc_stream, validate_record_batch_schema,
};

use super::constants::{
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_BRANCH_JUDGEMENTS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ONTOLOGY_REGISTRY_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_QUERY_UNDERSTANDING_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
};

const STRATEGY_CANDIDATES_REQUEST_TABLE: &str = "strategy_candidates_request";
const QUERY_UNDERSTANDING_TABLE: &str = "query_understanding";
const ONTOLOGY_REGISTRY_TABLE: &str = "ontology_registry";
const BRANCH_JUDGEMENTS_TABLE: &str = "branch_judgements";
const STRATEGY_CANDIDATES_TABLE: &str = "strategy_candidates";
const STRATEGY_TRANSITIONS_TABLE: &str = "strategy_transitions";
const STRATEGY_FRONTIER_TABLE: &str = "strategy_frontier";
const STRATEGY_PLANNER_ACTIONS_TABLE: &str = "strategy_planner_actions";

/// Row in the `SearchStrategyFlow` Arrow table-contract manifest.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchStrategyFlowTableContractRow {
    /// Logical Arrow table name.
    pub(crate) table_name: &'static str,
    /// Logical column name.
    pub(crate) column_name: &'static str,
    /// One-based column position.
    pub(crate) column_index: usize,
    /// Contract direction: `request` or `response`.
    pub(crate) direction: &'static str,
    /// Whether this column is required by the logical contract.
    pub(crate) required_column: bool,
    /// Whether schema validation requires exact column count and order.
    pub(crate) exact_column_set: bool,
}

/// Build the Arrow `Schema` for the `SearchStrategyFlow` Flight request bundle.
pub(super) fn search_strategy_flow_request_bundle_schema(
    metadata: HashMap<String, String>,
) -> Schema {
    build_arrow_schema(&request_bundle_contract(), metadata)
}

/// Build the Arrow `Schema` for the `SearchStrategyFlow` Flight response bundle.
#[cfg(test)]
pub(crate) fn search_strategy_flow_response_bundle_schema(
    metadata: HashMap<String, String>,
) -> Schema {
    build_arrow_schema(&response_bundle_contract(), metadata)
}

/// Build the Arrow `Schema` for direct `strategy_frontier` response rows.
#[cfg(test)]
pub(crate) fn search_strategy_flow_frontier_response_schema(
    metadata: HashMap<String, String>,
) -> Schema {
    build_arrow_schema(&strategy_frontier_contract(), metadata)
}

/// Logical table name for the request-side ontology-registry payload.
pub(super) const fn search_strategy_flow_ontology_registry_table_name() -> &'static str {
    ONTOLOGY_REGISTRY_TABLE
}

/// Build the Arrow `Schema` for a request-side payload table.
pub(super) fn search_strategy_flow_request_payload_schema(
    table_name: &str,
    metadata: HashMap<String, String>,
) -> Result<Schema, String> {
    let contract = table_contract(table_name)?;
    Ok(build_arrow_schema(&contract, metadata))
}

/// Return the Rust-owned table-contract manifest rows.
#[cfg(test)]
pub(crate) fn search_strategy_flow_table_contract_rows() -> Vec<SearchStrategyFlowTableContractRow>
{
    let mut rows = Vec::new();
    for (table_name, direction, contract) in table_contract_manifest_entries() {
        for (column_index, column) in contract.columns().iter().enumerate() {
            rows.push(SearchStrategyFlowTableContractRow {
                table_name,
                column_name: column.name(),
                column_index: column_index.saturating_add(1),
                direction,
                required_column: true,
                exact_column_set: contract.exact_column_set(),
            });
        }
    }
    rows
}

pub(super) fn search_strategy_flow_response_bundle_payload_columns() -> [&'static str; 4] {
    search_strategy_flow_response_payload_specs().map(|(column_name, _)| column_name)
}

pub(super) fn search_strategy_flow_response_candidates_payload_column() -> &'static str {
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN
}

pub(super) fn search_strategy_flow_response_transitions_payload_column() -> &'static str {
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN
}

pub(super) fn search_strategy_flow_response_frontier_payload_column() -> &'static str {
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN
}

pub(super) fn search_strategy_flow_response_planner_actions_payload_column() -> &'static str {
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN
}

pub(super) fn search_strategy_flow_response_payload_table_name(
    column_name: &str,
) -> Result<&'static str, String> {
    search_strategy_flow_response_payload_specs()
        .into_iter()
        .find_map(|(payload_column, table_name)| {
            (payload_column == column_name).then_some(table_name)
        })
        .ok_or_else(|| {
            format!("SearchStrategyFlow response bundle has no table contract for `{column_name}`")
        })
}

/// Validate a `SearchStrategyFlow` Flight request bundle batch.
pub(super) fn validate_search_strategy_flow_request_bundle_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_batch_against_contract(batch, &request_bundle_contract())
}

/// Validate a `SearchStrategyFlow` Flight response bundle batch.
pub(super) fn validate_search_strategy_flow_response_bundle_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_batch_against_contract(batch, &response_bundle_contract())
}

/// Validate Arrow IPC rows carried inside a `SearchStrategyFlow` request payload.
pub(super) fn validate_search_strategy_flow_request_payload_stream(
    table_name: &str,
    payload: &[u8],
) -> Result<(), String> {
    validate_search_strategy_flow_ipc_payload_stream(table_name, payload)
}

/// Validate Arrow IPC rows carried inside a `SearchStrategyFlow` response payload.
pub(super) fn validate_search_strategy_flow_response_payload_stream(
    table_name: &str,
    payload: &[u8],
) -> Result<(), String> {
    validate_search_strategy_flow_ipc_payload_stream(table_name, payload)
}

/// Validate a direct `strategy_candidates` response batch.
pub(super) fn validate_strategy_candidates_response_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_batch_against_contract(batch, &strategy_candidates_contract())
}

/// Validate a direct `strategy_frontier` response batch.
pub(super) fn validate_strategy_frontier_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_batch_against_contract(batch, &strategy_frontier_contract())
}

/// Validate a direct `strategy_planner_actions` response batch.
pub(super) fn validate_strategy_planner_actions_response_batch(
    batch: &RecordBatch,
) -> Result<(), String> {
    validate_batch_against_contract(batch, &strategy_planner_actions_contract())
}

fn validate_search_strategy_flow_ipc_payload_stream(
    table_name: &str,
    payload: &[u8],
) -> Result<(), String> {
    let contract = table_contract(table_name)?;
    validate_arrow_ipc_stream(payload, &contract)
        .map_err(|error| search_strategy_flow_schema_error(&error))
}

fn validate_batch_against_contract(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
) -> Result<(), String> {
    validate_record_batch_schema(batch, contract)
        .map_err(|error| search_strategy_flow_schema_error(&error))
}

fn search_strategy_flow_schema_error(error: &ArrowSchemaContractError) -> String {
    format!("SearchStrategyFlow {error}")
}

fn table_contract(table_name: &str) -> Result<ArrowSchemaContract, String> {
    match table_name {
        STRATEGY_CANDIDATES_REQUEST_TABLE => Ok(strategy_candidates_request_contract()),
        QUERY_UNDERSTANDING_TABLE => Ok(query_understanding_contract()),
        ONTOLOGY_REGISTRY_TABLE => Ok(ontology_registry_contract()),
        BRANCH_JUDGEMENTS_TABLE => Ok(branch_judgements_contract()),
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE => Ok(request_bundle_contract()),
        STRATEGY_CANDIDATES_TABLE => Ok(strategy_candidates_contract()),
        STRATEGY_TRANSITIONS_TABLE => Ok(strategy_transitions_contract()),
        STRATEGY_FRONTIER_TABLE => Ok(strategy_frontier_contract()),
        STRATEGY_PLANNER_ACTIONS_TABLE => Ok(strategy_planner_actions_contract()),
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE => Ok(response_bundle_contract()),
        _ => Err(format!(
            "SearchStrategyFlow has no Arrow table contract for `{table_name}`"
        )),
    }
}

#[cfg(test)]
fn table_contract_manifest_entries() -> [(&'static str, &'static str, ArrowSchemaContract); 10] {
    [
        (
            STRATEGY_CANDIDATES_REQUEST_TABLE,
            "request",
            strategy_candidates_request_contract(),
        ),
        (
            QUERY_UNDERSTANDING_TABLE,
            "request",
            query_understanding_contract(),
        ),
        (
            ONTOLOGY_REGISTRY_TABLE,
            "request",
            ontology_registry_contract(),
        ),
        (
            BRANCH_JUDGEMENTS_TABLE,
            "request",
            branch_judgements_contract(),
        ),
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE,
            "request",
            request_bundle_contract(),
        ),
        (
            STRATEGY_CANDIDATES_TABLE,
            "response",
            strategy_candidates_contract(),
        ),
        (
            STRATEGY_TRANSITIONS_TABLE,
            "response",
            strategy_transitions_contract(),
        ),
        (
            STRATEGY_FRONTIER_TABLE,
            "response",
            strategy_frontier_contract(),
        ),
        (
            STRATEGY_PLANNER_ACTIONS_TABLE,
            "response",
            strategy_planner_actions_contract(),
        ),
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE,
            "response",
            response_bundle_contract(),
        ),
    ]
}

fn search_strategy_flow_response_payload_specs() -> [(&'static str, &'static str); 4] {
    [
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
            STRATEGY_CANDIDATES_TABLE,
        ),
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
            STRATEGY_TRANSITIONS_TABLE,
        ),
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
            STRATEGY_FRONTIER_TABLE,
        ),
        (
            WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
            STRATEGY_PLANNER_ACTIONS_TABLE,
        ),
    ]
}

fn strategy_candidates_request_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STRATEGY_CANDIDATES_REQUEST_TABLE,
        false,
        vec![column("candidate_id", ArrowSchemaDataType::Utf8)],
    )
}

fn query_understanding_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        QUERY_UNDERSTANDING_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("intent_id", ArrowSchemaDataType::Utf8),
            column("signal_id", ArrowSchemaDataType::Utf8),
            column("signal_kind", ArrowSchemaDataType::Utf8),
            column("signal_value", ArrowSchemaDataType::Utf8),
            column("confidence", ArrowSchemaDataType::Float64),
            column("route_hint", ArrowSchemaDataType::Utf8),
            column("required_evidence", ArrowSchemaDataType::Utf8),
            column("ambiguity", ArrowSchemaDataType::Float64),
            column("weight", ArrowSchemaDataType::Float64),
            column("recommended_loop_budget", ArrowSchemaDataType::Int64),
            column("recommended_judgement_budget", ArrowSchemaDataType::Int64),
            column("recommended_beam_width", ArrowSchemaDataType::Int64),
            column("reason", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn ontology_registry_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        ONTOLOGY_REGISTRY_TABLE,
        true,
        vec![
            column("resource_family", ArrowSchemaDataType::Utf8),
            column("api_name", ArrowSchemaDataType::Utf8),
            column("requires_evidence", ArrowSchemaDataType::Boolean),
        ],
    )
}

fn branch_judgements_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        BRANCH_JUDGEMENTS_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("candidate_id", ArrowSchemaDataType::Utf8),
            column("branch_role", ArrowSchemaDataType::Utf8),
            column("judgement_score", ArrowSchemaDataType::Float64),
            column("confidence", ArrowSchemaDataType::Float64),
            column("decision", ArrowSchemaDataType::Utf8),
            column("blocked", ArrowSchemaDataType::Boolean),
            column("reason", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn request_bundle_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE,
        true,
        vec![
            column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            nullable_column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_QUERY_UNDERSTANDING_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            nullable_column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ONTOLOGY_REGISTRY_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
            nullable_column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_BRANCH_JUDGEMENTS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::Binary,
            ),
        ],
    )
}

fn strategy_candidates_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STRATEGY_CANDIDATES_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("revision_id", ArrowSchemaDataType::Utf8),
            column("candidate_id", ArrowSchemaDataType::Utf8),
            column("candidate_kind", ArrowSchemaDataType::Utf8),
            column("node_count", ArrowSchemaDataType::Int64),
            column("edge_kind_count", ArrowSchemaDataType::Int64),
            column("evidence_coverage", ArrowSchemaDataType::Float64),
            column("graph_score", ArrowSchemaDataType::Float64),
            column("authority_score", ArrowSchemaDataType::Float64),
            column("semantic_score", ArrowSchemaDataType::Float64),
            column("structural_score", ArrowSchemaDataType::Float64),
            column("context_cost", ArrowSchemaDataType::Int64),
            column("uncertainty", ArrowSchemaDataType::Float64),
            column("blocked", ArrowSchemaDataType::Boolean),
            column("final_score", ArrowSchemaDataType::Float64),
            column("action", ArrowSchemaDataType::Utf8),
            column("reason", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn strategy_transitions_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STRATEGY_TRANSITIONS_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("source_revision_id", ArrowSchemaDataType::Utf8),
            column("target_revision_id", ArrowSchemaDataType::Utf8),
            column("candidate_id", ArrowSchemaDataType::Utf8),
            column("transition_kind", ArrowSchemaDataType::Utf8),
            column("score_delta", ArrowSchemaDataType::Float64),
            column("missing_signal", ArrowSchemaDataType::Utf8),
            column("action", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn strategy_frontier_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STRATEGY_FRONTIER_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("frontier_id", ArrowSchemaDataType::Utf8),
            column("candidate_id", ArrowSchemaDataType::Utf8),
            column("revision_id", ArrowSchemaDataType::Utf8),
            column("rank", ArrowSchemaDataType::Int64),
            column("selected", ArrowSchemaDataType::Boolean),
            column("final_score", ArrowSchemaDataType::Float64),
            column("action", ArrowSchemaDataType::Utf8),
            column("context_budget", ArrowSchemaDataType::Int64),
            column("judgement_kind", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn strategy_planner_actions_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STRATEGY_PLANNER_ACTIONS_TABLE,
        true,
        vec![
            column("flow_id", ArrowSchemaDataType::Utf8),
            column("action_id", ArrowSchemaDataType::Utf8),
            column("source_revision_id", ArrowSchemaDataType::Utf8),
            column("target_revision_id", ArrowSchemaDataType::Utf8),
            column("candidate_id", ArrowSchemaDataType::Utf8),
            column("target_candidate_id", ArrowSchemaDataType::Utf8),
            column("frontier_id", ArrowSchemaDataType::Utf8),
            column("action_kind", ArrowSchemaDataType::Utf8),
            column("cycle_allowed", ArrowSchemaDataType::Boolean),
            column("requires_llm_judgement", ArrowSchemaDataType::Boolean),
            column("score", ArrowSchemaDataType::Float64),
            column("context_budget", ArrowSchemaDataType::Int64),
            column("reason", ArrowSchemaDataType::Utf8),
        ],
    )
}

fn response_bundle_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE,
        true,
        vec![
            column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
                ArrowSchemaDataType::BinaryPayload,
            ),
            column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::BinaryPayload,
            ),
            column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
                ArrowSchemaDataType::BinaryPayload,
            ),
            column(
                WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
                ArrowSchemaDataType::BinaryPayload,
            ),
        ],
    )
}

const fn column(name: &'static str, data_type: ArrowSchemaDataType) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, data_type)
}

const fn nullable_column(name: &'static str, data_type: ArrowSchemaDataType) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, data_type)
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/contract.rs"]
mod tests;
