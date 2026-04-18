use super::common::JuliaExampleServiceGuard;
use super::official_examples::spawn_wendaoanalyzer_service_from_artifact;
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE,
    LinkGraphJuliaRerankRuntimeConfig,
};

const JULIA_PLANNED_SEARCH_SCHEMA_VERSION: &str = "v1";
const JULIA_PLANNED_SEARCH_TIMEOUT_SECS: u64 = 30;
const JULIA_PLANNED_SEARCH_EMBEDDING_MODEL: &str = "glm-5";

/// Render the runtime-config TOML fixture used by the custom planned-search
/// Julia rerank integration test backed by an OpenAI-compatible embedding
/// service.
#[must_use]
pub fn julia_planned_search_openai_runtime_config_toml(
    vector_store_path: &str,
    embedding_base_url: &str,
    rerank_base_url: &str,
) -> String {
    format!(
        r#"[link_graph.retrieval]
mode = "hybrid"
candidate_multiplier = 2
max_sources = 2
graph_rows_per_source = 2

[link_graph.retrieval.semantic_ignition]
backend = "openai-compatible"
vector_store_path = "{vector_store_path}"
table_name = "wendao_semantic_docs"
embedding_base_url = "{embedding_base_url}"
embedding_model = "{JULIA_PLANNED_SEARCH_EMBEDDING_MODEL}"

[link_graph.retrieval.julia_rerank]
base_url = "{rerank_base_url}"
route = "{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}"
schema_version = "{JULIA_PLANNED_SEARCH_SCHEMA_VERSION}"
timeout_secs = {JULIA_PLANNED_SEARCH_TIMEOUT_SECS}
"#
    )
}

/// Render the runtime-config TOML fixture used by the custom planned-search
/// Julia rerank integration test backed by the vector-store semantic ignition
/// path.
#[must_use]
pub fn julia_planned_search_vector_store_runtime_config_toml(
    vector_store_path: &str,
    rerank_base_url: &str,
) -> String {
    format!(
        r#"[link_graph.retrieval]
mode = "hybrid"
candidate_multiplier = 2
max_sources = 2
graph_rows_per_source = 2

[link_graph.retrieval.semantic_ignition]
backend = "vector-store"
vector_store_path = "{vector_store_path}"
table_name = "wendao_semantic_docs"

[link_graph.retrieval.julia_rerank]
base_url = "{rerank_base_url}"
route = "{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}"
schema_version = "{JULIA_PLANNED_SEARCH_SCHEMA_VERSION}"
timeout_secs = {JULIA_PLANNED_SEARCH_TIMEOUT_SECS}
"#
    )
}

/// Render the runtime-config TOML fixture used by the analyzer-backed
/// similarity-only planned-search integration test.
#[must_use]
pub fn julia_planned_search_similarity_only_runtime_config_toml(
    vector_store_path: &str,
    rerank_base_url: &str,
) -> String {
    format!(
        r#"[link_graph.retrieval]
mode = "hybrid"
candidate_multiplier = 2
max_sources = 2
graph_rows_per_source = 2

[link_graph.retrieval.semantic_ignition]
backend = "vector-store"
vector_store_path = "{vector_store_path}"
table_name = "wendao_semantic_docs"

[link_graph.retrieval.julia_rerank]
base_url = "{rerank_base_url}"
route = "{DEFAULT_JULIA_RERANK_FLIGHT_ROUTE}"
schema_version = "{JULIA_PLANNED_SEARCH_SCHEMA_VERSION}"
timeout_secs = {JULIA_PLANNED_SEARCH_TIMEOUT_SECS}
service_mode = "stream"
analyzer_config_path = "{DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH}"
analyzer_strategy = "similarity_only"
"#
    )
}

/// Spawn the analyzer-backed similarity-only example service used by the
/// planned-search integration test.
pub async fn spawn_wendaoanalyzer_similarity_only_service() -> (String, JuliaExampleServiceGuard) {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        service_mode: Some("stream".to_string()),
        analyzer_config_path: Some(DEFAULT_JULIA_ANALYZER_EXAMPLE_CONFIG_PATH.to_string()),
        analyzer_strategy: Some("similarity_only".to_string()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    spawn_wendaoanalyzer_service_from_artifact(&runtime.deployment_artifact()).await
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/planned_search.rs"]
mod tests;
