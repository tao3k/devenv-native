//! `link_graph::runtime_config::resolve::cache` owns Wendao runtime config resolve cache behavior.

use crate::link_graph::runtime_config::LinkGraphCacheRuntimeConfig;
use crate::link_graph::runtime_config::settings::merged_wendao_settings;
use xiuxian_wendao_runtime::config::resolve_link_graph_cache_runtime_with_settings;

/// Resolve runtime cache configuration from merged settings and environment.
///
/// # Errors
///
/// Returns an error when no Valkey URL can be resolved from config or env.
pub fn resolve_link_graph_cache_runtime() -> Result<LinkGraphCacheRuntimeConfig, String> {
    let settings = merged_wendao_settings();
    resolve_link_graph_cache_runtime_with_settings(&settings)
}
