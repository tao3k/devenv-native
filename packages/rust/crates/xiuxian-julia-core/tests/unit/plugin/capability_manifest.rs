use std::fs;
use std::sync::Arc;

use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use tempfile::tempdir;
use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, RegisteredRepository, RepoIntelligencePlugin, RepositoryPluginConfig,
    RepositoryRefreshPolicy,
};

use super::{
    JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
    build_julia_capability_manifest_flight_transport_client,
    build_julia_plugin_capability_manifest_request_batch,
    decode_julia_plugin_capability_manifest_rows,
    discover_julia_graph_structural_binding_from_manifest_for_repository,
    fetch_julia_plugin_capability_manifest_rows_for_repository,
    graph_structural_binding_from_capability_manifest_rows,
    validate_julia_capability_manifest_preflight_for_repository,
    validate_julia_plugin_capability_manifest_response_batches,
};
use crate::compatibility::link_graph::{JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID, JULIA_PLUGIN_ID};
use crate::plugin::entry::JuliaRepoIntelligencePlugin;
use crate::plugin::graph_structural::GraphStructuralRouteKind;
use crate::plugin::graph_structural_transport::build_graph_structural_flight_transport_client;
use crate::plugin::test_support::wendaosearch_services::{
    LIVE_REQUEST_TIMEOUT_SECS, LIVE_SERVICE_STARTUP_TIMEOUT_SECS, await_live_step,
    local_wendaosearch_package_available, reserve_real_service_port,
    spawn_real_wendaosearch_demo_capability_manifest_service, wait_for_service_ready_with_attempts,
};

fn julia_plugin_capability_manifest_response_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
            DataType::UInt64,
            true,
        ),
        Field::new(
            super::JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
            DataType::Boolean,
            false,
        ),
    ]))
}

fn configured_repository(options: serde_json::Value) -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options,
        }],
        ..RegisteredRepository::default()
    }
}

fn live_capability_manifest_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-julia".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "capability_manifest_transport": {
                    "base_url": base_url,
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft",
                    "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                }
            }),
        }],
    }
}

fn sample_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        julia_plugin_capability_manifest_response_schema(),
        vec![
            Arc::new(StringArray::from(vec![
                Some("xiuxian-julia-core"),
                Some("xiuxian-julia-core"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("rerank"),
                Some("graph-structural"),
            ])),
            Arc::new(StringArray::from(vec![None, Some("structural_rerank")])),
            Arc::new(StringArray::from(vec![
                Some("arrow_flight"),
                Some("arrow_flight"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("http://127.0.0.1:8815"),
                Some("http://127.0.0.1:8816"),
            ])),
            Arc::new(StringArray::from(vec![
                Some("/rerank"),
                Some("/graph/structural/rerank"),
            ])),
            Arc::new(StringArray::from(vec![Some("/healthz"), Some("/ready")])),
            Arc::new(StringArray::from(vec![Some("v1"), Some("v0-draft")])),
            Arc::new(UInt64Array::from(vec![Some(15), None])),
            Arc::new(BooleanArray::from(vec![true, false])),
        ],
    )
    .unwrap_or_else(|error| panic!("sample response batch should build: {error}"))
}

mod local_contract {
    use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};
    use xiuxian_wendao_core::transport::PluginTransportKind;

    use super::{
        JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
        build_julia_capability_manifest_flight_transport_client,
        build_julia_plugin_capability_manifest_request_batch, configured_repository,
        decode_julia_plugin_capability_manifest_rows,
        graph_structural_binding_from_capability_manifest_rows,
        julia_plugin_capability_manifest_response_schema, sample_response_batch,
        validate_julia_plugin_capability_manifest_response_batches,
    };
    use crate::compatibility::link_graph::{JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID, JULIA_PLUGIN_ID};
    use crate::plugin::graph_structural::GraphStructuralRouteKind;
    use crate::{
        JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE, JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
        JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    };

    include!("capability_manifest/local_contract/client.rs");
    include!("capability_manifest/local_contract/request.rs");
    include!("capability_manifest/local_contract/response.rs");
    include!("capability_manifest/local_contract/selection.rs");
}
include!("capability_manifest/live_manifest.rs");
