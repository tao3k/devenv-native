//! Semantic-scope to `SearchStrategyFlow` ontology-registry bridge.

use std::collections::BTreeSet;

use arrow::array::{Array, StringArray};
use arrow::record_batch::RecordBatch;

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::metadata::{ANALYSIS_SEMANTIC_SCOPE_ROUTE, populate_semantic_scope_headers};

/// Fetch accepted semantic-scope rows and serialize them as ontology registry
/// TSV rows for the local `WendaoGraph.jl` `SearchStrategyFlow` host.
///
/// # Errors
///
/// Returns an error when the Flight endpoint cannot be reached or when the
/// semantic-scope payload cannot be decoded.
pub(crate) async fn search_strategy_flow_ontology_registry_tsv_from_semantic_scope(
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<String, String> {
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let batches = client
        .collect_route_batches_allow_empty(
            ANALYSIS_SEMANTIC_SCOPE_ROUTE,
            "SearchStrategyFlow semantic-scope ontology registry",
            |metadata| populate_semantic_scope_headers(metadata, &[]),
        )
        .await?;
    Ok(semantic_scope_batches_to_ontology_registry_tsv(&batches))
}

pub(super) fn semantic_scope_batches_to_ontology_registry_tsv(batches: &[RecordBatch]) -> String {
    let mut rows = BTreeSet::<OntologyRegistryTsvRow>::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(object_id) = string_at(batch, "objectId", row_index) else {
                continue;
            };
            if object_id.trim().is_empty() {
                continue;
            }
            let kind = string_at(batch, "kind", row_index);
            rows.insert(OntologyRegistryTsvRow::new(
                resource_family_for_kind(kind.as_deref()),
                object_id.as_str(),
                false,
            ));
            if let Some(title) = string_at(batch, "title", row_index)
                && !title.trim().is_empty()
                && title != object_id
            {
                rows.insert(OntologyRegistryTsvRow::new(
                    "object_type",
                    title.as_str(),
                    false,
                ));
            }
            for target in json_string_list_at(batch, "relationTargetsJson", row_index) {
                rows.insert(OntologyRegistryTsvRow::new(
                    "link_type",
                    format!("{object_id}.{target}").as_str(),
                    false,
                ));
                rows.insert(OntologyRegistryTsvRow::new(
                    "link_type",
                    target.as_str(),
                    false,
                ));
            }
            for validation in json_string_list_at(batch, "requiredValidationsJson", row_index) {
                rows.insert(OntologyRegistryTsvRow::new(
                    "action_type",
                    validation.as_str(),
                    true,
                ));
            }
        }
    }
    rows.into_iter()
        .map(|row| row.to_tsv())
        .collect::<Vec<_>>()
        .join("\n")
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
struct OntologyRegistryTsvRow {
    resource_family: String,
    api_name: String,
    requires_evidence: bool,
}

impl OntologyRegistryTsvRow {
    fn new(resource_family: &str, api_name: &str, requires_evidence: bool) -> Self {
        Self {
            resource_family: resource_family.to_owned(),
            api_name: api_name.to_owned(),
            requires_evidence,
        }
    }

    fn to_tsv(&self) -> String {
        [
            escape_tsv_field(self.resource_family.as_str()),
            escape_tsv_field(self.api_name.as_str()),
            self.requires_evidence.to_string(),
        ]
        .join("\t")
    }
}

fn escape_tsv_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/ontology_registry.rs"]
mod tests;
