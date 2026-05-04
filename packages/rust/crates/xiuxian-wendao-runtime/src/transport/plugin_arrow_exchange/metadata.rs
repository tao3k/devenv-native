//! Metadata helpers for Plugin Arrow exchange requests.

use arrow_array::RecordBatch;
use xiuxian_db_store::attach_record_batch_metadata;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use crate::transport::{FLIGHT_SCHEMA_VERSION_METADATA_KEY, FLIGHT_TRACE_ID_METADATA_KEY};

use super::errors::contract_request_error;
use super::{PluginArrowProviderIdRef, PluginArrowTraceIdRef};

/// Attach plugin rerank request metadata to one `WendaoArrow` request batch.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the metadata cannot be attached to
/// the supplied batch.
pub fn attach_plugin_arrow_request_metadata(
    batch: &RecordBatch,
    trace_id: PluginArrowTraceIdRef<'_>,
    schema_version: &str,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [
            (FLIGHT_TRACE_ID_METADATA_KEY, trace_id.to_string()),
            (
                FLIGHT_SCHEMA_VERSION_METADATA_KEY,
                schema_version.to_string(),
            ),
        ],
    )
    .map_err(|error| contract_request_error(format!("failed to attach request metadata: {error}")))
}

/// Build one stable provider-aware plugin rerank request trace id from the
/// provider id and operator query text.
#[must_use]
pub fn plugin_arrow_request_trace_id(
    provider_id: PluginArrowProviderIdRef<'_>,
    query_text: &str,
) -> String {
    let provider_id = provider_id.trim();
    let provider_id = if provider_id.is_empty() {
        "plugin"
    } else {
        provider_id
    };
    let normalized = query_text
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if normalized.is_empty() {
        format!("plugin-rerank:{provider_id}:query")
    } else {
        format!("plugin-rerank:{provider_id}:{normalized}")
    }
}
