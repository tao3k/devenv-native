use std::sync::Arc;

use crate::transport::RERANK_ROUTE;
use arrow_array::{Float64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

pub(super) fn response_batch_without_trace_id() -> RecordBatch {
    RecordBatch::try_new(
        xiuxian_wendao_core::repo_intelligence::julia_arrow_response_schema(false),
        vec![
            Arc::new(StringArray::from(vec!["doc-a", "doc-b"])),
            Arc::new(Float64Array::from(vec![0.2, 0.7])),
            Arc::new(Float64Array::from(vec![0.5, 0.9])),
        ],
    )
    .unwrap_or_else(|error| panic!("response batch should build: {error}"))
}

pub(super) fn sample_binding(base_url: Option<&str>) -> PluginCapabilityBinding {
    PluginCapabilityBinding {
        selector: PluginProviderSelector {
            capability_id: CapabilityId("rerank".to_string()),
            provider: PluginId("xiuxian-wendao-julia".to_string()),
        },
        endpoint: PluginTransportEndpoint {
            base_url: base_url.map(ToString::to_string),
            route: Some(RERANK_ROUTE.to_string()),
            health_route: Some("/healthz".to_string()),
            timeout_secs: Some(5),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion("v2".to_string()),
    }
}

pub(super) fn response_batch_with_duplicates() -> RecordBatch {
    RecordBatch::try_new(
        xiuxian_wendao_core::repo_intelligence::julia_arrow_response_schema(false),
        vec![
            Arc::new(StringArray::from(vec!["doc-a", "doc-a"])),
            Arc::new(Float64Array::from(vec![0.2, 0.7])),
            Arc::new(Float64Array::from(vec![0.5, 0.9])),
        ],
    )
    .unwrap_or_else(|error| panic!("duplicate response batch should build: {error}"))
}

pub(super) fn invalid_response_missing_analyzer_score_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("doc_id", DataType::Utf8, false),
            Field::new("final_score", DataType::Float64, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["doc-a"])),
            Arc::new(Float64Array::from(vec![0.5])),
        ],
    )
    .unwrap_or_else(|error| panic!("invalid response batch should build: {error}"))
}

pub(super) fn tempdir_or_panic() -> tempfile::TempDir {
    tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"))
}

pub(super) fn err_or_panic<T, E>(result: Result<T, E>, failure_message: &str) -> E {
    match result {
        Ok(_) => panic!("{failure_message}"),
        Err(error) => error,
    }
}
