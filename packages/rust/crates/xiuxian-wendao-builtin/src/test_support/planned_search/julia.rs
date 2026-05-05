//! Planned-search Julia runtime configuration fixtures for builtin tests.

use xiuxian_wendao_julia::integration_support::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};

/// Render the linked builtin OpenAI-compatible planned-search runtime-config
/// fixture.
#[must_use]
pub fn linked_builtin_julia_planned_search_openai_runtime_config_toml(
    vector_store_path: &str,
    embedding_base_url: &str,
    rerank_base_url: &str,
) -> String {
    julia_planned_search_openai_runtime_config_toml(
        vector_store_path,
        embedding_base_url,
        rerank_base_url,
    )
}

/// Render the linked builtin vector-store planned-search runtime-config
/// fixture.
#[must_use]
pub fn linked_builtin_julia_planned_search_vector_store_runtime_config_toml(
    vector_store_path: &str,
    rerank_base_url: &str,
) -> String {
    julia_planned_search_vector_store_runtime_config_toml(vector_store_path, rerank_base_url)
}
