//! `WendaoGraph` evidence schema materialization and validation.

use std::sync::Arc;

use arrow::datatypes::Schema;
use xiuxian_db_store::{
    ArrowSchemaContractError, ArrowSchemaNullabilityPolicy, ArrowSchemaValidationOptions,
    validate_schema_against_contract_with_options,
};

use super::contracts::{
    WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS, WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_CONTRACTS,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_CONTRACTS,
};
use super::types::{WendaoGraphEvidenceTableContract, WendaoGraphEvidenceTableKind};

/// Resolve one request table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` evidence request contract.
pub fn wendao_graph_evidence_request_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_EVIDENCE_REQUEST_TABLE_CONTRACTS,
        "request",
    )
}

/// Resolve one response table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` evidence response contract.
pub fn wendao_graph_evidence_response_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_EVIDENCE_RESPONSE_TABLE_CONTRACTS,
        "response",
    )
}

/// Resolve one `PageIndex` reasoning request table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` `PageIndex` reasoning request contract.
pub fn wendao_graph_page_index_reasoning_request_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_PAGE_INDEX_REASONING_REQUEST_TABLE_CONTRACTS,
        "PageIndex reasoning request",
    )
}

/// Resolve one `PageIndex` reasoning response table contract by table name.
///
/// # Errors
///
/// Returns an error when the table name is not part of the canonical
/// `WendaoGraph` `PageIndex` reasoning response contract.
pub fn wendao_graph_page_index_reasoning_response_table_contract(
    table_name: impl AsRef<str>,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    find_contract(
        table_name.as_ref(),
        &WENDAO_GRAPH_PAGE_INDEX_REASONING_RESPONSE_TABLE_CONTRACTS,
        "PageIndex reasoning response",
    )
}

/// Materialize the Arrow schema for one `WendaoGraph` evidence table.
///
/// # Errors
///
/// Returns an error when the table is unknown for the selected side.
pub fn wendao_graph_evidence_table_schema(
    kind: WendaoGraphEvidenceTableKind,
    table_name: impl AsRef<str>,
) -> Result<Arc<Schema>, String> {
    let contract = match kind {
        WendaoGraphEvidenceTableKind::Request => {
            wendao_graph_evidence_request_table_contract(table_name)?
        }
        WendaoGraphEvidenceTableKind::Response => {
            wendao_graph_evidence_response_table_contract(table_name)?
        }
    };
    Ok(contract.schema())
}

/// Materialize the Arrow schema for one `WendaoGraph` `PageIndex` reasoning table.
///
/// # Errors
///
/// Returns an error when the table is unknown for the selected side.
pub fn wendao_graph_page_index_reasoning_table_schema(
    kind: WendaoGraphEvidenceTableKind,
    table_name: impl AsRef<str>,
) -> Result<Arc<Schema>, String> {
    let contract = match kind {
        WendaoGraphEvidenceTableKind::Request => {
            wendao_graph_page_index_reasoning_request_table_contract(table_name)?
        }
        WendaoGraphEvidenceTableKind::Response => {
            wendao_graph_page_index_reasoning_response_table_contract(table_name)?
        }
    };
    Ok(contract.schema())
}

/// Validate a request table Arrow schema against the canonical contract.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical request contract.
pub fn validate_wendao_graph_evidence_request_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_evidence_request_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

/// Validate a response table Arrow schema against the canonical contract.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical response contract.
pub fn validate_wendao_graph_evidence_response_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_evidence_response_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

/// Validate a `PageIndex` reasoning request table Arrow schema.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical request contract.
pub fn validate_wendao_graph_page_index_reasoning_request_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_page_index_reasoning_request_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

/// Validate a `PageIndex` reasoning response table Arrow schema.
///
/// # Errors
///
/// Returns an error when the table name is unknown or the schema order, column
/// type, or nullability does not match the canonical response contract.
pub fn validate_wendao_graph_page_index_reasoning_response_schema(
    table_name: impl AsRef<str>,
    schema: &Schema,
) -> Result<(), String> {
    let contract = wendao_graph_page_index_reasoning_response_table_contract(table_name)?;
    validate_contract_schema(contract, schema)
}

fn find_contract(
    table_name: &str,
    contracts: &'static [WendaoGraphEvidenceTableContract],
    side: &str,
) -> Result<&'static WendaoGraphEvidenceTableContract, String> {
    contracts
        .iter()
        .find(|contract| contract.table_name == table_name)
        .ok_or_else(|| format!("unknown WendaoGraph evidence {side} table `{table_name}`"))
}

fn validate_contract_schema(
    contract: &WendaoGraphEvidenceTableContract,
    schema: &Schema,
) -> Result<(), String> {
    validate_schema_against_contract_with_options(
        schema,
        &(*contract).arrow_schema_contract(),
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| wendao_graph_schema_error(contract, &error))
}

fn wendao_graph_schema_error(
    contract: &WendaoGraphEvidenceTableContract,
    error: &ArrowSchemaContractError,
) -> String {
    match error {
        ArrowSchemaContractError::ColumnCountMismatch {
            expected_count,
            actual_count,
            ..
        } => format!(
            "WendaoGraph evidence table `{}` must have {expected_count} columns, got {actual_count}",
            contract.table_name
        ),
        ArrowSchemaContractError::ColumnOrderMismatch {
            column_index,
            expected_column_name,
            actual_column_name,
            ..
        } => format!(
            "WendaoGraph evidence table `{}` column {column_index} must be `{expected_column_name}`, got `{actual_column_name}`",
            contract.table_name
        ),
        ArrowSchemaContractError::DataTypeMismatch {
            column_name,
            expected_data_type,
            actual_data_type,
            ..
        } => format!(
            "WendaoGraph evidence table `{}` column `{column_name}` must be {expected_data_type}, got {actual_data_type:?}",
            contract.table_name
        ),
        ArrowSchemaContractError::NullabilityMismatch { column_name, .. } => format!(
            "WendaoGraph evidence table `{}` column `{column_name}` must be non-nullable",
            contract.table_name
        ),
        ArrowSchemaContractError::MissingRequiredColumn { column_name, .. } => format!(
            "WendaoGraph evidence table `{}` missing required column `{column_name}`",
            contract.table_name
        ),
        _ => error.to_string(),
    }
}
