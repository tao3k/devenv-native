mod api;
#[path = "artifacts.rs"]
mod artifacts;
#[path = "constants.rs"]
mod constants;
#[path = "models/mod.rs"]
pub(crate) mod models;
#[path = "resolve/mod.rs"]
pub mod resolve;
#[path = "settings/mod.rs"]
mod settings;

#[cfg(any(feature = "studio", feature = "zhenfa-router"))]
pub use artifacts::{
    render_link_graph_plugin_artifact_toml_for_selector,
    resolve_link_graph_plugin_artifact_for_selector,
};
pub(crate) use constants::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX;
pub(crate) use models::LinkGraphCacheRuntimeConfig;
pub use models::LinkGraphIndexRuntimeConfig;
pub use resolve::resolve_link_graph_index_runtime;
pub use resolve::{
    resolve_link_graph_agentic_runtime, resolve_link_graph_cache_runtime,
    resolve_link_graph_coactivation_runtime, resolve_link_graph_related_runtime,
};

pub use api::{
    LinkGraphRerankFlightRuntimeSettings, resolve_link_graph_rerank_binding,
    resolve_link_graph_rerank_flight_runtime_settings, resolve_link_graph_rerank_schema_version,
    resolve_link_graph_rerank_score_weights,
};
pub(crate) use resolve::resolve_link_graph_retrieval_policy_runtime;
pub use settings::{
    clear_link_graph_config_home_override, clear_link_graph_wendao_config_override,
    set_link_graph_config_home_override, set_link_graph_wendao_config_override,
};

#[cfg(test)]
#[path = "../../../tests/unit/link_graph/runtime_config/mod.rs"]
mod tests;
