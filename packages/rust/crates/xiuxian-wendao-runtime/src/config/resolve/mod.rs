//! Runtime configuration resolver boundary for merged Wendao settings.

mod agentic;
mod cache;
mod coactivation;
mod index_scope;
mod related;

pub use agentic::resolve_link_graph_agentic_runtime_with_settings;
pub use cache::resolve_link_graph_cache_runtime_with_settings;
pub use coactivation::resolve_link_graph_coactivation_runtime_with_settings;
pub use index_scope::resolve_link_graph_index_runtime_with_settings;
pub use related::resolve_link_graph_related_runtime_with_settings;
