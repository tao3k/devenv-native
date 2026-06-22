//! Semantic-scope to `SearchStrategyFlow` ontology-registry bridge.

use std::collections::{BTreeSet, HashMap};
use std::io::Cursor;
use std::sync::Arc;

use arrow::array::{Array, BooleanArray, StringArray};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::contract::{
    search_strategy_flow_ontology_registry_table_name, search_strategy_flow_request_payload_schema,
    validate_search_strategy_flow_request_payload_stream,
};
use super::metadata::{ANALYSIS_SEMANTIC_SCOPE_ROUTE, populate_semantic_scope_headers};

/// Fetch accepted semantic-scope rows and serialize them as ontology registry
/// Arrow IPC rows for the embedded `WendaoGraph.jl` adapter.
///
/// # Errors
///
/// Returns an error when the Flight endpoint cannot be reached or when the
/// semantic-scope payload cannot be decoded.
pub(crate) async fn search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope(
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<Vec<u8>, String> {
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let batches = client
        .collect_route_batches_allow_empty(
            ANALYSIS_SEMANTIC_SCOPE_ROUTE,
            "SearchStrategyFlow semantic-scope ontology registry",
            |metadata| populate_semantic_scope_headers(metadata, &[]),
        )
        .await?;
    semantic_scope_batches_to_ontology_registry_arrow_ipc(&batches)
}

/// Build Arrow IPC ontology-registry rows from accepted semantic-scope rows.
///
/// # Errors
///
/// Returns an error when the Arrow IPC stream cannot be encoded.
pub(crate) fn semantic_scope_batches_to_ontology_registry_arrow_ipc(
    batches: &[RecordBatch],
) -> Result<Vec<u8>, String> {
    ontology_registry_rows_to_arrow_ipc(ontology_registry_rows_from_semantic_scope_batches(batches))
}

fn ontology_registry_rows_from_semantic_scope_batches(
    batches: &[RecordBatch],
) -> BTreeSet<OntologyRegistryRow> {
    let mut rows = BTreeSet::<OntologyRegistryRow>::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(object_id) = string_at(batch, "objectId", row_index) else {
                continue;
            };
            if object_id.trim().is_empty() {
                continue;
            }
            let kind = string_at(batch, "kind", row_index);
            rows.insert(OntologyRegistryRow::new(
                resource_family_for_kind(kind.as_deref()),
                object_id.as_str(),
                false,
            ));
            if let Some(title) = string_at(batch, "title", row_index)
                && !title.trim().is_empty()
                && title != object_id
            {
                rows.insert(OntologyRegistryRow::new(
                    "object_type",
                    title.as_str(),
                    false,
                ));
            }
            for target in json_string_list_at(batch, "relationTargetsJson", row_index) {
                rows.insert(OntologyRegistryRow::new(
                    "link_type",
                    format!("{object_id}.{target}").as_str(),
                    false,
                ));
                rows.insert(OntologyRegistryRow::new(
                    "link_type",
                    target.as_str(),
                    false,
                ));
            }
            for validation in json_string_list_at(batch, "requiredValidationsJson", row_index) {
                rows.insert(OntologyRegistryRow::new(
                    "action_type",
                    validation.as_str(),
                    true,
                ));
            }
        }
    }
    rows
}

fn ontology_registry_rows_to_arrow_ipc(
    rows: BTreeSet<OntologyRegistryRow>,
) -> Result<Vec<u8>, String> {
    let rows = rows.into_iter().collect::<Vec<_>>();
    let table_name = search_strategy_flow_ontology_registry_table_name();
    let schema = Arc::new(search_strategy_flow_request_payload_schema(
        table_name,
        HashMap::from([(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            table_name.to_string(),
        )]),
    )?);
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.resource_family.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.api_name.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                rows.iter()
                    .map(|row| row.requires_evidence)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| format!("build SearchStrategyFlow ontology registry Arrow batch: {error}"))?;
    let mut writer =
        StreamWriter::try_new(Cursor::new(Vec::new()), schema.as_ref()).map_err(|error| {
            format!("build SearchStrategyFlow ontology registry IPC writer: {error}")
        })?;
    writer.write(&batch).map_err(|error| {
        format!("write SearchStrategyFlow ontology registry IPC batch: {error}")
    })?;
    writer.finish().map_err(|error| {
        format!("finish SearchStrategyFlow ontology registry IPC stream: {error}")
    })?;
    let payload = writer
        .into_inner()
        .map(Cursor::into_inner)
        .map_err(|error| {
            format!("finalize SearchStrategyFlow ontology registry IPC stream: {error}")
        })?;
    validate_search_strategy_flow_request_payload_stream(table_name, &payload)?;
    Ok(payload)
}

fn resource_family_for_kind(kind: Option<&str>) -> &'static str {
    match kind.unwrap_or_default() {
        "task" => "action_type",
        _ => "object_type",
    }
}

fn json_string_list_at(batch: &RecordBatch, column: &str, row_index: usize) -> Vec<String> {
    let Some(value) = string_at(batch, column, row_index) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<String>>(&value).unwrap_or_default()
}

fn string_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<String> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<StringArray>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        None
    } else {
        Some(array.value(row_index).to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OntologyRegistryRow {
    resource_family: String,
    api_name: String,
    requires_evidence: bool,
}

impl OntologyRegistryRow {
    fn new(resource_family: &str, api_name: &str, requires_evidence: bool) -> Self {
        Self {
            resource_family: resource_family.to_owned(),
            api_name: api_name.to_owned(),
            requires_evidence,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/ontology_registry.rs"]
mod tests;
