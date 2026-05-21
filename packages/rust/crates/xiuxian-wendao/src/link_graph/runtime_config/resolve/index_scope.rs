//! `link_graph::runtime_config::resolve::index_scope` owns Wendao runtime config resolve index scope behavior.

use crate::link_graph::runtime_config::LinkGraphIndexRuntimeConfig;
use crate::link_graph::runtime_config::settings::merged_wendao_settings;
use std::path::Path;
use xiuxian_wendao_runtime::config::resolve_link_graph_index_runtime_with_settings;

/// Resolve `LinkGraph` index scope from merged `wendao` settings.
///
/// Order:
/// 1) Explicit `link_graph.include_dirs`
/// 2) `link_graph.include_dirs_auto_candidates` when `include_dirs_auto=true`
///    and candidate directory exists under `root_dir`
/// 3) `link_graph.exclude_dirs` (non-hidden additions only)
#[must_use]
pub fn resolve_link_graph_index_runtime(root_dir: &Path) -> LinkGraphIndexRuntimeConfig {
    let settings = merged_wendao_settings();
    resolve_link_graph_index_runtime_with_settings(root_dir, &settings)
}
